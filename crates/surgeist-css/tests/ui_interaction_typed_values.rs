#![forbid(unsafe_code)]
//! Functional evidence for the newly exposed UI borrowed longhand views and
//! logical Resize variants. UI4 WD20260120 selects the authored domains; the
//! shared records own general metadata, initials, provenance and alias inventory.
//! These new symbols receive functional coverage alongside implementation,
//! while ui_interaction_authored_contract.rs retains the published callable RED.

use surgeist_css::*;

fn declaration(name: &str, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::from_name(name).unwrap()),
        parse_component_values(text).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

fn exact_payload(source: &CssDeclaration, ordinary: &CssLonghandValue) {
    assert_eq!(
        ordinary.property().known_property(),
        source.known().unwrap().property()
    );
    match (
        source.known().unwrap().property_value().unwrap(),
        ordinary.view(),
    ) {
        (
            CssKnownPropertyValueRef::CaretColor(expected),
            CssLonghandValueRef::CaretColor(actual),
        ) => {
            assert_eq!(actual, expected.caret());
        }
        (CssKnownPropertyValueRef::Cursor(expected), CssLonghandValueRef::Cursor(actual)) => {
            assert_eq!(actual, expected.value());
        }
        (
            CssKnownPropertyValueRef::PointerEvents(expected),
            CssLonghandValueRef::PointerEvents(actual),
        ) => {
            assert_eq!(actual, expected.value());
        }
        (
            CssKnownPropertyValueRef::UserSelect(expected),
            CssLonghandValueRef::UserSelect(actual),
        ) => {
            assert_eq!(actual, expected.value());
        }
        (CssKnownPropertyValueRef::Resize(expected), CssLonghandValueRef::Resize(actual)) => {
            assert_eq!(actual, expected.resize());
        }
        _ => panic!("property-specific UI borrowed projection"),
    }
}

#[test]
fn logical_resize_variants_emit_authored_axes_and_enforce_exact_primitive_limits() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for (value, expected) in [(CssResize::Block, "block"), (CssResize::Inline, "inline")] {
        let before = value;
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
            (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
        }
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        for source in [
            declaration("resize", expected),
            parse_style_attribute(&format!("resize:{expected}!important")).syntax()[0].clone(),
        ] {
            let CssKnownPropertyValueRef::Resize(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("logical resize keyword")
            };
            assert_eq!(*wrapper.resize(), value);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("one logical Resize terminal")
            };
            let [item] = values.items() else {
                panic!("one Resize")
            };
            let CssLonghandValueRef::Resize(actual) = item.ordinary_value().unwrap().view() else {
                panic!("typed Resize projection")
            };
            assert_eq!(*actual, value);
        }
    }
}

#[test]
fn new_ui_borrowed_views_preserve_complete_authored_payloads_and_occurrences() {
    let report = parse_style_attribute(
        "caret-color:contrast-color(hsl(0 100% 50%))!important;cursor:image-set(\"first.png\" type(\"not a MIME type\") 96.000dpi,url(second.png)) -3.500 +4.250, pointer!important;pointer-events:none!important;-webkit-user-select:contain!important;resize:inline!important",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    for source in report.syntax().iter() {
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(source).unwrap()
        else {
            panic!("ordinary UI terminal")
        };
        let [item] = values.items() else {
            panic!("one UI terminal")
        };
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
        exact_payload(source, item.ordinary_value().unwrap());
    }
    assert_eq!(report, before);
}

#[test]
fn new_ui_borrowed_views_retain_replacement_graphs_under_original_occurrences() {
    for (name, text) in [
        ("caret-color", "device-cmyk(0% 81% 81% 30%)"),
        (
            "cursor",
            "image-set(\"first.png\" type(\"image/png\") 96.000dpi,url(second.png)) calc(1 + 2) -.25, auto",
        ),
        ("pointer-events", "none"),
        ("user-select", "all"),
        ("resize", "block"),
    ] {
        let source = declaration(name, "var(--ui)");
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending UI occurrence")
        };
        let replacement = parse_component_values(text).unwrap();
        let snapshot = replacement.clone();
        let expected = parse_property_value_for_grammar(
            source.known().unwrap().grammar(),
            replacement.clone(),
            CssImportance::Important,
        )
        .unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed UI replacement")
        };
        let [item] = values.items() else {
            panic!("one UI replacement")
        };
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        exact_payload(&expected, item.ordinary_value().unwrap());
        assert_eq!(replacement, snapshot);
        assert_eq!(source, before);
    }
}
