#![forbid(unsafe_code)]
//! WM4 CR2019-07-30 §9.1: digits <integer>?, literal 2..4, inherited, initial none.
//! Values4 WD2024-03-12 §§5.2/10.9/10.12: integer-token literals;
//! number-valued math remains authored valid without computed rounding/clamping.
use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::TextCombineUpright;
const ORDINARY: &[&str] = &[
    "digits",
    "digits 2",
    "digits 3",
    "digits 4",
    "digits +0002",
    "DIGITS 4",
    r"d\69 gits 3",
];

fn parsed(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("text-combine-upright:{text}!important"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn checked(text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let source = parse_property_value(
        CssPropertyNameRef::Known(PROPERTY),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(source.value_components(), &components);
    source
}

fn assert_ordinary(source: &CssDeclaration, text: &str) {
    let CssKnownPropertyValueRef::TextCombineUpright(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("known ordinary text-combine-upright")
    };
    assert_eq!(value.as_css(), text);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(matches!(
        source.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn parsed_digits_forms_are_known_ordinary_values() {
    for text in ORDINARY {
        assert_ordinary(&parsed(text), text);
    }
}

#[test]
fn checked_digits_forms_retain_supplied_components() {
    for text in ORDINARY {
        assert_ordinary(&checked(text), text);
    }
}

#[test]
fn integer_math_is_admitted_without_specified_range_or_rounding() {
    for text in [
        "digits calc(1)",
        "digits calc(5)",
        "digits calc(2.5)",
        "digits calc(1 + 2)",
        "digits min(1, 5)",
        "digits calc(1px / 1em)",
        "digits calc(infinity)",
        "digits calc(NaN)",
    ] {
        assert_ordinary(&parsed(text), text);
        assert_ordinary(&checked(text), text);
    }
}

#[test]
fn inherited_longhand_metadata_has_an_ordinary_initial() {
    let metadata = PROPERTY.metadata().expect("text-combine-upright metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), PROPERTY);
    assert!(matches!(initial.view(), CssInitialValueRef::Value(_)));
}

#[test]
fn existing_keywords_and_globals_expand_once_with_source_importance() {
    for text in ["none", "all", "inherit"] {
        let source = parsed(text);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).expect("intrinsic longhand expansion")
        else {
            panic!("longhands")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert_eq!(value.property(), PROPERTY);
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert!(value.replacement_components().is_none());
        if text == "inherit" {
            assert_eq!(
                value.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
            );
        } else {
            assert_eq!(
                value.ordinary_value().unwrap().property().known_property(),
                PROPERTY
            );
        }
    }
}

#[test]
fn pending_digits_reentry_preserves_property_and_replacement_occurrence() {
    let source = parsed("var(--combine)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending known property")
    };
    for text in ["digits", "digits 4", "digits calc(2.5)"] {
        let replacement = parse_component_values(text).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("longhands")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert_eq!(value.property(), PROPERTY);
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
        assert_eq!(
            value.ordinary_value().unwrap().property().known_property(),
            PROPERTY
        );
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for text in [
        "digits 1",
        "digits 5",
        "digits 2.0",
        "digits 2 extra",
        "inherit extra",
    ] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
}

#[test]
fn existing_keyword_controls_keep_exact_views_and_programmatic_origin() {
    for (text, expected) in [
        ("none", CssTextCombineUpright::None),
        ("all", CssTextCombineUpright::All),
    ] {
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(PROPERTY),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        assert_eq!(checked.importance(), CssImportance::Normal);
        assert!(matches!(
            components.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        for source in [parsed(text), checked] {
            let CssKnownPropertyValueRef::TextCombineUpright(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("typed keyword")
            };
            assert_eq!(value.combine(), &expected);
            assert_eq!(value.as_css(), text);
        }
    }
}

#[test]
fn invalid_ordinary_counts_and_math_types_are_rejected_and_recovered() {
    for text in [
        "digits 1",
        "digits 5",
        "digits -2",
        "digits 999999999999999999999",
        "digits 2.0",
        "digits 2e0",
        "digits 2%",
        "digits 2px",
        "digits 2 3",
        "digits none",
        "all 2",
        "none all",
        "digits calc(2px)",
        "digits calc(2%)",
    ] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(PROPERTY),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
        let report = parse_style_attribute(&format!(
            "text-combine-upright:{text};text-combine-upright:none"
        ));
        assert!(!report.is_clean(), "{text}");
        let [remaining] = report.syntax().as_slice() else {
            panic!("only valid sibling survives: {text}")
        };
        let CssKnownPropertyValueRef::TextCombineUpright(value) =
            remaining.known().unwrap().property_value().unwrap()
        else {
            panic!("valid known sibling")
        };
        assert_eq!(value.combine(), &CssTextCombineUpright::None);
    }
}

#[test]
fn all_reset_includes_text_combine_upright() {
    let report = parse_style_attribute("all:initial!important");
    assert!(report.is_clean());
    let [source] = report.syntax().as_slice() else {
        panic!("all declaration")
    };
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(source).unwrap()
    else {
        panic!("universal reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(PROPERTY)));
    assert!(reset.source().same_occurrence(source));
    assert_eq!(reset.source().importance(), CssImportance::Important);
}
