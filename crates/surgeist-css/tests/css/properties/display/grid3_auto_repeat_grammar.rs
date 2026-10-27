//! Grid3 replaces only the auto-repeat body with nonrecursive <track-size>.
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat
//! Grid3 source SHA256:
//! ab3a5d476f748764136b3ad89f2b4c9c9b47d6d50304aaa5d0a02319b9f5f46b
//! The surrounding fixed auto-track-list, one auto-repeat per list, and implicit
//! <track-size>+ restrictions remain from the selected Grid2 publication:
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-auto-track-list
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#repeat-syntax
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#auto-tracks
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#grid-shorthand
//! Grid2 source SHA256:
//! 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2
//! These runtime tests use only preexisting public APIs. Source preservation,
//! checked construction, recovery, and validator parity are Surgeist contracts.

use crate::css::support::property_admission::Responsible::{self, At};
use crate::css::support::property_admission::{assert_checked_rejected, assert_invalid_value};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction, CssTokenKind};
use surgeist_css::property::CssKnownProperty as Property;
use surgeist_css::property::{CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTarget};

const TEMPLATE_AXES: [Property; 2] = [Property::GridTemplateRows, Property::GridTemplateColumns];

#[test]
fn automatic_repetition_accepts_general_track_sizes_in_either_axis_and_mode() {
    for property in TEMPLATE_AXES {
        for mode in ["auto-fill", "auto-fit"] {
            for body in [
                "auto",
                "min-content",
                "max-content",
                "0fr",
                "1fr",
                "fit-content(20%)",
                "minmax(auto, 1fr)",
                "minmax(min-content, max-content)",
            ] {
                assert_accepted(property, &format!("repeat({mode}, {body})"));
            }
        }
    }
}

#[test]
fn intrinsic_repeat_bodies_preserve_named_tracks_and_fixed_surroundings() {
    for property in TEMPLATE_AXES {
        for authored in [
            "repeat(auto-fill, [start] 50px [middle] auto auto [end])",
            "10px repeat(auto-fit, auto) minmax(auto, 20px) repeat(2, minmax(10px, 1fr))",
            "[outer] repeat(2, 5%) [before] repeat(auto-fill, minmax(auto, 1fr) fit-content(20px)) [after] 0",
            "minmax(10px, 1fr) repeat(auto-fit, max-content) minmax(min-content, 20%)",
        ] {
            assert_accepted(property, authored);
        }
    }
}

#[test]
fn template_axes_and_the_implemented_grid_auto_flow_branch_share_the_relaxed_body() {
    assert_accepted(
        Property::GridTemplate,
        "repeat(auto-fill, auto) / repeat(auto-fit, 1fr)",
    );
    assert_accepted(
        Property::GridTemplate,
        "10px / repeat(auto-fit, fit-content(20%))",
    );
    assert_accepted(
        Property::Grid,
        "repeat(auto-fill, max-content) / repeat(auto-fit, minmax(auto, 1fr))",
    );
    assert_accepted(
        Property::Grid,
        "auto-flow dense 12px / repeat(auto-fit, 1fr)",
    );
}

#[test]
fn fixed_surroundings_and_one_automatic_repeat_remain_required() {
    for property in TEMPLATE_AXES {
        for (authored, token) in [
            ("1fr repeat(auto-fit, 10px)", At("auto-fit")),
            ("repeat(auto-fit, 10px) 1fr", At("1fr")),
            ("auto repeat(auto-fill, 10px)", At("auto-fill")),
            ("repeat(auto-fill, 10px) min-content", At("min-content")),
            (
                "repeat(auto-fill, 10px) fit-content(20px)",
                At("fit-content("),
            ),
            ("minmax(auto, 1fr) repeat(auto-fit, 10px)", At("auto-fit")),
            ("repeat(2, 1fr) repeat(auto-fit, 10px)", At("auto-fit")),
            ("repeat(auto-fit, 10px) repeat(2, auto)", At("auto")),
            (
                "repeat(auto-fill, 10px) repeat(auto-fit, 20px)",
                At("auto-fit"),
            ),
            ("1fr repeat(auto-fit, auto)", At("auto-fit")),
            ("repeat(auto-fit, auto) auto", At("auto")),
            (
                "repeat(auto-fit, auto) fit-content(20px)",
                At("fit-content("),
            ),
            ("repeat(auto-fit, auto) repeat(2, 1fr)", At("1fr")),
            (
                "repeat(auto-fill, auto) repeat(auto-fit, 1fr)",
                At("auto-fit"),
            ),
        ] {
            assert_rejected(property, authored, token);
        }
    }
}

