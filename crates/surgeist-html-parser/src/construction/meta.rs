//! Original-controller Meta transitions. Missing external operations are data;
//! they never certify execution or reset an unknown current value.
use super::*;
use crate::keyed::{Table, slots};
use associated::HyperlinkUnavailable;
use surgeist_html::HtmlMetaPragmaState as Pragma;

/// Independent domains in the finite current Document history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaDomain {
    Refresh,
    Csp,
    ThemeColor,
    ColorScheme,
}
impl MetaDomain {
    pub(super) const ALL: [Self; 4] = [
        Self::Refresh,
        Self::Csp,
        Self::ThemeColor,
        Self::ColorScheme,
    ];
}
const META_DOMAIN_COUNT: usize = 4;
/// One value per [`MetaDomain`]; a lookup cannot be out of range.
pub(super) type MetaDomainTable<T> = Table<MetaDomain, T, META_DOMAIN_COUNT>;
slots!(MetaDomain, META_DOMAIN_COUNT, {
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
        assert!(MetaDomain::ALL[i] as usize == i);
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
    Unsatisfied(MetaMissingKind),
}
/// What a selection ends up with after reading candidates.
enum Decision {
    Nothing,
    Theme(usize, surgeist_color::Color),
    Scheme(usize, crate::parser::control::CssSchemeValue),
    /// The gap is already recorded; nothing is selected.
    Unsatisfied,
}
/// A change to one meta, as it bears on a selection.
#[derive(Clone, Copy)]
enum Change {
    /// It is attached or still attached and a candidate for the selection.
    Candidate,
    /// It is attached and no longer a candidate for the selection.
    NotCandidate,
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
pub enum MetaMissingKind {
    RefreshUrl(HyperlinkUnavailable),
    Csp,
    CspProcessing(surgeist_html::HtmlCspUnavailable),
    ThemeColor,
    ThemeColorProcessing(crate::parser::control::HtmlColorProcessingError),
}
impl MetaMissingKind {
    pub(super) fn domain(&self) -> MetaDomain {
        match self {
            Self::RefreshUrl(_) => MetaDomain::Refresh,
            Self::Csp | Self::CspProcessing(_) => MetaDomain::Csp,
            Self::ThemeColor | Self::ThemeColorProcessing(_) => MetaDomain::ThemeColor,
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
    pub(super) first: Option<MetaDomain>,
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
    /// No walk has run, or a document fact the readings depend on changed.
    Stale,
    /// The first candidate in tree order that selected a value, if any.
    Current(Option<usize>),
}
impl MetaHistory {
    /// The first unsatisfied operation recorded for `domain`; the table has one
    /// cell per `MetaDomain`.
    fn gap(&self, domain: MetaDomain) -> Option<usize> {
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
    /// Makes the next change to a color-scheme candidate walk the tree.
    pub(super) fn forget_color_scheme_selection(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, 0)?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        *self
            .meta_history_mut(slot)
            .map_err(|_| latch_registry_row(c))?
            .selections
            .get_mut(MetaDomain::ColorScheme) = SelectionIndex::Stale;
        Ok(())
    }
    pub(super) fn meta_policy_missing(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        let slot = self.document_slot(c, document)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self
            .meta_history(slot)
            .map_err(|_| latch_registry_row(c))?
            .gap(MetaDomain::Csp))
    }
    pub(super) fn meta_missing(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        subject: usize,
        kind: MetaMissingKind,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let domain = kind.domain();
        c.charge(WorkCategory::RecordVisit, 1)?;
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
            payload_write::<Option<MetaDomain>>(c, 1)?;
        }
        self.prepare_payload_slot_owned(c)?;
        let gap = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::MetaMissing {
            document,
            subject,
            kind,
            cause,
        });
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
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::ForeignNode)?
                .kind(),
            HtmlNodeKind::Document(_)
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
        c.charge(WorkCategory::ProviderCall, 1)?;
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
            bounded::meta_pragma_state_with(&raw, || c.charge(WorkCategory::RecordVisit, 1))?
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
                let Some(range) = bounded::meta_language_token_with(&content, || {
                    c.charge(WorkCategory::RecordVisit, 1)
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
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self
                    .meta_history(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .gap(MetaDomain::Refresh)
                    .is_some()
                    || self
                        .meta_history(slot)
                        .map_err(|_| latch_registry_row(c))?
                        .refresh
                {
                    return Ok(());
                }
                let target = bounded::meta_refresh_syntax_with(&content, |_| {
                    // DelayDigits retains the exact mathematical digit sequence;
                    // the selected Do-nothing branch never consumes a timer.
                    c.charge(WorkCategory::RecordVisit, 1)
                })?;
                let outcome = match target {
                    None => return Ok(()),
                    Some(bounded::HtmlMetaRefreshSyntaxTarget::DocumentUrl) => {
                        self.meta_document_url(c, document)?
                    }
                    Some(bounded::HtmlMetaRefreshSyntaxTarget::UrlString(range)) => {
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
                    Err(missing) => {
                        self.meta_missing(c, document, node, MetaMissingKind::RefreshUrl(missing))?
                    }
                }
            }
            Pragma::DefaultStyle if !content.is_empty() => {
                self.record_association(
                    c,
                    node,
                    pending_association::OccurrenceKind::ChangePreferred,
                    None,
                    None,
                    true,
                )?;
            }
            Pragma::ContentSecurityPolicy if !content.is_empty() => {
                if !self.defer_csp(c, node, document, content)? {
                    self.meta_missing(c, document, node, MetaMissingKind::Csp)?;
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
    ) -> Result<Result<bool, HyperlinkUnavailable>, SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::BaseDocument { node, facts, .. } = payload
                && *node == document
            {
                c.charge(WorkCategory::ProviderCall, 1)?;
                return Ok(facts.document().document_url().map(|_| true).ok_or(
                    HyperlinkUnavailable::Base(crate::HtmlBaseMissing::DocumentUrl),
                ));
            }
        }
        Ok(Err(HyperlinkUnavailable::Base(
            crate::HtmlBaseMissing::DocumentFacts,
        )))
    }
    fn meta_refresh_url(
        &self,
        c: &mut ParseControl,
        node: usize,
        raw: &[u16],
    ) -> Result<Result<bool, HyperlinkUnavailable>, SessionError> {
        let context = match self.encoding_url_context(c, node)? {
            Ok(value) => value,
            Err(missing) => return Ok(Err(missing)),
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        match context.facts.lookup_units_with(
            raw,
            context.base,
            context.encoding,
            context.expected,
            || associated::encoding_url_step(c),
        ) {
            Ok(surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Failure) => Ok(Ok(false)),
            Ok(surgeist_html_semantics::HtmlEncodingUrlParseOutcome::Parsed(record)) => {
                c.charge(WorkCategory::ProviderCall, 1)?;
                Ok(Ok(record.scheme() != crate::HtmlBaseUrlScheme::Javascript))
            }
            Err(surgeist_html_semantics::HtmlEncodingUrlLookupError::Facts(error)) => {
                Ok(Err(HyperlinkUnavailable::Observation(error)))
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
            bounded::meta_referrer_policy_with(&content, || c.charge(WorkCategory::RecordVisit, 1))?
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
            (theme, MetaDomain::ThemeColor),
            (scheme, MetaDomain::ColorScheme),
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
                c.charge(WorkCategory::RecordVisit, 1)?;
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
        domain: MetaDomain,
    ) -> Result<Option<Reading>, SessionError> {
        let expected = if domain == MetaDomain::ThemeColor {
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
        if domain == MetaDomain::ThemeColor {
            if let Some(media) = self.meta_attribute(c, node, "media")? {
                let mut known_match = true;
                for unit in media.code_units() {
                    c.charge(WorkCategory::InspectByte, 2)?;
                    if !bounded::is_ascii_whitespace(*unit) {
                        known_match = false;
                        break;
                    }
                }
                if !known_match {
                    return Ok(Some(Reading::Unsatisfied(MetaMissingKind::ThemeColor)));
                }
            }
            return Ok(Some(match theme_color(c, content.code_units())? {
                Ok(Some(color)) => Reading::Theme(color),
                Ok(None) => Reading::Rejected,
                Err(error) => Reading::Unsatisfied(MetaMissingKind::ThemeColorProcessing(error)),
            }));
        }
        let HtmlNodeKind::Document(mode) = self
            .construction
            .node_at(document)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::NotDocument);
        };
        let candidate = c.parse_meta_color_scheme(content.code_units(), *mode)?;
        if let Err(stop) = c.charge(WorkCategory::ProviderCall, 1) {
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
        domain: MetaDomain,
        decided: Decision,
    ) -> Result<(), SessionError> {
        let deciding = match &decided {
            Decision::Theme(node, _) | Decision::Scheme(node, _) => Some(*node),
            Decision::Nothing | Decision::Unsatisfied => None,
        };
        if domain == MetaDomain::ThemeColor {
            payload_write::<Option<surgeist_color::Color>>(c, 1)?;
            let meta = self
                .meta_history_mut(slot)
                .map_err(|_| latch_registry_row(c))?;
            meta.theme = match decided {
                Decision::Theme(_, color) => Some(color),
                _ => None,
            };
            *meta.selections.get_mut(domain) = SelectionIndex::Current(deciding);
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
        *meta.selections.get_mut(domain) = SelectionIndex::Current(deciding);
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
        c.charge(WorkCategory::RecordVisit, 1)?;
        let alone = node == root
            && self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .child_count()
                == 0;
        if !alone {
            return self.meta_candidates(c, document, detached, theme, scheme);
        }
        let effect = if removal {
            Change::Removed
        } else {
            Change::Candidate
        };
        for (enabled, domain) in [
            (theme, MetaDomain::ThemeColor),
            (scheme, MetaDomain::ColorScheme),
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
        domain: MetaDomain,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let history = self.meta_history(slot).map_err(|_| latch_registry_row(c))?;
        let SelectionIndex::Current(deciding) = history.selections.copy(domain) else {
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
    /// Brings the selection `domain` up to date with a change to `node`, the
    /// only candidate whose state changed, reading it only if the selection
    /// can depend on it.
    fn meta_settle(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        node: usize,
        detached: Option<usize>,
        domain: MetaDomain,
        change: Change,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let history = self.meta_history(slot).map_err(|_| latch_registry_row(c))?;
        if history.gap(domain).is_some() {
            return Ok(());
        }
        let known = history.selections.copy(domain);
        let (theme, scheme) = (
            domain == MetaDomain::ThemeColor,
            domain == MetaDomain::ColorScheme,
        );
        let SelectionIndex::Current(deciding) = known else {
            return self.meta_candidates(c, document, detached, theme, scheme);
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        let step = if deciding == Some(node) {
            // The change may end or replace the decision: only a walk knows
            // the candidate that decides next.
            Step::Walk
        } else {
            match (change, deciding) {
                (Change::Removed | Change::NotCandidate, _) => Step::Unchanged,
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
    pub(super) fn precedes(
        &self,
        c: &mut ParseControl,
        first: usize,
        second: usize,
    ) -> Result<bool, SessionError> {
        let storage = self.construction.storage(c)?;
        let parent = |c: &mut ParseControl, node: usize| -> Result<Option<usize>, SessionError> {
            c.charge(WorkCategory::RecordVisit, 1)?;
            Ok(storage
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .parent()
                .map(|parent| parent.position()))
        };
        let depth = |c: &mut ParseControl, mut node: usize| -> Result<usize, SessionError> {
            let mut depth = 0_usize;
            while let Some(above) = parent(c, node)? {
                node = above;
                depth = depth.checked_add(1).ok_or(SessionError::Cycle)?;
                if depth > storage.len() {
                    return Err(SessionError::Cycle);
                }
            }
            Ok(depth)
        };
        let (mut left, mut right) = (first, second);
        // Siblings are the usual pair: no ancestor needs to be found.
        let mut above = parent(c, left)?;
        if above.is_none() || above != parent(c, right)? {
            let (mut left_depth, mut right_depth) = (depth(c, left)?, depth(c, right)?);
            while left_depth > right_depth {
                left = parent(c, left)?.ok_or(SessionError::Cycle)?;
                left_depth -= 1;
                if left == right {
                    // `second` is an ancestor of `first`, so it comes first.
                    return Ok(false);
                }
            }
            while right_depth > left_depth {
                right = parent(c, right)?.ok_or(SessionError::Cycle)?;
                right_depth -= 1;
                if left == right {
                    // `first` is an ancestor of `second`.
                    return Ok(true);
                }
            }
            loop {
                let (up_left, up_right) = (parent(c, left)?, parent(c, right)?);
                if up_left == up_right {
                    above = up_left;
                    break;
                }
                left = up_left.ok_or(SessionError::Cycle)?;
                right = up_right.ok_or(SessionError::Cycle)?;
            }
        }
        let above = above.ok_or(SessionError::Cycle)?;
        // The later sibling is the one a walk back from the end of the child
        // order reaches first.
        c.charge(WorkCategory::RecordVisit, 1)?;
        let siblings = storage.node_at(above).ok_or(SessionError::ForeignNode)?;
        for ordinal in (0..siblings.child_count()).rev() {
            c.charge(WorkCategory::RecordVisit, 1)?;
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
        let HtmlNodeKind::Element(element) = record.kind() else {
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
        let HtmlNodeKind::Element(element) = record.kind() else {
            return Ok(());
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        let attribute = element
            .attributes()
            .as_slice()
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
            for (enabled, domain, expected) in [
                (true, MetaDomain::ThemeColor, "theme-color"),
                (name_or_content, MetaDomain::ColorScheme, "color-scheme"),
            ] {
                if !enabled {
                    continue;
                }
                let change = if self.meta_candidate(c, node, expected)? {
                    Change::Candidate
                } else {
                    Change::NotCandidate
                };
                self.meta_settle(c, document, node, None, domain, change)?;
                #[cfg(test)]
                self.assert_selection_matches_walk(c, document, None, domain)?;
            }
        }
        Ok(())
    }
}
/// HTML selects the ASCII-whitespace subrange; CSS owns preprocessing, grammar
/// and used-color resolution. A genuine syntax rejection has no input fallback.
fn theme_color(
    c: &mut ParseControl,
    mut input: &[u16],
) -> Result<
    Result<Option<surgeist_color::Color>, crate::parser::control::HtmlColorProcessingError>,
    SessionError,
> {
    while let Some((unit, rest)) = input.split_first() {
        c.charge(WorkCategory::InspectByte, 2)?;
        if !bounded::is_ascii_whitespace(*unit) {
            break;
        }
        input = rest;
    }
    while let Some((unit, rest)) = input.split_last() {
        c.charge(WorkCategory::InspectByte, 2)?;
        if !bounded::is_ascii_whitespace(*unit) {
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
fn meta_ascii(
    c: &mut ParseControl,
    value: &DocumentString,
    expected: &str,
) -> Result<bool, SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if value.len() != expected.len() {
        return Ok(false);
    }
    for (unit, byte) in value.code_units().iter().zip(expected.bytes()) {
        c.charge(WorkCategory::InspectByte, 2)?;
        if !u8::try_from(*unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(&byte)) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct RedPolicy;
    impl surgeist_html::HtmlCspPolicyArtifact for RedPolicy {
        fn retained_layout(&self) -> surgeist_html::HtmlCspArtifactLayout {
            surgeist_html::HtmlCspArtifactLayout::default()
        }
    }
    struct RedProcessor {
        authority: surgeist_html::HtmlCspProcessorAuthority,
        profile: surgeist_html::HtmlCspProcessorProfile,
        refuse: bool,
        calls: usize,
        completed: usize,
    }
    impl surgeist_html::HtmlMetaCspProcessor for RedProcessor {
        type Error = ();
        fn profile(&self) -> &surgeist_html::HtmlCspProcessorProfile {
            &self.profile
        }
        fn prepare<E>(
            &mut self,
            request: &surgeist_html::HtmlMetaCspRequest,
            admit: &mut impl FnMut(surgeist_html::HtmlCspLayout) -> Result<(), E>,
            release: &mut impl FnMut(surgeist_html::HtmlCspStorage),
        ) -> Result<surgeist_html::HtmlMetaCspOutcome<()>, surgeist_html::HtmlCspError<E>> {
            use surgeist_html::*;
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
            let layout = surgeist_html::HtmlCspArtifactLayout::default();
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
        use surgeist_html::*;
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
                    Some(ParseStop::CspProcessing)
                );
                assert_eq!(
                    failure.parse_failure().reason(),
                    crate::HtmlParseFailureReason::CspProcessing
                );
                CANCEL_AFTER_COMMIT.with(|hook| assert!(hook.borrow_mut().take().is_some()));
            } else {
                assert_eq!(
                    failure.parse_failure().first_stop(),
                    Some(ParseStop::Cancelled)
                );
                CANCEL_AFTER_COMMIT.with(|hook| assert!(hook.borrow().is_none()));
            }
        }
    }

    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};

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
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
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
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
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
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        c.admit_source(source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(source, State::Data, None, &mut c).unwrap();
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
        let (mut c, mut session, root, nodes, mut lexer) = detached_metas(
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
        let (mut c, mut session, root, nodes, mut lexer) = detached_metas(
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
        // A document mode changes how a value reads, so what an earlier walk
        // learned is not carried over it.
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
    fn a_theme_color_that_cannot_be_satisfied_ends_the_selection_when_the_walk_reaches_it() {
        let (mut c, mut session, root, nodes, mut lexer) = detached_metas(
            "<meta name=theme-color content=red media=print>\
             <meta name=theme-color content=blue><meta name=theme-color content=green>",
        );
        let [media, blue, green] = &nodes[..] else {
            panic!("three elements")
        };
        put(&mut session, &mut c, &root, blue, None);
        assert_eq!(theme(&session), BLUE);
        // The media list names a query nobody can answer: it is not reached
        // behind the color, and ends the selection once it is in front.
        put(&mut session, &mut c, &root, media, None);
        assert_eq!(theme(&session), BLUE);
        assert!(history(&session).gap(MetaDomain::ThemeColor).is_none());
        put(&mut session, &mut c, &root, media, Some(blue));
        assert_eq!(theme(&session), None);
        assert!(history(&session).gap(MetaDomain::ThemeColor).is_some());
        // Nothing later reselects: the first unsatisfied operation is recorded.
        put(&mut session, &mut c, &root, green, Some(media));
        assert_eq!(theme(&session), None);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
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
        let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
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
            assert_eq!(failure.first_stop(), Some(ParseStop::Cancelled));
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
