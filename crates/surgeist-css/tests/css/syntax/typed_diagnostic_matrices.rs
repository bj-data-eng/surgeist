//! Malformed sheets and property-family crossovers report the exact
//! diagnostic kind and authored identity responsible for the rejection.

use surgeist_css::values::animation::{CssAnimation, CssAnimationComponents, CssTransitionList};
use surgeist_css::values::font::CssFontFamilyList;
use surgeist_css::values::layout::CssGridGeneralTrackList;
use surgeist_css::values::position::{
    CssHorizontalPosition, CssPhysicalPosition, CssVerticalPosition,
};

use crate::css::support::sheet_cases::*;

#[test]
fn invalid_mixed_declarations_emit_expected_typed_diagnostics() {
    assert_sheet_rejected(
        ".panel { width: 10px; display: block inline; }",
        &ExpectedErrorKind::InvalidPropertyValue {
            property: "display",
        },
    );
    assert_sheet_rejected(
        ".panel { width: inherit 10px; height: 20px; }",
        &ExpectedErrorKind::InvalidPropertyValue { property: "width" },
    );
}

#[test]
fn malformed_sheet_surfaces_emit_expected_typed_diagnostics() {
    assert_rejects_sheets(&[
        RejectedSheetCase {
            label: "invalid declaration after a valid declaration is diagnosed",
            input: ".panel { width: 10px; display: block inline; height: 20px; }",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "display",
            },
        },
        RejectedSheetCase {
            label: "invalid declaration before a valid declaration is diagnosed",
            input: ".panel { display: block inline; width: 10px; }",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "display",
            },
        },
        RejectedSheetCase {
            label: "unknown property is diagnosed",
            input: ".panel { widht: 10px; width: 20px; }",
            expected_error: ExpectedErrorKind::UnknownProperty { name: "widht" },
        },
        RejectedSheetCase {
            label: "unknown at-rule is diagnosed",
            input: "@unknown screen { .panel { width: 10px; } }",
            expected_error: ExpectedErrorKind::UnknownAtRule { name: "unknown" },
        },
        RejectedSheetCase {
            label: "invalid selector is diagnosed",
            input: "??? { width: 10px; }",
            expected_error: ExpectedErrorKind::InvalidSelector,
        },
        RejectedSheetCase {
            label: "malformed declaration block is diagnosed",
            input: ".panel { width 10px; height: 20px; }",
            expected_error: ExpectedErrorKind::UnexpectedToken,
        },
        RejectedSheetCase {
            label: "trailing junk after a value is diagnosed",
            input: ".panel { width: 10px solid; }",
            expected_error: ExpectedErrorKind::InvalidPropertyValue { property: "width" },
        },
        RejectedSheetCase {
            label: "invalid comma-list item is diagnosed",
            input: ".panel { transition-duration: 150ms, solid; }",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "transition-duration",
            },
        },
        RejectedSheetCase {
            label: "invalid shorthand component is diagnosed",
            input: ".panel { border: 1px solid dotted; }",
            expected_error: ExpectedErrorKind::InvalidPropertyValue { property: "border" },
        },
    ]);
}

