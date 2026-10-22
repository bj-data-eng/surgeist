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
