use surgeist_html::security::opener_policy::{HtmlOpenerPolicy, HtmlOpenerPolicyValue};

#[test]
fn initial_record_has_four_source_defaults() {
    // HTML877d raw64823–64827: the two independent policy values begin
    // unsafe-none and both reporting endpoints begin null.
    let policy = HtmlOpenerPolicy::initial();
    assert_eq!(policy.value(), HtmlOpenerPolicyValue::UnsafeNone);
    assert_eq!(policy.reporting_endpoint(), None);
    assert_eq!(
        policy.report_only_value(),
        HtmlOpenerPolicyValue::UnsafeNone
    );
    assert_eq!(policy.report_only_reporting_endpoint(), None);
}

#[test]
fn opener_policy_language_values_keep_all_five_exact_source_spellings() {
    for (value, literal) in [
        (HtmlOpenerPolicyValue::UnsafeNone, "unsafe-none"),
        (
            HtmlOpenerPolicyValue::SameOriginAllowPopups,
            "same-origin-allow-popups",
        ),
        (HtmlOpenerPolicyValue::SameOrigin, "same-origin"),
        (
            HtmlOpenerPolicyValue::SameOriginPlusCoep,
            "same-origin-plus-COEP",
        ),
        (
            HtmlOpenerPolicyValue::NoopenerAllowPopups,
            "noopener-allow-popups",
        ),
    ] {
        assert_eq!(value.as_str(), literal);
    }
}
