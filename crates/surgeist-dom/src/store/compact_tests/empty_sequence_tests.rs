use super::*;
use crate::store::empty_sequence_observation::{Kind, Observation};
use crate::{DomProcessingInstruction, DomStyleInlineCspObservation};

fn string_extent(value: &DocumentString) -> (usize, usize) {
    let capacity = value.shared_code_units().capacity();
    (
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    )
}
fn expected_pi(
    pi: &DomProcessingInstruction,
    graph: &AllocationGraph,
    present: bool,
) -> (usize, usize) {
    let mut bytes = 0;
    let mut units = 0;
    for value in [pi.target(), pi.data()]
        .into_iter()
        .chain(pi.attributes().flat_map(|(n, v)| [n, v]))
    {
        let (b, u) = string_extent(value);
        bytes += b;
        units += u;
    }
    if present {
        bytes += size_of::<Vec<(DocumentString, DocumentString)>>();
        bytes += graph
            .summary()
            .get("pi.attributes.buffer")
            .map_or(0, |(_, bytes)| *bytes);
    }
    (bytes, units)
}

#[test]
fn native_empty_pi_has_no_pseudoattribute_target_or_receipt() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let document = document(&mut dom);
    let observed = scale_metrics::ENABLED.then(|| Observation::begin(Kind::PiAttributes));
    let mut batch = DomBatch::new();
    let pi = batch.create_processing_instruction(document, "xml-sheet", "");
    let commit = dom.apply(batch, &cancel).unwrap();
    let id = commit.created(&pi).unwrap().unwrap();
    let actual = observed.map(Observation::finish);
    let snapshot = dom.snapshot();
    let DomNodeKind::ProcessingInstruction(pi) = snapshot.node(id).unwrap().kind() else {
        panic!("native PI")
    };
    assert_eq!(pi.target(), &DocumentString::from("xml-sheet"));
    assert!(pi.data().is_empty());
    assert_eq!(pi.attributes().len(), 0);
    let mut graph = AllocationGraph::default();
    pi.inspect_compact_baseline(&mut graph);
    let expected = expected_pi(pi, &graph, false);
    let receipt = pi.retained_payload(|| Ok(())).unwrap();
    eprintln!(
        "empty PI targets={actual:?} raw={:?} actual_receipt={:?} expected={expected:?}",
        graph.summary(),
        (receipt.bytes, receipt.units)
    );
    if let Some(actual) = actual {
        assert_eq!(actual.targets, 0, "native empty PI allocates no map target");
        assert_eq!((actual.target_bytes, actual.capacity_bytes), (0, 0));
    }
    assert_eq!(graph.count("pi.attributes.target"), 0);
    assert_eq!((receipt.bytes, receipt.units), expected);
}

#[test]
fn source_empty_pi_admits_only_its_actual_string_occurrences() {
    let dom = Dom::new(limits()).unwrap();
    let cancel = AtomicBool::new(false);
    let batch = DomBatch::new();
    let mut draft = Draft::new_source(
        &dom,
        batch.owner,
        &batch.commands,
        crate::DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 16, 64, 64, 1),
        &cancel,
    )
    .unwrap()
    .0;
    let target = DocumentString::from("pi");
    let data = DocumentString::default();
    let expected = string_extent(&target).0 + string_extent(&data).0;
    let original = draft.source_limits.unwrap();
    draft.custody_registry().reconcile_releases();
    let baseline = draft.custody_ledger().total();
    for short in [true, false] {
        draft.source_limits = Some(original);
        let boundary = draft.custody_ledger().total().bytes() + draft.source_other_bytes + expected;
        let limit = boundary - usize::from(short);
        draft.source_limits.as_mut().unwrap().retained_bytes = limit;
        let formation = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let result = DomProcessingInstruction::new(&target, &data, Some(&formation), &cancel);
        if short {
            assert!(
                matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit: actual }) if actual == limit),
                "actual string-only B−1={limit} must refuse: {result:?}"
            );
        } else {
            assert!(
                result.is_ok(),
                "actual empty PI must fit raw string-only B={boundary}: {result:?}"
            );
            let pi = result.unwrap();
            let receipt = pi.retained_payload(|| Ok(())).unwrap();
            assert_eq!(receipt.bytes, expected);
            assert_eq!(pi.attributes().len(), 0);
            drop(pi);
        }
        drop(formation);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), baseline);
    }
}

