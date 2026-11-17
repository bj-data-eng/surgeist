#![forbid(unsafe_code)]

//! Every public error type implements `Display` with a semantic message and
//! `core::error::Error`. The expected texts are written from each type's
//! documented meaning: the name shapes in `name`, the span rules in `span`, the
//! strict syntax rules in `parse`, the registry and validation rules, and the
//! DOM adapter's preflight contract. Line and column numbers are counted from
//! the source text of each case.

use core::error::Error;

use surgeist_template::ast::{AstConstructionError, Node};
use surgeist_template::dom::{TemplateDomError, TemplateDomUnresolvedKind};
use surgeist_template::expr::VariablePath;
use surgeist_template::name::{
    AttributeName, ComponentName, NameError, NativeElementName, VariableName,
};
use surgeist_template::parse::{ParseError, ParseErrorKind};
use surgeist_template::registry::{
    AttributeKind, AttributeRule, AttributeSpec, ComponentRegistry, ComponentSpec,
    NativeElementRegistry, NativeElementSpec, RegistryError,
};
use surgeist_template::span::{SourcePos, SourceSpan, SpanError};
use surgeist_template::validation::{ValidationError, ValidationErrorKind};
use surgeist_template::{parse_template, validate_template};

fn assert_error<E: Error + Send + Sync + 'static>() {}

#[test]
fn every_public_error_type_implements_the_error_trait() {
    assert_error::<AstConstructionError>();
    assert_error::<NameError>();
    assert_error::<ParseError>();
    assert_error::<RegistryError>();
    assert_error::<SpanError>();
    assert_error::<TemplateDomError>();
    assert_error::<ValidationError>();
}

fn position(line: usize, column: usize, byte: usize) -> SourcePos {
    SourcePos::try_new(line, column, byte).unwrap()
}

fn span(start: (usize, usize, usize), end: (usize, usize, usize)) -> SourceSpan {
    SourceSpan::try_new(
        position(start.0, start.1, start.2),
        position(end.0, end.1, end.2),
    )
    .unwrap()
}

fn check(error: &impl Error, expected: &str) {
    assert_eq!(error.to_string(), expected);
    assert_ne!(
        error.to_string(),
        format!("{error:?}"),
        "Display must not repeat the Debug form"
    );
    assert!(error.source().is_none(), "{expected} wraps no error");
}

fn spec(name: &str, kind: AttributeKind) -> AttributeSpec {
    AttributeSpec::try_new(name, AttributeRule::one(kind)).unwrap()
}

#[test]
fn ast_construction_error_says_an_if_block_needs_a_branch() {
    let error = Node::if_block(Vec::new(), Vec::new(), span((1, 1, 0), (1, 1, 0))).unwrap_err();
    assert_eq!(error, AstConstructionError::EmptyIfBranches);
    check(&error, "an if block requires at least one branch");
}

#[test]
fn name_errors_state_the_rejected_text_and_the_shape_each_name_requires() {
    check(
        &AttributeName::try_new("-data").unwrap_err(),
        "invalid attribute name \"-data\": expected an ASCII letter or underscore followed by \
         ASCII letters, digits, underscores or hyphens",
    );
    check(
        &NativeElementName::try_new("Div").unwrap_err(),
        "invalid native element name \"Div\": expected a lowercase ASCII letter followed by \
         lowercase ASCII letters, digits, underscores or hyphens",
    );
    check(
        &ComponentName::try_new("panelHeader").unwrap_err(),
        "invalid component name \"panelHeader\": expected an uppercase ASCII letter followed by \
         ASCII letters, digits or underscores",
    );
    check(
        &VariableName::try_new("2item").unwrap_err(),
        "invalid variable name \"2item\"",
    );
}

#[test]
fn variable_name_errors_also_cover_reserved_roots_without_claiming_a_shape() {
    check(
        &VariablePath::try_new("GLOBALS", Vec::new()).unwrap_err(),
        "invalid variable name \"GLOBALS\"",
    );
}

#[test]
fn name_error_escapes_control_characters_in_the_rejected_text() {
    check(
        &NativeElementName::try_new("bad\n").unwrap_err(),
        "invalid native element name \"bad\\n\": expected a lowercase ASCII letter followed by \
         lowercase ASCII letters, digits, underscores or hyphens",
    );
}

