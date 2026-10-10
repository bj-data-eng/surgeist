#![forbid(unsafe_code)]

//! New-operation evidence: CSSOM WD 2021-08-26 §§6.5–6.7, Variables 1
//! §3.2/§4, Cascade 5 §3, Logical 1 §4, and the adopted intrinsic tariff.
//! Expected bytes and counts are authored from those contracts, not captured.

use surgeist_css::*;
#[path = "common/property_expectations.rs"]
mod expectations;
use expectations::{CASES, INDEPENDENT_TERMINALS, MetadataExpectation};

fn all_terminal_expectations() -> Vec<(CssPropertyNameRef<'static>, u8, &'static str)> {
    let mut values: Vec<_> = CASES
        .iter()
        .filter(|row| matches!(row.metadata, MetadataExpectation::Longhand { .. }))
        .filter(|row| {
            !matches!(
                row.property,
                CssKnownProperty::Direction | CssKnownProperty::UnicodeBidi
            )
        })
        .map(|row| {
            (
                CssPropertyNameRef::Known(row.property),
                row.mapping.bucket(),
                row.name,
            )
        })
        .chain(
            INDEPENDENT_TERMINALS
                .iter()
                .map(|row| (row.property, row.mapping.bucket(), row.name)),
        )
        .collect();
    values.sort_by_key(|(_, bucket, name)| (*bucket, *name));
    values
}

fn keyframe_terminal_expected(name: CssPropertyNameRef<'_>) -> bool {
    match name {
        CssPropertyNameRef::Known(property) => !matches!(
            property,
            CssKnownProperty::AnimationName
                | CssKnownProperty::AnimationDuration
                | CssKnownProperty::AnimationDelay
                | CssKnownProperty::AnimationIterationCount
                | CssKnownProperty::AnimationDirection
                | CssKnownProperty::AnimationFillMode
                | CssKnownProperty::AnimationPlayState
        ),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            INDEPENDENT_TERMINALS
                .iter()
                .find(|row| row.property == name)
                .unwrap()
                .keyframe_admitted
        }
        CssPropertyNameRef::Custom(_) => true,
        _ => panic!("future terminal needs an independent expectation"),
    }
}

fn declarations(source: &str) -> CssDeclarationList {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone()
}
fn specified_block(source: &str) -> CssSpecifiedDeclarationBlock {
    CssSpecifiedDeclarationBlock::try_from_declarations(&declarations(source)).unwrap()
}
fn read(block: &CssSpecifiedDeclarationBlock, property: CssKnownProperty) -> Option<String> {
    block
        .property_value(CssPropertyNameRef::Known(property))
        .unwrap()
}

#[test]
fn gradient_inverses_preserve_conic_interpolation_and_double_stop_roles() {
    // Independently authored canonical values exercise reconstruction from
    // separate image occurrences, rather than direct authored-value emission.
    for (authored, expected) in [
        ("conic-gradient(red,blue)", "conic-gradient(red, blue)"),
        (
            "conic-gradient(from 45deg in oklch longer hue,red 0deg 90deg,blue)",
            "conic-gradient(from 45deg in oklch longer hue, red 0deg 90deg, blue)",
        ),
        (
            "conic-gradient(in oklab,red 0 50%,blue)",
            "conic-gradient(in oklab, red 0 50%, blue)",
        ),
        (
            "linear-gradient(in lab,red 10% 20%,blue)",
            "linear-gradient(in lab, red 10% 20%, blue)",
        ),
        (
            "radial-gradient(in oklab,red 10px 20%,blue)",
            "radial-gradient(in oklab, red 10px 20%, blue)",
        ),
    ] {
        let source = format!("background:none;background-image:{authored}");
        let ordinary = specified_block(&source);
        let rule = keyframes(&format!("@keyframes k {{ from {{ {source} }} }}"));
        let keyframe = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
            rule.blocks()[0].declarations(),
        )
        .unwrap();
        for block in [&ordinary, &keyframe] {
            let sources: Vec<_> = block.entries().iter().map(|e| e.source().clone()).collect();
            assert_eq!(
                read(block, CssKnownProperty::Background).as_deref(),
                Some(expected)
            );
            let css = format!("background: {expected};");
            assert_eq!(block.serialize_cssom().unwrap(), css);
            let error = block
                .serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    0,
                ))
                .unwrap_err();
            assert_eq!(
                resource(&error),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            assert_eq!(block.serialize_cssom().unwrap(), css);
            for (entry, source) in block.entries().iter().zip(&sources) {
                assert!(entry.source().same_occurrence(source));
            }
        }
    }
}

fn keyframes(source: &str) -> CssKeyframesRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("expected keyframes")
    };
    rule.clone()
}
fn resource(error: &CssDeclarationBlockError) -> CssSpecifiedValueSerializationErrorKind {
    let CssDeclarationBlockErrorKind::Serialization(error) = error.kind() else {
        panic!("{error:?}")
    };
    error.kind()
}

#[test]
fn priority_winners_keep_their_own_expanded_positions_and_occurrences() {
    let list = declarations("width:1px!important;height:4px;width:2px;width:3px!important");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "height: 4px; width: 3px !important;"
    );
    assert_eq!(
        block
            .entries()
            .iter()
            .map(CssSpecifiedDeclarationEntry::authored_ordinal)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert!(block.entries()[0].source().same_occurrence(&list[1]));
    assert!(block.entries()[1].source().same_occurrence(&list[3]));
    let block = specified_block("width:1px!important;height:4px;width:2px");
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "width: 1px !important; height: 4px;"
    );
}

#[test]
fn custom_code_point_keys_and_payload_trivia_survive_winner_selection() {
    let list = declarations("--Case:A/**/ B;--case:C;--Case:D");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "--case: C; --Case: D;");
    assert_eq!(
        block
            .entries()
            .iter()
            .map(CssSpecifiedDeclarationEntry::authored_ordinal)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        specified_block("--Case:A/**/ B").serialize_cssom().unwrap(),
        "--Case: A/**/ B;"
    );
    let name = list[1].custom().unwrap().name();
    assert_eq!(
        block
            .property_value(CssPropertyNameRef::Custom(name))
            .unwrap()
            .as_deref(),
        Some("C")
    );
}

#[test]
fn canonical_aliases_share_a_winner_and_distinct_legacy_grammar_does_not_reappear() {
    assert_eq!(
        specified_block("page-break-before:always;color:red;break-before:avoid")
            .serialize_cssom()
            .unwrap(),
        "color: red; break-before: avoid;"
    );
    assert_eq!(
        specified_block("font-stretch:condensed;font-width:expanded")
            .serialize_cssom()
            .unwrap(),
        "font-width: expanded;"
    );
    // Color Adjustment 1 §4.2 defines an ordinary deprecated shorthand; it is
    // eligible under CSSOM preference despite its deprecation.
    assert_eq!(
        specified_block("print-color-adjust:exact")
            .serialize_cssom()
            .unwrap(),
        "color-adjust: exact;"
    );
}

#[test]
fn immutable_priority_transition_shares_original_payload_without_occurrence_identity() {
    let list = declarations("/*😀*/m\\61 rgin:1px;--Case:A/**/ B");
    let original = &list[0];
    assert!(!original.is_name_case_sensitive());
    assert!(list[1].is_name_case_sensitive());
    assert!(
        original
            .with_importance(CssImportance::Normal)
            .same_occurrence(original)
    );
    let important = original.with_importance(CssImportance::Important);
    assert!(!important.same_occurrence(original));
    assert_eq!(original.importance(), CssImportance::Normal);
    assert_eq!(important.parser_context(), original.parser_context());
    assert_eq!(important.body(), original.body());
    assert_eq!(important.value_components(), original.value_components());
    assert_eq!(important.parsed_name(), original.parsed_name());
    assert_eq!(important.parsed_value(), original.parsed_value());
    assert_eq!(
        important.to_specified_css().unwrap(),
        "margin: 1px !important;"
    );
    assert_eq!(original.to_specified_css().unwrap(), "margin: 1px;");
}

#[test]
fn complete_quad_reconstructs_over_unrelated_intervening_members() {
    let block = specified_block("margin:1px 2px;color:red;margin-left:3px");
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin: 1px 2px 1px 3px; color: red;"
    );
    assert_eq!(
        read(&block, CssKnownProperty::Margin).as_deref(),
        Some("1px 2px 1px 3px")
    );
    assert_eq!(
        block
            .entries()
            .iter()
            .filter(|e| matches!(
                e.property_name(),
                CssPropertyNameRef::Known(CssKnownProperty::MarginLeft)
            ))
            .map(CssSpecifiedDeclarationEntry::authored_ordinal)
            .collect::<Vec<_>>(),
        [2]
    );
    assert_eq!(
        read(
            &specified_block("margin-top:1px;margin-right:2px;margin-bottom:1px"),
            CssKnownProperty::Margin
        ),
        None
    );
}

#[test]
fn mixed_importance_and_intervening_logical_groups_disqualify_quad_reconstruction() {
    let block = specified_block(
        "margin-top:1px!important;margin-right:2px;margin-bottom:3px;margin-left:4px",
    );
    assert_eq!(read(&block, CssKnownProperty::Margin), None);
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin-top: 1px !important; margin-right: 2px; margin-bottom: 3px; margin-left: 4px;"
    );
    let block = specified_block(
        "margin-top:1px;margin-inline-start:7px;margin-right:2px;margin-bottom:3px;margin-left:4px",
    );
    assert_eq!(read(&block, CssKnownProperty::Margin), None);
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin-top: 1px; margin-inline-start: 7px; margin-right: 2px; margin-bottom: 3px; margin-left: 4px;"
    );
    let block = specified_block(
        "margin-top:1px;padding-inline-start:7px;margin-right:2px;margin-bottom:3px;margin-left:4px",
    );
    assert_eq!(
        read(&block, CssKnownProperty::Margin).as_deref(),
        Some("1px 2px 3px 4px")
    );
}

#[test]
fn global_quad_uses_physical_mode_and_rejects_mixed_declared_phases() {
    assert_eq!(
        specified_block(
            "margin-top:inherit;margin-right:inherit;margin-bottom:inherit;margin-left:inherit"
        )
        .serialize_cssom()
        .unwrap(),
        "margin: inherit;"
    );
    assert_eq!(
        read(
            &specified_block(
                "margin-top:inherit;margin-right:inherit;margin-bottom:inherit;margin-left:initial"
            ),
            CssKnownProperty::Margin
        ),
        None
    );
    assert_eq!(
        read(
            &specified_block(
                "margin-block-start:inherit;margin-inline-start:inherit;margin-block-end:inherit;margin-inline-end:inherit"
            ),
            CssKnownProperty::Margin
        ),
        None
    );
}

#[test]
fn fixed_pending_members_reconstruct_only_the_original_complete_shorthand() {
    let list = declarations("pause:var(--P)");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "pause: var(--P);");
    assert_eq!(
        read(&block, CssKnownProperty::PauseBefore).as_deref(),
        Some("")
    );
    assert_eq!(
        read(&block, CssKnownProperty::PauseAfter).as_deref(),
        Some("")
    );
    assert!(
        block
            .entries()
            .iter()
            .all(|e| e.source().same_occurrence(&list[0]))
    );
    assert_eq!(
        specified_block("pause:var(--P);pause-after:1s")
            .serialize_cssom()
            .unwrap(),
        "pause-before: ; pause-after: 1s;"
    );
    assert_eq!(
        specified_block("pause-before:var(--B)")
            .serialize_cssom()
            .unwrap(),
        "pause-before: var(--B);"
    );
}

#[test]
fn all_eight_pending_mode_families_fail_at_the_real_occurrence_and_retry_in_both_modes() {
    for (property, name, scalar) in [
        (CssKnownProperty::Inset, "inset", "1px"),
        (CssKnownProperty::Margin, "margin", "1px"),
        (CssKnownProperty::Padding, "padding", "1px"),
        (CssKnownProperty::BorderWidth, "border-width", "thin"),
        (CssKnownProperty::BorderStyle, "border-style", "solid"),
        (CssKnownProperty::BorderColor, "border-color", "red"),
        (CssKnownProperty::ScrollPadding, "scroll-padding", "1px"),
        (CssKnownProperty::ScrollMargin, "scroll-margin", "1px"),
    ] {
        let list = declarations(&format!("{name}:var(--M);opacity:1"));
        let error = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap_err();
        assert!(
            matches!(error.kind(), CssDeclarationBlockErrorKind::PendingFootprintUndetermined { property: p } if *p == property)
        );
        assert_eq!(error.authored_ordinal(), Some(0));
        assert_eq!(error.member_ordinal(), None);
        assert!(error.declaration().unwrap().same_occurrence(&list[0]));
        assert_eq!(
            list[0].to_specified_css().unwrap(),
            format!("{name}: var(--M);")
        );
        let CssExpansion::Pending(pending) = expand_declaration(&list[0]).unwrap() else {
            panic!("pending")
        };
        let following = expand_declaration(&list[1]).unwrap();
        for (replacement, expected) in [
            (scalar.to_string(), scalar.to_string()),
            (format!("logical {scalar}"), format!("logical {scalar}")),
        ] {
            let completed = CssExpansion::Contributions(
                pending
                    .reenter(parse_component_values(&replacement).unwrap())
                    .unwrap(),
            );
            let block =
                CssSpecifiedDeclarationBlock::try_from_expansions(&[completed, following.clone()])
                    .unwrap();
            assert_eq!(block.entries().len(), 5, "{name}");
            assert!(
                block.entries()[..4]
                    .iter()
                    .all(|e| e.source().same_occurrence(&list[0]))
            );
            assert_eq!(read(&block, property), Some(expected.clone()));
            assert_eq!(
                block.serialize_cssom().unwrap(),
                format!("{name}: {expected}; opacity: 1;")
            );
        }
        assert!(pending.source().same_occurrence(&list[0]));
    }
}

