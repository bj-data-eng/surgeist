use crate::html::support::html_element as element;
use surgeist_html::{
    Attribute, HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlHyperlinkAttributeContext, HtmlReferrerPolicy, HtmlValueInterpreter, KnownHtmlAttribute,
};

#[test]
fn missing_href_does_not_erase_the_missing_referrerpolicy_intrinsic_default() {
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy_for_hyperlink(
                &element(local),
                None::<&surgeist_html::Attribute>,
                HtmlHyperlinkAttributeContext::WithoutHref,
            )
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::Missing);
        assert_eq!(report.recognition(), Recognition::Missing);
        assert_eq!(report.author(), Author::Missing);
        assert_eq!(report.raw_attribute(), None);
    }
}

#[test]
fn missing_href_does_not_erase_a_recognized_referrerpolicy_keyword() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Referrerpolicy.qualified_name(),
        "OrIgIn",
    );
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy_for_hyperlink(
                &element(local),
                Some(&raw),
                HtmlHyperlinkAttributeContext::WithoutHref,
            )
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Origin));
        assert_eq!(
            report.recognition(),
            Recognition::Keyword {
                matched: "origin",
                canonical: "origin",
            }
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn missing_href_does_not_erase_conforming_empty_referrerpolicy() {
    let raw = Attribute::new(KnownHtmlAttribute::Referrerpolicy.qualified_name(), "");
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy_for_hyperlink(
                &element(local),
                Some(&raw),
                HtmlHyperlinkAttributeContext::WithoutHref,
            )
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::PresentEmpty);
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn missing_href_does_not_erase_invalid_literal_referrerpolicy_recovery() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Referrerpolicy.qualified_name(),
        " no-referrer ",
    );
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy_for_hyperlink(
                &element(local),
                Some(&raw),
                HtmlHyperlinkAttributeContext::WithoutHref,
            )
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.recognition(), Recognition::Unrecognized);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}
