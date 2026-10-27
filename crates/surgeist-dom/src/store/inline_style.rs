//! Exclusive inline owner and sparse exact-generation operational membership.
use super::*;
pub(crate) mod feedback;
pub(in crate::store) mod recording;
mod retirement;
pub(super) use retirement::RetirementPlan;
#[cfg(test)]
mod tests;
use crate::inline_style::*;
use source::formation::Formation;
use std::cell::Cell;
use surgeist_css::CssParserMode;
use surgeist_cssom::{
    CssStyleProperties, CssomAdmission, CssomContext, CssomDeclarationCandidate, CssomEditGuard,
    CssomInlineDeclarationContext, CssomInput, CssomInputData, CssomInputRole, CssomInputVersion,
    CssomStore, CssomStyleAttributeChange, CssomStyleAttributeInput,
};
use transaction::ordered_members::{
    Edit, Map, MapCustody, Members, MembersCustody, Membership as MemberChange, Mode, OrderView,
    Qualified, RootHeader,
};

pub(super) struct ObjectEntry {
    element: NodeId,
    properties: CssStyleProperties,
}

#[derive(Clone)]
struct Binding {
    element: NodeId,
    ordinal: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Group {
    key: Key,
    members: RootHeader,
}
impl Qualified for Group {
    fn key(&self) -> Key {
        self.key
    }
}
#[derive(Clone, Default)]
pub(super) struct Membership {
    bindings: bank::Bank<Binding>,
    _custody: bank::BankCustody<Binding>,
}
impl Membership {
    pub(super) fn contains(&self, id: NodeId) -> bool {
        self.bindings.get(id.slot).is_some_and(|b| b.element == id)
    }
}
#[derive(Clone, Default)]
pub(super) struct Index {
    membership: Membership,
    roots: Map<Group>,
    documents: Map<Group>,
    members: Members,
    root_custody: MapCustody<Group>,
    document_custody: MapCustody<Group>,
    member_custody: MembersCustody,
}
#[derive(Default)]
struct Selection {
    values: Map<Key>,
    custody: MapCustody<Key>,
}
impl Selection {
    fn insert(
        &mut self,
        key: Key,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        self.values.change(
            &mut self.custody,
            key,
            Some(key),
            &mut Edit {
                registry,
                admit: &mut |u| p.admit(u),
                step: &mut || p.step(),
            },
        )?;
        p.scratch(
            self.values
                .existing
                .index_charge()
                .bytes
                .checked_add(self.values.existing.totals().bytes)
                .and_then(|n| n.checked_add(self.values.created.index_charge().bytes))
                .and_then(|n| n.checked_add(self.values.created.totals().bytes))
                .ok_or(DomError::RepresentationExhausted)?,
        )
    }
}
pub(super) struct Owner {
    limits: DomInlineStyleLimits,
    pub(super) store: Option<CssomStore>,
    pub(super) index: Index,
    pub(super) objects: Vec<ObjectEntry>,
    issuer: Option<Arc<recording::InputIssuer>>,
    feedback: Option<custody::Held<Box<feedback::Resident>>>,
    object_debit: custody::Debit,
    _header: custody::Debit,
}

struct Processing<'a> {
    limits: &'a DomInlineStyleLimits,
    formation: Option<&'a Formation>,
    cancel: &'a AtomicBool,
    spent: Cell<u64>,
    shared: Option<&'a std::sync::atomic::AtomicU64>,
}
impl Processing<'_> {
    fn step(&self) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        let spent = self
            .shared
            .map_or_else(|| self.spent.get(), |value| value.load(Ordering::Relaxed));
        let next = spent
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::WorkLimit {
                resource: DomResource::InlineStyleWork,
                limit: self.limits.work,
            });
        }
        source::formation::step(self.formation, self.cancel)?;
        self.spent.set(next);
        if let Some(shared) = self.shared {
            shared.store(next, Ordering::Relaxed);
        }
        Ok(())
    }
    fn admit(&self, usage: custody::Usage) -> Result<(), DomError> {
        self.step()?;
        admit(
            usage.bytes(),
            self.limits.formation_bytes,
            DomResource::InlineStyleStorage,
        )?;
        match self.formation {
            Some(f) => f.admit_custody_usage(usage, self.cancel),
            None => Ok(()),
        }
    }
    fn scratch(&self, bytes: usize) -> Result<(), DomError> {
        admit(
            bytes,
            self.limits.scratch_bytes,
            DomResource::InlineStyleScratch,
        )
    }
}
fn processing<'a>(
    limits: &'a DomInlineStyleLimits,
    formation: Option<&'a Formation>,
    cancel: &'a AtomicBool,
) -> Processing<'a> {
    Processing {
        limits,
        formation,
        cancel,
        spent: Cell::new(0),
        shared: None,
    }
}

