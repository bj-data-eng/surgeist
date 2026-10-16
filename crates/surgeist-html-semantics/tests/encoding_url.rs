//! Shared exact encoding-URL observations, with no fabricated Element context.
use std::convert::Infallible;
use surgeist_html::DocumentString;
use surgeist_html_semantics::*;

fn context() -> (HtmlBaseFactsRevision, HtmlBaseUrlRecord, HtmlBaseEncoding) {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    (revision, base, HtmlBaseEncoding::fresh())
}
fn facts(
    revision: &HtmlBaseFactsRevision,
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    observations: Vec<HtmlEncodingUrlParseObservation>,
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        observations,
        HtmlBaseFactsLimits::new(16, 4096, 10000),
    )
    .unwrap()
}

#[test]
fn borrowed_exact_units_distinguish_observed_failure_javascript_success_and_missing() {
    let (revision, base, encoding) = context();
    let javascript = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "javascript:observed".into(),
        HtmlBaseUrlScheme::Javascript,
        false,
    )
    .unwrap();
    let result = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://observed.test/".into(),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let bank = facts(
        &revision,
        &base,
        &encoding,
        vec![
            HtmlEncodingUrlParseObservation::new(
                "bad".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            ),
            HtmlEncodingUrlParseObservation::new(
                "js".into(),
                HtmlEncodingUrlParseOutcome::Parsed(javascript.clone()),
            ),
            HtmlEncodingUrlParseObservation::new(
                DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
                HtmlEncodingUrlParseOutcome::Parsed(result.clone()),
            ),
        ],
    );
    let lookup = |raw: &[u16]| {
        bank.lookup_units_with(raw, &base, &encoding, &revision, || Ok::<_, Infallible>(()))
    };
    assert_eq!(
        lookup(&[98, 97, 100]).unwrap(),
        &HtmlEncodingUrlParseOutcome::Failure
    );
    assert_eq!(
        lookup(&[106, 115]).unwrap(),
        &HtmlEncodingUrlParseOutcome::Parsed(javascript)
    );
    let original = [32, 39, 0xd800, 0, 0xdc00, 39, 32];
    let selected = lookup(&original[2..5]).unwrap();
    assert_eq!(selected, &HtmlEncodingUrlParseOutcome::Parsed(result));
    assert!(std::ptr::eq(selected, lookup(&original[2..5]).unwrap()));
    assert_eq!(
        lookup(&original).unwrap_err(),
        HtmlEncodingUrlLookupError::Facts(HtmlEncodingUrlFactsError::MissingObservation)
    );
}

#[test]
fn opaque_context_and_complete_revision_are_checked_independently_from_text() {
    let (revision, base, encoding) = context();
    let bank = facts(
        &revision,
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
    );
    let equal_text_base = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    for (base, encoding, expected, error) in [
        (
            equal_text_base,
            encoding.clone(),
            revision.clone(),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            base.clone(),
            HtmlBaseEncoding::fresh(),
            revision.clone(),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            base.clone(),
            encoding.clone(),
            HtmlBaseFactsRevision::new(revision.producer().clone(), 1, 99, 3),
            HtmlEncodingUrlFactsError::RevisionMismatch,
        ),
        (
            base.clone(),
            encoding.clone(),
            HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
            HtmlEncodingUrlFactsError::ForeignProducer,
        ),
    ] {
        assert_eq!(
            bank.lookup_units_with(&[120], &base, &encoding, &expected, || Ok::<_, Infallible>(
                ()
            ))
            .unwrap_err(),
            HtmlEncodingUrlLookupError::Facts(error)
        );
    }
}

#[test]
fn original_hyperlink_history_and_borrowed_lookup_have_the_same_admission_boundary() {
    let (revision, base, encoding) = context();
    let bank = facts(
        &revision,
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "abc".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
    );
    for allowance in 0..=6 {
        let mut used = 0;
        let outcome = bank.lookup_units_with(&[97, 98, 99], &base, &encoding, &revision, || {
            if used == allowance {
                return Err("stop");
            }
            used += 1;
            Ok(())
        });
        if allowance < 6 {
            assert_eq!(
                outcome.unwrap_err(),
                HtmlEncodingUrlLookupError::Interrupted("stop")
            );
        } else {
            assert_eq!(outcome.unwrap(), &HtmlEncodingUrlParseOutcome::Failure);
        }
        assert_eq!(used, allowance);
        let mut history_used = 0;
        let history =
            bank.resolve_hyperlink_with(&"abc".into(), &base, &encoding, &revision, || {
                if history_used == allowance {
                    return Err("stop");
                }
                history_used += 1;
                Ok(())
            });
        if allowance < 6 {
            assert_eq!(
                history.unwrap_err(),
                HtmlEncodingUrlLookupError::Interrupted("stop")
            );
        } else {
            assert!(history.unwrap().url().is_none());
        }
        assert_eq!(history_used, allowance);
    }
}

#[test]
fn selected_historical_base_identity_does_not_replace_current_full_revision() {
    let (old, base, encoding) = context();
    let current = HtmlBaseFactsRevision::new(old.producer().clone(), 2, 3, 4);
    let result = HtmlBaseUrlRecord::observed(
        current.clone(),
        "https://observed.test/current".into(),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let bank = facts(
        &current,
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Parsed(result.clone()),
        )],
    );
    assert_ne!(base.revision(), &current);
    assert_eq!(
        bank.lookup_units_with(&[120], &base, &encoding, &current, || Ok::<_, Infallible>(
            ()
        ))
        .unwrap(),
        &HtmlEncodingUrlParseOutcome::Parsed(result)
    );
    assert_eq!(
        bank.lookup_units_with(&[120], &base, &encoding, &old, || Ok::<_, Infallible>(()))
            .unwrap_err(),
        HtmlEncodingUrlLookupError::Facts(HtmlEncodingUrlFactsError::RevisionMismatch)
    );
}
