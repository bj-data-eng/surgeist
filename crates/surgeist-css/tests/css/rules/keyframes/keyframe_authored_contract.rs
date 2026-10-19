#![forbid(unsafe_code)]

//! Authored keyframes follow Animations 1 §3 and retain ordered source structure.
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#keyframes
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#timing-functions
//! Selector percentages use binary64 conversion before inclusive range checking,
//! within Values 4's implementation-defined numeric precision allowance.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types

use surgeist_css::*;

fn only_keyframes(sheet: &CssSheet) -> &CssKeyframesRule {
    let [CssRule::Keyframes(rule)] = sheet.rules() else {
        panic!("one retained keyframes rule");
    };
    rule
}

fn assert_canonical(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(
        validate_sheet(source).unwrap().to_specified_css().unwrap(),
        expected
    );
    let reparsed = parse_sheet(expected);
    assert!(
        reparsed.is_clean(),
        "{expected}: {:?}",
        reparsed.diagnostics()
    );
    assert_eq!(reparsed.syntax().to_specified_css().unwrap(), expected);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("one intact normalized keyframes rule");
    };
    let CssRuleContextKindRef::Keyframes(rule) = context.kind() else {
        panic!("keyframes normalization context");
    };
    assert_eq!(rule, only_keyframes(report.syntax()));
}

