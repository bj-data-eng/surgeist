#![forbid(unsafe_code)]

//! Independent authored expectations: Fonts 4 WD 2026-09-07 §§2.2 and 9.1,
//! references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-prop
//! Frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d supplies the
//! palette-mix-valid/invalid parsing cases and omitted-method Oklab behavior.
//! These tests use existing public APIs. New payload construction and canonical
//! specified-output APIs require functional tests alongside implementation.
//! Palette lookup, computed simplification and mixing are downstream work.

use surgeist_css::*;

fn declaration(value: &str) -> CssDeclaration {
    let text = format!("font-palette:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one retained palette declaration: {text}")
    };
    assert_eq!(
        value.known().unwrap().property().canonical_name(),
        "font-palette"
    );
    value.clone()
}

fn grammar() -> CssPropertyGrammar {
    // Obtain the grammar only after exercising real property admission.
    declaration("normal").known().unwrap().grammar()
}

fn checked(value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked font-palette:{value}: {error:?}"))
}

fn longhands(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary or global palette contribution")
    };
    let [item] = values.items() else {
        panic!("one palette longhand")
    };
    assert_eq!(item.property().canonical_name(), "font-palette");
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    values
}

fn assert_admitted(value: &str) {
    let parsed = declaration(value);
    let supplied = parse_component_values(value).unwrap();
    let constructed = checked(value);
    assert!(parsed.known().unwrap().property_value().is_some());
    assert!(constructed.known().unwrap().property_value().is_some());
    assert_eq!(constructed.value_components(), &supplied);
    assert_eq!(constructed.importance(), CssImportance::Important);
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        value
    );
    assert!(longhands(&parsed).items()[0].ordinary_value().is_some());
    assert!(
        longhands(&constructed).items()[0]
            .ordinary_value()
            .is_some()
    );
    assert!(validate_style_attribute(&format!("font-palette:{value}")).is_ok());
}

#[test]
fn palette_property_is_retained_between_existing_declarations() {
    let report = parse_style_attribute("color:red;font-palette:normal;color:blue");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3);
    assert_eq!(
        report.syntax()[1]
            .known()
            .unwrap()
            .property()
            .canonical_name(),
        "font-palette"
    );
}

#[test]
fn keywords_and_dashed_names_admit_parsed_and_checked_values() {
    for value in [
        "normal",
        "LIGHT",
        "DaRk",
        "--theme",
        "--",
        "--日本",
        r"--\54 heme",
    ] {
        assert_admitted(value);
    }
}

#[test]
fn palette_names_preserve_decoded_case_and_escape_equivalence() {
    for (text, decoded) in [
        ("--Theme", "--Theme"),
        (r"--\54 heme", "--Theme"),
        ("--theme", "--theme"),
    ] {
        let source = declaration(text);
        let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) =
            source.value_components().items()[0].view()
        else {
            panic!("decoded palette identifier")
        };
        assert_eq!(name, decoded);
        assert_eq!(CssFontPaletteName::try_new(name).unwrap().as_str(), decoded);
    }
    assert_ne!(
        CssFontPaletteName::try_new("--Theme").unwrap(),
        CssFontPaletteName::try_new("--theme").unwrap()
    );
}

#[test]
fn parsed_name_value_and_token_origins_retain_exact_authored_ranges() {
    let value = r"--\54 heme";
    let text = format!("FoNt-PaLeTtE:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let source = &report.syntax()[0];
    let name = source.parsed_name().unwrap();
    assert_eq!(name.span().start().byte_offset().value(), 0);
    assert_eq!(name.span().end().byte_offset().value(), 12);
    assert_eq!(name.source().as_str(), text);
    let origin = source.parsed_value().unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), 13);
    assert_eq!(origin.span().end().byte_offset().value(), 13 + value.len());
    let CssValueOrigin::Parsed(token) = source.value_components().items()[0].origin() else {
        panic!("original palette name token")
    };
    assert_eq!(token.span(), origin.span());
    assert_eq!(token.source().as_str(), text);
    let before = source.clone();
    longhands(source);
    assert_eq!(source, &before);
    assert!(source.same_occurrence(&before));
}

