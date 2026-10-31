//! Actual Text metric frames anchor intrinsic lines at the physical block edge.
use super::*;

fn observed<S: LayoutScalar>(
    source: &str,
    whitespace: surgeist_text::WhiteSpace,
    available: AvailableOf<S>,
) -> (f64, f64, f64, f64, f64) {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = whitespace;
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(1, Source::new(source), style.clone())],
        style,
        ParagraphBaseDirection::RightToLeft,
    );
    let binding = paragraph.binding(0);
    let size = Size::new(S::from_f64(100.0), S::from_f64(10.0));
    let tree = paragraph.flat_text_tree::<S>(
        0,
        NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Rtl,
            line_box_direction_policy: crate::LineBoxDirectionPolicy::Normal,
            ..NodeInputOf::default()
        },
    );
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let [row] = flow.associations.runs() else {
        panic!("flat intrinsic fixture has one actual admitted native run");
    };
    let admitted = flow.associations.admitted_run(&flow.paragraphs, row);
    let binding = admitted.binding;
    let output = super::super::super::run::layout(
        &admitted,
        &flow.associations,
        &[],
        &[],
        super::super::super::run::Context {
            scope_alignments: crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
                flow.scope_alignments.clone(),
            ),
            orientation: super::super::super::run::text_orientation(
                WritingMode::HorizontalTb,
                &binding.prepared,
            )
            .unwrap_or_else(|_| panic!("horizontal fixture must resolve its Text orientation")),
            inline_baseline_context: crate::InlineBaselineContext::Physical,
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Rtl,
            ),
            frame: super::super::super::run::Frame::Canonical {
                origin: Point::ZERO,
                size,
            },
            containing_size: size,
            content_insets: crate::Edges::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl),
            available,
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail: crate::text_layout::fit::TailContext::Unrestricted,
        },
        &[],
        super::super::super::run::LineEnvironment {
            checkpoint_group: || {},
            reset: |_| {},
            band: |_, _| {
                Ok(super::super::super::fit::Band {
                    start: 0.0,
                    end: 100.0,
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |_: &surgeist_text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("actual intrinsic paragraph finalizes"));
    assert_eq!(output.paragraph.entries.len(), 1);
    let surgeist_text::PlacementEntry::Formatted(placed) = &output.paragraph.entries[0] else {
        panic!("native intrinsic line remains formatted");
    };
    assert_eq!(
        placed.line.feedback().advance,
        if source == "X" { 10.0 } else { 20.0 }
    );
    assert_eq!(
        placed
            .line
            .selected_line()
            .request()
            .context
            .available_inline,
        None
    );
    assert_eq!(
        output
            .paragraph
            .prepared
            .origin_views()
            .next()
            .unwrap()
            .source()
            .text()
            .code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    let mut glyph_x = None;
    for run in placed.line.run_views() {
        assert_eq!(
            run.face().bytes(),
            include_bytes!("../../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
        );
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        for glyph in run.glyphs() {
            if glyph.id() == 58 {
                assert_eq!(glyph.range(), surgeist_text::Range::new(0, 1));
                assert_eq!(glyph.advance(), 10.0);
                let local = run.origin().x + glyph.pen();
                glyph_x = Some(placed.anchor.inline_origin + local);
            }
        }
    }
    // Actual glyph, allocation origin/width and independent metric origin/width.
    (
        glyph_x.expect("actual X glyph"),
        output.line_bounds[0].allocation.origin().x.to_f64(),
        output.line_bounds[0].allocation.size().width.to_f64(),
        output.line_bounds[0].metrics.origin().x.to_f64(),
        output.line_bounds[0].metrics.size().width.to_f64(),
    )
}

fn intrinsic_frame<S: LayoutScalar>() {
    assert_eq!(
        observed::<S>(
            "X",
            surgeist_text::WhiteSpace::Collapse,
            AvailableOf::MAX_CONTENT
        ),
        (90.0, 90.0, 10.0, 90.0, 10.0)
    );
    // The genuine 10px Text metric frame ends at the block's100px right edge.
    // Raw20 retains the hanging EM SPACE; it is not the anchoring frame width.
    assert_eq!(
        observed::<S>(
            "X\u{2003}",
            surgeist_text::WhiteSpace::Collapse,
            AvailableOf::MAX_CONTENT
        ),
        (90.0, 90.0, 10.0, 90.0, 10.0)
    );
}

#[test]
fn intrinsic_rtl_line_anchors_its_actual_text_metric_frame_f32() {
    intrinsic_frame::<f32>();
}

#[test]
fn intrinsic_rtl_line_anchors_its_actual_text_metric_frame_f64() {
    intrinsic_frame::<f64>();
}

fn separate_allocation<S: LayoutScalar>() {
    assert_eq!(
        observed::<S>(
            "X",
            surgeist_text::WhiteSpace::Preserve,
            AvailableOf::MIN_CONTENT
        ),
        (90.0, 90.0, 10.0, 90.0, 10.0)
    );
    // Min allocation10 ends at100, independently of the actual20px Text frame.
    assert_eq!(
        observed::<S>(
            "X ",
            surgeist_text::WhiteSpace::Preserve,
            AvailableOf::MIN_CONTENT
        ),
        (90.0, 90.0, 10.0, 80.0, 20.0)
    );
}

#[test]
fn intrinsic_rtl_line_anchors_allocation_separately_from_text_metrics_f32() {
    separate_allocation::<f32>();
}

#[test]
fn intrinsic_rtl_line_anchors_allocation_separately_from_text_metrics_f64() {
    separate_allocation::<f64>();
}
