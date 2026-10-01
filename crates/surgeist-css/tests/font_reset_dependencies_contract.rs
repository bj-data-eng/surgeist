#![forbid(unsafe_code)]

//! Authored font reset dependencies from pinned CSS Fonts 4 WD 2026-09-07,
//! §§2.6, 4.6, 6.3, 6.12, 6.13, 8.1 and 8.2.

use surgeist_css::*;

const PROPERTIES: [(&str, &str); 6] = [
    ("font-feature-settings", "normal"),
    ("font-kerning", "auto"),
    ("font-size-adjust", "none"),
    ("font-language-override", "normal"),
    ("font-optical-sizing", "auto"),
    ("font-variation-settings", "normal"),
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known grammar: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn one_contribution(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one terminal contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    item.ordinary_value().unwrap().clone()
}

fn number_representation(component: &CssComponentValue) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        panic!("number token")
    };
    number.representation()
}

fn feature(value: &str) -> CssAuthoredFontFeatureSettings {
    let source = declaration("font-feature-settings", value);
    let CssKnownPropertyValueRef::FontFeatureSettings(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("feature wrapper")
    };
    wrapper.settings().clone()
}

fn variation(value: &str) -> CssFontVariationSettings {
    let source = declaration("font-variation-settings", value);
    let CssKnownPropertyValueRef::FontVariationSettings(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("variation wrapper")
    };
    wrapper.variations().clone()
}

fn descriptor(kind: CssFontFaceDescriptorKind, value: &str) -> CssFontFaceDescriptorValue {
    let report = parse_font_face_descriptor_value(value, kind);
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(value)) = report.syntax() else {
        panic!("ordinary descriptor")
    };
    value.clone()
}

fn descriptor_payload_css(value: &CssFontFaceDescriptorValue) -> String {
    match value {
        CssFontFaceDescriptorValue::FontFeatureSettings(settings) => {
            settings.serialize_specified().unwrap()
        }
        CssFontFaceDescriptorValue::FontVariationSettings(settings) => {
            settings.serialize_specified().unwrap()
        }
        _ => panic!("font settings descriptor payload"),
    }
}

