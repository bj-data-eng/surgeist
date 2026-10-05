#![forbid(unsafe_code)]

//! Values 4 §4.2: custom identifiers preserve decoded case and exclude CSS-wide
//! keywords and default. Consumer exclusions are defined by Grid 1 §7.2.2,
//! Animations 1 §3, and Transitions 1 §§2.1 and 2.5.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! https://www.w3.org/TR/2025/CRD-css-grid-1-20250326/#named-lines
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#keyframes
//! https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#transition-property-property
//! https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#transition-shorthand-property

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}");
    };
    assert!(report.clone().into_validation_result().is_ok());
    value.clone()
}

fn transition_properties(value: &str) -> CssTransitionPropertyList {
    let declaration = declaration(&format!("transition-property:{value}"));
    let CssKnownPropertyValueRef::TransitionProperty(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("transition-property");
    };
    value.properties().clone()
}

fn transitions(value: &str) -> CssTransitionList {
    let declaration = declaration(&format!("transition:{value}"));
    let CssKnownPropertyValueRef::Transition(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("transition");
    };
    value.transitions().clone()
}

fn animation_names(value: &str) -> CssAnimationNameList {
    let declaration = declaration(&format!("animation-name:{value}"));
    let CssKnownPropertyValueRef::AnimationName(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("animation-name");
    };
    value.names().clone()
}

fn animations(value: &str) -> CssAnimationList {
    let declaration = declaration(&format!("animation:{value}"));
    let CssKnownPropertyValueRef::Animation(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("animation");
    };
    value.animations().clone()
}

fn assert_transition_property_name(authored: &str, decoded: &str) {
    let value = transition_properties(authored);
    assert!(matches!(
        value.properties(),
        [CssTransitionProperty::Custom(name)] if name.as_str() == decoded
    ));
    let css = value.serialize_specified().unwrap();
    assert_eq!(css, decoded);
    assert_eq!(transition_properties(&css), value);
}

fn assert_transition_name(authored: &str, decoded: &str) {
    let value = transitions(&format!("{authored} 1s"));
    let [item] = value.values() else {
        panic!("one transition");
    };
    assert!(matches!(
        item.property(),
        Some(CssTransitionProperty::Custom(name)) if name.as_str() == decoded
    ));
    let duration = item.duration().unwrap().time().literal().unwrap();
    assert_eq!(duration.numeric().representation(), "1");
    assert_eq!(duration.unit(), CssTimeUnit::Seconds);
    assert!(item.delay().is_none());
    assert!(item.timing_function().is_none());
    let css = value.serialize_specified().unwrap();
    assert_eq!(css, format!("{decoded} 1s"));
    assert_eq!(transitions(&css), value);
}

fn assert_animation_name(authored: &str, decoded: &str) {
    let value = animation_names(authored);
    assert!(matches!(
        value.names(),
        [CssAnimationName::Custom(name)] if name.as_str() == decoded
    ));
    let css = value.serialize_specified().unwrap();
    assert_eq!(css, decoded);
    assert_eq!(animation_names(&css), value);
}

fn assert_animation_shorthand_name(authored: &str, decoded: &str) {
    let value = animations(&format!("{authored} 1s"));
    let [item] = value.values() else {
        panic!("one animation");
    };
    assert!(matches!(
        item.name(),
        Some(CssAnimationName::Custom(name)) if name.as_str() == decoded
    ));
    let duration = item.duration().unwrap().time().literal().unwrap();
    assert_eq!(duration.numeric().representation(), "1");
    assert_eq!(duration.unit(), CssTimeUnit::Seconds);
    assert!(item.delay().is_none());
    assert!(item.timing_function().is_none());
    let css = value.serialize_specified().unwrap();
    assert_eq!(css, format!("1s {decoded}"));
    assert_eq!(animations(&css), value);
}

