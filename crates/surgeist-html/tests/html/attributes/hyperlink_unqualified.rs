use crate::html::support::html_element as element;
use surgeist_html::{
    Attribute, HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlReferrerPolicy, HtmlValueInterpreter, KnownHtmlAttribute,
};

#[test]
fn absent_href_evidence_does_not_erase_the_missing_policy_default() {
    // a/area require omission without href as an author constraint. Their
    // known-value-limited reflection remains unconditional, so intrinsic
    // interpretation does not need evidence of the href author requirement.
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy(&element(local), None::<&surgeist_html::Attribute>)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::Missing);
        assert_eq!(report.recognition(), Recognition::Missing);
        assert_eq!(report.author(), Author::Missing);
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.raw_attribute(), None);
    }
}

#[test]
fn absent_href_evidence_does_not_erase_a_recognized_policy_keyword() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Referrerpolicy.qualified_name(),
        "OrIgIn",
    );
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy(&element(local), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Origin));
        assert_eq!(report.observation(), Observation::PresentNonempty);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword {
                matched: "origin",
                canonical: "origin",
            }
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn absent_href_evidence_does_not_erase_conforming_empty_policy() {
    let raw = Attribute::new(KnownHtmlAttribute::Referrerpolicy.qualified_name(), "");
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy(&element(local), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::PresentEmpty);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword {
                matched: "",
                canonical: ""
            }
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn absent_href_evidence_does_not_erase_invalid_literal_policy_recovery() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Referrerpolicy.qualified_name(),
        " origin ",
    );
    for local in ["a", "area"] {
        let report = HtmlValueInterpreter::new(100)
            .referrerpolicy(&element(local), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::PresentNonempty);
        assert_eq!(report.recognition(), Recognition::Unrecognized);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}
