//! A maximum cross size is a bound, not an allocated size or percentage basis.
use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn maximum_cross_source<S: LayoutScalar>(
    root_height: PreferredSizeOf<S>,
    max_height: Option<S>,
    child_height: PreferredSizeOf<S>,
    alignment: AlignItems,
) -> PublicLayoutTreeOf<S> {
    let mut root = NodeInputOf {
        display: Display::Flex,
        writing_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        size: Size::new(PreferredSizeOf::px(n(200.0)), root_height),
        align_items: Some(alignment),
        align_content: Some(AlignContent::FlexStart),
        justify_content: Some(AlignContent::FlexStart),
        ..NodeInputOf::default()
    };
    if let Some(height) = max_height {
        root.max_size.height = MaxSizeOf::px(height);
    }
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, root)
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, child_height),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                flex_grow: FlexGrowOf::ZERO,
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(1, Size::new(n(20.0), n(10.0)))
}

fn unbounded_normal_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let available = Size::new(AvailableOf::Definite(n(200.0)), AvailableOf::MAX_CONTENT);
    let request = LayoutRootRequestOf::viewport(available).unwrap();
    assert_eq!(request.available(), available);
    assert_eq!(request.context(), LayoutRootContextOf::Viewport);
    request
}

fn imposed_height_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    // The external parent's inline axis is physical height. This contextual
    // request supplies an actual100 allocation independently of any maximum.
    let parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let viewport = Size::splat(AvailableOf::Definite(n(400.0)));
    let context = FlexItemRootContextOf::under_viewport(viewport, parent).unwrap();
    let available = Size::new(
        AvailableOf::Definite(n(200.0)),
        AvailableOf::Definite(n(100.0)),
    );
    let request = LayoutRootRequestOf::flex_item_under_viewport(available, context).unwrap();
    assert_eq!(context.viewport_available(), viewport);
    assert_eq!(context.parent_flow_axes(), parent);
    assert_eq!(request.available(), available);
    assert_eq!(
        request.context(),
        LayoutRootContextOf::FlexItemUnderViewport(context)
    );
    request
}

fn assert_cross_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    root_height: f64,
    child_height: f64,
    context: &str,
) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        for (node, expected) in [
            (0, Size::new(n(200.0), n(root_height))),
            (1, Size::new(n(20.0), n(child_height))),
        ] {
            let actual = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{context} {phase} node {node}: missing native output"))
                .output();
            assert_eq!(
                actual.size, expected,
                "{context} {phase} node {node}: border size"
            );
            assert_eq!(
                actual.location,
                Point::ZERO,
                "{context} {phase} node {node}: location"
            );
            assert_eq!(
                actual.content_box_size(),
                expected,
                "{context} {phase} node {node}: content box"
            );
            assert_eq!(
                actual.source_index,
                SourceIndex::ZERO,
                "{context} {phase} node {node}: source index"
            );
            assert_eq!(
                (actual.margin, actual.border, actual.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{context} {phase} node {node}: edges"
            );
            let scroll = actual.scroll_geometry.unwrap_or_else(|| {
                panic!("{context} {phase} node {node}: missing canonical scroll geometry")
            });
            assert_eq!(
                scroll.flow_axes(),
                axes,
                "{context} {phase} node {node}: scroll axes"
            );
            for (field, rect) in [
                ("border box", scroll.border_box()),
                ("padding box", scroll.padding_box()),
                ("content box", scroll.content_box()),
                ("scrollport", scroll.scrollport()),
                ("scrollable overflow", scroll.scrollable_overflow()),
            ] {
                assert_eq!(
                    rect.origin(),
                    Point::ZERO,
                    "{context} {phase} node {node}: {field} origin"
                );
                assert_eq!(
                    rect.size(),
                    expected,
                    "{context} {phase} node {node}: {field} size"
                );
            }
            assert_eq!(
                scroll.scrollbar_size(),
                Size::ZERO,
                "{context} {phase} node {node}: scrollbars"
            );
            let range = scroll.physical_range();
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, S::ZERO, S::ZERO, S::ZERO),
                "{context} {phase} node {node}: scroll range"
            );
        }
    }
}

