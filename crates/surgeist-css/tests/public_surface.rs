use surgeist_css::{
    CssAnimationDirection, CssAuthoredColorComponent, CssAuthoredColorMix,
    CssAuthoredColorMixPercentage, CssAuthoredColorSyntax, CssAuthoredFontFeature,
    CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings, CssAuthoredFontFeatureValue,
    CssAuthoredSystemColor, CssCalcOperator, CssColorInterpolationMethod,
    CssColorInterpolationSpace, CssCounterStyleRange, CssCounterStyleSpeakAs, CssCounterSymbol,
    CssDefinedFalseMediaReason, CssErrorCode, CssExclusionReason, CssFeatureKind,
    CssFontFamilyNameKind, CssFontFeature, CssFontFeatureIndex, CssFontFeatureValue, CssFontSize,
    CssFontSizeAdjust, CssFontSizeLengthPercentage, CssFontSynthesis, CssFontSynthesisValues,
    CssFontVariantCaps, CssFontVariantEastAsianValues, CssFontVariantLigatureState,
    CssFontVariantLigatureValues, CssFontVariantNumericFigure, CssFontVariantNumericValues,
    CssFontVariantPosition, CssFontVariantValues, CssGenericFontFamily, CssGridAutoFlowAxis,
    CssHueInterpolationMethod, CssImportance, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLanguageRange, CssLength, CssLineHeightLengthPercentage,
    CssMediaConditionKind, CssMediaQuery, CssMediaQueryModifier, CssMediaType,
    CssNamespaceConstraint, CssNamespaceName, CssNamespacePrefix, CssOpenTypeTag,
    CssPredefinedColorSpace, CssPropertyNameRef, CssPseudoClass, CssPseudoElement,
    CssRecoveryAction, CssRelativeColorChannel, CssRelativeColorEnvironment,
    CssRelativeColorExpressionValue, CssRelativeColorFunction, CssRelativeColorResultDomain,
    CssRule, CssSelector, CssSelectorCombinator, CssSpecificationTier, CssSupportStatus,
    CssSupportsConditionKind, CssSupportsConditionList, ErrorKind, conformance_exclusion,
    feature_metadata, parse_sheet, parse_style_attribute, property_metadata, specification_source,
};

#[test]
fn public_surface_exposes_checked_private_field_namespace_models() {
    let report = parse_sheet("@namespace s\\76 g \"not a URI\";");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Namespace(rule)] = report.syntax().rules() else {
        panic!("expected namespace rule")
    };
    assert_eq!(rule.prefix().expect("decoded prefix").as_str(), "svg");
    assert_eq!(rule.name().as_str(), "not a URI");
    assert_eq!(rule.position().byte_offset().value(), 0);

    assert_eq!(
        CssNamespacePrefix::try_new("svg")
            .expect("one decoded identifier")
            .as_str(),
        "svg"
    );
    assert!(CssNamespacePrefix::try_new("svg icon").is_none());
    assert_eq!(CssNamespaceName::new("").as_str(), "");
}