fn assert_keyframes_ident(authored: &str, decoded: &str) {
    let source = format!("@keyframes {authored} {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("one retained keyframes rule");
    };
    assert!(matches!(rule.name(), CssKeyframesName::Ident(name) if name.as_str() == decoded));
    let css = report.syntax().to_specified_css().unwrap();
    assert_eq!(css, format!("@keyframes {decoded} {{ }}"));
    let reparsed = parse_sheet(&css);
    assert!(reparsed.is_clean());
    let [CssRule::Keyframes(reparsed_rule)] = reparsed.syntax().rules() else {
        panic!("one reparsed keyframes rule");
    };
    assert_eq!(reparsed_rule.name(), rule.name());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("one normalized terminal rule");
    };
    let CssRuleContextKindRef::Keyframes(normalized_rule) = context.kind() else {
        panic!("intact normalized keyframes payload");
    };
    assert_eq!(normalized_rule.name(), rule.name());
    assert_eq!(normalized_rule.position(), rule.position());
    assert!(report.into_validation_result().is_ok());
}

fn assert_rejected_property(property: CssKnownProperty, value: &str, responsible: Option<&str>) {
    let source = format!("{}:{value};color:red", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(!report.is_clean(), "{source}");
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one grammar diagnostic: {source}: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed property grammar error");
    };
    assert_eq!(detail.property(), property);
    if let Some(responsible) = responsible {
        let offset = source.find(responsible).unwrap();
        assert_eq!(diagnostic.error().position().byte_offset().value(), offset);
        assert_eq!(diagnostic.error().position().line().value(), 0);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            offset
        );
        if let Some(token) = detail.encountered() {
            assert_eq!(token.kind(), CssTokenKind::Ident);
            assert_eq!(token.authored(), responsible);
        }
    }
    let [sibling] = report.syntax().as_slice() else {
        panic!("only the following color declaration remains: {source}");
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    assert!(report.into_validation_result().is_err());
}

fn assert_rejected_keyframes_name(authored: &str) {
    let source = format!("@keyframes {authored} {{}} .after {{color:red}}");
    let report = parse_sheet(&source);
    assert!(!report.is_clean(), "{source}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid keyframes prelude: {:?}", report.diagnostics());
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
        panic!("typed at-rule prelude error");
    };
    assert_eq!(detail.name().as_str(), "keyframes");
    let position = diagnostic.error().position().byte_offset().value();
    assert!(diagnostic.span().start().byte_offset().value() <= position);
    assert!(position <= diagnostic.span().end().byte_offset().value());
    let [CssRule::Style(sibling)] = report.syntax().rules() else {
        panic!("following style rule survives");
    };
    let [declaration] = sibling.declarations().as_slice() else {
        panic!("following color declaration survives");
    };
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(report.into_validation_result().is_err());
}

fn grid_tracks(value: &str) -> CssGridTrackList {
    let declaration = declaration(&format!("grid-template-columns:{value}"));
    let CssKnownPropertyValueRef::GridTemplateColumns(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("grid-template-columns");
    };
    value.value().clone()
}

fn one_second_transition(property: CssTransitionProperty) -> CssTransition {
    let time =
        CssTimeValue::from_literal(CssTimeLiteral::try_new("1", CssTimeUnit::Seconds).unwrap());
    CssTransition::try_new(
        Some(property),
        Some(CssDuration::try_new(time).unwrap()),
        None,
        None,
    )
    .unwrap()
}

#[test]
fn custom_identifier_admits_auto() {
    let value = CssCustomIdent::try_new("auto").expect("valid custom identifier");
    assert_eq!(value.as_str(), "auto");
}

#[test]
fn custom_identifier_admits_span() {
    let value = CssCustomIdent::try_new("span").expect("valid custom identifier");
    assert_eq!(value.as_str(), "span");
}

#[test]
fn custom_identifier_admits_uppercase_auto() {
    let value = CssCustomIdent::try_new("AUTO").expect("valid custom identifier");
    assert_eq!(value.as_str(), "AUTO");
}

#[test]
fn custom_identifier_admits_mixed_case_span() {
    let value = CssCustomIdent::try_new("SpAn").expect("valid custom identifier");
    assert_eq!(value.as_str(), "SpAn");
}

#[test]
fn custom_identifier_rejects_default() {
    assert!(CssCustomIdent::try_new("default").is_none());
}

