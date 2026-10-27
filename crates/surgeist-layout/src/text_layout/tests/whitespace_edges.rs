//! Actual intrinsic consumer observations of source-forced preserved whitespace.
use super::*;

#[path = "whitespace_edges/intrinsic_frame.rs"]
mod intrinsic_frame;

#[path = "whitespace_edges/measurements.rs"]
mod measurements;

fn intrinsic<S: LayoutScalar>(
    text: &str,
    available: AvailableOf<S>,
) -> super::super::run::Output<u32, S> {
    run_in_band(text, available, false)
}

fn run_in_band<S: LayoutScalar>(
    text: &str,
    available: AvailableOf<S>,
    narrow_before_twenty: bool,
) -> super::super::run::Output<u32, S> {
    let paragraph = PreparedParagraph::plain(1, Source::new(text), ahem_style(10.0, 10.0));
    let binding = paragraph.binding(0);
    let size = Size::new(
        S::from_f64(if narrow_before_twenty { 40.0 } else { 20.0 }),
        S::from_f64(10.0),
    );
    super::super::run::layout(
        &binding,
        &Default::default(),
        &[],
        &[],
        super::super::run::Context {
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            frame: super::super::run::Frame::Canonical {
                origin: Point::ZERO,
                size,
            },
            containing_size: size,
            content_insets: crate::Edges::ZERO,
            cursor_block: S::ZERO,
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
        super::super::run::LineEnvironment {
            checkpoint_group: || {},
            reset: |_| {},
            band: |start, _| {
                Ok(super::super::fit::Band {
                    start: 0.0,
                    end: if narrow_before_twenty {
                        if start < 20.0 { 15.0 } else { 40.0 }
                    } else {
                        20.0
                    },
                    intrinsic_end_occupation: None,
                    next_transition: (narrow_before_twenty && start < 20.0).then_some(20.0),
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |_: &surgeist_text::FinalizedLine, _| Ok(Vec::new()),
            accept: || Ok(()),
        },
    )
    .unwrap_or_else(|_| panic!("actual intrinsic paragraph finalizes"))
}

fn qualify<S: LayoutScalar>(
    output: &super::super::run::Output<u32, S>,
    expected: &[u16],
    advances: &[f64],
) {
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
        expected
    );
    assert_eq!(
        output
            .paragraph
            .entries
            .iter()
            .map(|p| p.finalized_line().feedback().advance)
            .collect::<Vec<_>>(),
        advances
    );
    for line in &output.paragraph.entries {
        assert_eq!(
            line.finalized_line()
                .selected_line()
                .request()
                .context
                .available_inline,
            None
        );
        for run in line.finalized_line().run_views() {
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(
                run.face().bytes(),
                include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
            );
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        }
    }
}

fn minimum<S: LayoutScalar>() {
    let ordinary = intrinsic::<S>("XX", AvailableOf::MIN_CONTENT);
    qualify(&ordinary, &[88, 88], &[20.0]);
    assert_eq!(ordinary.report.inline_extent, S::from_f64(20.0));
    let output = intrinsic::<S>("X \nY", AvailableOf::MIN_CONTENT);
    qualify(&output, &[88, 32, 10, 89], &[20.0, 10.0]);
    // The real Forced separator contributes no minimum intrinsic width; this
    // does not change either source-forced line's actual shaped advance.
    assert_eq!(output.report.inline_extent, S::from_f64(10.0));
}

#[test]
fn minimum_content_omits_conditional_space_without_rewriting_raw_lines_f32() {
    minimum::<f32>();
}
#[test]
fn minimum_content_omits_conditional_space_without_rewriting_raw_lines_f64() {
    minimum::<f64>();
}

fn maximum<S: LayoutScalar>() {
    let ordinary = intrinsic::<S>("XX", AvailableOf::MAX_CONTENT);
    qualify(&ordinary, &[88, 88], &[20.0]);
    assert_eq!(
        ordinary.line_bounds[0].allocation.size().width,
        S::from_f64(20.0)
    );
    let output = intrinsic::<S>("X \nY", AvailableOf::MAX_CONTENT);
    qualify(&output, &[88, 32, 10, 89], &[20.0, 10.0]);
    assert_eq!(output.report.inline_extent, S::from_f64(20.0));
    // A genuine maximum request includes the retained conditional SPACE in
    // its actual intrinsic line allocation. Width None alone is not a purpose.
    assert_eq!(
        output.line_bounds[0].allocation.size().width,
        S::from_f64(20.0)
    );
}

#[test]
fn maximum_content_includes_conditional_space_in_actual_line_allocation_f32() {
    maximum::<f32>();
}
#[test]
fn maximum_content_includes_conditional_space_in_actual_line_allocation_f64() {
    maximum::<f64>();
}

fn finite_fit<S: LayoutScalar>() {
    let fitting = run_in_band::<S>("X", AvailableOf::definite(S::from_f64(40.0)), true);
    assert_eq!(fitting.paragraph.entries.len(), 1);
    assert_eq!(
        fitting.paragraph.entries[0]
            .finalized_line()
            .feedback()
            .advance,
        10.0
    );
    assert_eq!(fitting.line_bounds[0].allocation.origin().y, S::ZERO);
    let wider = run_in_band::<S>("XX", AvailableOf::definite(S::from_f64(40.0)), true);
    assert_eq!(wider.paragraph.entries.len(), 1);
    assert_eq!(
        wider.paragraph.entries[0]
            .finalized_line()
            .feedback()
            .advance,
        20.0
    );
    assert_eq!(
        wider.line_bounds[0].allocation.origin().y,
        S::from_f64(20.0)
    );
    let output = run_in_band::<S>("X \nY", AvailableOf::definite(S::from_f64(40.0)), true);
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
        &[88, 32, 10, 89]
    );
    let observed = output
        .paragraph
        .entries
        .iter()
        .zip(&output.line_bounds)
        .map(|(p, bounds)| {
            (
                bounds.allocation.origin().y.to_f64(),
                p.finalized_line().selected_line().request().context.end,
                p.finalized_line()
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_after()
                    .get(),
                p.finalized_line().feedback().advance,
                p.finalized_line().feedback().hanging_end,
            )
        })
        .collect::<Vec<_>>();
    // The owning environment offers a real narrow band with a lower wider
    // band. Raw20 must not move this forced line down: conditional fit is10.
    assert_eq!(
        observed,
        vec![
            (0.0, surgeist_text::LineEnd::Forced, 3, 20.0, 5.0),
            (10.0, surgeist_text::LineEnd::Paragraph, 4, 10.0, 0.0),
        ]
    );
}

#[test]
fn conditional_fit_stays_in_the_narrow_band_before_a_lower_transition_f32() {
    finite_fit::<f32>();
}
#[test]
fn conditional_fit_stays_in_the_narrow_band_before_a_lower_transition_f64() {
    finite_fit::<f64>();
}
