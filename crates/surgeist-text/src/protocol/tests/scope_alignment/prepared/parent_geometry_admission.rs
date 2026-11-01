//! Checked captured/native box ownership, parentage and endpoint contexts.
//! Authentic nominations and event order distinguish foreign, crossing and
//! inconsistent declarations; refusal preserves existing selected products.

use super::*;
use crate::protocol::{InlineBoxEdgeContext, InlineBoxEdgeError, InlineBoxSource};

struct Family {
    capture: Arc<ProcessedSource>,
    system: System,
    prepared: PreparedText,
    controls: Vec<PreparedControl>,
    owners: Vec<InlineBoxSource>,
}

fn family(depth: usize, nominated: bool) -> Family {
    family_body(depth, nominated, "A", WhiteSpace::Preserve)
}
fn family_body(depth: usize, nominated: bool, original: &str, whitespace: WhiteSpace) -> Family {
    assert!(depth > 0);
    assert_eq!(original.encode_utf16().count(), 1);
    let mut value = super::captured_input(if nominated { depth } else { 0 });
    super::raw_source(&mut value, original);
    let mut body = super::style(10.0, 10.0);
    body.white_space = whitespace;
    super::domain(&mut value, body.clone(), None);
    if nominated {
        for index in 1..=depth {
            super::inline(&mut value, index, None, None, Some(body.clone()));
            let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[index] else {
                unreachable!()
            };
            *boundaries = Some(SourceProcessingScopeBoundaries {
                start: index - 1,
                end: 2 * depth - index,
            });
        }
    }
    value.controls = (0..depth)
        .map(|_| ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        })
        .chain((0..depth).map(|_| ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        }))
        .collect();
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for index in 0..depth {
        if nominated {
            value
                .events
                .push(SourceProcessingEvent::EnterScope(index + 1));
        }
        value.events.push(SourceProcessingEvent::Control {
            control: index,
            affinity: Affinity::After,
        });
    }
    value.events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    for index in (0..depth).rev() {
        value.events.push(SourceProcessingEvent::Control {
            control: 2 * depth - index - 1,
            affinity: Affinity::After,
        });
        if nominated {
            value
                .events
                .push(SourceProcessingEvent::ExitScope(index + 1));
        }
    }
    value.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let (capture, analyzed) = super::analyze(value);
    assert_eq!(capture.source().text(), &Text::from(original));
    assert_eq!(
        capture.processed_text(),
        &Text::from(if whitespace == WhiteSpace::Discard {
            ""
        } else {
            original
        })
    );
    let mut system = super::system(AHEM, KERN);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2 * depth);
    for (index, control) in controls.iter().enumerate() {
        let real = capture.captured_control(index).unwrap();
        assert_eq!(control.captured_control(), Some(real.clone()));
        assert_eq!(real.paragraph(), capture.captured_paragraph(0).unwrap());
        assert_eq!(
            *real.kind(),
            if index < depth {
                ParagraphControlKind::InlineBoundaryStart
            } else {
                ParagraphControlKind::InlineBoundaryEnd
            }
        );
    }
    let owners = (0..depth)
        .map(|index| {
            let start = capture.captured_control(index).unwrap();
            let end = capture.captured_control(2 * depth - index - 1).unwrap();
            assert!(start.event_position() < capture.captured_origin(0).unwrap());
            assert!(capture.captured_origin(0).unwrap() < end.event_position());
            let declared = prepared
                .native_inline_box(start.clone(), end.clone())
                .unwrap();
            if nominated {
                let scope = capture.captured_scope(index + 1).unwrap();
                assert_eq!(scope.entry_control(), Some(start));
                assert_eq!(scope.exit_control(), Some(end));
                assert_eq!(declared, InlineBoxSource::captured(scope));
            }
            declared
        })
        .collect();
    if !nominated {
        assert!(capture.captured_scope(1).is_none());
    }
    Family {
        capture,
        system,
        prepared,
        controls,
        owners,
    }
}
fn root(f: &Family) -> InlineBoxSource {
    InlineBoxSource::captured(f.capture.captured_scope(0).unwrap())
}
fn context(f: &Family, index: usize, side: LineEdgeSide) -> InlineBoxEdgeContext {
    f.prepared
        .inline_box_edge_context(
            f.owners[index].clone(),
            if index == 0 {
                root(f)
            } else {
                f.owners[index - 1].clone()
            },
            side,
        )
        .unwrap()
}
fn allocation(context: &InlineBoxEdgeContext, advance: f64) -> InlineBoundaryAllocation {
    InlineBoundaryAllocation::box_edge(
        context.clone(),
        crate::InlineBoxEdgeComponents::from_f64(0.0, advance, 0.0).unwrap(),
    )
}
fn request_for(f: &Family, boundaries: Vec<InlineBoundaryAllocation>) -> LineSelectionRequest {
    assert_eq!(boundaries.len(), f.controls.len());
    let segment = f.prepared.source_segments().next().unwrap();
    let offer = f
        .prepared
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let mut request = super::selected_request(&f.prepared, segment.start(), offer);
    request.controls = f
        .controls
        .iter()
        .cloned()
        .zip(boundaries)
        .map(|(occurrence, boundary)| ControlAllocation {
            occurrence,
            strut: None,
            boundary: Some(boundary),
        })
        .collect();
    request
}
fn contributions(f: &Family) -> LineSelectionRequest {
    request_for(
        f,
        (1..=f.controls.len())
            .map(|n| InlineBoundaryAllocation::contribution(n as f64).unwrap())
            .collect(),
    )
}
fn proper_boxes(f: &Family) -> LineSelectionRequest {
    assert_eq!(f.owners.len(), 2);
    let p = context(f, 0, LineEdgeSide::Start);
    let c = context(f, 1, LineEdgeSide::Start);
    request_for(
        f,
        vec![
            allocation(&p, 1.0),
            allocation(&c, 2.0),
            allocation(&c, 3.0),
            allocation(&p, 4.0),
        ],
    )
}
fn finish(f: &Family, request: LineSelectionRequest) -> FinalizedLine {
    f.prepared
        .select(request)
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}
fn denied<T>(result: Result<T>, reason: InlineBoxEdgeError) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid BoxEdge declaration must refuse, never downgrade"),
    };
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InlineBoxEdge(reason)
        })
    );
}
fn plain20(f: &Family, line: &FinalizedLine) {
    // Real Ahem A35 advance10, LTR Source levels, physical starts on line-left:
    // Pstart1,Cstart2,A10,Cend3,Pend4. No inferred CSS direction or equal-edge
    // cancellation is used to identify an owner or preserve the old operation.
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.feedback().advance, 20.0);
    for (index, expected) in [
        (0, (0.0, 1.0)),
        (1, (1.0, 3.0)),
        (2, (13.0, 16.0)),
        (3, (16.0, 20.0)),
    ] {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == f.controls[index])
            .unwrap();
        assert_eq!(slot.strip_interval(), expected);
    }
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(35, 3.0, 10.0)]);
}
fn rejection_preserves(f: &Family, bad: LineSelectionRequest, reason: InlineBoxEdgeError) {
    let contribution = contributions(f);
    let old = finish(f, contribution.clone());
    plain20(f, &old);
    denied(f.prepared.select(bad), reason);
    assert_eq!(old.selected_line().request(), &contribution);
    plain20(f, &old);
    // This is an INDEPENDENT explicitly requested Contribution over the same
    // genuine selected controls, not a retry/downgrade of the failed BoxEdge.
    let legitimate = finish(f, contribution.clone());
    plain20(f, &legitimate);
    let boxes = proper_boxes(f);
    assert_ne!(boxes, contribution);
    let valid_retry = finish(f, boxes.clone());
    assert_eq!(valid_retry.selected_line().request(), &boxes);
    plain20(f, &valid_retry);
    let stats = f.system.stats();
    let replay = old.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &contribution);
    plain20(f, &replay);
    plain20(f, &old);
}

