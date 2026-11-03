use super::*;

// Each event tape has real C(A) and D(B) siblings, without nominated Inline
// scopes. Every endpoint belongs to the same actual captured domain scope.
fn native_family(crossing_parents: bool) -> Family {
    let mut value = super::super::captured_input(0);
    super::super::raw_source(&mut value, "AB");
    super::super::domain(&mut value, super::super::style(10.0, 10.0), None);
    value.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: value.source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    value.boundary_policies = vec![ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    let (controls, before_a, before_b, pairs) = if crossing_parents {
        // Q starts before C(A); R starts after C and before Q ends;
        // D(B) then lies in R's exclusive right-hand portion.
        (
            [
                (0, ParagraphControlKind::InlineBoundaryStart), // Q
                (0, ParagraphControlKind::InlineBoundaryStart), // C
                (1, ParagraphControlKind::InlineBoundaryEnd),   // C
                (1, ParagraphControlKind::InlineBoundaryStart), // R
                (1, ParagraphControlKind::InlineBoundaryEnd),   // Q
                (1, ParagraphControlKind::InlineBoundaryStart), // D
                (2, ParagraphControlKind::InlineBoundaryEnd),   // D
                (2, ParagraphControlKind::InlineBoundaryEnd),   // R
            ],
            2,
            6,
            [(0, 4), (3, 7), (1, 2), (5, 6)],
        )
    } else {
        // P contains Q; Q contains C(A) followed by D(B).
        (
            [
                (0, ParagraphControlKind::InlineBoundaryStart), // P
                (0, ParagraphControlKind::InlineBoundaryStart), // Q
                (0, ParagraphControlKind::InlineBoundaryStart), // C
                (1, ParagraphControlKind::InlineBoundaryEnd),   // C
                (1, ParagraphControlKind::InlineBoundaryStart), // D
                (2, ParagraphControlKind::InlineBoundaryEnd),   // D
                (2, ParagraphControlKind::InlineBoundaryEnd),   // Q
                (2, ParagraphControlKind::InlineBoundaryEnd),   // P
            ],
            3,
            5,
            [(0, 7), (1, 6), (2, 3), (4, 5)],
        )
    };
    value.controls = controls
        .into_iter()
        .map(|(at, kind)| ParagraphControl { at, kind })
        .collect();
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for control in 0..before_a {
        value.events.push(SourceProcessingEvent::Control {
            control,
            affinity: Affinity::After,
        });
    }
    value.events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    for control in before_a..before_b {
        value.events.push(SourceProcessingEvent::Control {
            control,
            affinity: Affinity::After,
        });
    }
    value.events.push(SourceProcessingEvent::Connection(0));
    value.events.push(SourceProcessingEvent::Origin {
        origin: 1,
        affinity: Affinity::After,
    });
    for control in before_b..8 {
        value.events.push(SourceProcessingEvent::Control {
            control,
            affinity: Affinity::After,
        });
    }
    value.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let (capture, analyzed) = super::super::analyze(value);
    assert_eq!(capture.source().text(), &Text::from("AB"));
    assert_eq!(capture.processed_text(), &Text::from("AB"));
    assert!(capture.captured_scope(1).is_none());
    let mut system = super::super::system(AHEM, KERN);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 8);
    let domain = capture.captured_scope(0).unwrap();
    for (index, control) in controls.iter().enumerate() {
        let captured = capture.captured_control(index).unwrap();
        assert_eq!(control.captured_control(), Some(captured.clone()));
        assert_eq!(captured.scope(), domain);
    }
    let a = capture.captured_origin(0).unwrap();
    let b = capture.captured_origin(1).unwrap();
    let (c_start, c_end) = pairs[2];
    let (d_start, d_end) = pairs[3];
    assert!(capture.captured_control(c_start).unwrap().event_position() < a);
    assert!(a < capture.captured_control(c_end).unwrap().event_position());
    assert!(
        capture.captured_control(c_end).unwrap().event_position()
            < capture.captured_control(d_start).unwrap().event_position()
    );
    assert!(capture.captured_control(d_start).unwrap().event_position() < b);
    assert!(b < capture.captured_control(d_end).unwrap().event_position());
    let owners = pairs
        .into_iter()
        .map(|(start, end)| {
            let source = prepared
                .native_inline_box(
                    capture.captured_control(start).unwrap(),
                    capture.captured_control(end).unwrap(),
                )
                .unwrap();
            assert!(source.captured_scope().is_none());
            source
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

fn sibling_request(f: &Family, skip_q_for_c: bool) -> LineSelectionRequest {
    let p = f
        .prepared
        .inline_box_edge_context(f.owners[0].clone(), root(f), LineEdgeSide::Start)
        .unwrap();
    let c = f
        .prepared
        .inline_box_edge_context(
            f.owners[2].clone(),
            f.owners[if skip_q_for_c { 0 } else { 1 }].clone(),
            LineEdgeSide::Start,
        )
        .unwrap();
    let d = f
        .prepared
        .inline_box_edge_context(
            f.owners[3].clone(),
            f.owners[1].clone(),
            LineEdgeSide::Start,
        )
        .unwrap();
    assert_eq!(d.owner(), &f.owners[3]);
    assert_eq!(d.parent(), &f.owners[1]);
    let request = request_for(
        f,
        vec![
            allocation(&p, 1.0),
            InlineBoundaryAllocation::contribution(2.0).unwrap(),
            allocation(&c, 3.0),
            allocation(&c, 4.0),
            allocation(&d, 5.0),
            allocation(&d, 6.0),
            InlineBoundaryAllocation::contribution(7.0).unwrap(),
            allocation(&p, 8.0),
        ],
    );
    // Q is known through D.parent, without claiming BoxEdge ownership for Q's
    // two independently valid endpoint Contributions.
    assert_eq!(
        request.controls[1].boundary,
        Some(InlineBoundaryAllocation::contribution(2.0).unwrap())
    );
    assert_eq!(
        request.controls[6].boundary,
        Some(InlineBoundaryAllocation::contribution(7.0).unwrap())
    );
    request
}

fn sibling_geometry(f: &Family, line: &FinalizedLine) {
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.feedback().advance, 56.0);
    for (control, bounds) in f.controls.iter().zip([
        (0.0, 1.0),
        (1.0, 3.0),
        (3.0, 6.0),
        (16.0, 20.0),
        (20.0, 25.0),
        (35.0, 41.0),
        (41.0, 48.0),
        (48.0, 56.0),
    ]) {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == *control)
            .unwrap();
        assert_eq!(slot.strip_interval(), bounds);
    }
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // Ahem A and B each advance10. Positive endpoint contributions total36.
    // C's A starts after1+2+3; D's B follows A10, Cend4 and Dstart5.
    assert_eq!(
        glyphs,
        [
            (Range::new(0, 1), 35, 6.0, 10.0),
            (Range::new(1, 2), 36, 25.0, 10.0),
        ]
    );
}

#[test]
fn a_parent_known_only_through_a_sibling_declaration_cannot_be_skipped() {
    let f = native_family(false);
    let valid = sibling_request(&f, false);
    let prior = finish(&f, valid.clone());
    assert_eq!(prior.selected_line().request(), &valid);
    sibling_geometry(&f, &prior);
    let bad = sibling_request(&f, true);
    assert_ne!(bad, valid);
    // Only C's declared parent changes. C's two endpoint contexts agree with
    // one another, and Q remains explicitly known through the unchanged D.
    let mut restored = bad.clone();
    restored.controls[2] = valid.controls[2].clone();
    restored.controls[3] = valid.controls[3].clone();
    assert_eq!(restored, valid);
    denied(f.prepared.select(bad), InlineBoxEdgeError::InvalidParent);
    assert_eq!(prior.selected_line().request(), &valid);
    sibling_geometry(&f, &prior);
    let retry = finish(&f, valid.clone());
    assert_eq!(retry.selected_line().request(), &valid);
    sibling_geometry(&f, &retry);
    let stats = f.system.stats();
    let replay = retry.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &valid);
    sibling_geometry(&f, &replay);
    sibling_geometry(&f, &prior);
}

