#![forbid(unsafe_code)]

//! CSS Grid 2 (2025-03-26) §§8.3–8.4 placement grammar and intrinsic expansion;
//! CSS Values 4 (2024-03-12) §4.2 identifier exclusions and math range timing.
//! Grid 3 (2026-01-21) §4.1 reuses these properties on its grid axis.

use surgeist_css::*;

const LONGHANDS: [&str; 4] = [
    "grid-row-start",
    "grid-row-end",
    "grid-column-start",
    "grid-column-end",
];

const SHORTHANDS: [(&str, &[&str]); 3] = [
    ("grid-row", &["grid-row-start", "grid-row-end"]),
    ("grid-column", &["grid-column-start", "grid-column-end"]),
    (
        "grid-area",
        &[
            "grid-row-start",
            "grid-column-start",
            "grid-row-end",
            "grid-column-end",
        ],
    ),
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known grid property: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}: {value} !important");
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
    .unwrap_or_else(|error| panic!("checked {name}: {value}: {error:?}"))
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("grid placement has intrinsic expansion")
    else {
        panic!("completed grid placement longhands")
    };
    values
}

fn assert_members(source: &CssDeclaration, expected: &[(&str, &str)]) {
    let values = expanded(source);
    assert_eq!(values.items().len(), expected.len());
    for (item, &(name, value)) in values.items().iter().zip(expected) {
        assert_eq!(item.property().canonical_name(), name);
        assert!(item.ordinary_value().is_some());
        // Exact integer payloads can retain original token origins; compare ordinary
        // values only where the independent reference uses origin-free components.
        if !value.contains("calc(") && !value.contains("214748364") {
            let expected_longhand = expanded(&declaration(name, value));
            assert_eq!(
                item.ordinary_value(),
                expected_longhand.items()[0].ordinary_value(),
                "{name} from {:?}",
                source.value_components()
            );
        }
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn seven_placement_properties_report_complete_authored_support() {
    for name in LONGHANDS
        .into_iter()
        .chain(SHORTHANDS.map(|(name, _)| name))
    {
        let support = property_support_metadata(name).expect("known placement property");
        assert_eq!(support.property(), grammar(name).target_property());
        assert_eq!(
            support.feature().status(),
            CssSupportStatus::Complete,
            "{name}"
        );
    }
}

#[test]
fn four_placement_longhands_have_auto_initials_and_are_not_inherited() {
    for name in LONGHANDS {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has fixed auto initial")
        };
        let auto = expanded(&declaration(name, "auto"));
        assert_eq!(Some(initial), auto.items()[0].ordinary_value());
    }
}

#[test]
fn placement_shorthands_advertise_only_their_ordered_members() {
    for (name, members) in SHORTHANDS {
        let CssPropertyKindRef::Shorthand(shorthand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a shorthand")
        };
        assert!(!shorthand.is_legacy());
        assert!(shorthand.reset_only_members().is_empty());
        assert_eq!(
            shorthand
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
    }
}

#[test]
fn named_signed_lines_admit_both_orders_and_exact_large_literals() {
    for name in LONGHANDS {
        for value in [
            "main 2",
            "2 main",
            "main -3",
            "-3 main",
            "2147483648",
            "-2147483649",
            "main 2147483648",
            "-2147483649 main",
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_members(&source, &[(name, value)]);
            }
        }
    }
}

#[test]
fn spans_admit_all_orders_of_keyword_count_and_name() {
    for name in LONGHANDS {
        for value in [
            "span 2 main",
            "span main 2",
            "2 span main",
            "2 main span",
            "main span 2",
            "main 2 span",
            "2 SpAn",
            "span 2147483648 main",
            "span main",
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_members(&source, &[(name, value)]);
            }
        }
    }
}

#[test]
fn integer_root_math_remains_authored_even_when_its_result_is_out_of_range() {
    for name in LONGHANDS {
        for value in [
            "calc(1 + 2)",
            "calc(0)",
            "calc(2) main",
            "span calc(3)",
            "span calc(-1) main",
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_members(&source, &[(name, value)]);
            }
        }
    }
}

#[test]
fn placement_rejects_zero_noninteger_and_duplicate_components_without_losing_neighbors() {
    for name in LONGHANDS {
        for value in [
            "0",
            "-0",
            "+00",
            "1.0",
            "1e0",
            "span 0",
            "span -1",
            "span +00",
            "2 3",
            "main other 2",
            "span span 2",
            "span main other",
            "2 main 3",
        ] {
            let source = format!("color: red; {name}: {value}; color: blue");
            let report = parse_style_attribute(&source);
            assert_eq!(report.syntax().len(), 2, "{source}");
            assert_eq!(report.diagnostics().len(), 1, "{source}");
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDeclaration
            );
            assert!(validate_style_attribute(&source).is_err(), "{source}");
            assert!(
                parse_property_value_for_grammar(
                    grammar(name),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked accepted {name}: {value}"
            );
        }
    }
}

