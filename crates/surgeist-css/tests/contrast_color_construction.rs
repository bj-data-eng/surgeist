#![forbid(unsafe_code)]
//! Color 5 WD20260908 §8 preserves one input; function nodes compose within 256 levels.
use surgeist_css::*;

fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}

fn contrast(color: CssColor) -> CssColor {
    CssColor::from_contrast_color(CssContrastColor::try_new(color).unwrap())
}

fn numeric_color(depth: usize, alpha: bool) -> CssColor {
    let text = format!("{}50%{}", "calc(".repeat(depth), ")".repeat(depth));
    let raw = parse_component_values(&text).unwrap();
    assert_eq!(raw.nesting_depth() as usize, depth);
    let calculation = CssColorComponent::PercentageCalculation(
        CssPercentageCalculation::try_from_components(raw).unwrap(),
    );
    let zero = CssColorComponent::Number(
        CssColorNumberLiteral::try_from_component(
            parse_component_values("0").unwrap().items()[0].clone(),
        )
        .unwrap(),
    );
    let (channels, alpha) = if alpha {
        ([zero.clone(), zero.clone(), zero], Some(calculation))
    } else {
        ([calculation, zero.clone(), zero], None)
    };
    CssColor::from_rgb(CssRgbColor::try_new(CssColorSyntax::Modern, channels, alpha).unwrap())
}

#[test]
fn checked_payload_retains_independent_input_and_exposes_one_semantic_accessor() {
    for input in [
        named("red"),
        CssColor::current_color(),
        CssColor::from_system(CssSystemColor::CanvasText),
    ] {
        let payload = CssContrastColor::try_new(input.clone()).unwrap();
        assert_eq!(payload.color(), &input);
        let value = CssColor::from_contrast_color(payload.clone());
        assert_eq!(value.kind_name(), "contrast-color");
        assert_eq!(value.contrast_color_value(), Some(&payload));
        assert!(input.contrast_color_value().is_none());
        assert!(value.light_dark_value().is_none());
        assert_eq!(
            value.absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::ContrastColor)
        );
    }
    assert_ne!(
        CssContrastColor::try_new(named("red")).unwrap(),
        CssContrastColor::try_new(named("blue")).unwrap()
    );
    assert_eq!(
        contrast(named("red")).to_specified_css().unwrap(),
        "contrast-color(red)"
    );
}

#[test]
fn checked_chains_admit_256_wrappers_and_reject_the_next() {
    let mut value = named("red");
    for _ in 0..256 {
        value = contrast(value);
    }
    assert_eq!(
        CssContrastColor::try_new(value).unwrap_err(),
        CssColorConstructionError::NestingLimit
    );
}

#[test]
fn constructor_counts_the_complete_numeric_channel_and_alpha_subtrees() {
    for alpha in [false, true] {
        // Contrast + RGB + 254 calc functions = 256, independently of channel role.
        let input = numeric_color(254, alpha);
        let value = contrast(input.clone());
        assert_eq!(value.contrast_color_value().unwrap().color(), &input);
        let expected = if alpha {
            "contrast-color(rgba(0, 0, 0, 0.5))"
        } else {
            "contrast-color(rgb(127.5, 0, 0))"
        };
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_eq!(
            CssContrastColor::try_new(value).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        // Existing RGB construction permits its own 256-level subtree, but
        // enclosing that independently valid child adds the forbidden 257th level.
        assert_eq!(
            CssContrastColor::try_new(numeric_color(255, alpha)).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
    }
}

#[test]
fn contrast_depth_composes_with_both_light_dark_branches_and_gradient_images() {
    let mut child = named("red");
    for _ in 0..255 {
        child = contrast(child);
    }
    for (light, dark) in [
        (child.clone(), named("blue")),
        (named("blue"), child.clone()),
    ] {
        let pair = CssColor::from_light_dark(CssLightDarkColor::try_new(light, dark).unwrap());
        assert_eq!(
            CssContrastColor::try_new(pair).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
    }
    let pair =
        CssColor::from_light_dark(CssLightDarkColor::try_new(named("red"), named("blue")).unwrap());
    assert_eq!(
        contrast(pair).to_specified_css().unwrap(),
        "contrast-color(light-dark(red, blue))"
    );
    let stops = CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(child, None))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            named("blue"),
            None,
        ))),
    ])
    .unwrap();
    let image = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(None, stops)));
    assert_eq!(
        CssLightDarkImage::try_new(image, CssImageValue::None).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
}

#[test]
fn metadata_identifies_the_complete_dated_authored_production_and_registered_consumers() {
    let metadata = feature_metadata("interop.value.contrast-color").unwrap();
    assert_eq!(metadata.kind(), CssFeatureKind::Value);
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    assert_eq!(metadata.production(), "#funcdef-contrast-color");
    assert_eq!(metadata.source().id().as_str(), "I-COLOR5-20260908");
    assert_eq!(
        metadata.source().url(),
        Some("https://www.w3.org/TR/2026/WD-css-color-5-20260908/")
    );
    assert!(metadata.supported_subset().is_none());
    assert!(metadata.unsupported_remainder().is_none());
    for source in [
        "color:contrast-color(red)",
        "background:contrast-color(red)",
        "mask-image:linear-gradient(contrast-color(red), blue)",
    ] {
        assert!(validate_style_attribute(source).is_ok());
    }
    assert_eq!(
        feature_metadata("ext.value.color-mix")
            .unwrap()
            .source()
            .id()
            .as_str(),
        "I-COLOR5"
    );
}