#[test]
fn custom_identifier_rejects_mixed_case_default() {
    assert!(CssCustomIdent::try_new("DeFaUlT").is_none());
}

#[test]
fn custom_identifier_rejects_nul() {
    assert!(CssCustomIdent::try_new("a\0b").is_none());
}

#[test]
fn custom_identifier_rejects_empty() {
    assert!(CssCustomIdent::try_new("").is_none());
}

#[test]
fn custom_identifier_rejects_inherit() {
    assert!(CssCustomIdent::try_new("inherit").is_none());
}

#[test]
fn custom_identifier_rejects_initial() {
    assert!(CssCustomIdent::try_new("initial").is_none());
}

#[test]
fn custom_identifier_rejects_unset() {
    assert!(CssCustomIdent::try_new("unset").is_none());
}

#[test]
fn custom_identifier_rejects_revert() {
    assert!(CssCustomIdent::try_new("revert").is_none());
}

#[test]
fn custom_identifier_rejects_revert_layer() {
    assert!(CssCustomIdent::try_new("revert-layer").is_none());
}

#[test]
fn custom_identifier_preserves_case_sensitive_identity() {
    let lower = CssCustomIdent::try_new("example").unwrap();
    let upper = CssCustomIdent::try_new("EXAMPLE").unwrap();
    assert_ne!(lower, upper);
    assert_eq!(upper.as_str(), "EXAMPLE");
}

#[test]
fn custom_identifier_preserves_decoded_punctuation() {
    let value = CssCustomIdent::try_new("a b").unwrap();
    assert_eq!(value.as_str(), "a b");
}

#[test]
fn custom_identifier_admits_none_outside_consumer_grammar() {
    assert_eq!(CssCustomIdent::try_new("none").unwrap().as_str(), "none");
}

#[test]
fn custom_identifier_admits_all_outside_consumer_grammar() {
    assert_eq!(CssCustomIdent::try_new("all").unwrap().as_str(), "all");
}

#[test]
fn checked_custom_identifier_treats_backslashes_as_decoded_content() {
    let decoded = r"\61uto";
    assert_eq!(CssCustomIdent::try_new(decoded).unwrap().as_str(), decoded);
}

#[test]
fn transition_property_admits_auto() {
    assert_transition_property_name("auto", "auto");
}

#[test]
fn transition_property_admits_span() {
    assert_transition_property_name("span", "span");
}

#[test]
fn transition_property_admits_escaped_uppercase_auto() {
    assert_transition_property_name("\\41uto", "Auto");
}

#[test]
fn transition_property_admits_escaped_mixed_case_span() {
    assert_transition_property_name("\\53pAn", "SpAn");
}

#[test]
fn transition_shorthand_admits_auto() {
    assert_transition_name("auto", "auto");
}

#[test]
fn transition_shorthand_admits_span() {
    assert_transition_name("span", "span");
}

#[test]
fn transition_shorthand_admits_escaped_uppercase_auto() {
    assert_transition_name("\\41uto", "Auto");
}

#[test]
fn transition_shorthand_admits_escaped_mixed_case_span() {
    assert_transition_name("\\53pAn", "SpAn");
}

#[test]
fn animation_name_admits_auto() {
    assert_animation_name("auto", "auto");
}

#[test]
fn animation_name_admits_span() {
    assert_animation_name("span", "span");
}

#[test]
fn animation_name_admits_escaped_uppercase_auto() {
    assert_animation_name("\\41uto", "Auto");
}

#[test]
fn animation_name_admits_escaped_mixed_case_span() {
    assert_animation_name("\\53pAn", "SpAn");
}

#[test]
fn animation_shorthand_admits_auto() {
    assert_animation_shorthand_name("auto", "auto");
}

#[test]
fn animation_shorthand_admits_span() {
    assert_animation_shorthand_name("span", "span");
}

#[test]
fn animation_shorthand_admits_escaped_uppercase_auto() {
    assert_animation_shorthand_name("\\41uto", "Auto");
}

#[test]
fn animation_shorthand_admits_escaped_mixed_case_span() {
    assert_animation_shorthand_name("\\53pAn", "SpAn");
}

