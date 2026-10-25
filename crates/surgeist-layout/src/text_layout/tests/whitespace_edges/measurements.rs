//! Purpose-specific allocation and independent actual Text metric geometry.
use super::*;

fn actual_geometry<S: LayoutScalar>() {
    let output = intrinsic::<S>("X \nY", AvailableOf::MIN_CONTENT);
    assert_eq!(output.report.inline_extent, S::from_f64(10.0));
    assert_eq!(
        output.line_bounds[0].allocation.size().width,
        S::from_f64(10.0)
    );
    assert_eq!(
        output.line_bounds[0].metrics.size().width,
        S::from_f64(20.0)
    );
    let first = &output.paragraph.lines[0].line;
    assert_eq!(
        first.metric_bounds(),
        surgeist_text::Rect::new(0.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(
        (
            first.measurements().min_content_contribution().advance(),
            first.measurements().max_content_contribution().advance()
        ),
        (10.0, 20.0)
    );
    let original = output
        .paragraph
        .prepared
        .origin_views()
        .next()
        .unwrap()
        .handle();
    let layout = surgeist_text::Layout::from_finalized_lines(surgeist_text::PlacementRequest {
        prepared: output.paragraph.prepared,
        coverage: surgeist_text::PlacementCoverage::WholeParagraph,
        lines: output.paragraph.lines,
    })
    .unwrap();
    assert_eq!(layout.source().text().code_units(), &[88, 32, 10, 89]);
    assert_eq!(layout.lines()[0].bounds().size.width, 20.0);
    let rects = layout
        .source_range_fragments(&original, surgeist_text::Range::new(1, 2))
        .unwrap();
    let units: Vec<_> = rects.units().collect();
    assert_eq!(units.len(), 1);
    let pieces: Vec<_> = units[0].pieces().collect();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].advance_edges(), (10.0, 20.0));
    assert_eq!(pieces[0].rect().size.width, 10.0);
    assert_eq!(
        layout
            .cursor(surgeist_text::Cursor::new(
                2,
                surgeist_text::Affinity::Before
            ))
            .unwrap()
            .rect()
            .origin
            .x,
        20.0
    );
}

#[test]
fn intrinsic_allocation_is_not_the_actual_text_metric_or_source_geometry_f32() {
    actual_geometry::<f32>();
}
#[test]
fn intrinsic_allocation_is_not_the_actual_text_metric_or_source_geometry_f64() {
    actual_geometry::<f64>();
}

#[test]
fn block_end_claim_cannot_hide_known_later_in_flow_content() {
    let paragraph = PreparedParagraph::plain(1, Source::new("X"), ahem_style(10.0, 10.0));
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    let error = match LayoutFlowInput::try_new(&tree, 0, vec![paragraph.binding(0)], Vec::new()) {
        Err(error) => error,
        Ok(_) => panic!("known later block contradicts BlockEnd"),
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::TerminalBlockEdge
        ))
    ));
}
