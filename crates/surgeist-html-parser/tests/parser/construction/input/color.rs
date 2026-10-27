use crate::parser::support::limits;
use surgeist_color::{Color, ColorSpace, PredefinedColorSpace};
use surgeist_html::*;
use surgeist_html_parser::{
    HtmlColorProcessingError, HtmlDocumentParseRequest, HtmlParseColorOperandRole,
    HtmlParseColorValueSource, HtmlParseCompletion, HtmlParseFailureReason, HtmlParseHistoryFrame,
    HtmlParseHistoryStep, HtmlParseLimits, HtmlParseMetadataStep, HtmlParseReadiness,
    HtmlParseResource, HtmlParseStop, HtmlSource,
};

#[test]
fn color_input_preserves_red_default_and_publishes_sanitized_current_with_form_sibling() {
    // HTML877d #update-a-color-well-control-color and
    // #serialize-a-color-well-control-color: the fresh clean value uses the
    // authored attribute. CSS Color 4's red is opaque sRGB (255, 0, 0); absent
    // alpha and the default Limited sRGB state request lowercase HTML hex.
    let literal = "<!doctype html><form id=f><input type=color value=red>Z</form>";
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
    let report = request
        .parse()
        .expect("ordinary Color input must complete its source-defined value update");
    let state = report.document_state().unwrap();
    assert_eq!(state.readiness(), HtmlParseReadiness::Complete);
    assert_eq!(state.completion(), HtmlParseCompletion::Complete);
    assert!(state.ready_for_post_load_tasks());
    assert_eq!(report.source().as_utf8(), Some(literal));

    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
    let roots: Vec<_> = document.root().children().collect();
    assert_eq!(roots.len(), 2);
    assert!(matches!(
        document.node(&roots[0]).unwrap().kind(),
        HtmlNodeKind::Doctype(_)
    ));
    let html = document.node(&roots[1]).unwrap();
    let branches: Vec<_> = html.children().collect();
    assert_eq!(branches.len(), 2);
    let body = document.node(&branches[1]).unwrap();
    let forms: Vec<_> = body.children().collect();
    assert_eq!(forms.len(), 1);
    let form = document.node(&forms[0]).unwrap();
    let HtmlNodeKind::Element(form_element) = form.kind() else {
        panic!("native form")
    };
    assert_eq!(
        form_element.name().local_name(),
        &DocumentString::from("form")
    );
    assert_eq!(
        form_element.attributes().as_slice()[0].value(),
        &DocumentString::from("f")
    );
    let children: Vec<_> = form.children().collect();
    assert_eq!(children.len(), 2);
    let input = document.node(&children[0]).unwrap();
    let HtmlNodeKind::Element(element) = input.kind() else {
        panic!("native input")
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("input"));
    assert_eq!(element.attributes().len(), 2);
    for (attribute, (name, value)) in element
        .attributes()
        .iter()
        .zip([("type", "color"), ("value", "red")])
    {
        assert_eq!(attribute.name().namespace().uri(), None);
        assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
        assert_eq!(attribute.value(), &DocumentString::from(value));
    }
    assert_eq!(input.children().count(), 0);
    assert!(matches!(
        document.node(&children[1]).unwrap().kind(),
        HtmlNodeKind::Text(value) if value == &DocumentString::from("Z")
    ));

    let (mut inputs, mut owners) = (0, 0);
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if record.node().id() != input.id() {
                    continue;
                }
                if let Some(flags) = record.input_flags() {
                    inputs += 1;
                    assert_eq!(flags.input_type(), HtmlInputType::Color);
                    assert_eq!(flags.input_type().value_mode(), HtmlInputValueMode::Value);
                    assert!(!flags.dirty_value());
                    assert!(!flags.dirty_checkedness());
                    assert!(!flags.checkedness());
                    assert!(!flags.indeterminate());
                    assert!(!flags.user_validity());
                    assert_eq!(
                        record.input_raw_value(),
                        Some(&DocumentString::from("#ff0000"))
                    );
                    assert_eq!(record.input_creation_writeback(), Some(false));
                }
                if let Some(owner) = record.form_owner() {
                    owners += 1;
                    assert_eq!(owner.unwrap().id(), form.id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!((inputs, owners), (1, 1));
}

#[test]
fn color_currentcolor_without_palette_is_unavailable_and_reclaims_partial_construction() {
    let source = HtmlSource::utf8("<!doctype html><p>A&amp;B<input type=color value=currentcolor>");
    let failure = HtmlDocumentParseRequest::offline(source.clone(), limits())
        .parse()
        .unwrap_err();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::ColorProcessing(
            HtmlColorProcessingError::Unavailable(
                surgeist_css::color::CssColorUnavailable::Palette(
                    surgeist_css::color::CssSystemColor::CanvasText
                )
            )
        ))
    );
    assert_eq!(failure.cleanup_error(), None);
    assert_eq!(failure.source().identity(), source.identity());
    for resource in [
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::OpenElements,
    ] {
        assert_eq!(
            failure
                .retained_usage()
                .unwrap()
                .resource(resource)
                .current(),
            0
        );
    }
}

