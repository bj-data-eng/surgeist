#![forbid(unsafe_code)]

//! Public semantic accessors preserve authored identity and symbolic branches.
use surgeist_css::*;

fn parsed(property: CssKnownProperty, css: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{}:{css}", property.canonical_name()));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn all_keeps_only_global_and_substitution_dependent_declared_values() {
    for (css, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let declaration = parsed(CssKnownProperty::All, css);
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), CssKnownProperty::All);
        assert_eq!(known.grammar(), CssKnownProperty::All.grammar());
        assert_eq!(known.global(), Some(keyword));
        assert_eq!(known.property_value(), None);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::All),
            parse_component_values(css).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.known().unwrap().global(), Some(keyword));
        assert_eq!(checked.known().unwrap().property_value(), None);
    }
    let declaration = parsed(CssKnownProperty::All, "var(--reset)");
    let known = declaration.known().unwrap();
    assert!(known.substitution_dependent().is_some());
    assert_eq!(known.property_value(), None);
    assert!(!parse_style_attribute("all:normal").is_clean());
}

#[test]
fn track_wrappers_expose_semantics_without_widening_the_selected_grammar() {
    let position = |overflow, position| CssAlignmentValue::Position { overflow, position };
    for property in [
        CssKnownProperty::JustifyTracks,
        CssKnownProperty::AlignTracks,
    ] {
        for (css, expected) in [
            ("normal", CssAlignmentValue::Normal { overflow: None }),
            ("start", position(None, CssAlignmentPosition::Start)),
            (
                "flex-start",
                position(None, CssAlignmentPosition::FlexStart),
            ),
            (
                "safe end",
                position(Some(CssOverflowPosition::Safe), CssAlignmentPosition::End),
            ),
            (
                "safe flex-end",
                position(
                    Some(CssOverflowPosition::Safe),
                    CssAlignmentPosition::FlexEnd,
                ),
            ),
            (
                "safe center",
                position(
                    Some(CssOverflowPosition::Safe),
                    CssAlignmentPosition::Center,
                ),
            ),
            (
                "first baseline",
                CssAlignmentValue::Baseline(CssBaselinePosition::First),
            ),
            (
                "last baseline",
                CssAlignmentValue::Baseline(CssBaselinePosition::Last),
            ),
            ("space-between", CssAlignmentValue::SpaceBetween),
            ("space-around", CssAlignmentValue::SpaceAround),
            ("space-evenly", CssAlignmentValue::SpaceEvenly),
            ("stretch", CssAlignmentValue::Stretch),
        ] {
            let declaration = parsed(property, css);
            let (authored, actual) = match declaration.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::JustifyTracks(value) => (value.as_css(), value.value()),
                CssKnownPropertyValueRef::AlignTracks(value) => (value.as_css(), value.value()),
                _ => panic!("track wrapper"),
            };
            assert_eq!(authored, css);
            assert_eq!(actual, &expected);
        }
        for css in [
            "unsafe end",
            "unsafe center",
            "unsafe flex-end",
            "safe start",
            "safe flex-start",
            "safe baseline",
            "safe first baseline",
            "safe last baseline",
            "safe normal",
            "safe stretch",
            "safe space-between",
            "safe space-around",
            "safe space-evenly",
        ] {
            let report = parse_style_attribute(&format!("{}:{css}", property.canonical_name()));
            assert!(!report.is_clean(), "{property:?}: {css}");
            assert_eq!(report.diagnostics().len(), 1);
            assert_eq!(
                report.diagnostics()[0].error().code(),
                CssErrorCode::InvalidPropertyValue
            );
        }
    }
}

#[test]
fn sole_semantic_values_preserve_authored_trivia_exact_numbers_and_omission() {
    let declaration = parsed(
        CssKnownProperty::CounterReset,
        " chapter /* gap */ section -9007199254740993 ",
    );
    let CssKnownPropertyValueRef::CounterReset(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("counter wrapper");
    };
    assert_eq!(
        value.as_css(),
        "chapter /* gap */ section -9007199254740993"
    );
    let [first, second] = value.value().changes().unwrap() else {
        panic!("two ordered changes");
    };
    assert_eq!(first.name().as_str(), "chapter");
    assert!(first.value().is_none());
    assert_eq!(second.name().as_str(), "section");
    let Some(CssIntegerValue::Literal(integer)) = second.value() else {
        panic!("exact integer");
    };
    assert_eq!(integer.numeric().representation(), "-9007199254740993");

    let declaration = parsed(CssKnownProperty::FlexGrow, "1e100");
    let CssKnownPropertyValueRef::FlexGrow(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("factor wrapper");
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
        value.factor().literal_component().unwrap().view()
    else {
        panic!("exact number");
    };
    assert_eq!(number.representation(), "1e100");

    let declaration = parsed(CssKnownProperty::Overflow, " hidden ");
    let CssKnownPropertyValueRef::Overflow(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("overflow wrapper");
    };
    assert_eq!(value.as_css(), "hidden");
    assert_eq!(value.value().x(), CssOverflow::Hidden);
    assert_eq!(value.value().authored_y(), None);
    assert_eq!(value.value().y(), CssOverflow::Hidden);
}
