//! Selection of the optional parse detail, in a build with the `detail`
//! feature and in one without it.

use surgeist_html_parser::{
    DetailSet, HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits,
    HtmlParseResource, HtmlSource,
};

fn limits() -> HtmlParseLimits {
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
        .open_elements(1_000)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(u64::MAX)
        .finish()
        .unwrap()
}

/// Parse errors, markup of every lexical shape, a `style` element and a `meta`
/// default-style name: what each kind of detail records.
const DOCUMENT: &str = "<p class=a>one<b>two</b></p>\r\n< <!-- c --><table><tr><td>x</table>\
<style title=A>a{}</style><meta http-equiv=default-style content=B>";

const DETAIL_RESOURCES: [HtmlParseResource; 4] = [
    HtmlParseResource::SourceMapEntries,
    HtmlParseResource::SourceMapPayloadUnits,
    HtmlParseResource::Diagnostics,
    HtmlParseResource::DiagnosticPayloadUnits,
];

#[test]
fn a_request_selects_no_detail_until_it_names_some() {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits());
    assert_eq!(request.detail(), DetailSet::NONE);
    assert!(DetailSet::NONE.is_empty());
    assert!(!DetailSet::ALL.is_empty());
}

#[test]
fn a_fragment_parse_that_selects_no_detail_retains_none_of_it() {
    use surgeist_html::parsing::HtmlParserScriptingMode;
    let context = support::fragment_context();
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<p>a<b>b</b>< <!-- c -->"),
        context,
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap();
    for resource in DETAIL_RESOURCES {
        let usage = report.usage().resource(resource);
        assert_eq!(
            (usage.current(), usage.high_water()),
            (0, 0),
            "{resource:?}"
        );
    }
}

#[cfg(feature = "detail")]
#[test]
fn a_parse_that_selects_detail_has_it_and_reads_it() {
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
        .with_detail(DetailSet::ALL)
        .unwrap();
    assert_eq!(request.detail(), DetailSet::ALL);
    let report = request.parse().unwrap();
    let detail = report.detail().expect("every kind is selected");
    assert_eq!(detail.selected(), DetailSet::ALL);
    assert!(!detail.diagnostics().unwrap().is_empty());
    assert!(
        report
            .usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current()
            > 0
    );
    let none = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
        .parse()
        .unwrap();
    assert!(none.detail().is_none());
}

#[cfg(not(feature = "detail"))]
#[test]
fn without_the_feature_a_non_empty_selection_is_refused_with_a_typed_error() {
    for selected in [
        DetailSet::SOURCE_MAP,
        DetailSet::LEXICAL,
        DetailSet::HISTORY,
        DetailSet::DIAGNOSTICS,
        DetailSet::ALL,
    ] {
        let refused = HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
            .with_detail(selected)
            .unwrap_err();
        assert_eq!(refused.requested(), selected);
        let context = support::fragment_context();
        let refused = HtmlFragmentParseRequest::new(HtmlSource::utf8("x"), context, limits())
            .unwrap()
            .with_detail(selected)
            .unwrap_err();
        assert_eq!(refused.requested(), selected);
    }
    // The empty selection is always available.
    assert!(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(DOCUMENT), limits())
            .with_detail(DetailSet::NONE)
            .is_ok()
    );
}

mod support {
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };
    use surgeist_html_semantics::*;

    /// An element fragment context: a captured no-quirks HTML document whose
    /// root is a `div`, with scripting disabled and a null registry.
    pub(super) fn fragment_context() -> HtmlFragmentContext {
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
        let element = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
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
}
