//! The selected Transitions 1 §§2.1–2.5 define the property/list production
//! independently of contextual matching and execution. The support catalog
//! describes that authored production; Easing owns easing-function syntax.
use crate::css::support::property_admission::Responsible::{self, At, End};
use crate::css::support::property_admission::{
    assert_checked_rejected, assert_style_attribute_rejected,
};
use surgeist_css::*;

fn grammar<'a, T: Copy + Into<Responsible<'a>>>(
    property: CssKnownProperty,
    accepted: &[&str],
    rejected: &[(&str, T)],
) {
    for value in accepted {
        let css = format!("{}:{value}", property.canonical_name());
        let report = CssParser::default().parse_block_contents(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("one retained occurrence")
        };
        assert_eq!(declaration.known().unwrap().property(), property);
        assert!(declaration.known().unwrap().property_value().is_some());
        assert!(
            CssParser::default()
                .parse_block_contents(&css)
                .into_validation_result()
                .is_ok()
        );
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(property.grammar()),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    }
    for &(value, token) in rejected {
        let css = format!("{}:{value};color:red", property.canonical_name());
        let report = assert_style_attribute_rejected(&css, property, token);
        assert_eq!(report.syntax().len(), 1, "{css}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_checked_rejected(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(property.grammar()),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal,
            ),
            property,
            token,
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
            ("", End),
            ("none, opacity", At(",")),
            ("opacity, none", At("none")),
            ("default", At("default")),
            ("revert-layer, opacity", At(",")),
            ("opacity,", At(",")),
            ("opacity ease", At("ease")),
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
            ("", End),
            ("none, opacity", At(",")),
            ("opacity, none 1s", At("none")),
            ("opacity -1s", At("-1s")),
            ("opacity 1s 2s 3s", At("3s")),
            ("opacity,", At(",")),
            ("opacity 1s ease linear extra", At("linear")),
        ],
    );
    complete(CssKnownProperty::Transition);
}

#[test]
fn time_and_easing_catalog_controls_cover_their_selected_lists() {
    grammar(
        CssKnownProperty::TransitionDuration,
        &["0s", "1s, calc(1s + 250ms)"],
        &[
            ("", None),
            ("-1s", Some("-1s")),
            ("1s,", Some(",")),
            ("1px", Some("1px")),
        ],
    );
    grammar(
        CssKnownProperty::TransitionDelay,
        &["-250ms", "0s, calc(-1s + 250ms)"],
        &[("", None), ("1s,", Some(",")), ("1px", Some("1px"))],
    );
    grammar(
        CssKnownProperty::TransitionTimingFunction,
        &["ease", "linear, cubic-bezier(.2,0,.8,1), steps(2,end)"],
        &[
            ("", None),
            ("ease,", Some(",")),
            ("steps(0,end)", Some("0")),
        ],
    );
    for property in [
        CssKnownProperty::TransitionDuration,
        CssKnownProperty::TransitionDelay,
        CssKnownProperty::TransitionTimingFunction,
    ] {
        complete(property);
    }
}
