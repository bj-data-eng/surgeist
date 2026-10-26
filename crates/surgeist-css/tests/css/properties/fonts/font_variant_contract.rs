//! Public authored font-variant contracts from CSS Fonts 4 WD 2026-09-07 §§6.4–6.11.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::important_declaration as parsed;
use crate::css::support::named_property::assert_implicit_closure_rejection;
use crate::css::support::named_property::invalid_named_value_at as invalid;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::*;

const LONGHANDS: [&str; 7] = [
    "font-variant-ligatures",
    "font-variant-caps",
    "font-variant-alternates",
    "font-variant-numeric",
    "font-variant-east-asian",
    "font-variant-position",
    "font-variant-emoji",
];

fn name(value: &str) -> CssFontFeatureValueName {
    CssFontFeatureValueName::try_new(value).unwrap()
}

fn ordinary(name: &str, value: &str) -> CssLonghandValue {
    let source = parsed(name, value);
    let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
        source.expand().unwrap()
    else {
        panic!("{name} has terminal expansion")
    };
    let [item] = contributions.items() else {
        panic!("one longhand contribution: {name}")
    };
    item.ordinary_value().unwrap().clone()
}

#[test]
fn alternates_functions_preserve_decoded_names_and_each_distinct_choice() {
    let authored = concat!(
        "annotation(AnNo) ornaments(Orn) swash(Swash) ",
        "character-variant(Second, first, Second) styleset(normal, FOO, normal) ",
        "historical-forms stylistic(\\41 \\!)",
    );
    let declaration = parsed("font-variant-alternates", authored);
    let CssKnownPropertyValueRef::FontVariantAlternates(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed alternates wrapper")
    };
    let CssFontVariantAlternates::Values(values) = wrapper.value() else {
        panic!("seven alternate choices")
    };
    assert_eq!(values.stylistic().unwrap().as_str(), "A!");
    assert!(values.historical_forms());
    assert_eq!(
        values
            .styleset()
            .unwrap()
            .names()
            .iter()
            .map(CssFontFeatureValueName::as_str)
            .collect::<Vec<_>>(),
        ["normal", "FOO", "normal"]
    );
    assert_eq!(
        values
            .character_variant()
            .unwrap()
            .names()
            .iter()
            .map(CssFontFeatureValueName::as_str)
            .collect::<Vec<_>>(),
        ["Second", "first", "Second"]
    );
    assert_eq!(values.swash().unwrap().as_str(), "Swash");
    assert_eq!(values.ornaments().unwrap().as_str(), "Orn");
    assert_eq!(values.annotation().unwrap().as_str(), "AnNo");
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "stylistic(A\\!) historical-forms styleset(normal, FOO, normal) character-variant(Second, first, Second) swash(Swash) ornaments(Orn) annotation(AnNo)"
    );
    assert!(declaration.value_components().items().len() >= 7);
}

#[test]
fn alternate_models_reject_empty_states_and_inspect_borrowed_names() {
    assert!(CssFontVariantAlternateNames::try_new(vec![]).is_none());
    assert!(
        CssFontVariantAlternateValues::try_new(None, false, None, None, None, None, None).is_none()
    );
    let list =
        CssFontVariantAlternateNames::try_new(vec![name("normal"), name("ABC"), name("normal")])
            .unwrap();
    let values = CssFontVariantAlternateValues::try_new(
        Some(name("A!")),
        true,
        Some(list),
        None,
        Some(name("Swash")),
        None,
        None,
    )
    .unwrap();
    assert_eq!(values.stylistic().unwrap().as_str(), "A!");
    assert!(values.historical_forms());
    assert_eq!(
        values
            .styleset()
            .unwrap()
            .names()
            .iter()
            .map(CssFontFeatureValueName::as_str)
            .collect::<Vec<_>>(),
        ["normal", "ABC", "normal"]
    );
    assert!(values.character_variant().is_none());
    assert_eq!(values.swash().unwrap().as_str(), "Swash");
    assert!(values.ornaments().is_none());
    assert!(values.annotation().is_none());
    assert_ne!(name("ABC"), name("abc"));
    assert_ne!(
        CssFontVariantAlternateNames::try_new(vec![name("A"), name("B")]),
        CssFontVariantAlternateNames::try_new(vec![name("B"), name("A")]),
    );
    assert_ne!(
        CssFontVariantAlternateNames::try_new(vec![name("A"), name("A")]),
        CssFontVariantAlternateNames::try_new(vec![name("A")]),
    );
    let aggregate = CssFontVariantValues::try_new(
        None,
        None,
        None,
        None,
        None,
        Some(values.clone()),
        Some(CssFontVariantEmoji::Text),
    )
    .unwrap();
    assert_eq!(aggregate.alternates(), Some(&values));
    assert_eq!(aggregate.emoji(), Some(CssFontVariantEmoji::Text));
    assert!(aggregate.caps().is_none());
    assert!(aggregate.position().is_none());
    assert!(aggregate.ligatures().is_none());
    assert!(aggregate.numeric().is_none());
    assert!(aggregate.east_asian().is_none());
    assert_eq!(
        CssFontVariantValue::Values(aggregate)
            .serialize_specified(Default::default())
            .unwrap(),
        "stylistic(A\\!) historical-forms styleset(normal, ABC, normal) swash(Swash) text"
    );
}

