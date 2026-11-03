//! Genuine CSS numeric boxes separate intrinsic demand from raw overflow.

use super::*;

fn numeric_tree<S: LayoutScalar>(
    available: AvailableOf<S>,
    end_margin: f64,
    first_indent: f64,
) -> PublicLayoutTreeOf<S> {
    let preferred = match available {
        AvailableOf::Definite(value) => PreferredSizeOf::px(value),
        _ => PreferredSizeOf::AUTO,
    };
    let mut host = PublicLayoutTreeOf::<S>::new().children(0, [1, 2]).style(
        0,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(preferred, PreferredSizeOf::AUTO),
            writing_mode: WritingMode::HorizontalTb,
            direction: Direction::Ltr,
            text_align: TextAlign::Start,
            text_wrap_style: TextWrapStyle::Auto,
            text_indent: TextIndentOf {
                amount: LengthPercentageOf::px(S::from_f64(first_indent)).unwrap(),
                each_line: false,
                hanging: false,
            },
            // Observe raw performed overflow without introducing scrollbar
            // reservation, clipping, an extra BFC or a known intrinsic width.
            overflow: ComputedOverflow::try_new(Overflow::Visible, Overflow::Visible).unwrap(),
            ..NodeInputOf::default()
        },
    );
    for node in [1, 2] {
        host = host.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                margin: Edges {
                    right: LengthAutoOf::px(S::from_f64(end_margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                ..NodeInputOf::default()
            },
        );
    }
    host
}

fn numeric_batch<S: LayoutScalar>(
    available: AvailableOf<S>,
    end_margin: f64,
    first_indent: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let host = numeric_tree(available, end_margin, first_indent);
    compute_layout(
        &host,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn assert_actual_boxes<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    expected_root: (f64, f64),
    expected_positions: [(f64, f64); 2],
    end_margin: f64,
    raw_inline_extent: f64,
) {
    // This is numeric participation, not a fake text paragraph or manufactured
    // Source. There is no Text layout/fragment whose geometry could replace the
    // actual box outputs below.
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let parent = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            parent.size,
            Size::new(S::from_f64(expected_root.0), S::from_f64(expected_root.1))
        );
        let first = entries
            .iter()
            .find(|entry| entry.node() == 1)
            .unwrap()
            .output();
        let second = entries
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_ne!(first.source_index, second.source_index);
        for (actual, (x, y)) in [first, second].into_iter().zip(expected_positions) {
            assert_eq!(actual.location, Point::new(S::from_f64(x), S::from_f64(y)));
            assert_eq!(actual.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
            assert_eq!(
                actual.margin,
                Edges {
                    right: S::from_f64(end_margin),
                    ..Edges::ZERO
                }
            );
            assert_eq!(actual.border, Edges::ZERO);
            assert_eq!(actual.padding, Edges::ZERO);
        }
        // content_size is actual raw physical content; it is not a demand that
        // may substitute for CSS intrinsic INLINE sizing. Default visible
        // overflow retains both actual border boxes, including their overhang.
        assert_eq!(parent.content_size.width.to_f64(), raw_inline_extent);
        let geometry = parent
            .scroll_geometry
            .expect("performed numeric Block retains canonical local geometry");
        assert_eq!(geometry.scrollable_overflow().origin(), Point::ZERO);
        assert_eq!(
            geometry.scrollable_overflow().size().width.to_f64(),
            raw_inline_extent
        );
        // Deliberately do not call this a scrollable range assertion: Visible
        // overflow has its own reachability/scroll-container policy.
    }
}

fn numeric_intrinsic_minimum<S: LayoutScalar>() {
    // Two actual used margin boxes each advance10-4=6. The genuine Allowed
    // middle admits two independent pieces: paragraph min=max(6,6)=6.
    // Border geometry remains10 in each line at x0, with row tops0 and10.
    let batch = numeric_batch::<S>(AvailableOf::MIN_CONTENT, -4.0, 0.0);
    assert_actual_boxes(&batch, (6.0, 20.0), [(0.0, 0.0), (0.0, 10.0)], -4.0, 10.0);
}

fn numeric_intrinsic_maximum<S: LayoutScalar>() {
    // Max-content keeps both on one line: 6+6=12. The second border is6..16;
    // actual raw physical overflow is16, not the used intrinsic width12.
    let batch = numeric_batch::<S>(AvailableOf::MAX_CONTENT, -4.0, 0.0);
    assert_actual_boxes(&batch, (12.0, 10.0), [(0.0, 0.0), (6.0, 0.0)], -4.0, 16.0);
}

fn numeric_signed_retraction<S: LayoutScalar>() {
    // Genuine finite band5 and first-line indent10. Each CSS margin advance is
    //10-20=-10, so traversal pens10 ->0 ->-10. There is no NativeOccupied
    // participant: final CSS demand max(0,-10)=0 fits5. Neither the leading
    // prefix10 nor borders10..20 /0..10 establish an admission requirement.
    // The middle is genuinely Allowed, but must not force a spurious second
    // line merely because the first prefix/border extends beyond the band.
    let batch = numeric_batch::<S>(AvailableOf::definite(S::from_f64(5.0)), -20.0, 10.0);
    assert_actual_boxes(&batch, (5.0, 10.0), [(10.0, 0.0), (0.0, 0.0)], -20.0, 20.0);
}

fn numeric_zero_flow<S: LayoutScalar>() {
    // Zero flow is not no participation: two distinct10px border boxes remain
    // at0..10, while each actual CSS margin box advances10-10=0.
    let batch = numeric_batch::<S>(AvailableOf::definite(S::from_f64(5.0)), -10.0, 0.0);
    assert_actual_boxes(&batch, (5.0, 10.0), [(0.0, 0.0), (0.0, 0.0)], -10.0, 10.0);
}

#[test]
fn numeric_root_min_content_uses_six_without_losing_ten_pixel_borders_f32() {
    numeric_intrinsic_minimum::<f32>();
}
#[test]
fn numeric_root_min_content_uses_six_without_losing_ten_pixel_borders_f64() {
    numeric_intrinsic_minimum::<f64>();
}
#[test]
fn numeric_root_max_content_uses_twelve_without_losing_sixteen_pixel_overflow_f32() {
    numeric_intrinsic_maximum::<f32>();
}
#[test]
fn numeric_root_max_content_uses_twelve_without_losing_sixteen_pixel_overflow_f64() {
    numeric_intrinsic_maximum::<f64>();
}
#[test]
fn numeric_css_retraction_after_indent_admits_one_line_in_band_five_f32() {
    numeric_signed_retraction::<f32>();
}
#[test]
fn numeric_css_retraction_after_indent_admits_one_line_in_band_five_f64() {
    numeric_signed_retraction::<f64>();
}
#[test]
fn numeric_css_zero_flow_retains_both_actual_border_boxes_f32() {
    numeric_zero_flow::<f32>();
}
#[test]
fn numeric_css_zero_flow_retains_both_actual_border_boxes_f64() {
    numeric_zero_flow::<f64>();
}