#[test]
fn a_checked_context_from_an_equal_but_foreign_capture_cannot_attach_to_local_endpoints() {
    let f = family(2, true);
    let foreign = family(2, true);
    assert_eq!(f.capture.source(), foreign.capture.source());
    assert_ne!(
        f.capture.captured_scope(1),
        foreign.capture.captured_scope(1)
    );
    let mut bad = proper_boxes(&f);
    bad.controls[0].boundary = Some(allocation(&context(&foreign, 0, LineEdgeSide::Start), 1.0));
    rejection_preserves(&f, bad, InlineBoxEdgeError::ForeignCapture);
}

#[test]
fn a_valid_child_context_cannot_be_applied_to_the_actual_parent_start() {
    let f = family(2, true);
    let mut bad = proper_boxes(&f);
    // Context is valid for C and its actual P parent, but occurrence is Pstart.
    bad.controls[0].boundary = Some(allocation(&context(&f, 1, LineEdgeSide::Start), 1.0));
    rejection_preserves(&f, bad, InlineBoxEdgeError::InvalidOwner);
}

#[test]
fn paired_selected_endpoints_cannot_disagree_about_their_source_start_side() {
    let f = family(2, true);
    let mut bad = proper_boxes(&f);
    bad.controls[2].boundary = Some(allocation(&context(&f, 1, LineEdgeSide::End), 3.0));
    rejection_preserves(&f, bad, InlineBoxEdgeError::InconsistentContext);
}