#[test]
fn mixes_admit_optional_interpolation_and_one_or_more_recursive_operands() {
    for value in [
        "palette-mix(dark)",
        "palette-mix(in srgb, dark 100%)",
        "palette-mix(in srgb, dark 0%)",
        "palette-mix(light, dark)",
        "palette-mix(in oklab, light 30%, dark)",
        "palette-mix(dark, light, --theme)",
        "palette-mix(dark 50%, light, --theme)",
        "palette-mix(dark 75%, light 75%, --theme)",
        "palette-mix(dark 0%, light 0%, --theme 0%)",
        "palette-mix(palette-mix(in srgb, light 30%, normal) 20%, dark)",
        "palette-mix(palette-mix(dark), palette-mix(light, --theme), normal)",
    ] {
        assert_admitted(value);
    }
}

#[test]
fn interpolation_reuses_rectangular_polar_and_symbolic_profile_grammar() {
    for space in [
        "srgb",
        "srgb-linear",
        "display-p3",
        "display-p3-linear",
        "a98-rgb",
        "prophoto-rgb",
        "rec2020",
        "lab",
        "xyz",
        "xyz-d50",
        "xyz-d65",
        "oklab",
        "hsl",
        "hwb",
        "lch",
        "oklch",
        "--Profile",
    ] {
        assert_admitted(&format!("palette-mix(in {space}, light 10%, dark)"));
    }
    for space in ["hsl", "hwb", "lch", "oklch"] {
        for hue in ["shorter", "longer", "increasing", "decreasing"] {
            assert_admitted(&format!("palette-mix(in {space} {hue} hue, light, dark)"));
        }
    }
}

#[test]
fn percentage_before_or_after_each_operand_is_admitted_without_reordering_source() {
    for (left, right) in [
        (
            "palette-mix(30% light, dark)",
            "palette-mix(light 30%, dark)",
        ),
        (
            "palette-mix(30% --theme, normal)",
            "palette-mix(--theme 30%, normal)",
        ),
        (
            "palette-mix(20% palette-mix(light, dark), --theme)",
            "palette-mix(palette-mix(light, dark) 20%, --theme)",
        ),
    ] {
        assert_admitted(left);
        assert_admitted(right);
    }
    for value in [
        "palette-mix(light 30%, dark 70%)",
        "palette-mix(dark 70%, light 30%)",
    ] {
        assert_admitted(value);
    }
}

#[test]
fn literal_endpoints_and_symbolic_percentage_math_remain_authored() {
    for value in [
        "palette-mix(light 0%, dark 100%)",
        "palette-mix(light 1e2%, dark 0%)",
        "palette-mix(light .125%, dark 99.875%)",
        "palette-mix(light calc(10%), dark)",
        "palette-mix(light calc(-10%), dark)",
        "palette-mix(light calc(150%), dark)",
        "palette-mix(calc(100% - 20%) light, dark 20%)",
        "palette-mix(light min(10%, 20%), dark max(30%, 40%))",
        "palette-mix(light clamp(0%, 30%, 100%), dark)",
    ] {
        assert_admitted(value);
    }
}

