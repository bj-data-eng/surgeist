use crate::test_support::readback::{self, Phase};
use std::collections::HashMap;

use crate::LayoutOutputEntryOf;
use crate::output::NormalGeometryOf;

use crate::{
    AvailableSpaceOf, Compute, ComputeInputOf, ComputeOutputOf, DefaultScalar, Display,
    InlineBoundaryInputOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInputOf,
    LayoutInternalInvariant, LayoutOperation, LayoutScalar, LayoutTree, LeafMeasureInputOf,
    LineBreakInput, LineBreakInputOf, NodeInputOf, NodeOutputOf, RequestedAxis, Round, RunMode,
    Size, SizingMode, SizingOperation, Traverse, compute_block, compute_flex, compute_grid,
};

/// The fake retained no current-trial custody for this actual request.
pub(crate) fn missing_staged_unrounded<Node, S: LayoutScalar, M>(
    node: Node,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::MissingStagedUnroundedOutput),
    )
}

/// Authors an ordinary box fixture with no relative visual displacement.
pub(crate) fn ordinary_unrounded_box<S: LayoutScalar>(
    node: u32,
    layout: NodeOutputOf<S>,
) -> LayoutOutputEntryOf<u32, S> {
    let content = layout
        .scroll_geometry
        .map(|geometry| geometry.content_box())
        .unwrap_or_else(|| {
            crate::ScrollRectOf::try_new(
                crate::Point::new(
                    layout.border.left + layout.padding.left,
                    layout.border.top + layout.padding.top,
                ),
                layout.content_box_size(),
            )
            .expect("ordinary oracle box has valid local content geometry")
        });
    LayoutOutputEntryOf::ordinary_unrounded(
        node,
        NormalGeometryOf::box_geometry(layout.location, layout.size, content),
        layout,
    )
}

pub type OracleTree = OracleTreeOf<DefaultScalar>;
pub type OracleMeasurement = OracleMeasurementOf<DefaultScalar>;

#[derive(Clone, Copy, Debug)]
enum PublicMeasurementOf<S: LayoutScalar> {
    Fixed(Size<S>),
    FromInput(fn(LeafMeasureInputOf<S>) -> Size<S>),
}

#[derive(Clone, Debug)]
pub struct PublicLayoutTreeOf<S: LayoutScalar = DefaultScalar> {
    children: HashMap<u32, Vec<u32>>,
    layout_inputs: HashMap<u32, LayoutInputOf<S>>,
    measurements: HashMap<u32, PublicMeasurementOf<S>>,
    non_box_input: NodeInputOf<S>,
    measurement_queries: std::cell::Cell<Option<usize>>,
}

impl<S: LayoutScalar> Default for PublicLayoutTreeOf<S> {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            layout_inputs: HashMap::new(),
            measurements: HashMap::new(),
            non_box_input: NodeInputOf::non_box(),
            measurement_queries: std::cell::Cell::new(None),
        }
    }
}

impl<S: LayoutScalar> PublicLayoutTreeOf<S> {
    pub fn reset_measurement_queries(&self) {
        self.measurement_queries.set(Some(0));
    }

    pub fn measurement_queries(&self) -> usize {
        self.measurement_queries
            .get()
            .expect("measurement query recording enabled")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub fn insert_children(&mut self, node: u32, children: impl IntoIterator<Item = u32>) {
        self.children.insert(node, children.into_iter().collect());
    }

    pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
        self
    }

    pub fn input(mut self, node: u32, input: LayoutInputOf<S>) -> Self {
        self.layout_inputs.insert(node, input);
        self
    }

    pub fn insert_input(&mut self, node: u32, input: LayoutInputOf<S>) {
        self.layout_inputs.insert(node, input);
    }

    pub fn measure(mut self, node: u32, size: Size<S>) -> Self {
        self.measurements
            .insert(node, PublicMeasurementOf::Fixed(size));
        self
    }

    pub fn measure_with(
        mut self,
        node: u32,
        measure: fn(LeafMeasureInputOf<S>) -> Size<S>,
    ) -> Self {
        self.measurements
            .insert(node, PublicMeasurementOf::FromInput(measure));
        self
    }
}

