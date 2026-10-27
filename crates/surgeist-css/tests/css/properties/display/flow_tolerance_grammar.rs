//! Grid3 §4.2 specifies `normal | <length-percentage> | infinite`, with no
//! nonnegative range restriction. The January 2026 publication also records
//! the rename from `item-tolerance`; obsolete spellings are not aliases in
//! Surgeist's selected published grammar.
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#placement-tolerance
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#recent-changes
//! Recovery, source preservation, and validator parity are Surgeist contracts.
//! These tests use the existing public lookup and parser APIs, so the missing
//! canonical property is an executed assertion failure, not a compiler error.

use crate::css::support::property_admission::Responsible::At;
use crate::css::support::property_admission::{assert_checked_rejected, assert_invalid_value};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};

fn flow_tolerance() -> CssKnownProperty {
    CssKnownProperty::from_name("flow-tolerance")
        .expect("the published Grid3 canonical flow-tolerance property is recognized")
}

#[test]
fn canonical_identity_accepts_ascii_case_and_drops_both_obsolete_spellings() {
    let property = flow_tolerance();
    assert_eq!(property.canonical_name(), "flow-tolerance");
    assert_eq!(
        CssKnownProperty::from_name("FLOW-TOLERANCE"),
        Some(property)
    );
    assert_eq!(
        CssKnownProperty::from_name("Flow-Tolerance"),
        Some(property)
    );
    for obsolete in ["grid-flow-tolerance", "item-tolerance"] {
        for spelling in [obsolete.to_owned(), obsolete.to_ascii_uppercase()] {
            assert_eq!(CssKnownProperty::from_name(&spelling), None, "{spelling}");
        }
    }
}

#[test]
fn obsolete_properties_are_unknown_and_recovery_retains_surrounding_declarations() {
    for obsolete in [
        "grid-flow-tolerance",
        "item-tolerance",
        "GRID-FLOW-TOLERANCE",
        r"item-\74 olerance",
        r"grid-\66 low-tolerance",
    ] {
        let dropped = format!("{obsolete}:normal;");
        let source = format!("color:red;{dropped}width:3px");
        let report = CssParser::default().parse_block_contents(&source);
        let properties: Vec<_> = report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property())
            .collect();
        assert_eq!(
            properties,
            [CssKnownProperty::Color, CssKnownProperty::Width]
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one dropped obsolete property: {source}: {report:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = "color:red;".len();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + dropped.len()
        );
        assert_eq!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics(),
        );
    }
}

#[test]
fn signed_lengths_percentages_and_dimensionally_valid_calculations_are_accepted() {
    for value in [
        "normal",
        "infinite",
        "0",
        "-0",
        "+0",
        "1px",
        "-1px",
        "+2px",
        "-0.5em",
        "2rem",
        "-3vw",
        "-4cqi",
        "0%",
        "-25%",
        "+125%",
        "calc(-1px)",
        "calc(-2px - 3%)",
        "calc(1px - 2px)",
        "calc(-2 * 3px)",
        "calc((1em + 10%) * -2)",
        "calc(3px / -2)",
    ] {
        let source = format!("flow-tolerance:{value}");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("one retained flow-tolerance declaration: {source}");
        };
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), flow_tolerance());
        assert!(known.property_value().is_some(), "ordinary value: {value}");
        assert!(known.global().is_none());
        assert!(known.substitution_dependent().is_none());
        assert_eq!(declaration.importance(), CssImportance::Normal);
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .expect("clean flow-tolerance validates");

        let sheet = format!(".grid {{{source}}}");
        assert!(
            CssParser::default().parse_sheet(&sheet).is_clean(),
            "{sheet}"
        );
        CssParser::default()
            .parse_sheet(&sheet)
            .into_validation_result()
            .expect("the same grammar applies in style rules");
    }
}

#[test]
fn property_and_keyword_escapes_keep_canonical_identity_and_authored_spelling() {
    for (name, value) in [
        ("FLOW-TOLERANCE", "NoRmAl"),
        (r"\66 low-tolerance", r"\6e ormal"),
        (r"flow-\74 olerance", r"\69 nfinite"),
    ] {
        let source = format!("{name}:  {value} !ImPoRtAnT;");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        assert_eq!(declaration.known().unwrap().property(), flow_tolerance());
        assert_eq!(declaration.importance(), CssImportance::Important);
        let parsed_name = declaration.parsed_name().unwrap();
        let span = parsed_name.span();
        assert_eq!(
            &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            name,
        );
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            declaration
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            format!("  {value} ")
        );
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .unwrap();
    }
}

#[test]
fn property_components_preserve_authored_boundary_whitespace() {
    // CssComponentValues::serialize preserves actual whitespace. Exercise an
    // existing property independently of the new flow-tolerance grammar.
    let source = "MARGIN:  AuTo !ImPoRtAnT;";
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one valid existing margin declaration");
    };
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Margin
    );
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "  AuTo "
    );
}

#[test]
fn invalid_keywords_dimensions_and_combinations_drop_only_the_offending_declaration() {
    let property = flow_tolerance();
    for (invalid, token) in [
        ("auto", At("auto")),
        ("min-content", At("min-content")),
        ("max-content", At("max-content")),
        ("fit-content", At("fit-content")),
        ("thin", At("thin")),
        ("medium", At("medium")),
        ("thick", At("thick")),
        ("none", At("none")),
        ("infinity", At("infinity")),
        ("-infinite", At("-infinite")),
        ("1", At("1")),
        ("-1", At("-1")),
        ("1fr", At("1fr")),
        ("1deg", At("1deg")),
        ("1s", At("1s")),
        ("normal infinite", At("infinite")),
        ("normal 1px", At("1px")),
        ("1px 2px", At("2px")),
        ("1px, 2px", At(",")),
        ("1px / 2px", At("/")),
        ("calc()", At(")")),
        ("calc(1px +)", At(")")),
        ("calc(1px + 1s)", At("1s")),
        ("calc(1px * 1px)", At("calc(")),
        ("calc(1 + 2)", At("calc(")),
    ] {
        let dropped = format!("flow-tolerance:{invalid};");
        let source = format!("color:red;{dropped}width:3px");
        let report = CssParser::default().parse_block_contents(&source);
        let properties: Vec<_> = report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property())
            .collect();
        assert_eq!(
            properties,
            [CssKnownProperty::Color, CssKnownProperty::Width],
            "{source}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one dropped invalid value: {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}"
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_invalid_value(diagnostic.error(), property, token);
        let start = "color:red;".len();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + dropped.len()
        );
        assert_eq!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics(),
        );
        assert_checked_rejected(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                CssComponentValues::parse(&CssParser::default(), invalid).unwrap(),
                CssImportance::Normal,
            ),
            property,
            token,
        );
    }
}

#[test]
fn css_wide_keywords_and_substitution_are_symbolic_whole_value_branches() {
    for (value, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = format!("flow-tolerance:{value}!important");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), flow_tolerance());
        assert_eq!(known.global(), Some(keyword));
        assert!(known.property_value().is_none());
        assert!(known.substitution_dependent().is_none());
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
    for value in [
        "var(--threshold)",
        "var(--threshold, -25%)",
        "calc(var(--threshold) - 1px)",
    ] {
        let source = format!("flow-tolerance:{value}!important");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), flow_tolerance());
        assert_eq!(known.substitution_dependent().unwrap().as_css(), value);
        assert!(known.property_value().is_none());
        assert!(known.global().is_none());
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
}