#[test]
fn context_construction_refuses_foreign_parent_wrong_owner_role_and_skipped_captured_parent() {
    let f = family(3, true);
    let foreign = family(3, true);
    denied(
        f.prepared.inline_box_edge_context(
            f.owners[0].clone(),
            root(&foreign),
            LineEdgeSide::Start,
        ),
        InlineBoxEdgeError::ForeignCapture,
    );
    denied(
        f.prepared
            .inline_box_edge_context(root(&f), root(&f), LineEdgeSide::Start),
        InlineBoxEdgeError::InvalidOwner,
    ); // Domain is a valid parent, never Inline owner.
    denied(
        f.prepared.inline_box_edge_context(
            f.owners[2].clone(),
            f.owners[0].clone(),
            LineEdgeSide::Start,
        ),
        InlineBoxEdgeError::InvalidParent,
    ); // Actual middle captured scope cannot disappear.
    denied(
        f.prepared.inline_box_edge_context(
            f.owners[0].clone(),
            f.owners[0].clone(),
            LineEdgeSide::Start,
        ),
        InlineBoxEdgeError::InvalidParent,
    );
    // The real immediate parent remains a usable declaration after each failure.
    assert_eq!(context(&f, 2, LineEdgeSide::Start).parent(), &f.owners[1]);
}

#[test]
fn declared_native_pairs_require_actual_roles_and_cannot_half_match_scope_nominations() {
    let f = family(2, true);
    denied(
        f.prepared.native_inline_box(
            f.capture.captured_control(0).unwrap(),
            f.capture.captured_control(1).unwrap(),
        ),
        InlineBoxEdgeError::InvalidPair,
    ); // two actual Starts, no invented End.
    denied(
        f.prepared.native_inline_box(
            f.capture.captured_control(2).unwrap(),
            f.capture.captured_control(3).unwrap(),
        ),
        InlineBoxEdgeError::InvalidPair,
    ); // two actual Ends, no invented Start.
    denied(
        f.prepared.native_inline_box(
            f.capture.captured_control(0).unwrap(),
            f.capture.captured_control(2).unwrap(),
        ),
        InlineBoxEdgeError::InvalidPair,
    ); // genuine Pstart cannot steal Cend.
    assert_eq!(
        f.prepared
            .native_inline_box(
                f.capture.captured_control(0).unwrap(),
                f.capture.captured_control(3).unwrap()
            )
            .unwrap(),
        f.owners[0]
    );
}

