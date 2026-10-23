//! Existing public checked fronts must preserve their responsible token origin
//! when Values 5 WD 2024-11-11 §4.1.1 rejects a known request modifier.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#request-url-modifiers
//! Detaching components does not make their retained source coordinates into
//! coordinates in the shorter serialization used by checked grammar admission.

use crate::css::support::value_assertions::url_consumer_text as property_text;
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
    let detached =
        CssComponentValues::try_new(parsed.items()[1..].to_vec(), Default::default()).unwrap();
    assert_eq!(
        detached
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        text
    );
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

/// The valid retry `src("#asset" integrity("hash"))` is admitted as written:
/// Values 5 §4.1.1 gives `src()` one `<string>` target and the recognized
/// `integrity()` modifier one `<string>` argument, kept in authored order. Its
/// argument keeps the origin of the detached replacement it came from.
fn assert_retried_url(url: &CssUrl, source: &str, input: &CssComponentValues) {
    assert_eq!(url.function(), CssUrlFunction::Src);
    assert_eq!(url.as_str(), "#asset");
    let [CssUrlModifier::Function(modifier)] = url.modifiers() else {
        panic!("one functional request modifier: {:?}", url.modifiers());
    };
    assert_eq!(modifier.name(), "integrity");
    assert_eq!(
        url.serialize_specified(Default::default()).unwrap(),
        "src(\"#asset\" integrity(\"hash\"))"
    );
    let [argument] = modifier.argument_components().items() else {
        panic!("one integrity argument");
    };
    original_argument(argument.origin(), input, source, "\"hash\"");
}

/// A valid control `function("#asset" modifier)` is admitted as written:
/// Values 5 §4.1.1 keeps the target string and the one recognized functional
/// request modifier, named by the text before its opening parenthesis.
fn assert_admitted_url(url: &CssUrl, function: &str, modifier: &str) {
    let expected = match function {
        "url" => CssUrlFunction::Url,
        "src" => CssUrlFunction::Src,
        other => panic!("URL function {other}"),
    };
    assert_eq!(url.function(), expected, "{function}({modifier})");
    assert_eq!(url.as_str(), "#asset");
    let [CssUrlModifier::Function(admitted)] = url.modifiers() else {
        panic!("one functional request modifier: {:?}", url.modifiers());
    };
    assert_eq!(admitted.name(), modifier.split_once('(').unwrap().0);
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
                let admitted = checked_property(property, input.clone())
                    .unwrap_or_else(|error| panic!("{url}: {property:?}: {error:?}"));
                assert_eq!(admitted.known().unwrap().property(), property, "{url}");
                assert_eq!(admitted.value_components(), &input, "{url}: {property:?}");
                assert_eq!(admitted.importance(), CssImportance::Normal);
            }
            let (_, input) = detached(&url);
            let profile = CssColorProfileDescriptorValue::try_new(
                CssColorProfileDescriptorKind::Src,
                input,
                Default::default(),
            )
            .unwrap();
            let CssColorProfileDescriptorValueRef::Src(admitted) = profile.view() else {
                panic!("ordinary profile source")
            };
            assert!(admitted.is_local_url());
            assert_admitted_url(admitted, function, modifier);
        }
    }
}