#[test]
fn feature_index_keeps_exact_integer_token_beyond_i32_and_rejects_wrong_roots() {
    for text in [
        "2147483648",
        "999999999999999999999999999999999999",
        "+0002147483648",
        "-0",
    ] {
        let parsed = parse_component_values(text).unwrap();
        let index = CssFontFeatureIndex::try_from_component(parsed.items()[0].clone()).unwrap();
        assert_eq!(
            number_representation(index.literal_component().unwrap()),
            text
        );
        assert!(index.calculation().is_none());
        let canonical = match text {
            "-0" => "0",
            "+0002147483648" => "2147483648",
            _ => text,
        };
        assert_eq!(index.serialize_specified().unwrap(), canonical);
        assert_eq!(index.i32_value(), if text == "-0" { Some(0) } else { None });
        assert!(matches!(index.origin(), CssValueOrigin::Parsed(_)));
    }
    let small = CssFontFeatureIndex::try_new(7).unwrap();
    assert_eq!(small.i32_value(), Some(7));
    assert_eq!(small.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(small.serialize_specified().unwrap(), "7");
    assert!(CssFontFeatureIndex::try_new(-1).is_none());
    for text in ["-1", "-1e-999", "1.5", "1e2", "2%"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        assert!(
            CssFontFeatureIndex::try_from_component(component).is_err(),
            "{text}"
        );
    }
    let math =
        CssIntegerCalculation::try_from_components(parse_component_values("calc(2 + 3)").unwrap())
            .unwrap();
    let symbolic = CssFontFeatureIndex::try_from_calculation(math).unwrap();
    assert!(symbolic.literal_component().is_none());
    assert!(symbolic.calculation().is_some());
    assert_eq!(symbolic.i32_value(), None);
    assert_eq!(
        symbolic
            .calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(2 + 3)"
    );
    assert_eq!(symbolic.serialize_specified().unwrap(), "calc(5)");
    let bare =
        CssIntegerCalculation::try_from_components(parse_component_values("2147483648").unwrap())
            .unwrap();
    let exact_bare = CssFontFeatureIndex::try_from_calculation(bare).unwrap();
    assert_eq!(
        number_representation(exact_bare.literal_component().unwrap()),
        "2147483648"
    );
    assert!(exact_bare.calculation().is_none());
    for text in ["-1", "2.5", "1e2"] {
        let calculation =
            CssIntegerCalculation::try_from_components(parse_component_values(text).unwrap());
        assert!(
            calculation.map_or(true, |calculation| {
                CssFontFeatureIndex::try_from_calculation(calculation).is_err()
            }),
            "{text}"
        );
    }
}

#[test]
fn feature_list_distinguishes_omitted_on_off_and_exact_index_without_deduplication() {
    let authored = feature("\"cv01\", \"cv01\" on, \"cv01\" off, \"cv01\" 2147483648, \"CV01\" 0");
    let CssAuthoredFontFeatureSettings::Features(list) = &authored else {
        panic!("five features")
    };
    let values = list.features();
    assert_eq!(values.len(), 5);
    assert_eq!(
        values
            .iter()
            .map(|value| value.tag().as_str())
            .collect::<Vec<_>>(),
        ["cv01", "cv01", "cv01", "cv01", "CV01"]
    );
    assert!(matches!(
        values[0].value(),
        CssAuthoredFontFeatureValue::Omitted
    ));
    assert!(matches!(values[1].value(), CssAuthoredFontFeatureValue::On));
    assert!(matches!(
        values[2].value(),
        CssAuthoredFontFeatureValue::Off
    ));
    let CssAuthoredFontFeatureValue::Index(index) = values[3].value() else {
        panic!("exact index")
    };
    assert_eq!(
        number_representation(index.literal_component().unwrap()),
        "2147483648"
    );
    assert_eq!(index.i32_value(), None);
    assert_eq!(
        authored.serialize_specified().unwrap(),
        "\"cv01\", \"cv01\" on, \"cv01\" off, \"cv01\" 2147483648, \"CV01\" 0"
    );
    assert!(CssAuthoredFontFeatureList::try_new(vec![]).is_none());
    assert!(CssOpenTypeTag::try_new(" abc").is_some());
    assert!(CssOpenTypeTag::try_new("~ABC").is_some());
    for invalid in ["abc", "abcde", "éabc", "\u{1}abc", "abc\u{7f}"] {
        assert!(CssOpenTypeTag::try_new(invalid).is_none(), "{invalid:?}");
    }
    let tag = CssOpenTypeTag::try_new("cv01").unwrap();
    let omitted = CssAuthoredFontFeature::new(tag.clone(), CssAuthoredFontFeatureValue::Omitted);
    let on = CssAuthoredFontFeature::new(tag.clone(), CssAuthoredFontFeatureValue::On);
    assert_ne!(omitted, on);
    let direct = CssAuthoredFontFeatureSettings::Features(
        CssAuthoredFontFeatureList::try_new(vec![CssAuthoredFontFeature::new(
            tag,
            CssAuthoredFontFeatureValue::Index(CssFontFeatureIndex::try_new(7).unwrap()),
        )])
        .unwrap(),
    );
    assert_eq!(direct, feature("\"cv01\" 7"));
    assert_ne!(direct, feature("\"cv01\" 07"));
    assert_ne!(
        authored,
        feature("\"cv01\", \"cv01\" on, \"cv01\" off, \"CV01\" 0")
    );
}

#[test]
fn scalar_number_and_size_adjust_keep_exact_sign_digits_origin_and_symbolic_math() {
    for (text, canonical) in [
        ("-1e-999", "0".to_owned()),
        (
            "-12345678901234567890.0000001",
            "-12345678901234567890".to_owned(),
        ),
        ("+0.5", "0.5".to_owned()),
        ("1e999", format!("1{}", "0".repeat(999))),
    ] {
        let components = parse_component_values(text).unwrap();
        let number = CssSpecifiedNumber::try_from_component(components.items()[0].clone()).unwrap();
        assert_eq!(
            number_representation(number.literal_component().unwrap()),
            text
        );
        assert_eq!(number.serialize_specified().unwrap(), canonical);
        assert!(matches!(number.origin(), CssValueOrigin::Parsed(_)));
    }
    let parsed = CssSpecifiedNumber::try_from_component(
        parse_component_values("-1.25").unwrap().items()[0].clone(),
    )
    .unwrap();
    let programmatic =
        CssSpecifiedNumber::try_from_component(CssComponentValue::try_number("-1.25").unwrap())
            .unwrap();
    assert_ne!(parsed, programmatic);
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    let math = CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 - 2)").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(math.literal_component().is_none());
    assert!(math.calculation().is_some());
    assert_eq!(
        math.calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(1 - 2)"
    );
    assert_eq!(math.serialize_specified().unwrap(), "calc(-1)");
    let bare =
        CssNumberCalculation::try_from_components(parse_component_values("-1e-999").unwrap())
            .unwrap();
    let exact_bare = CssSpecifiedNumber::try_from_calculation(bare).unwrap();
    assert_eq!(
        number_representation(exact_bare.literal_component().unwrap()),
        "-1e-999"
    );
    assert!(exact_bare.calculation().is_none());
    let adjust = CssFontSizeAdjust::Number(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number("0.000000000000000000000000000000000000001").unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(adjust.serialize_specified().unwrap(), "0");
    let CssFontSizeAdjust::Number(number) = &adjust else {
        panic!("numeric size adjust")
    };
    assert_eq!(
        number_representation(number.literal_component().unwrap()),
        "0.000000000000000000000000000000000000001"
    );
    for negative in ["-1e-999", "-0.000000000000000000000000000000000000001"] {
        assert!(
            CssSpecifiedNonNegativeNumber::try_from_component(
                CssComponentValue::try_number(negative).unwrap()
            )
            .is_err(),
            "{negative}"
        );
    }
    let source = declaration("font-size-adjust", "1e999");
    let CssKnownPropertyValueRef::FontSizeAdjust(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("size adjust")
    };
    let CssFontSizeAdjust::Number(value) = wrapper.size_adjust() else {
        panic!("number adjust")
    };
    assert_eq!(
        number_representation(value.literal_component().unwrap()),
        "1e999"
    );
    assert_eq!(
        wrapper.size_adjust().serialize_specified().unwrap(),
        format!("1{}", "0".repeat(999))
    );
}

#[test]
fn variation_list_keeps_signed_values_order_duplicates_and_case() {
    let authored = variation("\"wght\" -1.25, \"wdth\" +0.5, \"wdth\" -2, \"WDTH\" 1e3");
    let CssFontVariationSettings::Variations(list) = &authored else {
        panic!("four axes")
    };
    let axes = list.variations();
    assert_eq!(axes.len(), 4);
    assert_eq!(
        axes.iter()
            .map(|axis| axis.tag().as_str())
            .collect::<Vec<_>>(),
        ["wght", "wdth", "wdth", "WDTH"]
    );
    assert_eq!(
        axes.iter()
            .map(|axis| number_representation(axis.value().literal_component().unwrap()))
            .collect::<Vec<_>>(),
        ["-1.25", "+0.5", "-2", "1e3"]
    );
    assert_eq!(
        authored.serialize_specified().unwrap(),
        "\"wght\" -1.25, \"wdth\" 0.5, \"wdth\" -2, \"WDTH\" 1000"
    );
    assert!(CssFontVariationList::try_new(vec![]).is_none());
    let tag = CssOpenTypeTag::try_new("wght").unwrap();
    let programmatic = CssFontVariationSettings::Variations(
        CssFontVariationList::try_new(vec![CssFontVariation::new(
            tag,
            CssSpecifiedNumber::try_from_component(CssComponentValue::try_number("-1.25").unwrap())
                .unwrap(),
        )])
        .unwrap(),
    );
    assert_eq!(programmatic, variation("\"wght\" -1.25"));
    assert_ne!(
        authored,
        variation("\"wght\" -1.25, \"WDTH\" 1e3, \"wdth\" +0.5, \"wdth\" -2")
    );
}

#[test]
fn language_override_preserves_general_decoded_string_and_rejects_nul_construction() {
    for (source, decoded) in [
        ("\"\"", ""),
        ("\"SRB é Long\"", "SRB é Long"),
        (r#""a\22 b""#, "a\"b"),
    ] {
        let parsed = declaration("font-language-override", source);
        let CssKnownPropertyValueRef::FontLanguageOverride(wrapper) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("language wrapper")
        };
        let CssFontLanguageOverride::String(value) = wrapper.language_override() else {
            panic!("language string")
        };
        assert_eq!(value.as_str(), decoded);
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        let programmatic = CssFontLanguageString::try_new(decoded).unwrap();
        assert_eq!(programmatic.as_str(), decoded);
        assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(programmatic, *value);
        let from_component = CssFontLanguageString::try_from_component(
            CssComponentValue::try_string(decoded).unwrap(),
        )
        .unwrap();
        assert_eq!(from_component, programmatic);
    }
    assert!(CssFontLanguageString::try_new("a\0b").is_err());
    assert!(
        CssFontLanguageString::try_from_component(CssComponentValue::try_ident("SRB").unwrap())
            .is_err()
    );
    assert_eq!(
        CssFontLanguageOverride::Normal
            .serialize_specified()
            .unwrap(),
        "normal"
    );
    assert_eq!(
        CssFontLanguageOverride::String(CssFontLanguageString::try_new("SRB é Long").unwrap())
            .serialize_specified()
            .unwrap(),
        "\"SRB é Long\""
    );
}

#[test]
fn six_longhands_have_exact_initials_and_preserve_globals_and_reentry_provenance() {
    for (name, initial) in PROPERTIES {
        let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("longhand metadata: {name}")
        };
        assert!(metadata.inherited_by_default(), "{name}");
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("fixed initial: {name}")
        };
        assert_eq!(value, &one_contribution(&declaration(name, initial)));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("global expansion: {name}")
            };
            let [item] = values.items() else {
                panic!("one global: {name}")
            };
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
        }
        for symbolic in ["var(--setting)", "env(setting)"] {
            let source = declaration(name, symbolic);
            let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
                panic!("pending {name}")
            };
            assert!(pending.source().same_occurrence(&source));
            let replacement = parse_component_values(initial).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentry {name}")
            };
            let [item] = values.items() else {
                panic!("one reentry {name}")
            };
            assert_eq!(
                item.ordinary_value(),
                Some(&one_contribution(&declaration(name, initial)))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.replacement_components(), Some(&replacement));
            for residual in ["var(--again)", "env(again)"] {
                assert!(matches!(
                    pending
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
        }
    }
}