#[test]
fn completed_mode_footprints_override_only_their_selected_members() {
    let list = declarations("margin:var(--M);margin-left:7px");
    let CssExpansion::Pending(pending) = expand_declaration(&list[0]).unwrap() else {
        panic!("pending")
    };
    let after = expand_declaration(&list[1]).unwrap();
    for (input, expected) in [
        ("1px", "margin: 1px 1px 1px 7px;"),
        ("logical 1px", "margin: logical 1px; margin-left: 7px;"),
    ] {
        let complete = CssExpansion::Contributions(
            pending
                .reenter(parse_component_values(input).unwrap())
                .unwrap(),
        );
        let block =
            CssSpecifiedDeclarationBlock::try_from_expansions(&[complete, after.clone()]).unwrap();
        assert_eq!(block.serialize_cssom().unwrap(), expected);
    }
    let inline = expand_declaration(&declarations("margin-inline-start:7px")[0]).unwrap();
    for (input, expected) in [
        ("1px", "margin: 1px; margin-inline-start: 7px;"),
        ("logical 1px", "margin: logical 1px 7px 1px 1px;"),
    ] {
        let complete = CssExpansion::Contributions(
            pending
                .reenter(parse_component_values(input).unwrap())
                .unwrap(),
        );
        let block =
            CssSpecifiedDeclarationBlock::try_from_expansions(&[complete, inline.clone()]).unwrap();
        assert_eq!(block.serialize_cssom().unwrap(), expected);
    }
}

#[test]
fn reset_only_override_prevents_broad_border_but_keeps_smaller_exact_families() {
    let complete = specified_block("border:1px solid red");
    assert_eq!(
        read(&complete, CssKnownProperty::Border).as_deref(),
        Some("1px solid red")
    );
    let partial = specified_block("border:1px solid red;border-image-source:url(x)");
    assert_eq!(read(&partial, CssKnownProperty::Border), None);
    assert_eq!(
        read(&partial, CssKnownProperty::BorderWidth).as_deref(),
        Some("1px")
    );
    assert_eq!(
        read(&partial, CssKnownProperty::BorderStyle).as_deref(),
        Some("solid")
    );
    assert_eq!(
        read(&partial, CssKnownProperty::BorderColor).as_deref(),
        Some("red")
    );
}

#[test]
fn system_font_reconstructs_complete_environment_state_and_partial_override_fails_honestly() {
    let complete = specified_block("font:caption");
    assert_eq!(
        read(&complete, CssKnownProperty::Font).as_deref(),
        Some("caption")
    );
    assert_eq!(complete.serialize_cssom().unwrap(), "font: caption;");
    let partial = specified_block("font:caption;font-size:12px");
    assert_eq!(read(&partial, CssKnownProperty::Font), None);
    let error = partial.serialize_cssom().unwrap_err();
    assert!(matches!(
        error.kind(),
        CssDeclarationBlockErrorKind::UnresolvedEnvironment { .. }
    ));
    assert_eq!(error.authored_ordinal(), Some(0));
}

#[test]
fn complete_supported_all_universe_includes_independent_svg_with_custom_and_direction_exclusions() {
    let list = declarations("all:inherit;direction:rtl;--X:A");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let expected: Vec<_> = all_terminal_expectations()
        .into_iter()
        .map(|(name, _, _)| name)
        .collect();
    let actual: Vec<_> = block
        .entries()
        .iter()
        .filter(|e| e.authored_ordinal() == 0)
        .map(CssSpecifiedDeclarationEntry::property_name)
        .collect();
    assert_eq!(actual, expected);
    assert!(
        block
            .entries()
            .iter()
            .filter(|e| e.authored_ordinal() == 0)
            .all(|e| e.source().same_occurrence(&list[0]))
    );
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "all: inherit; direction: rtl; --X: A;"
    );
}

#[test]
fn every_supported_finite_shorthand_uses_its_own_independent_inverse_literal() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for row in CASES {
        let eligible = match row.property.metadata().unwrap().kind() {
            CssPropertyKindRef::Shorthand(meta) => !meta.is_legacy(),
            CssPropertyKindRef::FourSideShorthand(_) => true,
            _ => false,
        };
        if !eligible {
            continue;
        }
        checked += 1;
        let Some(expected) = row.inverse.as_ref() else {
            failures.push(format!("{} lacks independent inverse evidence", row.name));
            continue;
        };
        let source = format!("{}:{}", row.name, expected.authored);
        let report = parse_style_attribute(&source);
        if !report.is_clean() {
            failures.push(format!("{source:?}: independent stimulus was rejected"));
            continue;
        }
        let list = report.syntax();
        let block = match CssSpecifiedDeclarationBlock::try_from_declarations(list) {
            Ok(block) => block,
            Err(error) => {
                failures.push(format!(
                    "{source:?}: build error {:?}, source {:?}, member {:?}",
                    error.kind(),
                    error.authored_ordinal(),
                    error.member_ordinal()
                ));
                continue;
            }
        };
        match block.property_value(CssPropertyNameRef::Known(row.property)) {
            Ok(actual) if actual.as_deref() == Some(expected.value) => {}
            Ok(actual) => failures.push(format!(
                "{source:?}: expected {:?}, actual {actual:?}",
                expected.value
            )),
            Err(error) => failures.push(format!(
                "{source:?}: read error {:?}, source {:?}, member {:?}; expected {:?}",
                error.kind(),
                error.authored_ordinal(),
                error.member_ordinal(),
                expected.value
            )),
        }
        if list.len() != 1
            || !block
                .entries()
                .iter()
                .all(|entry| entry.source().same_occurrence(&list[0]))
        {
            failures.push(format!("{source:?}: original source identity changed"));
        }
    }
    assert_eq!(
        checked, 82,
        "complete independently authored finite family inventory"
    );
    assert!(
        failures.is_empty(),
        "{} family failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn ordinary_atomic_list_front_shares_build_output_and_retained_value_costs() {
    // Empty source + output aggregates = I2/P2. A custom one-token value
    // adds source I4/P2 and emission I4/P4: total I10/P8. Two originals
    // add another source I4/P2 even though only the second survives.
    for (source, input, projection, expected) in [
        ("", 2, 2, ""),
        ("--X:A", 10, 8, "--X: A;"),
        ("--X:A;--X:B", 14, 10, "--X: B;"),
    ] {
        let list = declarations(source);
        let limits = CssSpecifiedValueSerializationLimits::new(input, projection, expected.len());
        assert_eq!(list.serialize_cssom_with_limits(limits).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
        ] {
            assert_eq!(
                resource(&list.serialize_cssom_with_limits(limits).unwrap_err()),
                kind
            );
        }
        if !expected.is_empty() {
            let error = list
                .serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input,
                    projection,
                    expected.len() - 1,
                ))
                .unwrap_err();
            assert_eq!(
                resource(&error),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            assert_eq!(error.authored_ordinal(), Some(list.len() - 1));
            assert_eq!(error.member_ordinal(), Some(0));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(&list[list.len() - 1])
            );
        }
        assert_eq!(list.serialize_cssom().unwrap(), expected);
    }
    let list = declarations("width:1px!important;height:4px;width:2px;width:3px!important");
    assert_eq!(
        list.serialize_cssom().unwrap(),
        "height: 4px; width: 3px !important;"
    );
    assert_eq!(list.len(), 4);
}

#[test]
fn keyframe_atomic_list_front_adds_closed_source_and_terminal_admission_to_shared_budget() {
    for (source, input, projection, expected) in [
        ("", 2, 2, ""),
        ("--X:A", 10, 10, "--X: A;"),
        ("--X:A;--X:B", 14, 14, "--X: B;"),
    ] {
        let rule = keyframes(&format!("@keyframes k{{from{{{source}}}}}"));
        let list = rule.blocks()[0].declarations();
        let limits = CssSpecifiedValueSerializationLimits::new(input, projection, expected.len());
        assert_eq!(list.serialize_cssom_with_limits(limits).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
        ] {
            assert_eq!(
                resource(&list.serialize_cssom_with_limits(limits).unwrap_err()),
                kind
            );
        }
        assert_eq!(list.serialize_cssom().unwrap(), expected);
    }
    let rule =
        keyframes("@keyframes k { from { opacity:0;opacity:1;pause:var(--P);pause-after:1s } }");
    assert_eq!(
        rule.blocks()[0].declarations().serialize_cssom().unwrap(),
        "opacity: 1; pause-before: ; pause-after: 1s;"
    );
}

#[test]
fn failed_all_reconstruction_preserves_logical_before_physical_margin_order() {
    let block = specified_block("all:inherit;width:2px");
    assert_eq!(read(&block, CssKnownProperty::All), None);
    let margins: Vec<_> = block
        .entries()
        .iter()
        .filter_map(|e| match e.property_name() {
            CssPropertyNameRef::Known(p) if p.canonical_name().starts_with("margin-") => {
                Some(p.canonical_name())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        margins,
        [
            "margin-block-end",
            "margin-block-start",
            "margin-inline-end",
            "margin-inline-start",
            "margin-bottom",
            "margin-left",
            "margin-right",
            "margin-top"
        ]
    );
    let css = block.serialize_cssom().unwrap();
    let margin_declarations: Vec<_> = css
        .split("; ")
        .filter(|s| {
            s.starts_with("margin:")
                || s.starts_with("margin-block:")
                || s.starts_with("margin-inline:")
        })
        .map(|s| s.trim_end_matches(';'))
        .collect();
    assert_eq!(
        margin_declarations,
        [
            "margin-block: inherit",
            "margin-inline: inherit",
            "margin: inherit"
        ]
    );
}

#[test]
fn construction_tariff_charges_losers_and_zero_byte_build_budget_is_unspent() {
    for (source, input, projection) in [
        ("", 1, 1),
        ("opacity:0", 5, 3),
        ("opacity:0;opacity:1", 9, 5),
    ] {
        let list = declarations(source);
        let exact = CssSpecifiedValueSerializationLimits::new(input, projection, 0);
        CssSpecifiedDeclarationBlock::try_from_declarations_with_limits(&list, exact).unwrap();
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
        ] {
            let error =
                CssSpecifiedDeclarationBlock::try_from_declarations_with_limits(&list, limits)
                    .unwrap_err();
            assert_eq!(resource(&error), kind);
        }
        CssSpecifiedDeclarationBlock::try_from_declarations_with_limits(&list, exact).unwrap();
    }
}

#[test]
fn keyframe_sources_keep_exact_original_byte_utf16_origins_and_occurrence_identity() {
    let source = "/*😀*/\n@keyframes k {\n from { m\\61 rgin:1px; --X:/*a*/ready; }\n}";
    let rule = keyframes(source);
    let list = rule.blocks()[0].declarations();
    for (index, name_start, name_end, value_start, value_end, column) in
        [(0, 32, 41, 42, 45, 8), (1, 47, 50, 51, 61, 23)]
    {
        let declaration = &list.as_slice()[index];
        let retained = declaration.source();
        let name = retained.parsed_name().unwrap();
        let value = retained.parsed_value().unwrap();
        assert_eq!(name.span().start().byte_offset().value(), name_start);
        assert_eq!(name.span().end().byte_offset().value(), name_end);
        assert_eq!(value.span().start().byte_offset().value(), value_start);
        assert_eq!(value.span().end().byte_offset().value(), value_end);
        assert_eq!(name.span().start().line().value(), 2);
        assert_eq!(name.span().start().column().value(), column);
        assert_eq!(name.source().as_str(), source);
        assert!(declaration.clone().source().same_occurrence(retained));
        assert!(
            !keyframes(source).blocks()[0].declarations().as_slice()[index]
                .source()
                .same_occurrence(retained)
        );
        assert_eq!(retained.position(), Some(declaration.position()));
        assert_eq!(retained.body(), declaration.body());
        assert_eq!(retained.importance(), CssImportance::Normal);
    }
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap();
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin: 1px; --X: /*a*/ready;"
    );
}

#[test]
fn keyframe_normalization_and_expansion_domain_reject_important_and_excluded_sources() {
    let rule =
        keyframes("@keyframes k { from { opacity:0;opacity:1;pause:var(--P);pause-after:1s } }");
    let list = rule.blocks()[0].declarations();
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap();
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "opacity: 1; pause-before: ; pause-after: 1s;"
    );
    assert!(
        block.entries()[0]
            .source()
            .same_occurrence(list.as_slice()[1].source())
    );
    for (source, reason) in [
        (
            "opacity:0!important",
            CssInvalidKeyframeSourceReason::Importance,
        ),
        (
            "animation-duration:1s",
            CssInvalidKeyframeSourceReason::Property,
        ),
    ] {
        let declaration = &declarations(source)[0];
        let expansion = expand_declaration(declaration).unwrap();
        let error = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(
            std::slice::from_ref(&expansion),
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), CssDeclarationBlockErrorKind::InvalidKeyframeSource { reason: r } if *r == reason)
        );
        assert_eq!(error.authored_ordinal(), Some(0));
        assert!(error.declaration().unwrap().same_occurrence(declaration));
        CssSpecifiedDeclarationBlock::try_from_expansions(&[expansion]).unwrap();
    }
    let expansion = expand_declaration(&declarations("--animation-duration:1s")[0]).unwrap();
    assert_eq!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[expansion])
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        "--animation-duration: 1s;"
    );
}

#[test]
fn keyframe_all_uses_filtered_universe_and_never_emits_a_forbidden_animation_property() {
    let complete = keyframes("@keyframes k { from { all:inherit } }");
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        complete.blocks()[0].declarations(),
    )
    .unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "all: inherit;");
    for entry in block.entries() {
        assert!(keyframe_terminal_expected(entry.property_name()));
        if let CssPropertyNameRef::Known(p) = entry.property_name()
            && p.canonical_name().starts_with("animation-")
        {
            assert_eq!(p, CssKnownProperty::AnimationTimingFunction);
        }
    }
    let partial = keyframes("@keyframes k { from { all:inherit;width:2px } }");
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        partial.blocks()[0].declarations(),
    )
    .unwrap();
    assert_eq!(read(&block, CssKnownProperty::All), None);
    assert_eq!(read(&block, CssKnownProperty::AnimationDuration), None);
    assert_eq!(
        read(&block, CssKnownProperty::AnimationTimingFunction).as_deref(),
        Some("inherit")
    );
    let css = block.serialize_cssom().unwrap();
    assert!(!css.contains("animation-duration:") && !css.contains("animation:"));
    assert!(css.contains("animation-timing-function: inherit;"));
}

#[test]
fn keyframe_build_tariff_accounts_for_source_and_generated_terminal_admission() {
    for (source, input, projection) in [
        ("", 1, 1),
        ("opacity:0", 5, 5),
        ("opacity:0;opacity:1", 9, 9),
    ] {
        let rule = keyframes(&format!("@keyframes k{{from{{{source}}}}}"));
        let list = rule.blocks()[0].declarations();
        let exact = CssSpecifiedValueSerializationLimits::new(input, projection, 0);
        CssSpecifiedDeclarationBlock::try_from_keyframe_declarations_with_limits(list, exact)
            .unwrap();
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
        ] {
            assert_eq!(
                resource(
                    &CssSpecifiedDeclarationBlock::try_from_keyframe_declarations_with_limits(
                        list, limits
                    )
                    .unwrap_err()
                ),
                kind
            );
        }
    }
}

