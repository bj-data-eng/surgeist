//! Authentic entry summaries and the resident authority needed to resume them.
use super::*;
use crate::block::entry::canonical::{EntryPosition, Separation};
use std::convert::Infallible;
use std::sync::Arc;

fn float_style<S: LayoutScalar>(height: f64) -> crate::NodeInputOf<S> {
    crate::NodeInputOf {
        float: crate::Float::Left,
        size: Size::new(
            crate::PreferredSizeOf::px(S::from_f64(20.0)),
            crate::PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..block(0.0, 0.0, None)
    }
}

fn scene<S: LayoutScalar>(local_float: bool, target: u32) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(9, if local_float { vec![0] } else { vec![1, 0] })
        .children(
            0,
            if local_float {
                vec![1, target]
            } else {
                vec![target]
            },
        )
        .children(1, [])
        .children(target, [])
        .style(9, block(0.0, 0.0, Some(100.0)))
        .style(0, block(0.0, 0.0, None))
        .style(1, float_style(if local_float { 5.0 } else { 10.0 }))
        .style(
            target,
            crate::NodeInputOf {
                clear: crate::Clear::Left,
                ..block(10.0, 0.0, Some(10.0))
            },
        )
}

fn receiving_frame<S: LayoutScalar>(
    owner: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
    external_float: bool,
) -> Arc<InheritedFloatExclusions<S, u32>> {
    let input = query::<S>(EntryPurpose::FramedEntry).input;
    let parent = block_container_projection::<_, Infallible>(owner, 9);
    let constants =
        Constants::from_projection::<_, Infallible>(owner, 9, &parent, input, Size::NONE).unwrap();
    let mut ledger = FloatExclusions::new(
        constants.flow_axes,
        Size::splat(S::from_f64(100.0)),
        S::from_f64(100.0),
        LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    if external_float {
        let style = BlockChildProjection::from_node(owner.node_input(1));
        let mut first = true;
        let prepared = in_flow::prepare_child(
            owner,
            1,
            &style,
            &constants,
            in_flow::ChildSizingContext {
                inner_size: constants.node_inner_size,
                request: constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: crate::RequestedAxis::Both,
                first_in_flow: &mut first,
            },
        )
        .unwrap();
        let output = owner.compute_child(1, prepared.input).unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));
        let pending = PendingFloat {
            placement: crate::block::floats::PendingPlacement::Unresolved,
            node: 1,
            source_index: 0,
            side: style.float,
            clear: style.clear,
            block_start: S::ZERO,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            border: prepared.border,
            padding: prepared.padding,
            margin: Edges::all(S::ZERO),
            float_exclusion: style.float_exclusion,
            child_compute_geometry: output.scroll_geometry,
        };
        let placement = ledger
            .place_float_for_line::<Infallible>(9, &pending, None)
            .unwrap();
        assert_eq!(placement.logical.block, S::ZERO);
    }
    Arc::new(ledger.for_ordinary_child(LogicalPointOf::new(S::ZERO, S::ZERO)))
}

fn framed_request<S: LayoutScalar>(
    frame: Arc<InheritedFloatExclusions<S, u32>>,
    cut: Option<Separation<u32, S>>,
    purpose: EntryPurpose,
) -> EntryQuery<u32, S> {
    let mut request = query(purpose);
    // Component and framed-entry queries use the same real incoming protocol.
    let mut incoming = query::<S>(EntryPurpose::FramedEntry).incoming.unwrap();
    incoming.frame = frame;
    incoming.cut = cut;
    request.incoming = Some(incoming);
    if purpose == EntryPurpose::Component {
        request.position = Some(EntryPosition {
            cursor: S::ZERO,
            active: CollapsibleMarginOf::ZERO,
            start_open: false,
            collapses_start: false,
            child_adjoins_parent: true,
            own_start: CollapsibleMarginOf::ZERO,
            inline_offset: S::ZERO,
        });
    }
    request
}

