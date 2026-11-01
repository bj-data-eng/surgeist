use super::*;

// Each row identifies an actual nested source Inline level and whether that
// Inline genuinely nominates its existing Start/End controls. Other levels
// are explicitly declared native pairs within their actual enclosing scope.
fn mixed_family(depth: usize, scopes: &[(usize, bool)]) -> Family {
    assert!(depth > 0);
    assert!(scopes.iter().all(|(level, _)| *level < depth));
    assert!(scopes.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut value = super::super::captured_input(scopes.len());
    super::super::raw_source(&mut value, "A");
    let body = super::super::style(10.0, 10.0);
    super::super::domain(&mut value, body.clone(), None);
    for (ordinal, &(level, nominate)) in scopes.iter().enumerate() {
        super::super::inline(&mut value, ordinal + 1, None, None, Some(body.clone()));
        if nominate {
            let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[ordinal + 1]
            else {
                unreachable!()
            };
            *boundaries = Some(SourceProcessingScopeBoundaries {
                start: level,
                end: 2 * depth - level - 1,
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
    for level in 0..depth {
        if let Some(ordinal) = scopes.iter().position(|&(at, _)| at == level) {
            value
                .events
                .push(SourceProcessingEvent::EnterScope(ordinal + 1));
        }
        value.events.push(SourceProcessingEvent::Control {
            control: level,
            affinity: Affinity::After,
        });
    }
    value.events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    for level in (0..depth).rev() {
        value.events.push(SourceProcessingEvent::Control {
            control: 2 * depth - level - 1,
            affinity: Affinity::After,
        });
        if let Some(ordinal) = scopes.iter().position(|&(at, _)| at == level) {
            value
                .events
                .push(SourceProcessingEvent::ExitScope(ordinal + 1));
        }
    }
    value.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let (capture, analyzed) = super::super::analyze(value);
    assert_eq!(capture.source().text(), &Text::from("A"));
    let mut system = super::super::system(AHEM, KERN);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2 * depth);
    for (index, control) in controls.iter().enumerate() {
        assert_eq!(control.captured_control(), capture.captured_control(index));
    }
    let owners = (0..depth)
        .map(|level| {
            let start = capture.captured_control(level).unwrap();
            let end = capture.captured_control(2 * depth - level - 1).unwrap();
            assert_eq!(start.scope(), end.scope());
            let owner = prepared.native_inline_box(start, end).unwrap();
            if let Some(ordinal) = scopes
                .iter()
                .position(|&(at, nominate)| at == level && nominate)
            {
                assert_eq!(
                    owner,
                    InlineBoxSource::captured(capture.captured_scope(ordinal + 1).unwrap())
                );
            } else {
                assert!(owner.captured_scope().is_none());
            }
            owner
        })
        .collect();
    Family {
        capture,
        system,
        prepared,
        controls,
        owners,
    }
}

#[test]
fn un_nominated_native_endpoints_cannot_cross_their_actual_enclosing_scopes() {
    // S0(root), EnterP, S1(P), A, E2(P), ExitP, E3(root).
    // P has no nominated endpoints: refusal cannot come from half matching one.
    let f = mixed_family(2, &[(1, false)]);
    let p = f.capture.captured_scope(1).unwrap();
    assert!(p.entry_control().is_none() && p.exit_control().is_none());
    assert_eq!(
        f.capture.captured_control(0).unwrap().scope(),
        f.capture.captured_scope(0).unwrap()
    );
    assert_eq!(f.capture.captured_control(1).unwrap().scope(), p.clone());
    assert_eq!(f.capture.captured_control(2).unwrap().scope(), p);
    assert_eq!(
        f.capture.captured_control(3).unwrap().scope(),
        f.capture.captured_scope(0).unwrap()
    );
    let original = contributions(&f);
    let prior = finish(&f, original.clone());
    plain20(&f, &prior);
    for (start, end) in [(0, 2), (1, 3)] {
        denied(
            f.prepared.native_inline_box(
                f.capture.captured_control(start).unwrap(),
                f.capture.captured_control(end).unwrap(),
            ),
            InlineBoxEdgeError::InvalidPair,
        );
        plain20(&f, &prior);
    }
    // The valid root-local outer pair does not acquire P's distinct source
    // scope identity. Inner controls remain explicit ordinary contributions.
    let outer = context(&f, 0, LineEdgeSide::Start);
    let retry_request = request_for(
        &f,
        vec![
            allocation(&outer, 1.0),
            InlineBoundaryAllocation::contribution(2.0).unwrap(),
            InlineBoundaryAllocation::contribution(3.0).unwrap(),
            allocation(&outer, 4.0),
        ],
    );
    let retry = finish(&f, retry_request.clone());
    assert_eq!(retry.selected_line().request(), &retry_request);
    plain20(&f, &retry);
    assert_eq!(prior.selected_line().request(), &original);
    let stats = f.system.stats();
    let replay = retry.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &retry_request);
    plain20(&f, &replay);
    plain20(&f, &prior);
}

#[test]
fn a_root_local_native_pair_can_be_the_parent_of_a_captured_inline_child() {
    // S0(root), EnterC, S1(C), A, E2(C), ExitC, E3(root).
    let f = mixed_family(2, &[(1, true)]);
    assert!(f.owners[0].captured_scope().is_none());
    let child = f.capture.captured_scope(1).unwrap();
    assert_eq!(f.owners[1], InlineBoxSource::captured(child.clone()));
    assert_eq!(child.parent(), f.capture.captured_scope(0));
    let child_context = context(&f, 1, LineEdgeSide::Start);
    assert_eq!(child_context.owner(), &f.owners[1]);
    assert_eq!(child_context.parent(), &f.owners[0]);
    let original = contributions(&f);
    let prior = finish(&f, original.clone());
    plain20(&f, &prior);
    let boxed_request = proper_boxes(&f);
    let boxed = finish(&f, boxed_request.clone());
    assert_eq!(boxed.selected_line().request(), &boxed_request);
    plain20(&f, &boxed);
    let stats = f.system.stats();
    let replay = boxed.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &boxed_request);
    plain20(&f, &replay);
    assert_eq!(prior.selected_line().request(), &original);
    plain20(&f, &prior);
}

fn plain31(f: &Family, line: &FinalizedLine) {
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.feedback().advance, 31.0);
    for (control, bounds) in f.controls.iter().zip([
        (0.0, 1.0),
        (1.0, 3.0),
        (3.0, 6.0),
        (16.0, 20.0),
        (20.0, 25.0),
        (25.0, 31.0),
    ]) {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == *control)
            .unwrap();
        assert_eq!(slot.inline_bounds(), bounds);
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
    // Three real positive Start contributions1+2+3, real Ahem A10,
    // then corresponding End contributions4+5+6. All starts are physical Left.
    assert_eq!(glyphs, [(35, 6.0, 10.0)]);
}

#[test]
fn neither_native_nor_captured_children_can_skip_an_actual_captured_parent() {
    for captured_child in [false, true] {
        let scopes = if captured_child {
            vec![(1, true), (2, true)]
        } else {
            vec![(1, true)]
        };
        let f = mixed_family(3, &scopes);
        assert!(f.owners[0].captured_scope().is_none());
        assert_eq!(
            f.owners[1],
            InlineBoxSource::captured(f.capture.captured_scope(1).unwrap())
        );
        assert_eq!(f.owners[2].captured_scope().is_some(), captured_child);
        let original = contributions(&f);
        let prior = finish(&f, original.clone());
        plain31(&f, &prior);
        for skipped in [root(&f), f.owners[0].clone()] {
            denied(
                f.prepared.inline_box_edge_context(
                    f.owners[2].clone(),
                    skipped,
                    LineEdgeSide::Start,
                ),
                InlineBoxEdgeError::InvalidParent,
            );
            plain31(&f, &prior);
        }
        let outer = context(&f, 0, LineEdgeSide::Start);
        let parent = context(&f, 1, LineEdgeSide::Start);
        let child = context(&f, 2, LineEdgeSide::Start);
        let valid = request_for(
            &f,
            vec![
                allocation(&outer, 1.0),
                allocation(&parent, 2.0),
                allocation(&child, 3.0),
                allocation(&child, 4.0),
                allocation(&parent, 5.0),
                allocation(&outer, 6.0),
            ],
        );
        let retry = finish(&f, valid.clone());
        assert_eq!(retry.selected_line().request(), &valid);
        plain31(&f, &retry);
        let stats = f.system.stats();
        let replay = retry.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(f.system.stats(), stats);
        assert_eq!(replay.selected_line().request(), &valid);
        plain31(&f, &replay);
        assert_eq!(prior.selected_line().request(), &original);
        plain31(&f, &prior);
    }
}