#[test]
fn public_surface_exposes_checked_counter_style_descriptor_models() {
    let report = parse_sheet(concat!(
        "@counter-style complete { system: additive; negative: \"(\" \" )\"; ",
        "range: infinite -1, 1 infinite; pad: \"0\" 3; fallback: decimal; ",
        "additive-symbols: C 100, X 10, I 1; speak-as: spell-out; }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("expected retained counter style")
    };

    let negative = rule.descriptors().negative().unwrap();
    assert!(matches!(
        negative.prefix(),
        CssCounterSymbol::String(value) if value.as_str() == "("
    ));
    assert!(matches!(
        rule.descriptors().range().map(|value| value.value()),
        Some(CssCounterStyleRange::Ranges(ranges)) if ranges.ranges().len() == 2
    ));
    assert_eq!(rule.descriptors().pad().unwrap().minimum_length(), 3);
    assert_eq!(rule.descriptors().fallback().unwrap().as_str(), "decimal");
    assert_eq!(
        rule.descriptors().additive_symbols().unwrap().tuples()[0].weight(),
        100
    );
    assert!(matches!(
        rule.descriptors().speak_as().map(|value| value.value()),
        Some(CssCounterStyleSpeakAs::SpellOut)
    ));
}

#[test]
fn public_surface_exposes_namespace_qualified_selector_accessors_and_id_projection() {
    let report = parse_sheet(concat!(
        "@namespace svg \"urn:svg\";",
        "svg|a#primary[svg|href] { color: red; }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Namespace(_), CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected namespace and qualified style rules")
    };
    let CssSelector::Compound(selector) = rule.selector() else {
        panic!("expected qualified compound selector")
    };

    let type_selector = selector.type_selector().expect("qualified type selector");
    assert!(matches!(
        type_selector.namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(type_selector.local_name(), Some("a"));
    assert_eq!(selector.tag().map(String::as_str), Some("a"));
    assert_eq!(selector.ids(), ["primary"]);
    assert_eq!(selector.key().map(String::as_str), Some("primary"));

    let [attribute] = selector.attributes() else {
        panic!("expected qualified attribute selector")
    };
    assert!(matches!(
        attribute.namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(attribute.name().as_str(), "href");
}

#[test]
fn public_surface_exposes_checked_selectors3_language_pseudos_and_ordered_ids() {
    let report = parse_sheet("a#first#second:lang(e\\6e)::first-line { color: red; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected one retained style rule")
    };
    let CssSelector::Compound(selector) = rule.selector() else {
        panic!("expected compound selector")
    };

    assert_eq!(selector.ids(), ["first", "second"]);
    assert_eq!(selector.key().map(String::as_str), Some("second"));
    assert!(matches!(
        selector.pseudo_classes(),
        [CssPseudoClass::Lang(range)] if range.as_str() == "en"
    ));
    assert_eq!(
        selector
            .pseudo_elements()
            .expect("first-line pseudo-element")
            .pseudo_elements(),
        [CssPseudoElement::FirstLine]
    );

    assert_eq!(
        CssLanguageRange::try_new("en-US")
            .expect("one decoded CSS identifier")
            .as_str(),
        "en-US"
    );
    assert!(CssLanguageRange::try_new("").is_none());
    assert!(CssLanguageRange::try_new("en US").is_none());
}

#[test]
fn public_surface_exposes_private_field_supports_models_and_checked_lists() {
    let report = parse_sheet("@supports (display: grid) {} @supports selector(.x > .y) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(declaration), CssRule::Supports(selector)] = report.syntax().rules()
    else {
        panic!("expected supports rules");
    };
    assert!(matches!(
        declaration.condition().kind(),
        CssSupportsConditionKind::Declaration(_)
    ));
    assert!(matches!(
        selector.condition().kind(),
        CssSupportsConditionKind::Selector(_)
    ));
    assert!(CssSupportsConditionList::try_new(vec![declaration.condition().clone()]).is_none());
    let list = CssSupportsConditionList::try_new(vec![
        declaration.condition().clone(),
        selector.condition().clone(),
    ])
    .expect("two conditions form a checked operator list");
    assert_eq!(list.conditions().len(), 2);

    let import = parse_sheet("@import 'theme.css' supports(display: grid);");
    assert!(import.is_clean(), "{:?}", import.diagnostics());
    let [CssRule::Import(import)] = import.syntax().rules() else {
        panic!("expected import rule")
    };
    assert!(matches!(
        import
            .supports()
            .expect("import supports")
            .condition()
            .kind(),
        CssSupportsConditionKind::Declaration(_)
    ));
}

#[test]
fn public_surface_exposes_parser_owned_defined_false_media_models() {
    let source = "@media only F\\75ture and (unknown: calc(1foo + 2px)) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected retained media rule")
    };
    let [CssMediaQuery::Typed(query)] = rule.query().queries() else {
        panic!("expected typed unknown media query")
    };
    assert_eq!(query.media_type(), CssMediaType::Unknown);
    let unknown_type = query.unknown_media_type().expect("unknown type details");
    assert_eq!(unknown_type.as_css(), "F\\75ture");
    assert_eq!(
        unknown_type.reason(),
        CssDefinedFalseMediaReason::UnknownType
    );

    let condition = query.condition().expect("unknown type condition");
    let CssMediaConditionKind::DefinedFalse(defined_false) = condition.kind() else {
        panic!("expected defined-false condition")
    };
    assert_eq!(defined_false.as_css(), "(unknown: calc(1foo + 2px))");
    assert_eq!(
        defined_false.reason(),
        CssDefinedFalseMediaReason::UnknownFeature
    );
    assert_eq!(defined_false.position(), condition.position());
}

#[test]
fn public_surface_exposes_checked_core_font_models() {
    let report = parse_style_attribute("font-size: medium; font-family: serif");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FontSize(size) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-size");
    };
    let size_kind = match size.size() {
        CssFontSize::Medium => "medium",
        _ => "future font size",
    };
    assert_eq!(size_kind, "medium");

    let CssKnownPropertyValueRef::FontFamily(family) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-family");
    };
    let item = &family.families().families()[0];
    assert_eq!(item.kind(), CssFontFamilyNameKind::Generic);
    assert_eq!(item.generic_family(), Some(CssGenericFontFamily::Serif));

    assert!(CssFontSizeLengthPercentage::try_new(CssLength::try_px(-1.0).unwrap()).is_none());
    assert!(
        CssLineHeightLengthPercentage::try_new(CssLength::try_percent(-1.0).unwrap()).is_none()
    );
}

#[test]
fn public_surface_checks_current_opentype_construction_and_preserves_i01_construction() {
    let tag = CssOpenTypeTag::try_new("kern").expect("four ASCII characters");
    assert_eq!(tag.as_str(), "kern");
    assert!(CssOpenTypeTag::try_new("abc").is_none());
    assert!(CssOpenTypeTag::try_new("abcde").is_none());
    assert!(CssOpenTypeTag::try_new("éabc").is_none());
    assert!(CssOpenTypeTag::try_new("😀abc").is_none());

    let zero = CssFontFeatureIndex::try_new(0).expect("zero index");
    let positive = CssFontFeatureIndex::try_new(7).expect("positive index");
    assert_eq!(zero.value(), 0);
    assert_eq!(positive.value(), 7);
    assert!(CssFontFeatureIndex::try_new(-1).is_none());

    let feature = CssAuthoredFontFeature::new(tag, CssAuthoredFontFeatureValue::Index(positive));
    let list = CssAuthoredFontFeatureList::try_new(vec![feature]).expect("nonempty feature list");
    assert!(CssAuthoredFontFeatureList::try_new(Vec::new()).is_none());
    let settings = CssAuthoredFontFeatureSettings::Features(list);
    assert!(matches!(
        settings,
        CssAuthoredFontFeatureSettings::Features(_)
    ));

    let legacy = CssFontFeature::try_new("éabc", Some(CssFontFeatureValue::Integer(-1)))
        .expect("frozen I01 construction remains source-compatible");
    assert_eq!(legacy.tag(), "éabc");
    assert_eq!(legacy.value(), Some(CssFontFeatureValue::Integer(-1)));
}

#[test]
fn public_surface_exposes_checked_font_control_models() {
    assert!(CssFontSynthesisValues::try_new(false, false).is_none());
    let weight = CssFontSynthesisValues::try_new(true, false).expect("nonempty synthesis set");
    assert!(weight.weight());
    assert!(!weight.style());
    assert!(matches!(
        CssFontSynthesis::Values(weight),
        CssFontSynthesis::Values(values) if values.weight() && !values.style()
    ));

    let report = parse_style_attribute("font-size-adjust: none; font-synthesis: style weight");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FontSizeAdjust(adjust) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected font-size-adjust");
    };
    assert!(matches!(adjust.size_adjust(), CssFontSizeAdjust::None));
}