impl<S: LayoutScalar> Traverse for PublicLayoutTreeOf<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<S: LayoutScalar> LayoutTree for PublicLayoutTreeOf<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        let input = self
            .layout_inputs
            .get(&node)
            .unwrap_or_else(|| panic!("public layout node {node} must define a layout input"));
        input.as_box().unwrap_or(&self.non_box_input)
    }

    fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'_, S> {
        self.layout_inputs
            .get(&node)
            .map(|input| input.as_view())
            .unwrap_or_else(|| panic!("public layout node {node} must define a layout input"))
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        if let Some(count) = self.measurement_queries.get() {
            self.measurement_queries.set(Some(count + 1));
        }
        self.measurements.contains_key(&node)
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.measurements.get(&node).map(|measurement| {
            Ok(match measurement {
                PublicMeasurementOf::Fixed(size) => *size,
                PublicMeasurementOf::FromInput(measure) => measure(input),
            })
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct OracleTreeOf<S: LayoutScalar = DefaultScalar> {
    children: HashMap<u32, Vec<u32>>,
    layout_inputs: HashMap<u32, LayoutInputOf<S>>,
    measurements: HashMap<u32, Vec<OracleMeasurementOf<S>>>,
    compute_inputs: HashMap<u32, Vec<ComputeInputOf<S>>>,
    layouts: HashMap<u32, LayoutOutputEntryOf<u32, S>>,
    final_layouts: HashMap<u32, LayoutOutputEntryOf<u32, S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OracleMeasurementOf<S: LayoutScalar = DefaultScalar> {
    run_mode: Option<RunMode>,
    sizing_mode: Option<SizingMode>,
    axis: Option<RequestedAxis>,
    known: Option<Size<Option<S>>>,
    parent: Option<Size<Option<S>>>,
    operations: Size<Option<SizingOperation>>,
    spaces: Size<Option<AvailableSpaceOf<S>>>,
    output: ComputeOutputOf<S>,
}

impl<S: LayoutScalar> OracleMeasurementOf<S> {
    pub const fn new(output: ComputeOutputOf<S>) -> Self {
        Self {
            run_mode: None,
            sizing_mode: None,
            axis: None,
            known: None,
            parent: None,
            operations: Size::new(None, None),
            spaces: Size::new(None, None),
            output,
        }
    }

    pub const fn run_mode(mut self, run_mode: RunMode) -> Self {
        self.run_mode = Some(run_mode);
        self
    }

    pub const fn known(mut self, known: Size<Option<S>>) -> Self {
        self.known = Some(known);
        self
    }

    pub const fn parent(mut self, parent: Size<Option<S>>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub const fn operations(mut self, operations: Size<Option<SizingOperation>>) -> Self {
        self.operations = operations;
        self
    }

    pub const fn spaces(mut self, spaces: Size<Option<AvailableSpaceOf<S>>>) -> Self {
        self.spaces = spaces;
        self
    }

    fn matches(self, input: ComputeInputOf<S>) -> bool {
        let axes = input.sizing_request().axes();
        matches_or_any(self.run_mode, input.run_mode())
            && matches_or_any(self.sizing_mode, input.sizing_mode())
            && matches_or_any(self.axis, input.requested_axis())
            && matches_or_any(self.known, input.known())
            && matches_or_any(self.parent, input.parent())
            && matches_or_any(self.operations.width, axes.width.operation())
            && matches_or_any(self.operations.height, axes.height.operation())
            && matches_or_any(self.spaces.width, axes.width.space())
            && matches_or_any(self.spaces.height, axes.height.space())
    }
}

fn matches_or_any<T: Copy + PartialEq>(expected: Option<T>, actual: T) -> bool {
    match expected {
        Some(expected) => expected == actual,
        None => true,
    }
}

impl<S: LayoutScalar> OracleTreeOf<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, node: u32, children: impl IntoIterator<Item = u32>) -> Self {
        self.children.insert(node, children.into_iter().collect());
        self
    }

    pub fn insert_children(&mut self, node: u32, children: impl IntoIterator<Item = u32>) {
        self.children.insert(node, children.into_iter().collect());
    }

    pub fn style(mut self, node: u32, style: NodeInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
        self
    }

    pub fn insert_style(&mut self, node: u32, style: NodeInputOf<S>) {
        self.layout_inputs
            .insert(node, LayoutInputOf::box_input(style));
    }

    pub fn line_break(mut self, node: u32, input: LineBreakInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::LineBreak(input));
        self
    }

    pub fn inline_boundary(mut self, node: u32, input: InlineBoundaryInputOf<S>) -> Self {
        self.layout_inputs
            .insert(node, LayoutInputOf::InlineBoundary(input));
        self
    }

    pub fn measure(mut self, node: u32, output: ComputeOutputOf<S>) -> Self {
        self.measurements
            .entry(node)
            .or_default()
            .push(OracleMeasurementOf::new(output));
        self
    }

    pub fn insert_measure(&mut self, node: u32, output: ComputeOutputOf<S>) {
        self.measurements
            .entry(node)
            .or_default()
            .push(OracleMeasurementOf::new(output));
    }

    pub fn measure_when(mut self, node: u32, measurement: OracleMeasurementOf<S>) -> Self {
        self.measurements.entry(node).or_default().push(measurement);
        self
    }

    pub fn unrounded(mut self, node: u32, layout: NodeOutputOf<S>) -> Self {
        self.layouts
            .insert(node, ordinary_unrounded_box(node, layout));
        self
    }

    pub fn inputs(&self, node: u32) -> &[ComputeInputOf<S>] {
        self.compute_inputs
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.layouts.get(&node).map(LayoutOutputEntryOf::output)
    }

    pub fn final_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.final_layouts
            .get(&node)
            .map(LayoutOutputEntryOf::output)
    }

    pub fn output(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.final_layout(node).or_else(|| self.layout(node))
    }
}