fn assert_quoted_rule(authored: &str, decoded: &str, canonical: &str) {
    let source = format!("@keyframes {authored}{{from{{opacity:0}}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let rule = only_keyframes(report.syntax());
    assert!(matches!(rule.name(), CssKeyframesName::String(name) if name.as_str() == decoded));
    assert_eq!(rule.position().byte_offset().value(), 0);
    assert_eq!(rule.blocks().len(), 1);
    assert_canonical(
        &source,
        &format!("@keyframes {canonical} {{ 0% {{ opacity: 0; }} }}"),
    );
}

fn assert_quoted_animation(authored: &str, decoded: &str, canonical: &str, shorthand: bool) {
    let (property, value) = if shorthand {
        ("animation", format!("1s {authored}"))
    } else {
        ("animation-name", authored.to_owned())
    };
    let source = format!("{property}:{value};color:red");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration, color] = report.syntax().as_slice() else {
        panic!("animation declaration and following color");
    };
    assert_eq!(declaration.position().unwrap().byte_offset().value(), 0);
    assert_eq!(color.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(
        color.position().unwrap().byte_offset().value(),
        source.find("color").unwrap()
    );
    let css = match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::AnimationName(wrapper) if !shorthand => {
            assert_eq!(wrapper.as_css(), value);
            assert!(
                matches!(wrapper.names().names(), [CssAnimationName::String(name)] if name.as_str() == decoded)
            );
            wrapper.names().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::Animation(wrapper) if shorthand => {
            assert_eq!(wrapper.as_css(), value);
            assert!(
                matches!(wrapper.animations().values()[0].name(), Some(CssAnimationName::String(name)) if name.as_str() == decoded)
            );
            wrapper.animations().serialize_specified().unwrap()
        }
        _ => panic!("requested animation property"),
    };
    let expected = if shorthand {
        format!("1s {canonical}")
    } else {
        canonical.to_owned()
    };
    assert_eq!(css, expected);
    assert!(validate_style_attribute(&source).is_ok());
    let reentry = parse_style_attribute(&format!("{property}:{css}"));
    assert!(reentry.is_clean());
    match reentry.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    {
        CssKnownPropertyValueRef::AnimationName(wrapper) if !shorthand => {
            assert!(
                matches!(wrapper.names().names(), [CssAnimationName::String(name)] if name.as_str() == decoded)
            );
            assert_eq!(wrapper.names().serialize_specified().unwrap(), expected);
        }
        CssKnownPropertyValueRef::Animation(wrapper) if shorthand => {
            assert!(
                matches!(wrapper.animations().values()[0].name(), Some(CssAnimationName::String(name)) if name.as_str() == decoded)
            );
            assert_eq!(
                wrapper.animations().serialize_specified().unwrap(),
                expected
            );
        }
        _ => panic!("reparsed animation property"),
    }
}

macro_rules! quoted_names {
    ($rule:ident, $longhand:ident, $shorthand:ident, $authored:expr, $decoded:expr, $canonical:expr) => {
        #[test]
        fn $rule() {
            assert_quoted_rule($authored, $decoded, $canonical);
        }
        #[test]
        fn $longhand() {
            assert_quoted_animation($authored, $decoded, $canonical, false);
        }
        #[test]
        fn $shorthand() {
            assert_quoted_animation($authored, $decoded, $canonical, true);
        }
    };
}

quoted_names!(
    empty_quoted_keyframes_name_is_valid,
    empty_quoted_animation_name_is_valid,
    empty_quoted_animation_shorthand_name_is_valid,
    "\"\"",
    "",
    "\"\""
);
quoted_names!(
    space_quoted_keyframes_name_is_valid,
    space_quoted_animation_name_is_valid,
    space_quoted_animation_shorthand_name_is_valid,
    "\" \"",
    " ",
    "\" \""
);
quoted_names!(
    nbsp_quoted_keyframes_name_is_valid,
    nbsp_quoted_animation_name_is_valid,
    nbsp_quoted_animation_shorthand_name_is_valid,
    "\"\u{a0}\"",
    "\u{a0}",
    "\"\u{a0}\""
);
quoted_names!(
    ordinary_quoted_keyframes_name_preserves_case,
    ordinary_quoted_animation_name_preserves_case,
    ordinary_quoted_animation_shorthand_name_preserves_case,
    "\"Fade\"",
    "Fade",
    "\"Fade\""
);
quoted_names!(
    quoted_none_keyframes_name_is_valid,
    quoted_none_animation_name_is_valid,
    quoted_none_animation_shorthand_name_is_valid,
    "\"none\"",
    "none",
    "\"none\""
);

#[test]
fn escaped_space_quoted_keyframes_name_keeps_decoded_identity() {
    assert_quoted_rule(r#""\20""#, " ", "\" \"");
}

#[test]
fn escaped_nbsp_quoted_animation_name_keeps_decoded_identity() {
    assert_quoted_animation(r#""\a0""#, "\u{a0}", "\"\u{a0}\"", false);
}

fn assert_forbidden_keyframe_property(
    property: CssKnownProperty,
    authored_name: &str,
    value: &str,
) {
    let rejected = format!("{authored_name}:{value};");
    let source = format!(
        "@keyframes k{{from{{color:red;{rejected}--phase:A/**/B;opacity:.5;animation-timing-function:linear;color:blue;}}to{{opacity:1}}}}.after{{color:red}}"
    );
    let report = parse_sheet(&source);
    assert!(!report.is_clean(), "{source}");
    let [CssRule::Keyframes(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("keyframes and following style rule survive declaration recovery");
    };
    assert_eq!(rule.blocks().len(), 2);
    let declarations = rule.blocks()[0].declarations();
    assert_eq!(declarations.len(), 5);
    assert_eq!(
        declarations[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let phase = declarations[1].custom().unwrap();
    assert_eq!(phase.name().as_str(), "--phase");
    assert_eq!(phase.value().value().unwrap().as_css(), "A/**/B");
    assert_eq!(
        declarations[2].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(
        declarations[3].known().unwrap().property(),
        CssKnownProperty::AnimationTimingFunction
    );
    assert_eq!(
        declarations[4].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(declarations.iter().all(|declaration| {
        declaration
            .known()
            .is_none_or(|known| known.property() != property)
    }));
    for (declaration, token) in declarations.iter().zip([
        "color:red",
        "--phase",
        "opacity:.5",
        "animation-timing-function",
        "color:blue",
    ]) {
        assert_eq!(
            declaration.position().byte_offset().value(),
            source.find(token).unwrap()
        );
    }
    assert_eq!(
        rule.blocks()[1].declarations()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(
        after.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one forbidden declaration diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let start = source.find(&rejected).unwrap();
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + rejected.len()
    );
    assert!(validate_sheet(&source).is_err());
    let before = report.clone();
    let expected = "@keyframes k { 0% { color: red; --phase: A/**/B; opacity: 0.5; animation-timing-function: linear; color: blue; } 100% { opacity: 1; } }\n.after { color: red; }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    assert!(validate_sheet(expected).is_ok());
    assert_eq!(
        parse_sheet(expected).syntax().to_specified_css().unwrap(),
        expected
    );
    let normalized = normalize_report(&report).unwrap();
    let first = normalized.syntax().items().first().unwrap();
    let CssNormalizedItem::Rule(context) = first else {
        panic!("normalized keyframes first");
    };
    let CssRuleContextKindRef::Keyframes(normalized_rule) = context.kind() else {
        panic!("intact keyframes context");
    };
    assert_eq!(normalized_rule, rule);
}

fn assert_ordinary_animation_property(property: CssKnownProperty, value: &str) {
    let source = format!("{}:{value};color:red", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.syntax()[0].known().unwrap().property(), property);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(validate_style_attribute(&source).is_ok());
}

macro_rules! animation_property {
    ($literal:ident, $global:ident, $substitution:ident, $ordinary:ident, $property:ident, $value:expr) => {
        #[test]
        fn $literal() {
            assert_forbidden_keyframe_property(
                CssKnownProperty::$property,
                CssKnownProperty::$property.canonical_name(),
                $value,
            );
        }
        #[test]
        fn $global() {
            assert_forbidden_keyframe_property(
                CssKnownProperty::$property,
                CssKnownProperty::$property.canonical_name(),
                "inherit",
            );
        }
        #[test]
        fn $substitution() {
            assert_forbidden_keyframe_property(
                CssKnownProperty::$property,
                CssKnownProperty::$property.canonical_name(),
                "var(--animation)",
            );
        }
        #[test]
        fn $ordinary() {
            assert_ordinary_animation_property(CssKnownProperty::$property, $value);
        }
    };
}

animation_property!(
    keyframes_drop_animation_name,
    keyframes_drop_global_animation_name,
    keyframes_drop_symbolic_animation_name,
    ordinary_animation_name_remains_valid,
    AnimationName,
    "k"
);
animation_property!(
    keyframes_drop_animation_duration,
    keyframes_drop_global_animation_duration,
    keyframes_drop_symbolic_animation_duration,
    ordinary_animation_duration_remains_valid,
    AnimationDuration,
    "1s"
);
animation_property!(
    keyframes_drop_animation_delay,
    keyframes_drop_global_animation_delay,
    keyframes_drop_symbolic_animation_delay,
    ordinary_animation_delay_remains_valid,
    AnimationDelay,
    "1s"
);
animation_property!(
    keyframes_drop_animation_iteration_count,
    keyframes_drop_global_animation_iteration_count,
    keyframes_drop_symbolic_animation_iteration_count,
    ordinary_animation_iteration_count_remains_valid,
    AnimationIterationCount,
    "2"
);
animation_property!(
    keyframes_drop_animation_direction,
    keyframes_drop_global_animation_direction,
    keyframes_drop_symbolic_animation_direction,
    ordinary_animation_direction_remains_valid,
    AnimationDirection,
    "reverse"
);
animation_property!(
    keyframes_drop_animation_fill_mode,
    keyframes_drop_global_animation_fill_mode,
    keyframes_drop_symbolic_animation_fill_mode,
    ordinary_animation_fill_mode_remains_valid,
    AnimationFillMode,
    "both"
);
animation_property!(
    keyframes_drop_animation_play_state,
    keyframes_drop_global_animation_play_state,
    keyframes_drop_symbolic_animation_play_state,
    ordinary_animation_play_state_remains_valid,
    AnimationPlayState,
    "paused"
);
animation_property!(
    keyframes_drop_animation_shorthand,
    keyframes_drop_global_animation_shorthand,
    keyframes_drop_symbolic_animation_shorthand,
    ordinary_animation_shorthand_remains_valid,
    Animation,
    "1s k"
);

#[test]
fn keyframes_drop_cased_animation_property_name() {
    assert_forbidden_keyframe_property(CssKnownProperty::AnimationName, "AnImAtIoN-NaMe", "k");
}

#[test]
fn keyframes_drop_escaped_animation_property_name() {
    assert_forbidden_keyframe_property(CssKnownProperty::AnimationName, r"a\6e imation-name", "k");
}

fn assert_retained_easing(selector: &str, canonical_selector: &str) {
    let source = format!(
        "@keyframes k{{{selector}{{animation-timing-function:linear;opacity:1;--phase:end}}}}"
    );
    let expected = format!(
        "@keyframes k {{ {canonical_selector} {{ animation-timing-function: linear; opacity: 1; --phase: end; }} }}"
    );
    assert_canonical(&source, &expected);
    let report = parse_sheet(&source);
    let declarations = only_keyframes(report.syntax()).blocks()[0].declarations();
    assert_eq!(declarations.len(), 3);
    assert_eq!(
        declarations[0].known().unwrap().property(),
        CssKnownProperty::AnimationTimingFunction
    );
    assert!(declarations[2].custom().is_some());
}

#[test]
fn from_easing_remains_in_authored_keyframes() {
    assert_retained_easing("from", "0%");
}
#[test]
fn to_easing_remains_in_authored_keyframes() {
    assert_retained_easing("to", "100%");
}
#[test]
fn mixed_endpoint_easing_remains_in_authored_keyframes() {
    assert_retained_easing("from,to,100%", "0%, 100%, 100%");
}

fn assert_important_keyframe_declaration_is_dropped(
    name: &str,
    value: &str,
    property: Option<CssKnownProperty>,
) {
    let source =
        format!("@keyframes k{{from{{color:red;{name}:{value} !IMPORTANT;opacity:1}}to{{}}}}");
    let report = parse_sheet(&source);
    let rule = only_keyframes(report.syntax());
    assert_eq!(rule.blocks().len(), 2);
    assert_eq!(rule.blocks()[0].declarations().len(), 2);
    assert_eq!(
        rule.blocks()[0].declarations()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        rule.blocks()[0].declarations()[1]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::Opacity
    );
    assert!(rule.blocks()[1].declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one importance diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDeclarationAnnotation
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find('!').unwrap()
    );
    let ErrorKind::InvalidDeclarationAnnotation(detail) = diagnostic.error().kind() else {
        panic!("annotation detail");
    };
    assert_eq!(detail.encountered().kind(), CssTokenKind::Delim);
    assert_eq!(detail.encountered().authored(), "!");
    match (detail.context(), property) {
        (CssDeclarationContextRef::Keyframe(actual), Some(expected)) => {
            assert_eq!(actual, expected)
        }
        (CssDeclarationContextRef::KeyframeCustomProperty(actual), None) => {
            assert_eq!(actual.as_str(), name)
        }
        _ => panic!("keyframe declaration context"),
    }
    assert!(validate_sheet(&source).is_err());
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@keyframes k { 0% { color: red; opacity: 1; } 100% { } }"
    );
}

#[test]
fn important_opacity_is_dropped_from_keyframes() {
    assert_important_keyframe_declaration_is_dropped(
        "opacity",
        "0",
        Some(CssKnownProperty::Opacity),
    );
}
#[test]
fn important_custom_property_is_dropped_from_keyframes() {
    assert_important_keyframe_declaration_is_dropped("--phase", "end", None);
}
#[test]
fn important_easing_is_dropped_from_keyframes() {
    assert_important_keyframe_declaration_is_dropped(
        "animation-timing-function",
        "linear",
        Some(CssKnownProperty::AnimationTimingFunction),
    );
}

fn assert_out_of_range_selector_is_dropped(selector: &str) {
    let source =
        format!("@keyframes k{{{selector}{{opacity:0}}25%{{opacity:1}}}}.after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(!report.is_clean(), "{source}");
    let [CssRule::Keyframes(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("keyframes and forward style sibling");
    };
    assert_eq!(rule.blocks().len(), 1);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[CssKeyframeSelector::Percent(
            CssKeyframePercent::try_new(25.0).unwrap()
        )]
    );
    assert_eq!(
        after.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid selector diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropKeyframeBlock);
    let start = source.find(selector).unwrap();
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + selector.len() + "{opacity:0}".len()
    );
    assert!(validate_sheet(&source).is_err());
    let expected = "@keyframes k { 25% { opacity: 1; } }\n.after { color: red; }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(
        validate_sheet(expected)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        expected
    );
}

#[test]
fn percentage_above_hundred_in_binary64_drops_keyframe_block() {
    assert_out_of_range_selector_is_dropped("100.000001%");
}
#[test]
fn adjacent_binary64_percentage_above_hundred_drops_keyframe_block() {
    assert_out_of_range_selector_is_dropped("100.00000000000001%");
}
#[test]
fn negative_percentage_below_binary32_range_drops_keyframe_block() {
    assert_out_of_range_selector_is_dropped("-1e-46%");
}
#[test]
fn negative_minimum_binary64_subnormal_percentage_drops_keyframe_block() {
    assert_out_of_range_selector_is_dropped("-5e-324%");
}

fn assert_nonendpoint_percent(
    selector: &str,
    endpoint: CssKeyframePercent,
    expected_selector: &str,
) {
    let source = format!("@keyframes k{{{selector}{{opacity:1}}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssKeyframeSelector::Percent(percent)] = only_keyframes(report.syntax()).blocks()[0]
        .selectors()
        .selectors()
    else {
        panic!("one authored percent selector");
    };
    assert_ne!(
        *percent, endpoint,
        "retained scalar must differ from endpoint"
    );
    assert_canonical(
        &source,
        &format!("@keyframes k {{ {expected_selector} {{ opacity: 1; }} }}"),
    );
}

#[test]
fn interior_percentage_below_hundred_retains_distinct_identity() {
    assert_nonendpoint_percent(
        "99.999999%",
        CssKeyframePercent::try_new(100.0).unwrap(),
        "99.999999%",
    );
}
#[test]
fn positive_percentage_below_binary32_range_retains_distinct_identity() {
    assert_nonendpoint_percent("1e-46%", CssKeyframePercent::try_new(0.0).unwrap(), "0%");
}
#[test]
fn positive_minimum_binary64_subnormal_percentage_retains_distinct_identity() {
    assert_nonendpoint_percent("5e-324%", CssKeyframePercent::try_new(0.0).unwrap(), "0%");
}

fn assert_percent_converted_to_endpoint(
    selector: &str,
    endpoint: CssKeyframePercent,
    canonical: &str,
) {
    let source = format!("@keyframes k{{{selector}{{}}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        only_keyframes(report.syntax()).blocks()[0]
            .selectors()
            .selectors(),
        &[CssKeyframeSelector::Percent(endpoint)]
    );
    assert_canonical(&source, &format!("@keyframes k {{ {canonical} {{ }} }}"));
}

#[test]
fn percentage_rounding_to_binary64_hundred_is_admitted() {
    assert_percent_converted_to_endpoint(
        "100.000000000000000000001%",
        CssKeyframePercent::try_new(100.0).unwrap(),
        "100%",
    );
}
#[test]
fn negative_percentage_underflowing_binary64_to_zero_is_admitted() {
    assert_percent_converted_to_endpoint(
        "-1e-999%",
        CssKeyframePercent::try_new(0.0).unwrap(),
        "0%",
    );
}
#[test]
fn ordinary_thirty_percentage_is_admitted() {
    assert_percent_converted_to_endpoint("30%", CssKeyframePercent::try_new(30.0).unwrap(), "30%");
}

#[test]
fn aliases_and_repeated_endpoint_selectors_keep_order() {
    let source = r"@keyframes k{FrOm,0%,f\72 om,to,100%,TO{opacity:1}from{}0%{}}";
    assert_canonical(
        source,
        "@keyframes k { 0%, 0%, 0%, 100%, 100%, 100% { opacity: 1; } 0% { } 0% { } }",
    );
    let report = parse_sheet(source);
    let rule = only_keyframes(report.syntax());
    assert_eq!(rule.blocks().len(), 3);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[
            CssKeyframeSelector::From,
            CssKeyframeSelector::Percent(CssKeyframePercent::try_new(0.0).unwrap()),
            CssKeyframeSelector::From,
            CssKeyframeSelector::To,
            CssKeyframeSelector::Percent(CssKeyframePercent::try_new(100.0).unwrap()),
            CssKeyframeSelector::To
        ]
    );
}

#[test]
fn invalid_later_selector_reports_actual_token_after_trivia() {
    let source = "@keyframes k{from,  /**/ 101%{opacity:0}to{opacity:1}}.after{color:red}";
    let report = parse_sheet(source);
    let [CssRule::Keyframes(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("forward retained rules");
    };
    assert_eq!(rule.blocks().len(), 1);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[CssKeyframeSelector::To]
    );
    assert_eq!(
        after.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected selector list");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropKeyframeBlock);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("101%").unwrap()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("from,").unwrap()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find("to{").unwrap()
    );
    assert!(validate_sheet(source).is_err());
}

fn assert_percentage_math_selector_is_retained(selector: &str) {
    let source =
        format!("@keyframes k{{{selector}{{opacity:0}}to{{opacity:1}}}}.after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Keyframes(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("keyframes and forward style sibling");
    };
    assert_eq!(rule.blocks().len(), 2);
    assert_eq!(rule.blocks()[0].selectors().selectors().len(), 1);
    assert_eq!(
        rule.blocks()[0].declarations()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(
        rule.blocks()[0].position().byte_offset().value(),
        source.find(selector).unwrap()
    );
    assert_eq!(
        rule.blocks()[1].selectors().selectors(),
        &[CssKeyframeSelector::To]
    );
    assert_eq!(
        after.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(validate_sheet(&source).is_ok());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let CssNormalizedItem::Rule(context) = &normalized.items()[0] else {
        panic!("normalized keyframes");
    };
    let CssRuleContextKindRef::Keyframes(normalized_rule) = context.kind() else {
        panic!("intact math selector payload");
    };
    assert_eq!(normalized_rule, rule);
}

#[test]
fn percentage_calc_selector_is_valid() {
    assert_percentage_math_selector_is_retained("calc(25%)");
}
#[test]
fn percentage_min_selector_is_valid() {
    assert_percentage_math_selector_is_retained("min(25%,50%)");
}
#[test]
fn percentage_max_selector_is_valid() {
    assert_percentage_math_selector_is_retained("max(25%,50%)");
}
#[test]
fn percentage_clamp_selector_is_valid() {
    assert_percentage_math_selector_is_retained("clamp(0%,25%,100%)");
}
#[test]
fn percentage_calc_above_hundred_is_authored_valid() {
    assert_percentage_math_selector_is_retained("calc(101%)");
}
#[test]
fn negative_percentage_calc_is_authored_valid() {
    assert_percentage_math_selector_is_retained("calc(-1%)");
}

#[test]
fn context_dependent_percentage_math_selector_remains_authored_valid() {
    assert_percentage_math_selector_is_retained("calc(25% * sign(1em - 1px))");
}

fn assert_wrong_numeric_type_selector_is_dropped(selector: &str) {
    let source =
        format!("@keyframes k{{{selector}{{opacity:0}}to{{opacity:1}}}}.after{{color:red}}");
    let report = parse_sheet(&source);
    let [CssRule::Keyframes(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("retained forward rules");
    };
    assert_eq!(rule.blocks().len(), 1);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[CssKeyframeSelector::To]
    );
    assert_eq!(
        after.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one wrong selector type diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropKeyframeBlock);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find(selector).unwrap()
    );
    assert!(validate_sheet(&source).is_err());
}

#[test]
fn number_result_calc_does_not_become_percentage_selector() {
    assert_wrong_numeric_type_selector_is_dropped("calc(25)");
}
#[test]
fn length_result_calc_does_not_become_percentage_selector() {
    assert_wrong_numeric_type_selector_is_dropped("calc(25px)");
}
#[test]
fn mixed_length_percentage_calc_does_not_become_percentage_selector() {
    assert_wrong_numeric_type_selector_is_dropped("calc(25% + 1px)");
}
