//! Source-defined temporal content grammar through input_numeric_constraints.
//! HTML 877d6146487f: Date/Month/Week/Time/LocalDateTime min/max/value grammar.

use surgeist_html::{
    Attribute, HtmlAttributeApplicability, HtmlAttributeObservation, HtmlInputNumericAuthorState,
    HtmlInputType, HtmlValueInterpreter, KnownHtmlAttribute as A, KnownHtmlElement as E,
};

fn temporal_content_authorship(state: HtmlInputType, spelling: &str) {
    let minimum = Attribute::new(A::Min.qualified_name(), spelling);
    let maximum = Attribute::new(A::Max.qualified_name(), spelling);
    let value = Attribute::new(A::Value.qualified_name(), spelling);
    let report = HtmlValueInterpreter::new(64)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&minimum),
            Some(&maximum),
            None,
            Some(&value),
            state,
        )
        .unwrap();
    for (role, observed, raw) in [
        (A::Min, report.minimum(), &minimum),
        (A::Max, report.maximum(), &maximum),
        (A::Value, report.value(), &value),
    ] {
        assert_eq!(observed.attribute(), role);
        assert_eq!(observed.input_state(), state);
        assert_eq!(observed.raw_attribute(), Some(raw));
        assert_eq!(
            observed.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(
            observed.observation(),
            HtmlAttributeObservation::PresentNonempty
        );
        assert_eq!(
            observed.author(),
            HtmlInputNumericAuthorState::Conforming,
            "{state:?}/{role:?}/{spelling} must execute its source author grammar"
        );
    }
}

#[test]
fn date_content_author_grammar_is_applied() {
    // #date-state-(type=date):attr-input-min, valid Gregorian date.
    temporal_content_authorship(HtmlInputType::Date, "1970-01-01");
}

#[test]
fn month_content_author_grammar_is_applied() {
    // #month-state-(type=month):attr-input-min, valid month.
    temporal_content_authorship(HtmlInputType::Month, "1970-01");
}

#[test]
fn week_content_author_grammar_is_applied() {
    // #week-state-(type=week):attr-input-min, valid ISO week.
    temporal_content_authorship(HtmlInputType::Week, "1970-W01");
}

#[test]
fn time_content_author_grammar_is_applied() {
    // #time-state-(type=time):attr-input-min, valid clock time.
    temporal_content_authorship(HtmlInputType::Time, "00:00");
}

#[test]
fn local_date_time_content_author_grammar_is_applied() {
    // #local-date-and-time-state-(type=datetime-local):attr-input-min.
    temporal_content_authorship(HtmlInputType::LocalDateTime, "1970-01-01T00:00");
}
