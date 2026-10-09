#![forbid(unsafe_code)]
//! Existing public checked fronts must preserve their responsible token origin
//! when Values 5 WD 2024-11-11 §4.1.1 rejects a known request modifier.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#request-url-modifiers
//! Detaching components does not make their retained source coordinates into
//! coordinates in the shorter serialization used by checked grammar admission.

use surgeist_css::*;

const PREFIX_BYTES: usize = 128;
// Contextual property/font errors select the modifier opening; import errors
// select the target opening. Component-native profile/counter errors retain
// the invalid argument token. Each expected spelling is authored independently.
const INVALID: [(&str, &str, &str); 3] = [
    ("integrity(hash)", "hash", "integrity("),
    ("crossorigin(wrong)", "wrong", "crossorigin("),
    ("referrerpolicy(default)", "default", "referrerpolicy("),
];
const VALID: [&str; 3] = [
    "integrity(\"hash\")",
    "crossorigin(anonymous)",
    "referrerpolicy(origin)",
];
const PROPERTIES: [CssKnownProperty; 17] = [
    CssKnownProperty::BackgroundImage,
    CssKnownProperty::MaskImage,
    CssKnownProperty::BorderImageSource,
    CssKnownProperty::ListStyleImage,
    CssKnownProperty::Cursor,
    CssKnownProperty::Content,
    CssKnownProperty::Filter,
    CssKnownProperty::BackdropFilter,
    CssKnownProperty::ClipPath,
    CssKnownProperty::OffsetPath,
    CssKnownProperty::Background,
    CssKnownProperty::Mask,
    CssKnownProperty::BorderImage,
    CssKnownProperty::ListStyle,
    CssKnownProperty::CueBefore,
    CssKnownProperty::CueAfter,
    CssKnownProperty::Cue,
];
const COUNTERS: [(CssCounterStyleDescriptorKind, &str); 6] = [
    (CssCounterStyleDescriptorKind::Negative, ""),
    (CssCounterStyleDescriptorKind::Prefix, ""),
    (CssCounterStyleDescriptorKind::Suffix, ""),
    (CssCounterStyleDescriptorKind::Pad, "2 "),
    (CssCounterStyleDescriptorKind::Symbols, ""),
    (CssCounterStyleDescriptorKind::AdditiveSymbols, "1 "),
];

fn detached(text: &str) -> (String, CssComponentValues) {
    let source = format!("{}{text}", " ".repeat(PREFIX_BYTES));
    let parsed = parse_component_values(&source).unwrap();
    assert!(matches!(
        parsed.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    ));
    let detached = CssComponentValues::try_new(parsed.items()[1..].to_vec()).unwrap();
    assert_eq!(detached.serialize().unwrap().as_css(), text);
    let CssValueOrigin::Parsed(origin) = detached.items()[0].origin() else {
        panic!("retained original source")
    };
    assert_eq!(origin.span().start().byte_offset().value(), PREFIX_BYTES);
    // A deliberate coordinate mismatch; grammar must not use original offsets
    // as indexes into its serialized input.
    assert!(text.len() < PREFIX_BYTES);
    (source, detached)
}

fn original_argument(
    actual: &CssValueOrigin,
    input: &CssComponentValues,
    source: &str,
    spelling: &str,
) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(input_origin)) =
        (actual, input.items()[0].origin())
    else {
        panic!("original argument origin, not fabricated/programmatic: {actual:?}")
    };
    assert!(actual.source().same_snapshot(input_origin.source()));
    assert_eq!(actual.source().as_str(), source);
    let start = source.find(spelling).unwrap();
    assert_eq!(actual.span().start().byte_offset().value(), start);
    assert_eq!(
        actual.span().end().byte_offset().value(),
        start + spelling.len()
    );
    assert_eq!(&source[start..start + spelling.len()], spelling);
}

fn serialized_argument(
    actual: &CssSerializedOrigin,
    input: &CssComponentValues,
    source: &str,
    spelling: &str,
) {
    let CssSerializedOrigin::Token(origin) = actual else {
        panic!("responsible argument token: {actual:?}")
    };
    original_argument(origin, input, source, spelling);
}

fn property_text(property: CssKnownProperty, url: &str) -> String {
    if property == CssKnownProperty::Cursor {
        format!("{url}, auto")
    } else {
        url.to_owned()
    }
}

fn checked_property(
    property: CssKnownProperty,
    input: CssComponentValues,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        input,
        CssImportance::Normal,
    )
}

#[test]
fn valid_detached_property_and_profile_controls_are_admitted() {
    for function in ["url", "src"] {
        for modifier in VALID {
            let url = format!("{function}(\"#asset\" {modifier})");
            for property in PROPERTIES {
                let (_, input) = detached(&property_text(property, &url));
                assert!(
                    checked_property(property, input).is_ok(),
                    "{url}: {property:?}"
                );
            }
            let (_, input) = detached(&url);
            let profile =
                CssColorProfileDescriptorValue::try_new(CssColorProfileDescriptorKind::Src, input)
                    .unwrap();
            let CssColorProfileDescriptorValueRef::Src(url) = profile.view() else {
                panic!("ordinary profile source")
            };
            assert!(url.is_local_url());
            assert_eq!(url.as_str(), "#asset");
        }
    }
}