#[test]
fn color_independent_literals_preserve_defaults_and_source_ordered_alpha_colorspace() {
    // HTML877d updates each installed relevant attribute from the clean default,
    // then resets. Thus attribute ordering does not destroy authored alpha.
    // sRGB 0.5 * 255 is 127.5: source rounding toward +infinity yields 128.
    for (attributes, expected_default, expected_current) in [
        ("type=color", None, "#000000"),
        ("type=color value=''", Some(""), "#000000"),
        ("type=color value=garbage", Some("garbage"), "#000000"),
        (
            "type=color value='hsl(120 100% 25%)'",
            Some("hsl(120 100% 25%)"),
            "#008000",
        ),
        (
            "type=color value='hwb(240 0% 0%)'",
            Some("hwb(240 0% 0%)"),
            "#0000ff",
        ),
        (
            "type=color value='rgb(50% 0% 100%)'",
            Some("rgb(50% 0% 100%)"),
            "#8000ff",
        ),
        (
            "type=color value='rgb(255 0 0 / .5)'",
            Some("rgb(255 0 0 / .5)"),
            "#ff0000",
        ),
        (
            "type=color alpha value='rgb(255 0 0 / .5)'",
            Some("rgb(255 0 0 / .5)"),
            "color(srgb 1 0 0 / 0.5)",
        ),
        (
            "value='rgb(255 0 0 / .5)' type=color alpha",
            Some("rgb(255 0 0 / .5)"),
            "color(srgb 1 0 0 / 0.5)",
        ),
        (
            "type=color alpha value=red",
            Some("red"),
            "color(srgb 1 0 0)",
        ),
        (
            "type=color colorspace=display-p3 value='color(display-p3 1 0 0 / .5)'",
            Some("color(display-p3 1 0 0 / .5)"),
            "color(display-p3 1 0 0)",
        ),
        (
            "value='color(display-p3 1 0 0 / .5)' alpha colorspace=DISPLAY-P3 type=color",
            Some("color(display-p3 1 0 0 / .5)"),
            "color(display-p3 1 0 0 / 0.5)",
        ),
        (
            "type=color colorspace=' display-p3' value=red",
            Some("red"),
            "#ff0000",
        ),
        (
            "type=color colorspace=unknown value=red",
            Some("red"),
            "#ff0000",
        ),
        (
            "type=color value=' rgb(255 0 0) '",
            Some(" rgb(255 0 0) "),
            "#ff0000",
        ),
    ] {
        let literal = format!("<!doctype html><input {attributes}>Z");
        assert!(attributes.len() < 96);
        assert!(expected_default.is_none_or(|value| value.encode_utf16().count() < 64));
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), matrix_limits())
            .parse()
            .unwrap_or_else(|error| panic!("{attributes}: {error:?}"));
        let document = report.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let input = document.node(&body).unwrap().children().next().unwrap();
        let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
            panic!("native Color input");
        };
        let default = element
            .attributes()
            .iter()
            .find(|a| a.name().local_name() == &DocumentString::from("value"))
            .map(Attribute::value);
        assert_eq!(
            default,
            expected_default.map(DocumentString::from).as_ref(),
            "{attributes}"
        );
        let mut metadata = report.metadata();
        let mut count = 0;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) if record.node().id() == input => {
                    if let Some(flags) = record.input_flags() {
                        count += 1;
                        assert_eq!(flags.input_type(), HtmlInputType::Color);
                        assert!(!flags.dirty_value());
                        assert!(!flags.input_type().is_selectable());
                        assert_eq!(
                            record.input_raw_value(),
                            Some(&DocumentString::from(expected_current)),
                            "{attributes}"
                        );
                        assert_eq!(record.input_creation_writeback(), Some(false));
                    }
                }
                HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(count, 1);
    }
}

