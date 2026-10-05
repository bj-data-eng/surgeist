#![forbid(unsafe_code)]

//! Checked name owners separate Values 4 identifier validity from the extra
//! exclusions in Animations 1, Transitions 1, and Grid 1.

use surgeist_css::*;

fn custom(value: &str) -> CssCustomIdent {
    CssCustomIdent::try_new(value).unwrap()
}

fn keyframes_ident(value: &str) -> CssKeyframesIdent {
    CssKeyframesIdent::try_new(custom(value)).unwrap()
}

fn transition_name(value: &str) -> CssTransitionPropertyName {
    CssTransitionPropertyName::try_new(custom(value)).unwrap()
}

#[test]
fn checked_keyframes_names_admit_keywords_of_other_grammars() {
    for text in [
        "auto", "span", "all", "ease", "infinite", "normal", "running",
    ] {
        assert_eq!(keyframes_ident(text).as_str(), text);
    }
}

#[test]
fn checked_keyframes_names_exclude_every_ascii_case_of_none() {
    for text in ["none", "NONE", "NoNe", "nOnE"] {
        assert!(CssKeyframesIdent::try_new(custom(text)).is_none());
    }
}

#[test]
fn checked_keyframes_names_keep_decoded_case_and_punctuation() {
    let lower = keyframes_ident("a b");
    let upper = keyframes_ident("A B");
    assert_ne!(lower, upper);
    assert_eq!(upper.as_str(), "A B");
    let names = CssAnimationNameList::try_new(vec![
        CssAnimationName::Custom(upper.clone()),
        CssAnimationName::Custom(lower.clone()),
        CssAnimationName::Custom(upper),
    ])
    .unwrap();
    let css = names.serialize_specified().unwrap();
    assert_eq!(css, r"A\ B, a\ b, A\ B");
    let report = parse_style_attribute(&format!("animation-name:{css}"));
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::AnimationName(parsed) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("animation-name");
    };
    assert_eq!(parsed.names(), &names);
}

#[test]
fn checked_identifier_has_the_same_identity_in_definitions_and_references() {
    let ident = keyframes_ident("Fade");
    let definition = CssKeyframesName::Ident(ident.clone());
    let reference = CssAnimationName::Custom(ident);
    let sheet = parse_sheet("@keyframes Fade {}");
    assert!(sheet.is_clean());
    let [CssRule::Keyframes(rule)] = sheet.syntax().rules() else {
        panic!("keyframes definition");
    };
    assert_eq!(rule.name(), &definition);
    let names = CssAnimationNameList::try_new(vec![reference.clone()]).unwrap();
    assert_eq!(names.serialize_specified().unwrap(), "Fade");
    let report = parse_style_attribute("animation-name:Fade");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::AnimationName(parsed) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("animation reference");
    };
    assert_eq!(parsed.names().names(), &[reference]);
}

#[test]
fn checked_transition_names_exclude_none_and_all_keyword_variants() {
    for text in ["none", "NONE", "NoNe", "all", "ALL", "AlL"] {
        assert!(CssTransitionPropertyName::try_new(custom(text)).is_none());
    }
}

#[test]
fn checked_transition_names_keep_unknown_names_case_and_duplicate_indices() {
    let upper = transition_name("Auto");
    let lower = transition_name("auto");
    assert_ne!(upper, lower);
    assert_eq!(upper.as_str(), "Auto");
    let properties = CssTransitionPropertyList::try_new(vec![
        CssTransitionProperty::Custom(upper.clone()),
        CssTransitionProperty::Custom(lower),
        CssTransitionProperty::Custom(transition_name("span")),
        CssTransitionProperty::Custom(upper),
    ])
    .unwrap();
    assert_eq!(
        properties.serialize_specified().unwrap(),
        "Auto, auto, span, Auto"
    );
    assert_eq!(properties.properties().len(), 4);
}

#[test]
fn checked_consumer_names_keep_literal_backslashes_as_decoded_content() {
    let decoded = r"\61uto";
    let animation =
        CssAnimationNameList::try_new(vec![CssAnimationName::Custom(keyframes_ident(decoded))])
            .unwrap();
    let transition = CssTransitionPropertyList::try_new(vec![CssTransitionProperty::Custom(
        transition_name(decoded),
    )])
    .unwrap();
    for css in [
        animation.serialize_specified().unwrap(),
        transition.serialize_specified().unwrap(),
    ] {
        assert_eq!(css, r"\\61uto");
        let components = parse_component_values(&css).unwrap();
        assert!(matches!(
            components.items(),
            [component] if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == decoded)
        ));
    }
}

#[test]
fn checked_grid_groups_accept_only_checked_line_names_and_preserve_duplicates() {
    let name = CssGridLineName::try_new(CssIdent::try_new("none").unwrap()).unwrap();
    let names = CssGridLineNames::new(vec![name.clone(), name.clone()]);
    assert_eq!(names.names(), &[name.clone(), name]);
    let tracks = CssGridTrackList::general(
        CssGridGeneralTrackList::try_new(vec![
            CssGridGeneralTrackComponent::LineNames(names),
            CssGridGeneralTrackComponent::TrackSize(CssGridTrackSize::from_breadth(
                CssGridTrackBreadth::auto(),
            )),
        ])
        .unwrap(),
    );
    assert_eq!(tracks.serialize_specified().unwrap(), "[none none] auto");
}

#[test]
fn checked_transition_none_cannot_mix_with_an_omitted_property() {
    let none = CssTransition::try_new(Some(CssTransitionProperty::None), None, None, None).unwrap();
    let duration = CssDuration::try_new(CssTimeValue::from_literal(
        CssTimeLiteral::try_new("1", CssTimeUnit::Seconds).unwrap(),
    ))
    .unwrap();
    let omitted = CssTransition::try_new(None, Some(duration), None, None).unwrap();
    assert!(CssTransitionList::try_new(vec![none.clone(), omitted.clone()]).is_none());
    assert!(CssTransitionList::try_new(vec![omitted, none]).is_none());
}

#[test]
fn checked_name_projection_keeps_exact_budgets_and_atomic_failures() {
    let names =
        CssAnimationNameList::try_new(vec![CssAnimationName::Custom(keyframes_ident("Auto"))])
            .unwrap();
    let before = names.clone();
    assert_eq!(
        names
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 4))
            .unwrap(),
        "Auto"
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 2, 4),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 1, 4),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 2, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            names
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(names, before);
    }
}

#[test]
fn mixed_transition_none_diagnostic_identifies_the_property_after_a_duration() {
    let source = r"transition:1s \6e one, opacity 2s;color:red";
    let report = parse_style_attribute(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected mixed-none list");
    };
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("transition grammar error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Transition);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find(r"\6e one").unwrap()
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let [color] = report.syntax().as_slice() else {
        panic!("following color declaration");
    };
    assert_eq!(color.known().unwrap().property(), CssKnownProperty::Color);
}