#[test]
fn invalid_values_drop_only_the_palette_declaration_and_report_known_grammar_failure() {
    let owning_grammar = grammar();
    for value in [
        "none",
        "A",
        "normal light",
        "none, light",
        "1",
        "\"--theme\"",
        "palette-mix()",
        "palette-mix(in oklab,)",
        "palette-mix(, dark)",
        "palette-mix(dark,)",
        "palette-mix(dark,,light)",
        "palette-mix(dark light)",
        "palette-mix(oklab, dark)",
        "palette-mix(in oklab dark, light)",
        "palette-mix(dark, in oklab)",
        "palette-mix(in oklch hue, dark, light)",
        "palette-mix(in oklch shorter, dark, light)",
        "palette-mix(in oklab longer hue, dark, light)",
        "palette-mix(dark -10%)",
        "palette-mix(dark 150%, light)",
        "palette-mix(dark 20% 30%, light)",
        "palette-mix(20% 30% dark, light)",
        "palette-mix(dark 1, light)",
        "palette-mix(dark calc(1px), light)",
        "palette-mix(dark, palette-mix(none))",
        "var(theme)",
        "env()",
    ] {
        let text = format!("color:red;font-palette:{value};color:blue");
        let report = parse_style_attribute(&text);
        assert!(!report.is_clean(), "{value}");
        assert_eq!(report.syntax().len(), 2, "{value}");
        assert!(
            report
                .syntax()
                .iter()
                .all(|source| source.known().unwrap().property() == CssKnownProperty::Color)
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one local grammar error: {value}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        if value == "none" {
            assert_eq!(diagnostic.error().position().byte_offset().value(), 22);
        }
        assert!(validate_style_attribute(&text).is_err());
        let error = parse_property_value_for_grammar(
            owning_grammar,
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
            "{value}"
        );
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Parsed(_)) | CssSerializedOrigin::End(_)
        ));
    }
}

#[test]
fn checked_failures_map_to_supplied_tokens_and_reject_declaration_annotations() {
    let error = parse_property_value_for_grammar(
        grammar(),
        parse_component_values("none").unwrap(),
        CssImportance::Normal,
    )
    .unwrap_err();
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
        panic!("invalid keyword token origin")
    };
    assert_eq!(origin.source().as_str(), "none");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 4);
    for value in [
        "normal!important",
        "dark;",
        "normal {}",
        "{} var(--palette)",
    ] {
        assert!(matches!(
            parse_property_value_for_grammar(
                grammar(),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .unwrap_err()
            .kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
}

#[test]
fn palette_is_one_inherited_longhand_with_intrinsic_normal_initial() {
    let metadata = grammar().metadata().unwrap();
    let CssPropertyKindRef::Longhand(value) = metadata.kind() else {
        panic!("palette longhand")
    };
    assert!(value.inherited_by_default());
    let initial = value.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("intrinsic normal initial")
    };
    assert_eq!(
        longhands(&declaration("normal")).items()[0].ordinary_value(),
        Some(initial)
    );
}

#[test]
fn css_wide_values_keep_the_single_contribution_and_original_importance() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for source in [declaration(text), checked(text)] {
            assert_eq!(
                longhands(&source).items()[0].value(),
                CssContributionValueRef::Global(keyword)
            );
        }
    }
}