#[test]
fn color_utf16_reference_default_and_generated_history_keep_distinct_actual_values() {
    let literal =
        "<!doctype html><input type=color value='r&#101;d' alpha colorspace=limited-srgb>Z";
    let units: Vec<u16> = literal.encode_utf16().collect();
    let source = HtmlSource::utf16(DocumentString::from_utf16(units.clone()));
    let report = HtmlDocumentParseRequest::offline(source.clone(), matrix_limits())
        .parse()
        .unwrap();
    assert_eq!(
        report.source().as_utf16().map(DocumentString::code_units),
        Some(units.as_slice())
    );
    let mut metadata = report.metadata();
    let mut inputs = 0;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) if row.input_flags().is_some() => {
                inputs += 1;
                assert_eq!(
                    row.input_flags().unwrap().input_type(),
                    HtmlInputType::Color
                );
                assert_eq!(
                    row.input_raw_value(),
                    Some(&DocumentString::from("color(srgb 1 0 0)"))
                );
            }
            HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(inputs, 1);
    let history = report.history_view();
    let mut found = false;
    for index in 0..history.len() {
        let header = history.header(history.group(index).unwrap()).unwrap();
        let Some(color) = header.cause.color_computation() else {
            continue;
        };
        if !color.alpha_present() {
            continue;
        }
        found = true;
        assert_eq!(color.source(), HtmlParseColorValueSource::InstalledValue);
        assert_eq!(color.colorspace(), HtmlInputColorSpace::LimitedSrgb);
        assert_eq!(color.standard_failure(), None);
        let value: Color = color.value();
        assert_eq!(value.srgb_bytes(), Some([255, 0, 0]));
        assert_eq!(
            value.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb)
        );
        assert_eq!(
            value.channels(),
            Color::from_srgba8([255, 0, 0, 255]).channels()
        );
        assert_eq!(value.alpha(), Some(1.0));
        assert!(
            color
                .operand_group(HtmlParseColorOperandRole::Alpha)
                .is_some()
        );
        let original = color.original_group().unwrap();
        let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
        let mut cursor = history.cursor(original, &mut scratch, None).unwrap();
        let mut text = Vec::new();
        loop {
            match cursor.advance() {
                HtmlParseHistoryStep::Leaf(leaf) => text.extend_from_slice(leaf.units()),
                HtmlParseHistoryStep::Progress => {}
                HtmlParseHistoryStep::Complete => break,
                HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
            }
        }
        assert_eq!(text, [114, 101, 100]);
    }
    assert!(found);
}

