//! Authority: adopted #1627 physical parent-side endpoint schedule and the
//! existing #1623 NativeOccupied contract. One actual in-flow atomic has raw
//! occupied [0,10], border10 and independent native flow6. Source-Start2 is
//! allocated to physical Right; Source-End3 is its physical Left counterpart.
//! Hence End3,atomic,Start2 => atomic[3,13], signed flow11, Native requirement13.
//! Plain Contribution preserves Start2,atomic,End3 => atomic[2,12], requirement12.
//! All three measurement views have identical operands here: no text, trimming,
//! hanging, tabs, supplied spacing, CSS atomic fit receipt or scope displacement.
//! Selected component BoxEdges activate CSS-composed authority while this
//! atomic's actual NativeOccupied interval remains an independent constraint.

use super::*;
use crate::protocol::InlineBoxSource;

fn atomic_family() -> (Arc<ProcessedSource>, System, PreparedText) {
    let mut value = captured_input(1);
    value.source = Source::identified("", Id::from_u64(1_627_717), 1);
    value.source.inline_box(InlineBox::new(
        Id::from_u64(1_627_718),
        InlineBoxKind::InFlow,
        0,
        Size::new(10.0, 10.0),
    ));
    value.origins.clear();
    let own = style(10.0, 10.0);
    assert_eq!(own.wrap, Wrap::Word);
    domain(&mut value, own.clone(), None);
    inline(&mut value, 1, None, None, Some(own));
    // Both actual root and Inline contexts use Word. This source contains no
    // original connections and one real in-flow atomic occurrence.
    value.boundary_policies = [
        ParagraphBoundaryDefinition::AtomicBefore(0),
        ParagraphBoundaryDefinition::AtomicAfter(0),
    ]
    .into_iter()
    .map(|definition| ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    })
    .collect();
    let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 0, end: 1 });
    value.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Control {
            control: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let (capture, analyzed) = analyze(value);
    assert!(capture.source().text().is_empty());
    let mut system = system(AHEM, KERN);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.origin_views().len(), 0);
    assert_eq!(prepared.atomic_views().len(), 1);
    let atomic = prepared.atomic_views().next().unwrap();
    assert_eq!(atomic.original_box().kind(), InlineBoxKind::InFlow);
    assert_eq!(atomic.original_box().size(), Size::new(10.0, 10.0));
    assert_eq!(atomic.source_position().get(), 0);
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let scope = capture.captured_scope(1).unwrap();
    assert_eq!(scope.entry_control(), controls[0].captured_control());
    assert_eq!(scope.exit_control(), controls[1].captured_control());
    assert_eq!(scope.parent(), capture.captured_scope(0));
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        vec![
            PreparedParticipant::Control(controls[0].clone()),
            PreparedParticipant::Atomic(atomic.handle()),
            PreparedParticipant::Control(controls[1].clone()),
        ]
    );
    (capture, system, prepared)
}

fn atomic_request(
    capture: &Arc<ProcessedSource>,
    prepared: &PreparedText,
    boxed: bool,
) -> LineSelectionRequest {
    let segment = prepared.source_segments().next().unwrap();
    let mut request = selected_request(prepared, segment.start(), segment.end_opportunity());
    // Public select validates the complete request; it is not the Layout fit
    // chooser. A generous band deliberately isolates measurement from wrapping.
    request.context.available_inline = Some(100.0);
    request.context.spacing = SpacingPolicy::None;
    request.context.alignment = Alignment::Start;
    let context = prepared
        .inline_box_edge_context(
            InlineBoxSource::captured(capture.captured_scope(1).unwrap()),
            InlineBoxSource::captured(capture.captured_scope(0).unwrap()),
            LineEdgeSide::End,
        )
        .unwrap();
    assert_eq!(context.start_side(), LineEdgeSide::End);
    request.controls = prepared
        .control_views()
        .zip([2.0, 3.0])
        .map(|(view, advance)| ControlAllocation {
            occurrence: view.handle(),
            strut: None,
            boundary: Some(if boxed {
                InlineBoundaryAllocation::box_edge(
                    context.clone(),
                    crate::InlineBoxEdgeComponents::from_f64(0.0, advance, 0.0).unwrap(),
                )
            } else {
                InlineBoundaryAllocation::contribution(advance).unwrap()
            }),
        })
        .collect();
    request.atomics = vec![AtomicAllocation {
        occurrence: prepared.atomic_views().next().unwrap().handle(),
        allocation: AtomicLineAllocation::InFlow {
            flow: AtomicFlowAllocation {
                advance: 6.0,
                before_baseline: 8.0,
                after_baseline: 2.0,
                occupied_start: 0.0,
                occupied_end: 10.0,
            },
            fit: AtomicFitAllocation::NativeOccupied,
            box_geometry: AtomicBoxAllocation {
                border: Rect::new(0.0, -8.0, 10.0, 10.0),
                content: Rect::new(0.0, -8.0, 10.0, 10.0),
                baseline: 0.0,
            },
            alignment: LineBoxAlignment::Baseline,
        },
    }];
    request
}

