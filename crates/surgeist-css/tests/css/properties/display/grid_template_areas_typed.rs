use crate::css::support::failure_classes::SerializationFailure as SerializationError;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::{
    CssGridTemplateAreaCell as Cell, CssGridTemplateAreaName as Name,
    CssGridTemplateAreaRow as Row, CssGridTemplateAreas as Areas, CssInitialValueRef,
    CssKnownProperty, CssKnownPropertyValueRef, CssLonghandValueRef, CssPropertyKindRef,
    CssSerializationLimits as Limits, parse_style_attribute,
};

fn named(value: &str) -> Cell {
    Cell::Named(Name::try_new(value).expect("valid decoded area name"))
}

fn row(cells: Vec<Cell>) -> Row {
    Row::try_new(cells).expect("nonempty row")
}

#[test]
fn parsed_current_and_authored_text_remain_distinct_and_initial_is_none() {
    let authored = "\"1st...auto\\a A\\9 b\"";
    let report = parse_style_attribute(&format!("grid-template-areas:{authored}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::GridTemplateAreas(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed area wrapper")
    };
    assert_eq!(wrapper.as_css(), authored);
    let expected = Areas::try_rows(vec![row(vec![
        named("1st"),
        Cell::Empty,
        named("auto"),
        named("A"),
        named("b"),
    ])])
    .unwrap();
    assert_eq!(wrapper.value(), &expected);
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "\"1st . auto A b\""
    );

    for (authored, decoded) in [("\"a\\a0 b\"", "a\u{a0}b"), ("\"a\u{85}b\"", "a\u{85}b")] {
        let report = parse_style_attribute(&format!("grid-template-areas:{authored}"));
        assert!(
            report.is_clean(),
            "{authored:?}: {:?}",
            report.diagnostics()
        );
        let CssKnownPropertyValueRef::GridTemplateAreas(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("non-ASCII area wrapper")
        };
        assert_eq!(wrapper.as_css(), authored);
        assert_eq!(
            wrapper.value(),
            &Areas::try_rows(vec![row(vec![named(decoded)])]).unwrap()
        );
        assert_eq!(
            wrapper
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            format!("\"{decoded}\"")
        );
    }

    let ordinary = parse_style_attribute("grid-template-areas:\"A.b\"");
    assert!(ordinary.is_clean());
    let CssKnownPropertyValueRef::GridTemplateAreas(wrapper) = ordinary.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("ordinary area wrapper")
    };
    assert_eq!(
        wrapper.value(),
        &Areas::try_rows(vec![row(vec![named("A"), Cell::Empty, named("b")])]).unwrap()
    );

    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::GridTemplateAreas
        .metadata()
        .unwrap()
        .kind()
    else {
        panic!("area longhand metadata")
    };
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::GridTemplateAreas(initial) = initial.view() else {
        panic!("typed area initial")
    };
    assert_eq!(initial, &Areas::None);
}

#[test]
fn lexer_replacement_of_null_remains_an_ident_code_point() {
    for authored in ["\"a\\0 b\"", "\"a\0b\""] {
        let report = parse_style_attribute(&format!("grid-template-areas:{authored}"));
        assert!(
            report.is_clean(),
            "{authored:?}: {:?}",
            report.diagnostics()
        );
        let CssKnownPropertyValueRef::GridTemplateAreas(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed area wrapper")
        };
        assert_eq!(
            wrapper.value(),
            &Areas::try_rows(vec![row(vec![named("a\u{fffd}b")])]).unwrap()
        );
        assert_eq!(
            wrapper
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            "\"a\u{fffd}b\""
        );
    }
}

#[test]
fn decoded_name_domain_accepts_digits_keywords_case_and_non_ascii() {
    for value in [
        "1st",
        "auto",
        "span",
        "inherit",
        "revert-layer",
        "A",
        "a",
        "a\u{a0}b",
        "a\u{85}b",
    ] {
        assert_eq!(Name::try_new(value).unwrap().as_str(), value);
    }
    assert_ne!(Name::try_new("A").unwrap(), Name::try_new("a").unwrap());
    for value in [
        "", ".", "a.b", "a b", "a\tb", "a\nb", "a\rb", "a\u{c}b", "a#b", "a\\b", "a\"b",
    ] {
        assert!(
            Name::try_new(value).as_ref().unwrap_err().is_invalid_name(),
            "{value:?}"
        );
    }
}

#[test]
fn checked_rows_enforce_nonempty_equal_width_and_rectangles() {
    assert!(Row::try_new(vec![]).as_ref().unwrap_err().is_empty_row());
    assert!(
        Areas::try_rows(vec![])
            .as_ref()
            .unwrap_err()
            .is_missing_rows()
    );
    assert!(
        Areas::try_rows(vec![
            row(vec![named("a"), Cell::Empty]),
            row(vec![named("a")])
        ])
        .as_ref()
        .unwrap_err()
        .is_inconsistent_widths()
    );
    assert_eq!(
        Areas::try_rows(vec![
            row(vec![named("a"), named("a")]),
            row(vec![named("a"), Cell::Empty])
        ])
        .as_ref()
        .unwrap_err()
        .non_rectangular_area(),
        Some("a")
    );
    assert_eq!(
        Areas::try_rows(vec![row(vec![named("a"), Cell::Empty, named("a")])])
            .as_ref()
            .unwrap_err()
            .non_rectangular_area(),
        Some("a")
    );
    Areas::try_rows(vec![
        row(vec![named("a"), named("a")]),
        row(vec![named("a"), named("a")]),
    ])
    .expect("filled rectangle");
}

#[test]
fn canonical_serialization_uses_one_cell_separator_and_bounded_css_strings() {
    assert_eq!(
        Areas::None.serialize_specified(Default::default()).unwrap(),
        "none"
    );
    let areas = Areas::try_rows(vec![
        row(vec![named("a"), Cell::Empty, named("B")]),
        row(vec![named("auto"), named("a\u{a0}b"), named("a\u{85}b")]),
    ])
    .unwrap();
    let expected = "\"a . B\" \"auto a\u{a0}b a\u{85}b\"";
    assert_eq!(
        areas.serialize_specified(Default::default()).unwrap(),
        expected
    );
    let ordinary = Areas::try_rows(vec![row(vec![named("A"), Cell::Empty, named("b")])]).unwrap();
    assert_eq!(
        ordinary.serialize_specified(Default::default()).unwrap(),
        "\"A . b\""
    );
}

#[test]
fn specified_serialization_charges_each_node_and_exact_output_bytes() {
    let areas = Areas::try_rows(vec![row(vec![named("a"), Cell::Empty, named("b")])]).unwrap();
    let css = "\"a . b\"";
    assert_eq!(css.len(), 7);
    assert_eq!(
        areas.serialize_specified(Limits::new(5, 5, 7)).unwrap(),
        css
    );
    for (limits, expected) in [
        (Limits::new(4, 5, 7), SerializationError::InputNodeLimit),
        (
            Limits::new(5, 4, 7),
            SerializationError::ProjectionNodeLimit,
        ),
        (Limits::new(5, 5, 6), SerializationError::ByteLimit),
    ] {
        assert_eq!(
            serialization_failure(&areas.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    assert_eq!(
        Areas::None
            .serialize_specified(Limits::new(1, 1, 4))
            .unwrap(),
        "none"
    );
    assert!(
        Areas::None
            .serialize_specified(Limits::new(0, 1, 4))
            .unwrap_err()
            .is_input_node_limit()
    );
}
