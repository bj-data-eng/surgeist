#![forbid(unsafe_code)]
//! Property-specific canonical size text follows CSSWG resolution 7802 (2025-04-01):
//! https://github.com/w3c/csswg-drafts/issues/7802#issuecomment-2770612154
//! Independent examples: WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d,
//! LayoutTests/imported/w3c/web-platform-tests/css/css-backgrounds/parsing/background-size-valid.html.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as E, CssSpecifiedValueSerializationLimits as L, *,
};

fn parsed(property: &str, css: &str) -> CssBackgroundSizeList {
    let report = parse_style_attribute(&format!("{property}:{css}"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    match report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    {
        CssKnownPropertyValueRef::BackgroundSize(value) => value.sizes().clone(),
        CssKnownPropertyValueRef::MaskSize(value) => value.sizes().clone(),
        _ => panic!("size property"),
    }
}

fn typed(height: Option<CssBackgroundSizeComponent>) -> CssBackgroundSize {
    let width = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token("1px").unwrap(),
    )
    .unwrap();
    assert_eq!(width.origin(), &CssValueOrigin::Programmatic);
    CssBackgroundSize::Explicit {
        width: CssBackgroundSizeComponent::Length(width),
        height,
    }
}

#[test]
fn background_omitted_height_serializes_effective_auto() {
    assert_eq!(
        parsed("background-size", "1px")
            .serialize_specified()
            .unwrap(),
        "1px auto"
    );
}

#[test]
fn background_explicit_auto_height_remains_in_serialized_text() {
    assert_eq!(
        parsed("background-size", "1px auto")
            .serialize_specified()
            .unwrap(),
        "1px auto"
    );
}

#[test]
fn mask_omitted_height_serializes_effective_auto() {
    assert_eq!(
        parsed("mask-size", "1px").serialize_specified().unwrap(),
        "1px auto"
    );
}

#[test]
fn mask_explicit_auto_height_remains_in_serialized_text() {
    assert_eq!(
        parsed("mask-size", "1px auto")
            .serialize_specified()
            .unwrap(),
        "1px auto"
    );
}

fn exact_bounds(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    for (limits, error) in [
        (
            L::new(input - 1, projection, expected.len()),
            E::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            E::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), E::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), error);
    }
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
}

#[test]
fn checked_scalar_omitted_height_charges_generated_projection_without_authored_input() {
    let value = typed(None);
    // Size, width component, numeric literal are authored; effective auto is projection only.
    exact_bounds("1px auto", 3, 4, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value, typed(None));
}

#[test]
fn checked_scalar_explicit_auto_is_emitted_without_double_charging() {
    let value = typed(Some(CssBackgroundSizeComponent::Auto));
    exact_bounds("1px auto", 4, 4, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value, typed(Some(CssBackgroundSizeComponent::Auto)));
}

#[test]
fn mask_list_shares_generated_defaults_and_separators_in_one_budget() {
    let value = parsed("mask-size", "1px, 1px");
    // One list + two three-node authored layers; each generated auto adds one projection.
    exact_bounds("1px auto, 1px auto", 7, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value.serialize_specified().unwrap(), "1px auto, 1px auto");
}

#[test]
fn checked_list_combines_generated_and_explicit_auto_node_budgets() {
    let value = CssBackgroundSizeList::try_new(vec![
        typed(None),
        typed(Some(CssBackgroundSizeComponent::Auto)),
    ])
    .unwrap();
    exact_bounds("1px auto, 1px auto", 8, 9, |limits| {
        value.serialize_specified_with_limits(limits)
    });
}

#[test]
fn symbolic_width_stays_unresolved_while_effective_height_is_serialized() {
    let value = parsed("background-size", "calc(1px + 2em), calc(1px + 2em)");
    // Each math has four input/five projection nodes, plus size/component and generated auto.
    exact_bounds(
        "calc(2em + 1px) auto, calc(2em + 1px) auto",
        13,
        17,
        |limits| value.serialize_specified_with_limits(limits),
    );
    for layer in value.sizes() {
        let CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height: None,
        } = layer
        else {
            panic!("authored symbolic width with absent height")
        };
        assert!(width.calculation().is_some());
        assert!(width.literal_component().is_none());
    }
}

#[test]
fn serialization_success_and_failure_preserve_authored_height_and_numeric_provenance() {
    for authored in ["1px", "1px auto"] {
        let value = parsed("background-size", authored);
        let original = value.clone();
        let CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height,
        } = &value.sizes()[0]
        else {
            panic!("authored numeric size")
        };
        let origin = width.origin().clone();
        assert!(matches!(origin, CssValueOrigin::Parsed(_)));
        assert_eq!(height.is_some(), authored.ends_with(" auto"));
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(100, 100, 0))
                .unwrap_err()
                .kind(),
            E::ByteLimit
        );
        assert_eq!(value, original);
        let output = value.serialize_specified().unwrap();
        assert_eq!(value, original);
        let CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            ..
        } = &value.sizes()[0]
        else {
            panic!("numeric width retained")
        };
        assert_eq!(width.origin(), &origin);
        assert_eq!(output, "1px auto");
    }
}

#[test]
fn auto_width_and_other_size_forms_keep_their_canonical_controls() {
    for (authored, expected) in [
        ("auto", "auto"),
        ("auto auto", "auto"),
        ("auto 4%", "auto 4%"),
        ("2% 3%", "2% 3%"),
        ("cover", "cover"),
        ("contain", "contain"),
    ] {
        let value = parsed("background-size", authored);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value.sizes()[0].serialize_specified().unwrap(), expected);
    }
    let explicit_auto = CssBackgroundSize::Explicit {
        width: CssBackgroundSizeComponent::Auto,
        height: Some(CssBackgroundSizeComponent::Auto),
    };
    // Collapsing auto/auto still visits both authored components.
    exact_bounds("auto", 3, 3, |limits| {
        explicit_auto.serialize_specified_with_limits(limits)
    });
}