#[test]
fn default_and_reserved_keywords_cannot_serve_as_grid_line_names() {
    for name in LONGHANDS {
        for value in [
            "default",
            "DEFAULT 2",
            "span default",
            "2 Auto",
            "span auto 2",
            "span inherit",
            "d\\65 fault",
            "2 initial",
            "revert 2",
            "2 revert-layer",
            "unset 2",
        ] {
            let source = format!("{name}: {value}");
            assert!(!parse_style_attribute(&source).is_clean(), "{source}");
            assert!(
                parse_property_value_for_grammar(
                    grammar(name),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked accepted {source}"
            );
        }
        for value in [
            "auto",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
        ] {
            assert!(parse_style_attribute(&format!("{name}: {value}")).is_clean());
        }
    }
}

#[test]
fn row_and_column_omissions_copy_only_bare_names() {
    for (name, [start, end]) in [
        ("grid-row", ["grid-row-start", "grid-row-end"]),
        ("grid-column", ["grid-column-start", "grid-column-end"]),
    ] {
        for (value, first, second) in [
            ("hero", "hero", "hero"),
            ("4", "4", "auto"),
            ("4 hero", "4 hero", "auto"),
            ("span 2 hero", "span 2 hero", "auto"),
            ("calc(2)", "calc(2)", "auto"),
            ("hero / auto", "hero", "auto"),
            ("4 / span 2", "4", "span 2"),
        ] {
            for source in [declaration(name, value), checked(name, value)] {
                assert_members(&source, &[(start, first), (end, second)]);
            }
        }
    }
}

#[test]
fn area_omissions_follow_row_column_row_column_order() {
    for (value, row_start, column_start, row_end, column_end) in [
        ("hero", "hero", "hero", "hero", "hero"),
        ("2", "2", "auto", "auto", "auto"),
        ("2 hero", "2 hero", "auto", "auto", "auto"),
        ("hero / nav", "hero", "nav", "hero", "nav"),
        ("hero / 3", "hero", "3", "hero", "auto"),
        ("2 / nav", "2", "nav", "auto", "nav"),
        ("hero / auto", "hero", "auto", "hero", "auto"),
        ("hero / nav / auto", "hero", "nav", "auto", "nav"),
        ("hero / nav / end / auto", "hero", "nav", "end", "auto"),
    ] {
        for source in [declaration("grid-area", value), checked("grid-area", value)] {
            assert_members(
                &source,
                &[
                    ("grid-row-start", row_start),
                    ("grid-column-start", column_start),
                    ("grid-row-end", row_end),
                    ("grid-column-end", column_end),
                ],
            );
        }
    }
}

#[test]
fn placement_shorthands_reject_extra_or_empty_slash_members() {
    for (name, values) in [
        ("grid-row", &["1 /", "/ 2", "1 / / 2", "1 / 2 / 3"][..]),
        ("grid-column", &["1 /", "/ 2", "1 / / 2", "1 / 2 / 3"]),
        ("grid-area", &["1 /", "/ 2", "1 / / 2", "1 / 2 / 3 / 4 / 5"]),
    ] {
        for &value in values {
            let source = format!("{name}: {value}");
            assert!(!parse_style_attribute(&source).is_clean(), "{source}");
            assert!(
                parse_property_value_for_grammar(
                    grammar(name),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked accepted {source}"
            );
        }
    }
}

#[test]
fn valid_placement_conflicts_remain_authored_for_later_resolution() {
    for (name, value, members) in [
        (
            "grid-row",
            "5 / 2",
            [("grid-row-start", "5"), ("grid-row-end", "2")],
        ),
        (
            "grid-column",
            "8 / 8",
            [("grid-column-start", "8"), ("grid-column-end", "8")],
        ),
        (
            "grid-row",
            "span c / span c",
            [("grid-row-start", "span c"), ("grid-row-end", "span c")],
        ),
    ] {
        for source in [declaration(name, value), checked(name, value)] {
            assert_eq!(
                source
                    .value_components()
                    .serialize()
                    .unwrap()
                    .as_css()
                    .trim(),
                value
            );
            assert_members(&source, &members);
        }
    }
}

#[test]
fn grid_line_identifiers_preserve_case_and_decoded_escapes() {
    for value in ["none", "FoO", "foo", "\\31 foo"] {
        let source = declaration("grid-row-start", value);
        assert_eq!(
            source
                .value_components()
                .serialize()
                .unwrap()
                .as_css()
                .trim(),
            value
        );
    }
    for value in ["none", "FoO", "foo", "\\31 foo"] {
        let source = declaration("grid-row-start", value);
        assert_members(&source, &[("grid-row-start", value)]);
    }
    let upper = expanded(&declaration("grid-row-start", "FoO"));
    let lower = expanded(&declaration("grid-row-start", "foo"));
    assert_ne!(
        upper.items()[0].ordinary_value(),
        lower.items()[0].ordinary_value(),
        "custom identifiers preserve case"
    );
}

#[test]
fn frozen_i01_projections_keep_exactly_the_old_representable_forms() {
    let bare = declaration("grid-row", "hero");
    let CssKnownPropertyValueRef::GridRow(value) = bare.known().unwrap().property_value().unwrap()
    else {
        panic!("grid-row ordinary wrapper")
    };
    let old = value.i01_subset().expect("bare name belongs to I01");
    assert!(old.end().is_none(), "authored omission stays omitted");

    let explicit = declaration("grid-row", "hero / auto");
    let CssKnownPropertyValueRef::GridRow(value) =
        explicit.known().unwrap().property_value().unwrap()
    else {
        panic!("grid-row ordinary wrapper")
    };
    assert!(matches!(
        value.i01_subset().unwrap().end(),
        Some(CssGridLine::Auto)
    ));

    let area = declaration("grid-area", "hero / nav");
    let CssKnownPropertyValueRef::GridArea(value) = area.known().unwrap().property_value().unwrap()
    else {
        panic!("grid-area ordinary wrapper")
    };
    let old = value.i01_subset().expect("simple area belongs to I01");
    assert!(old.row_end().is_none());
    assert!(old.column_end().is_none());

    for (name, authored) in [
        ("grid-row-start", "main 2"),
        ("grid-row-start", "2147483648"),
        ("grid-row-start", "calc(2)"),
        ("grid-row", "main 2 / auto"),
        ("grid-area", "main 2 / nav"),
    ] {
        let source = declaration(name, authored);
        let value = source.known().unwrap().property_value().unwrap();
        let projected = match value {
            CssKnownPropertyValueRef::GridRowStart(value) => value.i01_subset().is_some(),
            CssKnownPropertyValueRef::GridRow(value) => value.i01_subset().is_some(),
            CssKnownPropertyValueRef::GridArea(value) => value.i01_subset().is_some(),
            _ => panic!("selected placement wrapper"),
        };
        assert!(
            !projected,
            "{name}: {authored} cannot project to frozen I01"
        );
    }
}

#[test]
fn globals_and_pending_reentry_keep_placement_members_and_source() {
    for (name, members) in [
        ("grid-row-start", &["grid-row-start"][..]),
        ("grid-row", &["grid-row-start", "grid-row-end"]),
        (
            "grid-area",
            &[
                "grid-row-start",
                "grid-column-start",
                "grid-row-end",
                "grid-column-end",
            ],
        ),
    ] {
        for (value, global) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, value);
            let values = expanded(&source);
            assert_eq!(values.items().len(), members.len());
            for (item, &member) in values.items().iter().zip(members) {
                assert_eq!(item.property().canonical_name(), member);
                assert_eq!(item.value(), CssContributionValueRef::Global(global));
                assert!(item.source().same_occurrence(&source));
            }
        }

        let source = declaration(name, "var(--placement)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("{name} remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        assert!(matches!(
            pending
                .reenter(parse_component_values("0").unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values("hero").unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed placement replacement")
        };
        assert_eq!(values.items().len(), members.len());
        for (item, &member) in values.items().iter().zip(members) {
            assert_eq!(item.property().canonical_name(), member);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn area_expansion_counts_four_members_before_normalization_allocates() {
    let report = parse_sheet(".p { grid-area: hero; }");
    assert!(report.is_clean());
    let original = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap();
    let error = match normalize_sheet_with_limits(report.syntax(), limits) {
        Ok(_) => panic!("four area contributions exceed a limit of three"),
        Err(error) => error,
    };
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3,
        }
    ));
    assert_eq!(
        report.syntax(),
        &original,
        "failure preserves the parsed source"
    );
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 4).unwrap(),
        )
        .is_ok()
    );
}
