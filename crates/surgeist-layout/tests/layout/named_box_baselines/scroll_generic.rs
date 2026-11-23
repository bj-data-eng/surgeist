use super::*;
use surgeist_text::font::FontBaseline;

fn last_physical_scroll_baseline<S: l::scalar::LayoutScalar>() {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let clip = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Clip,
        l::overflow::Overflow::Clip,
    )
    .unwrap();
    let hidden = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Hidden,
        l::overflow::Overflow::Hidden,
    )
    .unwrap();

    // The peer's synthesized baseline is 60. The target's last baseline
    // lies at 10 + 25 + 20 before clamping to its 40px border extent.
    for (overflow, target_y) in [(clip, 5.0), (hidden, 20.0)] {
        let mut flex = root(
            mode,
            l::geometry::Direction::Ltr,
            l::baseline::InlineBaselineContext::Physical,
        );
        flex.display = l::box_model::Display::Flex;
        flex.align_items = Some(l::box_model::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::node::LayoutInputOf::box_input(source_block(name, Some(60.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = overflow;
        target.baseline_source = l::baseline::BaselineSource::Last;
        assert_eq!(
            host.push(Some(0), l::node::LayoutInputOf::box_input(target)),
            2
        );
        source_content(&mut host, name, 25.0, 20.0);
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, 6).location.y, n(35.0));
        assert_eq!(node(phase, 2).location.y, n(target_y), "{overflow:?}");
    }
}

fn first_physical_scroll_baseline<S: l::scalar::LayoutScalar>() {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let clip = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Clip,
        l::overflow::Overflow::Clip,
    )
    .unwrap();
    let hidden = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Hidden,
        l::overflow::Overflow::Hidden,
    )
    .unwrap();

    // The 10px peer aligns against -16 + 10 before the scroll container
    // clamps its baseline to the start border edge.
    for (overflow, target_y) in [(clip, 16.0), (hidden, 10.0)] {
        let mut flex = root(
            mode,
            l::geometry::Direction::Ltr,
            l::baseline::InlineBaselineContext::Physical,
        );
        flex.display = l::box_model::Display::Flex;
        flex.align_items = Some(l::box_model::AlignItems::Baseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::node::LayoutInputOf::box_input(source_block(name, Some(10.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = overflow;
        target.block_flow_context = l::box_model::BlockFlowContext::Independent;
        target.baseline_source = l::baseline::BaselineSource::First;
        assert_eq!(
            host.push(Some(0), l::node::LayoutInputOf::box_input(target)),
            2
        );
        let line = source_line(&mut host, 2, name, 10.0);
        host.change(line, |input| {
            input.margin.top = l::length::LengthAutoOf::value(
                l::length::LengthPercentageOf::px(n(-16.0)).unwrap(),
            );
        });
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, line).location.y, n(-16.0));
        assert_eq!(node(phase, 2).location.y, n(target_y), "{overflow:?}");
    }
}

#[test]
fn flex_alignment_uses_block_scroll_start_baseline_in_both_lanes() {
    first_physical_scroll_baseline::<f32>();
    first_physical_scroll_baseline::<f64>();
}

#[test]
fn flex_alignment_uses_block_scroll_end_baseline_in_both_lanes() {
    last_physical_scroll_baseline::<f32>();
    last_physical_scroll_baseline::<f64>();
}

fn nested_physical_scroll_baseline<S: l::scalar::LayoutScalar>() {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let clip = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Clip,
        l::overflow::Overflow::Clip,
    )
    .unwrap();
    let hidden = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Hidden,
        l::overflow::Overflow::Hidden,
    )
    .unwrap();

    // The inner box begins at 10 and its last line ends at 55 in the inner
    // frame. The outer box exports 10 + 55 for clip and 10 + 40 for scroll.
    for (overflow, target_y) in [(clip, 15.0), (hidden, 30.0)] {
        let mut flex = root(
            mode,
            l::geometry::Direction::Ltr,
            l::baseline::InlineBaselineContext::Physical,
        );
        flex.display = l::box_model::Display::Flex;
        flex.align_items = Some(l::box_model::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::node::LayoutInputOf::box_input(source_block(name, Some(80.0))),
        );
        let target = host.push(
            Some(0),
            l::node::LayoutInputOf::box_input(source_block(name, Some(70.0))),
        );
        host.push(
            Some(target),
            l::node::LayoutInputOf::box_input(source_block(name, Some(10.0))),
        );
        let mut inner = source_block(name, Some(40.0));
        inner.overflow = overflow;
        inner.baseline_source = l::baseline::BaselineSource::Last;
        let inner = host.push(Some(target), l::node::LayoutInputOf::box_input(inner));
        source_line(&mut host, inner, name, 10.0);
        host.push(
            Some(inner),
            l::node::LayoutInputOf::box_input(source_block(name, Some(25.0))),
        );
        source_line(&mut host, inner, name, 20.0);

        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, inner).location.y, n(10.0));
        assert_eq!(node(phase, target).location.y, n(target_y), "{overflow:?}");
    }
}