fn stretch_from_actual_line<S: LayoutScalar>() {
    let natural = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let natural = compute_layout(&natural, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&natural, 10.0, 10.0, "native natural-height control");

    let definite = maximum_cross_source::<S>(
        PreferredSizeOf::px(n(100.0)),
        Some(n(100.0)),
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let definite = compute_layout(&definite, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&definite, 100.0, 100.0, "authored definite-height control");

    let maximum = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        Some(n(100.0)),
        PreferredSizeOf::AUTO,
        AlignItems::Stretch,
    );
    let imposed = compute_layout(&maximum, 0, imposed_height_request()).unwrap();
    assert_cross_geometry(&imposed, 100.0, 100.0, "actual host-known-height control");

    // The natural line is10; max-height100 only clamps it. Stretch uses that
    // actual line size rather than allocating the maximum in advance.
    let maximum = compute_layout(&maximum, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&maximum, 10.0, 10.0, "maximum-only cross size");
}

fn percentage_requires_actual_parent_size<S: LayoutScalar>() {
    let percentage =
        PreferredSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap());
    let definite = maximum_cross_source::<S>(
        PreferredSizeOf::px(n(100.0)),
        Some(n(100.0)),
        percentage.clone(),
        AlignItems::FlexStart,
    );
    let definite = compute_layout(&definite, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(
        &definite,
        100.0,
        50.0,
        "authored definite percentage basis control",
    );

    let maximum = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        Some(n(100.0)),
        percentage,
        AlignItems::FlexStart,
    );
    let imposed = compute_layout(&maximum, 0, imposed_height_request()).unwrap();
    assert_cross_geometry(
        &imposed,
        100.0,
        50.0,
        "actual host-known percentage basis control",
    );

    // A content-dependent parent height does not resolve the item's cyclic
    // percentage against its maximum. The percentage behaves as Auto.
    let maximum = compute_layout(&maximum, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&maximum, 10.0, 10.0, "maximum-only percentage parent");
}

fn percentage_under_auto_parent_behaves_as_auto<S: LayoutScalar>() {
    let automatic = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        PreferredSizeOf::AUTO,
        AlignItems::FlexStart,
    );
    let automatic = compute_layout(&automatic, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(&automatic, 10.0, 10.0, "automatic-height native control");

    // Resolving the line establishes the item's used cross size for layout;
    // it does not resolve the item's own cyclic percentage against its parent.
    let percentage =
        PreferredSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap());
    let natural = maximum_cross_source::<S>(
        PreferredSizeOf::AUTO,
        None,
        percentage,
        AlignItems::FlexStart,
    );
    let natural = compute_layout(&natural, 0, unbounded_normal_request()).unwrap();
    assert_cross_geometry(
        &natural,
        10.0,
        10.0,
        "auto parent keeps cyclic percentage automatic",
    );
}

#[test]
fn max_only_flex_cross_size_stretches_to_natural_line_f32() {
    stretch_from_actual_line::<f32>();
}
#[test]
fn max_only_flex_cross_size_stretches_to_natural_line_f64() {
    stretch_from_actual_line::<f64>();
}
#[test]
fn max_only_flex_cross_size_does_not_resolve_percentage_height_f32() {
    percentage_requires_actual_parent_size::<f32>();
}
#[test]
fn max_only_flex_cross_size_does_not_resolve_percentage_height_f64() {
    percentage_requires_actual_parent_size::<f64>();
}

#[test]
fn auto_height_flex_cross_size_does_not_resolve_percentage_height_f32() {
    percentage_under_auto_parent_behaves_as_auto::<f32>();
}
#[test]
fn auto_height_flex_cross_size_does_not_resolve_percentage_height_f64() {
    percentage_under_auto_parent_behaves_as_auto::<f64>();
}