#[test]
fn descriptor_payloads_use_property_models_and_effective_last_admitted_order() {
    let feature_text = "\"cv01\" 2147483648, \"cv01\" off";
    let variation_text = "\"wght\" -1.25, \"wght\" 2";
    assert_eq!(
        descriptor(CssFontFaceDescriptorKind::FontFeatureSettings, feature_text),
        CssFontFaceDescriptorValue::FontFeatureSettings(feature(feature_text))
    );
    assert_eq!(
        descriptor(
            CssFontFaceDescriptorKind::FontVariationSettings,
            variation_text
        ),
        CssFontFaceDescriptorValue::FontVariationSettings(variation(variation_text))
    );
    assert_eq!(
        descriptor_payload_css(&descriptor(
            CssFontFaceDescriptorKind::FontFeatureSettings,
            feature_text
        )),
        feature_text
    );
    assert_eq!(
        descriptor_payload_css(&descriptor(
            CssFontFaceDescriptorKind::FontVariationSettings,
            variation_text
        )),
        variation_text
    );
    let source = "@font-face{font-feature-settings:\"kern\" on;font-variation-settings:\"wght\" 1;font-variation-settings:bad;font-feature-settings:env(features);font-variation-settings:\"wght\" -2}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    let ErrorKind::InvalidDescriptorValue(detail) = report.diagnostics()[0].error().kind() else {
        panic!("descriptor-mapped grammar error")
    };
    assert_eq!(detail.at_rule().as_str(), "font-face");
    assert_eq!(detail.descriptor().as_str(), "font-variation-settings");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("retained face")
    };
    let records = face.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(records.len(), 4);
    assert_eq!(
        records
            .iter()
            .map(|record| record.value().kind())
            .collect::<Vec<_>>(),
        [
            CssFontFaceDescriptorKind::FontFeatureSettings,
            CssFontFaceDescriptorKind::FontVariationSettings,
            CssFontFaceDescriptorKind::FontFeatureSettings,
            CssFontFaceDescriptorKind::FontVariationSettings
        ]
    );
    assert!(matches!(
        records[2].value(),
        CssAuthoredFontFaceDescriptorValue::Pending(_)
    ));
    assert!(std::ptr::eq(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::FontFeatureSettings)
            .unwrap(),
        records[2]
    ));
    assert!(std::ptr::eq(
        face.descriptors()
            .effective(CssFontFaceDescriptorKind::FontVariationSettings)
            .unwrap(),
        records[3]
    ));
    for (record, needle) in records.iter().zip([
        "font-feature-settings:\"kern\"",
        "font-variation-settings:\"wght\" 1",
        "font-feature-settings:env",
        "font-variation-settings:\"wght\" -2",
    ]) {
        assert_eq!(
            record.position().unwrap().byte_offset().value(),
            source.find(needle).unwrap()
        );
    }
}