#[test]
fn public_surface_exposes_checked_font_variant_group_models() {
    assert!(CssFontVariantLigatureValues::try_new(None, None, None, None).is_none());
    let ligatures = CssFontVariantLigatureValues::try_new(
        Some(CssFontVariantLigatureState::Enabled),
        None,
        None,
        Some(CssFontVariantLigatureState::Disabled),
    )
    .expect("nonempty ligature set");
    assert_eq!(
        ligatures.common(),
        Some(CssFontVariantLigatureState::Enabled)
    );

    assert!(CssFontVariantNumericValues::try_new(None, None, None, false, false).is_none());
    let numeric = CssFontVariantNumericValues::try_new(
        Some(CssFontVariantNumericFigure::OldstyleNums),
        None,
        None,
        true,
        false,
    )
    .expect("nonempty numeric set");
    assert!(numeric.ordinal());

    assert!(CssFontVariantEastAsianValues::try_new(None, None, false).is_none());
    assert!(
        CssFontVariantValues::try_new(
            None,
            Some(CssFontVariantPosition::Normal),
            None,
            None,
            None,
        )
        .is_none()
    );
    assert!(
        CssFontVariantValues::try_new(None, None, Some(CssFontVariantCaps::Normal), None, None,)
            .is_none()
    );
    let values = CssFontVariantValues::try_new(
        Some(ligatures),
        Some(CssFontVariantPosition::Super),
        Some(CssFontVariantCaps::SmallCaps),
        Some(numeric),
        None,
    )
    .expect("compatible nonempty shorthand union");
    assert_eq!(values.position(), Some(CssFontVariantPosition::Super));
    assert_eq!(values.caps(), Some(CssFontVariantCaps::SmallCaps));
}

