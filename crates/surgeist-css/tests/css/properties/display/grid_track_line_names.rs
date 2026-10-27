//! Grid 2 track-list and repeat productions allow at most one optional
//! `<line-names>` block at each track boundary.
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-track-list
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#repeat-syntax
//! SHA256: 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2
//! Empty `[]` is still a valid single `<line-names>` block.

use crate::css::support::property_admission::assert_checked_matches_diagnostic;
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::CssKnownProperty as Property;
use surgeist_css::property::{CssPropertyNameRef, CssPropertyTarget};

#[test]
fn adjacent_line_name_blocks_do_not_form_general_track_lists() {
    for value in ["[a] [b] 10px", "10px [a] [b]", "10px [a] [b] 20px"] {
        rejected(value);
    }
}

#[test]
fn adjacent_line_name_blocks_do_not_form_fixed_integer_repeat_content() {
    for value in [
        "repeat(2, [a] [b] 10px)",
        "repeat(2, 10px [a] [b])",
        "repeat(2, 10px [a] [b] 20px)",
    ] {
        rejected(value);
    }
}

#[test]
fn adjacent_line_name_blocks_do_not_form_automatic_repeat_bodies_or_surroundings() {
    for value in [
        "repeat(auto-fit, [a] [b] 10px)",
        "repeat(auto-fill, 10px [a] [b])",
        "10px [a] [b] repeat(auto-fit, 10px)",
        "repeat(auto-fill, 10px) [a] [b] 20px",
    ] {
        rejected(value);
    }
}

#[test]
fn one_empty_line_name_block_remains_valid_at_each_boundary() {
    for value in [
        "[] 10px []",
        "repeat(2, [] 10px [])",
        "repeat(auto-fit, [] 10px [])",
        "[] 10px [] repeat(auto-fill, [] 10px []) []",
    ] {
        let source = format!("grid-template-columns:{value};color:red");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_ok(),
            "{source}"
        );
        assert!(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(
                    Property::GridTemplateColumns
                )),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal
            )
            .is_ok(),
            "checked construction: {value}"
        );
    }
}

fn rejected(value: &str) {
    let source = format!("grid-template-columns:{value};color:red");
    let report = CssParser::default().parse_block_contents(&source);
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        Property::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected track list: {source}: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics(),
        "{source}"
    );
    assert_checked_matches_diagnostic(Property::GridTemplateColumns, value, diagnostic);
}