#[test]
fn keyframe_pending_mode_error_and_completed_fronts_keep_the_real_original_source() {
    for (property, name, scalar) in [
        (CssKnownProperty::Inset, "inset", "1px"),
        (CssKnownProperty::Margin, "margin", "1px"),
        (CssKnownProperty::Padding, "padding", "1px"),
        (CssKnownProperty::BorderWidth, "border-width", "thin"),
        (CssKnownProperty::BorderStyle, "border-style", "solid"),
        (CssKnownProperty::BorderColor, "border-color", "red"),
        (CssKnownProperty::ScrollPadding, "scroll-padding", "1px"),
        (CssKnownProperty::ScrollMargin, "scroll-margin", "1px"),
    ] {
        let rule = keyframes(&format!(
            "@keyframes k{{from{{opacity:1;{name}:var(--M)}}}}"
        ));
        let list = rule.blocks()[0].declarations();
        let original = list.as_slice()[1].source();
        for error in [
            CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap_err(),
            list.serialize_cssom().unwrap_err(),
        ] {
            assert!(
                matches!(error.kind(), CssDeclarationBlockErrorKind::PendingFootprintUndetermined { property: p } if *p == property)
            );
            assert_eq!(error.authored_ordinal(), Some(1));
            assert_eq!(error.member_ordinal(), None);
            assert!(error.declaration().unwrap().same_occurrence(original));
        }
        assert_eq!(
            original.to_specified_css().unwrap(),
            format!("{name}: var(--M);")
        );
        let CssExpansion::Pending(pending) = expand_declaration(original).unwrap() else {
            panic!("pending")
        };
        let before = expand_declaration(list.as_slice()[0].source()).unwrap();
        for replacement in [scalar.to_string(), format!("logical {scalar}")] {
            let completed = CssExpansion::Contributions(
                pending
                    .reenter(parse_component_values(&replacement).unwrap())
                    .unwrap(),
            );
            let block = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[
                before.clone(),
                completed,
            ])
            .unwrap();
            assert_eq!(
                read(&block, property).as_deref(),
                Some(replacement.as_str())
            );
            assert_eq!(
                block.serialize_cssom().unwrap(),
                format!("opacity: 1; {name}: {replacement};")
            );
            assert!(
                block.entries()[1..]
                    .iter()
                    .all(|e| e.source().same_occurrence(original) && e.authored_ordinal() == 1)
            );
        }
        assert!(pending.source().same_occurrence(original));
    }
}

#[test]
fn keyframe_reentry_preserves_quirks_context_and_genuine_programmatic_absence() {
    let context = CssParserContext::new(CssParserMode::Quirks);
    let report = context.parse_sheet("@keyframes k{from{width:var(--W)}}");
    assert!(report.is_clean());
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("keyframes")
    };
    let source = rule.blocks()[0].declarations().as_slice()[0].source();
    assert_eq!(source.parser_context(), context);
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("pending")
    };
    let completed = CssExpansion::Contributions(
        pending
            .reenter(parse_component_values("1").unwrap())
            .unwrap(),
    );
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[completed]).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "width: 1px;");
    assert!(block.entries()[0].source().same_occurrence(source));
    assert_eq!(block.entries()[0].source().parser_context(), context);
    assert_eq!(block.entries()[0].source().position(), source.position());
    let ordinary = declarations("width:var(--W)");
    let CssExpansion::Pending(pending) = expand_declaration(&ordinary[0]).unwrap() else {
        panic!("pending")
    };
    assert!(
        pending
            .reenter(parse_component_values("1").unwrap())
            .is_err()
    );

    let source = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Opacity),
        parse_component_values("1").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(source.position(), None);
    let expansion = expand_declaration(&source).unwrap();
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[expansion]).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "opacity: 1;");
    assert_eq!(block.entries()[0].source().position(), None);
    assert!(block.entries()[0].source().same_occurrence(&source));
}

#[test]
fn keyframe_all_charges_even_the_filtered_generated_terminal_checks() {
    let expected = all_terminal_expectations();
    let universe = expected.len();
    let retained = expected
        .iter()
        .filter(|(name, _, _)| keyframe_terminal_expected(*name))
        .count();
    // Source/output aggregate P1, source admission P1, each generated entry P1,
    // every generated keyframe check P1, and each retained winner lookup P1.
    let projection = 2 + 2 * universe + retained;
    let rule = keyframes("@keyframes k{from{all:inherit}}");
    let list = rule.blocks()[0].declarations();
    let exact = CssSpecifiedValueSerializationLimits::new(5, projection, 0);
    let block =
        CssSpecifiedDeclarationBlock::try_from_keyframe_declarations_with_limits(list, exact)
            .unwrap();
    assert_eq!(block.entries().len(), retained);
    let error = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations_with_limits(
        list,
        CssSpecifiedValueSerializationLimits::new(5, projection - 1, 0),
    )
    .unwrap_err();
    assert_eq!(
        resource(&error),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(error.authored_ordinal(), Some(0));
    assert_eq!(
        error.member_ordinal(),
        Some(block.entries().last().unwrap().member_ordinal())
    );
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(list.as_slice()[0].source())
    );
    assert_eq!(block.serialize_cssom().unwrap(), "all: inherit;");
}

#[test]
fn failed_byte_output_retains_snapshot_and_retry_is_atomic() {
    let list = declarations("margin:1px 2px;color:red;margin-left:3px");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let expected = "margin: 1px 2px 1px 3px; color: red;";
    let limits = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len());
    assert_eq!(block.serialize_cssom_with_limits(limits).unwrap(), expected);
    let limits = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len() - 1);
    assert_eq!(
        resource(&block.serialize_cssom_with_limits(limits).unwrap_err()),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(block.serialize_cssom().unwrap(), expected);
    assert!(
        block
            .entries()
            .iter()
            .all(|e| e.source().same_occurrence(&list[e.authored_ordinal()]))
    );
}

#[test]
fn requested_value_probe_adds_output_aggregate_to_exact_success_and_rejection_costs() {
    let pair = specified_block("pause-before:none;pause-after:none");
    let name = CssPropertyNameRef::Known(CssKnownProperty::Pause);
    // I1/P1 aggregate + intrinsic P18 probe + owning I2/P2 pair output.
    assert_eq!(
        pair.property_value_with_limits(name, CssSpecifiedValueSerializationLimits::new(3, 21, 4))
            .unwrap()
            .as_deref(),
        Some("none")
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 21, 4),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 20, 4),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 21, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = pair.property_value_with_limits(name, limits).unwrap_err();
        assert_eq!(resource(&error), kind);
        assert_eq!(error.authored_ordinal(), Some(0));
        assert_eq!(error.member_ordinal(), Some(0));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(pair.entries()[0].source())
        );
    }
    for (source, cost) in [
        ("pause-before:none", 4),
        ("pause-before:none!important;pause-after:none", 6),
    ] {
        let block = specified_block(source);
        assert_eq!(
            block
                .property_value_with_limits(
                    name,
                    CssSpecifiedValueSerializationLimits::new(1, cost, 0)
                )
                .unwrap(),
            None
        );
        assert_eq!(
            resource(
                &block
                    .property_value_with_limits(
                        name,
                        CssSpecifiedValueSerializationLimits::new(1, cost - 1, 0)
                    )
                    .unwrap_err()
            ),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
    }
    assert_eq!(
        read(&pair, CssKnownProperty::Pause).as_deref(),
        Some("none")
    );
}

#[test]
fn requested_shorthand_output_error_reports_its_member_after_an_unrelated_source() {
    let list = declarations("opacity:0;pause-before:inherit;pause-after:inherit");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let name = CssPropertyNameRef::Known(CssKnownProperty::Pause);
    let exact = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, 7);
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("inherit")
    );
    let error = block
        .property_value_with_limits(
            name,
            CssSpecifiedValueSerializationLimits::new(65_536, 262_144, 6),
        )
        .unwrap_err();
    assert_eq!(
        resource(&error),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(error.authored_ordinal(), Some(1));
    assert_eq!(error.member_ordinal(), Some(0));
    assert!(error.declaration().unwrap().same_occurrence(&list[1]));
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("inherit")
    );
}

#[test]
fn exact_numeric_semantics_never_coalesce_from_rounded_equal_text() {
    let block = specified_block(
        "border-top-width:1.0000001px;border-right-width:1.0000002px;border-bottom-width:1.0000001px;border-left-width:1.0000002px",
    );
    // Two exact values remain two slots even though both providers round to 1px.
    assert_eq!(
        read(&block, CssKnownProperty::BorderWidth).as_deref(),
        Some("1px 1px")
    );
    assert_eq!(
        read(
            &specified_block(
                "margin-top:1.0PX;margin-right:1px;margin-bottom:1.00px;margin-left:1px"
            ),
            CssKnownProperty::Margin
        )
        .as_deref(),
        Some("1px")
    );
    assert_eq!(
        read(
            &specified_block("interest-delay-start:1s;interest-delay-end:1000ms"),
            CssKnownProperty::InterestDelay
        )
        .as_deref(),
        Some("1s")
    );
}

#[test]
fn inverse_numeric_graph_equality_ignores_original_coordinates_and_preserves_phase() {
    let equal = specified_block(
        "margin-top:calc(1px + 2px);margin-right:calc(1px + 2px);margin-bottom:calc(1px + 2px);margin-left:calc(1px + 2px)",
    );
    assert_eq!(
        read(&equal, CssKnownProperty::Margin).as_deref(),
        Some("calc(3px)")
    );
    let phases = specified_block(
        "margin-top:calc(1px + 2px);margin-right:3px;margin-bottom:calc(1px + 2px);margin-left:3px",
    );
    assert_eq!(
        read(&phases, CssKnownProperty::Margin).as_deref(),
        Some("calc(3px) 3px")
    );
    let graphs = specified_block(
        "margin-top:calc(1px + 2px);margin-right:calc(3px);margin-bottom:calc(1px + 2px);margin-left:calc(3px)",
    );
    assert_eq!(
        read(&graphs, CssKnownProperty::Margin).as_deref(),
        Some("calc(3px) calc(3px)")
    );
    let report = CssParserContext::new(CssParserMode::Quirks)
        .parse_style_attribute("margin-top:1;margin-right:1px;margin-bottom:1;margin-left:1px");
    assert!(report.is_clean());
    let quirky = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(
        read(&quirky, CssKnownProperty::Margin).as_deref(),
        Some("1px 1px")
    );
}

#[test]
fn mask_inverse_omits_whole_initial_facets_without_losing_ordered_layer_cardinality() {
    for (source, expected) in [
        ("mask:none", "none"),
        ("mask:none,none", "none, none"),
        (
            "mask:none 0% 0% / auto repeat border-box add match-source",
            "none",
        ),
        (
            "mask:none 0% 0% / auto auto repeat border-box add match-source",
            "none",
        ),
        ("mask:none no-clip", "no-clip"),
        (
            "mask:none padding-box no-clip subtract alpha",
            "padding-box no-clip subtract alpha",
        ),
        (
            "mask:none 0% 0% / contain repeat border-box add match-source",
            "0% 0% / contain",
        ),
        ("mask:none 0% 0% / 10px auto", "0% 0% / 10px auto"),
    ] {
        let block = specified_block(source);
        assert_eq!(
            read(&block, CssKnownProperty::Mask).as_deref(),
            Some(expected),
            "{source}"
        );
    }
    let mismatch = specified_block("mask:none,none;mask-mode:alpha");
    assert_eq!(read(&mismatch, CssKnownProperty::Mask), None);
}

