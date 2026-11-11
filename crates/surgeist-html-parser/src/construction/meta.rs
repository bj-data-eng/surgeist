//! Original-controller Meta transitions. Missing external operations are data;
//! they never certify execution or reset an unknown current value.
use super::*;
use crate::keyed::{Table, slots};
use associated::HtmlParseHyperlinkUnavailable;
use surgeist_html::attributes::meta_pragma::HtmlMetaPragmaState as Pragma;
use surgeist_html::{
    attributes::{HtmlReferrerPolicy, meta_processing::default_document_referrer_policy},
    construction::ConstructionNodeKind,
    values::DocumentString,
};

/// Independent domains in the finite current Document history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseMetaDomain {
    Refresh,
    Csp,
    ThemeColor,
    ColorScheme,
}
impl HtmlParseMetaDomain {
    pub(super) const ALL: [Self; 4] = [
        Self::Refresh,
        Self::Csp,
        Self::ThemeColor,
        Self::ColorScheme,
    ];
}
const META_DOMAIN_COUNT: usize = 4;
/// One value per [`HtmlParseMetaDomain`]; a lookup cannot be out of range.
pub(super) type MetaDomainTable<T> = Table<HtmlParseMetaDomain, T, META_DOMAIN_COUNT>;
slots!(HtmlParseMetaDomain, META_DOMAIN_COUNT, {
    Refresh => 0,
    Csp => 1,
    ThemeColor => 2,
    ColorScheme => 3,
});
// `ALL` lists every domain once, in discriminant order.
#[expect(
    clippy::indexing_slicing,
    reason = "evaluated at compile time with i < META_DOMAIN_COUNT, the array length: an index out of range fails the build"
)]
const _: () = {
    let mut i = 0;
    while i < META_DOMAIN_COUNT {
        assert!(HtmlParseMetaDomain::ALL[i] as usize == i);
        i += 1;
    }
};
/// What one candidate meta says about the selection it is a candidate for.
enum Reading {
    /// It selects nothing, so the next candidate decides.
    Rejected,
    Theme(surgeist_color::Color),
    Scheme(crate::parser::control::CssSchemeValue),
    /// An operation it needs cannot be completed, which ends the selection.
    Unsatisfied(HtmlParseMetaMissingKind),
}
/// What a selection ends up with after reading candidates.
enum Decision {
    Nothing,
    Theme(usize, surgeist_color::Color),
    Scheme(usize, crate::parser::control::CssSchemeValue),
    /// The gap is already recorded; nothing is selected.
    Unsatisfied,
}
/// The insertion or removal of a subtree during which a selection was last
/// walked. The topology count makes it one particular change. The subtree, the
/// stage and the detached root alone do not: a subtree that leaves the tree
/// unobserved, because it holds no candidate then, and returns with candidates
/// moved into it meanwhile, repeats all three while the selection has changed
/// without it.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct MetaBatch {
    root: usize,
    removal: bool,
    detached: Option<usize>,
    topology_changes: u64,
}

