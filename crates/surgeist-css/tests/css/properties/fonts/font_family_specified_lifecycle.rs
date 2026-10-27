//! Fonts 4 (7 September 2026) §2.1.1/§2.1.2 retain authored family forms;
//! §13 delegates escaping to CSSOM. These expectations distinguish specified
//! identifier tokens from the computed joined family name. Descriptor/feature
//! families instead own decoded literal names (§4.2 and §6.9.1).
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-name-syntax
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#serializing

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::important_declaration;
use crate::css::support::named_property::{assert_invalid_replacement_at, assert_origin_at};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssSerializedOrigin, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssTokenKind};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandValueRef, CssPropertyKindRef, CssPropertyNameRef,
    CssPropertyTarget, CssUserAgentInitial,
};
use surgeist_css::rule::CssRule;
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
    CssFontFaceFamily,
};
use surgeist_css::rule::font_palette_values::{
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue, CssFontPaletteDescriptorValueRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::font::{
    CssExplicitFont, CssFontFamilyList, CssFontFamilyName, CssFontFamilyNameKind, CssFontSize,
    CssGenericFontFamily,
};

fn families(value: &CssDeclaration) -> &CssFontFamilyList {
    let CssKnownPropertyValueRef::FontFamily(value) =
        value.known().unwrap().property_value().unwrap()
    else {
        panic!("family property")
    };
    value.value()
}

fn literal(text: &str) -> CssFontFaceFamily {
    let report = CssAuthoredFontFaceDescriptorValue::parse(
        &CssParser::default(),
        text,
        CssFontFaceDescriptorKind::FontFamily,
    );
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::FontFamily(
        value,
    ))) = report.syntax()
    else {
        panic!("literal family")
    };
    value.clone()
}

#[test]
fn parsed_property_forms_round_trip_without_joining_identifier_tokens() {
    let source = important_declaration(
        "font-family",
        r#"A\ B, A B, 'serif', SeRiF, generic(kai), menu"#,
    );
    let before = source.clone();
    let CssValueOrigin::Parsed(origin) = source.value_components().items()[0].origin() else {
        panic!("parsed first family token")
    };
    assert_eq!(origin.span().start().byte_offset().value(), 12);
    assert_eq!(origin.span().end().byte_offset().value(), 16);
    let value = families(&source);
    let css = value.serialize_specified(Default::default()).unwrap();
    assert_eq!(css, r#"A\ B, A B, "serif", serif, generic(kai), menu"#);
    assert_eq!(families(&important_declaration("font-family", &css)), value);
    assert_eq!(value.families()[0].identifier_tokens().unwrap(), &["A B"]);
    assert_eq!(
        value.families()[1].identifier_tokens().unwrap(),
        &["A", "B"]
    );
    assert_eq!(value.families()[2].kind(), CssFontFamilyNameKind::Quoted);
    assert_eq!(
        value.families()[3].generic_family(),
        Some(CssGenericFontFamily::Serif)
    );
    assert_eq!(source, before);
    assert!(source.same_occurrence(&before));
    assert_eq!(source.importance(), CssImportance::Important);
}

#[test]
fn checked_names_use_the_shared_cssom_escaping_and_preserve_authored_kind() {
    for (name, css) in [
        (CssFontFamilyName::try_ident("A B").unwrap(), r"A\ B"),
        (CssFontFamilyName::try_ident("1Face").unwrap(), r"\31 Face"),
        (CssFontFamilyName::try_ident("-").unwrap(), r"\-"),
        (CssFontFamilyName::try_ident("a+b").unwrap(), r"a\+b"),
        (CssFontFamilyName::try_ident("日本語").unwrap(), "日本語"),
        (
            CssFontFamilyName::try_quoted("a\"b\\c").unwrap(),
            r#""a\"b\\c""#,
        ),
        (CssFontFamilyName::try_quoted("").unwrap(), "\"\""),
        (
            CssFontFamilyName::try_quoted("default").unwrap(),
            "\"default\"",
        ),
    ] {
        assert_eq!(name.serialize_specified(Default::default()).unwrap(), css);
        assert_eq!(
            families(&important_declaration("font-family", css)).families(),
            &[name]
        );
    }
    for reserved in ["default", "DeFaUlT", "initial", "serif", "A"] {
        let tokens = vec!["Face".into(), reserved.into()];
        assert_eq!(
            CssFontFamilyName::try_ident_sequence(tokens).is_none(),
            reserved != "A"
        );
    }
    assert!(CssFontFamilyName::try_quoted("a\0b").is_none());
    assert!(CssFontFamilyName::try_ident("a\0b").is_none());
    assert!(CssFontFaceFamily::try_new("a\0b").is_none());
    assert!(CssFontFamilyList::try_new(vec![]).is_none());
}

#[test]
fn list_limits_are_cumulative_exact_and_atomic() {
    let value = families(&important_declaration("font-family", "A, serif")).clone();
    let before = value.clone();
    // A: name + identifier; serif: one generic. Container is not a node.
    let exact = CssSerializationLimits::new(3, 3, 8);
    assert_eq!(value.serialize_specified(exact).unwrap(), "A, serif");
    for (limits, expected) in [
        (
            CssSerializationLimits::new(2, 3, 8),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 8),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 7),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            expected
        );
        assert_eq!(value, before);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "A, serif"
        );
    }
}

