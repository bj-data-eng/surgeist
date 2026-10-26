//! Public-boundary regression contracts from CSS Box Alignment 3 (2026-01-30)
//! §§8.1, 8.2, and 8.4. This RED slice uses only pre-existing APIs.

use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::*;

#[test]
fn gap_accepts_one_or_two_values_with_row_then_column_assignment() {
    for (value, row, column) in [
        ("normal 2px", "normal", "2px"),
        ("3% 4px", "3%", "4px"),
        ("normal", "normal", "normal"),
        ("1px", "1px", "1px"),
    ] {
        for source in [declaration("gap", value), checked("gap", value)] {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                source.expand().expect("gap has intrinsic expansion")
            else {
                panic!("gap completes two contributions")
            };
            let [row_item, column_item] = values.items() else {
                panic!("gap sets exactly two longhands")
            };
            for (item, property, expected) in [
                (row_item, CssKnownProperty::RowGap, row),
                (column_item, CssKnownProperty::ColumnGap, column),
            ] {
                assert_eq!(item.property(), property);
                let expected = declaration(property.canonical_name(), expected);
                let CssExpansion::Contributions(CssContributions::Longhands(expected_values)) =
                    expected.expand().expect("longhand expands")
                else {
                    panic!("one longhand contribution")
                };
                assert_eq!(
                    item.ordinary_value(),
                    expected_values.items()[0].ordinary_value()
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn negative_gap_literals_are_rejected_without_losing_neighboring_declarations() {
    for name in ["gap", "row-gap", "column-gap"] {
        for value in ["-1px", "-2%"] {
            rejected(name, value, Some(value));
        }
    }
}

#[test]
fn grid_gap_names_are_aliases_of_canonical_property_identities() {
    for (alias, canonical, property) in [
        ("grid-row-gap", "row-gap", CssKnownProperty::RowGap),
        ("grid-column-gap", "column-gap", CssKnownProperty::ColumnGap),
        ("grid-gap", "gap", CssKnownProperty::Gap),
    ] {
        assert_eq!(CssKnownProperty::from_name(alias), Some(property));
        assert_eq!(
            CssKnownProperty::from_name(&alias.to_ascii_uppercase()),
            Some(property)
        );
        assert_eq!(grammar(alias).target_property(), property);
        assert_eq!(grammar(alias).name(), canonical);
        assert_eq!(grammar(alias).feature_id(), grammar(canonical).feature_id());
        assert_eq!(property.aliases(), &[alias]);

        let support = property_support_metadata(alias).expect("alias support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.canonical_name(), canonical);
        assert_eq!(support.aliases(), &[alias]);

        let value = if property == CssKnownProperty::Gap {
            "1px 2px"
        } else {
            "1px"
        };
        let parsed = declaration(alias, value);
        assert_eq!(parsed.known().unwrap().property(), property);
        assert!(
            parsed.parsed_name().is_some(),
            "authored alias keeps source provenance"
        );
        assert_eq!(checked(alias, value).known().unwrap().property(), property);
    }
}

#[test]
fn gap_metadata_has_two_noninherited_normal_longhands() {
    let CssPropertyKindRef::Shorthand(shorthand) = CssKnownProperty::Gap
        .metadata()
        .expect("gap metadata")
        .kind()
    else {
        panic!("gap is a shorthand")
    };
    assert_eq!(shorthand.settable_members().len(), 2);
    assert_eq!(shorthand.members().len(), 2);
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
    assert_eq!(
        shorthand.members()[0].known_property(),
        CssKnownProperty::RowGap
    );
    assert_eq!(
        shorthand.members()[1].known_property(),
        CssKnownProperty::ColumnGap
    );

    for property in [CssKnownProperty::RowGap, CssKnownProperty::ColumnGap] {
        let CssPropertyKindRef::Longhand(longhand) =
            property.metadata().expect("gap longhand metadata").kind()
        else {
            panic!("gap member is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let intrinsic = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = intrinsic.view() else {
            panic!("gap longhand has fixed normal initial")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration(property.canonical_name(), "normal")
                .expand()
                .unwrap()
        else {
            panic!("normal expands")
        };
        assert_eq!(initial, values.items()[0].ordinary_value().unwrap());
    }
}

#[test]
fn gap_global_and_pending_values_expand_to_both_members() {
    let global = declaration("gap", "inherit");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        global.expand().expect("global gap expands")
    else {
        panic!("global gap completes")
    };
    let [row, column] = values.items() else {
        panic!("two global gap members")
    };
    assert_eq!(row.property(), CssKnownProperty::RowGap);
    assert_eq!(column.property(), CssKnownProperty::ColumnGap);
    assert_eq!(
        row.value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
    assert_eq!(
        column.value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );

    let source = declaration("grid-gap", "var(--space)");
    let CssExpansion::Pending(pending) = source.expand().expect("gap is pending") else {
        panic!("substitution-dependent gap remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "-1px").unwrap())
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "1px 2px").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("replacement completes two gap members")
    };
    let [row, column] = values.items() else {
        panic!("two substituted members")
    };
    assert_eq!(row.property(), CssKnownProperty::RowGap);
    assert_eq!(column.property(), CssKnownProperty::ColumnGap);
    for item in [row, column] {
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}
