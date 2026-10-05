#![forbid(unsafe_code)]
//! Authored item-flow grammar and intrinsic declaration lifecycle.
//!
//! Independent authorities: selected Grid3 2026-01-21 Appendix A lines
//! 1485-1493, 1576-1584, 1643-1650, 1666-1674, 1702/1715; Values4 §2.2
//! lines 93 and 180-198 preserves quoted contiguous/nonassociative groups;
//! #992/#999 adopt flow-oriented names, complete-candidate ranking, full D/W/P/T
//! canonical output, omission initials, and four settable/zero reset members.
//! Values4 specified simplification/sorting owns the existing numeric goldens.
//! New work pricing is the adopted fixed-child policy, with existing Flow
//! scalar costs and declaration/name +2; no whole-sheet work counts are guessed.
//!
//! Runtime name/grammar lookup deliberately uses existing callable boundaries.
//! No missing Item enum/model/borrowed variants, stubs or private helpers.
//! New typed construction/getters, full tuple/initial payload inspection and
//! primitive provider costs need functional tests alongside implementation.
//! The 480-string group checks generic canonical admission/idempotence, not
//! absent typed tuple access. No issue/status/catalog/Debug-output oracles.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const FLOW: &str = "item-flow";
const TOLERANCE: &str = "flow-tolerance";
const FAMILY: [&str; 4] = ["item-direction", "item-wrap", "item-pack", FLOW];
const TERMINALS: [&str; 4] = ["item-direction", "item-wrap", "item-pack", TOLERANCE];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const DIRECTIONS: [&str; 5] = ["auto", "row", "column", "row-reverse", "column-reverse"];
const WRAPS: [&str; 12] = [
    "auto",
    "nowrap",
    "wrap",
    "normal",
    "reverse",
    "auto normal",
    "auto reverse",
    "nowrap normal",
    "nowrap reverse",
    "wrap normal",
    "wrap reverse",
    "wrap-reverse",
];
const PACKS: [&str; 4] = ["normal", "dense", "balance", "dense balance"];