fn crossing_request(f: &Family, include_r: bool) -> LineSelectionRequest {
    let c = f
        .prepared
        .inline_box_edge_context(
            f.owners[2].clone(),
            f.owners[0].clone(),
            LineEdgeSide::Start,
        )
        .unwrap();
    let d = f
        .prepared
        .inline_box_edge_context(
            f.owners[3].clone(),
            if include_r {
                f.owners[1].clone()
            } else {
                root(f)
            },
            LineEdgeSide::Start,
        )
        .unwrap();
    let request = request_for(
        f,
        vec![
            InlineBoundaryAllocation::contribution(1.0).unwrap(),
            allocation(&c, 2.0),
            allocation(&c, 3.0),
            InlineBoundaryAllocation::contribution(4.0).unwrap(),
            InlineBoundaryAllocation::contribution(5.0).unwrap(),
            allocation(&d, 6.0),
            allocation(&d, 7.0),
            InlineBoundaryAllocation::contribution(8.0).unwrap(),
        ],
    );
    // Neither parent acquires a BoxEdge operation on its own endpoints.
    for (index, amount) in [(0, 1.0), (3, 4.0), (4, 5.0), (7, 8.0)] {
        assert_eq!(
            request.controls[index].boundary,
            Some(InlineBoundaryAllocation::contribution(amount).unwrap())
        );
    }
    request
}