#[test]
fn mask_border_inverse_omits_initials_and_keeps_required_slice_slash_sections() {
    for (source, expected) in [
        ("mask-border:none", "none"),
        ("mask-border:none 0 / auto / 0 stretch alpha", "none"),
        ("mask-border:none 0.0 / auto / 0.00 stretch alpha", "none"),
        ("mask-border:none 10", "10"),
        ("mask-border:none 0 / 2", "0 / 2"),
        ("mask-border:none 0 / auto / 2", "0 / / 2"),
        (
            "mask-border:none 0 / 2 / 3 round luminance",
            "0 / 2 / 3 round luminance",
        ),
        ("mask-border:none 0 fill", "0 fill"),
    ] {
        let block = specified_block(source);
        assert_eq!(
            read(&block, CssKnownProperty::MaskBorder).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn radius_and_background_size_inverses_compare_effective_values_without_authored_omission_flags() {
    for (source, property, expected) in [
        (
            "border-radius:1px / 1px",
            CssKnownProperty::BorderRadius,
            "1px",
        ),
        (
            "border-top-left-radius:1px;border-top-right-radius:1.0px 1px;border-bottom-right-radius:1px 1.00px;border-bottom-left-radius:1px",
            CssKnownProperty::BorderRadius,
            "1px",
        ),
        (
            "border-radius:1px / 2px",
            CssKnownProperty::BorderRadius,
            "1px / 2px",
        ),
        (
            "border-radius:calc(1px + 2px) / calc(1px + 2px)",
            CssKnownProperty::BorderRadius,
            "calc(3px)",
        ),
        (
            "background:none 0% 0% / auto auto",
            CssKnownProperty::Background,
            "none",
        ),
        (
            "background:none 0% 0% / 10px auto",
            CssKnownProperty::Background,
            "0% 0% / 10px auto",
        ),
    ] {
        assert_eq!(
            read(&specified_block(source), property).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn layered_and_timing_inverses_preserve_cardinality_without_computed_cycling() {
    let background = specified_block("background:red;background-image:none,none");
    assert_eq!(read(&background, CssKnownProperty::Background), None);
    let mask = specified_block("mask:none;mask-mode:alpha,luminance");
    assert_eq!(read(&mask, CssKnownProperty::Mask), None);
    let transition = specified_block("transition:opacity 1s;transition-delay:0s,0s");
    assert_eq!(read(&transition, CssKnownProperty::Transition), None);
    let animation = specified_block("animation:fade 1s;animation-name:fade,spin");
    assert_eq!(read(&animation, CssKnownProperty::Animation), None);
    assert_eq!(
        read(
            &specified_block("transition:opacity 1s,transform 2s linear"),
            CssKnownProperty::Transition
        )
        .as_deref(),
        Some("opacity 1s, transform 2s linear")
    );
    assert_eq!(
        read(
            &specified_block("animation:fade 1s,spin 2s linear"),
            CssKnownProperty::Animation
        )
        .as_deref(),
        Some("1s fade, 2s linear spin")
    );
}

#[test]
fn inverse_defaults_and_keyword_disambiguation_use_the_owning_grammars() {
    for (source, property, expected) in [
        (
            "transition-property:linear;transition-duration:0ms;transition-timing-function:ease;transition-delay:0s",
            CssKnownProperty::Transition,
            "ease linear",
        ),
        ("animation:none", CssKnownProperty::Animation, "none"),
        (
            "animation-name:reverse;animation-duration:0ms;animation-delay:0s;animation-timing-function:ease;animation-iteration-count:1;animation-direction:normal;animation-fill-mode:none;animation-play-state:running",
            CssKnownProperty::Animation,
            "normal reverse",
        ),
        ("flex:none", CssKnownProperty::Flex, "none"),
        ("flex:auto", CssKnownProperty::Flex, "auto"),
        ("flex:2", CssKnownProperty::Flex, "2"),
        ("flex-flow:column", CssKnownProperty::FlexFlow, "column"),
        ("columns:3", CssKnownProperty::Columns, "3"),
        ("columns:10px", CssKnownProperty::Columns, "10px"),
        ("grid-area:header", CssKnownProperty::GridArea, "header"),
        ("grid-row:header", CssKnownProperty::GridRow, "header"),
        (
            "item-flow:row nowrap",
            CssKnownProperty::ItemFlow,
            "row nowrap",
        ),
        ("text-wrap:wrap auto", CssKnownProperty::TextWrap, "wrap"),
        (
            "text-wrap:wrap balance",
            CssKnownProperty::TextWrap,
            "balance",
        ),
        (
            "text-wrap:nowrap auto",
            CssKnownProperty::TextWrap,
            "nowrap",
        ),
        (
            "white-space:collapse wrap none",
            CssKnownProperty::WhiteSpace,
            "normal",
        ),
        (
            "white-space:preserve nowrap none",
            CssKnownProperty::WhiteSpace,
            "pre",
        ),
        (
            "white-space:preserve wrap none",
            CssKnownProperty::WhiteSpace,
            "pre-wrap",
        ),
        (
            "white-space:preserve-breaks wrap none",
            CssKnownProperty::WhiteSpace,
            "pre-line",
        ),
        (
            "white-space:discard nowrap discard-before",
            CssKnownProperty::WhiteSpace,
            "discard nowrap discard-before",
        ),
    ] {
        assert_eq!(
            read(&specified_block(source), property).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn offset_inverse_omits_initial_path_only_when_its_remaining_grammar_admits_it() {
    for (source, expected) in [
        ("offset:none", "none"),
        ("offset:auto", "auto"),
        ("offset:auto none", "auto"),
        ("offset:auto / left top", "auto / left top"),
        ("offset:none 5px", "none 5px"),
        ("offset:none reverse", "none reverse"),
        ("offset:auto none 5px reverse", "auto none 5px reverse"),
    ] {
        assert_eq!(
            read(&specified_block(source), CssKnownProperty::Offset).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn flex_inverse_compares_default_basis_exactly_without_merging_percentage_or_math_phases() {
    for (source, expected) in [
        ("flex:2 1 0", "2"),
        ("flex:2 1 0.0px", "2"),
        ("flex:2 1 0%", "2 0%"),
        ("flex:2 1 calc(0px)", "2 calc(0px)"),
        ("flex:2 1 1px", "2 1px"),
    ] {
        assert_eq!(
            read(&specified_block(source), CssKnownProperty::Flex).as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn grid_inverse_preserves_symbolic_bare_dense_and_explicit_axis_alternatives() {
    assert_eq!(
        read(
            &specified_block("grid:auto-flow dense 20px / 10px"),
            CssKnownProperty::Grid
        )
        .as_deref(),
        Some("auto-flow dense 20px / 10px")
    );
    assert_eq!(
        read(
            &specified_block("grid:10px / auto-flow dense 20px"),
            CssKnownProperty::Grid
        )
        .as_deref(),
        Some("10px / auto-flow dense 20px")
    );
    let bare = specified_block("grid:auto-flow dense 20px / 10px;grid-auto-flow:dense");
    assert_eq!(read(&bare, CssKnownProperty::Grid), None);
    assert_eq!(
        read(&bare, CssKnownProperty::GridAutoFlow).as_deref(),
        Some("dense")
    );
}

#[test]
fn inverse_math_arena_allocations_pay_their_independent_generated_node_tariff() {
    let block = specified_block("pause-before:calc(1s + 2s);pause-after:calc(1s + 2s)");
    let name = CssPropertyNameRef::Known(CssKnownProperty::Pause);
    // Each semantic calculation visit examines Calc, Sum and two leaves (P4),
    // and actually allocates two scalar nodes and their combined scalar (P3).
    // Complete pair work = eligibility6 + hypothetical19 + check3 + compare38
    // = P66, including three real four-node comparison work lists. Shortened
    // pair output adds I5/P4; the requested read aggregate adds I1/P1.
    let exact = CssSpecifiedValueSerializationLimits::new(6, 71, 8);
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("calc(3s)")
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 71, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 70, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 71, 7),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = block.property_value_with_limits(name, limits).unwrap_err();
        assert_eq!(resource(&error), kind);
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(block.entries()[0].source())
        );
        assert_eq!(error.authored_ordinal(), Some(0));
    }
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("calc(3s)")
    );
}

#[test]
fn complete_pause_read_pays_all_comparison_work_and_owns_final_emission_errors() {
    let list = declarations("pause-before:calc(1s + 2s);pause-after:calc(1s + 2s)");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let name = CssPropertyNameRef::Known(CssKnownProperty::Pause);
    // The independent complete private probe is I5/P70/B8: eligibility6 +
    // hypothetical19 + check3 + verification38 + final provider4. This public
    // request adds its own I1/P1 aggregate, giving I6/P71/B8.
    let exact = CssSpecifiedValueSerializationLimits::new(6, 71, 8);
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("calc(3s)")
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 71, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 70, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 71, 7),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = block.property_value_with_limits(name, limits).unwrap_err();
        assert_eq!(resource(&error), kind);
        // Each one-under reaches the final shortened pair's before value:
        // input visits, the last generated numeric node, or temporary bytes.
        assert!(error.declaration().unwrap().same_occurrence(&list[0]));
        assert_eq!(error.authored_ordinal(), Some(0));
        assert_eq!(error.member_ordinal(), Some(0));
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some("calc(3s)")
        );
    }
    assert!(block.entries()[0].source().same_occurrence(&list[0]));
    assert!(block.entries()[1].source().same_occurrence(&list[1]));
}

#[test]
fn complete_grid_area_reads_own_the_original_last_terminal_in_both_domains() {
    let source = "grid-row-start:calc(1 + 2);grid-column-start:calc(1 + 2);grid-row-end:calc(1 + 2);grid-column-end:calc(1 + 2)";
    let ordinary = declarations(source);
    let keyframes = keyframes(&format!("@keyframes k{{from{{{source}}}}}"));
    let keyframe = keyframes.blocks()[0].declarations();
    let ordinary_block = CssSpecifiedDeclarationBlock::try_from_declarations(&ordinary).unwrap();
    let keyframe_block =
        CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(keyframe).unwrap();
    let keyframe_sources: Vec<_> = keyframe
        .as_slice()
        .iter()
        .map(|value| value.source().clone())
        .collect();
    // The private prefix/verification/output probe is I16/P158/B37. Each of
    // four admitted calculations retains Calc, Sum and two leaves. The public
    // requested-value aggregate adds I1/P1, giving I17/P159/B37. Every one-under
    // reaches the last emitted column-end calculation, whose original source
    // is declaration 3, member 0; no shorthand-authored occurrence exists.
    let exact = CssSpecifiedValueSerializationLimits::new(17, 159, 37);
    let name = CssPropertyNameRef::Known(CssKnownProperty::GridArea);
    let expected = "calc(3) / calc(3) / calc(3) / calc(3)";
    for (block, sources, domain) in [
        (&ordinary_block, ordinary.as_slice(), "ordinary"),
        (&keyframe_block, keyframe_sources.as_slice(), "keyframe"),
    ] {
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected)
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(16, 159, 37),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(17, 158, 37),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(17, 159, 36),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind, "{domain:?} {limits:?}");
            assert!(
                error.declaration().unwrap().same_occurrence(&sources[3]),
                "{domain:?} {limits:?}"
            );
            assert_eq!(error.authored_ordinal(), Some(3), "{domain:?} {limits:?}");
            assert_eq!(error.member_ordinal(), Some(0), "{domain:?} {limits:?}");
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected)
            );
        }
        for (index, entry) in block.entries().iter().enumerate() {
            assert!(entry.source().same_occurrence(&sources[index]));
            assert_eq!(entry.authored_ordinal(), index);
            assert_eq!(entry.member_ordinal(), 0);
        }
    }
}

#[test]
fn reconstructed_owner_orders_attach_byte_exhaustion_to_the_field_being_written() {
    // These stimuli select three distinct owning causes: a retained pair end,
    // an explicit font's final family after grammar-order reordering, and the
    // colliding transition property after its duration and generated easing.
    // Bytes are counted from the independent literals; I/P are unrestricted
    // here so the test isolates final emission provenance rather than work.
    let mut failures = Vec::new();
    for (property, source, expected, bytes) in [
        (
            CssKnownProperty::MarginBlock,
            "margin-block-start:1px;margin-block-end:2px",
            "1px 2px",
            7,
        ),
        (
            CssKnownProperty::Font,
            "font:italic small-caps 700 condensed 16px/2 Alpha;font-family:Beta",
            "italic small-caps 700 condensed 16px/2 Beta",
            43,
        ),
        (
            CssKnownProperty::Transition,
            "transition-duration:1s,2s;transition-property:opacity,ease;transition-delay:0s,0s;transition-timing-function:ease,ease",
            "opacity 1s, 2s ease ease",
            24,
        ),
    ] {
        let list = declarations(source);
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let name = CssPropertyNameRef::Known(property);
        let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{property:?}"
        );
        let under = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes - 1);
        let error = block.property_value_with_limits(name, under).unwrap_err();
        assert_eq!(
            resource(&error),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
            "{property:?}"
        );
        if !error
            .declaration()
            .is_some_and(|value| value.same_occurrence(&list[1]))
            || error.authored_ordinal() != Some(1)
            || error.member_ordinal() != Some(0)
        {
            failures.push(format!(
                "{property:?}: expected original declaration 1/member 0, got {error:?}"
            ));
        }
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{property:?} atomic retry"
        );
        for entry in block.entries() {
            assert!(
                entry
                    .source()
                    .same_occurrence(&list[entry.authored_ordinal()])
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn suppressed_columns_count_keeps_its_original_source_when_input_is_exhausted() {
    let list = declarations("column-width:10px;column-count:auto");
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let name = CssPropertyNameRef::Known(CssKnownProperty::Columns);
    // Public request1 + actual columns aggregate1 + width1 + suppressed count1
    // = I4. The final count visits its owning provider even though CSSOM omits
    // its auto token. No input is charged by the preceding semantic probes.
    let exact = CssSpecifiedValueSerializationLimits::new(4, usize::MAX, 4);
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("10px")
    );
    let error = block
        .property_value_with_limits(
            name,
            CssSpecifiedValueSerializationLimits::new(3, usize::MAX, 4),
        )
        .unwrap_err();
    assert_eq!(
        resource(&error),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert!(error.declaration().unwrap().same_occurrence(&list[1]));
    assert_eq!(error.authored_ordinal(), Some(1));
    assert_eq!(error.member_ordinal(), Some(0));
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("10px")
    );
    for (index, entry) in block.entries().iter().enumerate() {
        assert!(entry.source().same_occurrence(&list[index]));
        assert_eq!(entry.authored_ordinal(), index);
        assert_eq!(entry.member_ordinal(), 0);
    }
}

#[test]
fn retained_unequal_time_reads_pay_the_emission_comparison_and_own_second_value_errors() {
    // The private retained-pair probe admits I9/P77/B17, including the owning
    // emitter's existing second comparison. The public request adds I1/P1.
    let exact = CssSpecifiedValueSerializationLimits::new(10, 78, 17);
    for (property, source) in [
        (
            CssKnownProperty::Pause,
            "pause-before:calc(1s + 2s);pause-after:calc(1s + 3s)",
        ),
        (
            CssKnownProperty::Rest,
            "rest-before:calc(1s + 2s);rest-after:calc(1s + 3s)",
        ),
        (
            CssKnownProperty::InterestDelay,
            "interest-delay-start:calc(1s + 2s);interest-delay-end:calc(1s + 3s)",
        ),
    ] {
        let list = declarations(source);
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let name = CssPropertyNameRef::Known(property);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some("calc(3s) calc(4s)")
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(9, 78, 17),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(10, 77, 17),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(10, 78, 16),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind);
            // All three boundaries reach the retained second value: its final
            // input visit, last generated arena node, or captured output byte.
            assert!(error.declaration().unwrap().same_occurrence(&list[1]));
            assert_eq!(error.authored_ordinal(), Some(1));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some("calc(3s) calc(4s)")
            );
        }
        for (entry, source) in block.entries().iter().zip(list.iter()) {
            assert!(entry.source().same_occurrence(source));
        }
    }
}

