#![forbid(unsafe_code)]

//! Public authored font-variant contracts from CSS Fonts 4 WD 2026-09-07 §§6.4–6.11.

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

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known grammar: {name}"))
}

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{name}:{value}: {error:?}"))
}

fn invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 2, "neighbors: {source}");
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::DropDeclaration
                && diagnostic.error().code() != CssErrorCode::UnknownProperty
        }),
        "invalid value should be diagnosed: {source}: {:?}",
        report.diagnostics()
    );
    assert!(validate_style_attribute(&source).is_err(), "{source}");
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
}

fn name(value: &str) -> CssFontFeatureValueName {
    CssFontFeatureValueName::try_new(value).unwrap()
}

fn ordinary(name: &str, value: &str) -> CssLonghandValue {
    let source = parsed(name, value);
    let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
        expand_declaration(&source).unwrap()
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
    let CssFontVariantAlternates::Values(values) = wrapper.alternates() else {
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
        wrapper.alternates().serialize_specified().unwrap(),
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
            .serialize_specified()
            .unwrap(),
        "stylistic(A\\!) historical-forms styleset(normal, ABC, normal) swash(Swash) text"
    );
}

#[test]
fn full_shorthand_preserves_groups_but_serializes_in_fonts_four_order() {
    let source = parsed(
        "font-variant",
        concat!(
            "emoji super ruby tabular-nums annotation(Ann) small-caps no-contextual ",
            "stylistic(NaMe) oldstyle-nums historical-forms"
        ),
    );
    let CssKnownPropertyValueRef::FontVariant(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("full shorthand wrapper")
    };
    let CssFontVariantValue::Values(values) = wrapper.variant() else {
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
        wrapper.variant().serialize_specified().unwrap(),
        "no-contextual small-caps stylistic(NaMe) historical-forms annotation(Ann) oldstyle-nums tabular-nums ruby super emoji"
    );
}

#[test]
fn longhands_and_shorthand_reject_duplicate_or_malformed_alternates() {
    for value in [
        "stylistic()",
        "stylistic(A, B)",
        "styleset()",
        "styleset(A,)",
        "styleset(,A)",
        "styleset(A B)",
        "character-variant()",
        "character-variant(A B)",
        "swash()",
        "ornaments(A, B)",
        "annotation(A B)",
        "historical-forms()",
        "unknown(A)",
        "stylistic(A) stylistic(B)",
        "styleset(A) styleset(B)",
        "historical-forms historical-forms",
        "normal stylistic(A)",
    ] {
        invalid("font-variant-alternates", value);
        invalid("font-variant", value);
    }
    for value in ["normal emoji", "text unicode", "emoji emoji", "none text"] {
        invalid("font-variant", value);
    }
    for value in ["none", "text emoji", "normal text", "emoji unicode"] {
        invalid("font-variant-emoji", value);
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
        assert_eq!(wrapper.emoji(), &expected);
        assert_eq!(wrapper.emoji().serialize_specified().unwrap(), value);
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
    assert_eq!(wrapper.variant().serialize_specified().unwrap(), expected);
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
            wrapper
                .variant()
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(wrapper.variant().serialize_specified().unwrap(), expected);
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
                w.ligatures().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontVariantCaps(w) => w.caps().serialize_specified().unwrap(),
            CssKnownPropertyValueRef::FontVariantAlternates(w) => {
                w.alternates().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontVariantNumeric(w) => {
                w.numeric().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontVariantEastAsian(w) => {
                w.east_asian().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontVariantPosition(w) => {
                w.position().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontVariantEmoji(w) => {
                w.emoji().serialize_specified().unwrap()
            }
            _ => panic!("{name} typed wrapper"),
        };
        assert_eq!(output, expected, "{name}");
        let limited = CssSpecifiedValueSerializationLimits::new(1000, 1000, expected.len() - 1);
        let error = match current {
            CssKnownPropertyValueRef::FontVariantLigatures(w) => w
                .ligatures()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantCaps(w) => w
                .caps()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantAlternates(w) => w
                .alternates()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantNumeric(w) => w
                .numeric()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantEastAsian(w) => w
                .east_asian()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantPosition(w) => w
                .position()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            CssKnownPropertyValueRef::FontVariantEmoji(w) => w
                .emoji()
                .serialize_specified_with_limits(limited)
                .unwrap_err(),
            _ => panic!("{name} typed wrapper"),
        };
        assert_eq!(
            error.kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
            "{name}"
        );
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
            expand_declaration(&source).unwrap()
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
            expand_declaration(&source).unwrap()
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
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending {property}")
            };
            assert!(handle.source().same_occurrence(&source));
            for residual in ["var(--again)", "env(--again)"] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            assert!(matches!(
                handle
                    .reenter(parse_component_values("unknown(A)").unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            let components = parse_component_values(replacement).unwrap();
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
    let report = parse_style_attribute(source);
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
    let CssFontVariantAlternates::Values(values) = wrapper.alternates() else {
        panic!("recovered stylistic choice")
    };
    assert_eq!(values.stylistic().unwrap().as_str(), "A");
    assert!(validate_style_attribute(source).is_err());
    let recovered_components = parse_component_values("stylistic(A").unwrap();
    assert!(
        parse_property_value_for_grammar(
            grammar("font-variant-alternates"),
            recovered_components.clone(),
            CssImportance::Normal,
        )
        .is_err()
    );
    let pending = parsed("font-variant-alternates", "var(--alt)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending alternates")
    };
    assert!(matches!(
        handle.reenter(recovered_components).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
}

#[test]
fn invalid_alternates_keep_parsed_and_programmatic_error_origins() {
    let source = "font-variant-alternates: stylistic(A) stylistic(B); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one duplicate-function diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property-mapped diagnostic")
    };
    assert_eq!(detail.property(), CssKnownProperty::FontVariantAlternates);
    let duplicate = source.rfind("stylistic").unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        duplicate
    );

    let checked_error = parse_property_value_for_grammar(
        grammar("font-variant-alternates"),
        parse_component_values("stylistic(A) stylistic(B)").unwrap(),
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(matches!(
        checked_error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(_))
    ));
    assert!(matches!(
        checked_error.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Parsed(_))
    ));

    let programmatic =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("none").unwrap()]).unwrap();
    let programmatic_error = parse_property_value_for_grammar(
        grammar("font-variant-emoji"),
        programmatic,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(matches!(
        programmatic_error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(_))
    ));
    assert_eq!(
        programmatic_error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn reentry_keeps_programmatic_ident_token_boundaries() {
    let source = parsed("font-variant-alternates", "var(--alt)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending alternates")
    };
    let separate_idents = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("historical-").unwrap(),
        CssComponentValue::try_ident("forms").unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        handle.reenter(separate_idents).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let CssContributions::Longhands(contributions) = handle
        .reenter(parse_component_values("historical-forms").unwrap())
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
    let report = parse_sheet(
        ".a{font-variant:emoji; font-variant-alternates:stylistic(A); font-variant:normal}",
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
    assert_eq!(declarations.len(), 3);
    for (order, expected) in ["emoji", "stylistic(A)", "normal"].into_iter().enumerate() {
        let declaration = declarations[order];
        assert_eq!(declaration.order(), order);
        assert_eq!(
            declaration
                .source()
                .value_components()
                .serialize()
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
    invalid("font", "emoji 16px Demo");
    invalid("font", "stylistic(A) 16px Demo");
}
