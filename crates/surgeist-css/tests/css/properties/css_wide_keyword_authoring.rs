//! CSS-wide keywords (Cascade 5 §7.3) are whole-value globals for every
//! property, including `all`.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

macro_rules! declaration_global {
    ($input:expr, $variant:ident) => {{
        let declaration = declaration($input, CssKnownProperty::$variant);
        let known = declaration.known().expect("known global declaration");
        assert!(known.property_value().is_none());
        known.global().expect("global declaration value")
    }};
}

#[test]
fn acceptance_css_wide_global_keyword_matrix_accepts_supported_globals() {
    let cases = [
        AcceptedDeclarationCase {
            label: "all inherit",
            property_name: "all",
            authored_value: "inherit",
            expected_property: CssKnownProperty::All,
            expected_global: CssGlobalKeyword::Inherit,
        },
        AcceptedDeclarationCase {
            label: "all initial",
            property_name: "all",
            authored_value: "initial",
            expected_property: CssKnownProperty::All,
            expected_global: CssGlobalKeyword::Initial,
        },
        AcceptedDeclarationCase {
            label: "all unset",
            property_name: "all",
            authored_value: "unset",
            expected_property: CssKnownProperty::All,
            expected_global: CssGlobalKeyword::Unset,
        },
        AcceptedDeclarationCase {
            label: "all revert",
            property_name: "all",
            authored_value: "revert",
            expected_property: CssKnownProperty::All,
            expected_global: CssGlobalKeyword::Revert,
        },
        AcceptedDeclarationCase {
            label: "all revert-layer",
            property_name: "all",
            authored_value: "revert-layer",
            expected_property: CssKnownProperty::All,
            expected_global: CssGlobalKeyword::RevertLayer,
        },
        AcceptedDeclarationCase {
            label: "display global initial",
            property_name: "display",
            authored_value: "initial",
            expected_property: CssKnownProperty::Display,
            expected_global: CssGlobalKeyword::Initial,
        },
        AcceptedDeclarationCase {
            label: "width global unset",
            property_name: "width",
            authored_value: "unset",
            expected_property: CssKnownProperty::Width,
            expected_global: CssGlobalKeyword::Unset,
        },
        AcceptedDeclarationCase {
            label: "color global revert",
            property_name: "color",
            authored_value: "revert",
            expected_property: CssKnownProperty::Color,
            expected_global: CssGlobalKeyword::Revert,
        },
        AcceptedDeclarationCase {
            label: "animation global revert-layer",
            property_name: "animation",
            authored_value: "revert-layer",
            expected_property: CssKnownProperty::Animation,
            expected_global: CssGlobalKeyword::RevertLayer,
        },
        AcceptedDeclarationCase {
            label: "background-color global inherit preserves authored property",
            property_name: "background-color",
            authored_value: "inherit",
            expected_property: CssKnownProperty::BackgroundColor,
            expected_global: CssGlobalKeyword::Inherit,
        },
        AcceptedDeclarationCase {
            label: "mask global unset",
            property_name: "mask",
            authored_value: "unset",
            expected_property: CssKnownProperty::Mask,
            expected_global: CssGlobalKeyword::Unset,
        },
    ];

    assert_accepts_declarations(&cases);
}

#[test]
fn typo_property_has_unknown_property_error_kind() {
    let error = CssParser::default()
        .parse_sheet(".panel { widht: 10px; }")
        .expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::UnknownProperty,
        "unexpected error kind: {error:?}"
    );
    assert_eq!(detail.property_name().unwrap().as_str(), "widht");
}

#[test]
fn parses_global_keywords_for_different_value_domains() {
    assert_eq!(
        declaration_global!(".panel { width: inherit; }", Width),
        CssGlobalKeyword::Inherit
    );
    assert_eq!(
        declaration_global!(".panel { display: initial; }", Display),
        CssGlobalKeyword::Initial
    );
    assert_eq!(
        declaration_global!(".panel { color: unset; }", Color),
        CssGlobalKeyword::Unset
    );
}

#[test]
fn parses_newer_global_keywords_as_authored_syntax() {
    assert_eq!(
        declaration_global!(".panel { padding: revert; }", Padding),
        CssGlobalKeyword::Revert
    );
    assert_eq!(
        declaration_global!(".panel { margin: revert-layer; }", Margin),
        CssGlobalKeyword::RevertLayer
    );
}

#[test]
fn parses_all_property_global_keywords_as_authored_syntax() {
    let cases = [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ];

    for (authored, expected) in cases {
        assert_eq!(
            declaration_global!(&format!(".panel {{ all: {authored}; }}"), All),
            expected
        );
    }
}

#[test]
fn rejects_non_global_all_values_with_typed_unsupported_value() {
    for input in [".panel { all: block; }", ".panel { all: 1px; }"] {
        let error = CssParser::default().parse_sheet(input).expect_err(input);

        let detail = &error;
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidPropertyValue,
            "unexpected error kind: {error:?}"
        );
        assert_eq!(detail.known_property().unwrap(), CssKnownProperty::All);
    }
}

#[test]
fn global_keyword_must_be_the_whole_value() {
    let error = CssParser::default()
        .parse_sheet(".panel { width: inherit 10px; }")
        .expect_err("rejected parse report");

    assert!(
        (error.code() == CssErrorCode::InvalidPropertyValue
            || error.code() == CssErrorCode::UnexpectedToken)
    );
}
