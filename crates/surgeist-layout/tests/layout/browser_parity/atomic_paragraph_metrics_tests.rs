use super::*;
use serde_json::{Value, json};
fn certificate() -> Value {
    json!({"profile":1,"font":{"sha256":"e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c","family":"ahem","size":10,"style":"normal","weight":400,"stretch":"normal","loaded":true},"writingMode":"vertical-rl","direction":"rtl","textOrientation":"mixed","dominantBaseline":"auto","alignmentBaseline":"auto","verticalAlign":"baseline","lineFit":"leading","reference":"central","before":0,"after":0,"lineHeight":0,"order":[{"kind":"atomic","child":0,"source":1,"bidiLevel":1},{"kind":"break","child":1,"source":2,"reference":"central","before":0,"after":0},{"kind":"atomic","child":2,"source":3,"bidiLevel":1}]})
}
fn source(capture: Option<&Value>) -> String {
    let capture = capture
        .map(|value| {
            format!(
                " atomic-paragraph-metrics=\"{}\"",
                value
                    .to_string()
                    .replace('&', "&amp;")
                    .replace('\"', "&quot;")
            )
        })
        .unwrap_or_default();
    format!(
        r#"<div source-tag="div" display="block" position="relative" writing-mode="vertical-rl" direction="rtl" font-family="ahem" font-size="10px" line-height="0px"{capture}>
      <div source-tag="span" display="inline-block" writing-mode="vertical-rl" direction="rtl" font-family="ahem" font-size="10px" line-height="0px" width="20px" height="20px"/>
      <div source-tag="br" display="inline" line-control="forced-break" writing-mode="vertical-rl" direction="rtl" font-family="ahem" font-size="10px" line-height="0px" inline-baseline="0px" inline-line-height="0px"/>
      <div source-tag="span" display="inline-block" writing-mode="vertical-rl" direction="rtl" font-family="ahem" font-size="10px" line-height="0px" width="20px" height="20px"/>
      <atomic-placeholder child-index="0" bidi-level="1" following-break="prohibited"/>
      <atomic-placeholder child-index="2" bidi-level="1" following-break="prohibited"/>
    </div>"#
    )
}
fn parse(raw: &str) -> Result<Node, Error> {
    let doc = roxmltree::Document::parse(raw).unwrap();
    parse_node(doc.root_element())
}
#[test]
fn named_atomic_certificate_binds_actual_paragraph_and_br_inputs() {
    let root = parse(&source(Some(&certificate()))).unwrap();
    assert_named_atomic_paragraph_inputs(&root).unwrap();
    let explicit_initial = source(Some(&certificate())).replace(
        "font-family=\"ahem\"",
        "vertical-align=\"baseline\" font-family=\"ahem\"",
    );
    assert_named_atomic_paragraph_inputs(&parse(&explicit_initial).unwrap()).unwrap();
    let physical = TestTree::from_golden(&parse(&source(None)).unwrap(), None).unwrap();
    assert_eq!(
        physical.nodes[0]
            .layout_input
            .as_box()
            .unwrap()
            .inline_baseline_context,
        layout::InlineBaselineContext::Physical
    );
    assert_eq!(
        physical.nodes[2]
            .layout_input
            .as_line_break()
            .unwrap()
            .metrics()
            .reference(),
        layout::InlineBaselineContext::Physical
    );
}
#[test]
fn named_atomic_reader_rejects_real_child_and_reference_mismatches() {
    let raw = source(Some(&certificate()));
    for invalid in [
        raw.replacen("source-tag=\"span\"", "source-tag=\"div\"", 1),
        raw.replacen("bidi-level=\"1\"", "bidi-level=\"0\"", 1),
        raw.replace("inline-baseline=\"0px\"", "inline-baseline=\"1px\""),
        raw.replace("line-control=\"forced-break\"", "line-control=\"other\""),
        raw.replacen(
            "display=\"block\"",
            "display=\"block\" vertical-align=\"middle\"",
            1,
        ),
        raw.replacen(
            "display=\"inline-block\"",
            "display=\"inline-block\" vertical-align=\"middle\"",
            1,
        ),
        raw.replacen(
            "display=\"inline\"",
            "display=\"inline\" vertical-align=\"middle\"",
            1,
        ),
    ] {
        assert!(parse(&invalid).is_err(), "{invalid}");
    }
    let mut missing = certificate();
    missing["order"][1]["child"] = json!(2);
    assert!(parse(&source(Some(&missing))).is_err());
    let mut asymmetric = certificate();
    asymmetric["before"] = json!(3);
    asymmetric["after"] = json!(-3);
    assert!(parse(&source(Some(&asymmetric))).is_err());
}
#[test]
fn checked_zero_central_metrics_work_in_both_scalar_lanes() {
    fn lane<S: layout::LayoutScalar>() {
        let reference = layout::InlineBaselineContext::Named(layout::BaselineName::Central);
        let metrics = layout::InlineMetricsOf::<S>::try_new(reference, S::ZERO, S::ZERO).unwrap();
        assert_eq!(metrics.reference(), reference);
        assert_eq!(metrics.after_baseline(), S::ZERO);
    }
    lane::<f32>();
    lane::<f64>();
}
