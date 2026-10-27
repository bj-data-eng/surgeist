//! Ordered inline input occurrences at the canonical relation/field barriers.
use super::*;
use crate::{DomInlineStyleBatchContext, DomInlineStyleCommit, DomInlineStyleProfile};
use std::sync::atomic::AtomicU64;

pub(in crate::store) type Resolver<'a> = dyn FnMut(
        &[CssomDeclarationCandidate],
        &CssomContext,
    ) -> Result<CssomAdmission, surgeist_cssom::CssomError>
    + 'a;

pub(super) struct InputIssuer {
    next: AtomicU64,
    _debit: custody::Debit,
}
impl InputIssuer {
    pub(super) fn issue(&self, dom_revision: u64) -> Result<u64, DomError> {
        let minimum = dom_revision
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let mut old = self.next.load(Ordering::Relaxed);
        loop {
            let value = old.max(minimum);
            let next = value
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            match self
                .next
                .compare_exchange_weak(old, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return Ok(value),
                Err(actual) => old = actual,
            }
        }
    }
}
pub(in crate::store) struct Profiles {
    context: DomInlineStyleBatchContext,
    issuer: Arc<InputIssuer>,
    revision: u64,
    limits: DomInlineStyleLimits,
    spent: AtomicU64,
    _debit: custody::Debit,
}

#[derive(Clone)]
struct Facts {
    document: Key,
    root: Key,
    host: Option<Key>,
    class: crate::DomDocumentClass,
    mode: CssParserMode,
    encoding: DocumentString,
    url: DocumentString,
    base: DocumentString,
}
#[derive(Clone, Copy)]
struct FactsRead<'a> {
    document: Key,
    root: Key,
    host: Option<Key>,
    class: crate::DomDocumentClass,
    mode: CssParserMode,
    encoding: &'a DocumentString,
    url: &'a DocumentString,
    base: &'a DocumentString,
}
impl Facts {
    fn read(&self) -> FactsRead<'_> {
        FactsRead {
            document: self.document,
            root: self.root,
            host: self.host,
            class: self.class,
            mode: self.mode,
            encoding: &self.encoding,
            url: &self.url,
            base: &self.base,
        }
    }
}
impl FactsRead<'_> {
    fn owned(self) -> Facts {
        Facts {
            document: self.document,
            root: self.root,
            host: self.host,
            class: self.class,
            mode: self.mode,
            encoding: self.encoding.clone(),
            url: self.url.clone(),
            base: self.base.clone(),
        }
    }
    fn usage(self) -> Result<custody::Usage, DomError> {
        let mut payload = source::storage::Payload::default();
        for text in [self.encoding, self.url, self.base] {
            payload.string(text)?;
        }
        Ok(custody::Usage::new(payload.bytes, payload.units))
    }
    fn same(
        self,
        other: Self,
        p: &Processing<'_>,
        normalize: impl Fn(Key) -> Key,
    ) -> Result<bool, DomError> {
        Ok(normalize(self.document) == normalize(other.document)
            && normalize(self.root) == normalize(other.root)
            && self.host.map(&normalize) == other.host.map(&normalize)
            && self.class == other.class
            && self.mode == other.mode
            && same_text(self.encoding, other.encoding, p)?
            && same_text(self.url, other.url, p)?
            && same_text(self.base, other.base, p)?)
    }
}
fn staged_base<'a>(
    view: OrderView<'a>,
    document: Key,
    p: &Processing<'_>,
) -> Result<&'a DocumentString, DomError> {
    p.step()?;
    let node = view.node(document)?;
    let transaction::KindRead::Document(metadata) = node.kind() else {
        return Err(DomError::WrongDocument);
    };
    let first = match node.base_document().ok_or(DomError::WrongDocument)?.first {
        None => surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
        Some(key) => {
            p.step()?;
            surgeist_html_semantics::HtmlDocumentFirstBase::Present(
                view.node(key)?
                    .base()
                    .and_then(|b| b.frozen)
                    .map(|b| b.value),
            )
        }
    };
    if let Some(inputs) = metadata.hosted_inputs.as_ref() {
        return surgeist_html_semantics::select_document_base(Some(&inputs.0.facts), first)
            .map(surgeist_html_semantics::HtmlBaseUrlRecord::serialization)
            .map_err(|m| DomError::BaseFacts(crate::DomBaseFactsError::Missing(m)));
    }
    if metadata.document_environment().has_ever_had_navigable()
        || !ascii(metadata.url(), "about:blank", p)?
    {
        return Err(DomError::NotSupported);
    }
    surgeist_html_semantics::select_document_base_serialization(metadata.url(), first)
        .map_err(|m| DomError::BaseFacts(crate::DomBaseFactsError::Missing(m)))
}
fn facts<'a>(view: OrderView<'a>, key: Key, p: &Processing<'_>) -> Result<FactsRead<'a>, DomError> {
    p.step()?;
    let document = view.node(key)?.document();
    let transaction::KindRead::Document(metadata) = view.node(document)?.kind() else {
        return Err(DomError::WrongDocument);
    };
    let root = view.root(key, Mode::Ordinary, &mut || p.step())?;
    let root_node = view.node(root)?;
    let host = matches!(root_node.kind(), transaction::KindRead::ShadowRoot(_))
        .then(|| root_node.host())
        .flatten();
    Ok(FactsRead {
        document,
        root,
        host,
        class: metadata.class(),
        mode: if metadata.mode() == crate::HtmlDocumentMode::Quirks {
            CssParserMode::Quirks
        } else {
            CssParserMode::Standards
        },
        encoding: metadata.encoding(),
        url: metadata.url(),
        base: staged_base(view, document, p)?,
    })
}

