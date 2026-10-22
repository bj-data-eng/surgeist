//! Public intrinsic-size qualification at finite, very small available widths.
//! CSS Grid1 §5.2 defines max-content from the fixed track sum; Overflow3
//! §4.1/§4.2 reserves stable classic gutters as border-box space, even for
//! hidden overflow. Available space is not an authored definite border box.
use surgeist_layout::*;

struct FixedGrid<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 2],
}

impl<S: LayoutScalar> Traverse for FixedGrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
    }

    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!((node, index), (0, 0));
        1
    }
}

impl<S: LayoutScalar> LayoutTree for FixedGrid<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn preferred<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn track<S: LayoutScalar>(value: f64) -> TrackComponentOf<S> {
    let value = SizingCalculationOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap());
    TrackComponentOf::minmax(
        MinTrackSizingOf::Calculation(value.clone()),
        MaxTrackSizingOf::Calculation(value),
    )
}

fn fixture<S: LayoutScalar>(
    display: Display,
    gutter: ScrollbarGutter,
    width: PreferredSizeOf<S>,
) -> FixedGrid<S> {
    FixedGrid {
        inputs: [
            LayoutInputOf::box_input(NodeInputOf {
                display,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(width, preferred(20.0)),
                grid_template_columns: vec![track(40.0)],
                grid_template_rows: vec![track(20.0)],
                overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
                scrollbar_gutter: gutter,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap(),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            }),
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(preferred(40.0), preferred(20.0)),
                ..NodeInputOf::default()
            }),
        ],
    }
}

fn qualify<S: LayoutScalar>(intrinsic: bool) {
    let mut failures = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for available in [0.0, 2.0] {
            for (gutter, reservation) in [
                (ScrollbarGutter::Auto, 0.0),
                (ScrollbarGutter::Stable, 5.0),
                (ScrollbarGutter::StableBothEdges, 10.0),
            ] {
                let width = if intrinsic {
                    PreferredSizeOf::MAX_CONTENT
                } else {
                    preferred(available)
                };
                let tree = fixture::<S>(display, gutter, width);
                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::new(
                        AvailableOf::Definite(S::from_f64(available)),
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap(),
                )
                .unwrap();
                let root = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output();
                let child = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 1)
                    .unwrap()
                    .output();
                let geometry = root
                    .scroll_geometry
                    .expect("hidden root is a genuine scroll container");
                let expected_width = if intrinsic {
                    40.0 + reservation
                } else {
                    available
                };
                let expected_content_width = if intrinsic {
                    40.0
                } else {
                    (available - reservation).max(0.0)
                };
                let actual = (root.size, child.size, geometry.content_box().size());
                let expected = (
                    Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                    Size::new(S::from_f64(40.0), S::from_f64(20.0)),
                    Size::new(S::from_f64(expected_content_width), S::from_f64(20.0)),
                );
                eprintln!(
                    "{} {display:?} intrinsic={intrinsic} available={available} {gutter:?}: actual={actual:?}, expected={expected:?}",
                    std::any::type_name::<S>()
                );
                if actual != expected {
                    failures.push(format!("{display:?} available={available} {gutter:?}: actual={actual:?}, expected={expected:?}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn max_content_fixed_grid_gutters_use_intrinsic_box_despite_tiny_available_width_f32() {
    qualify::<f32>(true);
}

#[test]
fn max_content_fixed_grid_gutters_use_intrinsic_box_despite_tiny_available_width_f64() {
    qualify::<f64>(true);
}

#[test]
fn authored_tiny_definite_grid_border_box_still_clamps_gutters_f32() {
    qualify::<f32>(false);
}

#[test]
fn authored_tiny_definite_grid_border_box_still_clamps_gutters_f64() {
    qualify::<f64>(false);
}

struct InheritedColumnGrid<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 3],
}

impl<S: LayoutScalar> Traverse for InheritedColumnGrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = match node {
            0 => &[1],
            1 => &[2],
            _ => &[],
        };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node < 2)
    }

    fn child(&self, node: usize, index: usize) -> usize {
        assert!(node < 2);
        assert_eq!(index, 0);
        node + 1
    }
}

impl<S: LayoutScalar> LayoutTree for InheritedColumnGrid<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn qualify_inherited_columns<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (overflow, gutter, leaf_height, expected_width, reservation) in [
            (Overflow::Auto, ScrollbarGutter::Auto, 20.0, 40.0, 0.0),
            (Overflow::Auto, ScrollbarGutter::Auto, 30.0, 45.0, 5.0),
            (Overflow::Hidden, ScrollbarGutter::Stable, 20.0, 45.0, 5.0),
        ] {
            // Grid2 §9 subgrid-item-contribution/subgrid-margins applies the
            // child's gutter to its edge item's intrinsic contribution. The
            // 40px leaf fills the inherited column. Its 30px block extent
            // overflows the child's independently fixed20 row, requiring5
            // inline gutter. No-overflow Auto and Hidden Stable are controls.
            let tree = InheritedColumnGrid::<S> {
                inputs: [
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        size: Size::new(PreferredSizeOf::MAX_CONTENT, preferred(20.0)),
                        grid_template_columns: vec![TrackComponentOf::MAX_CONTENT],
                        grid_template_rows: vec![track(20.0)],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display,
                        size: Size::new(PreferredSizeOf::AUTO, preferred(20.0)),
                        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                            vec![],
                        ))],
                        grid_template_rows: vec![track(20.0)],
                        grid_auto_flow: GridAutoFlow::Row,
                        overflow: ComputedOverflow::try_new(overflow, overflow).unwrap(),
                        scrollbar_gutter: gutter,
                        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap(),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Block,
                        size: Size::new(preferred(40.0), preferred(leaf_height)),
                        ..NodeInputOf::default()
                    }),
                ],
            };
            let request =
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap();
            let batch = match compute_layout(&tree, 0, request) {
                Ok(batch) => batch,
                Err(error) => {
                    failures.push(format!(
                        "{display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: {error:?}"
                    ));
                    continue;
                }
            };
            let output = |node| {
                batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            let parent = output(0);
            let child = output(1);
            let leaf = output(2);
            let geometry = child.scroll_geometry.expect("actual child scroll geometry");
            let actual = (
                parent.size,
                child.size,
                leaf.size,
                geometry.content_box().size(),
                geometry.scrollbar_size(),
            );
            let expected = (
                Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                Size::new(S::from_f64(40.0), S::from_f64(leaf_height)),
                Size::new(S::from_f64(40.0), S::from_f64(20.0)),
                Size::new(S::from_f64(reservation), S::ZERO),
            );
            eprintln!(
                "{} inherited_columns {display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: actual={actual:?}, expected={expected:?}",
                std::any::type_name::<S>()
            );
            if actual != expected {
                failures.push(format!("{display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: actual={actual:?}, expected={expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn inherited_columns_include_child_settled_auto_gutter_f32() {
    qualify_inherited_columns::<f32>();
}

#[test]
fn inherited_columns_include_child_settled_auto_gutter_f64() {
    qualify_inherited_columns::<f64>();
}
