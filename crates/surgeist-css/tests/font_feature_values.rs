use surgeist_css::{CssNamespaceContext, parse_rule, parse_sheet, validate_sheet};

// Fonts 4 (7 September 2026), section 6.9.1 defines these authored forms.
// The indices below satisfy both conflicting clauses in sections 6.9.1/6.9.2.
// Retention and report cleanliness are independent of font activation/cascade.
#[test]
fn font_feature_values_retains_empty_named_family_rules() {
    for source in [
        "@font-feature-values Demo {}",
        "@font-feature-values Demo, \"Other Font\" {}",
        "@font-feature-values \"serif\" {}",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(report.syntax().is_some(), "{source}");
        assert!(validate_sheet(source).is_ok(), "{source}");
    }
}

#[test]
fn font_feature_values_retains_all_defined_blocks_and_display_descriptor() {
    let source = "@font-feature-values Demo {\
        font-display: swap;\
        @stylistic { fancy: 1; }\
        @historical-forms { old: 1 2; }\
        @styleset { joined: 1 2; }\
        @character-variant { open: 1 2; }\
        @swash { flowing: 2; }\
        @ornaments { fleur: 3; }\
        @annotation { circled: 4; }\
    }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 1);
    assert!(validate_sheet(source).is_ok());
}

#[test]
fn font_feature_values_rejects_missing_or_generic_family_preludes() {
    for source in [
        "@font-feature-values {}",
        "@font-feature-values serif {}",
        "@font-feature-values Demo, {}",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(source).is_err(), "{source}");
    }
}
