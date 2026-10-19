#![forbid(unsafe_code)]
//! Functional Speech 1 §§11.1/11.5 authored models. Independent expectations
//! come from the pinned Speech grammar, borrowed Fonts family-name syntax and
//! CSSOM serialization: quoted/identifier form and optional variant identity
//! remain authored; ordinary positive indices have no machine magnitude bound.
use surgeist_css::*;

fn parsed(text: &str) -> CssVoiceFamily {
    let report = parse_style_attribute(&format!("voice-family:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::VoiceFamily(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("voice wrapper")
    };
    value.value().clone()
}
fn named(values: &[&str]) -> CssVoiceFamilyName {
    CssVoiceFamilyName::try_identifiers(
        values
            .iter()
            .map(|value| CssIdent::try_new(*value).unwrap())
            .collect(),
    )
    .unwrap()
}
fn family(entries: Vec<CssVoiceFamilyEntry>) -> CssVoiceFamily {
    CssVoiceFamily::Voices(CssVoiceFamilyList::try_new(entries).unwrap())
}
fn positive(text: &str) -> CssPositiveIntegerValue {
    CssPositiveIntegerValue::Literal(
        CssPositiveIntegerLiteral::try_new(
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn checked_voice_names_keep_quoted_and_identifier_forms_and_reject_reserved_tokens() {
    assert!(CssVoiceFamilyName::try_identifiers(vec![]).is_none());
    assert!(CssVoiceFamilyName::try_quoted("a\0b").is_err());
    for word in [
        "male",
        "FEMALE",
        "neutral",
        "preserve",
        "default",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert!(
            CssVoiceFamilyName::try_identifiers(vec![
                CssIdent::try_new("Mike").unwrap(),
                CssIdent::try_new(word).unwrap()
            ])
            .is_none()
        );
        assert!(CssVoiceFamilyName::try_quoted(word).is_ok());
    }
    let quoted = CssVoiceFamilyName::try_quoted("Mike").unwrap();
    let identifiers = named(&["Mike"]);
    assert_ne!(quoted, identifiers);
    assert!(matches!(
        quoted.view(),
        CssVoiceFamilyNameRef::Quoted("Mike")
    ));
    assert!(
        matches!(identifiers.view(), CssVoiceFamilyNameRef::Identifiers(values) if values[0].as_str() == "Mike")
    );
    for word in ["child", "young", "old", "serif", "sans-serif", "menu"] {
        assert_eq!(
            family(vec![CssVoiceFamilyEntry::Name(named(&[word]))])
                .serialize_specified()
                .unwrap(),
            word
        );
    }
}

#[test]
fn voice_list_is_nonempty_and_preserve_remains_a_distinct_alternative() {
    assert!(CssVoiceFamilyList::try_new(vec![]).is_none());
    assert_eq!(
        CssVoiceFamily::Preserve.serialize_specified().unwrap(),
        "preserve"
    );
    let value = family(vec![
        CssVoiceFamilyEntry::Name(named(&["john", "doe"])),
        CssVoiceFamilyEntry::Name(CssVoiceFamilyName::try_quoted("valley girl").unwrap()),
    ]);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "john doe, \"valley girl\""
    );
    assert!(matches!(&value, CssVoiceFamily::Voices(list) if list.entries().len() == 2));
    assert_ne!(
        CssVoiceFamily::Preserve,
        family(vec![CssVoiceFamilyEntry::Name(
            CssVoiceFamilyName::try_quoted("preserve").unwrap()
        )])
    );
}

#[test]
fn generic_voice_retains_optional_age_and_variant_without_default_voice_selection() {
    let omitted = CssGenericVoice::try_new(None, CssVoiceGender::Neutral, None).unwrap();
    let explicit =
        CssGenericVoice::try_new(None, CssVoiceGender::Neutral, Some(positive("1"))).unwrap();
    assert_eq!(omitted.age(), None);
    assert_eq!(omitted.gender(), CssVoiceGender::Neutral);
    assert!(omitted.variant().is_none());
    assert_ne!(omitted, explicit);
    assert_eq!(
        family(vec![CssVoiceFamilyEntry::Generic(omitted)])
            .serialize_specified()
            .unwrap(),
        "neutral"
    );
    assert_eq!(
        family(vec![CssVoiceFamilyEntry::Generic(explicit)])
            .serialize_specified()
            .unwrap(),
        "neutral 1"
    );
    for (age, spelling) in [
        (CssVoiceAge::Child, "child"),
        (CssVoiceAge::Young, "young"),
        (CssVoiceAge::Old, "old"),
    ] {
        let value =
            CssGenericVoice::try_new(Some(age), CssVoiceGender::Female, Some(positive("+0002")))
                .unwrap();
        assert_eq!(value.age(), Some(age));
        assert!(
            matches!(value.variant(), Some(CssPositiveIntegerValue::Literal(value)) if value.integer().numeric().representation() == "+0002")
        );
        assert_eq!(
            family(vec![CssVoiceFamilyEntry::Generic(value)])
                .serialize_specified()
                .unwrap(),
            format!("{spelling} female 2")
        );
    }
}

#[test]
fn voice_variant_serialization_keeps_exact_large_indices() {
    for digits in [
        "9007199254740993",
        "999999999999999999999999999999999999999999999999999999999999999",
    ] {
        let generic =
            CssGenericVoice::try_new(None, CssVoiceGender::Male, Some(positive(digits))).unwrap();
        let value = family(vec![CssVoiceFamilyEntry::Generic(generic)]);
        assert_eq!(
            value.serialize_specified().unwrap(),
            format!("male {digits}")
        );
        assert_eq!(
            parsed(&value.serialize_specified().unwrap())
                .serialize_specified()
                .unwrap(),
            format!("male {digits}")
        );
    }
}

#[test]
fn checked_generic_constructor_validates_bare_calculation_literals_and_retains_math() {
    for literal in [0, -1] {
        let error = CssGenericVoice::try_new(
            None,
            CssVoiceGender::Male,
            Some(CssPositiveIntegerValue::Calculation(
                CssIntegerCalculation::literal(literal),
            )),
        )
        .unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    }
    let literal = CssGenericVoice::try_new(
        None,
        CssVoiceGender::Male,
        Some(CssPositiveIntegerValue::Calculation(
            CssIntegerCalculation::literal(2),
        )),
    )
    .unwrap();
    assert!(matches!(
        literal.variant(),
        Some(CssPositiveIntegerValue::Literal(_))
    ));
    let calculation =
        CssIntegerCalculation::try_from_components(parse_component_values("calc(1 + 1)").unwrap())
            .unwrap();
    let generic = CssGenericVoice::try_new(
        None,
        CssVoiceGender::Male,
        Some(CssPositiveIntegerValue::Calculation(calculation)),
    )
    .unwrap();
    assert!(matches!(
        generic.variant(),
        Some(CssPositiveIntegerValue::Calculation(_))
    ));
    let value = family(vec![CssVoiceFamilyEntry::Generic(generic)]);
    // Specified math projection is owned by the shared integer engine.
    let serialized = value.serialize_specified().unwrap();
    assert!(parse_style_attribute(&format!("voice-family:{serialized}")).is_clean());
    assert!(
        matches!(parsed("male min(2, 3)"), CssVoiceFamily::Voices(list) if matches!(list.entries()[0], CssVoiceFamilyEntry::Generic(_)))
    );
}

#[test]
fn voice_names_use_shared_escaping_and_roundtrip_without_form_conversion() {
    for name in [
        "",
        "valley girl",
        "a\"b\\c",
        "Henry\tthe-8th",
        "line\nbreak",
    ] {
        let value = family(vec![CssVoiceFamilyEntry::Name(
            CssVoiceFamilyName::try_quoted(name).unwrap(),
        )]);
        assert_eq!(parsed(&value.serialize_specified().unwrap()), value);
    }
    for ident in ["john/doe", "1stvoice", "Mike Smith", "a,b"] {
        let value = family(vec![CssVoiceFamilyEntry::Name(named(&[ident]))]);
        assert_eq!(parsed(&value.serialize_specified().unwrap()), value);
    }
}

#[test]
fn voice_family_serialization_has_cumulative_atomic_limits() {
    let value = parsed("Mike, young female +0002");
    let original = value.clone();
    let css = value.serialize_specified().unwrap();
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, usize::MAX, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, 0, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, css.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
        // The first entry fits by itself, but all entries share one budget.
        (
            CssSpecifiedValueSerializationLimits::new(3, usize::MAX, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, original);
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                css.len()
            ))
            .unwrap(),
        css
    );
}

#[test]
fn voice_stress_models_serialize_all_five_symbolic_modes() {
    for (value, css) in [
        (CssVoiceStress::Normal, "normal"),
        (CssVoiceStress::Strong, "strong"),
        (CssVoiceStress::Moderate, "moderate"),
        (CssVoiceStress::None, "none"),
        (CssVoiceStress::Reduced, "reduced"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), css);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    css.len()
                ))
                .unwrap(),
            css
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    css.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        let report = parse_style_attribute(&format!("voice-stress:{css}"));
        let CssKnownPropertyValueRef::VoiceStress(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("stress wrapper")
        };
        assert_eq!(*wrapper.value(), value);
    }
}

#[test]
fn checked_generic_construction_rejects_original_recovered_variant_calculations() {
    let report = parse_style_attribute("voice-family:male calc(1");
    assert!(!report.is_clean());
    assert!(validate_style_attribute("voice-family:male calc(1").is_err());
    let CssKnownPropertyValueRef::VoiceFamily(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("voice wrapper")
    };
    let CssVoiceFamily::Voices(list) = wrapper.value() else {
        panic!("voice list")
    };
    let CssVoiceFamilyEntry::Generic(generic) = &list.entries()[0] else {
        panic!("generic voice")
    };
    assert!(
        CssVoiceFamilyList::try_new(vec![CssVoiceFamilyEntry::Generic(generic.clone())]).is_none()
    );
    assert!(matches!(
        generic.variant(),
        Some(CssPositiveIntegerValue::Calculation(_))
    ));
    assert_eq!(
        CssGenericVoice::try_new(generic.age(), generic.gender(), generic.variant().cloned())
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
}