#[test]
fn malformed_authored_surfaces_emit_recovery_diagnostics() {
    for (css, rejection) in [
        (
            "@layer reset; @layer , theme; .ok { color: black; }",
            Rejection::prelude("layer", Some(",")),
        ),
        (
            "@scope (.card) { .ok { color: black; } @font-face extra { font-family: Test; src: local(Test); } } .after { color: blue; }",
            Rejection::prelude("font-face", Some("extra")),
        ),
        (
            ".ok { color: black; } .bad::before .later { color: red; } .after { color: blue; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".ok { content: \"good\"; content: normal \"bad\"; color: red; }",
            Rejection::property("content", Some("\"bad\"")),
        ),
        (
            ".ok { list-style: square inside; counter-reset: none item; color: red; }",
            Rejection::property("counter-reset", Some("item")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn leakage_wrong_keyword_and_unit_matrix_rejects_property_family_crossovers() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "display rejects border style keyword",
            property_name: "display",
            authored_value: "solid",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "display",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "overflow rejects none keyword",
            property_name: "overflow",
            authored_value: "none",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "overflow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "align-items rejects content distribution keyword",
            property_name: "align-items",
            authored_value: "space-between",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "align-items",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "padding rejects auto keyword",
            property_name: "padding",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "padding",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-width rejects percentage",
            property_name: "border-width",
            authored_value: "10%",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "border-width",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-color rejects border style keyword",
            property_name: "border-color",
            authored_value: "solid",
            expected_error: ExpectedErrorKind::InvalidColorSyntax {
                encountered: "solid",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "font-size rejects auto keyword",
            property_name: "font-size",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "font-size",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "grid-auto-flow rejects position keyword",
            property_name: "grid-auto-flow",
            authored_value: "left",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "grid-auto-flow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "cursor rejects length",
            property_name: "cursor",
            authored_value: "10px",
            expected_error: ExpectedErrorKind::InvalidPropertyValue { property: "cursor" },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "transition-duration rejects length unit",
            property_name: "transition-duration",
            authored_value: "10px",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "transition-duration",
            },
            property_name_should_be_recognized: true,
        },
    ]);
}

#[test]
fn rejection_malformed_functions_lists_and_shorthands_matrix() {
    for input in [
        ".panel { width: calc(10px + ); }",
        ".panel { width: calc(10px * 2px); }",
        ".panel { transform: translate(red); }",
        ".panel { filter: opacity(red); }",
        ".panel { clip-path: polygon(0 0, ); }",
        ".panel { transition-timing-function: cubic-bezier(0.1, red, 0.3, 1); }",
        ".panel { font-family: sans-serif,; }",
        ".panel { background-image: none,; }",
        ".panel { transition-property: opacity,; }",
        ".panel { animation-name: fade,; }",
        ".panel { border: 1px 2px solid; }",
        ".panel { box-shadow: inset inset 1px 2px; }",
        ".panel { text-decoration: underline underline; }",
        ".panel { transition: opacity 1s 2s 3s; }",
        ".panel { animation: fade 1s 2s 3s; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}

#[test]
fn rejection_negative_numbers_and_public_constructor_invariants_matrix() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "flex-grow rejects negative numbers",
            property_name: "flex-grow",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "flex-grow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "flex-shrink rejects negative numbers",
            property_name: "flex-shrink",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "flex-shrink",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "padding rejects negative lengths",
            property_name: "padding",
            authored_value: "-1px",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "padding",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-radius rejects negative lengths",
            property_name: "border-radius",
            authored_value: "-1px",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "border-radius",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "animation-iteration-count rejects negative numbers",
            property_name: "animation-iteration-count",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "animation-iteration-count",
            },
            property_name_should_be_recognized: true,
        },
    ]);

    assert_eq!(CssFontFamilyList::try_new(Vec::new()), None);
    assert_eq!(CssGridGeneralTrackList::try_new(Vec::new()), None);
    let offset = signed_length_percentage("1px");
    assert!(
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::LeftOffset(offset),
            CssVerticalPosition::Top
        )
        .is_err()
    );
    assert_eq!(CssTransitionList::try_new(Vec::new()), None);
    assert_eq!(
        CssAnimation::try_new(CssAnimationComponents::default()),
        None
    );
}

#[test]
fn rejection_unsupported_but_syntactically_valid_css_keywords_stay_rejected() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "overflow none remains unsupported",
            property_name: "overflow",
            authored_value: "none",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "overflow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "background-origin margin-box remains unsupported",
            property_name: "background-origin",
            authored_value: "margin-box",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "background-origin",
            },
            property_name_should_be_recognized: true,
        },
    ]);
}