#[test]
fn standalone_name_limits_charge_tokens_and_emitted_utf8_bytes() {
    let value = CssFontFamilyName::try_ident_sequence(vec!["A".into(), "日本".into()]).unwrap();
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 3, 8))
            .unwrap(),
        "A 日本"
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(2, 3, 8))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 2, 8))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 3, 7))
            .unwrap_err()
            .is_byte_limit()
    );
    let quoted = CssFontFamilyName::try_quoted("a\"b").unwrap();
    assert_eq!(
        quoted
            .serialize_specified(CssSerializationLimits::new(1, 1, 6))
            .unwrap(),
        r#""a\"b""#
    );
    assert!(
        quoted
            .serialize_specified(CssSerializationLimits::new(1, 1, 5))
            .unwrap_err()
            .is_byte_limit()
    );
}

#[test]
fn checked_property_admission_retains_components_and_rejects_annotations() {
    let components = CssComponentValues::parse(&CssParser::default(), r#"A\ B, "serif""#).unwrap();
    let before = components.clone();
    let value = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::FontFamily)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(value.value_components(), &before);
    assert_eq!(
        families(&value)
            .serialize_specified(Default::default())
            .unwrap(),
        r#"A\ B, "serif""#
    );
    assert_eq!(value.value_components(), &before);
    let bad = CssComponentValues::parse(&CssParser::default(), "A!important").unwrap();
    let error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::FontFamily)),
        bad,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(error.code().is_some());
    assert!(matches!(
        error.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Parsed(_))
    ));
}

#[test]
fn metadata_globals_and_pending_reentry_keep_symbolic_lifecycle() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::FontFamily.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    assert!(metadata.inherited_by_default());
    assert!(matches!(
        metadata.initial_value().view(),
        CssInitialValueRef::UserAgent(CssUserAgentInitial::FontFamily)
    ));
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = important_declaration("font-family", text);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            source.expand().unwrap()
        else {
            panic!("global")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(
            values.items()[0].value(),
            CssContributionValueRef::Global(keyword)
        );
        assert!(values.items()[0].source().same_occurrence(&source));
    }
    for pending_text in ["var(--family)", "env(family)"] {
        let source = important_declaration("font-family", pending_text);
        let before = source.clone();
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("pending")
        };
        let replacement =
            CssComponentValues::parse(&CssParser::default(), r#"A\ B, "serif", serif"#).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("longhand")
        };
        let [item] = values.items() else {
            panic!("one family")
        };
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        let CssLonghandValueRef::FontFamily(value) = item.ordinary_value().unwrap().view() else {
            panic!("family")
        };
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            r#"A\ B, "serif", serif"#
        );
        // Fonts 4 §2.1: a trailing comma needs another family; input ends
        // there, so the comma is the last authored token.
        assert_invalid_replacement_at(
            &pending
                .reenter(CssComponentValues::parse(&CssParser::default(), "A,").unwrap())
                .unwrap_err(),
            CssKnownProperty::FontFamily,
            "A,",
            ",",
        );
        // CSS Syntax 3 §5.4.6: `!important` is a declaration annotation, not
        // part of a value, so strict reentry rejects its `!` delimiter as
        // syntax rather than as a family name.
        let error = pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "A!important").unwrap())
            .unwrap_err();
        let Some(error) = error.replacement_error() else {
            panic!("invalid replacement: {error:?}")
        };
        let token = &error;
        assert_eq!(
            token.code(),
            Some(CssErrorCode::UnexpectedToken),
            "unexpected `!`: {error:?}"
        );
        assert_eq!(token.encountered().unwrap().kind(), CssTokenKind::Delim);
        assert_eq!(token.encountered().unwrap().authored(), "!");
        assert_origin_at(&error.origin(), "A!important", "!important");
        assert_eq!(source, before);
    }
}

#[test]
fn invalid_family_lists_recover_locally_without_emitting_a_partial_list() {
    for text in ["A,", "A default", "A serif", "generic(unknown)", "A + B"] {
        let report =
            CssParser::default().parse_block_contents(format!("font-family:{text};color:red"));
        assert!(!report.is_clean(), "{text}");
        assert_eq!(report.syntax().len(), 1, "{text}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert!(
            CssParser::default()
                .parse_block_contents(format!("font-family:{text}"))
                .into_validation_result()
                .is_err()
        );
    }
}

