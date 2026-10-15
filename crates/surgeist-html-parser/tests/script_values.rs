use surgeist_html::{HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseMetadataStep, HtmlParseScriptReadySteps,
    HtmlParseScriptResult, HtmlSource,
};

fn limits() -> HtmlParseLimits {
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

// HTML877d raw47442–47447 initializes result/ready steps; Disabled preparation
// returns at raw47640 before installing a result or result-ready callback.
#[test]
fn disabled_preparation_preserves_uninitialized_result_and_absent_ready_steps() {
    for (kind, expected) in [
        ("text/javascript", HtmlScriptKind::Classic),
        ("module", HtmlScriptKind::Module),
        ("importmap", HtmlScriptKind::ImportMap),
        ("speculationrules", HtmlScriptKind::SpeculationRules),
    ] {
        let report = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!(
                "<!doctype html><script type='{kind}' src=''>body</script>"
            )),
            limits(),
        )
        .parse()
        .unwrap();
        let mut metadata = report.metadata();
        let mut found = false;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(script) = record.script_state() {
                        found = true;
                        assert!(script.already_started());
                        assert_eq!(script.script_type(), Some(expected));
                        // Both existing parser public names directly name the
                        // canonical HTML values returned by the actual reader.
                        let result: HtmlScriptResult = script.result();
                        let result: HtmlParseScriptResult = result;
                        let ready: Option<HtmlScriptReadySteps> = script.steps_when_result_ready();
                        let ready: Option<HtmlParseScriptReadySteps> = ready;
                        assert_eq!(result, HtmlScriptResult::Uninitialized);
                        assert_eq!(ready, None);
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(found);
    }
}
