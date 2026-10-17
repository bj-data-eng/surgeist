//! URL serialization/equality source: fde3f74f url.bs 3168–3271.
//! These observations explicitly attest the actual serializer boundary; this
//! fixture does not infer URL components from the serialized text.
#![forbid(unsafe_code)]

use std::{
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering},
};
use surgeist_html_semantics::{
    HtmlBaseFactsError, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn observed(serialization: &str, fragment: Option<usize>) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        serialization.into(),
        HtmlBaseUrlScheme::Other,
        false,
        fragment,
    )
    .unwrap()
}

fn equal(left: &HtmlBaseUrlRecord, right: &HtmlBaseUrlRecord) -> bool {
    left.equals_excluding_fragments_with(right, || Ok::<_, Infallible>(()))
        .unwrap()
}

#[test]
fn null_empty_and_different_fragments_compare_the_same_original_prefix() {
    // "about:blank" is eleven UTF-16 units. All three observations refer to
    // that exact prefix; null and empty fragments have different full strings.
    let blank = |serialization: &str, fragment| {
        HtmlBaseUrlRecord::observed(
            HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
            serialization.into(),
            HtmlBaseUrlScheme::Other,
            true,
            fragment,
        )
        .unwrap()
    };
    let null = blank("about:blank", None);
    let empty = blank("about:blank#", Some(11));
    let fragment = blank("about:blank#one", Some(11));
    let other_fragment = blank("about:blank#two", Some(11));
    assert_eq!(null.fragment_delimiter_offset(), None);
    assert_eq!(empty.fragment_delimiter_offset(), Some(11));
    assert_eq!(empty.serialization(), &"about:blank#".into());
    for other in [&empty, &fragment, &other_fragment] {
        assert!(equal(&null, other));
        assert!(equal(other, &null));
    }
    assert_ne!(
        null, empty,
        "value equality must not replace record identity"
    );
    assert_eq!(fragment, fragment.clone());
}

#[test]
fn exact_query_escape_and_path_contents_survive_fragment_exclusion() {
    // The nullable query contributes its '?' even when its value is empty.
    // URL equality compares the serializer output, with no decoding or trim.
    for (left, right, expected) in [
        ("x:path", "x:path?", false),
        ("x:path?", "x:path?", true),
        ("x:path?a=1", "x:path?a=2", false),
        ("x:%2f", "x:%2F", false),
        ("x:%23", "x:%23", true),
        ("x:path ", "x:path", false),
        ("x:path ", "x:path ", true),
        ("x:path", "x:Path", false),
    ] {
        let left = observed(left, None);
        let right = observed(right, None);
        assert_eq!(equal(&left, &right), expected);
        assert_eq!(equal(&right, &left), expected);
    }
    let query = observed("x:path?#old", Some(7));
    assert!(equal(&query, &observed("x:path?", None)));
    assert!(!equal(&query, &observed("x:path", None)));
}

#[test]
fn opaque_literal_number_sign_is_retained_until_the_attested_final_append() {
    // A non-parser provider's opaque path is "a#b". Null fragment retains
    // that literal path number sign. Its real fragment append starts at five.
    let null = observed("x:a#b", None);
    let empty = observed("x:a#b#", Some(5));
    let with_fragment = observed("x:a#b#fragment", Some(5));
    assert!(equal(&null, &empty));
    assert!(equal(&null, &with_fragment));
    assert!(!equal(&null, &observed("x:a", None)));

    // The identical complete text can describe a different actual record:
    // here the provider attests path "a" and fragment "b#fragment".
    let earlier_boundary = observed("x:a#b#fragment", Some(3));
    assert!(!equal(&with_fragment, &earlier_boundary));
    assert!(equal(&earlier_boundary, &observed("x:a", None)));
}

#[test]
fn fragment_metadata_checks_the_designated_unit_and_range_without_inference() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    // All invalid offsets are rejected before an owned record is formed.
    for offset in [0, 2, 6, usize::MAX] {
        assert_eq!(
            HtmlBaseUrlRecord::observed(
                revision.clone(),
                "x:a#b#".into(),
                HtmlBaseUrlScheme::Other,
                false,
                Some(offset),
            )
            .unwrap_err(),
            HtmlBaseFactsError::ContradictoryObservation,
        );
    }
    for offset in [3, 5] {
        let record = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "x:a#b#".into(),
            HtmlBaseUrlScheme::Other,
            false,
            Some(offset),
        )
        .unwrap();
        assert_eq!(record.fragment_delimiter_offset(), Some(offset));
        assert_eq!(record.serialization(), &"x:a#b#".into());
    }
    // Explicit null is meaningful even when full serialized path contains '#'.
    let null = observed("x:a#b#", None);
    assert_eq!(null.fragment_delimiter_offset(), None);
    assert!(equal(&null, &observed("x:a#b##ignored", Some(6))));
}

