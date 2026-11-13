//! Completed §13.4 parser-Document qualification from the original constructor.
use super::*;
use history::FrozenHistory;
use surgeist_html::construction::{ConstructionNodeKind, HtmlDocumentMode, HtmlNode, HtmlNodeKind};

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
pub struct HtmlParseFragmentParserDocument<'a> {
    source: &'a crate::HtmlSource,
    fragment: &'a FragmentContextStorage,
    document: HtmlNode<'a>,
    mode: HtmlDocumentMode,
    completion: HtmlParseDocumentState,
}
impl<'a> HtmlParseFragmentParserDocument<'a> {
    /// The Document, its mode and its completion are read once, here, so the
    /// readers cannot fail. The fault if the report holds no fragment context,
    /// or no longer holds the Document or the completion it was
    /// qualified against.
    pub(super) fn new(
        history: &'a FrozenHistory,
        source: &'a crate::HtmlSource,
    ) -> Result<Self, read::HtmlParseUnresolvedReference> {
        let fault = |kind| read::HtmlParseUnresolvedReference::new(kind);
        let storage = history.storage();
        let fragment = storage
            .fragment
            .as_ref()
            .ok_or(fault(read::HtmlParseUnresolvedKind::Fragment))?;
        let document = storage
            .view
            .node_at(0)
            .ok_or(fault(read::HtmlParseUnresolvedKind::Node))?
            .as_node()
            .map_err(|_| fault(read::HtmlParseUnresolvedKind::Node))?;
        let HtmlNodeKind::Document(mode) = document.kind() else {
            return Err(fault(read::HtmlParseUnresolvedKind::Node));
        };
        let Some(completion) = storage.payloads.facts.completion else {
            return Err(fault(read::HtmlParseUnresolvedKind::Fact));
        };
        Ok(Self {
            source,
            fragment,
            document,
            mode,
            completion: completion.state(),
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
    pub fn completion(&self) -> HtmlParseDocumentState {
        self.completion
    }
    pub fn origin(&self) -> &'a context::HtmlParseOpaqueOrigin {
        &self.fragment.origin
    }
}

pub(super) fn qualify(storage: &FrozenStorage, c: &mut ParseControl) -> Result<(), SessionError> {
    let fragment = storage
        .fragment
        .as_ref()
        .ok_or(SessionError::ContextUnavailable)?;
    c.charge(HtmlParseWorkCategory::RecordVisit, 4)?;
    c.charge(
        HtmlParseWorkCategory::InspectByte,
        mem::size_of::<completion::Completion>() as u64,
    )?;
    let Some(completion) = storage.payloads.facts.completion else {
        return Err(SessionError::CompletionMetadataUnavailable);
    };
    if !completion.is_complete() {
        return Err(SessionError::CompletionIncomplete);
    }
    let records = &storage.view;
    let document = records.node_at(0).ok_or(SessionError::ContextUnavailable)?;
    if !matches!(document.kind(), ConstructionNodeKind::Document(mode) if mode == fragment.context.document_mode())
        || document.first_child().map(|child| child.position())
            != document.last_child().map(|child| child.position())
        || document.first_child().is_none()
    {
        return Err(SessionError::ContextUnavailable);
    }
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let root = document
        .first_child()
        .ok_or(SessionError::ContextUnavailable)?;
    let ConstructionNodeKind::Element(element) = root.kind() else {
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
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };

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

#[cfg(test)]
mod inspection_tests {
    use super::*;

    #[test]
    fn qualifying_a_fragment_document_inspects_the_completion_and_the_root_name_it_reads() {
        let mut report = tests::fragment_report("<p>a</p>");
        let storage = report.storage_mut().history_mut().storage_mut();
        let mut c = ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        let before = c.usage().clone();
        qualify(storage, &mut c).unwrap();
        let inspected = c.usage().work(HtmlParseWorkCategory::InspectByte)
            - before.work(HtmlParseWorkCategory::InspectByte);
        // The completion is read once, as the two bytes of a `Completion`, not as the three of
        // the `HtmlParseDocumentState` it converts to. The root `html` element is compared with
        // its namespace, 28 units, and its name, 4 units, each a 2-byte unit paired with a byte.
        assert_eq!(mem::size_of::<completion::Completion>(), 2);
        assert_eq!(mem::size_of::<HtmlParseDocumentState>(), 3);
        assert_eq!(inspected, 2 + (28 + 4) * 3);
    }
}
