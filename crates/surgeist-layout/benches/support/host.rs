//! A minimal public-API layout host over generated trees.

use std::convert::Infallible;

use surgeist_layout::{
    box_model::Display,
    constraint::SizingOperation,
    geometry::{Direction, FlowAxes, Size, WritingMode},
    length::{LengthPercentageOf, NonNegativeFiniteOf},
    node::{LayoutInput, LayoutInputView, NodeInput},
    request::LayoutRootRequest,
    sizing::PreferredSize,
    tree::{LayoutTree, LeafMeasureInput, Traverse},
};

#[path = "../../tests/support/native_request.rs"]
mod native_request;

/// Generated nodes and their child lists. Node zero is the root.
pub struct Host {
    pub inputs: Vec<LayoutInput>,
    pub children: Vec<Vec<u32>>,
    non_box: NodeInput,
    /// Floats select the shape exclusion and an inscribed-ellipse provider answers them.
    pub shape_floats: bool,
    /// Leaves the host measures: min-content width 10, max-content width 80, a normal width
    /// of the available space, and height 20.
    pub measured: Vec<bool>,
}

impl Host {
    pub fn new(root: NodeInput) -> Self {
        Self {
            inputs: vec![LayoutInput::box_input(root)],
            children: vec![Vec::new()],
            non_box: NodeInput::non_box(),
            shape_floats: false,
            measured: vec![false],
        }
    }

    /// Appends a box node under `parent` and returns its id.
    pub fn add(&mut self, parent: u32, input: NodeInput) -> u32 {
        let node = self.push(LayoutInput::box_input(input));
        self.children[parent as usize].push(node);
        node
    }

    /// Appends a node without attaching it to a parent.
    pub fn push(&mut self, input: LayoutInput) -> u32 {
        let node = u32::try_from(self.inputs.len()).expect("generated trees fit u32 ids");
        self.inputs.push(input);
        self.children.push(Vec::new());
        self.measured.push(false);
        node
    }

    pub fn shape(&self) -> (usize, usize) {
        super::harness::shape(&self.children)
    }
}

impl Traverse for Host {
    type Node = u32;
    type Scalar = f32;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}

impl LayoutTree for Host {
    type MeasureError = Infallible;

    fn node_input(&self, node: u32) -> &NodeInput {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> LayoutInputView<'_> {
        self.inputs[node as usize].as_view()
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        self.measured[node as usize]
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInput,
    ) -> Option<Result<Size<f32>, Infallible>> {
        if !self.measured[node as usize] {
            return None;
        }
        let known = input.known_content_size();
        let width = known.width.unwrap_or_else(|| {
            match input.content_sizing_request().axes().width.operation() {
                SizingOperation::MinContent => 10.0,
                SizingOperation::MaxContent => 80.0,
                SizingOperation::Normal => input
                    .content_sizing_request()
                    .axes()
                    .width
                    .space()
                    .definite_size()
                    .map_or(80.0, NonNegativeFiniteOf::get),
            }
        });
        Some(Ok(Size::new(width, known.height.unwrap_or(20.0))))
    }
    fn float_exclusion_interval(
        &self,
        _node: u32,
        query: surgeist_layout::box_model::FloatExclusionQuery,
    ) -> Option<Result<Option<surgeist_layout::box_model::FloatExclusionInterval>, Infallible>>
    {
        if !self.shape_floats {
            return None;
        }
        // The widest chord of the ellipse inscribed in the margin box over the
        // queried vertical band.
        let origin = query.margin_box().origin();
        let size = query.margin_box().size();
        let radius = size.height / 2.0;
        let centre = origin.y + radius;
        let nearest = centre.clamp(query.band_minimum(), query.band_maximum());
        let offset = ((nearest - centre) / radius).abs().min(1.0);
        let half = size.width / 2.0 * (1.0 - offset * offset).sqrt();
        let across = origin.x + size.width / 2.0;
        Some(Ok(
            surgeist_layout::box_model::FloatExclusionInterval::try_new(
                query,
                across - half,
                across + half,
            )
            .expect("the ellipse chord is finite and ordered"),
        ))
    }
}

pub fn px(value: f32) -> PreferredSize {
    PreferredSize::value(LengthPercentageOf::px(value).expect("finite length"))
}

/// A root request over an inline extent and an unknown block extent, both sized normally.
pub fn root_request(width: f32) -> LayoutRootRequest {
    let sizing = native_request::sizing(
        Size::new(Some(width), None),
        Size::splat(SizingOperation::Normal),
    );
    LayoutRootRequest::standalone(
        sizing,
        flow_axes(),
        sizing.orthogonal_fallback().initial_containing_block(),
    )
}

pub fn flow_axes() -> FlowAxes {
    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
}

/// A box of the given display and optional fixed width and height.
pub fn boxed(display: Display, width: Option<f32>, height: Option<f32>) -> NodeInput {
    NodeInput {
        display,
        size: Size::new(
            width.map_or(PreferredSize::AUTO, px),
            height.map_or(PreferredSize::AUTO, px),
        ),
        ..NodeInput::default()
    }
}
