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
const _: () = {
    let mut i = 0;
    while i < META_DOMAIN_COUNT {
        assert!(MetaDomain::ALL[i] as usize == i);
        i += 1;
    }
};
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
                        self.meta_refresh_url(c, node, &content.code_units()[range])?
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
    fn meta_candidates(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        detached: Option<usize>,
        theme: bool,
        scheme: bool,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        for (enabled, expected, domain) in [
            (theme, "theme-color", MetaDomain::ThemeColor),
            (scheme, "color-scheme", MetaDomain::ColorScheme),
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
            let mut selected_theme = None;
            let mut selected_scheme = None;
            let mut next = Some(document);
            while let Some(node) = next {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.html_local(c, node, "meta")?
                    && let Some(name) = self.meta_attribute(c, node, "name")?
                    && meta_ascii(c, &name, expected)?
                    && let Some(content) = self.meta_attribute(c, node, "content")?
                {
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
                                self.meta_missing(c, document, node, MetaMissingKind::ThemeColor)?;
                                break;
                            }
                        }
                        match theme_color(c, content.code_units())? {
                            Ok(Some(color)) => {
                                selected_theme = Some(color);
                                break;
                            }
                            Ok(None) => (),
                            Err(error) => {
                                self.meta_missing(
                                    c,
                                    document,
                                    node,
                                    MetaMissingKind::ThemeColorProcessing(error),
                                )?;
                                break;
                            }
                        }
                    } else {
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
                            selected_scheme = Some(candidate);
                            break;
                        }
                        candidate.release(c)?;
                    }
                }
                next = self.next_view_node(c, document, node, detached)?;
            }
            if domain == MetaDomain::ThemeColor {
                payload_write::<Option<surgeist_color::Color>>(c, 1)?;
                self.meta_history_mut(slot)
                    .map_err(|_| latch_registry_row(c))?
                    .theme = selected_theme;
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
            } else {
                // Preserve the admitted new graph on a refused field write.
                if let Err(stop) =
                    payload_write::<Option<crate::parser::control::CssSchemeValue>>(c, 1)
                {
                    if let Some(candidate) = selected_scheme {
                        // The stop is latched; a refusal here would be a later one.
                        let _ = candidate.release(c);
                    }
                    return Err(stop);
                }
                let old = std::mem::replace(
                    &mut self
                        .meta_history_mut(slot)
                        .map_err(|_| latch_registry_row(c))?
                        .scheme,
                    selected_scheme,
                );
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
            }
        }
        Ok(())
    }
    pub(super) fn meta_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
        old_parent: Option<usize>,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<(), SessionError> {
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
        self.meta_candidates(c, document, detached, theme, scheme)
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
            self.meta_candidates(c, document, None, true, name_or_content)?;
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
    while let Some(unit) = input.first() {
        c.charge(WorkCategory::InspectByte, 2)?;
        if !bounded::is_ascii_whitespace(*unit) {
            break;
        }
        input = &input[1..];
    }
    while let Some(unit) = input.last() {
        c.charge(WorkCategory::InspectByte, 2)?;
        if !bounded::is_ascii_whitespace(*unit) {
            break;
        }
        input = &input[..input.len() - 1];
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