#[test]
fn public_surface_exposes_typed_authored_color_inspection() {
    let report = parse_style_attribute("color: rgb(none 120% calc(1 + 2)); color: WindowText");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Color(rgb) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected RGB color wrapper");
    };
    let rgb = rgb.current().rgb_value().expect("typed RGB branch");
    assert_eq!(rgb.syntax(), CssAuthoredColorSyntax::Modern);
    assert!(matches!(
        rgb.channels(),
        [
            CssAuthoredColorComponent::None,
            CssAuthoredColorComponent::Percentage(_),
            CssAuthoredColorComponent::NumberCalculation(_),
        ]
    ));

    let CssKnownPropertyValueRef::Color(system) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected system color wrapper");
    };
    let category = match system.current().system().unwrap() {
        CssAuthoredSystemColor::WindowText => "deprecated system",
        _ => "other future system color",
    };
    assert_eq!(category, "deprecated system");
}

#[test]
fn public_surface_exposes_perceptual_and_predefined_color_inspection() {
    let report = parse_style_attribute(concat!(
        "color: oklch(50% 0.2 calc(1turn - 90deg) / none); ",
        "color: color(xyz 1 2 3)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Color(oklch) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected Oklch color wrapper");
    };
    assert!(oklch.current().oklch_value().is_some());
    assert!(oklch.current().lab_value().is_none());

    let CssKnownPropertyValueRef::Color(predefined) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected predefined color wrapper");
    };
    assert_eq!(
        predefined
            .current()
            .predefined_value()
            .unwrap()
            .color_space(),
        CssPredefinedColorSpace::XyzD65,
    );
}

#[test]
fn public_surface_exposes_typed_relative_color_inspection_without_resolution() {
    let report =
        parse_style_attribute("color: oklch(from rgb(from red r g b) l c calc(h + 20deg) / alpha)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected color wrapper");
    };
    let relative = value.current().relative_value().unwrap();
    assert_eq!(relative.environment(), CssRelativeColorEnvironment::Oklch);
    assert_eq!(
        relative.channels()[2].result_domain(),
        CssRelativeColorResultDomain::Hue
    );
    let CssRelativeColorExpressionValue::Calculation(hue) = relative.channels()[2].value() else {
        panic!("expected symbolic hue calculation");
    };
    assert_eq!(hue.references(), &[CssRelativeColorChannel::H]);
    assert_eq!(hue.authored().as_css(), "calc(h + 20deg)");
    assert!(relative.source().relative_value().is_some());
    assert!(matches!(
        value.i01_subset(),
        Some(surgeist_css::CssColor::Relative(_))
    ));
}

#[test]
fn public_color_mix_construction_rejects_hue_in_a_rectangular_space() {
    let report = parse_style_attribute("color: color-mix(in oklch, red 25%, blue 75%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected color wrapper");
    };
    let color_mix = value.current().color_mix_value().unwrap();
    let invalid = CssAuthoredColorMix::try_new(
        CssColorInterpolationMethod::new(
            CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
            Some(CssHueInterpolationMethod::Longer),
        ),
        color_mix.left().clone(),
        color_mix.right().clone(),
    );
    assert!(invalid.is_none());

    assert_eq!(
        CssAuthoredColorMixPercentage::try_new(0.0).unwrap().value(),
        0.0
    );
    assert_eq!(
        CssAuthoredColorMixPercentage::try_new(100.0)
            .unwrap()
            .value(),
        100.0,
    );
    for invalid in [-1.0, 101.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssAuthoredColorMixPercentage::try_new(invalid).is_none());
    }
}

fn known_declared_value_kind(value: CssKnownDeclaredValueRef<'_>) -> &'static str {
    match value {
        CssKnownDeclaredValueRef::Property(_) => "property",
        CssKnownDeclaredValueRef::Global(_) => "global",
        CssKnownDeclaredValueRef::SubstitutionDependent(_) => "substitution-dependent",
        _ => "future",
    }
}

fn rule_kind(rule: &CssRule) -> &'static str {
    match rule {
        CssRule::Import(_) => "import",
        CssRule::LayerStatement(_) => "layer statement",
        CssRule::LayerBlock(_) => "layer block",
        CssRule::FontFace(_) => "font face",
        CssRule::Keyframes(_) => "keyframes",
        CssRule::Style(_) => "style",
        CssRule::Media(_) => "media",
        CssRule::Supports(_) => "supports",
        CssRule::Container(_) => "container",
        CssRule::Scope(_) => "scope",
        _ => "future rule",
    }
}

fn error_kind(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::UnknownProperty(_) => "unknown property",
        ErrorKind::InvalidPropertyValue(_) => "invalid property value",
        _ => "other or future error",
    }
}