fn property(name: &str) -> P {
    P::from_name(name)
        .unwrap_or_else(|| panic!("selected authored property is unavailable: {name}"))
}
fn expected_members(name: &str) -> Vec<&str> {
    if name == FLOW {
        TERMINALS.to_vec()
    } else {
        vec![name]
    }
}
fn parsed(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{name}:{value}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one retained authored occurrence")
    };
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    source.clone()
}
fn checked(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    let p = property(name);
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = if grammar {
        parse_property_value_for_grammar(p.grammar(), components.clone(), CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{name}:{value}: {error:?}"));
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}
fn text_front(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let p = property(name);
    let report = if grammar {
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important)
    } else {
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important)
    };
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let source = report
        .syntax()
        .as_ref()
        .expect("complete raw-value admission")
        .clone();
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
    source
}
fn fronts(name: &str, value: &str) -> [CssDeclaration; 5] {
    [
        parsed(name, value),
        checked(name, value, false),
        checked(name, value, true),
        text_front(name, value, false),
        text_front(name, value, true),
    ]
}
fn names(values: &CssLonghandContributions) -> Vec<&'static str> {
    values
        .items()
        .iter()
        .map(|v| v.property().canonical_name())
        .collect()
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete longhand contributions")
    };
    for item in values.items() {
        assert_source(item, source);
    }
    values
}
fn assert_tolerance(values: &CssLonghandContributions, expected: &str) {
    let item = values
        .items()
        .iter()
        .find(|v| v.property() == P::FlowTolerance)
        .expect("authorable tolerance member");
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::FlowTolerance(value)) = item.value()
    else {
        panic!("ordinary typed tolerance")
    };
    assert_eq!(value.serialize_specified().unwrap(), expected);
}
fn accepted(name: &str, input: &str, canonical: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {canonical} !important;"),
            "{name}:{input}"
        );
        let values = completed(&source);
        assert_eq!(names(&values), expected_members(name));
        for item in values.items() {
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                item.property()
            );
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(source, before);
    }
    let source = checked(name, canonical, true);
    assert_eq!(
        source.to_specified_css().unwrap(),
        format!("{name}: {canonical} !important;")
    );
}
fn invalid(name: &str, value: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic declaration failure")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("known property grammar failure")
    };
    assert_eq!(detail.property(), p);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components.clone(),
            CssImportance::Normal,
        ),
        parse_property_value_for_grammar(p.grammar(), components.clone(), CssImportance::Normal),
    ] {
        assert!(matches!(
            result.unwrap_err().kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    for report in [
        parse_property_value_text(value, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}

#[test]
fn canonical_names_are_known_in_authored_input_and_alternative_proposals_stay_unknown() {
    for (name, input, canonical) in [
        ("item-direction", "row", "row"),
        ("item-wrap", "auto", "auto"),
        ("item-pack", "normal", "normal"),
        (FLOW, "row", "row auto normal normal"),
    ] {
        accepted(name, input, canonical);
        assert_eq!(
            P::from_name(&name.to_ascii_uppercase()),
            Some(property(name))
        );
        assert_eq!(
            CssPropertyGrammar::from_name(name),
            Some(property(name).grammar())
        );
    }
    for name in ["item-track", "item-cross"] {
        assert!(P::from_name(name).is_none());
        assert!(CssPropertyGrammar::from_name(name).is_none());
        let report = parse_style_attribute(&format!("color:red;{name}:auto;color:blue"));
        assert_eq!(report.syntax().len(), 2);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
    }
    let report = parse_style_attribute(r"\69 tem-direction:RoW-ReVeRsE!important");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        property("item-direction")
    );
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "item-direction: row-reverse !important;"
    );
}
#[test]
fn all_longhand_states_preserve_authored_facets_and_canonical_order() {
    for value in DIRECTIONS {
        accepted("item-direction", value, value);
    }
    for value in WRAPS {
        accepted("item-wrap", value, value);
    }
    for mode in ["auto", "nowrap", "wrap"] {
        for order in ["normal", "reverse"] {
            accepted(
                "item-wrap",
                &format!("{order} {mode}"),
                &format!("{mode} {order}"),
            );
        }
    }
    for value in PACKS {
        accepted("item-pack", value, value);
    }
    accepted("item-pack", "balance dense", "dense balance");
    accepted("item-direction", r"\72 ow", "row");
    accepted("item-wrap", "ReVeRsE WrAp", "wrap reverse");
    accepted("item-wrap", "wrap-reverse", "wrap-reverse");
}
#[test]
fn checked_programmatic_components_keep_origins_importance_and_constructed_occurrence() {
    for (name, keyword, canonical) in [
        ("item-direction", "row", "row"),
        ("item-wrap", "reverse", "reverse"),
        ("item-pack", "balance", "balance"),
        (FLOW, "row", "row auto normal normal"),
    ] {
        let p = property(name);
        let component = CssComponentValue::try_ident(keyword).unwrap();
        assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
        let components = CssComponentValues::try_new(vec![component]).unwrap();
        let before = components.clone();
        for source in [
            parse_property_value(
                CssPropertyNameRef::Known(p),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap(),
            parse_property_value_for_grammar(
                p.grammar(),
                components.clone(),
                CssImportance::Important,
            )
            .unwrap(),
        ] {
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {canonical} !important;")
            );
            let values = completed(&source);
            assert_eq!(names(&values), expected_members(name));
            if name == FLOW {
                assert_tolerance(&values, "normal");
            }
        }
        assert_eq!(components, before);
    }
}
#[test]
fn invalid_longhand_choices_and_duplicate_facets_fail_without_losing_siblings() {
    for value in ["normal", "reverse", "row column", "row row", "1px"] {
        invalid("item-direction", value);
    }
    for value in [
        "wrap nowrap",
        "auto auto",
        "normal reverse",
        "reverse reverse",
        "wrap-reverse normal",
        "wrap-reverse reverse",
        "row",
        "1px",
    ] {
        invalid("item-wrap", value);
    }
    for value in [
        "normal dense",
        "normal balance",
        "dense dense",
        "balance balance",
        "reverse",
        "1px",
    ] {
        invalid("item-pack", value);
    }
}
#[test]
fn shorthand_overlap_ranking_and_omission_initials_have_explicit_canonical_goldens() {
    for (input, canonical, tolerance) in [
        ("row", "row auto normal normal", "normal"),
        ("auto", "auto auto normal normal", "normal"),
        ("normal", "auto normal normal normal", "normal"),
        ("normal normal", "auto normal normal normal", "normal"),
        (
            "normal normal normal",
            "auto normal normal normal",
            "normal",
        ),
        ("auto normal", "auto normal normal normal", "normal"),
        ("reverse auto", "auto reverse normal normal", "normal"),
        (
            "auto reverse normal",
            "auto reverse normal normal",
            "normal",
        ),
        ("wrap normal", "auto wrap normal normal", "normal"),
        ("wrap reverse", "auto wrap reverse normal normal", "normal"),
        ("normal dense", "auto normal dense normal", "normal"),
        (
            "normal dense balance",
            "auto normal dense balance normal",
            "normal",
        ),
        ("row wrap normal normal", "row wrap normal normal", "normal"),
        (
            "row wrap normal normal normal",
            "row wrap normal normal normal",
            "normal",
        ),
        (
            "balance dense reverse -2px row",
            "row reverse dense balance -2px",
            "-2px",
        ),
        (
            "wrap-reverse infinite",
            "auto wrap-reverse normal infinite",
            "infinite",
        ),
        (
            "calc(-2px - 3%)",
            "auto auto normal calc(-3% - 2px)",
            "calc(-3% - 2px)",
        ),
        ("-25%", "auto auto normal -25%", "-25%"),
        ("infinite", "auto auto normal infinite", "infinite"),
        ("auto normal -2px", "auto normal normal -2px", "-2px"),
    ] {
        accepted(FLOW, input, canonical);
        for source in fronts(FLOW, input) {
            assert_tolerance(&completed(&source), tolerance);
        }
    }
}
#[test]
fn contiguous_wrap_and_pack_chunks_permute_as_whole_constituents() {
    // Independent unordered grammar: four distinct chunks have 24 permutations;
    // their internally reversed wrap/pack spellings normalize within each chunk.
    let chunks = ["column-reverse", "reverse wrap", "balance dense", "-25%"];
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if a == b || a == c || a == d || b == c || b == d || c == d {
                        continue;
                    }
                    accepted(
                        FLOW,
                        &format!("{} {} {} {}", chunks[a], chunks[b], chunks[c], chunks[d]),
                        "column-reverse wrap reverse dense balance -25%",
                    );
                }
            }
        }
    }
    accepted(FLOW, "reverse wrap row", "row wrap reverse normal normal");
    accepted(FLOW, "balance dense row", "row auto dense balance normal");
    for value in ["wrap row reverse", "reverse row wrap", "dense row balance"] {
        invalid(FLOW, value);
    }
}
#[test]
fn duplicate_slots_excess_components_and_partial_global_mixtures_are_invalid() {
    for value in [
        "",
        "row row",
        "auto auto auto",
        "normal normal normal normal",
        "reverse reverse",
        "wrap nowrap",
        "wrap-reverse reverse",
        "dense balance dense",
        "1px 2px",
        "row/column",
        "row,wrap",
        "row initial",
        "inherit wrap",
        "calc(1px + 1s)",
        "-1",
        "1fr",
    ] {
        invalid(FLOW, value);
    }
}
#[test]
fn full_four_member_keyword_cross_product_readmits_its_independent_canonical_text() {
    // 5*12*4*2 = 480 complete tuples. These strings come directly from the
    // adopted four-member writer contract, never from a parser/model output.
    // Typed tuple equality remains a functional new-API obligation.
    for direction in DIRECTIONS {
        for wrap in WRAPS {
            for pack in PACKS {
                for tolerance in ["normal", "infinite"] {
                    let canonical = format!("{direction} {wrap} {pack} {tolerance}");
                    for source in [parsed(FLOW, &canonical), checked(FLOW, &canonical, true)] {
                        assert_eq!(
                            source.to_specified_css().unwrap(),
                            format!("item-flow: {canonical} !important;")
                        );
                        let values = completed(&source);
                        assert_eq!(names(&values), TERMINALS);
                        assert_tolerance(&values, tolerance);
                    }
                }
            }
        }
    }
}
#[test]
fn intrinsic_membership_is_one_or_four_with_no_reset_only_terminal() {
    for name in ["item-direction", "item-wrap", "item-pack", TOLERANCE] {
        let p = property(name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("intrinsic longhand")
        };
        assert_eq!(metadata.property().known_property(), p);
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        assert_eq!(initial.property().known_property(), p);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary intrinsic initial")
        };
        assert_eq!(initial.property().known_property(), p);
        if name == TOLERANCE {
            let CssLonghandValueRef::FlowTolerance(value) = initial.view() else {
                panic!("typed existing normal initial")
            };
            assert_eq!(value, &CssFlowTolerance::normal());
        }
    }
    let CssPropertyKindRef::Shorthand(metadata) = property(FLOW).metadata().unwrap().kind() else {
        panic!("ordinary shorthand")
    };
    assert_eq!(
        metadata
            .members()
            .iter()
            .map(|p| p.known_property().canonical_name())
            .collect::<Vec<_>>(),
        TERMINALS
    );
    assert_eq!(metadata.settable_members(), metadata.members());
    assert!(metadata.reset_only_members().is_empty());
    assert!(!metadata.is_legacy());
    for input in ["row", "wrap", "dense", "normal"] {
        assert_tolerance(&completed(&checked(FLOW, input, true)), "normal");
    }
}
#[test]
fn all_globals_propagate_symbolically_to_every_member_and_keep_importance() {
    for name in FAMILY {
        for (text, keyword) in GLOBALS {
            for source in fronts(name, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
                let values = completed(&source);
                assert_eq!(names(&values), expected_members(name));
                for item in values.items() {
                    assert!(
                        matches!(item.value(), CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert!(item.replacement_components().is_none());
                }
            }
        }
    }
}
fn replacement_for(name: &str) -> (&str, &str) {
    match name {
        "item-direction" => ("column-reverse", "column-reverse"),
        "item-wrap" => ("reverse wrap", "wrap reverse"),
        "item-pack" => ("balance dense", "dense balance"),
        FLOW => (
            "balance dense reverse calc(-2px - 3%) row",
            "row reverse dense balance calc(-3% - 2px)",
        ),
        _ => panic!("family role"),
    }
}
#[test]
fn var_env_attr_pending_reentry_preserves_original_and_replacement_provenance() {
    for name in FAMILY {
        for text in ["var(--flow)", "env(flow)", "attr(data-flow *)"] {
            for source in fronts(name, text) {
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("whole-value pending handle")
                };
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
                let (replacement_text, canonical) = replacement_for(name);
                let replacement = parse_component_values(replacement_text).unwrap();
                let before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed strict reentry")
                };
                assert_eq!(names(&values), expected_members(name));
                for item in values.items() {
                    assert_source(item, &source);
                    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (actual, supplied) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(actual.origin(), supplied.origin());
                        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                            (actual.origin(), supplied.origin())
                        else {
                            panic!("original replacement snapshot")
                        };
                        assert!(actual.source().same_snapshot(supplied.source()));
                        assert_eq!(actual.source().as_str(), replacement_text);
                    }
                }
                if name == FLOW {
                    assert_tolerance(&values, "calc(-3% - 2px)");
                }
                assert_eq!(replacement, before);
                assert_eq!(
                    checked(name, replacement_text, false)
                        .to_specified_css()
                        .unwrap(),
                    format!("{name}: {canonical} !important;")
                );
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global reentry")
                    };
                    assert_eq!(names(&values), expected_members(name));
                    for item in values.items() {
                        assert_source(item, &source);
                        assert!(
                            matches!(item.value(), CssContributionValueRef::Global(v) if v == keyword)
                        );
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
            }
        }
    }
}
#[test]
fn residual_or_invalid_replacements_fail_atomically_and_the_same_handle_retries() {
    for name in FAMILY {
        let source = checked(name, "var(--flow)", true);
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in [
            "var(--again)",
            "env(again)",
            "attr(data-again *)",
            "calc(var(--again) + 1px)",
            r"\76 ar(--again)",
        ] {
            for _ in 0..2 {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
        }
        let ordinary = replacement_for(name).0;
        let invalid_value = match name {
            "item-direction" => "row row",
            "item-wrap" => "normal reverse",
            "item-pack" => "normal dense",
            FLOW => "wrap row reverse",
            _ => unreachable!(),
        };
        for text in [
            invalid_value.to_owned(),
            format!("{ordinary}!important"),
            format!("{ordinary};color:red"),
        ] {
            for _ in 0..2 {
                let error = handle
                    .reenter(parse_component_values(&text).unwrap())
                    .unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("strict replacement grammar")
                };
                assert!(matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(_)
                ));
            }
        }
        assert_eq!(source, before);
        assert!(handle.source().same_occurrence(&source));
        let CssContributions::Longhands(values) = handle
            .reenter(parse_component_values(ordinary).unwrap())
            .unwrap()
        else {
            panic!("successful reusable retry")
        };
        assert_eq!(names(&values), expected_members(name));
        for item in values.items() {
            assert_source(item, &source);
        }
        assert!(source.known().unwrap().substitution_dependent().is_some());
    }
}
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("public original implicit closing origin")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF source")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}
#[test]
fn both_checked_fronts_and_reentry_reject_original_comment_and_function_closures() {
    for name in FAMILY {
        let p = property(name);
        let source = checked(name, "var(--flow)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let mut texts = vec![format!("{}/*", replacement_for(name).0)];
        texts.extend(GLOBALS.into_iter().map(|(text, _)| format!("{text}/*")));
        texts.extend(
            [
                "var(--flow)/*",
                "env(flow)/*",
                "attr(data-flow *)/*",
                "var(--flow",
                "env(flow",
                "attr(data-flow *",
            ]
            .map(str::to_owned),
        );
        if name == FLOW {
            texts.push("row reverse dense calc(-2px - 3%".into());
        }
        for text in texts {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for result in [
                parse_property_value(
                    CssPropertyNameRef::Known(p),
                    components.clone(),
                    CssImportance::Important,
                ),
                parse_property_value_for_grammar(
                    p.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ] {
                assert_closure(&result.unwrap_err(), &origin, &text);
            }
            if text.starts_with("var") || text.starts_with("env") || text.starts_with("attr") {
                // Residual substitution detection precedes replacement grammar.
                assert!(matches!(
                    handle.reenter(components.clone()).unwrap_err().kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            } else {
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original reentry closure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
            assert_eq!(components, before);
        }
        for text in [
            format!("{}/**/", replacement_for(name).0),
            "initial/**/".into(),
            "var(--flow)/**/".into(),
            "env(flow)/**/".into(),
            "attr(data-flow *)/**/".into(),
        ] {
            checked(name, &text, false);
            checked(name, &text, true);
        }
        assert!(
            handle
                .reenter(parse_component_values(replacement_for(name).0).unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
    accepted(
        FLOW,
        "row reverse dense calc(-2px - 3%)",
        "row reverse dense calc(-3% - 2px)",
    );
}
#[test]
fn illegal_annotations_map_to_the_original_token_through_checked_and_reentry_paths() {
    for name in FAMILY {
        let p = property(name);
        let text = format!("/*😀*/{}!important", replacement_for(name).0);
        let components = parse_component_values(&text).unwrap();
        let serialized = components.serialize().unwrap();
        let expected = serialized
            .origin_at(serialized.as_css().find('!').unwrap())
            .unwrap()
            .clone();
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(p),
                components.clone(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                p.grammar(),
                components.clone(),
                CssImportance::Normal,
            ),
        ] {
            assert_eq!(result.unwrap_err().origin(), &expected);
        }
        let source = checked(name, "var(--flow)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let error = handle.reenter(components).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
            panic!("strict annotation failure")
        };
        assert_eq!(error.origin(), &expected);
        assert!(
            handle
                .reenter(parse_component_values(replacement_for(name).0).unwrap())
                .is_ok()
        );
    }
}
#[test]
fn browser_eof_recovery_keeps_diagnostics_and_retained_normalized_occurrences() {
    for name in FAMILY {
        let mut texts = vec![
            format!("{}/*", replacement_for(name).0),
            "initial/*".into(),
            "var(--flow)/*".into(),
            "var(--flow".into(),
        ];
        if name == FLOW {
            texts.push("row reverse dense calc(-2px - 3%".into());
        }
        for text in texts {
            let css = format!(".a{{{name}:{text}");
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|v| v.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            assert!(validate_sheet(&css).is_err());
            let before = report.clone();
            let count = if text.starts_with("var") {
                1
            } else {
                expected_members(name).len()
            };
            let exact = CssNormalizationLimits::try_new(0, 1, 1, count).unwrap();
            let normalized = normalize_report_with_limits(&report, exact).unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let declarations: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|item| match item {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            let [item] = declarations.as_slice() else {
                panic!("one retained recovered occurrence")
            };
            assert_eq!(
                item.source().known().unwrap().property().canonical_name(),
                name
            );
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().importance(), CssImportance::Normal);
            match item.expansion() {
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(names(values), expected_members(name));
                    for terminal in values.items() {
                        assert_source(terminal, item.source());
                    }
                }
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                _ => panic!("retained intrinsic family expansion"),
            }
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, count - 1).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: count - 1
                }
            );
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    }
}
#[test]
fn normalization_keeps_order_context_importance_and_exact_one_four_pending_accounting() {
    let css = "@media screen{.a{item-direction:row!important;item-wrap:reverse wrap;item-pack:balance dense;item-flow:row;item-flow:var(--flow);item-flow:unset!important;flow-tolerance:-25%}}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // 1+1+1+4+one pending handle+4+1 = 13, seven occurrences, two rules.
    let exact = CssNormalizationLimits::try_new(1, 2, 7, 13).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 7);
    for (index, item) in declarations.iter().enumerate() {
        let name = [
            "item-direction",
            "item-wrap",
            "item-pack",
            FLOW,
            FLOW,
            FLOW,
            TOLERANCE,
        ][index];
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            name
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 5 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(names(values), expected_members(name));
                for terminal in values.items() {
                    assert_source(terminal, item.source());
                    if index == 5 {
                        assert!(matches!(
                            terminal.value(),
                            CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                        ));
                    } else {
                        assert!(matches!(
                            terminal.value(),
                            CssContributionValueRef::Ordinary(_)
                        ));
                    }
                }
                if index == 3 {
                    assert_tolerance(values, "normal");
                }
                if index == 6 {
                    assert_tolerance(values, "-25%");
                }
            }
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 4);
                assert!(handle.source().same_occurrence(item.source()));
            }
            _ => panic!("intrinsic expansion"),
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(1, 2, 7, 12).unwrap(),
            CssNormalizationResource::Contributions,
            12,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 6, 13).unwrap(),
            CssNormalizationResource::Declarations,
            6,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(6));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[6].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn adopted_keyword_and_numeric_declaration_costs_are_cumulative_atomic_and_reusable() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    // The literal table is the adopted public semantic pricing, not measured
    // implementation work. A declaration/name adds two nodes to child totals.
    for (name, input, canonical, inputs, projections) in [
        ("item-direction", "row", "row", 3, 3),
        ("item-wrap", "auto", "auto", 3, 3),
        ("item-wrap", "reverse", "reverse", 3, 3),
        ("item-wrap", "reverse wrap", "wrap reverse", 4, 4),
        ("item-wrap", "wrap-reverse", "wrap-reverse", 3, 3),
        ("item-pack", "normal", "normal", 3, 3),
        ("item-pack", "balance dense", "dense balance", 4, 4),
        (FLOW, "auto", "auto auto normal normal", 6, 6),
        (
            FLOW,
            "row wrap reverse dense balance -2px",
            "row wrap reverse dense balance -2px",
            8,
            8,
        ),
        (
            FLOW,
            "row auto normal calc(1px + 2em)",
            "row auto normal calc(2em + 1px)",
            9,
            10,
        ),
    ] {
        let source = checked(name, input, true);
        let before = source.clone();
        let expected = format!("{name}: {canonical} !important;");
        let exact = Limits::new(inputs, projections, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(inputs - 1, projections, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(inputs, projections - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(inputs, projections, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
    let source = checked(FLOW, "-1e999px", true);
    let before = source.clone();
    let expected = format!(
        "item-flow: auto auto normal -1{}px !important;",
        "0".repeat(999)
    );
    assert_eq!(
        source
            .to_specified_css_with_limits(Limits::new(6, 6, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        source
            .to_specified_css_with_limits(Limits::new(6, 6, expected.len() - 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(source, before);
    assert_eq!(source.to_specified_css().unwrap(), expected);
}
#[test]
fn item_sheet_siblings_share_exact_final_bytes_and_suppress_failed_output_atomically() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report =
        parse_sheet(".a{item-flow:row!important}.b{item-flow:row auto normal calc(1px + 2em)}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { item-flow: row auto normal normal !important; }\n.b { item-flow: row auto normal calc(2em + 1px); }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|v| v.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}

#[test]
fn existing_flow_signed_math_and_original_closure_control_is_independent_of_item_registration() {
    for (input, expected) in [
        ("normal", "normal"),
        ("infinite", "infinite"),
        ("-25%", "-25%"),
        ("calc(-2px - 3%)", "calc(-3% - 2px)"),
    ] {
        accepted(TOLERANCE, input, expected);
        assert_tolerance(&completed(&checked(TOLERANCE, input, true)), expected);
    }
    let components = parse_component_values("-2px/*").unwrap();
    let origin = implicit_origin(&components);
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(P::FlowTolerance),
            components.clone(),
            CssImportance::Normal,
        ),
        parse_property_value_for_grammar(
            P::FlowTolerance.grammar(),
            components.clone(),
            CssImportance::Normal,
        ),
    ] {
        assert_closure(&result.unwrap_err(), &origin, "-2px/*");
    }
}
#[test]
fn existing_flex_flow_keeps_its_two_members_and_own_omission_defaults() {
    let source = checked("flex-flow", "column", true);
    assert_eq!(
        source.to_specified_css().unwrap(),
        "flex-flow: column nowrap !important;"
    );
    let values = completed(&source);
    assert_eq!(names(&values), ["flex-direction", "flex-wrap"]);
    assert!(matches!(
        values.items()[0].value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::FlexDirection(
            CssFlexDirection::Column
        ))
    ));
    assert!(matches!(
        values.items()[1].value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::FlexWrap(CssFlexWrap::NoWrap))
    ));
    assert_eq!(
        checked("flex-flow", "wrap column", false)
            .to_specified_css()
            .unwrap(),
        "flex-flow: column wrap !important;"
    );
}
#[test]
fn existing_grid_template_grid_and_normal_auto_flow_membership_remain_independent() {
    for (name, input, expected, members) in [
        (
            "grid-template",
            "none",
            "none",
            vec![
                "grid-template-rows",
                "grid-template-columns",
                "grid-template-areas",
            ],
        ),
        (
            "grid",
            "none",
            "none",
            vec![
                "grid-template-rows",
                "grid-template-columns",
                "grid-template-areas",
                "grid-auto-rows",
                "grid-auto-columns",
                "grid-auto-flow",
            ],
        ),
        ("grid-auto-flow", "normal", "normal", vec!["grid-auto-flow"]),
    ] {
        let source = checked(name, input, true);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {expected} !important;")
        );
        assert_eq!(names(&completed(&source)), members);
    }
}
