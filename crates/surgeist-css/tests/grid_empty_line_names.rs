#![forbid(unsafe_code)]

//! Grid2's <line-names> uses zero or more custom identifiers.
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-line-names
//! Empty groups are authored components, not absent components or empty tracks.

use surgeist_css::{
    CssCustomIdent, CssGridAutoTrackComponent, CssGridGeneralTrackComponent, CssGridLineNames,
    CssGridTrackRepeatComponent, CssImportance, CssKnownPropertyValueRef, parse_style_attribute,
};

#[test]
fn construction_preserves_an_empty_line_name_group() {
    let empty = CssGridLineNames::try_new(Vec::new()).expect("[] is valid line-names");
    assert!(empty.names().is_empty());
    let names = CssGridLineNames::try_new(vec![
        CssCustomIdent::try_new("start").unwrap(),
        CssCustomIdent::try_new("start").unwrap(),
    ])
    .unwrap();
    assert_eq!(names.names().len(), 2);
}

#[test]
fn explicit_tracks_retain_both_empty_groups_and_importance() {
    let source = "grid-template-columns: [] 2px [/* end */] !important; color: red";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration, _sibling] = report.syntax().as_slice() else {
        panic!("both declarations retained");
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssKnownPropertyValueRef::GridTemplateColumns(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed columns");
    };
    assert_eq!(value.as_css(), "[] 2px [/* end */]");
    let [
        CssGridGeneralTrackComponent::LineNames(first),
        CssGridGeneralTrackComponent::TrackSize(_),
        CssGridGeneralTrackComponent::LineNames(last),
    ] = value.value().general_list().unwrap().components()
    else {
        panic!("empty groups retain their ordered authored positions");
    };
    assert!(first.names().is_empty());
    assert!(last.names().is_empty());
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn repetition_preserves_empty_groups_in_authored_graph() {
    for count in ["2", "auto-fill", "auto-fit"] {
        let source = format!("grid-template-columns: repeat({count}, [] 2px [])");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::GridTemplateColumns(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed columns");
        };
        let content = if count == "2" {
            let [CssGridGeneralTrackComponent::Repeat(repeat)] =
                value.value().general_list().unwrap().components()
            else {
                panic!("integer repeat")
            };
            repeat.content()
        } else {
            let [CssGridAutoTrackComponent::AutoRepeat(repeat)] =
                value.value().auto_list().unwrap().components()
            else {
                panic!("automatic repeat")
            };
            repeat.content()
        };
        let [
            CssGridTrackRepeatComponent::LineNames(first),
            CssGridTrackRepeatComponent::TrackSize(_),
            CssGridTrackRepeatComponent::LineNames(last),
        ] = content.components()
        else {
            panic!("three ordered components")
        };
        assert!(first.names().is_empty());
        assert!(last.names().is_empty());
    }
}

#[test]
fn empty_groups_do_not_replace_required_tracks_or_admit_reserved_names() {
    for value in ["[]", "repeat(2, [])", "[auto] 2px", "[span] 2px"] {
        let source = format!("grid-template-columns: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{source}");
        assert_eq!(report.syntax().len(), 1, "only the color sibling remains");
        assert!(report.into_validation_result().is_err());
    }
}