fn feature_kind(kind: CssFeatureKind) -> &'static str {
    match kind {
        CssFeatureKind::Rule => "rule",
        CssFeatureKind::Declaration => "declaration",
        CssFeatureKind::Descriptor => "descriptor",
        CssFeatureKind::Value => "value",
        CssFeatureKind::Property => "property",
        CssFeatureKind::Selector => "selector",
        CssFeatureKind::MediaQuery => "media query",
        CssFeatureKind::ContainerQuery => "container query",
        _ => "future feature kind",
    }
}

fn importance_kind(importance: CssImportance) -> &'static str {
    match importance {
        CssImportance::Normal => "normal",
        CssImportance::Important => "important",
    }
}

fn support_status_kind(status: CssSupportStatus) -> &'static str {
    match status {
        CssSupportStatus::Complete => "complete",
        CssSupportStatus::Partial => "partial",
        CssSupportStatus::RecognizedUnsupported => "recognized unsupported",
    }
}

fn specification_tier_kind(tier: CssSpecificationTier) -> &'static str {
    match tier {
        CssSpecificationTier::Snapshot2026Official => "official",
        CssSpecificationTier::Snapshot2026Reliable => "reliable",
        CssSpecificationTier::Snapshot2026Stable => "stable",
        CssSpecificationTier::Snapshot2026Interop => "interop",
        CssSpecificationTier::SurgeistExtension => "extension",
        CssSpecificationTier::LaterStandard => "later",
        _ => "future tier",
    }
}

fn exclusion_reason_kind(reason: CssExclusionReason) -> &'static str {
    match reason {
        CssExclusionReason::InformativeOnly => "informative",
        CssExclusionReason::SupersededWithoutCurrentProduction => "superseded",
        CssExclusionReason::OutsideAuthoredSyntaxBoundary => "outside authored syntax",
        _ => "future reason",
    }
}

fn representative_evolving_kind(
    media: CssMediaQueryModifier,
    selector: CssSelectorCombinator,
    calculation: CssCalcOperator,
    timing: CssAnimationDirection,
    grid: CssGridAutoFlowAxis,
    color: CssRelativeColorFunction,
) -> [&'static str; 6] {
    let media = match media {
        CssMediaQueryModifier::Not => "media not",
        CssMediaQueryModifier::Only => "media only",
        _ => "future media modifier",
    };
    let selector = match selector {
        CssSelectorCombinator::Descendant => "selector descendant",
        CssSelectorCombinator::Child => "selector child",
        CssSelectorCombinator::NextSibling => "selector next sibling",
        CssSelectorCombinator::SubsequentSibling => "selector subsequent sibling",
        _ => "future selector combinator",
    };
    let calculation = match calculation {
        CssCalcOperator::Add => "calculation add",
        CssCalcOperator::Subtract => "calculation subtract",
        _ => "future calculation operator",
    };
    let timing = match timing {
        CssAnimationDirection::Normal => "timing normal",
        CssAnimationDirection::Reverse => "timing reverse",
        CssAnimationDirection::Alternate => "timing alternate",
        CssAnimationDirection::AlternateReverse => "timing alternate reverse",
        _ => "future timing direction",
    };
    let grid = match grid {
        CssGridAutoFlowAxis::Row => "grid row",
        CssGridAutoFlowAxis::Column => "grid column",
        _ => "future grid axis",
    };
    let color = match color {
        CssRelativeColorFunction::Rgb => "relative rgb",
        CssRelativeColorFunction::Hsl => "relative hsl",
        CssRelativeColorFunction::Hwb => "relative hwb",
        CssRelativeColorFunction::Lab => "relative lab",
        CssRelativeColorFunction::Lch => "relative lch",
        CssRelativeColorFunction::Oklab => "relative oklab",
        CssRelativeColorFunction::Oklch => "relative oklch",
        CssRelativeColorFunction::Color(_) => "relative color",
        _ => "future relative color function",
    };
    [media, selector, calculation, timing, grid, color]
}

fn emitted_actions(source: &str) -> Vec<CssRecoveryAction> {
    parse_sheet(source)
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.action())
        .collect()
}

