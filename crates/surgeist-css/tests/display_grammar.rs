#![forbid(unsafe_code)]
//! Display3 2026-06-05 §2 and Grid3 2026-01-21 §2.2 define these
//! independent grammar expectations: 110 base forms plus two extension values.
//! https://www.w3.org/TR/2026/CRD-css-display-3-20260605/#the-display-properties
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#grid-lanes-containers
use std::collections::BTreeSet;
use surgeist_css::*;

fn ordinary_forms() -> BTreeSet<String> {
    let outside = ["block", "inline", "run-in"];
    let inside = ["flow", "flow-root", "table", "flex", "grid", "ruby"];
    let mut forms = BTreeSet::new();
    for value in outside.into_iter().chain(inside) {
        forms.insert(value.to_owned());
    }
    for outer in outside {
        for inner in inside {
            forms.insert(format!("{outer} {inner}"));
            forms.insert(format!("{inner} {outer}"));
        }
    }
    forms.insert("list-item".into());
    for value in outside.into_iter().chain(["flow", "flow-root"]) {
        forms.insert(format!("{value} list-item"));
        forms.insert(format!("list-item {value}"));
    }
    for outer in outside {
        for inner in ["flow", "flow-root"] {
            for [a, b, c] in [
                [outer, inner, "list-item"],
                [outer, "list-item", inner],
                [inner, outer, "list-item"],
                [inner, "list-item", outer],
                ["list-item", outer, inner],
                ["list-item", inner, outer],
            ] {
                forms.insert(format!("{a} {b} {c}"));
            }
        }
    }
    for value in [
        "table-row-group",
        "table-header-group",
        "table-footer-group",
        "table-row",
        "table-cell",
        "table-column-group",
        "table-column",
        "table-caption",
        "ruby-base",
        "ruby-text",
        "ruby-base-container",
        "ruby-text-container",
        "contents",
        "none",
        "inline-block",
        "inline-table",
        "inline-flex",
        "inline-grid",
        "grid-lanes",
        "inline-grid-lanes",
    ] {
        forms.insert(value.into());
    }
    forms
}

#[test]
fn complete_selected_display_grammar_is_retained() {
    let forms = ordinary_forms();
    assert_eq!(forms.len(), 112);
    assert_eq!(forms.iter().filter(|form| !form.contains(' ')).count(), 30);
    for value in forms {
        let source = format!("display:{value}!important");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{value}");
        let declaration = &report.syntax()[0];
        let CssKnownPropertyValueRef::Display(display) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("display ordinary value")
        };
        assert_eq!(display.as_css(), value);
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
}

#[test]
fn invalid_display_category_combinations_reject_only_the_bad_declaration() {
    for value in [
        "block inline",
        "flow flex",
        "block block",
        "list-item list-item",
        "list-item flex",
        "list-item ruby",
        "inline-block flow",
        "none block",
        "contents list-item",
        "table-cell block",
        "block,flex",
        "block/flow",
        "inherit block",
        "inline unknown",
        "",
        "block grid-lanes",
        "grid-lanes block",
        "inline grid-lanes",
        "grid-lanes inline",
        "run-in grid-lanes",
        "grid-lanes run-in",
        "inline-grid-lanes block",
        "grid-lanes list-item",
        "grid-lanes flow",
        "grid-lanes grid-lanes",
        "masonry",
        "inline-masonry",
    ] {
        let report = parse_style_attribute(&format!("color:red;display:{value};color:blue"));
        let [diagnostic] = report.diagnostics() else {
            panic!("one drop for {value}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2, "{value}");
    }
}

#[test]
fn checked_component_construction_uses_the_same_display_grammar() {
    for value in ordinary_forms() {
        let mut tokens = Vec::new();
        for word in value.split_whitespace() {
            if !tokens.is_empty() {
                tokens.push(CssComponentValue::try_token(" ").unwrap());
            }
            tokens.push(CssComponentValue::try_token(word).unwrap());
        }
        let components = CssComponentValues::try_new(tokens).unwrap();
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Display),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert!(declaration.parsed_value().is_none());
        assert_eq!(declaration.value_components(), &components);
    }
}

#[test]
fn already_admitted_display_values_have_intrinsic_expansion() {
    assert!(CssKnownProperty::Display.metadata().is_ok());
    for value in [
        "block",
        "flex",
        "grid",
        "inline-block",
        "inline-grid",
        "none",
        "grid-lanes",
        "inline-grid-lanes",
    ] {
        let report = parse_style_attribute(&format!("display:{value}"));
        assert!(report.is_clean());
        assert!(expand_declaration(&report.syntax()[0]).is_ok(), "{value}");
    }
}

#[test]
fn selected_grid_lanes_keywords_preserve_frozen_identity_and_authored_spelling() {
    for (source, expected) in [
        ("grid-lanes", CssDisplay::GridLanes),
        ("GRID-LANES", CssDisplay::GridLanes),
        (r"gr\69 d-lanes", CssDisplay::GridLanes),
        ("inline-grid-lanes", CssDisplay::InlineGridLanes),
        ("INLINE-GRID-LANES", CssDisplay::InlineGridLanes),
        (r"inline-gr\69 d-lanes", CssDisplay::InlineGridLanes),
    ] {
        for (suffix, importance) in [
            ("", CssImportance::Normal),
            ("!important", CssImportance::Important),
        ] {
            let report = parse_style_attribute(&format!("display:{source}{suffix}"));
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let declaration = &report.syntax()[0];
            let CssKnownPropertyValueRef::Display(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("display value")
            };
            assert_eq!(value.i01_subset(), Some(&expected));
            assert_eq!(value.as_css(), source);
            assert_eq!(declaration.importance(), importance);
            assert!(matches!(
                declaration.value_components().items()[0].origin(),
                CssValueOrigin::Parsed(_)
            ));
        }
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(source).unwrap()])
                .unwrap();
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Display),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert!(declaration.parsed_value().is_none());
        assert_eq!(declaration.value_components(), &components);
        let CssKnownPropertyValueRef::Display(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("display value")
        };
        assert_eq!(value.i01_subset(), Some(&expected));
    }
}