#[test]
fn valid_detached_font_import_and_counter_controls_are_admitted() {
    for function in ["url", "src"] {
        for modifier in VALID {
            let url = format!("{function}(\"#asset\" {modifier})");
            let (_, input) = detached(&url);
            assert!(
                CssAuthoredFontFaceDescriptorValue::try_from_components(
                    CssFontFaceDescriptorKind::Src,
                    input,
                )
                .is_ok(),
                "{url}"
            );
            let (_, input) = detached(&format!("@import {url} layer(theme);"));
            assert!(
                CssImportRule::try_from_components(input, &CssNamespaceContext::default()).is_ok(),
                "{url}"
            );
            for (kind, prefix) in COUNTERS {
                let (_, input) = detached(&format!("{prefix}{url}"));
                assert!(
                    CssCounterStyleDescriptorValue::try_from_components(kind, input).is_ok(),
                    "{url}: {kind:?}"
                );
            }
        }
    }
}

#[test]
fn detached_property_rejections_retain_the_contextual_modifier_origin() {
    for function in ["url", "src"] {
        for (modifier, _, opening) in INVALID {
            let url = format!("{function}(\"#asset\" {modifier})");
            let (source, input) = detached(&url);
            let error =
                checked_property(CssKnownProperty::BackgroundImage, input.clone()).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            serialized_argument(error.origin(), &input, &source, opening);
        }
    }
}

#[test]
fn detached_profile_rejections_return_typed_errors_with_original_argument_origins() {
    for function in ["url", "src"] {
        for (modifier, argument, _) in INVALID {
            let (source, input) = detached(&format!("{function}(\"#asset\" {modifier})"));
            let error = CssColorProfileDescriptorValue::try_new(
                CssColorProfileDescriptorKind::Src,
                input.clone(),
            )
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssColorProfileValueErrorKind::Grammar(_)
            ));
            serialized_argument(error.origin(), &input, &source, argument);
        }
    }
}

#[test]
fn detached_font_rejections_retain_the_contextual_modifier_origin() {
    for function in ["url", "src"] {
        for (modifier, _, opening) in INVALID {
            let (source, input) = detached(&format!("{function}(\"#asset\" {modifier})"));
            let error = CssAuthoredFontFaceDescriptorValue::try_from_components(
                CssFontFaceDescriptorKind::Src,
                input.clone(),
            )
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssFontFaceValueErrorKind::Grammar(_)
            ));
            serialized_argument(error.origin(), &input, &source, opening);
        }
    }
}

#[test]
fn detached_import_rejections_retain_the_target_grammar_origin() {
    for function in ["url", "src"] {
        for (modifier, _, _) in INVALID {
            let (source, input) = detached(&format!("@import {function}(\"#asset\" {modifier});"));
            let error =
                CssImportRule::try_from_components(input.clone(), &CssNamespaceContext::default())
                    .unwrap_err();
            assert!(matches!(
                &error,
                CssImportConstructionError::InvalidRuleGrammar { .. }
            ));
            original_argument(error.origin(), &input, &source, &format!("{function}("));
        }
    }
}

#[test]
fn detached_counter_rejections_return_typed_errors_with_original_argument_origins() {
    for function in ["url", "src"] {
        for (modifier, argument, _) in INVALID {
            for (kind, prefix) in COUNTERS {
                let (source, input) =
                    detached(&format!("{prefix}{function}(\"#asset\" {modifier})"));
                let error =
                    CssCounterStyleDescriptorValue::try_from_components(kind, input.clone())
                        .unwrap_err();
                assert!(matches!(
                    error.kind(),
                    CssCounterStyleValueErrorKind::Grammar(_)
                ));
                serialized_argument(error.origin(), &input, &source, argument);
            }
        }
    }
}

#[test]
fn detached_property_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let authored = checked_property(
        CssKnownProperty::BackgroundImage,
        parse_component_values("src(var(--asset))").unwrap(),
    )
    .unwrap();
    let CssExpansion::Pending(pending) = expand_declaration(&authored).unwrap() else {
        panic!("pending property")
    };
    for (modifier, _, opening) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending.reenter(input.clone()).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
            panic!("typed rejected replacement: {error:?}")
        };
        serialized_argument(error.origin(), &input, &source, opening);
        assert!(pending.source().same_occurrence(&authored));
    }
    let (_, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    assert!(pending.reenter(valid).is_ok());
}

#[test]
fn detached_profile_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let pending = CssColorProfileDescriptorValue::try_new(
        CssColorProfileDescriptorKind::Src,
        parse_component_values("env(profile)").unwrap(),
    )
    .unwrap();
    for (modifier, argument, _) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssColorProfileValueErrorKind::Grammar(_)
        ));
        serialized_argument(error.origin(), &input, &source, argument);
        assert!(matches!(
            pending.view(),
            CssColorProfileDescriptorValueRef::Pending(_)
        ));
    }
    let (_, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    assert!(pending.reparse_after_substitution(valid).is_ok());
}

#[test]
fn detached_font_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let authored = CssAuthoredFontFaceDescriptorValue::try_from_components(
        CssFontFaceDescriptorKind::Src,
        parse_component_values("env(font)").unwrap(),
    )
    .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = authored else {
        panic!("pending font source")
    };
    for (modifier, _, opening) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssFontFaceValueErrorKind::Grammar(_)
        ));
        serialized_argument(error.origin(), &input, &source, opening);
    }
    let (_, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    assert!(pending.reparse_after_substitution(valid).is_ok());
}

#[test]
fn detached_counter_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let pending = CssCounterStyleDescriptorValue::try_from_components(
        CssCounterStyleDescriptorKind::Symbols,
        parse_component_values("env(symbols)").unwrap(),
    )
    .unwrap();
    for (modifier, argument, _) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssCounterStyleValueErrorKind::Grammar(_)
        ));
        serialized_argument(error.origin(), &input, &source, argument);
        assert!(matches!(
            pending.view(),
            CssCounterStyleDescriptorValueRef::Pending(_)
        ));
    }
    let (_, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    assert!(pending.reparse_after_substitution(valid).is_ok());
}
