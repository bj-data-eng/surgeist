//! Cross-owner cumulative bridge contracts, with independently authored text.
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::*;
use CssSpecifiedValueSerializationErrorKind as Kind;
use CssSpecifiedValueSerializationLimits as Limits;
fn compose(writer: &mut SpecifiedRuleWriter) -> Result<(), CssSpecifiedValueSerializationError> {
    writer.append("start ")?;
    CssColor::transparent().append_specified(&mut writer.context, &mut writer.css)?;
    writer.append(" ")?;
    let report = parse_style_attribute("opacity:.25");
    let CssKnownPropertyValueRef::Opacity(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("opacity")
    };
    value.value().append_to_rule_writer(writer)?;
    writer.append(" ")?;
    CssBoxSizing::BorderBox.append_to_rule_writer(writer)?;
    writer.append(" ")?;
    writer.selector(&CssSelector::Class("x".into()))
}
#[test]
fn exact_shared_nodes_and_bytes_charge_final_text_once() {
    let expected = "start transparent 0.25 border-box .x";
    let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, expected.len()));
    compose(&mut writer).unwrap();
    assert_eq!(writer.css, expected);
    for (limits, kind) in [
        (Limits::new(3, 4, expected.len()), Kind::InputNodeLimit),
        (Limits::new(4, 3, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, expected.len() - 1), Kind::ByteLimit),
    ] {
        let mut writer = SpecifiedRuleWriter::new(limits);
        assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
    }
}
fn emit(
    writer: &mut SpecifiedRuleWriter,
    source: &str,
) -> Result<(), CssSpecifiedValueSerializationError> {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    match report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    {
        CssKnownPropertyValueRef::ColumnRule(v) => v.rule().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::FontVariant(v) => v.variant().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::FontSynthesis(v) => v.synthesis().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::CounterIncrement(v) => v
            .value()
            .append_to_rule_writer(CssCounterProperty::Increment, writer),
        CssKnownPropertyValueRef::CounterReset(v) => v
            .value()
            .append_to_rule_writer(CssCounterProperty::Reset, writer),
        CssKnownPropertyValueRef::Margin(v) => v.value().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::Flex(v) => v.value().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::GridTemplateAreas(v) => v.value().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::Opacity(v) => v.value().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::Order(v) => v.value().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::AspectRatio(v) => v.ratio().append_to_rule_writer(writer),
        CssKnownPropertyValueRef::Pause(v) => v.value().append_to_rule_writer(writer),
        _ => panic!("selected owner"),
    }
}

// Suppression consumes semantic work, but neither formats nor emits final text.
// The exact visit counts follow the retained scalar or pair graph, independently
// of text length and of the enclosing suppression mode.
fn assert_suppressed_work(
    emit: fn(&mut SpecifiedRuleWriter) -> Result<(), CssSpecifiedValueSerializationError>,
    expected: &str,
    nodes: usize,
) {
    let mut visible = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, expected.len()));
    emit(&mut visible).unwrap();
    assert_eq!(visible.css, expected);

    let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
    writer.without_output(emit).unwrap();
    assert!(writer.css.is_empty());
    assert!(!writer.context.output_suppressed());
    assert_eq!(
        CssBoxSizing::BorderBox
            .append_to_rule_writer(&mut writer)
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );

    for (limits, kind) in [
        (Limits::new(nodes - 1, nodes, 64), Kind::InputNodeLimit),
        (Limits::new(nodes, nodes - 1, 64), Kind::ProjectionNodeLimit),
    ] {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer
            .without_output(|writer| {
                assert_eq!(emit(writer).unwrap_err().kind(), kind);
                assert!(writer.context.output_suppressed());
                assert!(writer.css.is_empty());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
    }
}

#[test]
fn suppressed_opacity_scalar_has_no_final_byte_cost() {
    assert_suppressed_work(|writer| emit(writer, "opacity:.25"), "0.25", 1);
}

#[test]
fn suppressed_flex_literal_has_no_final_byte_cost() {
    assert_suppressed_work(
        |writer| {
            CssSpecifiedNonNegativeFlex::try_from_component(
                CssComponentValue::try_dimension("1", "fr").unwrap(),
            )
            .unwrap()
            .append_to_rule_writer(writer)
        },
        "1fr",
        1,
    );
}

#[test]
fn suppressed_integer_literal_has_no_final_byte_cost() {
    assert_suppressed_work(|writer| emit(writer, "order:1"), "1", 1);
}

#[test]
fn suppressed_ratio_operands_have_no_final_byte_cost() {
    assert_suppressed_work(|writer| emit(writer, "aspect-ratio:1/2"), "1 / 2", 2);
}

#[test]
fn suppressed_unequal_pause_pair_preserves_enclosing_mode_and_work() {
    assert_suppressed_work(|writer| emit(writer, "pause:1s 2s"), "1s 2s", 3);
}

#[test]
fn unequal_pause_pair_restores_outer_suppression_before_returning_child_error() {
    let mut writer = SpecifiedRuleWriter::new(Limits::new(3, 2, 64));
    writer
        .without_output(|writer| {
            assert_eq!(
                emit(writer, "pause:1s 2s").unwrap_err().kind(),
                Kind::ProjectionNodeLimit
            );
            assert!(writer.context.output_suppressed());
            assert!(writer.css.is_empty());
            assert_eq!(
                CssBoxSizing::BorderBox
                    .append_to_rule_writer(writer)
                    .unwrap_err()
                    .kind(),
                Kind::InputNodeLimit
            );
            Ok(())
        })
        .unwrap();
    assert!(!writer.context.output_suppressed());
}

fn emit_font_range(
    writer: &mut SpecifiedRuleWriter,
    source: &str,
    kind: CssFontFaceDescriptorKind,
) -> Result<(), CssSpecifiedValueSerializationError> {
    let report = parse_font_face_descriptor_value(source, kind);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(value)) = report.syntax() else {
        panic!("ordinary font range")
    };
    match value {
        CssFontFaceDescriptorValue::FontStyle(value) => value.append_to_rule_writer(writer),
        CssFontFaceDescriptorValue::FontWidth(value) => value.append_to_rule_writer(writer),
        CssFontFaceDescriptorValue::FontWeight(value) => value.append_to_rule_writer(writer),
        _ => panic!("selected font range"),
    }
}