#[test]
fn valid_detached_font_import_and_counter_controls_are_admitted() {
    for function in ["url", "src"] {
        for modifier in VALID {
            let url = format!("{function}(\"#asset\" {modifier})");
            let (_, input) = detached(&url);
            let font = CssAuthoredFontFaceDescriptorValue::try_from_components(
                CssFontFaceDescriptorKind::Src,
                input,
                Default::default(),
            )
            .unwrap_or_else(|error| panic!("{url}: {error:?}"));
            let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(
                sources,
            )) = font
            else {
                panic!("ordinary font source list: {url}");
            };
            let [CssFontFaceSource::Url(font_source)] = sources.sources() else {
                panic!("one URL font source: {:?}", sources.sources());
            };
            assert_admitted_url(font_source.url(), function, modifier);
            let (_, input) = detached(&format!("@import {url} layer(theme);"));
            let import = CssImportRule::try_from_components(
                input,
                &CssNamespaceContext::default(),
                Default::default(),
            )
            .unwrap_or_else(|error| panic!("{url}: {error:?}"));
            let CssImportTarget::Url(target) = import.target() else {
                panic!("URL import target: {url}");
            };
            assert_admitted_url(target.url(), function, modifier);
            assert!(import.layer().is_some(), "{url}: retained layer(theme)");
            for (kind, prefix) in COUNTERS {
                let (_, input) = detached(&format!("{prefix}{url}"));
                let counter = CssCounterStyleDescriptorValue::try_from_components(
                    kind,
                    input.clone(),
                    Default::default(),
                )
                .unwrap_or_else(|error| panic!("{url}: {kind:?}: {error:?}"));
                assert_eq!(counter.kind(), kind);
                assert_eq!(counter.components(), &input, "{url}: {kind:?}");
                assert!(
                    !matches!(
                        counter.view(),
                        CssCounterStyleDescriptorValueRef::Pending(_)
                    ),
                    "{url}: {kind:?} is an ordinary descriptor value"
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
                Default::default(),
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
                Default::default(),
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
            let error = CssImportRule::try_from_components(
                input.clone(),
                &CssNamespaceContext::default(),
                Default::default(),
            )
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
                let error = CssCounterStyleDescriptorValue::try_from_components(
                    kind,
                    input.clone(),
                    Default::default(),
                )
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
    let (source, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    let CssContributions::Longhands(values) = pending.reenter(valid.clone()).unwrap() else {
        panic!("completed background-image contribution")
    };
    let [contribution] = values.items() else {
        panic!("one longhand contribution");
    };
    assert!(contribution.source().same_occurrence(&authored));
    assert_eq!(contribution.replacement_components(), Some(&valid));
    let CssLonghandValueRef::BackgroundImage(images) = contribution
        .ordinary_value()
        .expect("ordinary value")
        .view()
    else {
        panic!("background-image value");
    };
    let [CssImageValue::Url(url)] = images.images() else {
        panic!("one URL image: {:?}", images.images());
    };
    assert_retried_url(url, &source, &valid);
}

#[test]
fn detached_profile_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let pending = CssColorProfileDescriptorValue::try_new(
        CssColorProfileDescriptorKind::Src,
        parse_component_values("env(profile)").unwrap(),
        Default::default(),
    )
    .unwrap();
    for (modifier, argument, _) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone(), Default::default())
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
    let (source, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    let retried = pending
        .reparse_after_substitution(valid.clone(), Default::default())
        .unwrap();
    let CssColorProfileDescriptorValueRef::Src(url) = retried.view() else {
        panic!("ordinary profile source");
    };
    assert_retried_url(url, &source, &valid);
}

#[test]
fn detached_font_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let authored = CssAuthoredFontFaceDescriptorValue::try_from_components(
        CssFontFaceDescriptorKind::Src,
        parse_component_values("env(font)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = authored else {
        panic!("pending font source")
    };
    for (modifier, _, opening) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone(), Default::default())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssFontFaceValueErrorKind::Grammar(_)
        ));
        serialized_argument(error.origin(), &input, &source, opening);
    }
    let (source, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    let CssFontFaceDescriptorValue::Src(sources) = pending
        .reparse_after_substitution(valid.clone(), Default::default())
        .unwrap()
    else {
        panic!("ordinary font source list");
    };
    let [CssFontFaceSource::Url(font_source)] = sources.sources() else {
        panic!("one URL font source: {:?}", sources.sources());
    };
    assert_retried_url(font_source.url(), &source, &valid);
}

#[test]
fn detached_counter_reentry_rejects_with_original_origin_and_allows_valid_retry() {
    let pending = CssCounterStyleDescriptorValue::try_from_components(
        CssCounterStyleDescriptorKind::Symbols,
        parse_component_values("env(symbols)").unwrap(),
        Default::default(),
    )
    .unwrap();
    for (modifier, argument, _) in INVALID {
        let (source, input) = detached(&format!("src(\"#asset\" {modifier})"));
        let error = pending
            .reparse_after_substitution(input.clone(), Default::default())
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
    let (source, valid) = detached("src(\"#asset\" integrity(\"hash\"))");
    let retried = pending
        .reparse_after_substitution(valid.clone(), Default::default())
        .unwrap();
    let CssCounterStyleDescriptorValueRef::Symbols(symbols) = retried.view() else {
        panic!("ordinary counter symbols");
    };
    let [CssCounterSymbol::Image(image)] = symbols.symbols() else {
        panic!("one image symbol: {:?}", symbols.symbols());
    };
    let CssImageValue::Url(url) = image.value() else {
        panic!("URL image symbol");
    };
    assert_retried_url(url, &source, &valid);
}