#[test]
fn whole_pending_values_reenter_strictly_without_changing_source_or_replacement_origins() {
    for symbolic in [
        "var(--palette)",
        "env(palette)",
        "var(--palette, {fallback})",
        "palette-mix(var(--palette), dark)",
    ] {
        for source in [declaration(symbolic), checked(symbolic)] {
            let before = source.clone();
            assert!(matches!(
                source.known().unwrap().declared_value(),
                CssKnownDeclaredValueRef::SubstitutionDependent(_)
            ));
            let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
                panic!("whole pending palette")
            };
            assert!(pending.source().same_occurrence(&source));
            for text in [
                "--Theme",
                "palette-mix(in oklch longer hue, light 30%, dark, --theme)",
            ] {
                let replacement = parse_component_values(text).unwrap();
                let CssContributions::Longhands(values) =
                    pending.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one completed palette")
                };
                let [item] = values.items() else {
                    panic!("one longhand")
                };
                assert_eq!(item.property().canonical_name(), "font-palette");
                assert!(item.ordinary_value().is_some());
                assert_eq!(item.replacement_components(), Some(&replacement));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
            let CssContributions::Longhands(values) = pending
                .reenter(parse_component_values("initial").unwrap())
                .unwrap()
            else {
                panic!("global replacement")
            };
            assert_eq!(values.items().len(), 1);
            assert_eq!(
                values.items()[0].value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Initial)
            );
            for invalid in ["none", "palette-mix(dark,)", "dark!important", "dark;"] {
                assert!(matches!(
                    pending
                        .reenter(parse_component_values(invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            for residual in [
                "var(--again)",
                "env(again)",
                "palette-mix(var(--again), dark)",
            ] {
                assert_eq!(
                    pending
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            }
            assert_eq!(source, before);
            assert!(source.same_occurrence(&before));
        }
    }
}

#[test]
fn normalization_retains_palette_order_occurrence_and_importance() {
    let text = ".a{font-palette:--Theme!important;color:red;font-palette:dark;font-palette:var(--p)!important}";
    let report = parse_sheet(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 4);
    for (index, (item, (name, authored, importance))) in values
        .iter()
        .zip([
            ("font-palette", "--Theme", CssImportance::Important),
            ("color", "red", CssImportance::Normal),
            ("font-palette", "dark", CssImportance::Normal),
            ("font-palette", "var(--p)", CssImportance::Important),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            name
        );
        assert_eq!(item.source().importance(), importance);
        let origin = item.source().parsed_value().unwrap();
        let offset = text.find(&format!("{name}:{authored}")).unwrap() + name.len() + 1;
        assert_eq!(origin.span().start().byte_offset().value(), offset);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            offset + authored.len()
        );
        if name == "font-palette" && authored == "var(--p)" {
            assert!(matches!(item.expansion(), CssExpansion::Pending(_)));
        } else if name == "font-palette" {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("palette contribution")
            };
            assert_eq!(values.items().len(), 1);
            assert!(values.items()[0].source().same_occurrence(item.source()));
        }
    }
}

#[test]
fn font_shorthand_neither_sets_nor_resets_palette() {
    for font in [
        "italic 16px Demo",
        "menu",
        "initial",
        "inherit",
        "var(--font)",
    ] {
        let text = format!("font-palette:--Theme!important;font:{font};font-palette:dark");
        let report = parse_style_attribute(&text);
        assert!(report.is_clean(), "{font}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 3);
        let metadata = report.syntax()[1]
            .known()
            .unwrap()
            .grammar()
            .metadata()
            .unwrap();
        let CssPropertyKindRef::Shorthand(font_metadata) = metadata.kind() else {
            panic!("font shorthand")
        };
        assert!(
            font_metadata
                .settable_members()
                .iter()
                .chain(font_metadata.reset_only_members())
                .all(|member| member.known_property().canonical_name() != "font-palette")
        );
        let normalized = normalize_sheet(parse_sheet(&format!(".a{{{text}}}")).syntax()).unwrap();
        let declarations = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(declarations.len(), 3);
        assert_eq!(
            declarations[0].source().importance(),
            CssImportance::Important
        );
        assert_eq!(declarations[2].source().importance(), CssImportance::Normal);
        for item in [declarations[0], declarations[2]] {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("independent palette")
            };
            assert_eq!(values.items().len(), 1);
            assert_eq!(
                values.items()[0].property().canonical_name(),
                "font-palette"
            );
        }
    }
}

#[test]
fn historical_palette_rule_and_font_parsing_remain_controls() {
    let source = "@font-palette-values --Theme{font-family:Demo;base-palette:light;override-colors:0 red} .a{font:italic 16px Demo;color:blue}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontPaletteValues(palette), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("palette definition followed by style")
    };
    assert_eq!(palette.name().as_str(), "--Theme");
    assert_eq!(palette.descriptors().len(), 3);
    assert_eq!(
        palette.descriptors()[1].value().to_specified_css().unwrap(),
        "light"
    );
    for font in ["italic 16px Demo", "menu", "initial", "var(--font)"] {
        let report = parse_style_attribute(&format!("font:{font};color:red"));
        assert!(report.is_clean(), "{font}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Font
        );
    }
}

#[test]
fn shared_component_limits_preserve_exact_mix_source_and_fail_at_each_boundary() {
    // One function + one identifier, depth one, seventeen source bytes.
    let text = "palette-mix(dark)";
    let exact = CssComponentValueLimits::try_new(1, 2, 17).unwrap();
    let value = parse_component_values_with_limits(text, exact).unwrap();
    assert_eq!(value.serialize().unwrap().as_css(), text);
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, 2, 17).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 1, 17).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 2, 16).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            parse_component_values_with_limits(text, limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(value.serialize().unwrap().as_css(), text);
}
