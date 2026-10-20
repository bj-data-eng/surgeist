//! Source: selected Alignment3/Grid3 omit these names; CSSWG resolution
//! https://github.com/w3c/csswg-drafts/issues/8207#issuecomment-1747805578
//! explicitly removed both former Masonry properties in October 2023.
use surgeist_css::{
    CssKnownProperty, CssRecoveryAction, ErrorKind, parse_style_attribute, validate_style_attribute,
};

fn require_unknown_declarations(source: &str, names: &[&str]) {
    let report = parse_style_attribute(source);
    assert!(
        report.syntax().is_empty(),
        "obsolete declarations survived: {report:?}"
    );
    assert_eq!(report.diagnostics().len(), names.len(), "{report:?}");
    for (diagnostic, expected) in report.diagnostics().iter().zip(names) {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::UnknownProperty(detail) = diagnostic.error().kind() else {
            panic!("expected unknown property for {expected}: {diagnostic:?}");
        };
        assert_eq!(detail.name().as_str(), *expected);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(expected).unwrap()
        );
    }
}

#[test]
fn removed_track_properties_have_no_known_or_property_identity() {
    for name in [
        "align-tracks",
        "justify-tracks",
        "ALIGN-TRACKS",
        "JUSTIFY-TRACKS",
    ] {
        assert_eq!(CssKnownProperty::from_name(name), None, "{name}");
    }
}

#[test]
fn canonical_property_inventory_excludes_removed_track_properties() {
    for property in CssKnownProperty::all() {
        assert!(!matches!(
            property.canonical_name(),
            "align-tracks" | "justify-tracks"
        ));
    }
}

#[test]
fn ordinary_removed_declarations_report_unknown_property() {
    require_unknown_declarations(
        "align-tracks: center; justify-tracks: space-between",
        &["align-tracks", "justify-tracks"],
    );
}

#[test]
fn css_wide_keywords_do_not_reintroduce_removed_properties() {
    require_unknown_declarations(
        "align-tracks: initial; justify-tracks: inherit",
        &["align-tracks", "justify-tracks"],
    );
}

#[test]
fn pending_substitutions_do_not_reintroduce_removed_properties() {
    require_unknown_declarations(
        "align-tracks: var(--alignment); justify-tracks: env(alignment)",
        &["align-tracks", "justify-tracks"],
    );
}

#[test]
fn strict_validation_rejects_removed_names_through_the_same_grammar() {
    for source in ["align-tracks: center", "justify-tracks: space-evenly"] {
        let failure = validate_style_attribute(source)
            .expect_err("obsolete property must fail clean validation");
        assert!(matches!(
            failure.first().error().kind(),
            ErrorKind::UnknownProperty(_)
        ));
    }
}

#[test]
fn neighboring_alignment_and_custom_property_controls_remain_clean() {
    let source = "align-content: center; justify-content: space-between; align-items: start; justify-items: end; --align-tracks: center; --justify-tracks: var(--alignment); color: red";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 7);
    assert_eq!(validate_style_attribute(source).unwrap().len(), 7);
}

#[test]
fn existing_unknown_property_control() {
    require_unknown_declarations("not-a-property: center", &["not-a-property"]);
}