// A distinct conservative verification envelope for the short literal matrix;
// this does not alter the published red fixture's 100M admission or product
// defaults. All arithmetic is checked before the first matrix execution.
// CSS bounded_decimal: at most64 searches,17 seed digits,1075 multiplications,
// and1100 scratch digits. Each cell is a separate read/write, with carry and
// fixed kernels. Four components, five source-required updates, and the
// corresponding finite serialization bound are accounted independently. The
// factor16 bounds fixed grammar/evaluation/slot handoffs around these requests;
// reserve/copy allowance is included in per-action compiled slot/layout cost.
fn matrix_limits() -> HtmlParseLimits {
    use surgeist_color::Serialization;
    use surgeist_css::color::controlled::{
        CssColorParse, CssColorSlot, CssColorTransform, CssUsedColorCheckpoint,
    };
    let register = [
        CssColorParse::register_bytes(),
        Serialization::register_bytes(),
        CssColorTransform::register_bytes(),
    ]
    .into_iter()
    .max()
    .unwrap() as u64;
    // A transformation has no separate action: its frame charges only the
    // register and the resulting checkpoint.
    let guard = [
        CssColorParse::action_bytes(),
        Serialization::action_bytes(),
        std::mem::size_of::<CssUsedColorCheckpoint<'_>>(),
    ]
    .into_iter()
    .max()
    .unwrap() as u64;
    let cost = register
        .checked_mul(2)
        .unwrap()
        .checked_add(guard)
        .unwrap()
        .checked_add(
            (std::mem::size_of::<CssColorSlot<'_>>() as u64)
                .checked_mul(16)
                .unwrap(),
        )
        .unwrap()
        .checked_add(4096)
        .unwrap();
    let boundary = 1075_u64
        .checked_mul(2 * 1100 + 4)
        .unwrap()
        .checked_add(4 * 1100 + 2 * 17 + 8)
        .unwrap();
    let ingress = 4_u64
        .checked_mul(64)
        .unwrap()
        .checked_mul(boundary)
        .unwrap();
    let formatting = 8_u64.checked_mul(1075).unwrap().checked_mul(1100).unwrap();
    let actions = 5_u64
        .checked_mul(
            ingress
                .checked_add(formatting)
                .unwrap()
                .checked_add(4096)
                .unwrap(),
        )
        .unwrap()
        .checked_mul(16)
        .unwrap();
    let credits = actions
        .checked_mul(cost)
        .unwrap()
        .checked_add(100_000_000)
        .unwrap();
    limits()
        .into_builder()
        .work_credits(credits)
        .finish()
        .unwrap()
}

#[test]
fn color_serialization_intrinsic_decisions_have_source_order_and_total_type_identity() {
    for (alpha, space, effects, html_compatible) in [
        (
            false,
            HtmlInputColorSpace::LimitedSrgb,
            vec![
                HtmlInputColorEffect::ForceOpaqueAlpha,
                HtmlInputColorEffect::Convert(HtmlInputColorSpace::LimitedSrgb),
                HtmlInputColorEffect::QuantizeSrgb,
            ],
            true,
        ),
        (
            true,
            HtmlInputColorSpace::LimitedSrgb,
            vec![
                HtmlInputColorEffect::Convert(HtmlInputColorSpace::LimitedSrgb),
                HtmlInputColorEffect::QuantizeSrgb,
                HtmlInputColorEffect::ColorFunction,
            ],
            false,
        ),
        (
            false,
            HtmlInputColorSpace::DisplayP3,
            vec![
                HtmlInputColorEffect::ForceOpaqueAlpha,
                HtmlInputColorEffect::Convert(HtmlInputColorSpace::DisplayP3),
            ],
            false,
        ),
        (
            true,
            HtmlInputColorSpace::DisplayP3,
            vec![HtmlInputColorEffect::Convert(
                HtmlInputColorSpace::DisplayP3,
            )],
            false,
        ),
    ] {
        let mut sequence = HtmlInputColorSerialization::new(alpha, space);
        let actual: Vec<_> = std::iter::from_fn(|| sequence.next_effect()).collect();
        assert_eq!(actual, effects);
        assert_eq!(sequence.html_compatible(), html_compatible);
        assert_eq!(sequence.next_effect(), None);
    }
    assert_eq!(HtmlInputType::Color.value_mode(), HtmlInputValueMode::Value);
    assert_eq!(
        HtmlInputType::Color.sanitization(),
        HtmlInputValueSanitization::Color
    );
    assert!(!HtmlInputType::Color.is_selectable());
}