#[test]
fn individually_ordered_native_pairs_cannot_form_a_crossing_selected_box_family() {
    let f = family(2, false);
    // Source has four REAL un-nominated captured controls S0,S1,E2,E3.
    // The negative declarations below pair0/2 and1/3; no private pair is forged.
    // Each has correct roles/event order, but the two declarations CROSS.
    let a = f
        .prepared
        .native_inline_box(
            f.capture.captured_control(0).unwrap(),
            f.capture.captured_control(2).unwrap(),
        )
        .unwrap();
    let b = f
        .prepared
        .native_inline_box(
            f.capture.captured_control(1).unwrap(),
            f.capture.captured_control(3).unwrap(),
        )
        .unwrap();
    let a = f
        .prepared
        .inline_box_edge_context(a, root(&f), LineEdgeSide::Start)
        .unwrap();
    let b = f
        .prepared
        .inline_box_edge_context(b, root(&f), LineEdgeSide::Start)
        .unwrap();
    let bad = request_for(
        &f,
        vec![
            allocation(&a, 1.0),
            allocation(&b, 2.0),
            allocation(&a, 3.0),
            allocation(&b, 4.0),
        ],
    );
    rejection_preserves(&f, bad, InlineBoxEdgeError::InvalidPair);
}

#[test]
fn a_native_child_cannot_skip_the_actual_containing_pair_declared_by_the_same_request() {
    let f = family(2, false);
    let p = context(&f, 0, LineEdgeSide::Start);
    // With no nominated Inline scope, the isolated constructor can authenticate
    // these references/order. Selection must also inspect the supplied P family.
    let skipped = f
        .prepared
        .inline_box_edge_context(f.owners[1].clone(), root(&f), LineEdgeSide::Start)
        .unwrap();
    let bad = request_for(
        &f,
        vec![
            allocation(&p, 1.0),
            allocation(&skipped, 2.0),
            allocation(&skipped, 3.0),
            allocation(&p, 4.0),
        ],
    );
    rejection_preserves(&f, bad, InlineBoxEdgeError::InvalidParent);
}

#[test]
fn box_edge_zero_still_requires_local_context_while_contribution_zero_remains_independent() {
    let f = family(2, true);
    let foreign = family(2, true);
    let p = context(&f, 0, LineEdgeSide::Start);
    let c = context(&f, 1, LineEdgeSide::Start);
    let boxed = request_for(
        &f,
        vec![
            allocation(&p, 0.0),
            allocation(&c, 0.0),
            allocation(&c, 0.0),
            allocation(&p, 0.0),
        ],
    );
    let contribution = request_for(
        &f,
        (0..4)
            .map(|_| InlineBoundaryAllocation::contribution(0.0).unwrap())
            .collect(),
    );
    assert_ne!(boxed, contribution);
    let old = finish(&f, boxed.clone());
    assert_eq!(old.feedback().advance, 10.0);
    let mut bad = boxed.clone();
    bad.controls[0].boundary = Some(allocation(&context(&foreign, 0, LineEdgeSide::Start), 0.0));
    denied(f.prepared.select(bad), InlineBoxEdgeError::ForeignCapture);
    assert_eq!(old.selected_line().request(), &boxed);
    assert_eq!(old.feedback().advance, 10.0);
    let legitimate = finish(&f, contribution.clone());
    assert_eq!(legitimate.feedback().advance, 10.0);
    assert_eq!(legitimate.selected_line().request(), &contribution);
    let retry = finish(&f, boxed.clone());
    assert_eq!(retry.selected_line().request(), &boxed);
    let replay = old.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line().request(), &boxed);
    assert_eq!(replay.feedback().advance, 10.0);
}

