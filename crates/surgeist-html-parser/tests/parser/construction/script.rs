use crate::parser::support::limits;
use surgeist_html::{HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseMetadataStep, HtmlSource};

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
                        let result: HtmlScriptResult = script.result();
                        let ready: Option<HtmlScriptReadySteps> = script.steps_when_result_ready();
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
