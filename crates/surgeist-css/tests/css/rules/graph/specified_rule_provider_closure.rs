#![forbid(unsafe_code)]
//! Wrapper consumption of provider owners #621 and #553/#555. These tests get
//! behavior RED through existing CssSheet emission, not through missing leaf APIs.
//! Their initial UnsupportedRule failure does not prove direct leaf-provider RED.
use surgeist_css::parse_sheet;

fn exact(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert!(parse_sheet(expected).is_clean());
}

#[test]
fn animation_direction_consumer_emits_typed_keywords_in_list_order() {
    exact(
        ".x{animation-direction:REVERSE,ALTERNATE-REVERSE}",
        ".x { animation-direction: reverse, alternate-reverse; }",
    );
}

#[test]
fn transform_consumer_emits_typed_functions_in_authored_list_order() {
    exact(
        ".x{transform:translateX(+001.5PX) rotate(+090DEG)}",
        ".x { transform: translateX(1.5px) rotate(90deg); }",
    );
}