impl<S: LayoutScalar> Traverse for OracleTreeOf<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

pub struct OracleComputeOf<'input, 'output, S: LayoutScalar = DefaultScalar> {
    block_floats: crate::block::float_context::State<u32, S>,
    children: &'input HashMap<u32, Vec<u32>>,
    layout_inputs: &'input HashMap<u32, LayoutInputOf<S>>,
    measurements: &'input HashMap<u32, Vec<OracleMeasurementOf<S>>>,
    compute_inputs: &'output mut HashMap<u32, Vec<ComputeInputOf<S>>>,
    layouts: &'output mut HashMap<u32, LayoutOutputEntryOf<u32, S>>,
    final_layouts: &'output mut HashMap<u32, LayoutOutputEntryOf<u32, S>>,
}

impl<'input, S: LayoutScalar> OracleComputeOf<'input, '_, S> {
    /// The oracle records real requests but gives each prospective traversal
    /// private output storage, just as its existing overflow observation does.
    fn staging<'trial>(
        &'trial mut self,
        block_floats: crate::block::float_context::State<u32, S>,
        layouts: &'trial mut HashMap<u32, LayoutOutputEntryOf<u32, S>>,
        final_layouts: &'trial mut HashMap<u32, LayoutOutputEntryOf<u32, S>>,
    ) -> OracleComputeOf<'input, 'trial, S> {
        OracleComputeOf {
            block_floats,
            children: self.children,
            layout_inputs: self.layout_inputs,
            measurements: self.measurements,
            compute_inputs: &mut *self.compute_inputs,
            layouts,
            final_layouts,
        }
    }
}

impl<S: LayoutScalar> OracleTreeOf<S> {
    pub fn borrow_compute(&mut self) -> OracleComputeOf<'_, '_, S> {
        OracleComputeOf {
            block_floats: crate::block::float_context::State::new(),
            children: &self.children,
            layout_inputs: &self.layout_inputs,
            measurements: &self.measurements,
            compute_inputs: &mut self.compute_inputs,
            layouts: &mut self.layouts,
            final_layouts: &mut self.final_layouts,
        }
    }