#[test]
fn public_surface_sheet_reports_expose_retained_syntax_and_structured_recovery() {
    let clean = parse_sheet(".clean { color: red; }");
    assert!(clean.is_clean());
    assert!(clean.diagnostics().is_empty());
    assert_eq!(clean.syntax().rules().len(), 1);
    assert_eq!(rule_kind(&clean.syntax().rules()[0]), "style");

    let source = ".before { color: red; } @unknown x; .after { color: blue; }";
    let recovered = parse_sheet(source);
    assert!(!recovered.is_clean());
    assert_eq!(recovered.syntax().rules().len(), 2);
    let diagnostic = &recovered.diagnostics()[0];
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("@unknown").expect("unknown at-rule")
    );
    assert_eq!(diagnostic.span().start(), diagnostic.error().position());
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find(" .after").expect("later retained sibling")
    );

    let (sheet, diagnostics) = recovered.into_parts();
    assert_eq!(sheet.rules().len(), 2);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn public_surface_closed_enums_are_exhaustive_and_evolving_enums_use_wildcards() {
    assert_eq!(importance_kind(CssImportance::Normal), "normal");
    assert_eq!(importance_kind(CssImportance::Important), "important");
    assert_eq!(support_status_kind(CssSupportStatus::Complete), "complete");
    assert_eq!(support_status_kind(CssSupportStatus::Partial), "partial");
    assert_eq!(
        support_status_kind(CssSupportStatus::RecognizedUnsupported),
        "recognized unsupported"
    );
    assert_eq!(
        representative_evolving_kind(
            CssMediaQueryModifier::Only,
            CssSelectorCombinator::Child,
            CssCalcOperator::Add,
            CssAnimationDirection::Alternate,
            CssGridAutoFlowAxis::Column,
            CssRelativeColorFunction::Oklch,
        ),
        [
            "media only",
            "selector child",
            "calculation add",
            "timing alternate",
            "grid column",
            "relative oklch",
        ]
    );
}