#[test]
fn real_controls_and_inline_owner_from_another_domain_of_the_same_capture_refuse() {
    // Extend the existing real capture INPUT before admission: outer Block/A
    // plus an actual child Block/B, each with its own paragraph and nominated
    // Inline pair. This is not a patched analyzed union or a foreign capture
    // mislabeled WrongDomain. Source ranges0..1 and1..2 remain distinct.
    let mut value = super::captured_input(1);
    super::domain(&mut value, super::style(10.0, 10.0), None);
    super::inline(&mut value, 1, None, None, Some(super::style(10.0, 10.0)));
    super::raw_source(&mut value, "AB");
    value.origins = (0..2)
        .map(|index| ParagraphOrigin {
            source: value.source.clone(),
            text_node: None,
            local: Range::new(index, index + 1),
            aggregate: Range::new(index, index + 1),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 0, end: 1 });
    value.scopes.push(value.scopes[0].clone()); // A REAL child Block, distinct domain.
    let mut child_inline = value.scopes[1].clone();
    let SourceProcessingScope::Inline { boundaries, .. } = &mut child_inline else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 2, end: 3 });
    value.scopes.push(child_inline);
    value.controls = [
        (0, ParagraphControlKind::InlineBoundaryStart),
        (1, ParagraphControlKind::InlineBoundaryEnd),
        (1, ParagraphControlKind::InlineBoundaryStart),
        (2, ParagraphControlKind::InlineBoundaryEnd),
    ]
    .into_iter()
    .map(|(at, kind)| ParagraphControl { at, kind })
    .collect();
    value.paragraphs.push(SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    });
    use SourceProcessingEvent as E;
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Control {
            control: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::EnterScope(2),
        E::EnterParagraph(1),
        E::EnterScope(3),
        E::Control {
            control: 2,
            affinity: Affinity::After,
        },
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::Control {
            control: 3,
            affinity: Affinity::After,
        },
        E::ExitScope(3),
        E::ExitParagraph(1),
        E::ExitScope(2),
        E::ExitScope(0),
    ];
    let (capture, analyzed) = super::analyze(value); // Existing helper chooses REAL raw paragraph0.
    let outer = capture.captured_scope(0).unwrap();
    let child_domain = capture.captured_scope(2).unwrap();
    let child = capture.captured_scope(3).unwrap();
    assert_ne!(outer, child_domain);
    assert_eq!(child.parent(), Some(child_domain.clone()));
    assert_eq!(child.domain(), child_domain);
    assert_eq!(
        capture.captured_control(0).unwrap().paragraph().domain(),
        outer
    );
    assert_eq!(
        capture.captured_control(2).unwrap().paragraph().domain(),
        child_domain
    );
    let mut system = super::system(AHEM, KERN);
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        p.control_views().count(),
        2,
        "the outer raw does not acquire child-domain endpoints"
    );
    denied(
        p.native_inline_box(
            capture.captured_control(2).unwrap(),
            capture.captured_control(3).unwrap(),
        ),
        InlineBoxEdgeError::WrongDomain,
    );
    denied(
        p.inline_box_edge_context(
            InlineBoxSource::captured(child),
            InlineBoxSource::captured(child_domain),
            LineEdgeSide::Start,
        ),
        InlineBoxEdgeError::WrongDomain,
    );
    // The actual local nomination still issues after both refusals.
    assert_eq!(
        p.native_inline_box(
            capture.captured_control(0).unwrap(),
            capture.captured_control(1).unwrap()
        )
        .unwrap(),
        InlineBoxSource::captured(capture.captured_scope(1).unwrap())
    );
}

fn deleted10(f: &Family, line: &FinalizedLine) {
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.feedback().advance, 10.0); // Four REAL endpoint advances1+2+3+4 only.
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    assert_eq!(line.cell_views().len(), 0);
    assert_eq!(line.atomic_slots().count(), 0);
    for (index, expected) in [
        (0, (0.0, 1.0)),
        (1, (1.0, 3.0)),
        (2, (3.0, 6.0)),
        (3, (6.0, 10.0)),
    ] {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == f.controls[index])
            .unwrap();
        assert_eq!(slot.strip_interval(), expected);
    }
}