#[test]
fn unequal_font_ranges_preserve_outer_suppression_even_on_child_failure() {
    assert_suppressed_work(
        |writer| emit_font_range(writer, "400 500", CssFontFaceDescriptorKind::FontWeight),
        "400 500",
        3,
    );
    assert_suppressed_work(
        |writer| emit_font_range(writer, "75% 100%", CssFontFaceDescriptorKind::FontWidth),
        "75% 100%",
        3,
    );
    assert_suppressed_work(
        |writer| {
            emit_font_range(
                writer,
                "oblique 10deg 20deg",
                CssFontFaceDescriptorKind::FontStyle,
            )
        },
        "oblique 10deg 20deg",
        3,
    );
}
#[test]
fn providers_start_their_own_value_segment_after_an_enclosing_prefix() {
    for (source, expected) in [
        ("column-rule:solid", "solid"),
        ("column-rule:red", "red"),
        (
            "font-variant:small-caps oldstyle-nums",
            "small-caps oldstyle-nums",
        ),
        ("font-synthesis:weight style", "weight style"),
        ("counter-increment:x", "x 1"),
        ("counter-reset:x", "x 0"),
        ("margin:1px 2px", "1px 2px"),
        ("flex:1 2 10px", "1 2 10px"),
        ("grid-template-areas:'a a' 'b c'", "\"a a\" \"b c\""),
    ] {
        let mut writer = SpecifiedRuleWriter::new(Limits::default());
        writer.append("value: ").unwrap();
        emit(&mut writer, source).unwrap();
        assert_eq!(writer.css, format!("value: {expected}"), "{source}");
    }
}
#[test]
fn suppressed_values_preserve_work_and_restore_enclosing_mode() {
    let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, 1));
    writer
        .without_output(|writer| emit(writer, "font-synthesis:weight style"))
        .unwrap();
    assert!(!writer.context.output_suppressed());
    assert!(writer.css.is_empty());
    assert_eq!(
        CssBoxSizing::BorderBox
            .append_to_rule_writer(&mut writer)
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    writer.append("x").unwrap();
    assert_eq!(writer.css, "x");
}

#[test]
fn grid_auto_flow_reuses_final_output_storage_and_cumulative_resources() {
    for (value, expected, nodes) in [
        (CssGridAutoFlow::Normal, "normal", 1),
        (CssGridAutoFlow::Dense, "dense", 1),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, false),
            "row",
            1,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, false),
            "column",
            1,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
            "row dense",
            2,
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
            "column dense",
            2,
        ),
    ] {
        let full = format!("prefix {expected}");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, full.len()));
        writer.append("prefix ").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, full);
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, full.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, full.len()),
                Kind::ProjectionNodeLimit,
            ),
            (Limits::new(nodes, nodes, full.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("prefix ").unwrap();
            assert_eq!(
                value.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
        }
        let mut writer = SpecifiedRuleWriter::new(Limits::new(nodes, nodes, 0));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        assert_eq!(
            CssBoxSizing::BorderBox
                .append_to_rule_writer(&mut writer)
                .unwrap_err()
                .kind(),
            Kind::InputNodeLimit
        );
    }
}
