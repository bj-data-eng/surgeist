//! Complete topology does not widen an exact output into a complete bidi body.
use super::*;

fn connected_body() -> (Arc<ProcessedSource>, AnalyzedSource) {
    let mut value = captured_input(1);
    raw_source(&mut value, "AB");
    domain(&mut value, style(10.0, 10.0), None);
    inline(&mut value, 1, None, None, Some(style(10.0, 10.0)));
    assert!(matches!(
        &value.scopes[0],
        SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Complete,
            ..
        }
    ));
    let SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 0, end: 1 });
    value.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            source: value.source.clone(),
            text_node: None,
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    value.paragraphs = vec![value.paragraphs[0]; 2];
    value.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
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
    // This is genuine joint authority, separate from the Continuous seam. The
    // exact constructor below still owns ONLY its requested first raw member.
    value.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![1, 0],
        facts_owner: 1,
    }];
    use SourceProcessingEvent as E;
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(1),
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
        E::Connection(0),
        E::ExitParagraph(1),
        E::EnterParagraph(0),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    // The maintained helper processes this real input and analyzes the first
    // event-ordered raw view, then binds its missing ordinary wrap policies.
    let (capture, exact) = analyze(value);
    assert_eq!(capture.source().text(), &Text::from("AB"));
    assert_eq!(capture.processed_text(), &Text::from("AB"));
    let raw = capture.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    for (view, id, range) in [
        (&raw[0], 1, Range::new(0, 1)),
        (&raw[1], 0, Range::new(1, 2)),
    ] {
        assert_eq!(
            view.captured_paragraph(),
            capture.captured_paragraph(id).unwrap()
        );
        assert_eq!(view.source_pieces().collect::<Vec<_>>(), [range]);
        assert_eq!(
            view.captured_paragraph().domain(),
            capture.captured_scope(0).unwrap()
        );
    }
    assert!(capture.captured_scope(2).is_none());
    assert_eq!(capture.captured_scope(0).unwrap().role(), ScopeRole::Domain);
    let child = capture.captured_scope(1).unwrap();
    assert_eq!(child.role(), ScopeRole::Inline);
    assert_eq!(child.parent(), capture.captured_scope(0));
    assert_eq!(child.entry_control(), capture.captured_control(0));
    assert_eq!(child.exit_control(), capture.captured_control(1));
    let owner = exact.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [capture.captured_paragraph(1).unwrap()]
    );
    assert_eq!(owner.facts_owner(), capture.captured_paragraph(1).unwrap());
    assert!(owner.joint_flow().is_none());
    assert!(capture.captured_joint_flow(0).is_some());
    (capture, exact)
}

fn boxed_body_request(p: &PreparedText, capture: &ProcessedSource) -> LineSelectionRequest {
    assert_eq!(p.source_segments().len(), 1);
    let segment = p.source_segments().next().unwrap();
    let mut request = selected_request(p, segment.start(), segment.end_opportunity());
    assert!(request.inline_boxes.is_none());
    let context = p
        .inline_box_edge_context(
            InlineBoxSource::captured(capture.captured_scope(1).unwrap()),
            InlineBoxSource::captured(capture.captured_scope(0).unwrap()),
            LineEdgeSide::Start,
        )
        .unwrap();
    let controls = p
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    for (index, control) in controls.iter().enumerate() {
        assert_eq!(control.captured_control(), capture.captured_control(index));
        assert_eq!(
            control.captured_control().unwrap().paragraph(),
            capture.captured_paragraph(1).unwrap()
        );
    }
    request.controls = controls
        .into_iter()
        .zip([2.0, 3.0])
        .map(|(occurrence, width)| ControlAllocation {
            occurrence,
            strut: None,
            boundary: Some(InlineBoundaryAllocation::box_edge(
                context.clone(),
                InlineBoxEdgeComponents::from_f64(0.0, width, 0.0).unwrap(),
            )),
        })
        .collect();
    request
}

fn finish_body(p: &PreparedText, request: LineSelectionRequest) -> FinalizedLine {
    p.select(request)
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn body_output(p: &PreparedText, line: &FinalizedLine, whole: bool) {
    assert_eq!(line.prepared_text(), *p);
    assert_eq!(line.feedback().advance, if whole { 25.0 } else { 15.0 });
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
    assert_eq!(
        glyphs,
        if whole {
            vec![(35, 2.0, 10.0), (36, 15.0, 10.0)]
        } else {
            vec![(35, 2.0, 10.0)]
        }
    );
    let controls = line.control_slots().collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    assert_eq!(controls[0].strip_interval(), (0.0, 2.0));
    assert_eq!(controls[1].strip_interval(), (12.0, 15.0));
}

#[test]
fn complete_topology_exact_member_retains_local_edges_but_requires_complete_body_ownership() {
    let (capture, exact) = connected_body();
    let domain = AnalyzedDomain::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let whole = domain.outputs().next().unwrap();
    let owner = whole.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [
            capture.captured_paragraph(1).unwrap(),
            capture.captured_paragraph(0).unwrap(),
        ]
    );
    assert_eq!(owner.facts_owner(), capture.captured_paragraph(1).unwrap());
    assert_eq!(owner.joint_flow(), capture.captured_joint_flow(0));
    let mut system = system(AHEM, KERN);
    let local = system
        .prepare(&exact, PreparationOptions::default())
        .unwrap();
    let complete = system
        .prepare(
            &crate::test_wrapping::bind_missing(whole, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    assert_eq!(
        local
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [capture.captured_paragraph(1).unwrap()]
    );
    assert!(local.captured_owner().unwrap().joint_flow().is_none());
    assert_eq!(
        complete
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [
            capture.captured_paragraph(1).unwrap(),
            capture.captured_paragraph(0).unwrap()
        ]
    );
    assert_eq!(
        local
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 0)]
    );
    assert_eq!(
        complete
            .unit_views()
            .map(|unit| (unit.source_range(), unit.level().get()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 0), (Range::new(1, 2), 0)]
    );
    assert_eq!(
        local
            .origin_views()
            .map(|view| view.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    assert_eq!(
        complete
            .origin_views()
            .map(|view| view.aggregate_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let bounded = boxed_body_request(&local, &capture);
    let prior = finish_body(&local, bounded.clone());
    assert!(prior.inline_box_family().is_none());
    body_output(&local, &prior, false);
    let error = local.inline_box_family(Vec::new(), None).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InlineBoxFragmentation(
                InlineBoxFragmentationError::IncompleteBodyCoverage {
                    section: 0,
                    paragraph: 0,
                }
            ),
        })
    );
    // The same capture has real complete output authority; neither topology,
    // continuity nor a declared joint silently enlarges the exact preparation.
    let family = complete.inline_box_family(Vec::new(), None).unwrap();
    assert_eq!(family.prepared_text(), complete);
    let mut global = boxed_body_request(&complete, &capture);
    global.inline_boxes = Some(family.clone());
    let finalized = finish_body(&complete, global);
    assert_eq!(finalized.inline_box_family(), Some(&family));
    assert!(finalized.inline_box_pieces().is_some());
    body_output(&complete, &finalized, true);
    assert_eq!(prior.selected_line().request(), &bounded);
    body_output(&local, &prior, false);
    let retry = finish_body(&local, bounded.clone());
    body_output(&local, &retry, false);
    let stats = system.stats();
    let replay = prior.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(system.stats(), stats);
    assert_eq!(replay.selected_line().request(), &bounded);
    body_output(&local, &replay, false);
    body_output(&local, &prior, false);
}