#[test]
fn descriptor_pending_reentry_is_strict_and_keeps_new_variation_kind() {
    for (kind, text) in [
        (
            CssFontFaceDescriptorKind::FontFeatureSettings,
            "\"cv01\" 2147483648",
        ),
        (
            CssFontFaceDescriptorKind::FontVariationSettings,
            "\"wght\" -1.25",
        ),
    ] {
        let pending = CssAuthoredFontFaceDescriptorValue::try_from_components(
            kind,
            parse_component_values("env(axis)").unwrap(),
        )
        .unwrap();
        let CssAuthoredFontFaceDescriptorValue::Pending(pending) = pending else {
            panic!("pending descriptor")
        };
        assert_eq!(pending.kind(), kind);
        let replacement = parse_component_values(text).unwrap();
        let ordinary = pending.reparse_after_substitution(replacement).unwrap();
        assert_eq!(ordinary.kind(), kind);
        assert_eq!(descriptor_payload_css(&ordinary), text);
        for invalid in ["env(axis)", "var(--axis)", "inherit"] {
            assert!(
                pending
                    .reparse_after_substitution(parse_component_values(invalid).unwrap())
                    .is_err(),
                "{invalid}"
            );
        }
        let recovered = parse_component_values("calc(1").unwrap();
        assert!(pending.reparse_after_substitution(recovered).is_err());
    }
}