    pub fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        match self.layout_inputs.get(&node) {
            Some(input) => input
                .as_box()
                .unwrap_or_else(|| panic!("line break node has no box NodeInput")),
            None => panic!("oracle node {node} must define a layout input"),
        }
    }

    pub fn layout_input(&self, node: u32) -> crate::LayoutInputViewOf<'_, S> {
        self.layout_inputs
            .get(&node)
            .map(|input| input.as_view())
            .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
    }
}

impl<'input, S: LayoutScalar> Traverse for OracleComputeOf<'input, '_, S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.children.get(&node).map(Vec::len).unwrap_or(0)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[&node][index]
    }
}

impl<'input, S: LayoutScalar> Compute<'input> for OracleComputeOf<'input, '_, S> {
    fn block_float_state(
        &mut self,
    ) -> Option<&mut crate::block::float_context::State<Self::Node, Self::Scalar>> {
        Some(&mut self.block_floats)
    }
    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.measurements.contains_key(&node)
    }

    fn qualify_entry(
        &mut self,
        query: crate::block::entry::EntryQuery<Self::Node, S>,
    ) -> crate::block::entry::EntryResult<Self::Node, S, core::convert::Infallible> {
        if query.purpose == crate::block::entry::EntryPurpose::StartOnly {
            return crate::block::entry::observe(self, query);
        }
        let incoming = query
            .incoming
            .as_ref()
            .filter(|incoming| incoming.node == query.node && incoming.request == query.input)
            .ok_or_else(|| crate::block::float_context::missing(query.node))?;
        let floats = self.block_floats.fork_framed(incoming.clone());
        let mut layouts = HashMap::new();
        let mut final_layouts = HashMap::new();
        let mut trial = self.staging(floats, &mut layouts, &mut final_layouts);
        crate::block::entry::resolve_framed(&mut trial, query)
    }

    fn measure_entry_float(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> crate::LayoutResultOf<Self::Node, ComputeOutputOf<S>, S> {
        if input.run_mode() != RunMode::ComputeSize || self.node_input(node).float.is_none() {
            return Err(crate::block::float_context::missing(node));
        }
        // This executor has no Text/Formatted context. A detached real numeric
        // computation supplies the float's used size without exporting its body.
        let floats = self.block_floats.fork();
        let mut layouts = HashMap::new();
        let mut final_layouts = HashMap::new();
        self.staging(floats, &mut layouts, &mut final_layouts)
            .compute_child(node, input)
    }

    fn node_input(&self, node: Self::Node) -> &'input NodeInputOf<S> {
        match self.layout_inputs.get(&node) {
            Some(input) => input
                .as_box()
                .unwrap_or_else(|| panic!("line break node has no box NodeInput")),
            None => panic!("oracle node {node} must define a layout input"),
        }
    }

    fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, S> {
        self.layout_inputs
            .get(&node)
            .map(|input| input.as_view())
            .unwrap_or_else(|| panic!("oracle node {node} must define a layout input"))
    }

    fn staged_unrounded(
        &self,
        node: Self::Node,
    ) -> crate::engine::contracts::UnroundedEntryResult<
        Self::Node,
        Self::Scalar,
        core::convert::Infallible,
    > {
        self.layouts
            .get(&node)
            .copied()
            .filter(|entry| entry.node() == node && entry.geometry_state().is_unrounded())
            .ok_or_else(|| crate::test_support::layout_tree::missing_staged_unrounded(node))
    }

    fn set_unrounded(&mut self, entry: LayoutOutputEntryOf<Self::Node, S>) {
        self.layouts.insert(entry.node(), entry);
    }

    fn observe_grid_overflow(
        &mut self,
        node: Self::Node,
        probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
    ) -> crate::LayoutResultOf<
        Self::Node,
        crate::grid::GridOverflowObservation<Self::Scalar>,
        Self::Scalar,
    > {
        let mut layouts = HashMap::new();
        let mut final_layouts = HashMap::new();
        let floats = self.block_floats.fork();
        let mut observation = self.staging(floats, &mut layouts, &mut final_layouts);
        probe.observe(&mut observation, node)
    }

    fn compute_child(
        &mut self,
        node: Self::Node,
        input: ComputeInputOf<S>,
    ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar> {
        self.compute_inputs.entry(node).or_default().push(input);

        if let Some(measurements) = self.measurements.get(&node) {
            let output = measurements
                .iter()
                .find(|measurement| measurement.matches(input))
                .unwrap_or_else(|| {
                    panic!("no oracle measurement matched node {node} input {input:?}")
                })
                .output;
            return Ok(output);
        }

        match self.node_input(node).display.inner_display() {
            Display::Block => compute_block(self, node, input),
            Display::Flex => compute_flex(self, node, input),
            Display::Grid | Display::GridLanes => compute_grid(self, node, input),
            Display::None => {
                self.set_unrounded(LayoutOutputEntryOf::ordinary_unrounded(
                    node,
                    NormalGeometryOf::metadata(crate::Point::ZERO, Size::ZERO),
                    NodeOutputOf::with_source_index(crate::SourceIndex::ZERO),
                ));
                Ok(ComputeOutputOf::HIDDEN)
            }
            Display::InlineBlock | Display::InlineGrid | Display::InlineGridLanes => {
                unreachable!("inner_display removes inline display variants")
            }
        }
    }
}