#[test]
fn ordinary_time_pair_providers_keep_their_own_comparison_and_projection_tariffs() {
    // The ordinary provider retains its existing comparator: aggregate I1/P1
    // plus two four-node input graphs and three-node arenas gives I9/P7.
    // An explicitly equal second still visits its graph under output suppression.
    for (source, expected, bytes) in [
        ("pause:calc(1s + 2s) calc(1s + 3s)", "calc(3s) calc(4s)", 17),
        ("rest:calc(1s + 2s) calc(1s + 3s)", "calc(3s) calc(4s)", 17),
        (
            "interest-delay:calc(1s + 2s) calc(1s + 3s)",
            "calc(3s) calc(4s)",
            17,
        ),
        ("pause:calc(1s + 2s) calc(1s + 2s)", "calc(3s)", 8),
        ("rest:calc(1s + 2s) calc(1s + 2s)", "calc(3s)", 8),
        ("interest-delay:calc(1s + 2s) calc(1s + 2s)", "calc(3s)", 8),
    ] {
        let list = declarations(source);
        let value = list[0].known().unwrap().property_value().unwrap();
        let render = |limits| match &value {
            CssKnownPropertyValueRef::Pause(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            CssKnownPropertyValueRef::Rest(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            CssKnownPropertyValueRef::InterestDelay(value) => {
                value.delay().serialize_specified_with_limits(limits)
            }
            _ => panic!("ordinary owning time pair"),
        };
        let exact = CssSpecifiedValueSerializationLimits::new(9, 7, bytes);
        assert_eq!(render(exact).unwrap(), expected, "{source}");
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(8, 7, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(9, 6, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(9, 7, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(render(limits).unwrap_err().kind(), kind, "{source}");
            assert_eq!(render(exact).unwrap(), expected, "{source}");
        }
    }
}

#[test]
fn ordinary_numeric_owner_keeps_its_existing_math_and_keyword_tariffs() {
    let calculation =
        CssTimeCalculation::try_from_components(parse_component_values("calc(1s + 2s)").unwrap())
            .unwrap();
    let value = CssTimeValue::try_from_calculation(calculation).unwrap();
    let exact = CssSpecifiedValueSerializationLimits::new(4, 3, 8);
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        "calc(3s)"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 2, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, 7),
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
    let keyword = CssOverflow::Hidden;
    assert_eq!(
        keyword
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "hidden"
    );
}

#[test]
fn color_inverse_omits_exact_ordinary_alpha_and_ignores_parsed_coordinates() {
    for (left, right, expected) in [
        ("rgb(1 2 3)", "rgb(1 2 3 / 1)", "rgb(1, 2, 3)"),
        ("rgb(1 2 3)", "rgba(1, 2, 3, 100%)", "rgb(1, 2, 3)"),
        ("rgb(1 2 3)", "rgb(1.00 2.0 3.000 / 120%)", "rgb(1, 2, 3)"),
        ("rgb(1 2 3 / .5)", "rgb(1 2 3 / 50%)", "rgba(1, 2, 3, 0.5)"),
        (
            "color(srgb 1 0 0)",
            "color(srgb 1 0 0 / 2)",
            "color(srgb 1 0 0)",
        ),
        (
            "rgb(calc(1 + 1) 2 3)",
            "rgb(calc(1 + 1) 2 3)",
            "rgb(2, 2, 3)",
        ),
    ] {
        let list = declarations(&format!(
            "border-block-start-color:{left};border-block-end-color:{right}"
        ));
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        assert_eq!(
            read(&block, CssKnownProperty::BorderBlockColor).as_deref(),
            Some(expected),
            "{left} / {right}"
        );
        assert_eq!(list, before);
        assert!(!list[0].same_occurrence(&list[1]));
        assert!(block.entries()[0].source().same_occurrence(&list[0]));
        assert!(block.entries()[1].source().same_occurrence(&list[1]));
    }
    assert_eq!(
        read(
            &specified_block(
                "border-color:rgb(1 2 3) rgb(1 2 3 / 1) rgb(1 2 3 / 100%) rgba(1,2,3,2)"
            ),
            CssKnownProperty::BorderColor
        )
        .as_deref(),
        Some("rgb(1, 2, 3)")
    );
    assert_eq!(
        read(
            &specified_block("border-inline-color:rgb(1 2 3) rgb(1 2 3 / 100%)"),
            CssKnownProperty::BorderInlineColor
        )
        .as_deref(),
        Some("rgb(1, 2, 3)")
    );
}

#[test]
fn color_inverse_keeps_representation_profiles_bindings_and_math_phases() {
    for (left, right, expected) in [
        ("red", "rgb(255 0 0)", "red rgb(255, 0, 0)"),
        (
            "color(srgb 1 0 0)",
            "color(display-p3 1 0 0)",
            "color(srgb 1 0 0) color(display-p3 1 0 0)",
        ),
        (
            "color(--P 1 0 0)",
            "color(--p 1 0 0)",
            "color(--P 1 0 0) color(--p 1 0 0)",
        ),
        (
            "rgb(calc(1) 2 3)",
            "rgb(1 2 3)",
            "rgb(1, 2, 3) rgb(1, 2, 3)",
        ),
        (
            "color(srgb 1 0 0 / .9999996)",
            "color(srgb 1 0 0 / 1)",
            "color(srgb 1 0 0 / 1) color(srgb 1 0 0)",
        ),
        (
            "alpha(from rgb(1 2 3))",
            "alpha(from rgb(1 2 3 / 1))",
            "alpha(from rgb(1 2 3)) alpha(from rgb(1 2 3 / 1))",
        ),
        (
            "alpha(from rgb(1 2 3 / 2))",
            "alpha(from rgb(1 2 3 / 1))",
            "alpha(from rgb(1 2 3 / 2)) alpha(from rgb(1 2 3 / 1))",
        ),
        (
            "rgb(from rgb(1 2 3 / .2) r g b)",
            "rgb(from rgb(1 2 3 / .2) r g b / 1)",
            "rgb(from rgb(1 2 3 / 0.2) r g b) rgb(from rgb(1 2 3 / 0.2) r g b / 1)",
        ),
        (
            "color(from red --P Cyan)",
            "color(from red --P cyan)",
            "color(from red --P Cyan) color(from red --P cyan)",
        ),
    ] {
        assert_eq!(
            read(
                &specified_block(&format!(
                    "border-block-start-color:{left};border-block-end-color:{right}"
                )),
                CssKnownProperty::BorderBlockColor
            )
            .as_deref(),
            Some(expected),
            "{left} / {right}"
        );
    }
}

#[test]
fn every_supported_color_representation_reconstructs_from_independent_terminal_occurrences() {
    // Values4 WD20240312 §10.13: Product serialization sorts Number first,
    // then percentage/dimensions, then symbolic operands. Thus magenta * 2
    // canonically writes 2 * magenta, without resolving profile binding.
    for (source, expected) in [
        ("currentcolor", "currentcolor"),
        ("transparent", "transparent"),
        ("#abc", "rgb(170, 187, 204)"),
        ("red", "red"),
        ("canvas", "canvas"),
        ("rgb(1 2 3)", "rgb(1, 2, 3)"),
        ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
        ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
        ("lab(50% 20 -30 / .5)", "lab(50 20 -30 / 0.5)"),
        ("lch(50% 20 30deg)", "lch(50 20 30)"),
        ("oklab(.5 .1 -.1)", "oklab(0.5 0.1 -0.1)"),
        ("oklch(.5 .1 30)", "oklch(0.5 0.1 30)"),
        ("color(srgb 1 0 0)", "color(srgb 1 0 0)"),
        ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
        (
            "color(from red --P Cyan calc(magenta * 2) / alpha)",
            "color(from red --P Cyan calc(2 * magenta) / alpha)",
        ),
        ("alpha(from red / .5)", "alpha(from red / 0.5)"),
        ("rgb(from red r g b / alpha)", "rgb(from red r g b / alpha)"),
        ("color-mix(red, blue)", "color-mix(red, blue)"),
        ("light-dark(red,blue)", "light-dark(red, blue)"),
        ("contrast-color(red)", "contrast-color(red)"),
        (
            "device-cmyk(0% 81% 81% 30%)",
            "device-cmyk(0 0.81 0.81 0.3)",
        ),
    ] {
        let source = format!("border-block-start-color:{source};border-block-end-color:{source}");
        assert_eq!(
            read(
                &specified_block(&source),
                CssKnownProperty::BorderBlockColor
            )
            .as_deref(),
            Some(expected),
            "{source}"
        );
    }
}

#[test]
fn color_mix_inverse_compares_exact_effective_declared_shares_and_default_interpolation() {
    for (left, right, expected) in [
        (
            "color-mix(red, blue)",
            "color-mix(in oklab, red 50%, blue 50%)",
            "color-mix(red, blue)",
        ),
        (
            "color-mix(in lch, red, blue)",
            "color-mix(in lch shorter hue, red 50.0%, blue 50%)",
            "color-mix(in lch, red, blue)",
        ),
        (
            "color-mix(red 50%, green, blue)",
            "color-mix(red 50%, green 25%, blue 25%)",
            "color-mix(red 50%, green 25%, blue 25%)",
        ),
        (
            "color-mix(red 70%, blue 70%, green)",
            "color-mix(red 70%, blue 70%, green 0%)",
            "color-mix(red 70%, blue 70%, green 0%)",
        ),
        (
            "color-mix(red calc(50%), blue)",
            "color-mix(red calc(50%), blue)",
            "color-mix(red calc(50%), blue)",
        ),
        (
            "color-mix(in --P, red, blue)",
            "color-mix(in --P, red 50%, blue 50%)",
            "color-mix(in --P, red, blue)",
        ),
    ] {
        assert_eq!(
            read(
                &specified_block(&format!(
                    "border-block-start-color:{left};border-block-end-color:{right}"
                )),
                CssKnownProperty::BorderBlockColor
            )
            .as_deref(),
            Some(expected),
            "{left} / {right}"
        );
    }
    for (left, right, expected) in [
        (
            "color-mix(red, blue)",
            "color-mix(red 70%, blue 70%)",
            "color-mix(red, blue) color-mix(red 70%, blue 70%)",
        ),
        (
            "color-mix(red 60%, green, blue, yellow)",
            "color-mix(red 60%, green 13.333333%, blue 13.333333%, yellow 13.333333%)",
            "color-mix(red 60%, green 13.333333%, blue 13.333333%, yellow 13.333333%) color-mix(red 60%, green 13.333333%, blue 13.333333%, yellow 13.333333%)",
        ),
        (
            "color-mix(red, blue)",
            "color-mix(red calc(50%), blue calc(50%))",
            "color-mix(red, blue) color-mix(red calc(50%), blue calc(50%))",
        ),
        (
            "color-mix(red, blue)",
            "color-mix(blue, red)",
            "color-mix(red, blue) color-mix(blue, red)",
        ),
        (
            "color-mix(in --P, red, blue)",
            "color-mix(in --p, red, blue)",
            "color-mix(in --P, red, blue) color-mix(in --p, red, blue)",
        ),
    ] {
        assert_eq!(
            read(
                &specified_block(&format!(
                    "border-block-start-color:{left};border-block-end-color:{right}"
                )),
                CssKnownProperty::BorderBlockColor
            )
            .as_deref(),
            Some(expected),
            "{left} / {right}"
        );
    }
}

#[test]
fn ordinary_color_owner_keeps_its_existing_exact_coefficient_tariff() {
    let report = parse_style_attribute("color:color(srgb 1 0 0)");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("color payload")
    };
    let exact = CssSpecifiedValueSerializationLimits::new(4, 4, 17);
    assert_eq!(
        value.value().to_specified_css_with_limits(exact).unwrap(),
        "color(srgb 1 0 0)"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, 17),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, 17),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, 16),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .value()
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn second_terminal_probe_errors_retain_the_actual_source_member_and_allow_retry() {
    let list = declarations(
        "border-block-start-color:currentcolor;border-block-end-color:color(srgb 1 0 0)",
    );
    let ordinary = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
    let parsed = keyframes(
        "@keyframes x{from{border-block-start-color:currentcolor;border-block-end-color:color(srgb 1 0 0)}}",
    );
    let keyframe = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
        parsed.blocks()[0].declarations(),
    )
    .unwrap();
    for block in [&ordinary, &keyframe] {
        let before = block.entries().to_vec();
        // Read aggregate P1 + eligibility P6 + hypothetical aggregate P1 +
        // first keyword visit P1 + second Color root P1 = P10. The next
        // second-terminal channel visit fails before coefficient construction.
        for limits in [
            CssSpecifiedValueSerializationLimits::new(1, 10, 0), // Original witness unchanged.
            CssSpecifiedValueSerializationLimits::new(1, 3, 0),  // Second presence check.
            CssSpecifiedValueSerializationLimits::new(1, 5, 0),  // Second importance check.
            CssSpecifiedValueSerializationLimits::new(1, 25, 0), // Original right value visit.
            CssSpecifiedValueSerializationLimits::new(1, 32, 0), // Generated right value visit.
            CssSpecifiedValueSerializationLimits::new(1, 39, 0), // Right exact comparison.
            CssSpecifiedValueSerializationLimits::new(usize::MAX, 46, 30), // Actual second color output.
        ] {
            let error = block
                .property_value_with_limits(
                    CssPropertyNameRef::Known(CssKnownProperty::BorderBlockColor),
                    limits,
                )
                .unwrap_err();
            assert_eq!(
                resource(&error),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            assert_eq!(error.authored_ordinal(), Some(1), "{limits:?}");
            assert_eq!(error.member_ordinal(), Some(0));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(block.entries()[1].source())
            );
        }
        assert_eq!(
            read(block, CssKnownProperty::BorderBlockColor).as_deref(),
            Some("currentcolor color(srgb 1 0 0)")
        );
        assert_eq!(block.entries().len(), before.len());
        for (entry, saved) in block.entries().iter().zip(before) {
            assert!(entry.source().same_occurrence(saved.source()));
            assert_eq!(entry.authored_ordinal(), saved.authored_ordinal());
            assert_eq!(entry.member_ordinal(), saved.member_ordinal());
        }
    }
}

#[test]
fn exact_color_coefficient_probe_and_output_share_the_whole_new_request_budget() {
    let block = specified_block(
        "border-block-start-color:color(srgb 1 0 0);border-block-end-color:color(srgb 1 0 0)",
    );
    let name = CssPropertyNameRef::Known(CssKnownProperty::BorderBlockColor);
    // Color semantic visit P7 = four nodes + three real coefficient limbs.
    // Eligibility6 + hypothetical15 + Color pair equality5 + check3 + two
    // comparison initiations with both visits and Color equality (2 * 20)
    // = P69. Actual pair output I5/P5; read aggregate I1/P1 => I6/P75/B17.
    let exact = CssSpecifiedValueSerializationLimits::new(6, 75, 17);
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("color(srgb 1 0 0)")
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 75, 17),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 74, 17),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 75, 16),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = block.property_value_with_limits(name, limits).unwrap_err();
        assert_eq!(resource(&error), expected);
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(block.entries()[0].source())
        );
        assert_eq!(error.authored_ordinal(), Some(0));
        assert_eq!(error.member_ordinal(), Some(0));
    }
    assert_eq!(
        block
            .property_value_with_limits(name, exact)
            .unwrap()
            .as_deref(),
        Some("color(srgb 1 0 0)")
    );
}