#[test]
fn otherwise_valid_implicitly_closed_math_recovers_only_in_ordinary_parsing() {
    for (property, incomplete, complete) in [
        (
            "font-feature-settings",
            "\"cv01\" calc(1",
            "\"cv01\" calc(1)",
        ),
        ("font-size-adjust", "calc(1", "calc(1)"),
        (
            "font-variation-settings",
            "\"wght\" calc(1",
            "\"wght\" calc(1)",
        ),
    ] {
        let source = format!("{property}:{incomplete}");
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "recovered value retained: {source}"
        );
        assert!(!report.is_clean(), "implicit closure diagnosed: {source}");
        assert!(validate_style_attribute(&source).is_err());
        let recovered = parse_component_values(incomplete).unwrap();
        let checked_error = parse_property_value_for_grammar(
            grammar(property),
            recovered.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(matches!(
            checked_error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { at, .. })) =
            checked_error.origin()
        else {
            panic!("checked error maps to original implicit EOF")
        };
        assert_eq!(at.source().as_str(), incomplete);
        assert_eq!(at.span().start().byte_offset().value(), incomplete.len());
        assert_eq!(at.span().end().byte_offset().value(), incomplete.len());
        let pending_source = declaration(property, "var(--font-setting)");
        let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
            panic!("pending property: {property}")
        };
        assert!(matches!(
            pending.reenter(recovered).unwrap_err().kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        declaration(property, complete);
        parse_property_value_for_grammar(
            grammar(property),
            parse_component_values(complete).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    }

    for (kind, incomplete, complete) in [
        (
            CssFontFaceDescriptorKind::FontFeatureSettings,
            "\"cv01\" calc(1",
            "\"cv01\" calc(1)",
        ),
        (
            CssFontFaceDescriptorKind::FontVariationSettings,
            "\"wght\" calc(1",
            "\"wght\" calc(1)",
        ),
    ] {
        let source = format!("@font-face{{{}:{incomplete}", kind.css_name());
        let report = parse_sheet(&source);
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("recovered font face retained")
        };
        assert_eq!(face.descriptors().occurrences().len(), 1);
        assert!(!report.is_clean());
        let recovered = parse_component_values(incomplete).unwrap();
        let function_origin = recovered
            .items()
            .iter()
            .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
            .expect("recovered calc function")
            .origin()
            .clone();
        let CssValueOrigin::Parsed(function_origin) = function_origin else {
            panic!("authored calc opening")
        };
        let checked_error =
            CssAuthoredFontFaceDescriptorValue::try_from_components(kind, recovered.clone())
                .unwrap_err();
        assert_eq!(
            checked_error.kind(),
            &CssFontFaceValueErrorKind::RecoveredComponent
        );
        let CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { opening, at }) =
            checked_error.origin()
        else {
            panic!("descriptor error maps to implicit closure")
        };
        assert_eq!(opening, &function_origin);
        assert_eq!(opening.source().as_str(), incomplete);
        assert_eq!(
            opening.span().start().byte_offset().value(),
            incomplete.find("calc").unwrap()
        );
        assert_eq!(
            opening.span().end().byte_offset().value(),
            incomplete.find("calc").unwrap() + 5
        );
        assert_eq!(at.source().as_str(), incomplete);
        assert_eq!(at.span().start().byte_offset().value(), incomplete.len());
        assert_eq!(at.span().end().byte_offset().value(), incomplete.len());
        let CssAuthoredFontFaceDescriptorValue::Pending(pending) =
            CssAuthoredFontFaceDescriptorValue::try_from_components(
                kind,
                parse_component_values("env(axis)").unwrap(),
            )
            .unwrap()
        else {
            panic!("pending descriptor")
        };
        assert_eq!(
            pending
                .reparse_after_substitution(recovered)
                .unwrap_err()
                .kind(),
            &CssFontFaceValueErrorKind::RecoveredComponent
        );
        descriptor(kind, complete);
    }
}