enum Input {
    Context,
    Attribute(Option<DocumentString>),
}
struct LiveInput {
    element: NodeId,
    facts: Facts,
    profile: usize,
    serial: u64,
    input: Input,
}
enum Event {
    Input(LiveInput),
    Retirement {
        element: NodeId,
        owner_revision: Option<u64>,
    },
}
impl Event {
    fn element(&self) -> NodeId {
        match self {
            Self::Input(input) => input.element,
            Self::Retirement { element, .. } => *element,
        }
    }
    fn live(&self) -> &LiveInput {
        match self {
            Self::Input(input) => input,
            Self::Retirement { .. } => unreachable!("live input frontier"),
        }
    }
}
#[derive(Clone, Default)]
struct Log {
    retirements: bank::Bank<Event>,
    retirements_custody: bank::BankCustody<Event>,
    entries: bank::Bank<Event>,
    latest: bank::Bank<Event>,
    attributes: bank::Bank<Event>,
    entries_custody: bank::BankCustody<Event>,
    latest_custody: bank::BankCustody<Event>,
    attributes_custody: bank::BankCustody<Event>,
    len: usize,
}
#[derive(Clone)]
enum Authority {
    Inputs(Arc<Profiles>),
    Blocked {
        target: NodeId,
        policy: Arc<custody::Held<DomInlineStyleLimits>>,
    },
}
#[derive(Clone)]
pub(in crate::store) struct Recording {
    authority: Authority,
    index: Index,
    log: Log,
}
impl Index {
    fn relocate_staged(
        &mut self,
        element: NodeId,
        view: OrderView<'_>,
        add: bool,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let key = Key::Existing(element);
        let root = view.root(key, Mode::Ordinary, &mut || p.step())?;
        let document = view.node(key)?.document();
        let membership = if add {
            MemberChange::Insert(key)
        } else {
            MemberChange::Remove(key)
        };
        self.change_group(root, membership, false, view, registry, p)?;
        self.change_group(document, membership, true, view, registry, p)
    }
    fn select_staged(
        &self,
        subtree: Key,
        view: OrderView<'_>,
        selected: &mut Selection,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let subtree = view.normalize(subtree);
        let root = view.root(subtree, Mode::Ordinary, &mut || p.step())?;
        let Some(group) = self.roots.get(root) else {
            return Ok(());
        };
        let mut next =
            self.members
                .lower_bound(group.members, subtree, view, Mode::Ordinary, &mut || {
                    p.step()
                })?;
        while let Some(key) = next {
            if !view.descendant(subtree, key, Mode::Ordinary, &mut || p.step())? {
                break;
            }
            selected.insert(key, registry, p)?;
            next = self
                .members
                .after(group.members, key, view, Mode::Ordinary, &mut || p.step())?;
        }
        Ok(())
    }
    fn document_staged(
        &self,
        document: Key,
        selected: &mut Selection,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let Some(group) = self.documents.get(document) else {
            return Ok(());
        };
        let mut next = self.members.first(group.members, &mut || p.step())?;
        while let Some(key) = next {
            selected.insert(key, registry, p)?;
            next = self
                .members
                .after_identity(group.members, key, &mut || p.step())?;
        }
        Ok(())
    }
}
impl Recording {
    pub(in crate::store) fn profiles(&self) -> Arc<Profiles> {
        self.inputs().clone()
    }
    fn inputs(&self) -> &Arc<Profiles> {
        match &self.authority {
            Authority::Inputs(profiles) => profiles,
            Authority::Blocked { .. } => {
                unreachable!("blocked authority cannot ingest producer inputs")
            }
        }
    }
    pub(in crate::store) fn require_unblocked(&self) -> Result<(), DomError> {
        if let Authority::Blocked { target, .. } = &self.authority {
            Err(DomError::InlineStyleOperationPending { element: *target })
        } else {
            Ok(())
        }
    }
    pub(in crate::store) fn is_blocked(&self) -> bool {
        matches!(self.authority, Authority::Blocked { .. })
    }
    pub(super) fn blocked(
        index: &super::Index,
        target: NodeId,
        policy: Arc<custody::Held<DomInlineStyleLimits>>,
    ) -> Self {
        Self {
            authority: Authority::Blocked { target, policy },
            index: index.clone(),
            log: Log::default(),
        }
    }
    pub(in crate::store) fn from_profiles(profiles: Arc<Profiles>, index: &super::Index) -> Self {
        Self {
            authority: Authority::Inputs(profiles),
            index: index.clone(),
            log: Log::default(),
        }
    }
    pub(in crate::store) fn has_inputs(&self) -> bool {
        self.log.len != 0
    }
    pub(in crate::store) fn require_unreached(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if !self.has_inputs() {
            return Ok(());
        }
        if let Some(event) = self.log.entries.get_with(0, &mut step)? {
            return Err(required(
                event.element(),
                match event {
                    Event::Input(input) => match input.input {
                        Input::Context => DomInlineStyleChange::Context,
                        Input::Attribute(_) => DomInlineStyleChange::Attribute,
                    },
                    Event::Retirement { .. } => DomInlineStyleChange::Retirement,
                },
            ));
        }
        Ok(())
    }
    pub(in crate::store) fn index(&self) -> &super::Index {
        &self.index
    }
    pub(in crate::store) fn restarted(&self, index: &super::Index) -> Self {
        Self {
            authority: self.authority.clone(),
            index: index.clone(),
            log: Log::default(),
        }
    }
    pub(super) fn processing<'a>(
        &'a self,
        formation: Option<&'a Formation>,
        cancel: &'a AtomicBool,
    ) -> Processing<'a> {
        Processing {
            limits: match &self.authority {
                Authority::Inputs(p) => &p.limits,
                Authority::Blocked { policy, .. } => policy.value(),
            },
            formation,
            cancel,
            spent: Cell::new(0),
            shared: match &self.authority {
                Authority::Inputs(p) => Some(&p.spent),
                Authority::Blocked { .. } => None,
            },
        }
    }

    fn profile(
        &self,
        draft: &Draft,
        element: NodeId,
        document: Key,
        p: &Processing<'_>,
    ) -> Result<usize, DomError> {
        let operand = |operand: &NodeOperand| match operand {
            NodeOperand::Existing(id) => Some(draft.live_key(Key::Existing(*id))),
            NodeOperand::Created(ticket) => draft
                .tickets
                .get(ticket.index)
                .map(|key| draft.live_key(*key)),
        };
        let mut found = None;
        for (index, row) in self.inputs().context.profiles.iter().enumerate() {
            p.step()?;
            if operand(&row.element) == Some(Key::Existing(element))
                && operand(&row.document) == Some(document)
                && found.replace(index).is_some()
            {
                return Err(DomError::InlineStyleProfileConflict { element });
            }
        }
        found.ok_or(DomError::InlineStyleProfileMissing { element })
    }
    fn append(
        &mut self,
        draft: &Draft,
        element: NodeId,
        next: FactsRead<'_>,
        input: Option<Option<&DocumentString>>,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        if input.is_none() {
            let before = match self.log.latest.get_with(element.slot, &mut || p.step())? {
                Some(event) => event.live().facts.read(),
                None => facts(
                    OrderView::state(&draft.base, draft.store),
                    Key::Existing(element),
                    p,
                )?,
            };
            if before.same(next, p, |key| draft.live_key(key))? {
                return Ok(());
            }
        }
        self.require_unblocked()?;
        let profile = self.profile(draft, element, next.document, p)?;
        let mut usage = next
            .usage()?
            .checked_add(custody::Usage::new(size_of::<Event>(), 0))?;
        if let Some(Some(value)) = input {
            let mut payload = source::storage::Payload::default();
            payload.string(value)?;
            usage = usage.checked_add(custody::Usage::new(payload.bytes, payload.units))?;
        }
        let debit = draft.custody_ledger().retain(usage, |u| p.admit(u))?;
        let event = Event::Input(LiveInput {
            element,
            facts: next.owned(),
            profile,
            serial: self.inputs().issuer.issue(self.inputs().revision)?,
            input: match input {
                None => Input::Context,
                Some(value) => Input::Attribute(value.cloned()),
            },
        });
        let controlled = record(
            custody::Held::new(event, debit),
            draft.custody_registry(),
            p,
        )?;
        let totals = bank::Totals {
            bytes: usage.bytes(),
            units: usage.units(),
            occurrences: 1,
        };
        let entries = self.log.entries.set_controlled(
            &self.log.entries_custody,
            self.log.len,
            controlled.clone(),
            false,
            totals,
            draft.custody_registry(),
            |u| p.admit(u),
        )?;
        let latest = self.log.latest.set_controlled(
            &self.log.latest_custody,
            element.slot,
            controlled.clone(),
            false,
            totals,
            draft.custody_registry(),
            |u| p.admit(u),
        )?;
        let attributes = if input.is_some() {
            Some(self.log.attributes.set_controlled(
                &self.log.attributes_custody,
                element.slot,
                controlled,
                false,
                totals,
                draft.custody_registry(),
                |u| p.admit(u),
            )?)
        } else {
            None
        };
        let len = self
            .log
            .len
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        self.log.entries = entries.0;
        self.log.entries_custody = entries.1;
        self.log.latest = latest.0;
        self.log.latest_custody = latest.1;
        if let Some((values, custody)) = attributes {
            self.log.attributes = values;
            self.log.attributes_custody = custody;
        }
        self.log.len = len;
        Ok(())
    }
    fn append_retirement(
        &mut self,
        draft: &Draft,
        element: NodeId,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        self.require_unblocked()?;
        let owner_revision = self
            .log
            .latest
            .get_with(element.slot, &mut || p.step())?
            .map(|event| event.live().serial);
        self.index.relocate_staged(
            element,
            draft.order_view(),
            false,
            draft.custody_registry(),
            p,
        )?;
        self.index
            .remove_binding(element, draft.custody_registry(), p)?;
        let usage = custody::Usage::new(size_of::<Event>(), 0);
        let debit = draft.custody_ledger().retain(usage, |u| p.admit(u))?;
        let controlled = record(
            custody::Held::new(
                Event::Retirement {
                    element,
                    owner_revision,
                },
                debit,
            ),
            draft.custody_registry(),
            p,
        )?;
        let totals = bank::Totals {
            bytes: usage.bytes(),
            units: 0,
            occurrences: 1,
        };
        let entries = self.log.entries.set_controlled(
            &self.log.entries_custody,
            self.log.len,
            controlled.clone(),
            false,
            totals,
            draft.custody_registry(),
            |u| p.admit(u),
        )?;
        let latest = self.log.latest.set_controlled(
            &self.log.latest_custody,
            element.slot,
            controlled.clone(),
            false,
            totals,
            draft.custody_registry(),
            |u| p.admit(u),
        )?;
        let retirements = self.log.retirements.set_controlled(
            &self.log.retirements_custody,
            self.retirement_count(),
            controlled,
            false,
            totals,
            draft.custody_registry(),
            |u| p.admit(u),
        )?;
        self.log.len = self
            .log
            .len
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        (self.log.entries, self.log.entries_custody) = entries;
        (self.log.latest, self.log.latest_custody) = latest;
        (self.log.retirements, self.log.retirements_custody) = retirements;
        Ok(())
    }
    pub(super) fn retirement_count(&self) -> usize {
        self.log.retirements.record_count()
    }
    pub(super) fn retirement_at(
        &self,
        index: usize,
        p: &Processing<'_>,
    ) -> Result<NodeId, DomError> {
        Ok(self
            .log
            .retirements
            .get_with(index, &mut || p.step())?
            .expect("dense actual retirement log")
            .element())
    }
    pub(super) fn retired(&self, element: NodeId, p: &Processing<'_>) -> Result<bool, DomError> {
        Ok(
            matches!(self.log.latest.get_with(element.slot, &mut || p.step())?,
            Some(Event::Retirement { element: retired, .. }) if *retired == element),
        )
    }
    fn contexts(
        &mut self,
        draft: &Draft,
        selected: &Selection,
        before: OrderView<'_>,
        after: OrderView<'_>,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        for key in selected.values.values() {
            let Key::Existing(id) = key else {
                unreachable!("live bindings have canonical identity")
            };
            self.index
                .relocate_staged(*id, before, false, draft.custody_registry(), p)?;
        }
        for key in selected.values.values() {
            let Key::Existing(id) = key else {
                unreachable!("live bindings have canonical identity")
            };
            self.index
                .relocate_staged(*id, after, true, draft.custody_registry(), p)?;
            self.append(draft, *id, facts(after, *key, p)?, None, p)?;
        }
        Ok(())
    }
    pub(super) fn final_context(
        &self,
        element: NodeId,
        view: OrderView<'_>,
        ids: &source::AllocationIds,
        p: &Processing<'_>,
    ) -> Result<bool, DomError> {
        let Some(event) = self.log.latest.get_with(element.slot, &mut || p.step())? else {
            return Ok(false);
        };
        event.live().facts.read().same(
            facts(view, Key::Existing(element), p)?,
            p,
            |key| match key {
                Key::Existing(_) => key,
                Key::Created(index) => ids.get(index).map_or(key, Key::Existing),
            },
        )
    }
    pub(super) fn final_attribute(
        &self,
        element: NodeId,
        value: Option<&DocumentString>,
        p: &Processing<'_>,
    ) -> Result<bool, DomError> {
        let Some(event) = self
            .log
            .attributes
            .get_with(element.slot, &mut || p.step())?
        else {
            return Ok(false);
        };
        let Input::Attribute(old) = &event.live().input else {
            unreachable!("attribute frontier")
        };
        same_optional_text(old.as_ref(), value, p)
    }
}