#[test]
fn flex_alignment_uses_nested_block_scroll_baseline_in_both_lanes() {
    nested_physical_scroll_baseline::<f32>();
    nested_physical_scroll_baseline::<f64>();
}

fn legacy_physical_scroll_baseline<S: l::scalar::LayoutScalar>() {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let hidden = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Hidden,
        l::overflow::Overflow::Hidden,
    )
    .unwrap();

    // The peer's baseline is 60. Auto uses the block-end margin edge in
    // the scrolling target's 40px border frame. Last uses the content's 24.
    for (source, end_margin, target_y) in [
        (l::baseline::BaselineSource::Last, -6.0, 36.0),
        (l::baseline::BaselineSource::Auto, 8.0, 20.0),
        (l::baseline::BaselineSource::Auto, -6.0, 26.0),
        (l::baseline::BaselineSource::Auto, -50.0, 60.0),
    ] {
        let mut flex = root(
            mode,
            l::geometry::Direction::Ltr,
            l::baseline::InlineBaselineContext::Physical,
        );
        flex.display = l::box_model::Display::Flex;
        flex.align_items = Some(l::box_model::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::node::LayoutInputOf::box_input(source_block(name, Some(60.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = hidden;
        target.baseline_source = source;
        target.margin.bottom = l::length::LengthAutoOf::value(
            l::length::LengthPercentageOf::px(n(end_margin)).unwrap(),
        );
        assert_eq!(
            host.push(Some(0), l::node::LayoutInputOf::box_input(target)),
            2
        );
        source_content(&mut host, name, 4.0, 10.0);

        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(
            node(phase, 2).location.y,
            n(target_y),
            "{source:?} with end margin {end_margin}"
        );
    }
}

#[test]
fn flex_alignment_uses_legacy_block_scroll_end_margin_in_both_lanes() {
    legacy_physical_scroll_baseline::<f32>();
    legacy_physical_scroll_baseline::<f64>();
}

fn scroll_export_target<S: l::scalar::LayoutScalar>(
    display: l::box_model::Display,
    align: l::box_model::AlignItems,
    overflow: l::overflow::ComputedOverflow,
    peer_height: f64,
    target_height: f64,
) -> Host<S> {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let mut outer = root(
        mode,
        l::geometry::Direction::Ltr,
        l::baseline::InlineBaselineContext::Physical,
    );
    outer.display = l::box_model::Display::Flex;
    outer.align_items = Some(align);
    let mut host = Host::<S>::new(outer);
    host.push(
        Some(0),
        l::node::LayoutInputOf::box_input(source_block(name, Some(peer_height))),
    );
    let mut target = source_block(name, Some(target_height));
    target.display = display;
    target.overflow = overflow;
    if display == l::box_model::Display::Flex {
        target.flex_direction = l::flex::FlexDirection::Column;
        target.justify_content = Some(l::box_model::AlignContent::Start);
    } else {
        target.align_content = Some(l::box_model::AlignContent::Start);
        target.grid_template_columns = vec![
            l::grid::TrackSizingOf::from(l::length::LengthPercentageOf::px(n(20.0)).unwrap())
                .into(),
        ];
    }
    assert_eq!(
        host.push(Some(0), l::node::LayoutInputOf::box_input(target)),
        2
    );
    host
}

fn freeze_column_items<S: l::scalar::LayoutScalar>(
    host: &mut Host<S>,
    display: l::box_model::Display,
    items: &[u32],
) {
    if display == l::box_model::Display::Flex {
        for &item in items {
            host.change(item, |input| {
                input.flex_shrink = l::flex::FlexShrinkOf::try_new(S::ZERO).unwrap()
            });
        }
    }
}

fn scroll_container_export<S: l::scalar::LayoutScalar>(display: l::box_model::Display) {
    let name = FontBaseline::Alphabetic;
    let mode = l::geometry::WritingMode::HorizontalTb;
    let clip = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Clip,
        l::overflow::Overflow::Clip,
    )
    .unwrap();
    let hidden = l::overflow::ComputedOverflow::try_new(
        l::overflow::Overflow::Hidden,
        l::overflow::Overflow::Hidden,
    )
    .unwrap();

    for (overflow, target_y) in [(clip, 16.0), (hidden, 10.0)] {
        let mut host = scroll_export_target::<S>(
            display,
            l::box_model::AlignItems::Baseline,
            overflow,
            10.0,
            40.0,
        );
        let child = source_line(&mut host, 2, name, 10.0);
        host.change(child, |input| {
            input.margin.top = l::length::LengthAutoOf::value(
                l::length::LengthPercentageOf::px(n(-16.0)).unwrap(),
            );
        });
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, child).location.y, n(-16.0));
        assert_eq!(
            node(phase, 2).location.y,
            n(target_y),
            "{display:?} {overflow:?} first"
        );
    }

    // The 55px content baseline exceeds a 40px border and fits a 70px border.
    for (overflow, height, target_y) in
        [(clip, 40.0, 5.0), (hidden, 40.0, 20.0), (hidden, 70.0, 5.0)]
    {
        let mut host = scroll_export_target::<S>(
            display,
            l::box_model::AlignItems::LastBaseline,
            overflow,
            60.0,
            height,
        );
        source_content(&mut host, name, 25.0, 20.0);
        freeze_column_items(&mut host, display, &[3, 5, 6]);
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, 6).location.y, n(35.0));
        assert_eq!(
            node(phase, 2).location.y,
            n(target_y),
            "{display:?} {overflow:?} last at height {height}"
        );
    }

    // The nested Block clamps its own 55px baseline to 40px before the
    // target translates it by 10px. The target's scroll border ends at 45px.
    for (overflow, target_y) in [(clip, 30.0), (hidden, 35.0)] {
        let mut host = scroll_export_target::<S>(
            display,
            l::box_model::AlignItems::LastBaseline,
            overflow,
            80.0,
            45.0,
        );
        let spacer = host.push(
            Some(2),
            l::node::LayoutInputOf::box_input(source_block(name, Some(10.0))),
        );
        let mut inner = source_block(name, Some(40.0));
        inner.overflow = hidden;
        inner.baseline_source = l::baseline::BaselineSource::Last;
        let inner = host.push(Some(2), l::node::LayoutInputOf::box_input(inner));
        freeze_column_items(&mut host, display, &[spacer, inner]);
        source_line(&mut host, inner, name, 10.0);
        host.push(
            Some(inner),
            l::node::LayoutInputOf::box_input(source_block(name, Some(25.0))),
        );
        source_line(&mut host, inner, name, 20.0);
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, inner).location.y, n(10.0));
        assert_eq!(
            node(phase, 2).location.y,
            n(target_y),
            "{display:?} {overflow:?} nested"
        );
    }
}

#[test]
fn flex_scroll_baseline_exports_use_the_initial_border_frame_in_both_lanes() {
    scroll_container_export::<f32>(l::box_model::Display::Flex);
    scroll_container_export::<f64>(l::box_model::Display::Flex);
}

#[test]
fn grid_scroll_baseline_exports_use_the_initial_border_frame_in_both_lanes() {
    scroll_container_export::<f32>(l::box_model::Display::Grid);
    scroll_container_export::<f64>(l::box_model::Display::Grid);
}

#[test]
fn grid_lanes_scroll_baseline_exports_use_the_initial_border_frame_in_both_lanes() {
    scroll_container_export::<f32>(l::box_model::Display::GridLanes);
    scroll_container_export::<f64>(l::box_model::Display::GridLanes);
}