#[test]
fn full_shorthand_preserves_groups_but_serializes_in_fonts_four_order() {
    let source = parsed(
        "font-variant",
        concat!(
            "emoji super ruby tabular-nums oldstyle-nums annotation(Ann) stylistic(NaMe) ",
            "historical-forms small-caps no-contextual"
        ),
    );
    let CssKnownPropertyValueRef::FontVariant(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("full shorthand wrapper")
    };
    let CssFontVariantValue::Values(values) = wrapper.value() else {
        panic!("combined shorthand")
    };
    assert_eq!(
        values.ligatures().unwrap().contextual(),
        Some(CssFontVariantLigatureState::Disabled)
    );
    assert_eq!(values.caps(), Some(CssFontVariantCaps::SmallCaps));
    assert_eq!(values.position(), Some(CssFontVariantPosition::Super));
    assert_eq!(
        values.numeric().unwrap().figure(),
        Some(CssFontVariantNumericFigure::OldstyleNums)
    );
    assert_eq!(
        values.numeric().unwrap().spacing(),
        Some(CssFontVariantNumericSpacing::TabularNums)
    );
    assert!(values.east_asian().unwrap().ruby());
    assert_eq!(
        values.alternates().unwrap().stylistic().unwrap().as_str(),
        "NaMe"
    );
    assert!(values.alternates().unwrap().historical_forms());
    assert_eq!(
        values.alternates().unwrap().annotation().unwrap().as_str(),
        "Ann"
    );
    assert_eq!(values.emoji(), Some(CssFontVariantEmoji::Emoji));
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "no-contextual small-caps stylistic(NaMe) historical-forms annotation(Ann) oldstyle-nums tabular-nums ruby super emoji"
    );
}

#[test]
fn longhands_and_shorthand_reject_duplicate_or_malformed_alternates() {
    // Fonts 4 font-variant-alternates: stylistic() and ornaments() take one
    // feature-value name, so the comma after it is responsible.
    for name in ["font-variant-alternates", "font-variant"] {
        invalid(name, "stylistic(A, B)", ", B)");
    }
    for name in ["font-variant-alternates", "font-variant"] {
        invalid(name, "ornaments(A, B)", ", B)");
    }
    for (value, rest) in [
        ("stylistic()", ")"),
        ("styleset()", ")"),
        ("styleset(A,)", ")"),
        ("styleset(,A)", ",A)"),
        ("styleset(A B)", "B)"),
        ("character-variant()", ")"),
        ("character-variant(A B)", "B)"),
        ("swash()", ")"),
        ("annotation(A B)", "B)"),
        ("historical-forms()", "historical-forms()"),
        ("unknown(A)", "unknown(A)"),
        ("stylistic(A) stylistic(B)", "stylistic(B)"),
        ("styleset(A) styleset(B)", "styleset(B)"),
        ("historical-forms historical-forms", "historical-forms"),
        ("normal stylistic(A)", "stylistic(A)"),
    ] {
        invalid("font-variant-alternates", value, rest);
        invalid("font-variant", value, rest);
    }
    for (value, rest) in [
        ("normal emoji", "emoji"),
        ("text unicode", "unicode"),
        ("emoji emoji", "emoji"),
        ("none text", "text"),
    ] {
        invalid("font-variant", value, rest);
    }
    for (value, rest) in [
        ("none", "none"),
        ("text emoji", "emoji"),
        ("normal text", "text"),
        ("emoji unicode", "unicode"),
    ] {
        invalid("font-variant-emoji", value, rest);
    }
    for value in ["normal", "text", "emoji", "unicode"] {
        let declaration = parsed("font-variant-emoji", value);
        let CssKnownPropertyValueRef::FontVariantEmoji(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("emoji wrapper")
        };
        let expected = match value {
            "normal" => CssFontVariantEmoji::Normal,
            "text" => CssFontVariantEmoji::Text,
            "emoji" => CssFontVariantEmoji::Emoji,
            "unicode" => CssFontVariantEmoji::Unicode,
            _ => unreachable!(),
        };
        assert_eq!(wrapper.value(), &expected);
        assert_eq!(
            wrapper
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            value
        );
    }
}