fn summary<S: LayoutScalar>(
    owner: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
    frame: Arc<InheritedFloatExclusions<S, u32>>,
) -> EntrySummary<u32, S> {
    let EntryObservation::Summary(summary) = owner
        .qualify_entry(framed_request(frame, None, EntryPurpose::Component))
        .unwrap()
    else {
        panic!("this complete box-only component has an issued summary");
    };
    summary
}

fn install<S: LayoutScalar>(
    owner: &mut ComputeSession<'_, PublicLayoutTreeOf<S>>,
    incoming: float_context::Incoming<u32, S>,
) {
    let resident = owner.block_float_state().unwrap();
    *resident = resident.fork_framed(incoming);
}

fn required_basis<S: LayoutScalar>(error: &crate::LayoutErrorOf<u32, S, Infallible>) {
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), crate::LayoutOperation::ChildLayout);
    assert!(matches!(
        error.kind(),
        crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis)
    ));
}

fn admitted_hypothetical<S: LayoutScalar>() {
    let local = scene::<S>(true, 2);
    let mut owner = ComputeSession::new(&local, Vec::new());
    let frame = receiving_frame(&mut owner, false);
    let local_summary = summary(&mut owner, frame);
    let cut = local_summary.separation.expect("H10 precedes local Fh15");
    assert_eq!((cut.container, cut.node, cut.index), (0, 2, 1));
    assert_eq!(cut.hypothetical.get(), S::from_f64(10.0));
    assert_eq!(cut.relevant_bottom.get(), S::from_f64(15.0));
    assert_eq!(local_summary.start_fact().margin.resolve(), S::ZERO);
    local_summary.validate_separation(&mut owner).unwrap();
    assert!(owner.staged_unrounded(0).is_err());
    assert!(owner.staged_unrounded(2).is_err());

    // The same MT10 child against an external F10 is exact contact. Absence
    // of a separation is distinct from the local introduced-zero case above;
    // its later settlement amount is covered by the public geometry tests.
    let inherited = scene::<S>(false, 2);
    let mut owner = ComputeSession::new(&inherited, Vec::new());
    let frame = receiving_frame(&mut owner, true);
    let contact = summary(&mut owner, frame);
    assert!(contact.separation.is_none());
    assert_eq!(contact.start_fact().margin.resolve(), S::from_f64(10.0));
    contact.validate_separation(&mut owner).unwrap();
}

fn resident_frame_authority<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::new()
        .children(0, [2, 3])
        .style(0, block::<S>(0.0, 0.0, None))
        .style(2, block(5.0, 7.0, None))
        .style(3, block(29.0, 0.0, Some(20.0)))
        .measure_with(2, zero::<S>);
    for variant in 0..3 {
        let mut owner = ComputeSession::new(&tree, Vec::new());
        let mut request = query::<S>(EntryPurpose::FramedEntry);
        if variant != 1 {
            install(&mut owner, request.incoming.as_ref().unwrap().clone());
        }
        if variant == 2 {
            // Equal coordinates from a different snapshot do not establish
            // identity with the actually installed resident frame.
            let original = request.incoming.as_ref().unwrap().frame.clone();
            let other = query::<S>(EntryPurpose::FramedEntry)
                .incoming
                .unwrap()
                .frame;
            assert_eq!(
                original.common_border(S::ZERO).unwrap(),
                other.common_border(S::ZERO).unwrap()
            );
            assert!(!Arc::ptr_eq(&original, &other));
            request.incoming.as_mut().unwrap().frame = other;
        }
        CALLS.with(|calls| calls.set(0));
        let result = crate::block::entry::resolve_framed(&mut owner, request);
        if variant == 0 {
            let EntryObservation::Summary(summary) = result.unwrap() else {
                panic!("an authenticated resident frame resumes the opaque leaf");
            };
            assert_eq!(summary.start_fact().margin.resolve(), S::from_f64(29.0));
            CALLS.with(|calls| assert_eq!(calls.get(), 1));
        } else {
            required_basis(
                &result
                    .err()
                    .expect("missing or foreign residency must refuse"),
            );
            CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }
}