#[test]
fn list_serialization_obeys_shared_atomic_input_projection_and_byte_limits() {
    let value = feature("\"cv01\" 2147483648, \"cv02\" off");
    let expected = "\"cv01\" 2147483648, \"cv02\" off";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(
        feature("\"cv01\" 2147483648")
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 100))
            .unwrap(),
        "\"cv01\" 2147483648"
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1000, 1000),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1000, 0, 1000),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1000, 1000, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
    let axes = variation("\"wght\" -1.25, \"wdth\" 2");
    let exact = "\"wght\" -1.25, \"wdth\" 2";
    assert_eq!(axes.serialize_specified().unwrap(), exact);
    assert_eq!(
        variation("\"wght\" -1.25")
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 100))
            .unwrap(),
        "\"wght\" -1.25"
    );
    assert_eq!(
        axes.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        axes.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            1000,
            1000,
            exact.len() - 1
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(axes.serialize_specified().unwrap(), exact);
}

#[test]
fn normalized_nested_declarations_retain_order_and_source_occurrences() {
    let report = parse_sheet(
        ".a{font-kerning:none;font-size-adjust:0.5;& .b{font-optical-sizing:none;font-language-override:\"SRB\"}font-variation-settings:\"wght\" -1}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 5);
    assert_eq!(
        declarations
            .iter()
            .map(|value| value.source().known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "font-kerning",
            "font-size-adjust",
            "font-optical-sizing",
            "font-language-override",
            "font-variation-settings"
        ]
    );
    for (order, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("terminal normalized declaration")
        };
        let [contribution] = values.items() else {
            panic!("one normalized contribution")
        };
        assert!(contribution.source().same_occurrence(item.source()));
    }
}