/// How many levels of each record `branches` remembers.
const CHAIN: usize = 24;
/// How two records of one tree lie in relation to each other.
enum Lineage {
    /// The first is the second or one of its ancestors.
    FirstEncloses,
    /// The second is an ancestor of the first.
    SecondEncloses,
    /// They lie in different children, `first` and `second`, of `above`.
    Branches {
        above: usize,
        first: usize,
        second: usize,
    },
}
/// The levels from a record up that a climb has passed.
struct Chain {
    nodes: [usize; CHAIN],
    len: usize,
}
impl Chain {
    const fn new() -> Self {
        Self {
            nodes: [0; CHAIN],
            len: 0,
        }
    }
    /// Remembers one more level, or reports that there is no room.
    fn push(&mut self, node: usize) -> bool {
        match self.nodes.get_mut(self.len) {
            Some(slot) => {
                *slot = node;
                self.len = self.len.saturating_add(1);
                true
            }
            None => false,
        }
    }
    fn levels(&self) -> &[usize] {
        self.nodes.get(..self.len).unwrap_or(&[])
    }
}
type Parent<'a> = &'a dyn Fn(&mut ParseControl, usize) -> Result<Option<usize>, SessionError>;
/// One record's climb toward the root.
struct Side {
    /// The levels passed so far, kept while the other side may still meet them.
    chain: Chain,
    /// The next level up, or `None` once the root has been passed.
    next: Option<usize>,
    /// The level passed last, which is the child of the next one on this path.
    last: Option<usize>,
}
/// What one step of a climb found.
enum Climb {
    /// Neither a meeting nor the end of the memory.
    Climbed,
    /// The memory is full and the paths have not met.
    Overflow,
    /// The paths meet at `node`; `below_this` and `below_other` are the levels
    /// under it on the climbing side's path and on the other's.
    Met {
        node: usize,
        below_this: Option<usize>,
        below_other: Option<usize>,
    },
}
/// Advances `this` one level and looks for it among the levels `other` has
/// passed. A side remembers its levels only while the other side is still
/// climbing, since only then will the other test its levels against them.
fn climb_one(
    c: &mut ParseControl,
    parent: Parent<'_>,
    this: &mut Side,
    other: &Side,
) -> Result<Climb, SessionError> {
    let Some(node) = this.next else {
        return Ok(Climb::Climbed);
    };
    if other.next.is_some() && !this.chain.push(node) {
        return Ok(Climb::Overflow);
    }
    if let Some(level) = other.chain.levels().iter().position(|&m| m == node) {
        return Ok(Climb::Met {
            node,
            below_this: this.last,
            below_other: level
                .checked_sub(1)
                .and_then(|below| other.chain.levels().get(below))
                .copied(),
        });
    }
    this.last = Some(node);
    this.next = parent(c, node)?;
    Ok(Climb::Climbed)
}
/// Finds how `first` and `second` branch. The records climb in step and each
/// level is tested against the levels the other has passed, so the climb ends
/// at their lowest common ancestor: two looks for each level between the
/// farther record and that ancestor. A record that reaches the root stops
/// climbing and the other goes on alone, one look for each level.
fn branches(
    c: &mut ParseControl,
    parent: Parent<'_>,
    first: usize,
    second: usize,
    records: usize,
) -> Result<Lineage, SessionError> {
    let side = |node: usize| Side {
        chain: Chain::new(),
        next: Some(node),
        last: None,
    };
    let (mut one, mut two) = (side(first), side(second));
    let enclose = |one_below: Option<usize>, two_below: Option<usize>, above: usize| match (
        one_below, two_below,
    ) {
        (None, _) => Lineage::FirstEncloses,
        (_, None) => Lineage::SecondEncloses,
        (Some(first), Some(second)) => Lineage::Branches {
            above,
            first,
            second,
        },
    };
    for _ in 0..records.saturating_mul(2) {
        match climb_one(c, parent, &mut one, &two)? {
            Climb::Climbed => {}
            Climb::Overflow => return branches_by_depth(c, parent, first, second, records),
            Climb::Met {
                node,
                below_this,
                below_other,
            } => return Ok(enclose(below_this, below_other, node)),
        }
        match climb_one(c, parent, &mut two, &one)? {
            Climb::Climbed => {}
            Climb::Overflow => return branches_by_depth(c, parent, first, second, records),
            Climb::Met {
                node,
                below_this,
                below_other,
            } => return Ok(enclose(below_other, below_this, node)),
        }
        if one.next.is_none() && two.next.is_none() {
            break;
        }
    }
    Err(SessionError::Cycle)
}
/// As [`branches`] for records too deep to remember: both are climbed to the
/// root to learn their depths, the deeper is lifted to the other's, and both
/// climb to the ancestor they share.
fn branches_by_depth(
    c: &mut ParseControl,
    parent: Parent<'_>,
    first: usize,
    second: usize,
    records: usize,
) -> Result<Lineage, SessionError> {
    let depth = |c: &mut ParseControl, mut node: usize| -> Result<usize, SessionError> {
        let mut depth = 0_usize;
        while let Some(above) = parent(c, node)? {
            node = above;
            depth = depth.checked_add(1).ok_or(SessionError::Cycle)?;
            if depth > records {
                return Err(SessionError::Cycle);
            }
        }
        Ok(depth)
    };
    let (mut left, mut right) = (first, second);
    let (mut left_depth, mut right_depth) = (depth(c, left)?, depth(c, right)?);
    while left_depth > right_depth {
        left = parent(c, left)?.ok_or(SessionError::Cycle)?;
        left_depth = left_depth.saturating_sub(1);
        if left == right {
            return Ok(Lineage::SecondEncloses);
        }
    }
    while right_depth > left_depth {
        right = parent(c, right)?.ok_or(SessionError::Cycle)?;
        right_depth = right_depth.saturating_sub(1);
        if left == right {
            return Ok(Lineage::FirstEncloses);
        }
    }
    loop {
        let (up_left, up_right) = (parent(c, left)?, parent(c, right)?);
        if up_left == up_right {
            let above = up_left.ok_or(SessionError::Cycle)?;
            return Ok(Lineage::Branches {
                above,
                first: left,
                second: right,
            });
        }
        left = up_left.ok_or(SessionError::Cycle)?;
        right = up_right.ok_or(SessionError::Cycle)?;
    }
}
/// A change to one meta, as it bears on a selection.
#[derive(Clone, Copy)]
enum Change {
    /// It is attached and a candidate for the selection.
    Candidate,
    /// It leaves the tree.
    Removed,
}
/// What a selection needs after one meta changes.
enum Step {
    Unchanged,
    /// Read the changed meta alone.
    Read,
    /// Walk the tree again.
    Walk,
}
/// First actual unsatisfied operation in one independent domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlParseMetaMissingKind {
    RefreshUrl(HtmlParseHyperlinkUnavailable),
    Csp,
    CspProcessing(surgeist_html::security::csp::HtmlCspUnavailable),
    ThemeColor,
    ThemeColorProcessing(crate::parser::control::HtmlColorProcessingError),
}
impl HtmlParseMetaMissingKind {
    pub(super) fn domain(&self) -> HtmlParseMetaDomain {
        match self {
            Self::RefreshUrl(_) => HtmlParseMetaDomain::Refresh,
            Self::Csp | Self::CspProcessing(_) => HtmlParseMetaDomain::Csp,
            Self::ThemeColor | Self::ThemeColorProcessing(_) => HtmlParseMetaDomain::ThemeColor,
        }
    }
}
/// Closed current state issued only by the source-created Document constructor.
/// The original controller has no external UI association or execution path.
/// It chooses HTML877d's Do-nothing refresh policy, independently of BC state.
/// Gap slots are at most one per domain, never a transition/event log.
pub(super) struct MetaHistory {
    pub(super) language: Option<usize>,
    pub(super) csp: Option<usize>,
    pub(super) base_csp: Option<usize>,
    pub(super) referrer: HtmlReferrerPolicy,
    pub(super) refresh: bool,
    pub(super) theme: Option<surgeist_color::Color>,
    pub(super) scheme: Option<crate::parser::control::CssSchemeValue>,
    pub(super) gaps: MetaDomainTable<Option<usize>>,
    pub(super) first: Option<HtmlParseMetaDomain>,
    /// Where each color selection stands since the last walk of the tree.
    pub(super) selections: MetaDomainTable<SelectionIndex>,
}
/// What a walk of the document tree last learned about a color selection.
///
/// A walk reads candidates in tree order and stops at the first one that
/// decides something, so it also learns that every attached candidate before
/// it decides nothing. Each change made to one candidate keeps that true or
/// ends it, and while it holds, a candidate that changes needs reading only if
/// it precedes the decisive one, and only a change to the decisive one itself
/// needs another walk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SelectionIndex {
    /// No walk has run, or the record cannot name the deciding candidate.
    Stale,
    /// The first candidate in tree order that selected a value, if any, as a
    /// record position plus one.
    Current(Option<NonZeroU32>),
}
impl SelectionIndex {
    /// The state after a walk found `deciding`. A record too far along to be
    /// named by the cell leaves the state stale, which only costs a walk.
    fn after_walk(deciding: Option<usize>) -> Self {
        match deciding {
            None => Self::Current(None),
            Some(node) => joined_cell(node).map_or(Self::Stale, |cell| Self::Current(Some(cell))),
        }
    }
    /// The deciding candidate the last walk found, or `None` when the state is
    /// stale.
    fn deciding(self) -> Option<Option<usize>> {
        match self {
            Self::Stale => None,
            Self::Current(None) => Some(None),
            Self::Current(Some(cell)) => cell_slot(cell).map(Some),
        }
    }
}
impl MetaHistory {
    /// The first unsatisfied operation recorded for `domain`; the table has one
    /// cell per `HtmlParseMetaDomain`.
    fn gap(&self, domain: HtmlParseMetaDomain) -> Option<usize> {
        self.gaps.copy(domain)
    }
    pub(super) fn offline() -> Self {
        Self {
            language: None,
            csp: None,
            base_csp: None,
            referrer: default_document_referrer_policy(),
            refresh: false,
            theme: None,
            scheme: None,
            gaps: MetaDomainTable::filled(None),
            first: None,
            // A new document holds no candidate yet, which is what a walk of
            // its empty tree would have learned.
            selections: MetaDomainTable::filled(SelectionIndex::Current(None)),
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum CancelPoint {
    Language,
    Gap,
    Theme,
    Scheme,
}
#[cfg(test)]
std::thread_local! {
    static CANCEL_AFTER_COMMIT: std::cell::RefCell<Option<(CancelPoint, usize, crate::HtmlParseCancellation)>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
fn after_commit(point: CancelPoint) {
    CANCEL_AFTER_COMMIT.with(|hook| {
        let mut hook = hook.borrow_mut();
        if let Some((expected, skip, _)) = hook.as_mut()
            && *expected == point
        {
            if *skip != 0 {
                *skip -= 1;
            } else if let Some((_, _, cancel)) = hook.take() {
                cancel.cancel();
            }
        }
    });
}
impl SessionStorage {
    pub(super) fn meta_history(&self, slot: usize) -> Result<&MetaHistory, SessionError> {
        let Some(RetainedPayload::Document { meta, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::registry_row());
        };
        Ok(meta)
    }
    pub(super) fn meta_history_mut(
        &mut self,
        slot: usize,
    ) -> Result<&mut MetaHistory, SessionError> {
        let Some(RetainedPayload::Document { meta, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(SessionError::registry_row());
        };
        Ok(meta)
    }
    pub(super) fn meta_policy_missing(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        let slot = self.document_slot(c, document)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self
            .meta_history(slot)
            .map_err(|_| latch_registry_row(c))?
            .gap(HtmlParseMetaDomain::Csp))
    }
    pub(super) fn meta_missing(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        subject: usize,
        kind: HtmlParseMetaMissingKind,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let domain = kind.domain();
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self
            .meta_history(slot)
            .map_err(|_| latch_registry_row(c))?
            .gap(domain)
            .is_some()
        {
            return Ok(());
        }
        let cause = self.associated_cause(c)?;
        let first = self
            .meta_history(slot)
            .map_err(|_| latch_registry_row(c))?
            .first
            .is_none();
        payload_write::<Option<usize>>(c, 1)?;
        if first {
            payload_write::<Option<HtmlParseMetaDomain>>(c, 1)?;
        }
        let prepared = self.prepare_payload_slot_owned(c)?;
        let gap = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::MetaMissing {
                    document,
                    subject,
                    kind,
                    cause,
                },
            )
            .map_err(|_| latch_registry_row(c))?;
        let meta = self
            .meta_history_mut(slot)
            .map_err(|_| latch_registry_row(c))?;
        *meta.gaps.get_mut(domain) = Some(gap);
        if first {
            meta.first = Some(domain);
        }
        #[cfg(test)]
        after_commit(CancelPoint::Gap);
        Ok(())
    }
    fn meta_document_tree(
        &self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let root = self.ordinary_root(c, node, detached)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            ConstructionNodeKind::Document(_)
        )
        .then_some(root))
    }
    /// Attribute backing is already owned; only this temporary inline handle is
    /// copied, with no new retained Arc payload or independent string ticket.
    fn meta_attribute(
        &self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<Option<DocumentString>, SessionError> {
        let raw = self.form_attribute(c, node, name)?;
        payload_write::<Option<DocumentString>>(c, 1)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        Ok(raw.cloned())
    }
    fn meta_range(
        c: &mut ParseControl,
        raw: &DocumentString,
        range: std::ops::Range<usize>,
    ) -> Result<FrozenUtf16, SessionError> {
        let units = raw
            .code_units()
            .get(range)
            .ok_or(SessionError::WrongToken)?;
        let mut output = Utf16Buffer::new(c)?;
        if let Err(stop) = output.extend_units(units, c) {
            output.release(c)?;
            return Err(stop.into());
        }
        Ok(output.freeze(c)?)
    }
    fn meta_pragma(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        document: usize,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let Some(raw) = self.meta_attribute(c, node, "http-equiv")? else {
            return Ok(());
        };
        let Some(pragma) =
            surgeist_html::bounded::attributes::meta_pragma_state_with(&raw, || {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)
            })?
        else {
            return Ok(());
        };
        if matches!(
            pragma,
            Pragma::EncodingDeclaration | Pragma::SetCookie | Pragma::XUaCompatible
        ) {
            // Decoded Unicode has no tentative encoding restart. These two
            // obsolete pragmas are explicitly ignored by the pinned source.
            return Ok(());
        }
        if pragma == Pragma::ContentSecurityPolicy {
            let Some(parent) = self.ordinary_parent(c, node, None)? else {
                return Ok(());
            };
            if !self.html_local(c, parent, "head")? {
                return Ok(());
            }
        }
        let Some(content) = self.meta_attribute(c, node, "content")? else {
            return Ok(());
        };
        match pragma {
            Pragma::ContentLanguage => {
                let Some(range) =
                    surgeist_html::bounded::attributes::meta_language_token_with(&content, || {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)
                    })?
                else {
                    return Ok(());
                };
                let value = Self::meta_range(c, &content, range)?;
                let (handle, value) = self.register_group_string_owned(c, value, None)?;
                drop(handle);
                let slot = self.document_slot(c, document)?;
                payload_write::<Option<usize>>(c, 1)?;
                let old = self
                    .meta_history_mut(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .language
                    .replace(value);
                #[cfg(test)]
                after_commit(CancelPoint::Language);
                if let Some(old) = old {
                    self.retire_registered_string(c, old)?;
                }
            }
            Pragma::Refresh if !content.is_empty() => {
                let slot = self.document_slot(c, document)?;
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if self
                    .meta_history(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .gap(HtmlParseMetaDomain::Refresh)
                    .is_some()
                    || self
                        .meta_history(slot)
                        .map_err(|_| latch_registry_row(c))?
                        .refresh
                {
                    return Ok(());
                }
                let target =
                    surgeist_html::bounded::attributes::meta_refresh_syntax_with(&content, |_| {
                        // DelayDigits retains the exact mathematical digit sequence;
                        // the selected Do-nothing branch never consumes a timer.
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)
                    })?;
                let outcome = match target {
                    None => return Ok(()),
                    Some(surgeist_html::bounded::attributes::HtmlMetaRefreshSyntaxTarget::DocumentUrl) => {
                        self.meta_document_url(c, document)?
                    }
                    Some(surgeist_html::bounded::attributes::HtmlMetaRefreshSyntaxTarget::UrlString(range)) => {
                        let units = content
                            .code_units()
                            .get(range)
                            .ok_or_else(|| latch_registry_row(c))?;
                        self.meta_refresh_url(c, node, units)?
                    }
                };
                match outcome {
                    Ok(false) => (),
                    Ok(true) => {
                        payload_write::<bool>(c, 1)?;
                        self.meta_history_mut(slot)
                            .map_err(|_| latch_registry_row(c))?
                            .refresh = true;
                    }
                    Err(missing) => self.meta_missing(
                        c,
                        document,
                        node,
                        HtmlParseMetaMissingKind::RefreshUrl(missing),
                    )?,
                }
            }
            Pragma::DefaultStyle if !content.is_empty() => {
                self.record_association(
                    c,
                    node,
                    pending_association::HtmlParseAssociationOccurrenceKind::ChangePreferred,
                    None,
                    None,
                    true,
                )?;
            }
            Pragma::ContentSecurityPolicy if !content.is_empty() => {
                if !self.defer_csp(c, node, document, content)? {
                    self.meta_missing(c, document, node, HtmlParseMetaMissingKind::Csp)?;
                } else {
                    self.service_meta_csp(
                        c,
                        service.as_deref_mut().ok_or(SessionError::WrongToken)?,
                    )?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    fn meta_document_url(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Result<bool, HtmlParseHyperlinkUnavailable>, SessionError> {
        for payload in &self.payloads.values {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::BaseDocument { node, facts, .. } = payload
                && *node == document
            {
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                return Ok(facts.document().document_url().map(|_| true).ok_or(
                    HtmlParseHyperlinkUnavailable::Base(crate::HtmlBaseMissing::DocumentUrl),
                ));
            }
        }
        Ok(Err(HtmlParseHyperlinkUnavailable::Base(
            crate::HtmlBaseMissing::DocumentFacts,
        )))
    }
    fn meta_refresh_url(
        &self,
        c: &mut ParseControl,
        node: usize,
        raw: &[u16],
    ) -> Result<Result<bool, HtmlParseHyperlinkUnavailable>, SessionError> {
        let context = match self.encoding_url_context(c, node)? {
            Ok(value) => value,
            Err(missing) => return Ok(Err(missing)),
        };
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        match context.facts.lookup_units_with(
            raw,
            context.base,
            context.encoding,
            context.expected,
            || associated::encoding_url_step(c),
        ) {
            Ok(surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Failure) => Ok(Ok(false)),
            Ok(surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Parsed(record)) => {
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                Ok(Ok(record.scheme() != crate::HtmlBaseUrlScheme::Javascript))
            }
            Err(surgeist_html_semantics::HtmlEncodingUrlLookupError::Facts(error)) => {
                Ok(Err(HtmlParseHyperlinkUnavailable::Observation(error)))
            }
            Err(surgeist_html_semantics::HtmlEncodingUrlLookupError::Interrupted(stop)) => {
                Err(stop.into())
            }
        }
    }
    fn meta_referrer(&mut self, c: &mut ParseControl, node: usize) -> Result<(), SessionError> {
        let Some(document) = self.meta_document_tree(c, node, None)? else {
            return Ok(());
        };
        let Some(name) = self.meta_attribute(c, node, "name")? else {
            return Ok(());
        };
        if !meta_ascii(c, &name, "referrer")? {
            return Ok(());
        }
        let Some(content) = self.meta_attribute(c, node, "content")? else {
            return Ok(());
        };
        if let Some(policy) =
            surgeist_html::bounded::attributes::meta_referrer_policy_with(&content, || {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)
            })?
        {
            let slot = self.document_slot(c, document)?;
            payload_write::<HtmlReferrerPolicy>(c, 1)?;
            self.meta_history_mut(slot)
                .map_err(|_| latch_registry_row(c))?
                .referrer = policy;
        }
        Ok(())
    }
    /// Walks the document tree for each selection in `theme` and `scheme` that
    /// can still change, reads the candidates in tree order up to the first
    /// that decides something and stores that decision.
    fn meta_candidates(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        detached: Option<usize>,
        theme: bool,
        scheme: bool,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        for (enabled, domain) in [
            (theme, HtmlParseMetaDomain::ThemeColor),
            (scheme, HtmlParseMetaDomain::ColorScheme),
        ] {
            if !enabled
                || self
                    .meta_history(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .gap(domain)
                    .is_some()
            {
                continue;
            }
            let mut decided = Decision::Nothing;
            let mut next = Some(document);
            while let Some(node) = next {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(reading) = self.read_candidate(c, document, node, domain)? {
                    decided = self.decide(c, document, node, reading)?;
                    if !matches!(decided, Decision::Nothing) {
                        break;
                    }
                }
                next = self.next_view_node(c, document, node, detached)?;
            }
            self.store_decision(c, slot, domain, decided)?;
        }
        Ok(())
    }
    /// What `node` contributes to the selection `domain`, or `None` when it is
    /// no candidate for it: an HTML `meta` with the exact name and a `content`
    /// attribute.
    fn read_candidate(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        node: usize,
        domain: HtmlParseMetaDomain,
    ) -> Result<Option<Reading>, SessionError> {
        let expected = if domain == HtmlParseMetaDomain::ThemeColor {
            "theme-color"
        } else {
            "color-scheme"
        };
        if !self.html_local(c, node, "meta")? {
            return Ok(None);
        }
        let Some(name) = self.meta_attribute(c, node, "name")? else {
            return Ok(None);
        };
        if !meta_ascii(c, &name, expected)? {
            return Ok(None);
        }
        let Some(content) = self.meta_attribute(c, node, "content")? else {
            return Ok(None);
        };
        if domain == HtmlParseMetaDomain::ThemeColor {
            if let Some(media) = self.meta_attribute(c, node, "media")?
                && !matches_without_environment(c, media.code_units())?
            {
                return Ok(Some(Reading::Unsatisfied(
                    HtmlParseMetaMissingKind::ThemeColor,
                )));
            }
            return Ok(Some(match theme_color(c, content.code_units())? {
                Ok(Some(color)) => Reading::Theme(color),
                Ok(None) => Reading::Rejected,
                Err(error) => {
                    Reading::Unsatisfied(HtmlParseMetaMissingKind::ThemeColorProcessing(error))
                }
            }));
        }
        let ConstructionNodeKind::Document(mode) = self
            .construction
            .node_at(document)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::NotDocument);
        };
        let candidate = c.parse_meta_color_scheme(content.code_units(), mode)?;
        if let Err(stop) = c.charge(HtmlParseWorkCategory::ProviderCall, 1) {
            // The stop is latched; a refusal here would be a later one.
            let _ = candidate.release(c);
            return Err(stop.into());
        }
        if candidate.report().result().is_ok() {
            return Ok(Some(Reading::Scheme(candidate)));
        }
        candidate.release(c)?;
        Ok(Some(Reading::Rejected))
    }
    /// Turns a reading of `node` into what the walk does with it. A reading
    /// that cannot be satisfied is recorded as the domain's gap at once.
    fn decide(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        node: usize,
        reading: Reading,
    ) -> Result<Decision, SessionError> {
        Ok(match reading {
            Reading::Rejected => Decision::Nothing,
            Reading::Theme(color) => Decision::Theme(node, color),
            Reading::Scheme(value) => Decision::Scheme(node, value),
            Reading::Unsatisfied(kind) => {
                self.meta_missing(c, document, node, kind)?;
                Decision::Unsatisfied
            }
        })
    }
    /// Stores what a walk or a single reading decided for `domain` and notes
    /// which candidate decided it.
    fn store_decision(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        domain: HtmlParseMetaDomain,
        decided: Decision,
    ) -> Result<(), SessionError> {
        let deciding = match &decided {
            Decision::Theme(node, _) | Decision::Scheme(node, _) => Some(*node),
            Decision::Nothing | Decision::Unsatisfied => None,
        };
        if domain == HtmlParseMetaDomain::ThemeColor {
            payload_write::<Option<surgeist_color::Color>>(c, 1)?;
            let meta = self
                .meta_history_mut(slot)
                .map_err(|_| latch_registry_row(c))?;
            meta.theme = match decided {
                Decision::Theme(_, color) => Some(color),
                _ => None,
            };
            *meta.selections.get_mut(domain) = SelectionIndex::after_walk(deciding);
            // This original constructor has no associated UI recipients.
            // Selection above still performed all reached CSS work.
            #[cfg(test)]
            if self
                .meta_history(slot)
                .map_err(|_| latch_registry_row(c))?
                .theme
                .is_some()
            {
                after_commit(CancelPoint::Theme);
            }
            return Ok(());
        }
        let selected_scheme = match decided {
            Decision::Scheme(_, value) => Some(value),
            _ => None,
        };
        // Preserve the admitted new graph on a refused field write.
        if let Err(stop) = payload_write::<Option<crate::parser::control::CssSchemeValue>>(c, 1) {
            if let Some(candidate) = selected_scheme {
                // The stop is latched; a refusal here would be a later one.
                let _ = candidate.release(c);
            }
            return Err(stop);
        }
        let meta = self
            .meta_history_mut(slot)
            .map_err(|_| latch_registry_row(c))?;
        *meta.selections.get_mut(domain) = SelectionIndex::after_walk(deciding);
        let old = std::mem::replace(&mut meta.scheme, selected_scheme);
        #[cfg(test)]
        if self
            .meta_history(slot)
            .map_err(|_| latch_registry_row(c))?
            .scheme
            .is_some()
        {
            after_commit(CancelPoint::Scheme);
        }
        if let Some(old) = old {
            old.release(c)?;
        }
        Ok(())
    }
    /// `effects` is the insertion or removal of the subtree that holds `node`.
    pub(super) fn meta_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        effects: &FormEffects,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
        let removal = effects.stage == FormEffectStage::Removal;
        let (old_parent, root) = (effects.old_parent, effects.root);
        if !self.html_local(c, node, "meta")? {
            return Ok(());
        }
        // Physical staging may already put the moved root in another tree.
        // The retained original parent identifies the actual old ordinary tree;
        // candidate traversal excludes the virtually detached removal root.
        let anchor = if removal { old_parent } else { Some(node) };
        let Some(anchor) = anchor else {
            return Ok(());
        };
        let Some(document) = self.meta_document_tree(c, anchor, None)? else {
            return Ok(());
        };
        if !removal {
            self.meta_pragma(c, node, document, service)?;
            self.meta_referrer(c, node)?;
        }
        // Inserting or removing a meta that is no candidate for a selection
        // changes nothing the selection depends on, so that walk is not run.
        let theme = self.meta_candidate(c, node, "theme-color")?;
        let scheme = self.meta_candidate(c, node, "color-scheme")?;
        if !theme && !scheme {
            return Ok(());
        }
        // A meta that is the whole subtree changes one candidate. A larger
        // subtree may bring or take other candidates the hooks of its other
        // nodes have not reached, which only a walk of the tree sees.
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let alone = node == root
            && self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .child_count()
                == 0;
        if !alone {
            return self.meta_walk_subtree(c, document, detached, effects, theme, scheme);
        }
        let effect = if removal {
            Change::Removed
        } else {
            Change::Candidate
        };
        for (enabled, domain) in [
            (theme, HtmlParseMetaDomain::ThemeColor),
            (scheme, HtmlParseMetaDomain::ColorScheme),
        ] {
            if enabled {
                self.meta_settle(c, document, node, detached, domain, effect)?;
                #[cfg(test)]
                self.assert_selection_matches_walk(c, document, detached, domain)?;
            }
        }
        Ok(())
    }
    /// The oracle for [`Self::meta_settle`] in test builds: a walk of the tree
    /// reads the candidates in tree order, and the first one that decides
    /// something must be the one the selection recorded.
    #[cfg(test)]
    fn assert_selection_matches_walk(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        detached: Option<usize>,
        domain: HtmlParseMetaDomain,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let history = self.meta_history(slot).map_err(|_| latch_registry_row(c))?;
        let Some(deciding) = history.selections.copy(domain).deciding() else {
            return Ok(());
        };
        if history.gap(domain).is_some() {
            return Ok(());
        }
        let mut found = None;
        let mut next = Some(document);
        while let Some(node) = next {
            match self.read_candidate(c, document, node, domain)? {
                None | Some(Reading::Rejected) => {}
                Some(Reading::Theme(_)) => {
                    found = Some(node);
                    break;
                }
                Some(Reading::Scheme(value)) => {
                    value.release(c)?;
                    found = Some(node);
                    break;
                }
                Some(Reading::Unsatisfied(kind)) => {
                    panic!(
                        "{domain:?}: {kind:?} at record {node} was reached but no gap is recorded"
                    )
                }
            }
            next = self.next_view_node(c, document, node, detached)?;
        }
        assert_eq!(
            deciding, found,
            "{domain:?}: the recorded deciding candidate differs from the walk's"
        );
        Ok(())
    }
    /// Settles the selections for one candidate of a larger subtree. The first
    /// candidate's hook walks the tree, and that walk sees the whole subtree
    /// as it now stands, so the hooks of the other candidates of the same
    /// insertion or removal find the selection exact and walk nothing.
    fn meta_walk_subtree(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        detached: Option<usize>,
        effects: &FormEffects,
        theme: bool,
        scheme: bool,
    ) -> Result<(), SessionError> {
        let batch = self
            .payloads
            .topology_changes
            .map(|topology_changes| MetaBatch {
                root: effects.root,
                removal: effects.stage == FormEffectStage::Removal,
                detached,
                topology_changes,
            });
        let slot = self.document_slot(c, document)?;
        for (enabled, domain) in [
            (theme, HtmlParseMetaDomain::ThemeColor),
            (scheme, HtmlParseMetaDomain::ColorScheme),
        ] {
            if !enabled {
                continue;
            }
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let settled = batch.is_some()
                && self.meta_batches.copy(domain) == batch
                && self
                    .meta_history(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .selections
                    .copy(domain)
                    .deciding()
                    .is_some();
            if !settled {
                self.meta_candidates(
                    c,
                    document,
                    detached,
                    domain == HtmlParseMetaDomain::ThemeColor,
                    domain == HtmlParseMetaDomain::ColorScheme,
                )?;
                *self.meta_batches.get_mut(domain) = batch;
            }
            #[cfg(test)]
            self.assert_selection_matches_walk(c, document, detached, domain)?;
        }
        Ok(())
    }
    /// Brings the selection `domain` up to date with a change to `node`, the
    /// only candidate whose state changed, reading it only if the selection
    /// can depend on it.
    fn meta_settle(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        node: usize,
        detached: Option<usize>,
        domain: HtmlParseMetaDomain,
        change: Change,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let history = self.meta_history(slot).map_err(|_| latch_registry_row(c))?;
        if history.gap(domain).is_some() {
            return Ok(());
        }
        let known = history.selections.copy(domain);
        let (theme, scheme) = (
            domain == HtmlParseMetaDomain::ThemeColor,
            domain == HtmlParseMetaDomain::ColorScheme,
        );
        let Some(deciding) = known.deciding() else {
            return self.meta_candidates(c, document, detached, theme, scheme);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let step = if deciding == Some(node) {
            // The change may end or replace the decision: only a walk knows
            // the candidate that decides next.
            Step::Walk
        } else {
            match (change, deciding) {
                (Change::Removed, _) => Step::Unchanged,
                (Change::Candidate, None) => Step::Read,
                (Change::Candidate, Some(deciding)) => {
                    if self.ordinary_root(c, deciding, None)? != document {
                        // A decision that left the tree is not one to compare with.
                        Step::Walk
                    } else if self.precedes(c, node, deciding)? {
                        Step::Read
                    } else {
                        Step::Unchanged
                    }
                }
            }
        };
        match step {
            Step::Unchanged => Ok(()),
            Step::Walk => self.meta_candidates(c, document, detached, theme, scheme),
            Step::Read => {
                let Some(reading) = self.read_candidate(c, document, node, domain)? else {
                    return Ok(());
                };
                let decided = self.decide(c, document, node, reading)?;
                if matches!(decided, Decision::Nothing) {
                    return Ok(());
                }
                self.store_decision(c, slot, domain, decided)
            }
        }
    }
    /// Whether `first` comes before `second` in tree order. Both are in one
    /// tree and are different records.
    ///
    /// Tree order is defined by the pinned algorithms that ask for it (the
    /// selections of a theme color and of a color scheme take their candidates
    /// "in tree order"), and two records in different branches are ordered by
    /// the children of their lowest common ancestor, so the levels between each
    /// record and that ancestor are looked at. A look is one read of a record's
    /// parent. With `a` and `b` the levels from the two records up to their
    /// lowest common ancestor:
    ///
    /// * Siblings cost 2 looks.
    /// * When the ancestor is within 24 levels of both records, they climb in
    ///   step, testing each level against the other's, and the climb ends at
    ///   it: 2 looks for each of the greater of `a` and `b`, plus the 2 looks
    ///   at the parents.
    /// * When one record reaches the root first, at depth `s`, it stops, and
    ///   the other goes on alone: `s` looks in step on each side, then 1 look
    ///   for each level the other still climbs to the ancestor, so about
    ///   `s + b` for the other's `b` levels. A shallow record against a deep
    ///   one costs about the depth of the deep one.
    /// * When neither record has reached the root or the ancestor after 24
    ///   levels, the climb is done again by depths: 48 looks wasted, then both
    ///   records are climbed to the root (their two depths), the deeper is
    ///   lifted to the other's depth and both climb to the ancestor. Two deep
    ///   records whose ancestor is far up cost about 4 looks for each level of
    ///   their depth, 2 to learn the depths and 2 to meet.
    pub(super) fn precedes(
        &self,
        c: &mut ParseControl,
        first: usize,
        second: usize,
    ) -> Result<bool, SessionError> {
        let storage = &self.construction;
        let parent = |c: &mut ParseControl, node: usize| -> Result<Option<usize>, SessionError> {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            Ok(storage
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .parent()
                .map(|parent| parent.position()))
        };
        // Siblings are the usual pair: no ancestor needs to be found.
        let first_parent = parent(c, first)?;
        let (left, right, above) = if first_parent.is_some() && first_parent == parent(c, second)? {
            (first, second, first_parent)
        } else {
            match branches(c, &parent, first, second, storage.len())? {
                Lineage::FirstEncloses => return Ok(true),
                Lineage::SecondEncloses => return Ok(false),
                Lineage::Branches {
                    above,
                    first,
                    second,
                } => (first, second, Some(above)),
            }
        };
        let above = above.ok_or(SessionError::Cycle)?;
        // The later sibling is the one a walk back from the end of the child
        // order reaches first.
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let siblings = storage.node_at(above).ok_or(SessionError::ForeignNode)?;
        for ordinal in (0..siblings.child_count()).rev() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let sibling = siblings
                .child_at(ordinal)
                .ok_or_else(|| latch_registry_row(c))?
                .position();
            if sibling == right {
                return Ok(true);
            }
            if sibling == left {
                return Ok(false);
            }
        }
        Err(SessionError::WrongSibling)
    }
    /// Whether `meta_candidates` takes `node` as a candidate for the selection
    /// named `expected`: the exact `name` and some `content` attribute. Whether
    /// its content then selects anything is for that walk to decide.
    fn meta_candidate(
        &self,
        c: &mut ParseControl,
        node: usize,
        expected: &str,
    ) -> Result<bool, SessionError> {
        let Some(name) = self.form_attribute(c, node, "name")? else {
            return Ok(false);
        };
        if !meta_ascii(c, name, expected)? {
            return Ok(false);
        }
        Ok(self.form_attribute(c, node, "content")?.is_some())
    }
    pub(super) fn initial_meta_attributes(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "meta")? {
            return Ok(());
        }
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let ConstructionNodeKind::Element(element) = record.kind() else {
            return Ok(());
        };
        let len = element.attributes().len();
        for position in 0..len {
            self.initial_meta_attribute(c, node, position)?;
        }
        Ok(())
    }
    pub(super) fn initial_meta_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "meta")? {
            return Ok(());
        }
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let ConstructionNodeKind::Element(element) = record.kind() else {
            return Ok(());
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let attribute = element
            .attributes()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        if attribute.name().namespace().uri().is_some() {
            return Ok(());
        }
        for name in ["name", "content", "media"] {
            if exact_ascii(c, Some(attribute.name().local_name()), name)? {
                return self.meta_attribute_changed(c, node, name);
            }
        }
        Ok(())
    }
    pub(super) fn meta_attribute_changed(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "meta")? {
            return Ok(());
        }
        let name_or_content = matches!(local, "name" | "content");
        if name_or_content {
            self.meta_referrer(c, node)?;
        }
        if name_or_content || local == "media" {
            let document = match self.node_document_ref(c, node)? {
                DocumentRef::Owned(node) => node,
                DocumentRef::Captured(_) => return Ok(()),
            };
            // The selection walks the document's own tree. A meta outside it,
            // such as one still being created, is no candidate yet, and its
            // insertion will recompute the selection.
            if self.ordinary_root(c, node, None)? != document {
                return Ok(());
            }
            // This hook runs for an element being created or cloned, which is in
            // no tree yet, so it returns above and its insertion decides the
            // selection. Should an attached meta ever reach it, the tree is
            // walked: nothing here knows which candidate the change concerns.
            self.meta_candidates(c, document, None, true, name_or_content)?;
        }
        Ok(())
    }
}
/// HTML selects the ASCII-whitespace subrange; CSS owns preprocessing, grammar
/// and used-color resolution. A syntax rejection has no input fallback.
fn theme_color(
    c: &mut ParseControl,
    mut input: &[u16],
) -> Result<
    Result<Option<surgeist_color::Color>, crate::parser::control::HtmlColorProcessingError>,
    SessionError,
> {
    while let Some((unit, rest)) = input.split_first() {
        c.charge_unit_inspection()?;
        if !surgeist_html::bounded::lexical::is_ascii_whitespace(*unit) {
            break;
        }
        input = rest;
    }
    while let Some((unit, rest)) = input.split_last() {
        c.charge_unit_inspection()?;
        if !surgeist_html::bounded::lexical::is_ascii_whitespace(*unit) {
            break;
        }
        input = rest;
    }
    let mut owner = surgeist_css::color::controlled::CssColorOperationOwner::new();
    let facts = surgeist_css::color::CssColorFacts::new(
        surgeist_css::color::CssColorFactIdentity::new(0x5041_5253_4d45_5441, 1),
        surgeist_css::color::CssColorPropertyFacts::InitialProperties,
        surgeist_css::color::CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        surgeist_css::color::CssDeviceColorFact::Unknown,
    );
    match css_color::parse(
        c,
        input,
        &mut owner,
        &facts,
        css_color::source_processing,
        0,
        #[cfg(test)]
        || {},
    ) {
        Ok(surgeist_css::color::controlled::CssColorParseOutcome::Used(color)) => {
            Ok(Ok(Some(color.value())))
        }
        Ok(surgeist_css::color::controlled::CssColorParseOutcome::StandardFailure(_)) => {
            Ok(Ok(None))
        }
        Err(css_color::Failure::Session(error)) => Err(error),
        Err(css_color::Failure::Processing(error)) => Ok(Err(error)),
    }
}
/// Whether a `media` value matches the environment whatever the environment
/// is. HTML "matches the environment" holds for the empty string and for
/// ASCII whitespace only; otherwise the value is a media query list, and the
/// media type `all` matches all devices (Media Queries 4, media types), ASCII
/// case-insensitively and ignoring surrounding whitespace. Every other query
/// depends on an environment this parser does not have.
fn matches_without_environment(c: &mut ParseControl, media: &[u16]) -> Result<bool, SessionError> {
    const ALL: &[u8; 3] = b"all";
    let mut matched = 0;
    let mut trailing = false;
    for &unit in media {
        c.charge_unit_inspection()?;
        if surgeist_html::bounded::lexical::is_ascii_whitespace(unit) {
            trailing = trailing || matched != 0;
            continue;
        }
        let Some(&expected) = ALL.get(matched) else {
            return Ok(false);
        };
        if trailing || !surgeist_html::bounded::lexical::ascii_case_insensitive_unit(unit, expected)
        {
            return Ok(false);
        }
        matched = matched.saturating_add(1);
    }
    // Nothing but whitespace, or exactly `all`.
    Ok(matched == 0 || matched == ALL.len())
}
fn meta_ascii(
    c: &mut ParseControl,
    value: &DocumentString,
    expected: &str,
) -> Result<bool, SessionError> {
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (unit, byte) in value.code_units().iter().zip(expected.bytes()) {
        c.charge_unit_inspection()?;
        if !surgeist_html::bounded::lexical::ascii_case_insensitive_unit(*unit, byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::{construction::HtmlDocumentMode, values::DocumentString};
    #[derive(Debug)]
    struct RedPolicy;
    impl surgeist_html::security::csp::HtmlCspPolicyArtifact for RedPolicy {
        fn retained_layout(&self) -> surgeist_html::security::csp::HtmlCspArtifactLayout {
            surgeist_html::security::csp::HtmlCspArtifactLayout::default()
        }
    }
    struct RedProcessor {
        authority: surgeist_html::security::csp::HtmlCspProcessorAuthority,
        profile: surgeist_html::security::csp::HtmlCspProcessorProfile,
        refuse: bool,
        calls: usize,
        completed: usize,
    }
    impl surgeist_html::security::csp::HtmlMetaCspProcessor for RedProcessor {
        type Error = ();
        fn profile(&self) -> &surgeist_html::security::csp::HtmlCspProcessorProfile {
            &self.profile
        }
        fn prepare<E>(
            &mut self,
            request: &surgeist_html::security::csp::HtmlMetaCspRequest,
            admit: &mut impl FnMut(surgeist_html::security::csp::HtmlCspLayout) -> Result<(), E>,
            release: &mut impl FnMut(surgeist_html::security::csp::HtmlCspStorage),
        ) -> Result<
            surgeist_html::security::csp::HtmlMetaCspOutcome<()>,
            surgeist_html::security::csp::HtmlCspError<E>,
        > {
            use surgeist_html::security::csp::{HtmlCspError, HtmlCspLayout, HtmlMetaCspOutcome};
            self.profile
                .validate_selected_with(request.qualification().processor(), admit)?;
            admit(HtmlCspLayout::processing(
                3,
                Default::default(),
                Default::default(),
                0,
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            assert_eq!(
                request.qualification().content().code_units(),
                [114, 101, 100]
            );
            self.calls += 1;
            if self.refuse {
                return Ok(HtmlMetaCspOutcome::Refused(()));
            }
            // This closed adapter recognizes exactly the request-derived red
            // fixture policy, independently of the later CSS color parser.
            let layout = surgeist_html::security::csp::HtmlCspArtifactLayout::default();
            admit(HtmlCspLayout::processing(
                1,
                layout.storage(),
                layout.disposal(),
                0,
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            let policy = std::sync::Arc::new(RedPolicy);
            match self.authority.attest_changed_with(request, policy, admit) {
                Ok(p) => {
                    self.completed += 1;
                    Ok(HtmlMetaCspOutcome::Completed(p))
                }
                Err(e) => {
                    drop(e.retained);
                    release(layout.storage());
                    Err(e.error)
                }
            }
        }
    }
    #[test]
    fn csp_result_precedes_dual_meta_theme_cancellation_and_later_tail_stop() {
        use surgeist_html::security::csp::{HtmlCspLayout, HtmlCspProcessorAuthority};
        for refuse in [true, false] {
            let mut allow = |_: HtmlCspLayout| Ok::<_, std::convert::Infallible>(());
            let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
            let profile = authority.profile_with(1, &mut allow).unwrap();
            let mut processor = RedProcessor {
                authority,
                profile,
                refuse,
                calls: 0,
                completed: 0,
            };
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(
                    "<!doctype html><meta http-equiv=content-security-policy name=theme-color content=red>",
                ),
                super::super::tests::limits(),
            );
            CANCEL_AFTER_COMMIT.with(|hook| {
                *hook.borrow_mut() = Some((CancelPoint::Theme, 0, request.cancellation().clone()))
            });
            let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
            assert_eq!(processor.calls, 1);
            assert_eq!(processor.completed, usize::from(!refuse));
            assert_eq!(failure.parse_failure().cleanup_error(), None);
            if refuse {
                assert_eq!(failure.kind(), crate::HtmlParseCspFailureKind::Refused);
                assert_eq!(
                    failure.parse_failure().first_stop(),
                    Some(HtmlParseStop::CspProcessing)
                );
                assert_eq!(
                    failure.parse_failure().reason(),
                    crate::HtmlParseFailureReason::CspProcessing
                );
                CANCEL_AFTER_COMMIT.with(|hook| assert!(hook.borrow_mut().take().is_some()));
            } else {
                assert_eq!(
                    failure.parse_failure().first_stop(),
                    Some(HtmlParseStop::Cancelled)
                );
                CANCEL_AFTER_COMMIT.with(|hook| assert!(hook.borrow().is_none()));
            }
        }
    }

    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};

    fn next_node(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
    ) -> ParserNode {
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    if let TokenKind::Tag(tag) = token.kind {
                        return session.allocate_html_tag(c, &mut Some(tag)).unwrap();
                    }
                    release_token_kind(&mut Some(token.kind), c).unwrap();
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    #[test]
    fn language_replacement_admits_both_real_backings_before_releasing_prior_value() {
        // Actual managed fields before the hook: Meta(4), XHTML URI(28),
        // http-equiv(10), content-language(16), content(7), en(2) = 67 units.
        // The current token adds 2. Reprocessing requires the previous and new
        // 2-unit tokens together: 67+2+2=71, then releases the old token to 69.
        for cap in [70, 71] {
            let limits = super::super::tests::limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap();
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8("<meta http-equiv=content-language content=en>"),
                limits,
            );
            let mut c = ParseControl::new(limits, request.cancellation().clone());
            c.admit_source(request.source()).unwrap();
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            let node = next_node(&mut session, &mut c, &mut lexer);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current(),
                67
            );
            session
                .active_mut()
                .unwrap()
                .insert(&mut c, &root, &node, None, InsertionCause::NonParser)
                .unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current(),
                69
            );
            session.active_mut().unwrap().remove(&mut c, &node).unwrap();
            let result = session.active_mut().unwrap().insert(
                &mut c,
                &root,
                &node,
                None,
                InsertionCause::NonParser,
            );
            if cap == 70 {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedUtf16Units,
                        current: 69,
                        requested_delta: 2,
                        ..
                    }))
                ));
            } else {
                result.unwrap();
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current(),
                    69
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .high_water(),
                    71
                );
            }
            let storage = session.active().unwrap();
            let slot = storage
                .payloads
                .values
                .iter()
                .find_map(|p| match p {
                    RetainedPayload::Document { meta, .. } => meta.language,
                    _ => None,
                })
                .unwrap();
            let RetainedPayload::String { value, .. } = &storage.payloads.values[slot] else {
                panic!("actual current language backing")
            };
            assert_eq!(value, &DocumentString::from("en"));
            let first = c.stop();
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), first);
            super::super::tests::storage_zero(&c);
        }
    }
    /// Metas built detached from `source`, one per start tag, with the session
    /// that holds them and the document root. Every change a test makes to the
    /// tree runs the walk oracle of `meta_tree_hook`.
    fn detached_metas(
        source: &str,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ParserNode,
        Vec<ParserNode>,
        LexerFixture<'static>,
    ) {
        let source: &'static crate::HtmlSource =
            Box::leak(Box::new(crate::HtmlSource::utf8(source)));
        // The walk oracle reads every candidate after every change.
        let limits = super::super::tests::limits()
            .into_builder()
            .work_credits(100_000_000_000)
            .source_units(1_000_000)
            .buffered_utf16_units(1_000_000)
            .managed_utf16_units(1_000_000)
            .managed_capacity_bytes(1_000_000_000)
            .peak_transient_bytes(1_000_000_000)
            .atomic_layout_bytes(1_000_000_000)
            .source_map_entries(1_000_000)
            .source_map_payload_units(1_000_000)
            .managed_nodes(1_000_000)
            .managed_edges(1_000_000)
            .auxiliary_entries(100_000_000)
            .active_formatting_entries(100_000_000)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        c.admit_source(source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut nodes = Vec::new();
        for _ in 0..source.len() {
            // One node for each start tag; the loop ends when the source does.
            let Some(node) = try_next_node(&mut session, &mut c, &mut lexer) else {
                break;
            };
            nodes.push(node);
        }
        (c, session, root, nodes, lexer)
    }
    fn try_next_node(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
    ) -> Option<ParserNode> {
        loop {
            match lexer.next(c, false).unwrap().unwrap() {
                Event::Token(event) => {
                    let token = event.into_token(c).unwrap();
                    match token.kind {
                        TokenKind::Tag(tag) => {
                            return Some(session.allocate_html_tag(c, &mut Some(tag)).unwrap());
                        }
                        TokenKind::Eof => return None,
                        kind => release_token_kind(&mut Some(kind), c).unwrap(),
                    }
                }
                event => event.release(c).unwrap(),
            }
        }
    }
    fn history(session: &ParserArenaSession) -> &MetaHistory {
        session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|row| match row {
                RetainedPayload::Document { meta, .. } => Some(meta),
                _ => None,
            })
            .unwrap()
    }
    fn theme(session: &ParserArenaSession) -> Option<[u8; 3]> {
        history(session).theme.and_then(|color| color.srgb_bytes())
    }
    fn scheme(session: &ParserArenaSession) -> Option<Vec<u16>> {
        history(session).scheme.as_ref().map(|value| {
            value
                .report()
                .source()
                .source()
                .original_utf16()
                .unwrap()
                .as_slice()
                .to_vec()
        })
    }
    fn put(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        parent: &ParserNode,
        node: &ParserNode,
        before: Option<&ParserNode>,
    ) {
        session
            .active_mut()
            .unwrap()
            .insert(c, parent, node, before, InsertionCause::NonParser)
            .unwrap();
    }
    fn take(session: &mut ParserArenaSession, c: &mut ParseControl, node: &ParserNode) {
        session.active_mut().unwrap().remove(c, node).unwrap();
    }
    const RED: Option<[u8; 3]> = Some([255, 0, 0]);
    const BLUE: Option<[u8; 3]> = Some([0, 0, 255]);
    const GREEN: Option<[u8; 3]> = Some([0, 128, 0]);

    #[test]
    fn theme_color_follows_the_first_color_in_tree_order_through_every_kind_of_change() {
        let (mut c, mut session, root, nodes, lexer) = detached_metas(
            "<meta name=theme-color content=bogus><meta name=theme-color content=red>\
             <meta name=theme-color content=blue><meta name=theme-color content=green>\
             <meta name=viewport content=x><div>",
        );
        let [bogus, red, blue, green, viewport, div] = &nodes[..] else {
            panic!("six elements")
        };
        // The tree order after each change decides the color; each expected
        // color is the first listed that names one.
        put(&mut session, &mut c, &root, blue, None); // [blue]
        assert_eq!(theme(&session), BLUE);
        put(&mut session, &mut c, &root, green, None); // [blue, green]
        assert_eq!(theme(&session), BLUE);
        put(&mut session, &mut c, &root, bogus, None); // [blue, green, bogus]
        assert_eq!(theme(&session), BLUE);
        put(&mut session, &mut c, &root, red, Some(blue)); // [red, blue, green, bogus]
        assert_eq!(theme(&session), RED);
        take(&mut session, &mut c, red); // [blue, green, bogus]
        assert_eq!(theme(&session), BLUE);
        put(&mut session, &mut c, &root, red, Some(green)); // [blue, red, green, bogus]
        assert_eq!(theme(&session), BLUE);
        put(&mut session, &mut c, &root, blue, None); // moves: [red, green, bogus, blue]
        assert_eq!(theme(&session), RED);
        take(&mut session, &mut c, red); // [green, bogus, blue]
        assert_eq!(theme(&session), GREEN);
        put(&mut session, &mut c, &root, viewport, Some(green)); // not a candidate
        assert_eq!(theme(&session), GREEN);
        take(&mut session, &mut c, green); // [viewport, bogus, blue]
        assert_eq!(theme(&session), BLUE);
        take(&mut session, &mut c, blue); // [viewport, bogus]
        assert_eq!(theme(&session), None);
        put(&mut session, &mut c, &root, bogus, Some(viewport)); // moves: [bogus, viewport]
        assert_eq!(theme(&session), None);
        // A subtree that holds candidates arrives and leaves as one change.
        put(&mut session, &mut c, div, green, None);
        put(&mut session, &mut c, div, red, Some(green)); // div: [red, green]
        assert_eq!(theme(&session), None);
        put(&mut session, &mut c, &root, div, Some(bogus)); // [div[red, green], bogus, viewport]
        assert_eq!(theme(&session), RED);
        put(&mut session, &mut c, &root, blue, Some(div)); // [blue, div[red, green], ...]
        assert_eq!(theme(&session), BLUE);
        take(&mut session, &mut c, blue);
        assert_eq!(theme(&session), RED);
        take(&mut session, &mut c, div); // [bogus, viewport]
        assert_eq!(theme(&session), None);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn color_scheme_follows_the_first_scheme_in_tree_order_through_every_kind_of_change() {
        let utf16 = |text: &str| Some(text.encode_utf16().collect::<Vec<_>>());
        let (mut c, mut session, root, nodes, lexer) = detached_metas(
            "<meta name=color-scheme content=1><meta name=color-scheme content=dark>\
             <meta name=color-scheme content=light>",
        );
        let [bogus, dark, light] = &nodes[..] else {
            panic!("three elements")
        };
        put(&mut session, &mut c, &root, bogus, None); // [bogus]
        assert_eq!(scheme(&session), None);
        put(&mut session, &mut c, &root, light, None); // [bogus, light]
        assert_eq!(scheme(&session), utf16("light"));
        put(&mut session, &mut c, &root, dark, Some(light)); // [bogus, dark, light]
        assert_eq!(scheme(&session), utf16("dark"));
        take(&mut session, &mut c, dark); // [bogus, light]
        assert_eq!(scheme(&session), utf16("light"));
        // The mode does not change how a scheme reads, so nothing learned
        // before it is lost.
        session
            .set_document_mode(&mut c, HtmlDocumentMode::Quirks)
            .unwrap();
        put(&mut session, &mut c, &root, dark, None); // [bogus, light, dark]
        assert_eq!(scheme(&session), utf16("light"));
        put(&mut session, &mut c, &root, dark, Some(bogus)); // moves: [dark, bogus, light]
        assert_eq!(scheme(&session), utf16("dark"));
        take(&mut session, &mut c, dark);
        take(&mut session, &mut c, light);
        assert_eq!(scheme(&session), None);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_subtree_inserted_again_walks_although_its_last_insertion_did() {
        // The subtree S is inserted with two candidates and walked. Both
        // candidates leave it one at a time, which settles the selection, and S
        // leaves the tree holding none, which no candidate's hook observes. The
        // candidates return while S is detached, in the other order, and S is
        // inserted again with the same root, stage and detached root as the
        // first time. Only the count of changes tells the two insertions apart.
        let (mut c, mut session, root, nodes, lexer) = detached_metas(
            "<meta name=theme-color content=red><meta name=theme-color content=blue><div>",
        );
        let [red, blue, subtree] = &nodes[..] else {
            panic!("three elements")
        };
        put(&mut session, &mut c, subtree, red, None);
        put(&mut session, &mut c, subtree, blue, None);
        put(&mut session, &mut c, &root, subtree, None);
        assert_eq!(theme(&session), RED);
        take(&mut session, &mut c, red);
        assert_eq!(theme(&session), BLUE);
        take(&mut session, &mut c, blue);
        assert_eq!(theme(&session), None);
        take(&mut session, &mut c, subtree);
        put(&mut session, &mut c, subtree, blue, None);
        put(&mut session, &mut c, subtree, red, None);
        put(&mut session, &mut c, &root, subtree, None);
        assert_eq!(theme(&session), BLUE);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_theme_color_that_cannot_be_satisfied_ends_the_selection_when_the_walk_reaches_it() {
        let (mut c, mut session, root, nodes, lexer) = detached_metas(
            "<meta name=theme-color content=red media=print>\
             <meta name=theme-color content=blue><meta name=theme-color content=green>",
        );
        let [media, blue, green] = &nodes[..] else {
            panic!("three elements")
        };
        put(&mut session, &mut c, &root, blue, None);
        assert_eq!(theme(&session), BLUE);
        // The media list needs an environment: it is not reached
        // behind the color, and ends the selection once it is in front.
        put(&mut session, &mut c, &root, media, None);
        assert_eq!(theme(&session), BLUE);
        assert!(
            history(&session)
                .gap(HtmlParseMetaDomain::ThemeColor)
                .is_none()
        );
        put(&mut session, &mut c, &root, media, Some(blue));
        assert_eq!(theme(&session), None);
        assert!(
            history(&session)
                .gap(HtmlParseMetaDomain::ThemeColor)
                .is_some()
        );
        // Nothing later reselects: the first unsatisfied operation is recorded.
        put(&mut session, &mut c, &root, green, Some(media));
        assert_eq!(theme(&session), None);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    /// The record visits inserting a `div` that holds `children` spans adds,
    /// the spans having been put in the div first.
    fn visits_inserting_a_wide_subtree(children: usize) -> u64 {
        let (mut c, mut session, root, nodes, lexer) =
            detached_metas(&format!("<div>{}", "<span>".repeat(children)));
        let (div, spans) = nodes.split_first().unwrap();
        for span in spans {
            put(&mut session, &mut c, div, span, None);
        }
        let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        put(&mut session, &mut c, &root, div, None);
        let added = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        added
    }

    #[test]
    fn inserting_a_subtree_whole_visits_each_child_without_counting_its_siblings() {
        // The insertion walks the inserted subtree to hide nonces and to settle
        // form effects, stepping from each child to the next. Counting a child's
        // place among its siblings from the first would add a term quadratic in
        // the children, so one doubling would add more than two times the
        // previous one.
        let visits = [100, 200, 400].map(visits_inserting_a_wide_subtree);
        let (first, second) = (visits[1] - visits[0], visits[2] - visits[1]);
        assert!(
            second * 10 <= first * 21,
            "inserting 100, 200 and 400 children added {visits:?} record visits"
        );
    }

    #[test]
    fn a_color_scheme_reads_the_same_in_every_document_mode() {
        // The pinned text reads a `color-scheme` meta by "parsing a list of
        // component values" and asking whether the result is a valid
        // 'color-scheme' value (lines 1874-1890 of
        // `references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md`),
        // and the grammar of that value, keywords and identifiers, reads no
        // quirk of the mode, which only changes lengths, colors and glyph
        // sizes in surgeist-css (quirky_length.rs, quirky_color.rs and
        // svg_glyph.rs). So what a walk learned of the selection stays true
        // when the mode changes.
        let mut c = ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        for content in [
            "dark",
            "light dark",
            "only dark",
            "light only",
            "normal",
            "normal light",
            "only",
            "1",
            "DARK",
            "foo bar",
            "dark 1",
            "",
        ] {
            let units: Vec<u16> = content.encode_utf16().collect();
            let results = [HtmlDocumentMode::NoQuirks, HtmlDocumentMode::Quirks].map(|mode| {
                let value = c.parse_meta_color_scheme(&units, mode).unwrap();
                let valid = value.report().result().is_ok();
                value.release(&mut c).unwrap();
                valid
            });
            assert_eq!(results[0], results[1], "{content:?}");
        }
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn tree_order_agrees_with_the_construction_of_deep_and_shallow_branches() {
        // Two chains of thirty nested divs under the root, a span at the bottom
        // of the first, one under its twenty-sixth div after that div's child,
        // one under its eleventh, and one under the root after both chains.
        // Pre-order is built into the list: each node's place is its index. Pairs
        // from the two chains meet only at the root, thirty levels up, beyond
        // what the climb remembers.
        let depth = 30;
        let (mut c, mut session, root, nodes, lexer) = detached_metas(&format!(
            "{}<span><span><span><span>",
            "<div>".repeat(2 * depth)
        ));
        let (divs, spans) = nodes.split_at(2 * depth);
        let (first, second) = divs.split_at(depth);
        let [bottom, under_twenty_sixth, under_eleventh, under_root] = spans else {
            panic!("four spans")
        };
        for chain in [first, second] {
            put(&mut session, &mut c, &root, &chain[0], None);
            for pair in chain.windows(2) {
                put(&mut session, &mut c, &pair[0], &pair[1], None);
            }
        }
        put(&mut session, &mut c, &first[depth - 1], bottom, None);
        put(&mut session, &mut c, &first[25], under_twenty_sixth, None);
        put(&mut session, &mut c, &first[10], under_eleventh, None);
        put(&mut session, &mut c, &root, under_root, None);
        let order: Vec<&ParserNode> = first
            .iter()
            .chain([bottom, under_twenty_sixth, under_eleventh])
            .chain(second)
            .chain([under_root])
            .collect();
        let storage = session.active().unwrap();
        for (i, earlier) in order.iter().enumerate() {
            for (j, later) in order.iter().enumerate() {
                if i != j {
                    assert_eq!(
                        storage
                            .precedes(&mut c, earlier.index, later.index)
                            .unwrap(),
                        i < j,
                        "record {} against record {}",
                        earlier.index,
                        later.index
                    );
                }
            }
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
    }

    #[test]
    fn css_selection_tracks_original_removal_and_reinsertion_and_releases_replaced_reports() {
        let limits = super::super::tests::limits();
        let source = crate::HtmlSource::utf8(
            "<meta name=color-scheme content=dark><meta name=color-scheme content=light><meta name=theme-color content=red>",
        );
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let first = next_node(&mut session, &mut c, &mut lexer);
        let second = next_node(&mut session, &mut c, &mut lexer);
        let theme = next_node(&mut session, &mut c, &mut lexer);
        for node in [&first, &second, &theme] {
            session
                .active_mut()
                .unwrap()
                .insert(&mut c, &root, node, None, InsertionCause::NonParser)
                .unwrap();
        }
        let selected = |session: &ParserArenaSession| {
            let storage = session.active().unwrap();
            let meta = storage
                .payloads
                .values
                .iter()
                .find_map(|p| {
                    if let RetainedPayload::Document { meta, .. } = p {
                        Some(meta)
                    } else {
                        None
                    }
                })
                .unwrap();
            meta.scheme
                .as_ref()
                .unwrap()
                .report()
                .source()
                .source()
                .original_utf16()
                .unwrap()
                .as_slice()
                .to_vec()
        };
        assert_eq!(
            selected(&session),
            "dark".encode_utf16().collect::<Vec<_>>()
        );
        session
            .active_mut()
            .unwrap()
            .remove(&mut c, &first)
            .unwrap();
        assert_eq!(
            selected(&session),
            "light".encode_utf16().collect::<Vec<_>>()
        );
        session
            .active_mut()
            .unwrap()
            .insert(
                &mut c,
                &root,
                &first,
                Some(&second),
                InsertionCause::NonParser,
            )
            .unwrap();
        assert_eq!(
            selected(&session),
            "dark".encode_utf16().collect::<Vec<_>>()
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn cancellation_after_committed_meta_fields_keeps_original_stop_and_cleans_storage() {
        for (point, skip, source) in [
            (
                CancelPoint::Language,
                1,
                "<!doctype html><meta http-equiv=content-language content=en><meta http-equiv=content-language content=ja>",
            ),
            (
                CancelPoint::Gap,
                0,
                "<!doctype html><meta http-equiv=content-security-policy content=default-src-none>",
            ),
            (
                CancelPoint::Theme,
                0,
                "<!doctype html><meta name=theme-color content=red>",
            ),
            (
                CancelPoint::Scheme,
                0,
                "<!doctype html><meta name=color-scheme content=dark>",
            ),
        ] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                super::super::tests::limits(),
            );
            let revision =
                crate::HtmlBaseFactsRevision::new(crate::HtmlBaseProducerId::fresh(), 1, 2, 3);
            let base = crate::HtmlBaseUrlRecord::observed(
                revision.clone(),
                "https://example.test/".into(),
                crate::HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap();
            let encoding = crate::HtmlBaseEncoding::fresh();
            let facts = surgeist_html_semantics::HtmlEncodingUrlParseFacts::new(
                base,
                encoding,
                revision,
                vec![
                    surgeist_html_semantics::HtmlEncodingUrlParseObservation::new(
                        "unused".into(),
                        surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Failure,
                    ),
                ],
                crate::HtmlBaseFactsLimits::new(4, 1000, 10000),
            )
            .unwrap();
            let mut inputs = Vec::with_capacity(4);
            inputs.push(facts.clone());
            let request = request.with_encoding_url_facts(inputs);
            let original = request.source().identity().clone();
            CANCEL_AFTER_COMMIT.with(|hook| {
                *hook.borrow_mut() = Some((point, skip, request.cancellation().clone()))
            });
            let failure = request.parse().unwrap_err();
            assert_eq!(failure.first_stop(), Some(HtmlParseStop::Cancelled));
            assert_eq!(failure.source().identity(), &original);
            assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&facts));
            assert_eq!(failure.encoding_url_fact_capacity(), Some(4));
            assert_eq!(failure.cleanup_error(), None);
            for resource in [
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::BufferedUtf16Units,
            ] {
                assert_eq!(
                    failure
                        .after_cleanup()
                        .unwrap()
                        .resource(resource)
                        .current(),
                    0,
                    "{resource:?}"
                );
            }
            CANCEL_AFTER_COMMIT.with(|hook| assert!(hook.borrow().is_none()));
        }
    }
}