#[test]
fn public_surface_style_attributes_preserve_importance_custom_and_substitution_syntax() {
    let source = "color: red; --Theme: RGB(1, 2, var(--fallback)); width: var(--size, 2px) !important; mystery: 1";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().as_slice().len(), 3);
    assert_eq!(report.syntax().iter().count(), 3);
    assert_eq!(report.syntax().len(), 3);
    assert!(!report.syntax().is_empty());
    assert_eq!(report.diagnostics().len(), 1);

    let custom = &report.syntax()[1];
    assert!(matches!(
        custom.body(),
        surgeist_css::CssDeclarationBody::Custom(_)
    ));
    assert!(custom.known().is_none());
    let custom = custom.custom().expect("custom declaration");
    assert_eq!(custom.name().as_str(), "--Theme");
    assert_eq!(
        custom
            .value()
            .value()
            .expect("authored custom value")
            .as_css(),
        "RGB(1, 2, var(--fallback))"
    );
    assert!(custom.value().global().is_none());

    let width = &report.syntax()[2];
    assert_eq!(width.importance(), CssImportance::Important);
    assert_eq!(
        width.position().byte_offset().value(),
        source.find("width").expect("width declaration")
    );
    assert!(matches!(
        width.property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Width)
    ));
    let value = width.known().expect("expected coupled width declaration");
    assert!(value.property_value().is_none());
    assert!(value.global().is_none());
    assert_eq!(
        value
            .substitution_dependent()
            .expect("substitution-dependent value")
            .as_css(),
        "var(--size, 2px)"
    );
    assert!(matches!(
        value.declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

#[test]
fn public_surface_known_declarations_expose_coupled_authored_value_views() {
    let report =
        parse_style_attribute("width: calc(100% - 12px); color: inherit; opacity: var(--alpha)");
    assert!(report.is_clean());

    let width = report.syntax()[0].known().expect("known width");
    assert_eq!(width.property(), CssKnownProperty::Width);
    assert_eq!(
        known_declared_value_kind(width.declared_value()),
        "property"
    );
    let Some(CssKnownPropertyValueRef::Width(value)) = width.property_value() else {
        panic!("expected width wrapper");
    };
    assert_eq!(value.as_css(), "calc(100% - 12px)");
    assert!(value.i01_subset().is_some());
    assert!(width.global().is_none());
    assert!(width.substitution_dependent().is_none());

    let color = report.syntax()[1].known().expect("known color");
    assert_eq!(known_declared_value_kind(color.declared_value()), "global");
    assert_eq!(
        color.global(),
        Some(surgeist_css::CssGlobalKeyword::Inherit)
    );
    assert!(color.property_value().is_none());
    assert!(color.substitution_dependent().is_none());

    let opacity = report.syntax()[2].known().expect("known opacity");
    assert_eq!(
        known_declared_value_kind(opacity.declared_value()),
        "substitution-dependent"
    );
    assert_eq!(
        opacity
            .substitution_dependent()
            .expect("substitution-dependent opacity")
            .as_css(),
        "var(--alpha)"
    );
    assert!(opacity.property_value().is_none());
    assert!(opacity.global().is_none());
}

#[test]
fn public_surface_metadata_exposes_every_final_accessor_and_bounded_status() {
    let complete =
        feature_metadata("foundation.declaration.importance").expect("importance catalog record");
    assert_eq!(complete.id().as_str(), "foundation.declaration.importance");
    assert_eq!(feature_kind(complete.kind()), "declaration");
    assert_eq!(
        complete.spelling(),
        "terminal declaration !important annotation"
    );
    assert!(!complete.production().is_empty());
    assert_eq!(complete.status(), CssSupportStatus::Complete);
    assert_eq!(complete.supported_subset(), None);
    assert_eq!(complete.unsupported_remainder(), None);
    assert_eq!(complete.recognized_unsupported_code(), None);
    assert!(complete.baseline_alias_targets().is_empty());
    assert!(complete.source().url().is_some());
    assert_eq!(complete.source().repository_provenance(), None);

    let partial = feature_metadata("baseline.rule.style").expect("style-rule record");
    assert_eq!(partial.status(), CssSupportStatus::Partial);
    assert!(
        partial
            .supported_subset()
            .is_some_and(|text| !text.is_empty())
    );
    assert!(
        partial
            .unsupported_remainder()
            .is_some_and(|text| !text.is_empty())
    );
    assert_eq!(partial.source().id().as_str(), "O-SYNTAX3");
    assert_eq!(partial.source().repository_provenance(), None);

    let alias = feature_metadata("baseline.media.query-list").expect("media baseline alias");
    assert_eq!(alias.baseline_alias_targets().len(), 3);
    assert_eq!(
        alias.baseline_alias_targets()[0].as_str(),
        "official.media.query-list-core"
    );

    let namespace = feature_metadata("later.rule.namespace").expect("namespace record");
    assert_eq!(namespace.status(), CssSupportStatus::Complete);
    assert_eq!(namespace.recognized_unsupported_code(), None);

    let property = property_metadata("WiDtH").expect("ASCII-insensitive property lookup");
    assert_eq!(property.feature().id().as_str(), "baseline.property.width");
    assert_eq!(property.property(), CssKnownProperty::Width);
    assert_eq!(property.canonical_name(), "width");
    assert!(property.aliases().is_empty());
    assert_eq!(property_metadata("--width"), None);
    assert_eq!(property_metadata("not-a-property"), None);
    assert_eq!(feature_metadata("BASELINE.PROPERTY.WIDTH"), None);
}

#[test]
fn public_surface_exposes_dated_sources_and_exclusion_metadata() {
    let color = specification_source("O-COLOR4").expect("dated Color 4 source");
    assert_eq!(color.id().as_str(), "O-COLOR4");
    assert_eq!(color.module(), "CSS Color");
    assert_eq!(color.level(), "4");
    assert_eq!(specification_tier_kind(color.tier()), "official");
    assert_eq!(
        color.url(),
        Some("https://www.w3.org/TR/2026/CRD-css-color-4-20260326/")
    );
    assert_eq!(color.repository_provenance(), None);

    let grid = specification_source("R-GRID1").expect("dated Grid 1 source");
    assert_eq!(grid.module(), "CSS Grid Layout");
    assert_eq!(grid.level(), "1");
    assert_eq!(specification_tier_kind(grid.tier()), "reliable");

    let glyph = conformance_exclusion("excluded.O-WRITING3.property.glyph-orientation-horizontal")
        .expect("removed horizontal glyph-orientation property");
    assert_eq!(
        glyph.id().as_str(),
        "excluded.O-WRITING3.property.glyph-orientation-horizontal"
    );
    assert_eq!(glyph.source().id().as_str(), "O-WRITING3");
    assert_eq!(glyph.production(), "#propdef-glyph-orientation-horizontal");
    assert_eq!(exclusion_reason_kind(glyph.reason()), "superseded");
    assert_eq!(
        glyph.superseding_ids().map(|ids| ids[0].as_str()),
        Some("official.property.text-orientation")
    );
}

#[test]
fn public_surface_non_bmp_coordinates_are_byte_line_and_utf16_based() {
    let source = ".😀 { mystery: 1; color: red; }";
    let report = parse_sheet(source);
    let diagnostic = &report.diagnostics()[0];
    let position = diagnostic.error().position();
    let byte_offset = source.find("mystery").expect("unknown property");

    assert_eq!(position.byte_offset().value(), byte_offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value(),
        u32::try_from(source[..byte_offset].encode_utf16().count()).expect("UTF-16 column")
    );
    assert_ne!(
        position.byte_offset().value(),
        position.column().value() as usize
    );
    assert_eq!(error_kind(diagnostic.error().kind()), "unknown property");
    let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
        panic!("expected typed unknown-property detail");
    };
    assert_eq!(detail.name().as_str(), "mystery");
}

#[test]
fn public_surface_emits_all_ten_recovery_actions() {
    let cases = [
        (
            CssRecoveryAction::DropDeclaration,
            ".x { mystery: 1; color: red; }",
        ),
        (
            CssRecoveryAction::DropDescriptor,
            "@font-face { font-family: Demo; src: url(demo); mystery: 1; }",
        ),
        (
            CssRecoveryAction::DropQualifiedRule,
            "??? { color: red; } .after { color: blue; }",
        ),
        (CssRecoveryAction::DropAtRule, "@unknown value;"),
        (
            CssRecoveryAction::DropKeyframeBlock,
            "@keyframes fade { fn(a) { opacity: .5; } to { opacity: 1; } }",
        ),
        (
            CssRecoveryAction::DropSelectorListItem,
            ":is(.kept,???) { color: red; }",
        ),
        (
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            "@media screen, ??? { .x { color: red; } }",
        ),
        (
            CssRecoveryAction::RetainWithImplicitClosure,
            ".x { color: red;",
        ),
        (
            CssRecoveryAction::IgnoreLegacyToken,
            "<!-- .x { color: red; }",
        ),
    ];

    for (expected, source) in cases {
        assert!(
            emitted_actions(source).contains(&expected),
            "{source} did not emit {expected:?}"
        );
    }

    let mut over_limit = ":is(".repeat(257);
    over_limit.push_str(".leaf");
    over_limit.push_str(&")".repeat(257));
    over_limit.push_str(" { color: red; }");
    assert!(
        emitted_actions(&over_limit).contains(&CssRecoveryAction::StopAtNestingLimit),
        "over-limit selector did not emit StopAtNestingLimit"
    );
}

#[test]
fn public_surface_font_format_models_are_checked_and_compatibility_preserving() {
    use surgeist_css::{
        CssFontFaceUrlSource, CssFontFormatHint, CssFontFormatList, CssFontFormatString,
        CssFontTechHint,
    };

    assert_eq!(CssFontFormatString::try_new(""), None);
    let arbitrary = CssFontFormatString::try_new("zebra").unwrap();
    assert_eq!(arbitrary.as_str(), "zebra");
    let recognized = CssFontFormatString::try_new("WoFf2").unwrap();
    assert_eq!(recognized.as_str(), "WoFf2");
    assert_eq!(CssFontFormatList::try_new(Vec::new()), None);
    let formats = CssFontFormatList::try_new(vec![arbitrary, recognized]).unwrap();
    assert_eq!(formats.formats().len(), 2);

    let legacy = CssFontFaceUrlSource::try_new(
        "face.woff2",
        Some(CssFontFormatHint::Woff2),
        vec![CssFontTechHint::Variations],
    )
    .unwrap();
    assert_eq!(legacy.format(), Some(&CssFontFormatHint::Woff2));
    assert_eq!(legacy.formats().unwrap().formats()[0].as_str(), "woff2");
}

#[test]
fn public_surface_font_face_descriptor_current_models_are_checked() {
    use surgeist_css::{
        CssFontFaceStretch, CssFontFaceStretchKeyword, CssFontFaceWeight, CssFontFaceWeightKeyword,
    };

    assert_eq!(
        CssFontFaceWeight::normal().keyword(),
        Some(CssFontFaceWeightKeyword::Normal)
    );
    assert_eq!(
        CssFontFaceWeight::bold().keyword(),
        Some(CssFontFaceWeightKeyword::Bold)
    );
    assert_eq!(CssFontFaceWeight::try_range(700.0, 400.0), None);
    assert_eq!(
        CssFontFaceStretch::from_keyword(CssFontFaceStretchKeyword::UltraExpanded).keyword(),
        Some(CssFontFaceStretchKeyword::UltraExpanded)
    );
    assert_eq!(CssFontFaceStretch::try_range_percent(125.0, 75.0), None);
}

#[cfg(feature = "app-strict")]
#[test]
fn public_surface_enabled_validators_accept_clean_reports_and_preserve_failures() {
    let sheet = surgeist_css::validate_sheet(".x { color: red; }")
        .expect("clean sheet passes application-strict validation");
    assert_eq!(sheet.rules().len(), 1);

    let ordinary = parse_style_attribute("color: red; mystery: 1");
    let failure = surgeist_css::validate_style_attribute("color: red; mystery: 1")
        .expect_err("recovered style attribute fails application-strict validation");
    assert_eq!(failure.first(), &ordinary.diagnostics()[0]);
    assert_eq!(failure.diagnostics(), ordinary.diagnostics());
    assert_eq!(failure.into_diagnostics(), ordinary.diagnostics());
}