#[test]
fn span_errors_name_the_violated_coordinate_rule() {
    check(
        &SourcePos::try_new(0, 1, 0).unwrap_err(),
        "invalid source position: line 0, column 1; lines and columns start at 1",
    );
    check(
        &SourceSpan::try_new(position(2, 5, 14), position(1, 9, 18)).unwrap_err(),
        "source span ends at line 1, column 9 (byte 18) before it starts at line 2, column 5 \
         (byte 14)",
    );
    // Order is decided by byte offset: this end has the later column but the earlier byte.
    check(
        &SourceSpan::try_new(position(2, 5, 14), position(2, 9, 12)).unwrap_err(),
        "source span ends at line 2, column 9 (byte 12) before it starts at line 2, column 5 \
         (byte 14)",
    );
    check(
        &SourceSpan::try_new(position(1, 1, 0), position(1, 2, 0)).unwrap_err(),
        "source span from line 1, column 1 to line 1, column 2 has equal byte offsets but \
         different positions",
    );
}

#[test]
fn registry_errors_name_the_duplicate_or_wrap_the_invalid_name() {
    check(
        &NativeElementSpec::try_new(
            "div",
            vec![
                spec("class", AttributeKind::Static),
                spec("class", AttributeKind::Bool),
            ],
        )
        .unwrap_err(),
        "attribute \"class\" is specified more than once for one element",
    );
    let div = || NativeElementSpec::try_new("div", Vec::new()).unwrap();
    check(
        &NativeElementRegistry::try_from_specs(vec![div(), div()]).unwrap_err(),
        "native element \"div\" is registered more than once",
    );
    let panel = || ComponentSpec::try_new("Panel", Vec::new()).unwrap();
    check(
        &ComponentRegistry::try_from_specs(vec![panel(), panel()]).unwrap_err(),
        "component \"Panel\" is registered more than once",
    );
}

#[test]
fn registry_name_error_keeps_its_detail_in_the_source_not_the_message() {
    let error = NativeElementSpec::try_new("Div", Vec::new()).unwrap_err();
    assert!(matches!(error, RegistryError::Name(_)));
    assert_eq!(error.to_string(), "invalid name in registry specification");
    let source = error
        .source()
        .expect("the wrapped name error is the source");
    assert_eq!(
        source.to_string(),
        "invalid native element name \"Div\": expected a lowercase ASCII letter followed by \
         lowercase ASCII letters, digits, underscores or hyphens"
    );
    assert!(source.source().is_none());
}

#[test]
fn parse_error_names_the_violation_and_where_it_starts() {
    // Columns count characters from 1; `<a></b>` places the close name at column 6.
    let cases = [
        (
            "<div>text",
            "element \"div\" is not closed at line 1, column 1",
        ),
        (
            "<a></b>",
            "closing tag \"b\" does not match open element \"a\" at line 1, column 6",
        ),
        (
            "{foo bar}",
            "template tag \"foo\" is not supported at line 1, column 1",
        ),
        (
            "a\n  {else}",
            "template tag \"else\" has no matching opening tag at line 2, column 3",
        ),
        (
            "<",
            "unexpected token, expected element name at line 1, column 2",
        ),
    ];
    for (source, expected) in cases {
        let error = parse_template(source).unwrap_err();
        check(&error, expected);
    }
}

#[test]
fn parse_error_kinds_have_semantic_messages() {
    let cases = [
        (
            ParseErrorKind::UnexpectedEof,
            "unexpected end of template source",
        ),
        (
            ParseErrorKind::UnexpectedToken {
                expected: "attribute boundary",
            },
            "unexpected token, expected attribute boundary",
        ),
        (
            ParseErrorKind::UnclosedElement {
                name: "Panel".to_owned(),
            },
            "element \"Panel\" is not closed",
        ),
        (
            ParseErrorKind::MismatchedCloseTag {
                expected: "div".to_owned(),
                found: "span".to_owned(),
            },
            "closing tag \"span\" does not match open element \"div\"",
        ),
        (
            ParseErrorKind::InvalidName {
                name: "1x".to_owned(),
            },
            "\"1x\" is not a valid element or attribute name",
        ),
        (
            ParseErrorKind::InvalidExpression {
                reason: "empty expression",
            },
            "invalid expression: empty expression",
        ),
        (
            ParseErrorKind::StrayTemplateTag {
                tag: "/if".to_owned(),
            },
            "template tag \"/if\" has no matching opening tag",
        ),
        (
            ParseErrorKind::UnsupportedTemplateTag {
                tag: "include".to_owned(),
            },
            "template tag \"include\" is not supported",
        ),
    ];
    for (kind, expected) in cases {
        assert_eq!(kind.to_string(), expected);
    }
}