#[test]
fn color_pair_and_quad_owners_keep_existing_ordering_and_tariffs() {
    for property in ["border-block-color", "border-color"] {
        let list = declarations(&format!("{property}:currentcolor color(srgb 1 0 0)"));
        let serialize = |limits| match list[0].known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::BorderBlockColor(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            CssKnownPropertyValueRef::BorderColor(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            _ => panic!("color shorthand"),
        };
        // Aggregate I1/P1 + keyword Color I1/P1 + predefined Color I4/P4.
        let exact = CssSpecifiedValueSerializationLimits::new(6, 6, 30);
        assert_eq!(serialize(exact).unwrap(), "currentcolor color(srgb 1 0 0)");
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(5, 6, 30),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(6, 5, 30),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(6, 6, 29),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialize(limits).unwrap_err().kind(),
                expected,
                "{property}"
            );
        }
    }
}

#[test]
fn reconstructed_capture_layers_and_nested_shapes_keep_original_terminal_sources() {
    // Distinct owner causes: capture-then-append, two box fields in one layer,
    // and a radius inside a basic shape whose whole terminal is OffsetPath.
    // Byte counts are independent literals; I/P are unrestricted to isolate
    // the final append phase from eligibility and verification work.
    for (property, source, expected, bytes, ordinal) in [
        (
            CssKnownProperty::ScrollPaddingBlock,
            "scroll-padding-block-start:1px;scroll-padding-block-end:2px",
            "1px 2px",
            7,
            1,
        ),
        (
            CssKnownProperty::Background,
            "background:none;background-origin:padding-box;background-clip:content-box",
            "padding-box content-box",
            23,
            2,
        ),
        (
            CssKnownProperty::Mask,
            "mask:none;mask-origin:padding-box;mask-clip:no-clip",
            "padding-box no-clip",
            19,
            2,
        ),
        (
            CssKnownProperty::Offset,
            "offset:none;offset-path:inset(1px round 2px)",
            "inset(1px round 2px)",
            20,
            1,
        ),
    ] {
        let list = declarations(source);
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let name = CssPropertyNameRef::Known(property);
        let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{property:?}"
        );
        let under = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes - 1);
        let error = block.property_value_with_limits(name, under).unwrap_err();
        assert_eq!(
            resource(&error),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
            "{property:?}"
        );
        assert!(
            error.declaration().unwrap().same_occurrence(&list[ordinal]),
            "{property:?} {error:?}"
        );
        assert_eq!(error.authored_ordinal(), Some(ordinal), "{property:?}");
        assert_eq!(error.member_ordinal(), Some(0), "{property:?}");
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{property:?} atomic retry"
        );
        assert_eq!(list, before);
        for entry in block.entries() {
            assert!(
                entry
                    .source()
                    .same_occurrence(&list[entry.authored_ordinal()])
            );
        }
    }
}

#[test]
fn grid_templates_reorder_real_area_and_auto_flow_fields_without_replacing_sources() {
    // Public request1 + template aggregate1 + two track lists/leaves2 each =
    // I6. An area form has aggregate1 + area row/cell2 + row-size1, giving I5.
    // Dense auto-flow has aggregate1 + flow1 + dense1 + two list/leaves2 each,
    // giving I8. Probe input is suppressed, so each I-1 reaches the last real
    // emitted field; P is unrestricted here and bytes come from the literals.
    for (property, source, expected, inputs, bytes, ordinal) in [
        (
            CssKnownProperty::GridTemplate,
            "grid-template-rows:1px;grid-template-columns:2px;grid-template-areas:none",
            "1px / 2px",
            6,
            9,
            1,
        ),
        (
            CssKnownProperty::GridTemplate,
            "grid-template-rows:1px;grid-template-columns:none;grid-template-areas:\"a\"",
            "\"a\" 1px",
            5,
            7,
            0,
        ),
        (
            CssKnownProperty::Grid,
            "grid-template-rows:none;grid-template-columns:2px;grid-template-areas:none;grid-auto-rows:1px;grid-auto-columns:auto;grid-auto-flow:row dense",
            "auto-flow dense 1px / 2px",
            8,
            25,
            1,
        ),
        (
            CssKnownProperty::Grid,
            "grid-template-rows:1px;grid-template-columns:none;grid-template-areas:none;grid-auto-rows:auto;grid-auto-columns:2px;grid-auto-flow:column dense",
            "1px / auto-flow dense 2px",
            8,
            25,
            4,
        ),
    ] {
        let list = declarations(source);
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let name = CssPropertyNameRef::Known(property);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, usize::MAX, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{source}"
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, usize::MAX, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, usize::MAX, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind, "{source} {limits:?}");
            assert!(
                error.declaration().unwrap().same_occurrence(&list[ordinal]),
                "{source} {limits:?} {error:?}"
            );
            assert_eq!(
                error.authored_ordinal(),
                Some(ordinal),
                "{source} {limits:?}"
            );
            assert_eq!(error.member_ordinal(), Some(0), "{source} {limits:?}");
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "{source} atomic retry"
            );
        }
        for entry in block.entries() {
            assert!(
                entry
                    .source()
                    .same_occurrence(&list[entry.authored_ordinal()])
            );
        }
    }
}

#[test]
fn nonauto_flex_verification_pays_its_complete_basis_comparison_work() {
    // Three original terminals give eligibility P9: enter1, presence3,
    // importance3, intervening entry1, probe1. LP hypothetical work is P10
    // (aggregate1 + grow1 + shrink1 + four source/three arena nodes7), check
    // expansion P4, and verification P25 (grow3 + shrink3 + basis19, including
    // four comparison slots). Final emission I6/P5 gives private P53; this
    // public request adds I1/P1: I7/P54/B11. Fit-content and CalcSize each have
    // hypothetical11 + check4 + verification27 + final6 + eligibility9, giving
    // private P57/public P58 and I8. Their independently counted bytes differ.
    let mut failures = Vec::new();
    for (basis, expected, inputs, projections, bytes) in [
        ("calc(1px + 2px)", "1 calc(3px)", 7, 54, 11),
        (
            "fit-content(calc(1px + 2px))",
            "1 fit-content(calc(3px))",
            8,
            58,
            24,
        ),
        (
            "calc-size(any, 1px + 2px)",
            "1 calc-size(any, 3px)",
            8,
            58,
            21,
        ),
    ] {
        let list = declarations(&format!("flex-grow:1;flex-shrink:1;flex-basis:{basis}"));
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let name = CssPropertyNameRef::Known(CssKnownProperty::Flex);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{basis}"
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            match block.property_value_with_limits(name, limits) {
                Err(error) => {
                    assert_eq!(resource(&error), kind, "{basis} {limits:?}");
                    // Every one-under reaches actual final basis work, never
                    // the omitted shrink or a fabricated shorthand occurrence.
                    assert!(
                        error.declaration().unwrap().same_occurrence(&list[2]),
                        "{basis} {limits:?} {error:?}"
                    );
                    assert_eq!(error.authored_ordinal(), Some(2), "{basis} {limits:?}");
                    assert_eq!(error.member_ordinal(), Some(0), "{basis} {limits:?}");
                }
                actual => failures.push(format!(
                    "{basis}: one-under {kind:?} at {limits:?} returned {actual:?}"
                )),
            }
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "{basis}: adequate atomic retry"
            );
        }
        assert_eq!(list, before);
        for (index, entry) in block.entries().iter().enumerate() {
            assert!(entry.source().same_occurrence(&list[index]));
            assert_eq!(entry.authored_ordinal(), index);
            assert_eq!(entry.member_ordinal(), 0);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn ordinary_flex_basis_writers_keep_numeric_and_outer_function_tariffs() {
    // LP: four source visits, three arena nodes. Fit-content adds one I/P.
    // CalcSize: owner input1 (no projection fee), Any input/projection1,
    // and bare Sum/two Values input3 with three generated arena nodes.
    for (source, expected, inputs, projections, bytes) in [
        ("flex-basis:calc(1px + 2px)", "calc(3px)", 4, 3, 9),
        (
            "flex-basis:fit-content(calc(1px + 2px))",
            "fit-content(calc(3px))",
            5,
            4,
            22,
        ),
        (
            "flex-basis:calc-size(any, 1px + 2px)",
            "calc-size(any, 3px)",
            5,
            4,
            19,
        ),
    ] {
        let list = declarations(source);
        let CssKnownPropertyValueRef::FlexBasis(wrapper) =
            list[0].known().unwrap().property_value().unwrap()
        else {
            panic!("ordinary flex basis")
        };
        let value = wrapper.value();
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected,
            "{source}"
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind,
                "{source} {limits:?}"
            );
            assert_eq!(
                value.serialize_specified_with_limits(exact).unwrap(),
                expected,
                "{source}: adequate retry"
            );
        }
    }
}

#[test]
fn mask_position_omission_preserves_structure_math_phase_layers_and_final_sources() {
    // Input/projection maxima isolate actual captured-byte emission sources.
    // Initial comparison has no graph work when one operand is literal; these
    // controls do not manufacture a runtime allocation tariff for that branch.
    for (source, expected, bytes, authored, member) in [
        ("mask:none;mask-position:0% 0%", "none", 4, 0, 0),
        ("mask:none;mask-position:0.0% 0%", "0% 0%", 5, 1, 0),
        ("mask:none;mask-position:10% 20%", "10% 20%", 7, 1, 0),
        ("mask:none;mask-position:left top", "left top", 8, 1, 0),
        (
            "mask:none;mask-position:calc(1px + 2px) 0%",
            "calc(3px) 0%",
            12,
            1,
            0,
        ),
        (
            "mask:none;mask-position:0% calc(1px + 2px)",
            "0% calc(3px)",
            12,
            1,
            0,
        ),
        (
            "mask:none,none;mask-position:10% 20%,0% 0%",
            "10% 20%, none",
            13,
            0,
            0,
        ),
    ] {
        let declarations = declarations(source);
        let before = declarations.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&declarations).unwrap();
        let name = CssPropertyNameRef::Known(CssKnownProperty::Mask);
        let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{source}"
        );
        let under = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, bytes - 1);
        let error = block.property_value_with_limits(name, under).unwrap_err();
        assert_eq!(
            resource(&error),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
            "{source}"
        );
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(&declarations[authored]),
            "{source}: {error:?}"
        );
        assert_eq!(error.authored_ordinal(), Some(authored), "{source}");
        assert_eq!(error.member_ordinal(), Some(member), "{source}");
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{source}: adequate atomic retry"
        );
        assert_eq!(declarations, before);
        for entry in block.entries() {
            assert!(
                entry
                    .source()
                    .same_occurrence(&declarations[entry.authored_ordinal()])
            );
        }
    }
}

fn svg_context() -> CssParserContext {
    CssParserContext::new(CssParserMode::Standards).with_svg_glyph_orientation_vertical()
}

fn all_source_block(
    context: CssParserContext,
    keyframe: bool,
    value: &str,
) -> (CssDeclaration, CssSpecifiedDeclarationBlock) {
    if keyframe {
        let report = context.parse_sheet(&format!("@keyframes k{{from{{all:{value}}}}}"));
        assert!(report.is_clean());
        let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
            panic!("keyframes")
        };
        let list = rule.blocks()[0].declarations();
        let source = list[0].source().clone();
        let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap();
        (source, block)
    } else {
        let report = context.parse_style_attribute(&format!("all:{value}"));
        assert!(report.is_clean());
        let source = report.syntax()[0].clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
        (source, block)
    }
}

#[test]
fn explicit_svg_terminal_winners_and_legacy_alias_keep_distinct_semantic_owners() {
    let report = svg_context().parse_style_attribute("glyph-orientation-vertical:180deg!important;glyph-orientation-vertical:90deg;text-orientation:upright");
    assert!(report.is_clean());
    let list = report.syntax();
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(list).unwrap();
    let name = CssPropertyNameRef::SvgGlyphOrientationVertical;
    assert_eq!(block.entries().len(), 2);
    let entry = &block.entries()[0];
    assert_eq!(entry.property_name(), name);
    assert_eq!(entry.importance(), CssImportance::Important);
    assert_eq!(entry.authored_ordinal(), 0);
    assert_eq!(entry.member_ordinal(), 0);
    assert!(entry.source().same_occurrence(&list[0]));
    assert!(!list[0].is_name_case_sensitive());
    let CssSpecifiedDeclarationValueRef::SvgGlyphOrientationVertical(value) = entry.value() else {
        panic!("actual SVG contribution")
    };
    assert_eq!(value.property(), name);
    assert!(value.source().same_occurrence(&list[0]));
    assert!(value.replacement_components().is_none());
    assert_eq!(
        block.property_value(name).unwrap().as_deref(),
        Some("180deg")
    );
    assert_eq!(
        read(&block, CssKnownProperty::TextOrientation).as_deref(),
        Some("upright")
    );
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: 180deg !important; text-orientation: upright;"
    );

    let explicit = parse_property_value(
        name,
        parse_component_values("180deg").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(explicit.svg_glyph_orientation_vertical().is_some());
    assert_eq!(explicit.position(), None);
    let legacy = declarations("glyph-orientation-vertical:0")[0].clone();
    assert!(legacy.svg_glyph_orientation_vertical().is_none());
    let expansions = [
        expand_declaration(&explicit).unwrap(),
        expand_declaration(&legacy).unwrap(),
    ];
    let combined = CssSpecifiedDeclarationBlock::try_from_expansions(&expansions).unwrap();
    assert_eq!(
        combined
            .entries()
            .iter()
            .map(CssSpecifiedDeclarationEntry::property_name)
            .collect::<Vec<_>>(),
        [
            name,
            CssPropertyNameRef::Known(CssKnownProperty::TextOrientation)
        ]
    );
    assert_eq!(
        combined.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: 180deg; text-orientation: upright;"
    );
    assert!(combined.entries()[0].source().same_occurrence(&explicit));
    assert!(combined.entries()[1].source().same_occurrence(&legacy));
}

