use super::*;

fn last_physical_scroll_baseline<S: l::LayoutScalar>() {
    let name = l::BaselineName::Alphabetic;
    let mode = l::WritingMode::HorizontalTb;
    let clip = l::ComputedOverflow::try_new(l::Overflow::Clip, l::Overflow::Clip).unwrap();
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();

    // The peer's synthesized baseline is 60. The target's last baseline
    // lies at 10 + 25 + 20 before clamping to its 40px border extent.
    for (overflow, target_y) in [(clip, 5.0), (hidden, 20.0)] {
        let mut flex = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
        flex.display = l::Display::Flex;
        flex.align_items = Some(l::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(source_block(name, Some(60.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = overflow;
        target.baseline_source = l::BaselineSource::Last;
        assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(target)), 2);
        source_content(&mut host, name, 25.0, 20.0);
        let batch = host.compute(mode, &[]).unwrap();
        let phase = crate::layout::observation::phase(&batch, false);
        assert_eq!(node(phase, 1).location.y, S::ZERO);
        assert_eq!(node(phase, 6).location.y, n(35.0));
        assert_eq!(node(phase, 2).location.y, n(target_y), "{overflow:?}");
    }
}

fn first_physical_scroll_baseline<S: l::LayoutScalar>() {
    let name = l::BaselineName::Alphabetic;
    let mode = l::WritingMode::HorizontalTb;
    let clip = l::ComputedOverflow::try_new(l::Overflow::Clip, l::Overflow::Clip).unwrap();
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();

    // The 10px peer aligns against -16 + 10 before the scroll container
    // clamps its baseline to the start border edge.
    for (overflow, target_y) in [(clip, 16.0), (hidden, 10.0)] {
        let mut flex = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
        flex.display = l::Display::Flex;
        flex.align_items = Some(l::AlignItems::Baseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(source_block(name, Some(10.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = overflow;
        target.block_flow_context = l::BlockFlowContext::Independent;
        target.baseline_source = l::BaselineSource::First;
        assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(target)), 2);
        let line = source_line(&mut host, 2, name, 10.0);
        host.change(line, |input| {
            input.margin.top = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(-16.0)).unwrap());
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

fn nested_physical_scroll_baseline<S: l::LayoutScalar>() {
    let name = l::BaselineName::Alphabetic;
    let mode = l::WritingMode::HorizontalTb;
    let clip = l::ComputedOverflow::try_new(l::Overflow::Clip, l::Overflow::Clip).unwrap();
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();

    // The inner box begins at 10 and its last line ends at 55 in the inner
    // frame. The outer box exports 10 + 55 for clip and 10 + 40 for scroll.
    for (overflow, target_y) in [(clip, 15.0), (hidden, 30.0)] {
        let mut flex = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
        flex.display = l::Display::Flex;
        flex.align_items = Some(l::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(source_block(name, Some(80.0))),
        );
        let target = host.push(
            Some(0),
            l::LayoutInputOf::box_input(source_block(name, Some(70.0))),
        );
        host.push(
            Some(target),
            l::LayoutInputOf::box_input(source_block(name, Some(10.0))),
        );
        let mut inner = source_block(name, Some(40.0));
        inner.overflow = overflow;
        inner.baseline_source = l::BaselineSource::Last;
        let inner = host.push(Some(target), l::LayoutInputOf::box_input(inner));
        source_line(&mut host, inner, name, 10.0);
        host.push(
            Some(inner),
            l::LayoutInputOf::box_input(source_block(name, Some(25.0))),
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

fn legacy_physical_scroll_baseline<S: l::LayoutScalar>() {
    let name = l::BaselineName::Alphabetic;
    let mode = l::WritingMode::HorizontalTb;
    let hidden = l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();

    // The peer's baseline is 60. Auto uses the block-end margin edge in
    // the scrolling target's 40px border frame. Last uses the content's 24.
    for (source, end_margin, target_y) in [
        (l::BaselineSource::Last, -6.0, 36.0),
        (l::BaselineSource::Auto, 8.0, 20.0),
        (l::BaselineSource::Auto, -6.0, 26.0),
        (l::BaselineSource::Auto, -50.0, 60.0),
    ] {
        let mut flex = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
        flex.display = l::Display::Flex;
        flex.align_items = Some(l::AlignItems::LastBaseline);
        let mut host = Host::<S>::new(flex);
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(source_block(name, Some(60.0))),
        );
        let mut target = source_block(name, Some(40.0));
        target.overflow = hidden;
        target.baseline_source = source;
        target.margin.bottom =
            l::LengthAutoOf::value(l::LengthPercentageOf::px(n(end_margin)).unwrap());
        assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(target)), 2);
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
