//! Signed edge demand preserves raw text and allows genuine later flow retraction.
//! Actual captured Layout input retains intrinsic sizing, legal cuts and float-band placement.
use super::*;

fn signed_compute<S: LayoutScalar>(
    fixture: &Fixture,
    start: [f64; 3],
    end: [f64; 3],
    width: Option<f64>,
    available: AvailableOf<S>,
    float: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    // Construct only neutral existing Native inputs first. A negative component
    // must never pass through the fallible Contribution operation as a shortcut.
    let mut tree = fixture.tree(
        InlineBoundaryEdgeOf::ZERO,
        InlineBoundaryEdgeOf::ZERO,
        false,
        width,
        float,
    );
    for (node, parts) in [(20, start), (21, end)] {
        let old = tree.layout_input(node).as_inline_boundary().unwrap();
        let edge = InlineBoundaryEdgeOf::try_new(
            S::from_f64(parts[0]),
            S::from_f64(parts[1]),
            S::from_f64(parts[2]),
        )
        .unwrap();
        let input = InlineBoundaryInputOf::box_edge(
            old.kind(),
            old.metrics(),
            edge,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        )
        .with_writing_mode(old.writing_mode())
        .with_direction(old.direction())
        .with_vertical_align(old.vertical_align());
        tree = tree.input(node, LayoutInputOf::inline_boundary(input));
    }
    let scope = fixture
        .binding
        .controls
        .iter()
        .find(|r| r.node == 20)
        .unwrap()
        .occurrence
        .captured_control()
        .unwrap()
        .scope();
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        WrappingInput {
            gaps: Vec::new(),
            scopes: vec![WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            }],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &flow,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}
fn demand(line: &text::FinalizedLine, expected: f64) {
    let fit = line.measurements().margin_edge_fit();
    assert_eq!(fit.authority(), text::MarginEdgeFitAuthority::CssComposed);
    assert_eq!(fit.advance(), expected);
    assert_eq!(fit.required_inline_size(), expected);
}

