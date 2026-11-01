//! Completed §13.4 parser-Document qualification from the original constructor.
use super::*;
use history::FrozenHistory;

/// The actual auxiliary Document created by HTML877d §13.4, issued only by a
/// completed contextual report. This is a distinct source construction cause;
/// it is neither native creation nor an ordinary offline Document grant.
///
/// The constructor establishes HTML class with literal `application/xml`
/// content type, UTF-8, `about:blank`, known-null about-base, no iframe-srcdoc,
/// a Null Document registry and false Document allow-declarative-shadow-roots.
/// It creates a fresh policy container. Current per-Document Meta history is
/// independently retained by `HtmlParseMetadata::document_meta`; the descriptor
/// does not replace processing history with defaults.
///
/// This parser Document has no browsing context, never has a navigable, is not
/// fully active, and has disabled scripting. Output-node scripting and parser
/// DSD permission remain separate report inputs. No platform, host, loader, UI,
/// image-resource or future current-policy capability is granted here.
///
/// `document()` retains the real ordinary html child. Captured context/target
/// Documents and independently created inert owners retain separate identities.
/// All readers are fixed borrowed observations after bounded qualification.
pub struct FragmentParserDocument<'a> {
    source: &'a crate::HtmlSource,
    fragment: &'a FragmentContextStorage,
    document: HtmlNode<'a>,
    mode: HtmlDocumentMode,
    completion: StandaloneDocumentState,
}
impl<'a> FragmentParserDocument<'a> {
    /// The Document, its mode and its completion are read once, here, so the
    /// readers cannot fail. The fault if the report holds no fragment context,
    /// or no longer holds the Document or the completion row it was
    /// qualified against.
    pub(super) fn new(
        history: &'a FrozenHistory,
        source: &'a crate::HtmlSource,
    ) -> Result<Self, read::UnresolvedReference> {
        let fault = |kind| read::UnresolvedReference::new(kind);
        let storage = history.storage();
        let fragment = storage
            .fragment
            .as_ref()
            .ok_or(fault(read::UnresolvedKind::Fragment))?;
        let document = storage
            .view
            .node_at(0)
            .ok_or(fault(read::UnresolvedKind::Node))?
            .as_node();
        let HtmlNodeKind::Document(mode) = document.kind() else {
            return Err(fault(read::UnresolvedKind::Node));
        };
        let Some(RetainedPayload::StandaloneDocumentCompletion { state, .. }) =
            storage.payloads.values.get(fragment.completion)
        else {
            return Err(fault(read::UnresolvedKind::Fact));
        };
        Ok(Self {
            source,
            fragment,
            document,
            mode: *mode,
            completion: *state,
        })
    }
    pub fn document(&self) -> HtmlNode<'a> {
        self.document
    }
    pub fn context(&self) -> &'a crate::HtmlFragmentContext {
        &self.fragment.context
    }
    pub fn source(&self) -> &'a crate::HtmlSource {
        self.source
    }
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
    }
    pub fn completion(&self) -> StandaloneDocumentState {
        self.completion
    }
    pub fn origin(&self) -> &'a context::FragmentParserOrigin {
        &self.fragment.origin
    }
}

pub(super) fn qualify(storage: &FrozenStorage, c: &mut ParseControl) -> Result<(), SessionError> {
    let fragment = storage
        .fragment
        .as_ref()
        .ok_or(SessionError::ContextUnavailable)?;
    c.charge(WorkCategory::RecordVisit, 4)?;
    c.charge(
        WorkCategory::InspectByte,
        mem::size_of::<StandaloneDocumentState>() as u64,
    )?;
    let Some(RetainedPayload::StandaloneDocumentCompletion { node: 0, state }) =
        storage.payloads.values.get(fragment.completion)
    else {
        return Err(SessionError::CompletionMetadataUnavailable);
    };
    if state.completion != ConstructionCompletion::Complete
        || state.readiness != ConstructionReadiness::Complete
        || !state.ready_for_post_load_tasks
    {
        return Err(SessionError::CompletionIncomplete);
    }
    let records = &storage.view;
    let document = records.node_at(0).ok_or(SessionError::ContextUnavailable)?;
    if !matches!(document.kind(), HtmlNodeKind::Document(mode) if *mode == fragment.context.document_mode())
        || document.children().len() != 1
    {
        return Err(SessionError::ContextUnavailable);
    }
    c.charge(WorkCategory::RecordVisit, 1)?;
    let root = document
        .child_at(0)
        .ok_or(SessionError::ContextUnavailable)?;
    let HtmlNodeKind::Element(element) = root.kind() else {
        return Err(SessionError::ContextUnavailable);
    };
    if !exact_ascii(
        c,
        element.name().namespace().uri(),
        "http://www.w3.org/1999/xhtml",
    )? || !exact_ascii(c, Some(element.name().local_name()), "html")?
    {
        return Err(SessionError::ContextUnavailable);
    }
    // The original closed constructor/end capability already excludes execution
    // and external producers. The identity and completion joins above select
    // that actual Document; no final-tree inference issues its creation proof.
    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use crate::*;

    /// A complete fragment report whose context element is a `div`.
    pub(in crate::construction) fn fragment_report(literal: &str) -> HtmlParseReport {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let div = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&div).unwrap();
        builder.establish_no_shadow_root(&div).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(literal),
            context,
            crate::construction::tests::limits(),
        )
        .unwrap()
        .parse()
        .unwrap()
    }
}
