use super::*;

fn recipient_request(
    f: &Family,
    prefix: f64,
    side: LineEdgeSide,
    margin: f64,
) -> LineSelectionRequest {
    let context = f
        .prepared
        .inline_box_edge_context(f.owners[1].clone(), f.owners[0].clone(), side)
        .unwrap();
    request_for(
        f,
        vec![
            InlineBoundaryAllocation::contribution(prefix).unwrap(),
            InlineBoundaryAllocation::box_edge(
                context.clone(),
                InlineBoxEdgeComponents::from_f64(margin, 0.0, 0.0).unwrap(),
            ),
            InlineBoundaryAllocation::box_edge(
                context,
                InlineBoxEdgeComponents::from_f64(0.0, 0.0, 0.0).unwrap(),
            ),
            InlineBoundaryAllocation::CONTRIBUTION_ZERO,
        ],
    )
}

fn recipient_geometry(f: &Family, line: &FinalizedLine, prefix: f64, side: LineEdgeSide) {
    assert_eq!(line.prepared_text(), f.prepared);
    let after_body = prefix + 10.0;
    assert_eq!(line.feedback().advance, after_body);
    let first = line
        .control_slots()
        .find(|slot| slot.control() == f.controls[0])
        .unwrap();
    assert_eq!(first.flow_interval(), (0.0, prefix));
    assert_eq!(first.strip_interval(), (0.0, prefix));
    let start = line
        .control_slots()
        .find(|slot| slot.control() == f.controls[1])
        .unwrap();
    let end = line
        .control_slots()
        .find(|slot| slot.control() == f.controls[2])
        .unwrap();
    let (start_at, end_at) = if side == LineEdgeSide::Start {
        (prefix, after_body)
    } else {
        (after_body, prefix)
    };
    assert_eq!(start.flow_interval(), (start_at, start_at));
    assert_eq!(start.strip_interval(), (start_at, start_at));
    assert_eq!(end.flow_interval(), (end_at, end_at));
    assert_eq!(end.strip_interval(), (end_at, end_at));
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
    assert_eq!(glyphs, [(35, prefix, 10.0)]);
}

#[test]
fn actual_left_and_right_text_margin_recipients_refuse_absorption_without_replacing_prior_output() {
    for (side, prefix) in [
        (LineEdgeSide::Start, 9_007_199_254_740_992.0),
        (LineEdgeSide::End, 9_007_199_254_740_994.0),
    ] {
        let f = family(2, true);
        let valid = recipient_request(&f, prefix, side, 0.0);
        let prior = finish(&f, valid.clone());
        recipient_geometry(&f, &prior, prefix, side);
        let bad = recipient_request(&f, prefix, side, 1.0);
        let selected = f.prepared.select(bad).unwrap();
        let error = selected
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::None,
            })
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(
                error.detail(),
                Some(ErrorDetail::InvalidTextNumeric {
                    phase: TextNumericPhase::Finalization,
                    category: TextNumericError::NonZeroLost,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(prior.selected_line().request(), &valid);
        recipient_geometry(&f, &prior, prefix, side);
        let retry = finish(&f, valid.clone());
        recipient_geometry(&f, &retry, prefix, side);
        let stats = f.system.stats();
        let replay = retry.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(f.system.stats(), stats);
        assert_eq!(replay.selected_line().request(), &valid);
        recipient_geometry(&f, &replay, prefix, side);
        recipient_geometry(&f, &prior, prefix, side);
    }
}
