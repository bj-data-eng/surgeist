//! Grid 2 (2025-03-26) §§7.3–7.3.1: decoded template strings define
//! cell runs, equal-width rows and rectangular named regions.
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#grid-template-areas-property
//! This RED uses only the existing public parsing and expansion interfaces.

use crate::css::support::property_admission::assert_same_grammar_facts;
use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::GridTemplateAreas;

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name("grid-template-areas").expect("known Grid longhand")
}

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("grid-template-areas:{value}!important");
    let report = CssParser::default().parse_block_contents(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .is_ok(),
        "{source}"
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(value: &str) -> CssDeclaration {
    CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Grammar(grammar()),
        CssComponentValues::parse(&CssParser::default(), value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked grid-template-areas {value}: {error:?}"))
}

fn terminal(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        source.expand().expect("intrinsic area expansion")
    else {
        panic!("one completed area longhand")
    };
    let [item] = values.items() else {
        panic!("one area contribution")
    };
    assert_eq!(item.property(), PROPERTY);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    values
}

fn authored_rows(source: &CssDeclaration) -> Vec<Vec<String>> {
    let CssKnownPropertyValueRef::GridTemplateAreas(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed area value")
    };
    let CssGridTemplateAreas::Rows(rows) = value.value() else {
        panic!("authored area rows")
    };
    rows.rows()
        .iter()
        .map(|row| {
            row.cells()
                .iter()
                .map(|cell| match cell {
                    CssGridTemplateAreaCell::Empty => ".".to_owned(),
                    CssGridTemplateAreaCell::Named(name) => name.as_str().to_owned(),
                    _ => panic!("future area cell"),
                })
                .collect()
        })
        .collect()
}

fn invalid(value: &str) {
    let source = format!("color:red;grid-template-areas:{value};color:blue");
    let report = CssParser::default().parse_block_contents(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid area diagnostic: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        report.syntax().len(),
        2,
        "valid neighbors survive: {source}"
    );
    assert_eq!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
    assert_same_grammar_facts(
        &CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(grammar()),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err(),
        diagnostic.error(),
        "checked construction accepted invalid area: {value}",
    );
}

#[test]
fn none_and_rectangular_rows_remain_valid_controls() {
    for source in [parsed("none"), checked("none")] {
        let CssKnownPropertyValueRef::GridTemplateAreas(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed area value")
        };
        assert_eq!(value.value(), &CssGridTemplateAreas::None);
    }
    for source in [
        parsed("\"header header\" \"nav main\""),
        checked("\"header header\" \"nav main\""),
    ] {
        assert_eq!(
            authored_rows(&source),
            [["header", "header"], ["nav", "main"]]
        );
    }
}

#[test]
fn adjacent_named_and_dot_runs_create_independent_cells() {
    for source in [parsed("\"a...b\""), checked("\"a...b\"")] {
        assert_eq!(authored_rows(&source), [["a", ".", "b"]]);
    }
    for source in [parsed("\"a.b\""), checked("\"a.b\"")] {
        assert_eq!(authored_rows(&source), [["a", ".", "b"]]);
    }
    for source in [parsed("\"....\""), checked("\"....\"")] {
        assert_eq!(authored_rows(&source), [["."]]);
    }
}

#[test]
fn digit_names_case_and_css_whitespace_are_preserved_after_string_decoding() {
    for source in [
        parsed("\"1st\\a A\\9 b C\""),
        checked("\"1st\\a A\\9 b C\""),
    ] {
        assert_eq!(authored_rows(&source), [["1st", "A", "b", "C"]]);
    }
    for source in [parsed("\"a A\""), checked("\"a A\"")] {
        assert_eq!(authored_rows(&source), [["a", "A"]]);
    }
    // The terminated hex escape decodes to NBSP, an ident code point, not
    // CSS whitespace. The whole string denotes one named cell.
    for source in [parsed("\"a\\a0 b\""), checked("\"a\\a0 b\"")] {
        assert_eq!(authored_rows(&source), [["a\u{a0}b"]]);
    }
}

#[test]
fn area_names_accept_reserved_spellings() {
    for source in [
        parsed("\"auto span inherit revert-layer\""),
        checked("\"auto span inherit revert-layer\""),
    ] {
        let CssKnownPropertyValueRef::GridTemplateAreas(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed area value")
        };
        assert_eq!(
            value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            "\"auto span inherit revert-layer\""
        );
        terminal(&source);
    }
}

#[test]
fn trash_empty_width_and_nonrectangular_regions_invalidate_the_declaration() {
    for value in [
        "\"\"",
        "\" \\a \"",
        "\"a#b\"",
        "\"a,b\"",
        "\"a\\d b\"",
        "\"a\\c b\"",
        "\"a b\" \"a\"",
        "\"a a\" \"a .\"",
        "\"a . a\"",
    ] {
        invalid(value);
    }
}

#[test]
fn global_values_pending_reentry_and_initial_metadata_use_one_terminal() {
    let CssPropertyKindRef::Longhand(longhand) = grammar().metadata().unwrap().kind() else {
        panic!("area is one longhand")
    };
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary none initial")
    };
    assert_eq!(initial.property().known_property(), PROPERTY);
    assert_eq!(
        Some(initial),
        terminal(&parsed("none")).items()[0].ordinary_value()
    );

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = parsed(spelling);
        assert_eq!(
            terminal(&source).items()[0].value(),
            CssContributionValueRef::Global(keyword)
        );
    }

    let CssPropertyKindRef::UniversalReset(all) = CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all reset metadata")
    };
    assert!(!all.excludes(CssPropertyNameRef::Known(PROPERTY)));
    let all_source = CssParser::default().parse_block_contents("all:initial!important");
    assert!(all_source.is_clean());
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        all_source.syntax()[0].expand().unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(PROPERTY)));

    let source = parsed("var(--areas)");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("area substitution remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid_value in ["\"a#b\"", "\"a a\" \"a .\""] {
        assert!(
            pending
                .reenter(CssComponentValues::parse(&CssParser::default(), invalid_value).unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some(),
            "strict reentry rejects {invalid_value}"
        );
    }
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "\"a.b\"").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one reentered longhand")
        };
        let [item] = items.items() else {
            panic!("one reentered contribution")
        };
        assert_eq!(item.property(), PROPERTY);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn normalization_preserves_declaration_order_and_area_occurrences() {
    let source =
        ".p{color:red;grid-template-areas:\"a.b\"!important;grid-template-areas:none;opacity:1}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("one style rule")
    };
    let originals = rule.declarations();
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (index, value) in values.iter().enumerate() {
        assert_eq!(value.order(), index);
        assert!(value.source().same_occurrence(&originals[index]));
    }
    for (index, importance) in [(1, CssImportance::Important), (2, CssImportance::Normal)] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            values[index].expansion()
        else {
            panic!("one area terminal")
        };
        assert_eq!(items.items().len(), 1);
        assert_eq!(items.items()[0].property(), PROPERTY);
        assert_eq!(items.items()[0].source().importance(), importance);
        assert!(items.items()[0].source().same_occurrence(&originals[index]));
    }
    let limits = CssNormalizationLimits::try_new(1, 1, 1, 0).unwrap();
    let one = CssParser::default().parse_sheet(".p{grid-template-areas:none}");
    assert!(one.is_clean());
    let error = one.syntax().normalize(limits).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 0))
    );
}