#[test]
fn keyframes_admits_auto() {
    assert_keyframes_ident("auto", "auto");
}

#[test]
fn keyframes_admits_span() {
    assert_keyframes_ident("span", "span");
}

#[test]
fn keyframes_admits_escaped_uppercase_auto() {
    assert_keyframes_ident("\\41uto", "Auto");
}

#[test]
fn keyframes_admits_escaped_mixed_case_span() {
    assert_keyframes_ident("\\53pAn", "SpAn");
}

#[test]
fn transition_property_rejects_default() {
    assert_rejected_property(
        CssKnownProperty::TransitionProperty,
        "default",
        Some("default"),
    );
}

#[test]
fn transition_property_rejects_mixed_case_default() {
    assert_rejected_property(
        CssKnownProperty::TransitionProperty,
        "DeFaUlT",
        Some("DeFaUlT"),
    );
}

#[test]
fn transition_property_rejects_escaped_default() {
    assert_rejected_property(
        CssKnownProperty::TransitionProperty,
        "\\64 efault",
        Some("\\64 efault"),
    );
}

#[test]
fn transition_shorthand_rejects_default() {
    assert_rejected_property(CssKnownProperty::Transition, "default 1s", Some("default"));
}

#[test]
fn transition_shorthand_rejects_mixed_case_default() {
    assert_rejected_property(CssKnownProperty::Transition, "DeFaUlT 1s", Some("DeFaUlT"));
}

#[test]
fn transition_shorthand_rejects_escaped_default() {
    assert_rejected_property(
        CssKnownProperty::Transition,
        "\\64 efault 1s",
        Some("\\64 efault"),
    );
}

#[test]
fn animation_name_rejects_default() {
    assert_rejected_property(CssKnownProperty::AnimationName, "default", Some("default"));
}

#[test]
fn animation_name_rejects_mixed_case_default() {
    assert_rejected_property(CssKnownProperty::AnimationName, "DeFaUlT", Some("DeFaUlT"));
}

#[test]
fn animation_name_rejects_escaped_default() {
    assert_rejected_property(
        CssKnownProperty::AnimationName,
        "\\64 efault",
        Some("\\64 efault"),
    );
}

#[test]
fn animation_shorthand_rejects_default() {
    assert_rejected_property(CssKnownProperty::Animation, "default 1s", Some("default"));
}

#[test]
fn animation_shorthand_rejects_mixed_case_default() {
    assert_rejected_property(CssKnownProperty::Animation, "DeFaUlT 1s", Some("DeFaUlT"));
}

#[test]
fn animation_shorthand_rejects_escaped_default() {
    assert_rejected_property(
        CssKnownProperty::Animation,
        "\\64 efault 1s",
        Some("\\64 efault"),
    );
}

#[test]
fn keyframes_rejects_default() {
    assert_rejected_keyframes_name("default");
}

#[test]
fn keyframes_rejects_mixed_case_default() {
    assert_rejected_keyframes_name("DeFaUlT");
}

#[test]
fn keyframes_rejects_escaped_default() {
    assert_rejected_keyframes_name("\\64 efault");
}

#[test]
fn grid_track_names_reject_default() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[default] 1fr",
        Some("default"),
    );
}

#[test]
fn grid_track_names_reject_mixed_case_default() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[DeFaUlT] 1fr",
        Some("DeFaUlT"),
    );
}

#[test]
fn grid_track_names_reject_escaped_default() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[\\64 efault] 1fr",
        Some("\\64 efault"),
    );
}

#[test]
fn grid_track_names_reject_auto() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[auto] 1fr",
        Some("auto"),
    );
}

#[test]
fn grid_track_names_reject_span() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[span] 1fr",
        Some("span"),
    );
}

#[test]
fn grid_track_names_reject_escaped_auto() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[\\61uto] 1fr",
        Some("\\61uto"),
    );
}

#[test]
fn grid_track_names_reject_escaped_span() {
    assert_rejected_property(
        CssKnownProperty::GridTemplateColumns,
        "[\\73pan] 1fr",
        Some("\\73pan"),
    );
}