#[test]
fn checked_aggregate_rejects_absence_and_normal_subgroups() {
    assert!(CssFontVariantValues::try_new(None, None, None, None, None, None, None).is_none());
    assert!(
        CssFontVariantValues::try_new(
            None,
            Some(CssFontVariantPosition::Normal),
            None,
            None,
            None,
            None,
            None
        )
        .is_none()
    );
    assert!(
        CssFontVariantValues::try_new(
            None,
            None,
            Some(CssFontVariantCaps::Normal),
            None,
            None,
            None,
            None
        )
        .is_none()
    );
    assert!(
        CssFontVariantValues::try_new(
            None,
            None,
            None,
            None,
            None,
            None,
            Some(CssFontVariantEmoji::Normal)
        )
        .is_none()
    );
    assert!(CssFontVariantLigatureValues::try_new(None, None, None, None).is_none());
    assert!(CssFontVariantNumericValues::try_new(None, None, None, false, false).is_none());
    assert!(CssFontVariantEastAsianValues::try_new(None, None, false).is_none());
}

#[test]
fn specified_serialization_orders_all_seven_groups_and_fails_atomically_at_limits() {
    let source = parsed(
        "font-variant",
        "unicode super ruby full-width jis04 slashed-zero ordinal stacked-fractions tabular-nums oldstyle-nums annotation(Ann) ornaments(Orn) swash(Swa) character-variant(A, B) styleset(C, D) historical-forms stylistic(E) titling-caps no-contextual historical-ligatures discretionary-ligatures common-ligatures",
    );
    let CssKnownPropertyValueRef::FontVariant(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("shorthand wrapper")
    };
    let expected = "common-ligatures discretionary-ligatures historical-ligatures no-contextual titling-caps stylistic(E) historical-forms styleset(C, D) character-variant(A, B) swash(Swa) ornaments(Orn) annotation(Ann) oldstyle-nums tabular-nums stacked-fractions ordinal slashed-zero jis04 full-width ruby super unicode";
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1000, 1000),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1000, 0, 1000),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1000, 1000, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&wrapper.value().serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    for (name, value, expected) in [
        (
            "font-variant-ligatures",
            "no-contextual common-ligatures",
            "common-ligatures no-contextual",
        ),
        ("font-variant-caps", "SMALL-CAPS", "small-caps"),
        (
            "font-variant-alternates",
            "swash(NaMe) historical-forms",
            "historical-forms swash(NaMe)",
        ),
        (
            "font-variant-numeric",
            "ordinal oldstyle-nums",
            "oldstyle-nums ordinal",
        ),
        ("font-variant-east-asian", "ruby jis04", "jis04 ruby"),
        ("font-variant-position", "SUPER", "super"),
        ("font-variant-emoji", "UNICODE", "unicode"),
    ] {
        let declaration = parsed(name, value);
        let current = declaration.known().unwrap().property_value().unwrap();
        let output = match current {
            CssKnownPropertyValueRef::FontVariantLigatures(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantCaps(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantAlternates(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantNumeric(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantEastAsian(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantPosition(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            CssKnownPropertyValueRef::FontVariantEmoji(w) => {
                w.value().serialize_specified(Default::default()).unwrap()
            }
            _ => panic!("{name} typed wrapper"),
        };
        assert_eq!(output, expected, "{name}");
        let limited = CssSerializationLimits::new(1000, 1000, expected.len() - 1);
        let error = match current {
            CssKnownPropertyValueRef::FontVariantLigatures(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantCaps(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantAlternates(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantNumeric(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantEastAsian(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantPosition(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            CssKnownPropertyValueRef::FontVariantEmoji(w) => {
                w.value().serialize_specified(limited).unwrap_err()
            }
            _ => panic!("{name} typed wrapper"),
        };
        assert!(error.is_byte_limit(), "{name}");
    }
}

#[test]
fn expansion_has_exact_seven_inherited_targets_and_preserves_sources() {
    let shorthand = grammar("font-variant");
    let CssPropertyKindRef::Shorthand(metadata) = shorthand.metadata().unwrap().kind() else {
        panic!("shorthand metadata")
    };
    assert!(metadata.reset_only_members().is_empty());
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|property| property.known_property().canonical_name())
            .collect::<Vec<_>>(),
        LONGHANDS
    );
    for name in LONGHANDS {
        let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("longhand metadata: {name}")
        };
        assert!(metadata.inherited_by_default(), "{name}");
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("normal initial: {name}")
        };
        assert_eq!(initial, &ordinary(name, "normal"));
    }
    for (value, none) in [
        ("normal", false),
        ("none", true),
        ("text small-caps", false),
    ] {
        let source = parsed("font-variant", value);
        let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
            source.expand().unwrap()
        else {
            panic!("seven contributions")
        };
        assert_eq!(contributions.items().len(), 7);
        for (index, item) in contributions.items().iter().enumerate() {
            assert_eq!(item.property().canonical_name(), LONGHANDS[index]);
            let expected = if none && index == 0 {
                "none"
            } else if value == "text small-caps" && index == 1 {
                "small-caps"
            } else if value == "text small-caps" && index == 6 {
                "text"
            } else {
                "normal"
            };
            assert_eq!(
                item.ordinary_value(),
                Some(&ordinary(LONGHANDS[index], expected))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    for (value, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = parsed("font-variant", value);
        checked("font-variant", value);
        let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
            source.expand().unwrap()
        else {
            panic!("CSS-wide contributions")
        };
        assert_eq!(
            contributions
                .items()
                .iter()
                .map(|item| item.property().canonical_name())
                .collect::<Vec<_>>(),
            LONGHANDS
        );
        assert!(
            contributions
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(&source)
                    && item.source().importance() == CssImportance::Important)
        );
        assert!(
            contributions
                .items()
                .iter()
                .all(|item| item.value() == CssContributionValueRef::Global(keyword))
        );
    }
}

#[test]
fn pending_reentry_is_strict_and_keeps_occurrence_and_replacement() {
    for (property, replacement) in [
        ("font-variant", "stylistic(A) text"),
        ("font-variant-alternates", "styleset(A, A)"),
        ("font-variant-emoji", "unicode"),
    ] {
        for pending in ["var(--variant)", "env(--variant)"] {
            let source = parsed(property, pending);
            let CssExpansion::Pending(handle) = source.expand().unwrap() else {
                panic!("pending {property}")
            };
            assert!(handle.source().same_occurrence(&source));
            for residual in ["var(--again)", "env(--again)"] {
                assert!(
                    handle
                        .reenter(
                            CssComponentValues::parse(&CssParser::default(), residual).unwrap()
                        )
                        .unwrap_err()
                        .is_residual_substitution()
                );
            }
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "unknown(A)").unwrap()
                    )
                    .unwrap_err()
                    .replacement_error()
                    .is_some()
            );
            let components = CssComponentValues::parse(&CssParser::default(), replacement).unwrap();
            let CssContributions::Longhands(contributions) =
                handle.reenter(components.clone()).unwrap()
            else {
                panic!("terminal reentry")
            };
            assert_eq!(
                contributions.items().len(),
                if property == "font-variant" { 7 } else { 1 }
            );
            assert!(contributions.items().iter().all(
                |item| item.source().same_occurrence(&source)
                    && item.source().importance() == CssImportance::Important
                    && item.replacement_components() == Some(&components)
            ));
        }
    }
}

#[test]
fn recovered_eof_function_retains_diagnostic_but_strict_inputs_reject_it() {
    let source = "font-variant-alternates: stylistic(A";
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(report.syntax().len(), 1);
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::FontVariantAlternates(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("recovered alternates value")
    };
    let CssFontVariantAlternates::Values(values) = wrapper.value() else {
        panic!("recovered stylistic choice")
    };
    assert_eq!(values.stylistic().unwrap().as_str(), "A");
    assert!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .is_err()
    );
    let recovered_components =
        CssComponentValues::parse(&CssParser::default(), "stylistic(A").unwrap();
    assert_implicit_closure_rejection(
        &CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(grammar("font-variant-alternates")),
            recovered_components.clone(),
            CssImportance::Normal,
        )
        .unwrap_err(),
        "stylistic(A",
    );
    let pending = parsed("font-variant-alternates", "var(--alt)");
    let CssExpansion::Pending(handle) = pending.expand().unwrap() else {
        panic!("pending alternates")
    };
    assert!(
        handle
            .reenter(recovered_components)
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
}

#[test]
fn invalid_alternates_keep_parsed_and_programmatic_error_origins() {
    let source = "font-variant-alternates: stylistic(A) stylistic(B); color: red";
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one duplicate-function diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "property-mapped diagnostic"
    );
    assert_eq!(
        detail.known_property().unwrap(),
        CssKnownProperty::FontVariantAlternates
    );
    let duplicate = source.rfind("stylistic").unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        duplicate
    );

    let checked_error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Grammar(grammar("font-variant-alternates")),
        CssComponentValues::parse(&CssParser::default(), "stylistic(A) stylistic(B)").unwrap(),
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(checked_error.code() == Some(CssErrorCode::InvalidPropertyValue));
    assert!(matches!(
        checked_error.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Parsed(_))
    ));

    let programmatic = CssComponentValues::try_new(
        vec![CssComponentValue::try_ident("none").unwrap()],
        Default::default(),
    )
    .unwrap();
    let programmatic_error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Grammar(grammar("font-variant-emoji")),
        programmatic,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(programmatic_error.code() == Some(CssErrorCode::InvalidPropertyValue));
    assert_eq!(
        programmatic_error.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn reentry_keeps_programmatic_ident_token_boundaries() {
    let source = parsed("font-variant-alternates", "var(--alt)");
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("pending alternates")
    };
    let separate_idents = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_ident("historical-").unwrap(),
            CssComponentValue::try_ident("forms").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    assert!(
        handle
            .reenter(separate_idents)
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
    let CssContributions::Longhands(contributions) = handle
        .reenter(CssComponentValues::parse(&CssParser::default(), "historical-forms").unwrap())
        .unwrap()
    else {
        panic!("one valid historical-forms contribution")
    };
    assert_eq!(contributions.items().len(), 1);
    assert_eq!(
        contributions.items()[0].ordinary_value(),
        Some(&ordinary("font-variant-alternates", "historical-forms"))
    );
}

#[test]
fn normalization_keeps_distinct_font_variant_occurrences_in_order() {
    let report = CssParser::default().parse_sheet(
        ".a{font-variant:emoji; font-variant-alternates:stylistic(A); font-variant:normal}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (order, expected) in ["emoji", "stylistic(A)", "normal"].into_iter().enumerate() {
        let declaration = declarations[order];
        assert_eq!(declaration.order(), order);
        assert_eq!(
            declaration
                .source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            expected
        );
        let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
            declaration.expansion()
        else {
            panic!("normalized font variant declaration")
        };
        assert_eq!(contributions.items().len(), if order == 1 { 1 } else { 7 });
        assert!(
            contributions
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(declaration.source()))
        );
    }
}

#[test]
fn full_variant_does_not_expand_the_restricted_font_prefix() {
    parsed("font", "small-caps 16px Demo");
    parsed("font", "normal 16px Demo");
    invalid("font", "emoji 16px Demo", "emoji 16px Demo");
    invalid("font", "stylistic(A) 16px Demo", "stylistic(A) 16px Demo");
}
