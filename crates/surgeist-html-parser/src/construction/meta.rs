//! Original-controller Meta transitions. Missing external operations are data;
//! they never certify execution or reset an unknown current value.
use super::*;
use associated::HyperlinkUnavailable;
use surgeist_html::HtmlMetaPragmaState as Pragma;

/// Independent domains in the finite current Document history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaDomain {
    Refresh,
    PreferredStyle,
    Csp,
    ThemeColor,
    ColorScheme,
}
impl MetaDomain {
    pub(super) const ALL: [Self; 5] = [
        Self::Refresh,
        Self::PreferredStyle,
        Self::Csp,
        Self::ThemeColor,
        Self::ColorScheme,
    ];
    pub(super) const fn index(self) -> usize {
        match self {
            Self::Refresh => 0,
            Self::PreferredStyle => 1,
            Self::Csp => 2,
            Self::ThemeColor => 3,
            Self::ColorScheme => 4,
        }
    }
}
/// First actual unsatisfied operation in one independent domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetaMissingKind {
    RefreshUrl(HyperlinkUnavailable),
    PreferredStyle,
    Csp,
    ThemeColor,
    ColorScheme,
}
impl MetaMissingKind {
    pub(super) fn domain(&self) -> MetaDomain {
        match self {
            Self::RefreshUrl(_) => MetaDomain::Refresh,
            Self::PreferredStyle => MetaDomain::PreferredStyle,
            Self::Csp => MetaDomain::Csp,
            Self::ThemeColor => MetaDomain::ThemeColor,
            Self::ColorScheme => MetaDomain::ColorScheme,
        }
    }
}
/// Closed current state issued only by the source-created Document constructor.
/// The original controller has no external UI association or execution path.
/// It chooses HTML877d's Do-nothing refresh policy, independently of BC state.
/// Gap slots are at most one per domain, never a transition/event log.
pub(super) struct MetaHistory {
    pub(super) language: Option<usize>,
    pub(super) referrer: HtmlReferrerPolicy,
    pub(super) refresh: bool,
    pub(super) gaps: [Option<usize>; 5],
    pub(super) first: Option<MetaDomain>,
}
impl MetaHistory {
    pub(super) fn offline() -> Self {
        Self {
            language: None,
            referrer: default_document_referrer_policy(),
            refresh: false,
            gaps: [None; 5],
            first: None,
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum CancelPoint {
    Language,
    Gap,
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
    fn meta_history(&self, slot: usize) -> &MetaHistory {
        let RetainedPayload::Document { meta, .. } = &self.payloads.values[slot] else {
            unreachable!("qualified actual Document row")
        };
        meta
    }
    fn meta_history_mut(&mut self, slot: usize) -> &mut MetaHistory {
        let RetainedPayload::Document { meta, .. } = &mut self.payloads.values[slot] else {
            unreachable!("qualified actual Document row")
        };
        meta
    }
    pub(super) fn meta_policy_missing(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        let slot = self.document_slot(c, document)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.meta_history(slot).gaps[MetaDomain::Csp.index()])
    }
    fn meta_missing(
        &mut self,
        c: &mut ParseControl,
        document: usize,
        subject: usize,
        kind: MetaMissingKind,
    ) -> Result<(), SessionError> {
        let slot = self.document_slot(c, document)?;
        let domain = kind.domain();
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.meta_history(slot).gaps[domain.index()].is_some() {
            return Ok(());
        }
        let cause = self.associated_cause(c)?;
        let first = self.meta_history(slot).first.is_none();
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
        let meta = self.meta_history_mut(slot);
        meta.gaps[domain.index()] = Some(gap);
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
                let old = self.meta_history_mut(slot).language.replace(value);
                #[cfg(test)]
                after_commit(CancelPoint::Language);
                if let Some(old) = old {
                    self.retire_registered_string(c, old)?;
                }
            }
            Pragma::Refresh if !content.is_empty() => {
                let slot = self.document_slot(c, document)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.meta_history(slot).gaps[MetaDomain::Refresh.index()].is_some()
                    || self.meta_history(slot).refresh
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
                        self.meta_history_mut(slot).refresh = true;
                    }
                    Err(missing) => {
                        self.meta_missing(c, document, node, MetaMissingKind::RefreshUrl(missing))?
                    }
                }
            }
            Pragma::DefaultStyle if !content.is_empty() => {
                self.meta_missing(c, document, node, MetaMissingKind::PreferredStyle)?;
            }
            Pragma::ContentSecurityPolicy if !content.is_empty() => {
                self.meta_missing(c, document, node, MetaMissingKind::Csp)?;
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
            self.meta_history_mut(slot).referrer = policy;
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
        for (enabled, expected, kind) in [
            (theme, "theme-color", MetaMissingKind::ThemeColor),
            (scheme, "color-scheme", MetaMissingKind::ColorScheme),
        ] {
            if !enabled {
                continue;
            }
            let mut next = Some(document);
            while let Some(node) = next {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.html_local(c, node, "meta")?
                    && let Some(name) = self.meta_attribute(c, node, "name")?
                    && meta_ascii(c, &name, expected)?
                    && self.form_attribute(c, node, "content")?.is_some()
                {
                    // Actual first candidate reaches unavailable CSS/media
                    // parsing. No-UI does not skip this dependency or call it
                    // invalid. Keep the first missing operation and stop only
                    // this semantic pass; later independent fields still run.
                    self.meta_missing(c, document, node, kind)?;
                    break;
                }
                next = self.next_view_node(c, document, node, detached)?;
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
            self.meta_pragma(c, node, document)?;
            self.meta_referrer(c, node)?;
        }
        self.meta_candidates(c, document, detached, true, true)
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
            self.meta_candidates(c, document, None, true, name_or_content)?;
        }
        Ok(())
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
    fn cancellation_after_language_replacement_or_first_gap_keeps_original_stop_and_cleans_storage()
    {
        for (point, skip, source) in [
            (
                CancelPoint::Language,
                1,
                "<!doctype html><meta http-equiv=content-language content=en><meta http-equiv=content-language content=ja>",
            ),
            (
                CancelPoint::Gap,
                0,
                "<!doctype html><meta http-equiv=default-style content=chosen>",
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