#[test]
fn reserved_populated_and_invalid_pi_maps_preserve_actual_backing_and_order() {
    for populated in [false, true] {
        let mut attributes = Vec::with_capacity(7);
        if populated {
            attributes.extend([
                ("b".into(), "second".into()),
                ("a".into(), "first".into()),
                ("b".into(), "duplicate".into()),
            ]);
        }
        let capacity = attributes.capacity();
        let observed = scale_metrics::ENABLED.then(|| Observation::begin(Kind::PiAttributes));
        let pi = DomProcessingInstruction::from_parsed("pi".into(), "literal".into(), attributes);
        let clone = pi.clone();
        if let Some(observed) = observed {
            let actual = observed.finish();
            assert_eq!(actual.targets, 1);
            assert_eq!(
                actual.target_bytes,
                size_of::<Vec<(DocumentString, DocumentString)>>()
            );
            assert_eq!(
                actual.capacity_bytes,
                capacity * size_of::<(DocumentString, DocumentString)>()
            );
        }
        let mut graph = AllocationGraph::default();
        pi.inspect_compact_baseline(&mut graph);
        let before = graph.clone();
        clone.inspect_compact_baseline(&mut graph);
        assert_eq!(graph, before);
        assert_eq!(graph.count("pi.attributes.target"), 1);
        assert_eq!(graph.count("pi.attributes.buffer"), 1);
        let receipt = pi.retained_payload(|| Ok(())).unwrap();
        assert_eq!(
            (receipt.bytes, receipt.units),
            expected_pi(&pi, &graph, true)
        );
        drop(pi);
        let pairs: Vec<_> = clone
            .attributes()
            .map(|(n, v)| (n.to_utf8().unwrap(), v.to_utf8().unwrap()))
            .collect();
        if populated {
            assert_eq!(
                pairs,
                [
                    (String::from("b"), String::from("second")),
                    (String::from("a"), String::from("first")),
                    (String::from("b"), String::from("duplicate"))
                ]
            );
        } else {
            assert!(pairs.is_empty());
        }
    }
    let pi = DomProcessingInstruction::new(
        &"pi".into(),
        &"a='valid' broken".into(),
        None,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(pi.attributes().len(), 0);
    let mut graph = AllocationGraph::default();
    pi.inspect_compact_baseline(&mut graph);
    assert_eq!(graph.count("pi.attributes.target"), 1);
    assert!(
        graph.summary()["pi.attributes.buffer"].1 > 0,
        "failed parsing retains its real reserved bank"
    );
    let receipt = pi.retained_payload(|| Ok(())).unwrap();
    assert_eq!(
        (receipt.bytes, receipt.units),
        expected_pi(&pi, &graph, true)
    );
}

#[test]
fn complete_empty_style_capture_has_no_policy_target() {
    let (mut dom, document, style) = source::operation::empty_policy_tests::style_fixture();
    let (_lease, request) =
        source::operation::empty_policy_tests::begin_style(&mut dom, document, style);
    let observed = scale_metrics::ENABLED.then(|| Observation::begin(Kind::StylePolicies));
    let capture = DomStyleInlineCspObservation::complete_empty(request.source(), 31);
    let actual = observed.map(Observation::finish);
    assert!(capture.allowed());
    assert!(capture.policies().is_empty());
    assert_eq!(capture.policy_revision(), 31);
    let mut graph = AllocationGraph::default();
    capture.inspect_policy_allocations(&mut graph);
    eprintln!(
        "complete empty Style targets={actual:?} raw={:?}",
        graph.summary()
    );
    if let Some(actual) = actual {
        assert_eq!(actual.targets, 0);
        assert_eq!((actual.target_bytes, actual.capacity_bytes), (0, 0));
    }
    assert_eq!(graph.count("style.policies.target"), 0);
    assert_eq!(graph.bytes(), 0);
}

#[test]
fn evaluated_style_capture_preserves_supplied_empty_reserved_and_populated_arcs() {
    for case in 0..3 {
        let (mut dom, document, style) = source::operation::empty_policy_tests::style_fixture();
        let (_lease, request) =
            source::operation::empty_policy_tests::begin_style(&mut dom, document, style);
        let mut values = if case == 0 {
            Vec::new()
        } else {
            Vec::with_capacity(7)
        };
        if case == 2 {
            values.extend([
                DocumentString::from("second"),
                "first".into(),
                "second".into(),
            ]);
        }
        let capacity = values.capacity();
        let values = Arc::new(values);
        let weak = Arc::downgrade(&values);
        let observed = scale_metrics::ENABLED.then(|| Observation::begin(Kind::StylePolicies));
        let capture =
            DomStyleInlineCspObservation::evaluated(request.source(), values.clone(), 41, false);
        let clone = capture.clone();
        if let Some(observed) = observed {
            assert_eq!(
                observed.finish().targets,
                0,
                "the caller allocated this target"
            );
        }
        drop(values);
        assert!(
            weak.upgrade().is_some(),
            "even supplied zero-capacity Arc remains retained"
        );
        let mut graph = AllocationGraph::default();
        capture.inspect_policy_allocations(&mut graph);
        let before = graph.clone();
        clone.inspect_policy_allocations(&mut graph);
        assert_eq!(graph, before);
        assert_eq!(graph.count("style.policies.target"), 1);
        assert_eq!(
            graph.count("style.policies.buffer"),
            usize::from(capacity != 0)
        );
        let mut strings = AllocationGraph::default();
        for value in capture.policies() {
            strings.string(value);
        }
        assert_eq!(
            graph.bytes(),
            strings.bytes()
                + size_of::<Vec<DocumentString>>()
                + capacity * size_of::<DocumentString>()
        );
        assert!(!capture.allowed());
        drop(capture);
        assert!(weak.upgrade().is_some());
        if case == 2 {
            assert_eq!(
                clone.policies(),
                &[
                    DocumentString::from("second"),
                    DocumentString::from("first"),
                    DocumentString::from("second")
                ]
            );
        }
        drop(clone);
        assert!(weak.upgrade().is_none());
    }
}