#[test]
fn svg_keyframe_accessors_borrow_the_actual_retained_body_and_normal_occurrence() {
    let report = svg_context().parse_sheet(
        "@keyframes k{from{glyph-orientation-vertical:180deg;glyph-orientation-vertical:90deg}}",
    );
    assert!(report.is_clean());
    let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
        panic!("keyframes")
    };
    let list = rule.blocks()[0].declarations();
    assert_eq!(list.len(), 2);
    for declaration in list.iter() {
        let source = declaration.source();
        assert_eq!(
            source.property_name(),
            CssPropertyNameRef::SvgGlyphOrientationVertical
        );
        assert!(std::ptr::eq(
            declaration.svg_glyph_orientation_vertical().unwrap(),
            source.svg_glyph_orientation_vertical().unwrap()
        ));
        assert!(std::ptr::eq(declaration.body(), source.body()));
        assert!(declaration.known().is_none() && declaration.custom().is_none());
        assert_eq!(source.importance(), CssImportance::Normal);
        assert_eq!(source.position(), Some(declaration.position()));
        assert!(source.parsed_name().is_some() && source.parsed_value().is_some());
        assert!(declaration.clone().source().same_occurrence(source));
    }
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap();
    assert_eq!(block.entries().len(), 1);
    assert!(
        block.entries()[0]
            .source()
            .same_occurrence(list[1].source())
    );
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: 90deg;"
    );
    assert_eq!(
        list.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: 90deg;"
    );
}

#[test]
fn svg_direct_pending_and_completed_reentry_keep_original_and_replacement_origins() {
    let report = svg_context().parse_style_attribute("glyph-orientation-vertical:var(--G)");
    assert!(report.is_clean());
    let original = report.syntax()[0].clone();
    let name = CssPropertyNameRef::SvgGlyphOrientationVertical;
    let CssExpansion::Pending(pending) = expand_declaration(&original).unwrap() else {
        panic!("SVG pending")
    };
    let pending_block =
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(pending_block.entries().len(), 1);
    assert_eq!(
        pending_block.property_value(name).unwrap().as_deref(),
        Some("var(--G)")
    );
    assert!(matches!(
        pending_block.entries()[0].value(),
        CssSpecifiedDeclarationValueRef::UnresolvedLonghand(_)
    ));
    assert_eq!(
        pending_block.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: var(--G);"
    );
    assert_eq!(
        original.to_specified_css().unwrap(),
        "glyph-orientation-vertical: var(--G);"
    );
    let replacement = parse_component_values("45deg").unwrap();
    let completed = CssExpansion::Contributions(pending.reenter(replacement.clone()).unwrap());
    for block in [
        CssSpecifiedDeclarationBlock::try_from_expansions(std::slice::from_ref(&completed))
            .unwrap(),
        CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(std::slice::from_ref(
            &completed,
        ))
        .unwrap(),
    ] {
        assert_eq!(
            block.property_value(name).unwrap().as_deref(),
            Some("45deg")
        );
        let entry = &block.entries()[0];
        assert!(entry.source().same_occurrence(&original));
        assert_eq!(entry.authored_ordinal(), 0);
        assert_eq!(entry.member_ordinal(), 0);
        let CssSpecifiedDeclarationValueRef::SvgGlyphOrientationVertical(value) = entry.value()
        else {
            panic!("completed SVG owner")
        };
        assert!(value.source().same_occurrence(&original));
        assert_eq!(value.replacement_components(), Some(&replacement));
    }
    assert!(
        pending
            .reenter(parse_component_values("var(--G)").unwrap())
            .is_err()
    );
    assert!(pending.source().same_occurrence(&original));
    assert_eq!(
        pending.source().to_specified_css().unwrap(),
        "glyph-orientation-vertical: var(--G);"
    );
    pending.reenter(replacement).unwrap();

    // The explicit SVG name has SVG semantics even with the default finite
    // alias lookup selector. Its Normal constructed source has no position.
    let explicit = parse_property_value(
        name,
        parse_component_values("var(--G)").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(explicit.parser_context(), CssParserContext::default());
    assert_eq!(explicit.position(), None);
    let CssExpansion::Pending(pending) = expand_declaration(&explicit).unwrap() else {
        panic!("explicit SVG pending")
    };
    let complete = CssExpansion::Contributions(
        pending
            .reenter(parse_component_values("45deg").unwrap())
            .unwrap(),
    );
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[complete]).unwrap();
    assert_eq!(
        block.property_value(name).unwrap().as_deref(),
        Some("45deg")
    );
    assert!(block.entries()[0].source().same_occurrence(&explicit));
    assert_eq!(block.entries()[0].source().position(), None);

    let report =
        svg_context().parse_sheet("@keyframes k{from{glyph-orientation-vertical:var(--G)}}");
    assert!(report.is_clean());
    let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
        panic!("keyframes")
    };
    let list = rule.blocks()[0].declarations();
    let original = list[0].source();
    assert_eq!(
        list.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: var(--G);"
    );
    let CssExpansion::Pending(pending) = expand_declaration(original).unwrap() else {
        panic!("keyframe SVG pending")
    };
    let completed = CssExpansion::Contributions(
        pending
            .reenter(parse_component_values("90deg").unwrap())
            .unwrap(),
    );
    let block = CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[completed]).unwrap();
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: 90deg;"
    );
    assert!(block.entries()[0].source().same_occurrence(original));
    assert_eq!(
        list.serialize_cssom().unwrap(),
        "glyph-orientation-vertical: var(--G);"
    );
}

#[test]
fn all_supported_union_ignores_lookup_selection_and_preserves_complete_pending_identity() {
    let expected = all_terminal_expectations();
    let svg_name = CssPropertyNameRef::SvgGlyphOrientationVertical;
    let metadata = CssSvgGlyphOrientationVerticalDeclaration::metadata();
    let row = INDEPENDENT_TERMINALS
        .iter()
        .find(|row| row.property == svg_name)
        .unwrap();
    assert_eq!(metadata.property(), row.property);
    assert_eq!(metadata.name(), row.name);
    assert_eq!(metadata.settable_members(), &[row.property]);
    assert!(metadata.reset_only_members().is_empty());
    assert!(metadata.initial_value().is_auto());
    assert_eq!(row.mapping.bucket(), 1);
    for context in [CssParserContext::default(), svg_context()] {
        for keyframe in [false, true] {
            let admitted: Vec<_> = expected
                .iter()
                .enumerate()
                .filter(|(_, (name, _, _))| !keyframe || keyframe_terminal_expected(*name))
                .collect();
            for (value, svg_value) in [("inherit", "inherit"), ("var(--A)", "")] {
                let (source, block) = all_source_block(context, keyframe, value);
                assert_eq!(
                    block
                        .entries()
                        .iter()
                        .map(CssSpecifiedDeclarationEntry::property_name)
                        .collect::<Vec<_>>(),
                    admitted
                        .iter()
                        .map(|(_, (name, _, _))| *name)
                        .collect::<Vec<_>>()
                );
                for (entry, (member, _)) in block.entries().iter().zip(&admitted) {
                    assert!(entry.source().same_occurrence(&source));
                    assert_eq!(entry.authored_ordinal(), 0);
                    assert_eq!(entry.member_ordinal(), *member);
                }
                let svg = block
                    .entries()
                    .iter()
                    .find(|entry| entry.property_name() == svg_name)
                    .unwrap();
                assert!(svg.source().same_occurrence(&source));
                assert_eq!(
                    block.property_value(svg_name).unwrap().as_deref(),
                    Some(svg_value)
                );
                assert_eq!(read(&block, CssKnownProperty::All).as_deref(), Some(value));
                assert_eq!(block.serialize_cssom().unwrap(), format!("all: {value};"));
                if value == "var(--A)" {
                    let CssSpecifiedDeclarationValueRef::PendingShorthand(original) = svg.value()
                    else {
                        panic!("All SVG pending member")
                    };
                    assert!(original.same_occurrence(&source));
                    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap()
                    else {
                        panic!("All pending")
                    };
                    let complete = CssExpansion::Contributions(
                        pending
                            .reenter(parse_component_values("inherit").unwrap())
                            .unwrap(),
                    );
                    let completed = if keyframe {
                        CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[complete])
                            .unwrap()
                    } else {
                        CssSpecifiedDeclarationBlock::try_from_expansions(&[complete]).unwrap()
                    };
                    assert_eq!(
                        completed.property_value(svg_name).unwrap().as_deref(),
                        Some("inherit")
                    );
                    assert_eq!(
                        read(&completed, CssKnownProperty::All).as_deref(),
                        Some("inherit")
                    );
                    assert!(
                        completed
                            .entries()
                            .iter()
                            .all(|entry| entry.source().same_occurrence(&source))
                    );
                    assert_eq!(source.to_specified_css().unwrap(), "all: var(--A);");
                }
            }
        }
    }
}

#[test]
fn svg_all_overrides_mixed_priority_and_different_pending_occurrences_disqualify_all() {
    let name = CssPropertyNameRef::SvgGlyphOrientationVertical;
    for (source, expected) in [
        ("all:inherit;glyph-orientation-vertical:auto", "auto"),
        (
            "all:inherit;glyph-orientation-vertical:inherit!important",
            "inherit",
        ),
        (
            "all:var(--A);glyph-orientation-vertical:var(--A)",
            "var(--A)",
        ),
    ] {
        let report = svg_context().parse_style_attribute(source);
        assert!(report.is_clean());
        let list = report.syntax();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(list).unwrap();
        assert_eq!(read(&block, CssKnownProperty::All), None, "{source}");
        assert_eq!(
            block.property_value(name).unwrap().as_deref(),
            Some(expected)
        );
        let entry = block
            .entries()
            .iter()
            .find(|entry| entry.property_name() == name)
            .unwrap();
        assert!(entry.source().same_occurrence(&list[1]));
        assert_eq!(entry.authored_ordinal(), 1);
        assert_eq!(entry.member_ordinal(), 0);
    }
    let report = svg_context()
        .parse_style_attribute("all:inherit!important;glyph-orientation-vertical:auto");
    assert!(report.is_clean());
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "all: inherit !important;");
    assert!(
        block
            .entries()
            .iter()
            .all(|entry| entry.source().same_occurrence(&report.syntax()[0]))
    );

    // Completed globals compare exact semantic values, while pending members
    // require one original occurrence. Equal Normal globals may coalesce.
    let report =
        svg_context().parse_style_attribute("all:inherit;glyph-orientation-vertical:inherit");
    assert!(report.is_clean());
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(block.serialize_cssom().unwrap(), "all: inherit;");
    let svg = block
        .entries()
        .iter()
        .find(|entry| entry.property_name() == name)
        .unwrap();
    assert!(svg.source().same_occurrence(&report.syntax()[1]));

    for source in [
        "all:inherit;glyph-orientation-vertical:auto",
        "all:var(--A);glyph-orientation-vertical:var(--A)",
    ] {
        let report = svg_context().parse_sheet(&format!("@keyframes k{{from{{{source}}}}}"));
        assert!(report.is_clean());
        let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
            panic!("keyframes")
        };
        let list = rule.blocks()[0].declarations();
        let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(list).unwrap();
        assert_eq!(read(&block, CssKnownProperty::All), None);
        assert_eq!(read(&block, CssKnownProperty::AnimationDuration), None);
        let entry = block
            .entries()
            .iter()
            .find(|entry| entry.property_name() == name)
            .unwrap();
        assert!(entry.source().same_occurrence(list[1].source()));
    }
    let important = parse_property_value(
        name,
        parse_component_values("auto").unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    let expansion = expand_declaration(&important).unwrap();
    let error =
        CssSpecifiedDeclarationBlock::try_from_keyframe_expansions(&[expansion]).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Importance
        }
    ));
    assert!(error.declaration().unwrap().same_occurrence(&important));
    assert_eq!(error.authored_ordinal(), Some(0));
    assert_eq!(error.member_ordinal(), None);
}

#[test]
fn svg_and_all_union_pay_exact_source_projection_and_atomic_output_work() {
    let name = CssPropertyNameRef::SvgGlyphOrientationVertical;
    for (value, expected, inputs, projections, bytes) in [
        ("auto", "auto", 2, 2, 4),
        ("0", "0deg", 2, 2, 4),
        ("180deg", "180deg", 2, 2, 6),
        ("calc(30deg + 60deg)", "calc(90deg)", 5, 4, 11),
    ] {
        let report =
            svg_context().parse_style_attribute(&format!("glyph-orientation-vertical:{value}"));
        assert!(report.is_clean());
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected)
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind);
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(&report.syntax()[0])
            );
            assert_eq!(error.authored_ordinal(), Some(0));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected)
            );
        }
    }
    let expected = all_terminal_expectations();
    let n = expected.len();
    let r = expected
        .iter()
        .filter(|(name, _, _)| keyframe_terminal_expected(*name))
        .count();
    for keyframe in [false, true] {
        let (source, block) = all_source_block(CssParserContext::default(), keyframe, "inherit");
        let admitted = if keyframe { r } else { n };
        // Requested aggregate1; eligibility3m; hypothetical/check2+2m;
        // comparisons3m; emitted global1 = P8m+4 and I2, independent of lookup.
        let exact = CssSpecifiedValueSerializationLimits::new(2, 8 * admitted + 4, 7);
        assert_eq!(
            block
                .property_value_with_limits(CssPropertyNameRef::Known(CssKnownProperty::All), exact)
                .unwrap()
                .as_deref(),
            Some("inherit")
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 8 * admitted + 4, 7),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 8 * admitted + 3, 7),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(2, 8 * admitted + 4, 6),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block
                .property_value_with_limits(
                    CssPropertyNameRef::Known(CssKnownProperty::All),
                    limits,
                )
                .unwrap_err();
            assert_eq!(resource(&error), kind);
            assert!(error.declaration().unwrap().same_occurrence(&source));
            assert_eq!(error.authored_ordinal(), Some(0));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                block
                    .property_value_with_limits(
                        CssPropertyNameRef::Known(CssKnownProperty::All),
                        exact
                    )
                    .unwrap()
                    .as_deref(),
                Some("inherit")
            );
        }
    }
    // A single SVG Auto source builds I5/P3 (keyframe P5), then output costs
    // I4/P5 plus the one All presence pass, whose admitted footprint is n/r.
    let report = svg_context().parse_style_attribute("glyph-orientation-vertical:auto");
    assert!(report.is_clean());
    let list = report.syntax();
    let exact = CssSpecifiedValueSerializationLimits::new(9, n + 8, 33);
    assert_eq!(
        list.serialize_cssom_with_limits(exact).unwrap(),
        "glyph-orientation-vertical: auto;"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, n + 8, 33),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, n + 7, 33),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, n + 8, 32),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = list.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(resource(&error), kind);
        assert!(error.declaration().unwrap().same_occurrence(&list[0]));
        assert_eq!(error.member_ordinal(), Some(0));
        assert_eq!(
            list.serialize_cssom_with_limits(exact).unwrap(),
            "glyph-orientation-vertical: auto;"
        );
    }
    let report = svg_context().parse_sheet("@keyframes k{from{glyph-orientation-vertical:auto}}");
    assert!(report.is_clean());
    let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
        panic!("keyframes")
    };
    let list = rule.blocks()[0].declarations();
    let exact = CssSpecifiedValueSerializationLimits::new(9, r + 10, 33);
    assert_eq!(
        list.serialize_cssom_with_limits(exact).unwrap(),
        "glyph-orientation-vertical: auto;"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, r + 10, 33),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, r + 9, 33),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, r + 10, 32),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let error = list.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(resource(&error), kind);
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(list[0].source())
        );
        assert_eq!(error.member_ordinal(), Some(0));
        assert_eq!(
            list.serialize_cssom_with_limits(exact).unwrap(),
            "glyph-orientation-vertical: auto;"
        );
    }
}