fn crossing_geometry(f: &Family, line: &FinalizedLine) {
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.feedback().advance, 56.0);
    for (control, bounds) in f.controls.iter().zip([
        (0.0, 1.0),
        (1.0, 3.0),
        (13.0, 16.0),
        (16.0, 20.0),
        (20.0, 25.0),
        (25.0, 31.0),
        (41.0, 48.0),
        (48.0, 56.0),
    ]) {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == *control)
            .unwrap();
        assert_eq!(slot.strip_interval(), bounds);
    }
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        run.origin().x + glyph.pen(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // C's A follows1+2; D's B follows A10 plus3+4+5+6.
    assert_eq!(
        glyphs,
        [
            (Range::new(0, 1), 35, 3.0, 10.0),
            (Range::new(1, 2), 36, 31.0, 10.0),
        ]
    );
}

#[test]
fn disjoint_child_owners_cannot_introduce_crossing_parent_only_pairs() {
    let f = native_family(true);
    // Qstart < Cstart < A < Cend < Rstart < Qend < Dstart < B < Dend < Rend.
    // C and D are disjoint; their separately authenticated parents Q/R cross.
    // R's unreferenced pair does not promote its Contribution endpoints when
    // the valid request names only Q and the actual domain as child parents.
    let valid = crossing_request(&f, false);
    let prior = finish(&f, valid.clone());
    crossing_geometry(&f, &prior);
    let bad = crossing_request(&f, true);
    let mut restored = bad.clone();
    restored.controls[5] = valid.controls[5].clone();
    restored.controls[6] = valid.controls[6].clone();
    assert_eq!(restored, valid);
    denied(f.prepared.select(bad), InlineBoxEdgeError::InvalidPair);
    assert_eq!(prior.selected_line().request(), &valid);
    crossing_geometry(&f, &prior);
    let retry = finish(&f, valid.clone());
    assert_eq!(retry.selected_line().request(), &valid);
    crossing_geometry(&f, &retry);
    let stats = f.system.stats();
    let replay = retry.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &valid);
    crossing_geometry(&f, &replay);
    crossing_geometry(&f, &prior);
}