#[test]
fn real_phase_one_discarded_only_box_uses_its_authenticated_gap_without_a_fake_packet() {
    let f = family_body(2, true, " ", WhiteSpace::Discard);
    assert_eq!(f.capture.source().text(), &Text::from(" "));
    assert!(f.capture.processed_text().is_empty());
    let origins = f.prepared.origin_views().collect::<Vec<_>>();
    let [origin] = origins.as_slice() else {
        panic!("the discarded original occurrence stays retained")
    };
    assert_eq!(origin.source(), f.capture.source());
    assert_eq!(origin.local_range(), Range::new(0, 1));
    assert_eq!(origin.aggregate_range(), Range::new(0, 1));
    assert_eq!(
        origin.capture_position(),
        Some(f.capture.captured_origin(0).unwrap())
    );
    let boxed = proper_boxes(&f);
    let old = finish(&f, boxed.clone());
    deleted10(&f, &old);
    let mut bad = boxed.clone();
    bad.controls[0].boundary = Some(allocation(&context(&f, 1, LineEdgeSide::Start), 1.0));
    denied(f.prepared.select(bad), InlineBoxEdgeError::InvalidOwner);
    assert_eq!(old.selected_line().request(), &boxed);
    deleted10(&f, &old);
    // A separate actual Native contribution request remains independently valid
    // over the exact same source controls and deleted original, without a pair
    // being inferred from geometry or its amount.
    let contribution = contributions(&f);
    let independent = finish(&f, contribution.clone());
    assert_eq!(independent.selected_line().request(), &contribution);
    deleted10(&f, &independent);
    let stats = f.system.stats();
    let replay = old.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &boxed);
    deleted10(&f, &replay);
    deleted10(&f, &old);
}

#[test]
fn plain_native_preparation_cannot_acquire_full_box_authority_from_another_real_capture() {
    // Existing OWNing protocol parent helper, not a flattened/reconstructed
    // capture. Its original Ahem20 A remains an ordinary supported paragraph.
    let mut system = crate::protocol::tests::system();
    let plain = crate::protocol::tests::prepared(&mut system, "A", WhiteSpace::Preserve);
    assert!(plain.captured_owner().is_none());
    let request = crate::protocol::tests::request(&plain);
    let old = plain
        .select(request.clone())
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(old.feedback().advance, 20.0);
    let actual = family(2, true);
    denied(
        plain.native_inline_box(
            actual.capture.captured_control(0).unwrap(),
            actual.capture.captured_control(3).unwrap(),
        ),
        InlineBoxEdgeError::MissingCapture,
    );
    denied(
        plain.inline_box_edge_context(actual.owners[0].clone(), root(&actual), LineEdgeSide::Start),
        InlineBoxEdgeError::MissingCapture,
    );
    assert_eq!(old.selected_line().request(), &request);
    assert_eq!(old.feedback().advance, 20.0);
    let retry = plain
        .select(request.clone())
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(retry.selected_line().request(), &request);
    assert_eq!(retry.feedback().advance, 20.0);
    let replay = old.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(replay.selected_line().request(), &request);
    assert_eq!(replay.feedback().advance, 20.0);
}

// Uncaptured scalar contributions retain their separate owning tests. These
// families distinguish wrong/skipped/self parents, a genuine child domain from
// a foreign capture, and a Phase-I-discarded-only owner's authenticated gap.
// Parent ownership is supplied through real source handles; malformed source,
// arbitrary cuts and unsupported Cell profiles do not stand in for missing
// incidence. The no-alignment family uses Native fallible allocation.

#[path = "parent_geometry_admission/local_constructors.rs"]
mod local_constructors;

#[path = "parent_geometry_admission/parent_only_family.rs"]
mod parent_only_family;

#[path = "parent_geometry_admission/signed_recipient.rs"]
mod signed_recipient;