fn validation_error(source: &str) -> ValidationError {
    let native = NativeElementRegistry::try_from_specs(vec![
        NativeElementSpec::try_new("div", vec![spec("class", AttributeKind::Static)]).unwrap(),
    ])
    .unwrap();
    let document = parse_template(source).unwrap();
    validate_template(&document, &native, &ComponentRegistry::default()).unwrap_err()
}

#[test]
fn validation_error_names_the_violation_and_where_it_starts() {
    // Each attribute's span starts at its name; `<div class="a" class="b"/>`
    // puts the second `class` at column 16.
    let cases = [
        (
            "<section/>",
            "unknown native element \"section\" at line 1, column 1",
        ),
        (
            "<Widget/>",
            "unknown component \"Widget\" at line 1, column 1",
        ),
        (
            "<div class=\"a\" class=\"b\"/>",
            "attribute \"class\" appears more than once on one element at line 1, column 16",
        ),
        (
            "<div id=\"x\"/>",
            "attribute \"id\" is not allowed on \"div\" at line 1, column 6",
        ),
        (
            "<div class={$x}/>",
            "attribute \"class\" on \"div\" does not accept this value form at line 1, column 6",
        ),
    ];
    for (source, expected) in cases {
        let error = validation_error(source);
        check(&error, expected);
    }
}

#[test]
fn validation_error_kinds_have_semantic_messages() {
    let cases = [
        (
            ValidationErrorKind::UnknownNativeElement {
                name: "blink".to_owned(),
            },
            "unknown native element \"blink\"",
        ),
        (
            ValidationErrorKind::UnknownComponent {
                name: "Gauge".to_owned(),
            },
            "unknown component \"Gauge\"",
        ),
        (
            ValidationErrorKind::DuplicateAttribute {
                name: "title".to_owned(),
            },
            "attribute \"title\" appears more than once on one element",
        ),
        (
            ValidationErrorKind::InvalidAttribute {
                element: "Panel".to_owned(),
                attribute: "tone".to_owned(),
            },
            "attribute \"tone\" is not allowed on \"Panel\"",
        ),
        (
            ValidationErrorKind::InvalidAttributeValue {
                element: "Panel".to_owned(),
                attribute: "title".to_owned(),
            },
            "attribute \"title\" on \"Panel\" does not accept this value form",
        ),
    ];
    for (kind, expected) in cases {
        assert_eq!(kind.to_string(), expected);
    }
}

#[test]
fn template_dom_errors_name_what_blocked_the_concrete_dom() {
    let at = span((2, 3, 10), (2, 8, 15));
    let unresolved = |kind| TemplateDomError::Unresolved { span: at, kind };
    let cases = [
        (
            unresolved(TemplateDomUnresolvedKind::Component),
            "component element at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            unresolved(TemplateDomUnresolvedKind::Interpolation),
            "interpolation at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            unresolved(TemplateDomUnresolvedKind::Conditional),
            "conditional block at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            unresolved(TemplateDomUnresolvedKind::Iteration),
            "foreach block at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            unresolved(TemplateDomUnresolvedKind::AttributeExpression),
            "attribute expression at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            unresolved(TemplateDomUnresolvedKind::AttributeInterpolation),
            "interpolated attribute at line 2, column 3 is symbolic and cannot become a DOM node",
        ),
        (
            TemplateDomError::UnsupportedTemplateContentsTarget { span: at },
            "template element at line 2, column 3 needs a template-contents association that \
             is not supported",
        ),
        (
            TemplateDomError::Allocation,
            "allocation failed while preparing the DOM batch",
        ),
        (
            TemplateDomError::RepresentationExhausted,
            "the DOM command count exceeds the representable range",
        ),
    ];
    for (error, expected) in cases {
        check(&error, expected);
    }
}

#[test]
fn unresolved_kinds_have_semantic_messages() {
    let cases = [
        (TemplateDomUnresolvedKind::Component, "component element"),
        (TemplateDomUnresolvedKind::Interpolation, "interpolation"),
        (TemplateDomUnresolvedKind::Conditional, "conditional block"),
        (TemplateDomUnresolvedKind::Iteration, "foreach block"),
        (
            TemplateDomUnresolvedKind::AttributeExpression,
            "attribute expression",
        ),
        (
            TemplateDomUnresolvedKind::AttributeInterpolation,
            "interpolated attribute",
        ),
    ];
    for (kind, expected) in cases {
        assert_eq!(kind.to_string(), expected);
    }
}
