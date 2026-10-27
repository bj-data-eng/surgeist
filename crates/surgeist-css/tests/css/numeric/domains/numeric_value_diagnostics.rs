//! Values 4 numeric property grammars report the first token they cannot
//! accept. Every public front reports `InvalidPropertyValue`, names that token
//! as encountered and positions the error at it.

use crate::css::support::property_admission::GrammarFacts;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssSerializedOrigin, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssTokenKind};
use surgeist_css::property::{
    CssKnownProperty, CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget,
};

/// The responsible token: its kind, authored spelling and offset in the value.
struct Responsible {
    kind: CssTokenKind,
    authored: &'static str,
    offset: usize,
}

fn assert_detail(
    property: CssKnownProperty,
    error: &dyn GrammarFacts,
    expected: &Responsible,
    front: &str,
) {
    assert_eq!(
        error.grammar_code(),
        Some(CssErrorCode::InvalidPropertyValue),
        "{front}: expected InvalidPropertyValue, got {error:?}"
    );
    assert_eq!(error.rejecting_property(), Some(property), "{front}");
    let encountered = error.responsible_token().expect("responsible token");
    assert_eq!(encountered.kind(), expected.kind, "{front}");
    assert_eq!(encountered.authored(), expected.authored, "{front}");
}

/// Checks the style-attribute, sheet, by-name, by-grammar and raw-text fronts.
fn assert_every_front(name: &str, value: &str, expected: Responsible) {
    let property = CssKnownProperty::from_name(name).unwrap();
    let declaration = format!("{name}:{value}");
    let value_offset = name.len() + 1;

    let report = CssParser::default().parse_block_contents(&declaration);
    let [diagnostic] = report.diagnostics() else {
        panic!("{declaration}: one diagnostic")
    };
    let front = format!("attribute {declaration}");
    assert_detail(property, diagnostic.error(), &expected, &front);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        value_offset + expected.offset,
        "{front}"
    );

    let sheet = format!(".a{{{declaration}}}");
    let failure = CssParser::default()
        .parse_sheet(&sheet)
        .into_validation_result()
        .unwrap_err();
    let [diagnostic] = failure.diagnostics() else {
        panic!("{sheet}: one diagnostic")
    };
    let front = format!("sheet {sheet}");
    assert_detail(property, diagnostic.error(), &expected, &front);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        3 + value_offset + expected.offset,
        "{front}"
    );

    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
    for (front, result) in [
        (
            "by-name",
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                components.clone(),
                CssImportance::Normal,
            ),
        ),
        (
            "by-grammar",
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(property.grammar()),
                components.clone(),
                CssImportance::Normal,
            ),
        ),
    ] {
        let front = format!("{front} {declaration}");
        let error = result.unwrap_err();
        assert!(error.code().is_some(), "{front}: grammar error");
        assert_detail(property, &error, &expected, &front);
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
            panic!("{front}: parsed token origin")
        };
        assert_eq!(
            origin.span().start().byte_offset().value(),
            expected.offset,
            "{front}"
        );
    }

    let text = CssParser::default().parse_property_value(
        value,
        CssPropertyTextTarget::Property(CssPropertyNameRef::Known(property)),
        CssImportance::Normal,
    );
    let [diagnostic] = text.diagnostics() else {
        panic!("{declaration}: one raw-text diagnostic")
    };
    let front = format!("text {declaration}");
    assert_detail(property, diagnostic.error(), &expected, &front);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        expected.offset,
        "{front}"
    );
}

fn ident(authored: &'static str, offset: usize) -> Responsible {
    Responsible {
        kind: CssTokenKind::Ident,
        authored,
        offset,
    }
}

fn function(authored: &'static str, offset: usize) -> Responsible {
    Responsible {
        kind: CssTokenKind::Function,
        authored,
        offset,
    }
}

#[test]
fn length_grammars_report_a_non_numeric_identifier_or_function_as_encountered() {
    // `<length>`, `<length [0,∞]>`, `<length-percentage>` and
    // `<length-percentage [0,∞]>` owners each share one numeric parser.
    for name in ["outline-offset", "perspective", "margin-top", "padding-top"] {
        assert_every_front(name, "foo", ident("foo", 0));
        assert_every_front(name, "foo()", function("foo(", 0));
    }
}

#[test]
fn outline_width_and_border_width_share_the_line_width_grammar() {
    // UI 4 `outline-width: <line-width>`; Backgrounds 3 `border-*-width: <line-width>`.
    for name in ["outline-width", "border-top-width"] {
        assert_every_front(name, "foo", ident("foo", 0));
        assert_every_front(name, "foo()", function("foo(", 0));
        // An unknown keyword is the responsible token, not the length after it.
        assert_every_front(name, "auto 1px", ident("auto", 0));
    }
}