#[test]
fn repeat_content_remains_nonrecursive_nonempty_and_subject_to_track_size_grammar() {
    for property in TEMPLATE_AXES {
        for (authored, token) in [
            ("repeat(auto-fit, repeat(2, 10px))", Some("repeat(")),
            ("repeat(auto-fill, repeat(auto-fit, 10px))", Some("repeat(")),
            ("repeat(2, repeat(auto-fill, 10px))", Some("repeat(")),
            ("repeat(auto-fit,)", Some(")")),
            ("repeat(auto-fill, [only])", Some(")")),
            ("repeat(auto-fit, -1px)", Some("-1px")),
            ("repeat(auto-fill, -1%)", Some("-1%")),
            ("repeat(auto-fit, -1fr)", Some("-1fr")),
            ("repeat(auto-fit, fit-content(-1px))", Some("-1px")),
            ("repeat(auto-fill, minmax(1fr, 10px))", Some("1fr")),
            ("repeat(auto-fit, minmax(auto, -1fr))", Some("-1fr")),
            ("repeat(auto-fit, 1)", Some("1")),
            ("repeat(auto-fill, none)", Some("none")),
        ] {
            assert_rejected(property, authored, token);
        }
    }
}

#[test]
fn implicit_track_lists_accept_sizes_without_repetition_or_line_names() {
    for property in [Property::GridAutoRows, Property::GridAutoColumns] {
        assert_accepted(property, "auto 1fr fit-content(20%) minmax(auto, 1fr)");
        for (authored, token) in [
            ("repeat(2, 10px)", Some("repeat(")),
            ("repeat(auto-fill, 10px)", Some("repeat(")),
            ("repeat(auto-fit, auto)", Some("repeat(")),
            ("[start] 10px", Some("[")),
        ] {
            assert_rejected(property, authored, token);
        }
    }
    assert_accepted(Property::Grid, "auto-flow 1fr / 10px");
    assert_rejected(
        Property::Grid,
        "auto-flow repeat(auto-fit, 10px) / 10px",
        Some("repeat("),
    );
    assert_rejected(Property::Grid, "auto-flow [start] 10px / 10px", Some("["));
}

#[test]
fn escaped_property_names_and_authored_keyword_case_preserve_source_and_importance() {
    assert_accepted_with_spelling(
        Property::GridTemplateColumns,
        r"GRID-TEMPLATE-C\4f LUMNS",
        "RePeAt(AUTO-FIT, [Start] MINMAX(AUTO, 1fr) [End])",
    );
}

#[test]
fn checked_property_construction_accepts_general_automatic_repeat_bodies() {
    for (property, authored) in [
        (Property::GridTemplateColumns, "repeat(auto-fill, auto)"),
        (Property::GridTemplateRows, "repeat(auto-fit, 1fr)"),
        (
            Property::GridTemplate,
            "repeat(auto-fit, max-content) / 10px",
        ),
        (
            Property::Grid,
            "auto-flow 12px / repeat(auto-fill, fit-content(20%))",
        ),
    ] {
        assert_checked(property, authored);
    }
}

#[test]
fn a_flexible_minimum_inside_auto_repeat_reports_its_source_token() {
    for property in TEMPLATE_AXES {
        let source = format!(
            "{}:repeat(auto-fit, minmax(1fr, 10px)); color:red",
            property.canonical_name()
        );
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid minimum: {source}: {report:?}");
        };
        let responsible = source.find("1fr").unwrap();
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible
        );
        assert_eq!(diagnostic.error().position().line().value(), 0);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible
        );
        let detail = diagnostic.error();
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidPropertyValue,
            "a Grid grammar error"
        );
        let token = detail.encountered().expect("responsible fraction");
        assert_eq!(token.kind(), CssTokenKind::Dimension);
        assert_eq!(token.authored(), "1fr");
        assert_rejected(property, "repeat(auto-fit, minmax(1fr, 10px))", Some("1fr"));
    }
}