#[test]
fn grid_track_names_preserve_empty_group() {
    let value = grid_tracks("[] 1fr");
    let [CssGridGeneralTrackComponent::LineNames(names), _] =
        value.general_list().unwrap().components()
    else {
        panic!("empty authored line-name group");
    };
    assert!(names.names().is_empty());
    assert_eq!(value.serialize_specified().unwrap(), "[] 1fr");
}

#[test]
fn grid_track_names_preserve_duplicates_and_case() {
    let value = grid_tracks("[Start start Start] 1fr");
    let [CssGridGeneralTrackComponent::LineNames(names), _] =
        value.general_list().unwrap().components()
    else {
        panic!("ordered authored line-name group");
    };
    assert_eq!(names.names().len(), 3);
    assert_ne!(names.names()[0], names.names()[1]);
    assert_eq!(names.names()[0], names.names()[2]);
    let css = value.serialize_specified().unwrap();
    assert_eq!(css, "[Start start Start] 1fr");
    assert_eq!(grid_tracks(&css), value);
}

#[test]
fn checked_grid_group_preserves_empty_group() {
    assert!(CssGridLineNames::new(Vec::new()).names().is_empty());
}

#[test]
fn checked_grid_line_name_rejects_auto() {
    assert!(CssGridLineName::try_new(CssIdent::try_new("auto").unwrap()).is_none());
}

#[test]
fn checked_grid_line_name_rejects_span() {
    assert!(CssGridLineName::try_new(CssIdent::try_new("span").unwrap()).is_none());
}

#[test]
fn checked_grid_line_name_rejects_default() {
    assert!(CssGridLineName::try_new(CssIdent::try_new("default").unwrap()).is_none());
}

#[test]
fn transition_property_rejects_leading_none_in_list() {
    assert_rejected_property(CssKnownProperty::TransitionProperty, "none, opacity", None);
}

#[test]
fn transition_property_rejects_trailing_none_in_list() {
    assert_rejected_property(CssKnownProperty::TransitionProperty, "opacity, none", None);
}

#[test]
fn transition_property_rejects_escaped_trailing_none_in_list() {
    assert_rejected_property(
        CssKnownProperty::TransitionProperty,
        "opacity, \\6e one",
        None,
    );
}

#[test]
fn transition_shorthand_rejects_leading_none_in_list() {
    assert_rejected_property(CssKnownProperty::Transition, "none 1s, opacity 2s", None);
}

#[test]
fn transition_shorthand_rejects_trailing_none_in_list() {
    assert_rejected_property(CssKnownProperty::Transition, "opacity 1s, none 2s", None);
}

#[test]
fn transition_shorthand_rejects_escaped_trailing_none_in_list() {
    assert_rejected_property(
        CssKnownProperty::Transition,
        "opacity 1s, \\6e one 2s",
        None,
    );
}

#[test]
fn checked_transition_property_list_rejects_leading_none() {
    assert!(
        CssTransitionPropertyList::try_new(vec![
            CssTransitionProperty::None,
            CssTransitionProperty::All
        ])
        .is_none()
    );
}

#[test]
fn checked_transition_shorthand_list_rejects_leading_none() {
    assert!(
        CssTransitionList::try_new(vec![
            one_second_transition(CssTransitionProperty::None),
            one_second_transition(CssTransitionProperty::All)
        ])
        .is_none()
    );
}

#[test]
fn checked_transition_property_list_rejects_trailing_none() {
    assert!(
        CssTransitionPropertyList::try_new(vec![
            CssTransitionProperty::All,
            CssTransitionProperty::None
        ])
        .is_none()
    );
}

#[test]
fn checked_transition_shorthand_list_rejects_trailing_none() {
    assert!(
        CssTransitionList::try_new(vec![
            one_second_transition(CssTransitionProperty::All),
            one_second_transition(CssTransitionProperty::None)
        ])
        .is_none()
    );
}

#[test]
fn transition_property_accepts_singleton_none() {
    let value = transition_properties("NoNe");
    assert_eq!(value.properties(), &[CssTransitionProperty::None]);
    assert_eq!(value.serialize_specified().unwrap(), "none");
}