impl<'input, S: LayoutScalar> Round for OracleComputeOf<'input, '_, S> {
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> crate::LayoutResultOf<Self::Node, LayoutOutputEntryOf<Self::Node, S>, S> {
        self.layouts.get(&node).copied().ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::MissingStagedUnroundedOutput,
                ),
            )
        })
    }

    fn set_final(&mut self, entry: LayoutOutputEntryOf<Self::Node, S>) {
        self.final_layouts.insert(entry.node(), entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContainingLayoutContext, Direction, FlowAxes, LayoutErrorKind, LayoutErrorSite,
        LayoutInternalInvariant, LayoutOperation, LayoutRootRequestOf, LineBreakDisplay,
        ParentFormattingContext, PreferredSizeOf, SourceIndex, WritingMode, compute_layout,
    };

    fn assert_fri06_c13_t01_public_tree_typed_contract<S: LayoutScalar>()
    where
        PublicLayoutTreeOf<S>:
            LayoutTree<MeasureError = core::convert::Infallible> + Traverse<Node = u32, Scalar = S>,
    {
        let style = NodeInputOf::<S> {
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(41.0)),
                PreferredSizeOf::px(S::from_f64(17.0)),
            ),
            ..NodeInputOf::default()
        };
        let line_break = LineBreakInputOf::<S>::new().hidden();
        let tree = PublicLayoutTreeOf::new()
            .children(1, [7, 3, 5])
            .style(1, style.clone())
            .input(7, LayoutInputOf::line_break(line_break));

        assert_eq!(
            <PublicLayoutTreeOf<S> as Traverse>::children(&tree, 1).collect::<Vec<_>>(),
            [7, 3, 5]
        );
        assert_eq!(tree.child_count(1), 3);
        assert_eq!(tree.child(1, 0), 7);
        assert_eq!(tree.child(1, 1), 3);
        assert_eq!(tree.child(1, 2), 5);
        assert_eq!(tree.node_input(1), &style);
        assert_eq!(tree.layout_input(1), crate::LayoutInputViewOf::Box(&style));
        assert_eq!(
            tree.layout_input(7),
            crate::LayoutInputViewOf::LineBreak(line_break)
        );
        assert_eq!(tree.node_input(7), &NodeInputOf::non_box());
    }

    #[test]
    fn fri06_c13_t01_public_tree_preserves_typed_inputs_and_child_order_in_both_scalar_lanes() {
        assert_fri06_c13_t01_public_tree_typed_contract::<f32>();
        assert_fri06_c13_t01_public_tree_typed_contract::<f64>();
    }

    fn final_output<S: LayoutScalar>(
        batch: &crate::CompletedLayoutBatchOf<u32, S>,
        node: u32,
    ) -> NodeOutputOf<S> {
        readback::single_box_output(batch, node, Phase::Final)
    }

    fn assert_fri06_c13_t01_public_layout_and_measurement<S: LayoutScalar>() {
        let fixed_size = Size::new(S::from_f64(31.0), S::from_f64(29.0));
        let ordinary = PublicLayoutTreeOf::new().style(
            1,
            NodeInputOf {
                size: fixed_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
        assert!(!ordinary.has_leaf_measurement(1));

        let request = || {
            let viewport = Size::splat(
                crate::NonNegativeFiniteOf::new(S::from_f64(100.0))
                    .expect("the declared native viewport is finite and nonnegative"),
            );
            LayoutRootRequestOf::standalone(
                crate::SizingRequestOf::new(
                    Size::splat(crate::AxisSizingRequestOf::new(
                        crate::AvailableSpaceOf::Definite(viewport.width),
                        crate::SizingOperation::Normal,
                    )),
                    crate::OrthogonalFallbackOf::new(
                        viewport.map(Some),
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                viewport.map(Some),
            )
        };
        let ordinary_batch =
            compute_layout(&ordinary, 1, request()).expect("ordinary fixed box lays out");
        assert_eq!(final_output(&ordinary_batch, 1).size, fixed_size);

        let measured_size = Size::new(S::from_f64(19.0), S::from_f64(23.0));
        let measured = PublicLayoutTreeOf::new()
            .style(2, NodeInputOf::default())
            .measure(2, measured_size);
        assert!(measured.has_leaf_measurement(2));

        let measured_request = || {
            LayoutRootRequestOf::standalone(
                crate::SizingRequestOf::new(
                    Size::splat(crate::AxisSizingRequestOf::new(
                        crate::AvailableSpaceOf::Indefinite,
                        crate::SizingOperation::Normal,
                    )),
                    crate::OrthogonalFallbackOf::new(
                        Size::new(None, None),
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, None),
            )
        };
        let first =
            compute_layout(&measured, 2, measured_request()).expect("measured leaf lays out");
        let second =
            compute_layout(&measured, 2, measured_request()).expect("measurement is deterministic");
        assert_eq!(final_output(&first, 2).size, measured_size);
        assert_eq!(final_output(&second, 2), final_output(&first, 2));
    }

    #[test]
    fn fri06_c13_t01_public_tree_preserves_measurement_absence_presence_and_public_layout() {
        assert_fri06_c13_t01_public_layout_and_measurement::<f32>();
        assert_fri06_c13_t01_public_layout_and_measurement::<f64>();
    }

    #[test]
    #[should_panic(expected = "public layout node 1 must define a layout input")]
    fn fri06_c13_t01_public_tree_missing_input_preserves_panic_boundary() {
        let tree = PublicLayoutTreeOf::<f32>::new();

        let _ = tree.node_input(1);
    }

    fn fri06_mr01_oracle_generic_input<S: LayoutScalar>() -> ComputeInputOf<S> {
        // This neutral scalar query has no supplied ICB, scrollport or local-bound
        // authority. Missing fallback facts are not inferred from its query space.
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            crate::SizingRequestOf::new(
                Size::splat(crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    Size::new(None, None),
                    crate::NearestScrollportOf::Unknown,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        )
    }

    fn assert_fri06_mr01_oracle_generic_records_and_prefers_measurement<S: LayoutScalar>()
    where
        OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
    {
        let style = NodeInputOf::<S> {
            display: Display::Grid,
            ..NodeInputOf::default()
        };
        let expected =
            ComputeOutputOf::from_outer_size(Size::new(S::from_f64(17.0), S::from_f64(23.0)));
        let input = fri06_mr01_oracle_generic_input();
        let mut tree = OracleTreeOf::new()
            .style(1, style.clone())
            .measure(1, expected);

        assert_eq!(tree.node_input(1), &style);
        assert_eq!(tree.layout_input(1), crate::LayoutInputViewOf::Box(&style));
        assert_eq!(
            tree.borrow_compute().compute_child(1, input).unwrap(),
            expected
        );
        assert_eq!(tree.inputs(1), &[input]);
        assert_eq!(tree.layout(1), None);
    }

    #[test]
    fn fri06_mr01_oracle_generic_records_inputs_and_prefers_measurements_in_both_scalar_lanes() {
        assert_fri06_mr01_oracle_generic_records_and_prefers_measurement::<f32>();
        assert_fri06_mr01_oracle_generic_records_and_prefers_measurement::<f64>();
    }

    fn assert_fri06_mr01_oracle_generic_hidden_and_dispatch<S: LayoutScalar>()
    where
        OracleTreeOf<S>: Traverse<Node = u32, Scalar = S>,
    {
        let input = fri06_mr01_oracle_generic_input();
        let mut hidden = OracleTreeOf::new().style(
            1,
            NodeInputOf {
                display: Display::None,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            hidden.borrow_compute().compute_child(1, input).unwrap(),
            ComputeOutputOf::HIDDEN
        );
        assert_eq!(hidden.inputs(1), &[input]);
        assert_eq!(
            hidden.layout(1),
            Some(NodeOutputOf::with_source_index(SourceIndex::ZERO))
        );

        for (index, display) in [
            Display::Block,
            Display::Flex,
            Display::Grid,
            Display::GridLanes,
        ]
        .into_iter()
        .enumerate()
        {
            let node = u32::try_from(index + 2).unwrap();
            let expected_size = Size::new(S::from_f64(31.0), S::from_f64(29.0));
            let mut tree = OracleTreeOf::new().style(
                node,
                NodeInputOf {
                    display,
                    size: Size::new(
                        PreferredSizeOf::px(expected_size.width),
                        PreferredSizeOf::px(expected_size.height),
                    ),
                    ..NodeInputOf::default()
                },
            );

            let output = tree.borrow_compute().compute_child(node, input).unwrap();

            assert_eq!(tree.inputs(node), &[input]);
            assert_eq!(output.size, expected_size, "unexpected {display:?} output");
        }
    }

    #[test]
    fn fri06_mr01_oracle_generic_stages_hidden_and_dispatches_algorithms_in_both_scalar_lanes() {
        assert_fri06_mr01_oracle_generic_hidden_and_dispatch::<f32>();
        assert_fri06_mr01_oracle_generic_hidden_and_dispatch::<f64>();
    }

    #[test]
    fn output_returns_none_without_a_staged_layout() {
        let tree = OracleTree::new();

        assert_eq!(tree.output(41), None);
    }

    #[test]
    fn rounding_missing_unrounded_output_returns_typed_error() {
        let mut tree = OracleTree::new();

        let error = crate::round_layout(&mut tree.borrow_compute(), 41)
            .expect_err("rounding without staged output must fail instead of synthesizing output");

        assert_eq!(error.site(), LayoutErrorSite::Node(41));
        assert_eq!(error.operation(), LayoutOperation::RoundingFinalization);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKind::InternalInvariant(
                LayoutInternalInvariant::MissingStagedUnroundedOutput,
            ),
        );
        assert_eq!(tree.final_layout(41), None);
    }

    #[test]
    fn layout_input_returns_declared_line_break() {
        let input = LineBreakInput::new().hidden();
        let tree = OracleTree::new().line_break(1, input);

        assert_eq!(
            tree.layout_input(1),
            crate::LayoutInputViewOf::LineBreak(input)
        );
        assert_eq!(
            tree.layout_input(1).as_line_break().unwrap().display(),
            LineBreakDisplay::None
        );
    }

    #[test]
    #[should_panic(expected = "line break node has no box NodeInput")]
    fn node_input_panics_for_line_break_node() {
        let tree = OracleTree::new().line_break(1, LineBreakInput::new());

        let _ = tree.node_input(1);
    }

    #[test]
    #[should_panic(expected = "oracle node 1 must define a layout input")]
    fn node_input_panics_for_missing_node() {
        let tree = OracleTree::new();

        let _ = tree.node_input(1);
    }
}