fn assert_accepted(property: Property, authored: &str) {
    assert_accepted_with_spelling(property, property.canonical_name(), authored);
}

fn assert_accepted_with_spelling(property: Property, spelling: &str, authored: &str) {
    let source = format!("color:red;{spelling}:  {authored} !IMPORTANT;width:3px");
    let report = CssParser::default().parse_block_contents(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        properties(report.syntax().as_slice()),
        [Property::Color, property, Property::Width]
    );
    let declaration = &report.syntax()[1];
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(authored_value(declaration), authored);
    let name = declaration
        .parsed_name()
        .expect("original property-name provenance");
    assert_eq!(name.source().as_str(), source);
    assert_eq!(
        name.span().start().byte_offset().value(),
        "color:red;".len()
    );
    assert_eq!(
        name.span().end().byte_offset().value(),
        "color:red;".len() + spelling.len()
    );
    assert!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .is_ok(),
        "{source}"
    );
    assert_checked(property, authored);
}

fn assert_checked(property: Property, authored: &str) {
    let components = CssComponentValues::parse(&CssParser::default(), authored)
        .expect("balanced authored components");
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked construction accepts {authored}: {error:?}"));
    assert_eq!(checked.known().unwrap().property(), property);
    assert_eq!(authored_value(&checked), authored);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert!(checked.position().is_none());
    assert!(checked.parsed_name().is_none());
    assert!(checked.parsed_value().is_none());
    let original_tokens = components.serialize_tokens(Default::default()).unwrap();
    let retained_tokens = checked
        .value_components()
        .serialize_tokens(Default::default())
        .unwrap();
    assert_eq!(retained_tokens.as_css(), original_tokens.as_css());
    assert_eq!(retained_tokens.segments(), original_tokens.segments());
    assert_eq!(
        retained_tokens.origin_at(retained_tokens.as_css().len()),
        original_tokens.origin_at(original_tokens.as_css().len())
    );
    let surgeist_css::component::CssValueOrigin::Parsed(original) = components.items()[0].origin()
    else {
        panic!("the supplied first token has parsed provenance");
    };
    let surgeist_css::component::CssValueOrigin::Parsed(retained) =
        checked.value_components().items()[0].origin()
    else {
        panic!("checked construction retains the first token's provenance");
    };
    assert!(retained.source().same_snapshot(original.source()));
}

fn assert_rejected<'a>(property: Property, authored: &str, token: impl Into<Responsible<'a>>) {
    let token = token.into();
    let dropped = format!("{}:{authored};", property.canonical_name());
    let source = format!("color:red;{dropped}width:3px");
    let report = CssParser::default().parse_block_contents(&source);
    assert_eq!(
        properties(report.syntax().as_slice()),
        [Property::Color, Property::Width],
        "{source}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected Grid declaration: {source}: {report:?}");
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
        report.diagnostics()
    );
    assert_checked_rejected(
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            CssComponentValues::parse(&CssParser::default(), authored).unwrap(),
            CssImportance::Normal,
        ),
        property,
        token,
    );
}

fn properties(declarations: &[CssDeclaration]) -> Vec<Property> {
    declarations
        .iter()
        .map(|value| value.known().unwrap().property())
        .collect()
}

fn authored_value(declaration: &CssDeclaration) -> &str {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplateRows(value) => value.as_css(),
        CssKnownPropertyValueRef::GridTemplateColumns(value) => value.as_css(),
        CssKnownPropertyValueRef::GridAutoRows(value) => value.as_css(),
        CssKnownPropertyValueRef::GridAutoColumns(value) => value.as_css(),
        CssKnownPropertyValueRef::GridTemplate(value) => value.as_css(),
        CssKnownPropertyValueRef::Grid(value) => value.as_css(),
        _ => panic!("one of the six Grid repetition consumers"),
    }
}
