#![forbid(unsafe_code)]
//! The selected Transitions 1 §§2.1–2.5 define the property/list production
//! independently of contextual matching and execution. The support catalog
//! describes that authored production; Easing owns easing-function syntax.
use surgeist_css::*;

fn grammar(property: CssKnownProperty, accepted: &[&str], rejected: &[&str]) {
    for value in accepted {
        let css = format!("{}:{value}", property.canonical_name());
        let report = parse_style_attribute(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("one retained occurrence")
        };
        assert_eq!(declaration.known().unwrap().property(), property);
        assert!(declaration.known().unwrap().property_value().is_some());
        assert!(validate_style_attribute(&css).is_ok());
        parse_property_value_for_grammar(
            property.grammar(),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    }
    for value in rejected {
        let css = format!("{}:{value};color:red", property.canonical_name());
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean(), "{css}");
        assert_eq!(report.syntax().len(), 1, "{css}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert!(validate_style_attribute(&css).is_err());
        assert!(
            parse_property_value_for_grammar(
                property.grammar(),
                parse_component_values(value).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

fn complete(property: CssKnownProperty) {
    let support = property_support_metadata(property.canonical_name()).unwrap();
    let feature = support.feature();
    assert_eq!(feature.kind(), CssFeatureKind::Property);
    assert_eq!(feature.source().id().as_str(), "I-TRANSITIONS1");
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.supported_subset(), None);
    assert_eq!(feature.unsupported_remainder(), None);
    assert_eq!(feature.recognized_unsupported_code(), None);
}

#[test]
fn property_list_catalog_covers_the_selected_identifier_and_none_production() {
    grammar(
        CssKnownProperty::TransitionProperty,
        &[
            "none",
            "all, opacity, opacity, auto, span, MiXeD",
            r"all, \31 Unknown",
            "ease, infinite, normal, running",
        ],
        &[
            "",
            "none, opacity",
            "opacity, none",
            "default",
            "revert-layer, opacity",
            "opacity,",
            "opacity ease",
        ],
    );
    complete(CssKnownProperty::TransitionProperty);
}

#[test]
fn shorthand_catalog_covers_selected_slots_and_whole_list_none_rejection() {
    grammar(
        CssKnownProperty::Transition,
        &[
            "none 1s ease -250ms",
            "ease-out linear",
            "opacity calc(1s + 250ms) cubic-bezier(.2,0,.8,1) -250ms",
            "opacity 1s, transform ease-in",
            "auto, span",
            "1s -1s",
            "all",
        ],
        &[
            "",
            "none, opacity",
            "opacity, none 1s",
            "opacity -1s",
            "opacity 1s 2s 3s",
            "opacity,",
            "opacity 1s ease linear extra",
        ],
    );
    complete(CssKnownProperty::Transition);
}

#[test]
fn existing_time_and_easing_catalog_controls_cover_their_selected_lists() {
    grammar(
        CssKnownProperty::TransitionDuration,
        &["0s", "1s, calc(1s + 250ms)"],
        &["", "-1s", "1s,", "1px"],
    );
    grammar(
        CssKnownProperty::TransitionDelay,
        &["-250ms", "0s, calc(-1s + 250ms)"],
        &["", "1s,", "1px"],
    );
    grammar(
        CssKnownProperty::TransitionTimingFunction,
        &["ease", "linear, cubic-bezier(.2,0,.8,1), steps(2,end)"],
        &["", "ease,", "steps(0,end)"],
    );
    for property in [
        CssKnownProperty::TransitionDuration,
        CssKnownProperty::TransitionDelay,
        CssKnownProperty::TransitionTimingFunction,
    ] {
        complete(property);
    }
}