fn source_cut_authority<S: LayoutScalar>() {
    let tree = scene::<S>(true, 2);
    let mut owner = ComputeSession::new(&tree, Vec::new());
    let frame = receiving_frame(&mut owner, false);
    let proof = summary(&mut owner, frame);
    proof.validate_separation(&mut owner).unwrap();

    // The target is still the same node, but is no longer at the issued cut.
    let reordered = tree.clone().children(0, [2, 1]);
    let mut changed = ComputeSession::new(&reordered, Vec::new());
    required_basis(&proof.validate_separation(&mut changed).unwrap_err());

    // A changed relevant side cannot reuse the old left-float bound.
    let wrong_side = tree.clone().style(
        2,
        crate::NodeInputOf {
            clear: crate::Clear::Right,
            ..block(10.0, 0.0, Some(10.0))
        },
    );
    let mut changed = ComputeSession::new(&wrong_side, Vec::new());
    required_basis(&proof.validate_separation(&mut changed).unwrap_err());
}

fn resident_cut_authority<S: LayoutScalar>() {
    let tree = scene::<S>(true, 2);
    let mut producer = ComputeSession::new(&tree, Vec::new());
    let frame = receiving_frame(&mut producer, false);
    let first = summary(&mut producer, frame.clone()).separation.unwrap();
    let other_tree = scene::<S>(true, 3);
    let mut other_producer = ComputeSession::new(&other_tree, Vec::new());
    let other_frame = receiving_frame(&mut other_producer, false);
    let other = summary(&mut other_producer, other_frame)
        .separation
        .unwrap();
    // Both authentic cuts have H10/Fh15, but name different source occurrences.
    assert_eq!(first.hypothetical.get(), S::from_f64(10.0));
    assert_eq!(first.relevant_bottom.get(), S::from_f64(15.0));
    assert_eq!(first.hypothetical, other.hypothetical);
    assert_eq!(first.relevant_bottom, other.relevant_bottom);
    assert!(!first.same_cut(other));
    for supplied in [Some(first), None, Some(other)] {
        let mut owner = ComputeSession::new(&tree, Vec::new());
        let installed = framed_request(frame.clone(), Some(first), EntryPurpose::Component);
        install(&mut owner, installed.incoming.unwrap());
        let request = framed_request(frame.clone(), supplied, EntryPurpose::Component);
        let result = crate::block::entry::resolve_framed(&mut owner, request);
        if supplied.is_some_and(|cut| cut.same_cut(first)) {
            let EntryObservation::Summary(prefix) = result.unwrap() else {
                panic!("the resident cut stops before its own top group");
            };
            assert_eq!(prefix.start_fact().margin.resolve(), S::ZERO);
        } else {
            required_basis(&result.err().expect("an absent or foreign cut must refuse"));
        }
    }
}

#[test]
fn admitted_hypothetical_and_contact_f32() {
    admitted_hypothetical::<f32>();
}
#[test]
fn admitted_hypothetical_and_contact_f64() {
    admitted_hypothetical::<f64>();
}
#[test]
fn resident_frame_requires_actual_custody_f32() {
    resident_frame_authority::<f32>();
}
#[test]
fn resident_frame_requires_actual_custody_f64() {
    resident_frame_authority::<f64>();
}
#[test]
fn separation_revalidates_source_cut_and_side_f32() {
    source_cut_authority::<f32>();
}
#[test]
fn separation_revalidates_source_cut_and_side_f64() {
    source_cut_authority::<f64>();
}
#[test]
fn resident_cut_retains_occurrence_beyond_numeric_bounds_f32() {
    resident_cut_authority::<f32>();
}
#[test]
fn resident_cut_retains_occurrence_beyond_numeric_bounds_f64() {
    resident_cut_authority::<f64>();
}