#[test]
fn different_producer_and_revision_records_compare_actual_values_independently() {
    let producer = HtmlBaseProducerId::fresh();
    let first = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(producer.clone(), 1, 2, 3),
        "x:path#old".into(),
        HtmlBaseUrlScheme::Other,
        false,
        Some(6),
    )
    .unwrap();
    let later = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(producer, 4, 5, 6),
        "x:path#new".into(),
        HtmlBaseUrlScheme::Other,
        false,
        Some(6),
    )
    .unwrap();
    let foreign = observed("x:path", None);
    assert_ne!(first.revision(), later.revision());
    assert_ne!(first.revision().producer(), foreign.revision().producer());
    assert_ne!(first, later);
    assert_ne!(first, foreign);
    assert!(equal(&first, &later));
    assert!(equal(&later, &foreign));
}

#[derive(Debug, Eq, PartialEq)]
enum Stop {
    WorkLimit { limit: usize },
    Cancelled,
}

#[test]
fn exact_work_and_each_callback_failure_remain_distinct_from_inequality() {
    let left = observed("x:abc#left", Some(5));
    let right = observed("x:abc#right", Some(5));
    // Two admitted record metadata reads, then two per each of five actual
    // prefix units. This is the public comparison tariff, not a layout guess.
    let exact = 2 + 2 * 5;
    for limit in 0..exact {
        let mut spent = 0;
        let result = left.equals_excluding_fragments_with(&right, || {
            if spent == limit {
                return Err(Stop::WorkLimit { limit });
            }
            spent += 1;
            Ok(())
        });
        assert_eq!(result, Err(Stop::WorkLimit { limit }));
        assert_eq!(spent, limit);
    }
    let mut spent = 0;
    assert_eq!(
        left.equals_excluding_fragments_with(&right, || {
            if spent == exact {
                return Err(Stop::WorkLimit { limit: exact });
            }
            spent += 1;
            Ok(())
        }),
        Ok(true)
    );
    assert_eq!(spent, exact);

    // A real mismatching first unit still cannot turn a refused operand read
    // into false. A length mismatch must admit both fixed metadata reads too.
    for (other, reached) in [(observed("y:abc", None), 4), (observed("x:abcd", None), 2)] {
        for limit in 0..reached {
            let mut spent = 0;
            assert_eq!(
                left.equals_excluding_fragments_with(&other, || {
                    if spent == limit {
                        return Err(Stop::WorkLimit { limit });
                    }
                    spent += 1;
                    Ok(())
                }),
                Err(Stop::WorkLimit { limit })
            );
            assert_eq!(spent, limit);
        }
        assert!(!equal(&left, &other));
    }
}

#[test]
fn cancellation_before_each_reached_read_propagates_without_changing_records() {
    let left = observed("x:abc#left", Some(5));
    let right = observed("x:abc#right", Some(5));
    let left_backing = left.serialization().shared_code_units().clone();
    let right_backing = right.serialization().shared_code_units().clone();
    for stop_before in 0..12 {
        let cancel = AtomicBool::new(false);
        let mut admitted = 0;
        let result = left.equals_excluding_fragments_with(&right, || {
            if admitted == stop_before {
                cancel.store(true, Ordering::Relaxed);
            }
            if cancel.load(Ordering::Relaxed) {
                return Err(Stop::Cancelled);
            }
            admitted += 1;
            Ok(())
        });
        assert_eq!(result, Err(Stop::Cancelled));
        assert_eq!(admitted, stop_before);
        assert!(std::sync::Arc::ptr_eq(
            &left_backing,
            left.serialization().shared_code_units()
        ));
        assert!(std::sync::Arc::ptr_eq(
            &right_backing,
            right.serialization().shared_code_units()
        ));
        assert!(
            equal(&left, &right),
            "a later comparison uses the original records"
        );
    }
}