fn record<T>(
    mut held: custody::Held<T>,
    registry: &bank::CustodyRegistry,
    p: &Processing<'_>,
) -> Result<bank::ControlledRecord<T>, DomError> {
    let usage = held.debit().usage();
    let retained = held.debit_mut().share(|u| p.admit(u))?;
    let (value, empty) = held.into_parts();
    debug_assert_eq!(empty.usage(), custody::Usage::ZERO);
    bank::ControlledRecord::new(
        value,
        bank::RecordDescription::from(usage).with_retained(retained),
        registry,
        |u| p.admit(u),
    )
}
impl Index {
    fn change_group(
        &mut self,
        group: Key,
        membership: MemberChange,
        document: bool,
        order: OrderView<'_>,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let (groups, controls) = if document {
            (&mut self.documents, &mut self.document_custody)
        } else {
            (&mut self.roots, &mut self.root_custody)
        };
        let root = groups
            .get(group)
            .map_or(RootHeader::default(), |g| g.members);
        let mut admit = |u| p.admit(u);
        let mut step = || p.step();
        let edit = Edit {
            registry,
            admit: &mut admit,
            step: &mut step,
        };
        let change = if document {
            self.members
                .change_identity(&self.member_custody, root, membership, edit)
        } else {
            self.members.change(
                &self.member_custody,
                root,
                membership,
                order,
                Mode::Ordinary,
                edit,
            )
        }?;
        self.members = change.members;
        self.member_custody = change.custody;
        groups.change(
            controls,
            group,
            (!change.root.is_empty()).then_some(Group {
                key: group,
                members: change.root,
            }),
            &mut Edit {
                registry,
                admit: &mut admit,
                step: &mut step,
            },
        )?;
        Ok(())
    }
    fn relocate(
        &mut self,
        element: NodeId,
        state: &State,
        owner: DomOwnerId,
        add: bool,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let order = OrderView::state(state, owner);
        let key = Key::Existing(element);
        let root = order.root(key, Mode::Ordinary, &mut || p.step())?;
        let document = Key::Existing(state.node(owner, element)?.document().node());
        let membership = if add {
            MemberChange::Insert(key)
        } else {
            MemberChange::Remove(key)
        };
        self.change_group(root, membership, false, order, registry, p)?;
        self.change_group(document, membership, true, order, registry, p)
    }
    fn replace_binding(
        &mut self,
        binding: Binding,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let element = binding.element;
        let usage = custody::Usage::new(size_of::<Binding>(), 0);
        let debit = registry.ledger().retain(usage, |u| p.admit(u))?;
        let value = record(custody::Held::new(binding, debit), registry, p)?;
        let (bank, custody) = self.membership.bindings.set_controlled(
            &self.membership._custody,
            element.slot,
            value,
            false,
            bank::Totals {
                bytes: usage.bytes(),
                units: 0,
                occurrences: 1,
            },
            registry,
            |u| p.admit(u),
        )?;
        self.membership = Membership {
            bindings: bank,
            _custody: custody,
        };
        Ok(())
    }
    fn install(
        &mut self,
        binding: Binding,
        state: &State,
        owner: DomOwnerId,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let element = binding.element;
        self.replace_binding(binding, registry, p)?;
        self.relocate(element, state, owner, true, registry, p)
    }
    fn remove_binding(
        &mut self,
        element: NodeId,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        p.step()?;
        let (bindings, custody, removed) = self.membership.bindings.remove_controlled(
            &self.membership._custody,
            element.slot,
            registry,
            |u| p.admit(u),
        )?;
        assert!(
            removed
                .as_ref()
                .is_some_and(|row| row.value().element == element)
        );
        self.membership = Membership {
            bindings,
            _custody: custody,
        };
        Ok(())
    }
    fn select(
        &self,
        subtree: NodeId,
        state: &State,
        owner: DomOwnerId,
        selected: &mut Selection,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let order = OrderView::state(state, owner);
        let subtree = Key::Existing(subtree);
        let root = order.root(subtree, Mode::Ordinary, &mut || p.step())?;
        let Some(group) = self.roots.get(root) else {
            return Ok(());
        };
        let mut next =
            self.members
                .lower_bound(group.members, subtree, order, Mode::Ordinary, &mut || {
                    p.step()
                })?;
        while let Some(key) = next {
            if !order.descendant(subtree, key, Mode::Ordinary, &mut || p.step())? {
                break;
            }
            selected.insert(key, registry, p)?;
            next = self
                .members
                .after(group.members, key, order, Mode::Ordinary, &mut || p.step())?;
        }
        Ok(())
    }
    fn select_document(
        &self,
        document: NodeId,
        selected: &mut Selection,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<(), DomError> {
        let Some(group) = self.documents.get(Key::Existing(document)) else {
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

fn ascii(value: &DocumentString, expected: &str, p: &Processing<'_>) -> Result<bool, DomError> {
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (a, b) in value.code_units().iter().zip(expected.bytes()) {
        p.step()?;
        if *a != u16::from(b) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn same_text(a: &DocumentString, b: &DocumentString, p: &Processing<'_>) -> Result<bool, DomError> {
    if Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()) {
        return Ok(true);
    }
    if a.len() != b.len() {
        return Ok(false);
    }
    for (a, b) in a.code_units().iter().zip(b.code_units()) {
        p.step()?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}
fn same_optional_text(
    a: Option<&DocumentString>,
    b: Option<&DocumentString>,
    p: &Processing<'_>,
) -> Result<bool, DomError> {
    match (a, b) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => same_text(a, b, p),
        _ => Ok(false),
    }
}
fn support_bytes(value: &surgeist_cssom::CssomDeclarationSupport) -> Result<usize, DomError> {
    value
        .properties()
        .len()
        .checked_mul(size_of::<(
            surgeist_css::property::CssPropertyGrammar,
            surgeist_cssom::CssomUsabilityRequirement,
        )>())
        .and_then(|n| {
            value
                .page_descriptors()
                .len()
                .checked_mul(size_of::<(
                    surgeist_css::rule::page::CssPageDescriptorKind,
                    surgeist_cssom::CssomUsabilityRequirement,
                )>())
                .and_then(|v| n.checked_add(v))
        })
        .and_then(|n| {
            value
                .font_face_descriptors()
                .len()
                .checked_mul(size_of::<(
                    surgeist_css::rule::font_face::CssFontFaceDescriptorKind,
                    surgeist_cssom::CssomUsabilityRequirement,
                )>())
                .and_then(|v| n.checked_add(v))
        })
        .ok_or(DomError::RepresentationExhausted)
}
fn style_value<'a>(
    element: &'a DomElement,
    p: &Processing<'_>,
) -> Result<Option<&'a DocumentString>, DomError> {
    for attribute in element.attributes() {
        p.step()?;
        if attribute.name().namespace_uri().is_none()
            && ascii(attribute.name().local_name(), "style", p)?
        {
            return Ok(Some(attribute.value()));
        }
    }
    Ok(None)
}
fn scope(
    state: &State,
    owner: DomOwnerId,
    element: NodeId,
    p: &Processing<'_>,
) -> Result<(NodeId, Option<NodeId>), DomError> {
    let mut root = element;
    loop {
        p.step()?;
        match state.node(owner, root)?.parent() {
            Some(parent) => root = parent,
            None => break,
        }
    }
    let node = state.node(owner, root)?;
    Ok((
        root,
        matches!(node.kind(), crate::DomNodeKind::ShadowRoot(_))
            .then(|| node.host())
            .flatten(),
    ))
}
fn base<'a>(
    state: &'a State,
    owner: DomOwnerId,
    document: DocumentId,
    p: &Processing<'_>,
) -> Result<&'a DocumentString, DomError> {
    p.step()?;
    let node = state.node(owner, document.node())?;
    let metadata = match node.kind() {
        crate::DomNodeKind::Document(m) => m,
        _ => return Err(DomError::WrongDocument),
    };
    let first = match node.base_document().ok_or(DomError::WrongDocument)?.first {
        None => surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
        Some(id) => {
            p.step()?;
            surgeist_html_semantics::HtmlDocumentFirstBase::Present(
                state
                    .node(owner, id)?
                    .base()
                    .and_then(|b| b.frozen.as_ref())
                    .map(|b| &b.value),
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
fn same_context(
    before: &State,
    after: &State,
    owner: DomOwnerId,
    id: NodeId,
    p: &Processing<'_>,
) -> Result<bool, DomError> {
    p.step()?;
    let old = before.node(owner, id)?;
    let new = after.node(owner, id)?;
    if old.document() != new.document()
        || scope(before, owner, id, p)? != scope(after, owner, id, p)?
    {
        return Ok(false);
    }
    let document = old.document();
    let a = before.node(owner, document.node())?;
    let b = after.node(owner, document.node())?;
    let (crate::DomNodeKind::Document(a), crate::DomNodeKind::Document(b)) = (a.kind(), b.kind())
    else {
        return Err(DomError::WrongDocument);
    };
    if a.class() != b.class()
        || a.mode() != b.mode()
        || a.origin() != b.origin()
        || !same_text(a.encoding(), b.encoding(), p)?
        || !same_text(a.url(), b.url(), p)?
        || a.document_environment() != b.document_environment()
    {
        return Ok(false);
    }
    same_text(
        base(before, owner, document, p)?,
        base(after, owner, document, p)?,
        p,
    )
}
struct PacketInput<'a> {
    element: NodeId,
    document: NodeId,
    parser: surgeist_css::CssParserContext,
    support_version: &'a CssomInputVersion,
    support: &'a surgeist_cssom::CssomDeclarationSupport,
    base: &'a DocumentString,
    revision: u64,
}
fn form_packet(
    input: PacketInput<'_>,
    registry: &bank::CustodyRegistry,
    p: &Processing<'_>,
) -> Result<custody::Held<CssomInlineDeclarationContext>, DomInlineStyleError> {
    let base = input.base;
    let support_bytes = support_bytes(input.support)?;
    // Prefix/suffix plus owner, slot and generation have at most 96
    // characters. 128 also
    // covers temporary formatting capacity. The
    // packet, its creation operand and current-version/owner operands are
    // simultaneously live; admit their conservative complete overlap.
    const ID_CAPACITY: usize = 128;
    const ID_OCCURRENCES: usize = 24;
    let packet_bytes = size_of::<CssomContext>()
        .checked_add(10 * size_of::<CssomInput>())
        .and_then(|n| n.checked_add(5 * size_of::<CssomInputVersion>()))
        .and_then(|n| n.checked_add(ID_OCCURRENCES * ID_CAPACITY + 15))
        .and_then(|n| base.len().checked_mul(6).and_then(|v| n.checked_add(v)))
        .and_then(|n| {
            input
                .support_version
                .identity
                .len()
                .checked_mul(3)
                .and_then(|v| n.checked_add(v))
        })
        .and_then(|n| support_bytes.checked_mul(2).and_then(|v| n.checked_add(v)))
        .ok_or(DomError::RepresentationExhausted)?;
    p.scratch(packet_bytes)?;
    for _ in 0..base
        .len()
        .checked_add(support_bytes)
        .and_then(|n| n.checked_add(ID_OCCURRENCES * ID_CAPACITY))
        .and_then(|n| {
            input
                .support_version
                .identity
                .len()
                .checked_mul(3)
                .and_then(|v| n.checked_add(v))
        })
        .ok_or(DomError::RepresentationExhausted)?
    {
        p.step()?;
    }
    let packet_credit = registry
        .ledger()
        .retain(custody::Usage::new(packet_bytes, 0), |u| p.admit(u))?;
    let base = base.to_utf8().map_err(|_| DomError::NotSupported)?;
    let identity = format!(
        "inline:element:{}:{}:{}",
        input.element.owner.get(),
        input.element.slot,
        input.element.generation
    );
    let document_identity = format!(
        "inline:document:{}:{}:{}",
        input.element.owner.get(),
        input.document.slot,
        input.document.generation
    );
    let parser_identity = format!("{identity}:parser");
    let revision = input.revision;
    let version = |role, identity| CssomInputVersion {
        role,
        identity,
        revision,
    };
    let packet = CssomInlineDeclarationContext::try_new(CssomContext {
        inputs: vec![
            CssomInput {
                version: version(CssomInputRole::Owner, identity.clone()),
                data: CssomInputData::Owner {
                    identity,
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: version(CssomInputRole::Document, document_identity.clone()),
                data: CssomInputData::Document {
                    identity: document_identity.clone(),
                },
            },
            CssomInput {
                version: version(CssomInputRole::ParserMode, parser_identity),
                data: CssomInputData::ParserMode(input.parser),
            },
            CssomInput {
                version: version(CssomInputRole::Base, format!("{document_identity}:base")),
                data: CssomInputData::Base {
                    url: Some(base),
                    location: None,
                },
            },
            CssomInput {
                version: input.support_version.clone(),
                data: CssomInputData::Support(input.support.clone()),
            },
        ],
        linked: Vec::new(),
    })?;
    Ok(custody::Held::new(packet, packet_credit))
}
fn required(element: NodeId, change: DomInlineStyleChange) -> DomError {
    DomError::InlineStyleContextRequired { element, change }
}

impl Dom {
    /// Install finite inline policy without allocating a CSSOM store or binding.
    pub fn configure_inline_styles(
        &mut self,
        limits: DomInlineStyleLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomInlineStyleError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if self.inline_styles.is_some() {
            return Err(DomInlineStyleError::AlreadyConfigured);
        }
        let formation = self.source_observation_formation(cancel)?;
        let p = processing(&limits, formation.as_ref(), cancel);
        let header = self
            .custody_registry
            .ledger()
            .retain(custody::Usage::new(size_of::<Owner>(), 0), |u| p.admit(u))?;
        self.inline_styles = Some(Box::new(Owner {
            limits,
            store: None,
            index: Index::default(),
            objects: Vec::new(),
            issuer: None,
            feedback: None,
            object_debit: self
                .custody_registry
                .ledger()
                .retain(custody::Usage::ZERO, |_| Ok(()))?,
            _header: header,
        }));
        Ok(())
    }
    /// Obtain the actual same-object CSSStyleProperties, lazily initializing it
    /// from this Element's current canonical UTF-16 style attribute. Existing
    /// bindings require the same explicit external profile/support packet.
    pub fn inline_style(
        &mut self,
        element: NodeId,
        inputs: &DomInlineStyleInputs,
        limits: DomInlineStyleRequestLimits,
        mut resolver: impl FnMut(
            &[CssomDeclarationCandidate],
            &CssomContext,
        ) -> Result<CssomAdmission, surgeist_cssom::CssomError>,
        cancel: &AtomicBool,
    ) -> Result<CssStyleProperties, DomInlineStyleError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        let node = self.state.node(self.owner, element)?;
        let language = node.element().ok_or(DomError::WrongNodeKind)?;
        let namespace = language.name().namespace_uri();
        let owner = self
            .inline_styles
            .as_ref()
            .ok_or(DomInlineStyleError::Unconfigured)?;
        let formation = self.source_observation_formation(cancel)?;
        let policy = owner.limits.clone();
        let p = processing(&policy, formation.as_ref(), cancel);
        if !namespace
            .map(|ns| {
                for expected in [
                    "http://www.w3.org/1999/xhtml",
                    "http://www.w3.org/2000/svg",
                    "http://www.w3.org/1998/Math/MathML",
                ] {
                    if ascii(ns, expected, &p)? {
                        return Ok::<_, DomError>(true);
                    }
                }
                Ok(false)
            })
            .transpose()?
            .unwrap_or(false)
        {
            return Err(DomInlineStyleError::Inapplicable);
        }
        if inputs.basis.store().owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        if inputs.basis.store() != self.snapshot().stamp()
            || inputs.basis.document() != node.document()
        {
            return Err(DomInlineStyleError::StaleContext);
        }
        let document = self.state.node(self.owner, node.document().node())?;
        let crate::DomNodeKind::Document(metadata) = document.kind() else {
            return Err(DomError::WrongDocument.into());
        };
        let mode = if metadata.mode() == crate::HtmlDocumentMode::Quirks {
            CssParserMode::Quirks
        } else {
            CssParserMode::Standards
        };
        if inputs.parser.mode() != mode {
            return Err(DomInlineStyleError::ParserModeMismatch);
        }
        if let Some(binding) = owner
            .index
            .membership
            .bindings
            .get_with(element.slot, &mut || p.step())?
            .filter(|b| b.element == element)
        {
            // Earlier binding admission belongs to an earlier invocation. Read
            // qualification consumes this caller's current original work too.
            let comparisons = support_bytes(&inputs.support)?
                .checked_add(inputs.support_version.identity.len())
                .ok_or(DomError::RepresentationExhausted)?;
            for _ in 0..comparisons {
                p.step()?;
            }
            let store = owner.store.as_ref().expect("bound inline store");
            let snapshot = store.snapshot();
            let context = snapshot
                .block(owner.objects[binding.ordinal].properties.id())?
                .inline_context()
                .expect("DOM-created inline packet")
                .context();
            if !context.inputs.iter().any(|i| i.data == CssomInputData::ParserMode(inputs.parser))
                || !context.inputs.iter().any(|i| i.version == inputs.support_version && matches!(&i.data, CssomInputData::Support(support) if support == &inputs.support)) {
                return Err(DomInlineStyleError::ContextUpdateRequired);
            }
            return Ok(owner.objects[binding.ordinal].properties.clone());
        }
        owner.require_no_feedback()?;
        admit(
            owner
                .index
                .membership
                .bindings
                .record_count()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            owner.limits.bindings,
            DomResource::InlineStyleBindings,
        )?;
        // Strings are strict UTF-8 views of already canonical URL facts. The
        // style attribute itself never passes through this conversion.
        let base = base(&self.state, self.owner, node.document(), &p)?;
        // CSSOM owns the resulting packet and its independent state quotas.
        // Admit this adapter's complete prospective construction overlap before
        // allocating identifiers, URL encoding, membership clones or the packet.
        let revision = match &owner.issuer {
            Some(issuer) => issuer.issue(self.revision.get())?,
            None => self.revision.get(),
        };
        let packet = form_packet(
            PacketInput {
                element,
                document: node.document().node(),
                parser: inputs.parser,
                support_version: &inputs.support_version,
                support: &inputs.support,
                base,
                revision,
            },
            &self.custody_registry,
            &p,
        )?;
        let mut new_store = match owner.store {
            Some(_) => None,
            None => Some(CssomStore::new(
                owner.limits.cssom.clone(),
                CssomContext::default(),
            )?),
        };
        let store = owner
            .store
            .as_ref()
            .or(new_store.as_ref())
            .expect("existing or provisional store");
        let mut batch = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
        let ticket = batch.create_inline_declarations(packet.value().clone())?;
        if let Some(value) = style_value(language, &p)? {
            let change = CssomStyleAttributeChange {
                owner: packet.value().owner().clone(),
                local_name: "style".into(),
                namespace: None,
                value: Some(CssomStyleAttributeInput::Utf16(
                    value.shared_code_units().clone(),
                )),
                echo: None,
            };
            if let Some(prepared) = batch.prepare_created_style_attribute_change(
                &ticket,
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
        // The sparse row names a pre-admitted slot, never a tentative CSSOM ID.
        let ordinal = owner.objects.len();
        let mut index = owner.index.clone();
        index.install(
            Binding { element, ordinal },
            &self.state,
            self.owner,
            &self.custody_registry,
            &p,
        )?;
        let mut replacement = None;
        if owner.objects.len() == owner.objects.capacity() {
            let capacity = owner
                .objects
                .capacity()
                .checked_mul(2)
                .unwrap_or(owner.limits.bindings)
                .max(4)
                .min(owner.limits.bindings);
            let bytes = capacity
                .checked_mul(size_of::<ObjectEntry>())
                .ok_or(DomError::RepresentationExhausted)?;
            let debit = self
                .custody_registry
                .ledger()
                .retain(custody::Usage::new(bytes, 0), |u| p.admit(u))?;
            let mut next = custody::Held::new(Vec::new(), debit);
            next.value_mut()
                .try_reserve_exact(capacity)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = next
                .value()
                .capacity()
                .checked_mul(size_of::<ObjectEntry>())
                .ok_or(DomError::RepresentationExhausted)?;
            transaction::nodes::resize_debit(
                next.debit_mut(),
                custody::Usage::new(actual, 0),
                |u| p.admit(u),
            )?;
            for _ in 0..owner.objects.len() {
                p.step()?;
            }
            replacement = Some(next);
        }
        check_cancel(cancel)?;
        // Both writers are exclusively borrowed; no callbacks or admission
        // occur between this commit and the infallible registry handoff.
        let store = self
            .inline_styles
            .as_mut()
            .expect("configured owner")
            .store
            .as_mut()
            .or(new_store.as_mut())
            .expect("prepared store");
        let commit = store.commit(batch)?;
        assert!(
            commit.owner_effects().is_empty(),
            "external attribute initialization never echoes"
        );
        let id = commit
            .block(&ticket)
            .expect("the private ticket belongs to the committed creation");
        let properties = store
            .css_style_properties(id)
            .expect("the created ordinary inline block is Properties");
        let owner = self.inline_styles.as_mut().expect("configured owner");
        if owner.store.is_none() {
            owner.store = new_store;
        }
        assert_eq!(
            owner.objects.len(),
            ordinal,
            "exclusive prepared inline slot"
        );
        if let Some(mut replacement) = replacement {
            replacement.value_mut().append(&mut owner.objects);
            let (values, debit) = replacement.into_parts();
            let old = custody::Held::new(
                std::mem::replace(&mut owner.objects, values),
                std::mem::replace(&mut owner.object_debit, debit),
            );
            drop(old);
        }
        owner.objects.push(ObjectEntry {
            element,
            properties: properties.clone(),
        });
        owner.index = index;
        Ok(properties)
    }

    /// Capture the current coupled state without retaining its live writer.
    pub fn capture_inline_style(
        &self,
        element: NodeId,
    ) -> Result<Option<DomInlineStyleCapture>, DomInlineStyleError> {
        self.state.node(self.owner, element)?;
        self.snapshot().require_complete_source()?;
        let Some(owner) = &self.inline_styles else {
            return Ok(None);
        };
        let Some(binding) = owner
            .index
            .membership
            .bindings
            .get(element.slot)
            .filter(|b| b.element == element)
        else {
            return Ok(None);
        };
        if owner.feedback_target() == Some(element) {
            return Err(DomInlineStyleError::OperationPending { element });
        }
        let dom = self.snapshot();
        let scope = dom.node(element)?.tree_scope();
        let properties = owner.objects[binding.ordinal]
            .properties
            .capture(&owner.store.as_ref().expect("bound store").snapshot())?;
        Ok(Some(DomInlineStyleCapture {
            element,
            scope,
            dom,
            properties,
        }))
    }

    pub(super) fn prepare_inline_publication(
        &self,
        prepared: &Prepared,
        cancel: &AtomicBool,
    ) -> Result<(Option<Index>, Option<RetirementPlan>), DomError> {
        let Some(owner) = self
            .inline_styles
            .as_ref()
            .filter(|o| o.index.membership.bindings.record_count() != 0)
        else {
            return Ok((None, None));
        };
        let p = match &prepared.inline_recording {
            Some(recording) => recording.processing(prepared.formation.as_ref(), cancel),
            None => processing(&owner.limits, prepared.formation.as_ref(), cancel),
        };
        let mut selected = Selection::default();
        if let Some(positions) = &prepared.inline_positions {
            for key in positions.keys.values() {
                p.step()?;
                let id = match key {
                    Key::Existing(id) => Some(*id),
                    Key::Created(index) => prepared.allocation_ids.get(*index),
                };
                if let Some(id) = id.filter(|id| self.state.node(self.owner, *id).is_ok()) {
                    owner.index.select(
                        id,
                        &self.state,
                        self.owner,
                        &mut selected,
                        &self.custody_registry,
                        &p,
                    )?;
                }
            }
        }
        for change in &prepared.changes {
            p.step()?;
            match change {
                DomChange::Parent(id) if self.state.node(self.owner, *id).is_ok() => {
                    owner.index.select(
                        *id,
                        &self.state,
                        self.owner,
                        &mut selected,
                        &self.custody_registry,
                        &p,
                    )?
                }
                DomChange::DocumentBase(id)
                | DomChange::DocumentEncodingUrlFacts(id)
                | DomChange::DocumentMeta(id) => owner.index.select_document(
                    id.node(),
                    &mut selected,
                    &self.custody_registry,
                    &p,
                )?,
                DomChange::Data(id) => {
                    owner
                        .index
                        .select_document(*id, &mut selected, &self.custody_registry, &p)?
                }
                DomChange::Destroyed(id) if owner.index.membership.contains(*id) => {
                    if !prepared
                        .inline_recording
                        .as_ref()
                        .map(|recording| recording.retired(*id, &p))
                        .transpose()?
                        .unwrap_or(false)
                    {
                        return Err(required(*id, DomInlineStyleChange::Retirement));
                    }
                    selected.insert(Key::Existing(*id), &self.custody_registry, &p)?;
                }
                DomChange::NodeDocument(id) if owner.index.membership.contains(*id) => {
                    selected.insert(Key::Existing(*id), &self.custody_registry, &p)?;
                }
                DomChange::Attributes(id) if owner.index.membership.contains(*id) => {
                    let a = self
                        .state
                        .node(self.owner, *id)?
                        .element()
                        .ok_or(DomError::WrongNodeKind)?;
                    if prepared
                        .inline_recording
                        .as_ref()
                        .map(|recording| recording.retired(*id, &p))
                        .transpose()?
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    let b = prepared
                        .state
                        .node(self.owner, *id)?
                        .element()
                        .ok_or(DomError::WrongNodeKind)?;
                    let matching_echo = match &prepared.inline_echo {
                        Some(echo) if echo.element == *id => {
                            same_optional_text(Some(&echo.value), style_value(b, &p)?, &p)?
                        }
                        _ => false,
                    };
                    if !matching_echo
                        && !same_optional_text(style_value(a, &p)?, style_value(b, &p)?, &p)?
                        && !prepared
                            .inline_recording
                            .as_ref()
                            .map(|r| r.final_attribute(*id, style_value(b, &p)?, &p))
                            .transpose()?
                            .unwrap_or(false)
                    {
                        return Err(required(*id, DomInlineStyleChange::Attribute));
                    }
                }
                _ => {}
            }
        }
        if selected.values.values().next().is_none() {
            return Ok((None, None));
        }
        for key in selected.values.values() {
            let Key::Existing(id) = key else {
                unreachable!("published binding identity");
            };
            if prepared
                .inline_recording
                .as_ref()
                .map(|recording| recording.retired(*id, &p))
                .transpose()?
                .unwrap_or(false)
            {
                continue;
            }
            if owner.feedback_target() == Some(*id)
                && !same_context(&self.state, &prepared.state, self.owner, *id, &p)?
            {
                return Err(DomError::InlineStyleOperationPending { element: *id });
            }
            if !same_context(&self.state, &prepared.state, self.owner, *id, &p)?
                && !prepared
                    .inline_recording
                    .as_ref()
                    .map(|r| {
                        r.final_context(
                            *id,
                            OrderView::state(&prepared.state, self.owner),
                            &prepared.allocation_ids,
                            &p,
                        )
                    })
                    .transpose()?
                    .unwrap_or(false)
            {
                return Err(required(*id, DomInlineStyleChange::Context));
            }
        }
        let mut candidate = owner.index.clone();
        for key in selected.values.values() {
            let Key::Existing(id) = key else {
                unreachable!("published binding identity");
            };
            candidate.relocate(
                *id,
                &self.state,
                self.owner,
                false,
                &self.custody_registry,
                &p,
            )?;
        }
        for key in selected.values.values() {
            let Key::Existing(id) = key else {
                unreachable!("published binding identity");
            };
            if prepared
                .inline_recording
                .as_ref()
                .map(|recording| recording.retired(*id, &p))
                .transpose()?
                .unwrap_or(false)
            {
                candidate.remove_binding(*id, &self.custody_registry, &p)?;
                continue;
            }
            candidate.relocate(
                *id,
                &prepared.state,
                self.owner,
                true,
                &self.custody_registry,
                &p,
            )?;
        }
        let retirement = prepared
            .inline_recording
            .as_ref()
            .map(|recording| {
                owner.prepare_retirement(recording, &mut candidate, &self.custody_registry, &p)
            })
            .transpose()?
            .flatten();
        Ok((Some(candidate), retirement))
    }
}

impl Dom {
    pub(super) fn inline_membership(&self) -> Option<Membership> {
        self.inline_styles
            .as_ref()
            .filter(|o| o.index.membership.bindings.record_count() != 0)
            .map(|o| o.index.membership.clone())
    }
}
