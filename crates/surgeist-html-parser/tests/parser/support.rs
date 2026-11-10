//! Shared builders for the parser integration tests.
//!
//! Each helper here is used by more than one domain module. A module whose
//! fixtures need different parse limits starts from `limits_builder` or
//! `small_limits_builder` and overrides only the caps that differ, so the
//! difference stays visible. Limits unique to one module's measurement (a cap
//! table, a linear-work budget) stay local.

use surgeist_html::{
    construction::HtmlDocumentMode,
    values::{Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

/// The generous parse limits, unfinished, so a module whose fixture needs a
/// different cap states exactly which caps it overrides.
pub(crate) fn limits_builder() -> HtmlParseLimitsBuilder {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(1_000_000)
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
}

/// Generous parse limits that ordinary test fixtures do not reach.
pub(crate) fn limits() -> HtmlParseLimits {
    limits_builder().finish().unwrap()
}

/// Small parse limits, unfinished: ten thousand source units and one thousand
/// nodes, edges, attributes, diagnostics and context effects, with ten million
/// bytes and work credits.
pub(crate) fn small_limits_builder() -> HtmlParseLimitsBuilder {
    HtmlParseLimits::builder()
        .source_units(10000)
        .buffered_utf16_units(10000)
        .token_attributes(1000)
        .auxiliary_entries(10000)
        .active_formatting_entries(10000)
        .managed_nodes(1000)
        .managed_edges(1000)
        .managed_attributes(1000)
        .managed_utf16_units(10000)
        .source_map_entries(10000)
        .source_map_payload_units(10000)
        .diagnostics(1000)
        .diagnostic_payload_units(1000)
        .context_effects(1000)
        .open_elements(100)
        .tree_depth(1000)
        .managed_capacity_bytes(10000000)
        .peak_transient_bytes(10000000)
        .atomic_layout_bytes(10000000)
        .work_credits(10000000)
}

/// Small parse limits.
pub(crate) fn small_limits() -> HtmlParseLimits {
    small_limits_builder().finish().unwrap()
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
    fragment_context_for("div", HtmlScriptingStatus::Disabled)
}

/// As [`fragment_context`], with an HTML context element named `local` in a
/// captured document whose scripting status is `scripting`.
pub(crate) fn fragment_context_for(
    local: &str,
    scripting: HtmlScriptingStatus,
) -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            scripting,
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
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, local),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    builder.establish_no_shadow_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap()
}

/// An offline document request that selects every kind of parse detail, for the
/// tests that read it.
pub(crate) fn offline_all(source: HtmlSource, limits: HtmlParseLimits) -> HtmlDocumentParseRequest {
    HtmlDocumentParseRequest::offline(source, limits)
        .with_detail(DetailSet::ALL)
        .expect("the detail feature is enabled in these tests")
}

/// The readers a report had before they moved to its detail, for the tests
/// whose requests select every kind.
pub(crate) trait ReportDetail {
    fn associations(&self) -> HtmlParseAssociations<'_>;
    fn history_view(&self) -> HtmlParseHistory<'_>;
    fn diagnostics(&self) -> HtmlParseDiagnostics<'_>;
}

impl ReportDetail for HtmlParseReport {
    fn associations(&self) -> HtmlParseAssociations<'_> {
        self.detail()
            .expect("the request selected detail")
            .associations()
            .expect("the request selected the source map")
    }
    fn history_view(&self) -> HtmlParseHistory<'_> {
        self.detail()
            .expect("the request selected detail")
            .history()
            .expect("the request selected the history")
    }
    fn diagnostics(&self) -> HtmlParseDiagnostics<'_> {
        self.detail()
            .expect("the request selected detail")
            .diagnostics()
            .expect("the request selected diagnostics")
    }
}

/// The recorded cause of a value of a report whose request selected every kind.
pub(crate) trait SubjectCause<'a> {
    fn cause(&self) -> Option<HtmlParseCause<'a>>;
}

impl<'a, T: HtmlParseCaused<'a>> SubjectCause<'a> for T {
    fn cause(&self) -> Option<HtmlParseCause<'a>> {
        HtmlParseCaused::recorded_cause(self)
    }
}

/// A fragment request that selects every kind of parse detail.
pub(crate) fn fragment_all(
    source: HtmlSource,
    context: HtmlFragmentContext,
    limits: HtmlParseLimits,
) -> Result<HtmlFragmentParseRequest, HtmlParseRequestError> {
    HtmlFragmentParseRequest::new(source, context, limits).map(select_all)
}

/// As [`fragment_all`], with the scripting mode and the shadow-root choice.
pub(crate) fn fragment_all_with_options(
    source: HtmlSource,
    context: HtmlFragmentContext,
    limits: HtmlParseLimits,
    requested: surgeist_html::parsing::HtmlParserScriptingMode,
    allow_declarative_shadow_roots: bool,
) -> Result<HtmlFragmentParseRequest, HtmlParseRequestError> {
    HtmlFragmentParseRequest::with_options(
        source,
        context,
        limits,
        requested,
        allow_declarative_shadow_roots,
    )
    .map(select_all)
}

fn select_all(request: HtmlFragmentParseRequest) -> HtmlFragmentParseRequest {
    request
        .with_detail(DetailSet::ALL)
        .expect("the detail feature is enabled in these tests")
}