fn finish_atomic(prepared: &PreparedText, request: LineSelectionRequest) -> FinalizedLine {
    prepared
        .select(request)
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn atomic_geometry(prepared: &PreparedText, line: &FinalizedLine, boxed: bool) {
    assert_eq!(line.prepared_text(), *prepared);
    assert!(line.run_views().all(|run| run.glyphs().next().is_none()));
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    let expected = if boxed {
        [(9.0, 11.0), (0.0, 3.0)]
    } else {
        [(0.0, 2.0), (8.0, 11.0)]
    };
    for (control, expected) in controls.into_iter().zip(expected) {
        let slot = line
            .control_slots()
            .find(|v| v.control() == control)
            .unwrap();
        assert_eq!(slot.strip_interval(), expected);
    }
    let slots = line.atomic_slots().collect::<Vec<_>>();
    let [slot] = slots.as_slice() else {
        panic!("one real atomic slot")
    };
    assert_eq!(
        slot.occurrence,
        prepared.atomic_views().next().unwrap().handle()
    );
    let AtomicLineAllocation::InFlow {
        flow,
        fit,
        box_geometry,
        ..
    } = slot.allocation
    else {
        panic!("genuine in-flow atomic")
    };
    assert_eq!(fit, AtomicFitAllocation::NativeOccupied);
    assert_eq!(
        (flow.advance, flow.occupied_start, flow.occupied_end),
        (6.0, 0.0, 10.0)
    );
    assert_eq!(box_geometry.border, Rect::new(0.0, -8.0, 10.0, 10.0));
    let incoming = if boxed { 3.0 } else { 2.0 };
    assert_eq!(slot.origin.x, incoming);
    assert_eq!(
        (
            slot.origin.x + flow.occupied_start,
            slot.origin.x + flow.occupied_end
        ),
        (incoming, incoming + 10.0)
    );
    assert_eq!(line.feedback().advance, 11.0);
}

fn atomic_measurements(line: &FinalizedLine, occupied_end: f64, boxed: bool) {
    let measurements = line.measurements();
    let raw = measurements.fit();
    assert_eq!(
        (raw.advance(), raw.occupied_start(), raw.occupied_end()),
        (11.0, 0.0, occupied_end)
    );
    assert_eq!(
        (line.feedback().occupied_start, line.feedback().occupied_end),
        (0.0, occupied_end)
    );
    for admission in [
        measurements.margin_edge_fit(),
        measurements.margin_edge_min_content_contribution(),
        measurements.margin_edge_max_content_contribution(),
    ] {
        assert_eq!(
            admission.authority(),
            if boxed {
                MarginEdgeFitAuthority::CssComposed
            } else {
                MarginEdgeFitAuthority::NativeOnly
            }
        );
        assert_eq!(
            admission.native_occupied_bounds(),
            if boxed { Some((3.0, 13.0)) } else { None }
        );
        assert_eq!(admission.advance(), 11.0);
        assert_eq!(
            (admission.constraint_start(), admission.constraint_end()),
            (0.0, occupied_end)
        );
        assert_eq!(admission.required_inline_size(), occupied_end);
    }
}

#[test]
fn physical_parent_edges_measure_the_atomic_at_its_executed_position() {
    let (capture, system, prepared) = atomic_family();
    let plain_request = atomic_request(&capture, &prepared, false);
    let prior = finish_atomic(&prepared, plain_request.clone());
    atomic_geometry(&prepared, &prior, false);
    atomic_measurements(&prior, 12.0, false);
    // Only endpoint authority changes. The atomic occurrence, source, levels,
    // Native raw geometry, independent advance6 and every other request stay.
    let boxed_request = atomic_request(&capture, &prepared, true);
    let mut demoted = boxed_request.clone();
    for (allocation, amount) in demoted.controls.iter_mut().zip([2.0, 3.0]) {
        allocation.boundary = Some(InlineBoundaryAllocation::contribution(amount).unwrap());
    }
    assert_eq!(demoted, plain_request);
    let boxed = finish_atomic(&prepared, boxed_request.clone());
    atomic_geometry(&prepared, &boxed, true);
    atomic_measurements(&boxed, 13.0, true);
    assert_eq!(boxed.selected_line().request(), &boxed_request);
    let stats = system.stats();
    let replay = boxed.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &boxed_request);
    atomic_geometry(&prepared, &replay, true);
    atomic_measurements(&replay, 13.0, true);
    // Both independent owners remain usable while the replay coexists.
    assert_eq!(prior.selected_line().request(), &plain_request);
    atomic_geometry(&prepared, &prior, false);
    atomic_measurements(&prior, 12.0, false);
    atomic_geometry(&prepared, &boxed, true);
    atomic_measurements(&boxed, 13.0, true);
}