#[test]
fn transition_shorthand_accepts_singleton_none() {
    let value = transitions("NoNe 1s");
    assert!(matches!(
        value.values()[0].property(),
        Some(CssTransitionProperty::None)
    ));
    assert_eq!(value.serialize_specified().unwrap(), "none 1s");
}

#[test]
fn checked_transition_property_list_accepts_singleton_none() {
    let value = CssTransitionPropertyList::try_new(vec![CssTransitionProperty::None]).unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "none");
}

#[test]
fn checked_transition_shorthand_list_accepts_singleton_none() {
    let value =
        CssTransitionList::try_new(vec![one_second_transition(CssTransitionProperty::None)])
            .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "none 1s");
}

#[test]
fn transition_property_preserves_unknown_case_and_duplicate_indices() {
    let value = transition_properties("Unknown, unknown, Unknown");
    let [
        CssTransitionProperty::Custom(first),
        CssTransitionProperty::Custom(second),
        CssTransitionProperty::Custom(third),
    ] = value.properties()
    else {
        panic!("three unknown identifiers");
    };
    assert_eq!(
        (first.as_str(), second.as_str(), third.as_str()),
        ("Unknown", "unknown", "Unknown")
    );
    assert_ne!(first, second);
    assert_eq!(first, third);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "Unknown, unknown, Unknown"
    );
}

#[test]
fn animation_name_preserves_case_and_duplicate_indices() {
    let value = animation_names("Name, name, Name");
    let [
        CssAnimationName::Custom(first),
        CssAnimationName::Custom(second),
        CssAnimationName::Custom(third),
    ] = value.names()
    else {
        panic!("three animation identifiers");
    };
    assert_eq!(
        (first.as_str(), second.as_str(), third.as_str()),
        ("Name", "name", "Name")
    );
    assert_ne!(first, second);
    assert_eq!(first, third);
}

#[test]
fn animation_name_accepts_easing_keyword_as_name() {
    assert_animation_name("ease", "ease");
}

#[test]
fn animation_shorthand_disambiguates_easing_keyword_name() {
    let value = animations("ease ease");
    assert!(
        matches!(value.values()[0].name(), Some(CssAnimationName::Custom(name)) if name.as_str() == "ease")
    );
    assert_eq!(value.serialize_specified().unwrap(), "ease ease");
    assert_eq!(animations("ease ease"), value);
}

#[test]
fn transition_property_accepts_easing_keyword_as_name() {
    assert_transition_property_name("ease", "ease");
}

#[test]
fn transition_shorthand_disambiguates_easing_keyword_name() {
    let value = transitions("ease ease");
    assert!(
        matches!(value.values()[0].property(), Some(CssTransitionProperty::Custom(name)) if name.as_str() == "ease")
    );
    assert_eq!(value.serialize_specified().unwrap(), "ease ease");
    assert_eq!(transitions("ease ease"), value);
}

#[test]
fn keyframes_preserve_ordinary_identifier_through_normalization() {
    assert_keyframes_ident("Fade", "Fade");
}

#[test]
fn keyframes_reject_none_identifier() {
    assert_rejected_keyframes_name("none");
}

#[test]
fn keyframes_accept_quoted_none_name() {
    let report = parse_sheet("@keyframes \"none\" {}");
    assert!(report.is_clean());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("keyframes");
    };
    assert!(matches!(rule.name(), CssKeyframesName::String(name) if name.as_str() == "none"));
}

#[test]
fn transition_property_accepts_standalone_css_wide_value() {
    let declaration = declaration("transition-property:InHeRiT");
    assert!(matches!(
        declaration.known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
}

#[test]
fn transition_shorthand_accepts_standalone_css_wide_value() {
    let declaration = declaration("transition:InHeRiT");
    assert!(matches!(
        declaration.known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
}

#[test]
fn animation_name_accepts_standalone_css_wide_value() {
    let declaration = declaration("animation-name:InHeRiT");
    assert!(matches!(
        declaration.known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
}

#[test]
fn animation_shorthand_accepts_standalone_css_wide_value() {
    let declaration = declaration("animation:InHeRiT");
    assert!(matches!(
        declaration.known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
}