#[test]
fn all_svg_winner_work_retains_the_full_unfiltered_member_ordinal() {
    let expected = all_terminal_expectations();
    let n = expected.len();
    let s = expected
        .iter()
        .position(|(name, _, _)| *name == CssPropertyNameRef::SvgGlyphOrientationVertical)
        .unwrap();
    let r = expected
        .iter()
        .filter(|(name, _, _)| keyframe_terminal_expected(*name))
        .count();
    let t = expected[..s]
        .iter()
        .filter(|(name, _, _)| keyframe_terminal_expected(*name))
        .count();
    for keyframe in [false, true] {
        let (source, _) = all_source_block(CssParserContext::default(), keyframe, "inherit");
        let expansions = [expand_declaration(&source).unwrap()];
        let build = |limits| {
            if keyframe {
                CssSpecifiedDeclarationBlock::try_from_keyframe_expansions_with_limits(
                    &expansions,
                    limits,
                )
            } else {
                CssSpecifiedDeclarationBlock::try_from_expansions_with_limits(&expansions, limits)
            }
        };
        // I5 visits aggregate, occurrence, name, value-list and keyword.
        // Before winner lookup: ordinary P1+n, keyframe P2+2n. The s/t
        // prior retained winners spend one each; SVG's next lookup must fail.
        let before_svg = if keyframe { 2 + 2 * n + t } else { 1 + n + s };
        let exact = CssSpecifiedValueSerializationLimits::new(
            5,
            if keyframe { 2 + 2 * n + r } else { 1 + 2 * n },
            0,
        );
        let block = build(exact).unwrap();
        assert_eq!(block.entries().len(), if keyframe { r } else { n });
        let error = build(CssSpecifiedValueSerializationLimits::new(5, before_svg, 0)).unwrap_err();
        assert_eq!(
            resource(&error),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(error.declaration().unwrap().same_occurrence(&source));
        assert_eq!(error.authored_ordinal(), Some(0));
        assert_eq!(error.member_ordinal(), Some(s));
        let retry = build(exact).unwrap();
        let svg = retry
            .entries()
            .iter()
            .find(|entry| entry.property_name() == CssPropertyNameRef::SvgGlyphOrientationVertical)
            .unwrap();
        assert!(svg.source().same_occurrence(&source));
        assert_eq!(svg.member_ordinal(), s);
        assert_eq!(retry.serialize_cssom().unwrap(), "all: inherit;");
    }
}

#[test]
fn reconstructed_column_aliases_pay_complete_comparison_and_emission_tariffs() {
    // Columns LP: request1 + eligibility6 + hypothetical9 + check3 +
    // verification22 (width19/count3) + final5 = P46; final I6 + request1.
    // FitContent/CalcSize add one visit to each width traversal: P50/I8.
    // ColumnRule: request1 + eligibility9 + hypothetical10 + initial-color
    // equality1 + check4 + verification26 (width19/style3/color4) + final5
    // = P56/I7. Initial width Length/Medium stops before graph admission.
    // The comparison cutoffs are one below the final reached width work slot,
    // not measured request maxima. Final suppressed auto still visits count.
    for (
        property,
        source,
        expected,
        inputs,
        projections,
        bytes,
        comparison_cutoff,
        final_ordinal,
        byte_ordinal,
    ) in [
        (
            CssKnownProperty::Columns,
            "column-width:calc(1px + 2px);column-count:auto",
            "calc(3px)",
            7,
            46,
            9,
            37,
            1,
            0,
        ),
        (
            CssKnownProperty::Columns,
            "column-width:fit-content(calc(1px + 2px));column-count:auto",
            "fit-content(calc(3px))",
            8,
            50,
            22,
            39,
            1,
            0,
        ),
        (
            CssKnownProperty::Columns,
            "column-width:calc-size(any, 1px + 2px);column-count:auto",
            "calc-size(any, 3px)",
            8,
            50,
            19,
            39,
            1,
            0,
        ),
        (
            CssKnownProperty::ColumnRule,
            "column-rule-width:calc(1px + 2px);column-rule-style:solid;column-rule-color:currentcolor",
            "calc(3px) solid",
            7,
            56,
            15,
            43,
            1,
            1,
        ),
    ] {
        let list = declarations(source);
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let entries_before = block.entries().to_vec();
        let name = CssPropertyNameRef::Known(property);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{source}"
        );
        for (limits, kind, ordinal) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs, comparison_cutoff, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                0,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
                final_ordinal,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                final_ordinal,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
                byte_ordinal,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind, "{source} {limits:?}");
            assert!(
                error.declaration().unwrap().same_occurrence(&list[ordinal]),
                "{source} {limits:?}: {error:?}"
            );
            assert_eq!(
                error.authored_ordinal(),
                Some(ordinal),
                "{source} {limits:?}"
            );
            assert_eq!(error.member_ordinal(), Some(0), "{source} {limits:?}");
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "{source}: adequate atomic retry"
            );
        }
        assert_eq!(list, before);
        assert_eq!(block.entries().len(), entries_before.len());
        for (entry, saved) in block.entries().iter().zip(entries_before) {
            assert!(entry.source().same_occurrence(saved.source()));
            assert_eq!(entry.authored_ordinal(), saved.authored_ordinal());
            assert_eq!(entry.member_ordinal(), saved.member_ordinal());
        }
    }
}

#[test]
fn grid_template_counted_repeat_verification_pays_complete_work_and_keeps_selected_sources() {
    // General rows: list1/repeat1/count7/primitive breadth1 = semantic10;
    // final I7/P6. Auto adds repeat1/breadth1; Subgrid replaces breadth1
    // with names1/name1. The count graph has Function/Sum/two Values P4;
    // no tariff is invented for carrier wrappers or raw Grid breadth equality.
    // Request1 + eligibility9 + hypothetical (rows+3) + check4 +
    // verification (2*rows+11) + final (rows-2) gives P66/74/70.
    // Final template/list/none visits plus request give I10/12/11.
    // The rows comparison cutoffs 51/57/54 are one below the fourth count
    // slot; final one-under limits reach actual columns:none declaration1.
    for (rows, expected, inputs, projections, bytes, comparison_cutoff) in [
        (
            "repeat(calc(1 + 2), 1px)",
            "repeat(calc(3), 1px) / none",
            10,
            66,
            27,
            51,
        ),
        (
            "repeat(calc(1 + 2), 1px) repeat(auto-fill, 1px)",
            "repeat(calc(3), 1px) repeat(auto-fill, 1px) / none",
            12,
            74,
            50,
            57,
        ),
        (
            "subgrid repeat(calc(1 + 2), [a])",
            "subgrid repeat(calc(3), [a]) / none",
            11,
            70,
            35,
            54,
        ),
    ] {
        let source = format!(
            "grid-template-rows:{rows};grid-template-columns:none;grid-template-areas:none"
        );
        let list = declarations(&source);
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let entries_before = block.entries().to_vec();
        let name = CssPropertyNameRef::Known(CssKnownProperty::GridTemplate);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{rows}"
        );
        for (limits, kind, ordinal) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs, comparison_cutoff, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                0,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
                1,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                1,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
                1,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind, "{rows} {limits:?}");
            assert!(
                error.declaration().unwrap().same_occurrence(&list[ordinal]),
                "{rows} {limits:?}: {error:?}"
            );
            assert_eq!(error.authored_ordinal(), Some(ordinal), "{rows} {limits:?}");
            assert_eq!(error.member_ordinal(), Some(0), "{rows} {limits:?}");
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "{rows}: adequate atomic retry"
            );
        }
        assert_eq!(list, before);
        assert_eq!(block.entries().len(), entries_before.len());
        for (entry, saved) in block.entries().iter().zip(entries_before) {
            assert!(entry.source().same_occurrence(saved.source()));
            assert_eq!(entry.authored_ordinal(), saved.authored_ordinal());
            assert_eq!(entry.member_ordinal(), saved.member_ordinal());
        }
    }
}

#[test]
fn cue_inverse_shortens_separately_authored_exact_offsets_and_zero_defaults() {
    // Two members: eligibility6, expansion3. Each Audio visits owner1,
    // URL aggregate/target2, and an explicit dB1 when present. The selected
    // semantic comparison borrows URL/decimal data at P0. Omitting the second
    // cue from the typed inverse still verifies both original terminals.
    // Nonzero pair: request1 + eligibility6 + hypothetical9 + check3 +
    // verification18 + final5 = P42/I6. Omitted-before/zero-after uses
    // hypothetical8/verification15/final4 => P37/I5. Reversed retains explicit
    // zero in both generated terminals: verification17/final5 => P40/I6.
    for (before_value, after_value, expected, inputs, projections, bytes) in [
        ("url(a) 1dB", "url('a') 01DB", "url(\"a\") 1db", 6, 42, 12),
        (
            "url(a) -3dB",
            "url(\"a\") -3.0dB",
            "url(\"a\") -3db",
            6,
            42,
            13,
        ),
        ("url(a)", "url('a') -0dB", "url(\"a\")", 5, 37, 8),
        ("url(a) 0dB", "url('a')", "url(\"a\")", 6, 40, 8),
    ] {
        let list = declarations(&format!(
            "cue-before:{before_value};cue-after:{after_value}"
        ));
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let entries_before = block.entries().to_vec();
        let name = CssPropertyNameRef::Known(CssKnownProperty::Cue);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected)
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind);
            assert!(
                error.declaration().unwrap().same_occurrence(&list[0]),
                "{before_value} / {after_value}: {error:?}"
            );
            assert_eq!(error.authored_ordinal(), Some(0));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "adequate retry"
            );
        }
        assert_eq!(list, before);
        assert_eq!(block.entries().len(), entries_before.len());
        for (entry, saved) in block.entries().iter().zip(entries_before) {
            assert!(entry.source().same_occurrence(saved.source()));
            assert_eq!(entry.authored_ordinal(), saved.authored_ordinal());
            assert_eq!(entry.member_ordinal(), saved.member_ordinal());
        }
    }
}

#[test]
fn nested_grid_math_verification_admits_selected_leaf_work_and_original_terminal_errors() {
    // Single LP rows semantic R8 = list1 + four source/three arena visits.
    // Request1 + eligibility9 + hypothetical11 + check4 + verification27
    // (rows21, columns3, areas3) + final6 => I8/P58/B16.
    // Counted repeat with LP math has R16 and E8 (count4 before breadth4):
    // hypothetical19 + verification47 + final10 + request1/eligibility9/
    // check4 => I13/P90. Before rows equality has P66; the count ends at70
    // and the breadth ends at74, so P73 fails on the original rows terminal.
    // These public inverse routes clone retained descendants; they qualify
    // newly selected work admission, without inventing a shifted-origin false.
    for (rows, expected, inputs, projections, bytes, cutoff) in [
        ("calc(1px + 2px)", "calc(3px) / none", 8, 58, 16, 45),
        (
            "repeat(calc(1 + 2), calc(1px + 2px))",
            "repeat(calc(3), calc(3px)) / none",
            13,
            90,
            33,
            73,
        ),
    ] {
        let list = declarations(&format!(
            "grid-template-rows:{rows};grid-template-columns:none;grid-template-areas:none"
        ));
        let before = list.clone();
        let block = CssSpecifiedDeclarationBlock::try_from_declarations(&list).unwrap();
        let entries_before = block.entries().to_vec();
        let name = CssPropertyNameRef::Known(CssKnownProperty::GridTemplate);
        let exact = CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes);
        assert_eq!(
            block
                .property_value_with_limits(name, exact)
                .unwrap()
                .as_deref(),
            Some(expected),
            "{rows}"
        );
        for (limits, kind, ordinal) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs, cutoff, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                0,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
                1,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                1,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
                1,
            ),
        ] {
            let error = block.property_value_with_limits(name, limits).unwrap_err();
            assert_eq!(resource(&error), kind, "{rows} {limits:?}");
            assert!(
                error.declaration().unwrap().same_occurrence(&list[ordinal]),
                "{rows}: {error:?}"
            );
            assert_eq!(error.authored_ordinal(), Some(ordinal));
            assert_eq!(error.member_ordinal(), Some(0));
            assert_eq!(
                block
                    .property_value_with_limits(name, exact)
                    .unwrap()
                    .as_deref(),
                Some(expected),
                "adequate retry"
            );
        }
        assert_eq!(list, before);
        assert_eq!(block.entries().len(), entries_before.len());
        for (entry, saved) in block.entries().iter().zip(entries_before) {
            assert!(entry.source().same_occurrence(saved.source()));
            assert_eq!(entry.authored_ordinal(), saved.authored_ordinal());
            assert_eq!(entry.member_ordinal(), saved.member_ordinal());
        }
    }
}

#[test]
fn normal_cue_pair_provider_still_visits_explicit_redundant_children() {
    let list = declarations("cue:url(a) 1dB url('a') 01DB");
    let before = list.clone();
    let CssKnownPropertyValueRef::Cue(wrapper) = list[0].known().unwrap().property_value().unwrap()
    else {
        panic!("cue pair")
    };
    let value = wrapper.value();
    assert!(value.authored_after().is_some());
    assert_ne!(
        value.before(),
        value.after(),
        "ordinary Eq retains exact authored decibel spelling"
    );
    // Pair1 + two Audio owner/URL aggregate/target/dB groups4 each = I9/P9.
    let exact = CssSpecifiedValueSerializationLimits::new(9, 9, 12);
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        "url(\"a\") 1db"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, 9, 12),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 8, 12),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 9, 11),
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
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            "url(\"a\") 1db"
        );
    }
    assert_eq!(list, before);
    assert!(value.authored_after().is_some());
}
