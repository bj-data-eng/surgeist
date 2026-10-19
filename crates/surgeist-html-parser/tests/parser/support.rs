//! Shared builders for the parser integration tests.
//!
//! Each helper here is used by more than one domain module. Modules whose
//! fixtures need different values (for example tighter limits) keep a local
//! helper instead.

use surgeist_html::*;
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

/// Generous parse limits that ordinary test fixtures do not reach.
pub(crate) fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

/// A fresh base-facts revision with fixed counters.
pub(crate) fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}

/// An observed base URL record with the `Other` scheme under `revision`.
pub(crate) fn record(
    revision: &HtmlBaseFactsRevision,
    url: &str,
    blank: bool,
    fragment: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        url.into(),
        HtmlBaseUrlScheme::Other,
        blank,
        fragment,
    )
    .unwrap()
}

/// Base facts whose document base is `base` with `encoding`.
pub(crate) fn environment(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            base.revision().clone(),
            Some(base.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap()
}

/// An encoding URL-parse bank for `base` with the given observations.
pub(crate) fn bank(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    observations: Vec<HtmlEncodingUrlParseObservation>,
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        base.revision().clone(),
        observations,
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap()
}

/// An element fragment context: a captured no-quirks HTML document whose
/// root is a `div`, with scripting disabled and a null registry.
pub(crate) fn fragment_context() -> HtmlFragmentContext {
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
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}