impl Draft {
    pub(in crate::store) fn inline_retirement(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Key::Existing(element) = self.live_key(key) else {
            return Ok(());
        };
        let Some(membership) = &self.inline_membership else {
            return Ok(());
        };
        if !membership
            .bindings
            .get_with(element.slot, &mut || self.source_step(cancel))?
            .is_some_and(|row| row.element == element)
        {
            return Ok(());
        }
        let Some(recording) = &self.inline_recording else {
            return Err(required(element, DomInlineStyleChange::Retirement));
        };
        let undo = self.prepare_undo_slots(1, cancel)?;
        let formation = Formation::new(self, cancel)?;
        let p = recording.processing(formation.as_ref(), cancel);
        let mut next = recording.clone();
        next.append_retirement(self, element, &p)?;
        drop(formation);
        self.commit_inline_recording(Some(next), undo);
        Ok(())
    }
    pub(in crate::store) fn prepare_inline_relation(
        &self,
        change: transaction::RelationChange<'_>,
        after: OrderView<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Recording>, DomError> {
        let Some(recording) = &self.inline_recording else {
            return Ok(None);
        };
        if !matches!(change.field, transaction::RelationField::Children) {
            return Ok(None);
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        let p = recording.processing(formation.as_ref(), cancel);
        let before = self.order_view();
        let mut selected = Selection::default();
        for child in (change.index..change.index + change.removed)
            .filter_map(|i| change.before.get(i))
            .chain(change.added.iter().copied())
        {
            recording.index.select_staged(
                self.live_key(child),
                before,
                &mut selected,
                self.custody_registry(),
                &p,
            )?;
        }
        if selected.values.values().next().is_none() {
            return Ok(None);
        }
        let mut next = recording.clone();
        next.contexts(self, &selected, before, after, &p)?;
        Ok(Some(next))
    }
    pub(in crate::store) fn prepare_inline_fields(
        &self,
        key: Key,
        next: transaction::NodeRead<'_>,
        after: OrderView<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Recording>, DomError> {
        let Some(recording) = &self.inline_recording else {
            return Ok(None);
        };
        let old = self.node(key)?;
        let document_changed = old.document() != next.document();
        let host_changed = old.host() != next.host();
        let metadata_changed = matches!((old.kind(),next.kind()),(transaction::KindRead::Document(a),transaction::KindRead::Document(b)) if !std::ptr::eq(a,b));
        let base_document_changed =
            old.base_document()
                .zip(next.base_document())
                .is_some_and(|(a, b)| {
                    a.first != b.first
                        || a.facts.map(|v| v as *const _) != b.facts.map(|v| v as *const _)
                });
        let base_changed = old.base().zip(next.base()).is_some_and(|(a, b)| {
            a.frozen.map(|v| v.value as *const _) != b.frozen.map(|v| v.value as *const _)
        });
        if !(document_changed
            || host_changed
            || metadata_changed
            || base_document_changed
            || base_changed)
        {
            return Ok(None);
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        let p = recording.processing(formation.as_ref(), cancel);
        let before = self.order_view();
        let mut selected = Selection::default();
        let key = self.live_key(key);
        if document_changed
            && let Key::Existing(id) = key
            && recording.index.membership.contains(id)
        {
            selected.insert(key, self.custody_registry(), &p)?;
        }
        if host_changed {
            recording.index.select_staged(
                key,
                before,
                &mut selected,
                self.custody_registry(),
                &p,
            )?;
        }
        if metadata_changed || base_document_changed {
            recording
                .index
                .document_staged(key, &mut selected, self.custody_registry(), &p)?;
        }
        if base_changed {
            recording.index.document_staged(
                old.document(),
                &mut selected,
                self.custody_registry(),
                &p,
            )?;
        }
        if selected.values.values().next().is_none() {
            return Ok(None);
        }
        let mut current = recording.clone();
        current.contexts(self, &selected, before, after, &p)?;
        Ok(Some(current))
    }
    pub(in crate::store) fn commit_inline_recording(
        &mut self,
        next: Option<Recording>,
        undo: bool,
    ) {
        if let Some(next) = next {
            let old = self
                .inline_recording
                .replace(next)
                .expect("prepared inline recording");
            if undo {
                self.journal.push_prepared(transaction::Undo::Inline(old));
            }
        }
    }
    pub(in crate::store) fn inline_attribute(
        &mut self,
        key: Key,
        name: &crate::DomQualifiedName,
        value: Option<&DocumentString>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(membership) = &self.inline_membership else {
            return Ok(());
        };
        if name.namespace_uri().is_some() || name.local_name().len() != 5 {
            return Ok(());
        }
        for (unit, byte) in name.local_name().code_units().iter().zip(b"style") {
            self.source_step(cancel)?;
            if *unit != u16::from(*byte) {
                return Ok(());
            }
        }
        let Key::Existing(id) = self.live_key(key) else {
            return Ok(());
        };
        if let Some(echo) = &self.inline_echo
            && echo.element == id
            && let Some(value) = value
        {
            let mut same = value.len() == echo.value.len();
            if same {
                for (a, b) in value.code_units().iter().zip(echo.value.code_units()) {
                    self.source_step(cancel)?;
                    if a != b {
                        same = false;
                        break;
                    }
                }
            }
            if same {
                return Ok(());
            }
        }
        if !membership
            .bindings
            .get_with(id.slot, &mut || self.source_step(cancel))?
            .is_some_and(|b| b.element == id)
        {
            return Ok(());
        }
        let Some(recording) = &self.inline_recording else {
            return Err(required(id, DomInlineStyleChange::Attribute));
        };
        let undo = self.prepare_undo_slots(1, cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let p = recording.processing(formation.as_ref(), cancel);
        let mut next = recording.clone();
        next.append(
            self,
            id,
            facts(self.order_view(), Key::Existing(id), &p)?,
            Some(value),
            &p,
        )?;
        drop(formation);
        self.commit_inline_recording(Some(next), undo);
        Ok(())
    }
}

// Rebuild opaque producer memberships from bounded, measured backing before
// retaining them. A caller's spare Vec capacity never enters the recording.
fn copy_rows<T: Copy>(
    rows: &[T],
    debit: &mut custody::Debit,
    p: &Processing<'_>,
) -> Result<Vec<T>, DomError> {
    let bytes = rows
        .len()
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    let target = debit.usage().checked_add(custody::Usage::new(bytes, 0))?;
    transaction::nodes::resize_debit(debit, target, |u| p.admit(u))?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(rows.len())
        .map_err(|_| DomError::AllocationFailed)?;
    let extra = values
        .capacity()
        .checked_sub(rows.len())
        .and_then(|n| n.checked_mul(size_of::<T>()))
        .ok_or(DomError::RepresentationExhausted)?;
    let target = debit.usage().checked_add(custody::Usage::new(extra, 0))?;
    transaction::nodes::resize_debit(debit, target, |u| p.admit(u))?;
    // Producer constructors validate duplicate-free membership with bounded
    // prefix comparisons. Pay that finite scan before invoking them.
    let comparisons = rows
        .len()
        .checked_mul(rows.len())
        .ok_or(DomError::RepresentationExhausted)?;
    for _ in 0..comparisons {
        p.step()?;
    }
    values.extend_from_slice(rows);
    Ok(values)
}
impl Dom {
    pub(in crate::store) fn begin_inline_recording(
        &mut self,
        context: DomInlineStyleBatchContext,
        cancel: &AtomicBool,
    ) -> Result<Option<Recording>, DomInlineStyleError> {
        if context.basis.owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        if context.basis != self.snapshot().stamp() {
            return Err(DomInlineStyleError::StaleContext);
        }
        if self
            .inline_styles
            .as_ref()
            .is_none_or(|owner| owner.index.membership.bindings.record_count() == 0)
        {
            return Ok(None);
        }
        let formation = self.source_observation_formation(cancel)?;
        self.form_inline_recording(context, formation.as_ref(), false, cancel)
    }
    pub(in crate::store) fn form_inline_recording(
        &mut self,
        context: DomInlineStyleBatchContext,
        formation: Option<&Formation>,
        required: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Recording>, DomInlineStyleError> {
        let Some(owner) = self.inline_styles.as_ref() else {
            return if required {
                Err(DomInlineStyleError::Unconfigured)
            } else {
                Ok(None)
            };
        };
        if !required && owner.index.membership.bindings.record_count() == 0 {
            return Ok(None);
        }
        let p = processing(&owner.limits, formation, cancel);
        let mut bytes = size_of::<Profiles>()
            .checked_add(
                context
                    .profiles
                    .capacity()
                    .checked_mul(size_of::<DomInlineStyleProfile>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        for row in &context.profiles {
            p.step()?;
            bytes = bytes
                .checked_add(row.support_version.identity.capacity())
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let debit = self
            .custody_registry
            .ledger()
            .retain(custody::Usage::new(bytes, 0), |u| p.admit(u))?;
        let issuer = match &owner.issuer {
            Some(issuer) => issuer.clone(),
            None => {
                let debit = self
                    .custody_registry
                    .ledger()
                    .retain(custody::Usage::new(size_of::<InputIssuer>(), 0), |u| {
                        p.admit(u)
                    })?;
                Arc::new(InputIssuer {
                    next: AtomicU64::new(0),
                    _debit: debit,
                })
            }
        };
        // Keep already-normalized rows before their debit even if a later
        // row's allocation, original admission, or work check refuses.
        let mut profiles = Profiles {
            context,
            issuer: issuer.clone(),
            revision: self.revision.get(),
            limits: owner.limits.clone(),
            spent: AtomicU64::new(0),
            _debit: debit,
        };
        for row in &mut profiles.context.profiles {
            let properties = copy_rows(row.support.properties(), &mut profiles._debit, &p)?;
            let page = copy_rows(row.support.page_descriptors(), &mut profiles._debit, &p)?;
            let font_face = copy_rows(
                row.support.font_face_descriptors(),
                &mut profiles._debit,
                &p,
            )?;
            let glyph = row.support.svg_glyph_orientation_vertical();
            let mut support = surgeist_cssom::CssomDeclarationSupport::try_new(properties)?
                .with_page_descriptors(page)?
                .with_font_face_descriptors(font_face)?;
            if let Some(glyph) = glyph {
                support = support.with_svg_glyph_orientation_vertical(glyph);
            }
            row.support = support;
        }
        profiles.spent.store(p.spent.get(), Ordering::Relaxed);
        let recording = Recording {
            authority: Authority::Inputs(Arc::new(profiles)),
            index: owner.index.clone(),
            log: Log::default(),
        };
        // Issuance is operational identity, not canonical publication. Retain it
        // even if later resolver work observes an input and refuses the batch.
        self.inline_styles.as_mut().expect("live owner").issuer = Some(issuer);
        Ok(Some(recording))
    }

    /// Apply an ordinary DOM batch with authentic, invocation-ordered inline
    /// attribute and context inputs. All DOM formation precedes the single
    /// CSSOM commit; the following canonical DOM publication is infallible.
    /// Unbound Elements remain lazy; retirement keeps its original source order.
    pub fn apply_with_inline_styles(
        &mut self,
        batch: DomBatch,
        context: DomInlineStyleBatchContext,
        limits: DomInlineStyleRequestLimits,
        mut resolver: impl FnMut(
            &[CssomDeclarationCandidate],
            &CssomContext,
        ) -> Result<CssomAdmission, surgeist_cssom::CssomError>,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleCommit, DomInlineStyleError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if !Arc::ptr_eq(&batch.owner, &context.batch) {
            return Err(DomError::ForeignBatch.into());
        }
        let recording = self.begin_inline_recording(context, cancel)?;
        let prepared = self.prepare_with_inline(batch, recording, cancel)?;
        let mut publication = self.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        let inline = self
            .prepare_inline_batch(&publication, limits, &mut resolver, cancel)?
            .map(|batch| self.commit_inline_batch(batch))
            .transpose()?;
        Ok(DomInlineStyleCommit {
            dom: self.publish_prepared(publication),
            inline,
        })
    }
    pub(in crate::store) fn prepare_inline_batch(
        &self,
        publication: &PreparedPublication,
        limits: DomInlineStyleRequestLimits,
        resolver: &mut Resolver<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<surgeist_cssom::CssomBatch>, DomInlineStyleError> {
        let Some(recording) = publication
            .inline_recording
            .as_ref()
            .filter(|r| r.has_inputs())
        else {
            return Ok(None);
        };
        let owner = self.inline_styles.as_ref().expect("recorded live owner");
        owner.require_no_feedback()?;
        let store = owner.store.as_ref().expect("bound store");
        let mut batch = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
        let p = recording.processing(publication._formation.as_ref(), cancel);
        for index in 0..recording.log.len {
            p.step()?;
            let event = recording
                .log
                .entries
                .get_with(index, &mut || p.step())?
                .expect("ordered recorded occurrence");
            let element = event.element();
            let binding = owner
                .index
                .membership
                .bindings
                .get_with(element.slot, &mut || p.step())?
                .filter(|binding| binding.element == element)
                .ok_or(DomError::StaleNode)?;
            let object = &owner.objects[binding.ordinal];
            assert_eq!(object.element, element, "canonical exact inline locator");
            let id = object.properties.id();
            if let Event::Retirement { owner_revision, .. } = event {
                let snapshot = store.snapshot();
                let current = snapshot.block(id)?.owner().expect("bound inline owner");
                let bytes = size_of::<CssomInputVersion>()
                    .checked_add(current.identity.capacity())
                    .ok_or(DomError::RepresentationExhausted)?;
                p.scratch(bytes)?;
                let mut debit = self
                    .custody_registry
                    .ledger()
                    .retain(custody::Usage::new(bytes, 0), |u| p.admit(u))?;
                for _ in 0..current.identity.len() {
                    p.step()?;
                }
                let mut expected = current.clone();
                transaction::nodes::resize_debit(
                    &mut debit,
                    custody::Usage::new(
                        size_of::<CssomInputVersion>()
                            .checked_add(expected.identity.capacity())
                            .ok_or(DomError::RepresentationExhausted)?,
                        0,
                    ),
                    |u| p.admit(u),
                )?;
                if let Some(revision) = owner_revision {
                    expected.revision = *revision;
                }
                // Context events used this original serial in their accepted
                // packet. The producer preserves owner identity across updates.
                batch.retire_inline_declarations(id, &expected)?;
                continue;
            }
            let event = event.live();
            let profile = &recording.inputs().context.profiles[event.profile];
            let document = match event.facts.document {
                Key::Existing(id) => id,
                Key::Created(index) => publication
                    .allocation_ids
                    .get(index)
                    .expect("a created Document cannot retire through ordinary Destroy"),
            };
            let packet = form_packet(
                PacketInput {
                    element: event.element,
                    document,
                    parser: profile.parser.with_mode(event.facts.mode),
                    support_version: &profile.support_version,
                    support: &profile.support,
                    base: &event.facts.base,
                    revision: event.serial,
                },
                &self.custody_registry,
                &p,
            )?;
            batch.update_inline_declaration_context(id, packet.value().clone())?;
            if let Input::Attribute(value) = &event.input {
                let change = CssomStyleAttributeChange {
                    owner: packet.value().owner().clone(),
                    local_name: "style".into(),
                    namespace: None,
                    value: value
                        .as_ref()
                        .map(|v| CssomStyleAttributeInput::Utf16(v.shared_code_units().clone())),
                    echo: None,
                };
                if let Some(prepared) = batch.prepare_style_attribute_change(
                    id,
                    &change,
                    limits.input,
                    limits.declaration,
                    || p.step(),
                )? {
                    let admission = if prepared.candidates().is_empty() {
                        CssomAdmission::new(
                            Vec::new(),
                            packet
                                .value()
                                .context()
                                .inputs
                                .iter()
                                .map(|i| i.version.clone())
                                .collect(),
                        )
                    } else {
                        resolver(prepared.candidates(), packet.value().context())?
                    };
                    batch.apply_declaration_with_inputs(
                        prepared,
                        admission.decisions(),
                        admission.current_inputs(),
                    )?;
                }
            }
        }
        check_cancel(cancel)?;
        Ok(Some(batch))
    }
    pub(in crate::store) fn commit_inline_batch(
        &mut self,
        batch: surgeist_cssom::CssomBatch,
    ) -> Result<surgeist_cssom::CssomCommit, DomInlineStyleError> {
        let commit = self
            .inline_styles
            .as_mut()
            .expect("bound owner")
            .store
            .as_mut()
            .expect("bound store")
            .commit(batch)?;
        assert!(
            commit.owner_effects().is_empty(),
            "DOM inputs never echo owner effects"
        );
        Ok(commit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cancel() -> AtomicBool {
        AtomicBool::new(false)
    }
    fn request() -> DomInlineStyleRequestLimits {
        DomInlineStyleRequestLimits {
            input: surgeist_css::source::CssUtf16InputLimits::default(),
            declaration: surgeist_cssom::CssomDeclarationRequestLimits::default(),
        }
    }
    fn row(element: NodeId, document: DocumentId) -> DomInlineStyleProfile {
        DomInlineStyleProfile::new(
            element,
            document,
            surgeist_css::CssParserContext::default(),
            CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "empty".into(),
                revision: 1,
            },
            surgeist_cssom::CssomDeclarationSupport::default(),
        )
        .unwrap()
    }
    fn fixture(work: u64) -> (Dom, DocumentId, DocumentId, NodeId, NodeId, NodeId) {
        let limits = DomLimits::new(
            64,
            65536,
            128,
            64,
            64,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 4096, 1_000_000, 65536).unwrap(),
        )
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let a = batch.create_document();
        let b = batch.create_document();
        let root = batch.create_element_ns(&a, crate::Namespace::html(), "main");
        let first = batch.create_element_ns(&a, crate::Namespace::html(), "div");
        let second = batch.create_element_ns(&a, crate::Namespace::html(), "aside");
        batch.insert_before(&root, &a, None);
        batch.insert_before(&first, &root, None);
        batch.insert_before(&second, &root, None);
        let commit = dom.apply(batch, &cancel()).unwrap();
        let a = commit.document(&a).unwrap();
        let b = commit.document(&b).unwrap();
        let root = commit.created(&root).unwrap().unwrap();
        let first = commit.created(&first).unwrap().unwrap();
        let second = commit.created(&second).unwrap().unwrap();
        dom.configure_inline_styles(
            DomInlineStyleLimits::new(
                surgeist_cssom::CssomLimits::default(),
                8,
                64_000_000,
                8_000_000,
                work,
            )
            .unwrap(),
            &cancel(),
        )
        .unwrap();
        for element in [first, second] {
            let row = row(element, a);
            let input = DomInlineStyleInputs::new(
                dom.snapshot().document(a).unwrap().stamp(),
                row.parser,
                row.support_version,
                row.support,
            )
            .unwrap();
            dom.inline_style(
                element,
                &input,
                request(),
                |_, _| panic!("empty"),
                &cancel(),
            )
            .unwrap();
        }
        (dom, a, b, root, first, second)
    }
    #[test]
    fn vector_and_owned_child_replacement_reorder_current_membership_before_later_adoption() {
        for owned in [false, true] {
            let (mut dom, a, b, root, first, second) = fixture(1_000_000);
            let batch = DomBatch::new();
            let context = DomInlineStyleBatchContext::new(
                &batch,
                dom.snapshot().stamp(),
                vec![row(first, a), row(first, b), row(second, a), row(second, b)],
            )
            .unwrap();
            let recording = dom
                .begin_inline_recording(context, &cancel())
                .unwrap()
                .unwrap();
            let mut draft = Draft::new(&dom, batch.owner, &cancel()).unwrap();
            draft.inline_recording = Some(recording);
            let root = Key::Existing(root);
            let values = [Key::Existing(second), Key::Existing(first)];
            if owned {
                let mut scratch = draft.effect_vec();
                draft.select_reserve(&mut scratch, 2, &cancel()).unwrap();
                scratch.extend(values);
                draft
                    .relations_replace_owned(
                        root,
                        transaction::RelationField::Children,
                        scratch.into_values(),
                        &cancel(),
                    )
                    .unwrap();
            } else {
                draft
                    .relations_replace(
                        root,
                        transaction::RelationField::Children,
                        values.to_vec(),
                        &cancel(),
                    )
                    .unwrap();
            }
            draft
                .adopt(root, Key::Existing(b.node()), dom.limits, &cancel())
                .unwrap();
            let prepared = draft.finish(&dom, &cancel()).unwrap();
            let recording = prepared.inline_recording.as_ref().unwrap();
            let cancelled = cancel();
            let p = recording.processing(None, &cancelled);
            for element in [first, second] {
                assert!(
                    recording
                        .final_context(
                            element,
                            OrderView::state(&prepared.state, dom.owner),
                            &prepared.allocation_ids,
                            &p
                        )
                        .unwrap()
                );
                assert_eq!(
                    prepared.state.node(dom.owner, element).unwrap().document(),
                    b
                );
            }
            // The normal publication guard consumes this exact prepared result;
            // no direct canonical mutation is used by the fixture.
            let publication = dom.prepare_publication(prepared, &cancel()).unwrap();
            assert!(publication.inline_index.is_some());
        }
    }
    #[test]
    fn finite_cumulative_work_refusal_releases_private_events_and_preserves_both_owners() {
        let (mut dom, a, _, _, first, _) = fixture(8192);
        let old = dom.capture_inline_style(first).unwrap().unwrap();
        let mut batch = DomBatch::new();
        for _ in 0..4 {
            batch.set_attribute(first, "style", "--x:new");
        }
        let context =
            DomInlineStyleBatchContext::new(&batch, dom.snapshot().stamp(), vec![row(first, a)])
                .unwrap();
        let result = dom.apply_with_inline_styles(
            batch,
            context,
            request(),
            |_, _| panic!("empty"),
            &cancel(),
        );
        assert!(matches!(
            result,
            Err(DomInlineStyleError::Dom(DomError::WorkLimit {
                resource: DomResource::InlineStyleWork,
                limit: 8192
            }))
        ));
        let after = dom.capture_inline_style(first).unwrap().unwrap();
        assert_eq!(after.dom().stamp(), old.dom().stamp());
        assert_eq!(
            after.properties().snapshot().revision(),
            old.properties().snapshot().revision()
        );
        // Reuse the same owner after dropping all refused event/control banks.
        let mut batch = DomBatch::new();
        batch.remove_attribute(first, "style");
        let context =
            DomInlineStyleBatchContext::new(&batch, dom.snapshot().stamp(), Vec::new()).unwrap();
        assert!(
            dom.apply_with_inline_styles(
                batch,
                context,
                request(),
                |_, _| panic!("absent"),
                &cancel()
            )
            .unwrap()
            .inline()
            .is_none()
        );
    }
    #[test]
    fn actual_checkpoint_restores_recorded_suffix_and_candidate_membership_without_refunding_work()
    {
        let (mut dom, a, b, root, first, second) = fixture(1_000_000);
        let batch = DomBatch::new();
        let context = DomInlineStyleBatchContext::new(
            &batch,
            dom.snapshot().stamp(),
            vec![row(first, a), row(first, b), row(second, a), row(second, b)],
        )
        .unwrap();
        let recording = dom
            .begin_inline_recording(context, &cancel())
            .unwrap()
            .unwrap();
        let mut draft = Draft::new(&dom, batch.owner, &cancel()).unwrap();
        draft.inline_recording = Some(recording);
        let checkpoint = draft.source_checkpoint(0, 0, &cancel()).unwrap();
        draft
            .set_attribute(
                Key::Existing(first),
                None,
                &"style".into(),
                &"--x:discard".into(),
                dom.limits,
                &cancel(),
            )
            .unwrap();
        draft
            .adopt(
                Key::Existing(root),
                Key::Existing(b.node()),
                dom.limits,
                &cancel(),
            )
            .unwrap();
        let recording = draft.inline_recording.as_ref().unwrap();
        assert!(recording.log.len >= 3);
        let spent = recording.inputs().spent.load(Ordering::Relaxed);
        let serial = recording.log.entries.get(0).unwrap().live().serial;
        checkpoint.restore(&mut draft);
        let recording = draft.inline_recording.as_ref().unwrap();
        assert_eq!(recording.log.len, 0);
        assert_eq!(recording.inputs().spent.load(Ordering::Relaxed), spent);
        assert_eq!(
            draft.node(Key::Existing(first)).unwrap().document(),
            Key::Existing(a.node())
        );
        draft
            .set_attribute(
                Key::Existing(first),
                None,
                &"style".into(),
                &"--x:kept".into(),
                dom.limits,
                &cancel(),
            )
            .unwrap();
        let recording = draft.inline_recording.as_ref().unwrap();
        assert_eq!(recording.log.len, 1);
        assert!(recording.log.entries.get(0).unwrap().live().serial > serial);
        let prepared = draft.finish(&dom, &cancel()).unwrap();
        assert_eq!(prepared.state.node(dom.owner, first).unwrap().document(), a);
        assert!(dom.prepare_publication(prepared, &cancel()).is_ok());
    }
    #[test]
    fn later_profile_normalization_refusal_releases_earlier_copied_membership_and_issuer() {
        let (mut dom, a, _, _, first, second) = fixture(8192);
        dom.custody_registry.reconcile_releases();
        let before = dom.custody_registry.ledger().total();
        let mut first_row = row(first, a);
        first_row.support = surgeist_cssom::CssomDeclarationSupport::try_new(vec![(
            surgeist_css::property::CssPropertyGrammar::from_name("width").unwrap(),
            surgeist_cssom::CssomUsabilityRequirement::AllCheckedValues,
        )])
        .unwrap();
        let mut second_row = row(second, a);
        second_row.support = surgeist_cssom::CssomDeclarationSupport::try_new(
            surgeist_css::property::CssKnownProperty::all()
                .iter()
                .take(100)
                .map(|p| {
                    (
                        p.grammar(),
                        surgeist_cssom::CssomUsabilityRequirement::AllCheckedValues,
                    )
                })
                .collect(),
        )
        .unwrap();
        assert_eq!(second_row.support.properties().len(), 100);
        let batch = DomBatch::new();
        let context = DomInlineStyleBatchContext::new(
            &batch,
            dom.snapshot().stamp(),
            vec![first_row, second_row],
        )
        .unwrap();
        assert!(matches!(
            dom.apply_with_inline_styles(
                batch,
                context,
                request(),
                |_, _| panic!("normalization precedes parsing"),
                &cancel()
            ),
            Err(DomInlineStyleError::Dom(DomError::WorkLimit {
                resource: DomResource::InlineStyleWork,
                limit: 8192
            }))
        ));
        dom.custody_registry.reconcile_releases();
        assert_eq!(dom.custody_registry.ledger().total(), before);
        assert!(dom.inline_styles.as_ref().unwrap().issuer.is_none());
    }
}