fn reduced<S: LayoutScalar>() {
    let fixture = Fixture::new("A", (0, 1), text::WhiteSpace::Collapse, false, false);
    // The ONE-A finite line alone is not proof against unbreakable overflow
    // fallback. Its canonical demand/raw observation and separate intrinsic
    // sizes are the meaningful independent obligations here.
    for (width, available) in [
        (Some(5.0), AvailableOf::definite(S::from_f64(5.0))),
        (None, AvailableOf::MAX_CONTENT),
        (None, AvailableOf::MIN_CONTENT),
    ] {
        let batch = signed_compute::<S>(
            &fixture,
            [0.0; 3],
            [-5.0, 0.0, 0.0],
            width,
            available,
            false,
        );
        let lines = finalized_lines(&batch);
        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        qualify(line);
        assert_eq!(line.feedback().advance, 5.0);
        assert_eq!(
            (line.feedback().occupied_start, line.feedback().occupied_end),
            (0.0, 10.0)
        );
        demand(line, 5.0);
        assert_eq!(
            line.measurements()
                .margin_edge_min_content_contribution()
                .required_inline_size(),
            5.0
        );
        assert_eq!(
            line.measurements()
                .margin_edge_max_content_contribution()
                .required_inline_size(),
            5.0
        );
        assert_eq!(
            line.run_views()
                .flat_map(|run| {
                    let x = run.origin().x;
                    run.glyphs()
                        .map(move |g| (g.id(), g.advance(), x + g.pen() + f64::from(g.x())))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            [(35, 10.0, 0.0)]
        );
        let end = fixture
            .binding
            .controls
            .iter()
            .find(|r| r.node == 21)
            .unwrap()
            .occurrence
            .clone();
        let slot = line.control_slots().find(|r| r.control() == end).unwrap();
        assert_eq!(slot.flow_interval(), (10.0, 5.0));
        assert_eq!(slot.strip_interval(), (10.0, 10.0));
        assert_eq!(output(&batch, 0).size.width, S::from_f64(5.0));
        assert_eq!(output(&batch, 21).location.x, S::from_f64(10.0));
        assert_eq!(output(&batch, 21).size.width, S::ZERO);
        assert_eq!(output(&batch, 21).margin.right, S::from_f64(-5.0));
    }
}
#[test]
fn actual_end_margin_demands_five_without_shrinking_ahem_text_or_reinflating_intrinsics() {
    reduced::<f32>();
    reduced::<f64>();
}

fn retraction<S: LayoutScalar>() {
    let fixture = Fixture::at_soft_seam(); // real A / OptionalBreak / <Inline>B</Inline>
    assert!(
        fixture
            .binding
            .prepared
            .opportunity_views()
            .any(|offer| offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 1)
    );
    let old = signed_compute::<S>(
        &fixture,
        [0.0; 3],
        [0.0; 3],
        Some(10.0),
        AvailableOf::definite(S::from_f64(10.0)),
        false,
    );
    let old_lines = finalized_lines(&old);
    assert_eq!(old_lines.len(), 2);
    assert!(old_lines.iter().all(|line| line.feedback().advance == 10.0));
    // Genuine earlier A10 opportunity FITS this10px band. A wrong raw20/prefix
    // maximum check would choose it; the later included end margin retracts the
    // WHOLE actual AB candidate to5, without borrowing an offselected edge.
    let batch = signed_compute::<S>(
        &fixture,
        [0.0; 3],
        [-15.0, 0.0, 0.0],
        Some(10.0),
        AvailableOf::definite(S::from_f64(10.0)),
        false,
    );
    let lines = finalized_lines(&batch);
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    demand(line, 5.0);
    assert_eq!(line.feedback().advance, 5.0);
    assert_eq!(
        (line.feedback().occupied_start, line.feedback().occupied_end),
        (0.0, 20.0)
    );
    assert!(
        matches!(&line.selected_line().request().termination,text::PreparedTermination::Ordinary(offer)
        if offer.view().kind()==text::PreparedBreakKind::ParagraphEnd)
    );
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                let x = run.origin().x;
                run.glyphs()
                    .map(move |g| (g.id(), x + g.pen() + f64::from(g.x()), g.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [(35, 0.0, 10.0), (36, 10.0, 10.0)]
    );
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    demand(&replay, 5.0);
    assert_eq!(
        replay.selected_line().request(),
        line.selected_line().request()
    );
    assert_eq!(old_lines.len(), 2);
    assert!(old_lines.iter().all(|line| line.feedback().advance == 10.0));
}
#[test]
fn later_selected_margin_retraction_keeps_the_whole_candidate_over_a_real_fitting_soft_prefix() {
    retraction::<f32>();
    retraction::<f64>();
}

fn beside_float<S: LayoutScalar>() {
    let fixture = Fixture::new("A", (0, 1), text::WhiteSpace::Collapse, false, false);
    // Maintained real float is25x20. At root35 its10px remaining band admits
    // ordinary A10 beside it; this independently qualifies the actual band.
    let wide = signed_compute::<S>(
        &fixture,
        [0.0; 3],
        [0.0; 3],
        Some(35.0),
        AvailableOf::definite(S::from_f64(35.0)),
        true,
    );
    let positions = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch.placed_text()[0]
            .layout()
            .run_views()
            .flat_map(|run| {
                let point = run.origin();
                run.glyphs()
                    .map(move |g| {
                        (
                            g.id(),
                            point.x + g.pen() + f64::from(g.x()),
                            point.y + f64::from(g.y()),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(positions(&wide), [(35, 25.0, 8.0)]);
    let batch = signed_compute::<S>(
        &fixture,
        [-5.0, 0.0, 0.0],
        [0.0; 3],
        Some(30.0),
        AvailableOf::definite(S::from_f64(30.0)),
        true,
    );
    let lines = finalized_lines(&batch);
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].selected_line().request().context.available_inline,
        Some(5.0)
    );
    assert_eq!(
        (
            lines[0].feedback().baseline,
            lines[0].feedback().line_height
        ),
        (8.0, 10.0)
    );
    demand(&lines[0], 5.0);
    assert_eq!(
        positions(&batch),
        [(35, 20.0, 8.0)],
        "real margin overhang overlaps the float by5; do not move below its20px height or re-zero text"
    );
    let actual_float = output(&batch, 2);
    assert_eq!(actual_float.location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(
        actual_float.size,
        Size::new(S::from_f64(25.0), S::from_f64(20.0))
    );
    assert_eq!(positions(&wide), [(35, 25.0, 8.0)]);
}
#[test]
fn actual_negative_start_margin_can_overhang_the_real_float_band_without_false_height_retry() {
    beside_float::<f32>();
    beside_float::<f64>();
}
