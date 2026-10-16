use super::*;
use crate::{CssParserContext, CssRetainedPayloadError, CssSourceRetention, CssUtf16InputLimits};
fn source(original: Arc<Vec<u16>>) -> CssSourceSnapshot {
    CssParserContext::default()
        .parse_sheet_utf16_with_limits(original, CssUtf16InputLimits::default(), || Ok::<_, ()>(()))
        .unwrap()
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}
fn observed_snapshot(source: &CssSourceSnapshot) -> usize {
    let data = &source.0;
    std::mem::size_of::<SourceSnapshotData>()
        + data.text.capacity()
        + data.checkpoints.capacity() * std::mem::size_of::<CssSourcePosition>()
        + data.utf16.as_ref().map_or(0, |value| {
            value.map.capacity() * std::mem::size_of::<(usize, usize)>()
        })
}
#[test]
fn complete_utf8_payload_keeps_existing_backing_quota_and_identity_policy() {
    let mut text = String::with_capacity(213);
    text.push_str("/* comment */");
    let text_bytes = text.capacity();
    let first = CssSourceSnapshot::try_from_owned(text).unwrap();
    let equal_distinct = CssSourceSnapshot::new(first.as_str());
    assert_eq!(first, equal_distinct);
    assert!(!first.same_snapshot(&equal_distinct));
    let first_backing =
        text_bytes + first.0.checkpoints.capacity() * std::mem::size_of::<CssSourcePosition>();
    assert_eq!(first.storage_usage().retained_bytes(), first_backing);
    let expected = observed_snapshot(&first) + observed_snapshot(&equal_distinct);
    let mut retained = CssSourceRetention::new(usize::MAX, 100, 10);
    retained.source(&first).unwrap();
    retained.source(&first.clone()).unwrap();
    retained.source(&equal_distinct).unwrap();
    let old_usage = retained.usage();
    let old_steps = retained.steps();
    assert_eq!(
        retained.complete_payload_bytes_with(|| Ok::<_, ()>(())),
        Ok(expected)
    );
    assert_eq!(retained.steps(), old_steps);
    assert_eq!(retained.usage(), old_usage);
    assert_eq!(
        old_usage.retained_bytes(),
        first.storage_usage().retained_bytes() + equal_distinct.storage_usage().retained_bytes()
    );
}
#[test]
fn shared_utf16_original_is_unique_across_distinct_snapshots_including_empty_header() {
    for text in ["", "a\r\n\0𐀀"] {
        let mut units = Vec::with_capacity(107);
        units.extend(text.encode_utf16());
        let capacity = units.capacity();
        let original = Arc::new(units);
        let first = source(original.clone());
        let second = source(original.clone());
        assert!(!first.same_snapshot(&second));
        assert!(Arc::ptr_eq(
            first.original_utf16().unwrap(),
            second.original_utf16().unwrap()
        ));
        let expected = observed_snapshot(&first)
            + observed_snapshot(&second)
            + std::mem::size_of::<Vec<u16>>()
            + capacity * std::mem::size_of::<u16>();
        let mut retained = CssSourceRetention::new(usize::MAX, 100, 10);
        retained.source(&first).unwrap();
        retained.source(&second).unwrap();
        assert_eq!(
            retained.complete_payload_bytes_with(|| Ok::<_, ()>(())),
            Ok(expected)
        );
        // Equal contents in a different original allocation contribute separately.
        let distinct = source(Arc::new(original.as_ref().clone()));
        let extra = observed_snapshot(&distinct)
            + std::mem::size_of::<Vec<u16>>()
            + distinct.original_utf16().unwrap().capacity() * std::mem::size_of::<u16>();
        retained.source(&distinct).unwrap();
        assert_eq!(
            retained.complete_payload_bytes_with(|| Ok::<_, ()>(())),
            Ok(expected + extra)
        );
    }
}
#[test]
fn payload_observation_admits_each_action_and_leaves_collector_unchanged_on_refusal() {
    let original = Arc::new(vec![b'a' as u16]);
    let mut retained = CssSourceRetention::new(usize::MAX, 100, 10);
    retained.source(&source(original.clone())).unwrap();
    retained.source(&source(original)).unwrap();
    let usage = retained.usage();
    let visits = retained.steps();
    let mut steps = 0;
    let expected = retained
        .complete_payload_bytes_with(|| {
            steps += 1;
            Ok::<_, usize>(())
        })
        .unwrap();
    for allowance in 0..steps {
        let mut reached = 0;
        assert_eq!(
            retained.complete_payload_bytes_with(|| {
                if reached == allowance {
                    return Err(reached);
                }
                reached += 1;
                Ok(())
            }),
            Err(CssRetainedPayloadError::Admission(allowance))
        );
        assert_eq!(reached, allowance);
        assert_eq!(retained.usage(), usage);
        assert_eq!(retained.steps(), visits);
    }
    assert_eq!(
        retained.complete_payload_bytes_with(|| Ok::<_, usize>(())),
        Ok(expected)
    );
}