#[test]
fn embedded_shorthand_shares_output_and_cumulative_node_contract() {
    let list = families(&important_declaration("font-family", "A, serif")).clone();
    let font =
        CssExplicitFont::try_new(None, None, None, None, CssFontSize::Medium, None, list).unwrap();
    // Size adds one node; the family list remains three nodes as above.
    assert_eq!(
        font.serialize_specified(CssSerializationLimits::new(4, 4, 15))
            .unwrap(),
        "medium A, serif"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(3, 4, 15),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 3, 15),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 4, 14),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&font.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
}

#[test]
fn decoded_descriptor_families_round_trip_as_literals_in_face_and_feature_consumers() {
    for (name, expected) in [
        ("Demo Font", "Demo Font"),
        ("serif", "\"serif\""),
        ("default", "\"default\""),
        ("A default", "\"A default\""),
        ("", "\"\""),
        (" A ", "\" A \""),
        ("A  B", "\"A  B\""),
        ("1Face", "\"1Face\""),
        ("日本", "日本"),
        ("a+b", "\"a+b\""),
        ("a\"b\\c", r#""a\"b\\c""#),
    ] {
        let family = CssFontFaceFamily::try_new(name).unwrap();
        assert_eq!(
            family.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(literal(expected), family);
        let face_sheet = CssParser::default().parse_sheet(format!(
            "@font-face {{font-family:{expected};src:url(font)}}"
        ));
        assert!(
            face_sheet.is_clean(),
            "{name}: {:?}",
            face_sheet.diagnostics()
        );
        let [CssRule::FontFace(face)] = face_sheet.syntax().rules() else {
            panic!("font face")
        };
        let descriptor = face
            .descriptors()
            .effective(CssFontFaceDescriptorKind::FontFamily)
            .unwrap();
        let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::FontFamily(
            face_family,
        )) = descriptor.value()
        else {
            panic!("face family")
        };
        assert_eq!(face_family, &family);
        let sheet =
            CssParser::default().parse_sheet(format!("@font-feature-values {expected} {{}}"));
        assert!(sheet.is_clean(), "{name}: {:?}", sheet.diagnostics());
        let [CssRule::FontFeatureValues(rule)] = sheet.syntax().rules() else {
            panic!("feature rule")
        };
        assert_eq!(rule.families(), &[family]);
    }
}

#[test]
fn literal_and_palette_routes_share_output_without_inventing_identifier_nodes() {
    let family = literal("'Demo Font'");
    assert_eq!(
        family
            .serialize_specified(CssSerializationLimits::new(1, 1, 9))
            .unwrap(),
        "Demo Font"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(0, 1, 9),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 9),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 8),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&family.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    let parsed = CssFontPaletteDescriptorValue::parse(
        &CssParser::default(),
        "'Demo Font', 'serif'",
        CssFontPaletteDescriptorKind::FontFamily,
    );
    assert!(parsed.is_clean(), "{:?}", parsed.diagnostics());
    let value = parsed.syntax().as_ref().unwrap();
    let css = "Demo Font, \"serif\"";
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(2, 2, css.len()))
            .unwrap(),
        css
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(1, 2, css.len()))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(2, 1, css.len()))
            .unwrap_err()
            .is_projection_node_limit()
    );
    let CssFontPaletteDescriptorValueRef::FontFamily(names) = value.view() else {
        panic!("palette family")
    };
    assert_eq!(
        names
            .iter()
            .map(|name| name.serialize_specified(Default::default()).unwrap())
            .collect::<Vec<_>>()
            .join(", "),
        css
    );
}

#[test]
fn all_generic_branches_emit_canonical_keywords_and_functions() {
    for css in [
        "serif",
        "sans-serif",
        "cursive",
        "fantasy",
        "monospace",
        "system-ui",
        "math",
        "ui-serif",
        "ui-sans-serif",
        "ui-monospace",
        "ui-rounded",
        "generic(fangsong)",
        "generic(kai)",
        "generic(khmer-mul)",
        "generic(nastaliq)",
    ] {
        let source = important_declaration("font-family", css);
        let [value] = families(&source).families() else {
            panic!("one generic")
        };
        assert_eq!(value.kind(), CssFontFamilyNameKind::Generic);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, css.len()))
                .unwrap(),
            css
        );
        assert_eq!(
            families(&important_declaration(
                "font-family",
                &value.serialize_specified(Default::default()).unwrap()
            ))
            .families(),
            std::slice::from_ref(value)
        );
    }
}
