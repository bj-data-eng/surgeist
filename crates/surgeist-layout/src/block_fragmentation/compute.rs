#[path = "scope_compute.rs"]
mod scoped;
use super::source::{Inventory, unsupported};
use super::{BoundMode, FragmentBoxAllocation};
use crate::geometry::LogicalSizeOf;
use crate::output::{GeometryStateOf, NormalGeometryOf};
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

type NativeResult<Tree, T> = LayoutResultOf<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

type MinimumSnapshot<Tree> = Vec<(<Tree as Traverse>::Node, <Tree as Traverse>::Scalar)>;

type IntrinsicContext<Tree> = Arc<
    crate::block::intrinsic::FragmentContext<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
>;

struct NativeProvider<'a, 'tree, Tree: LayoutTree> {
    capture: &'a LayoutFlowInput<'tree, Tree>,
    inventory: &'a Inventory<Tree::Node>,
    frames: &'a [BlockFragmentainerOf<Tree::Scalar>],
    axes: FlowAxes,
    kind: FragmentationKind,
    target: LayoutTargetBasis,
    template: LayoutRootRequestOf<Tree::Scalar>,
    intrinsics: Rc<RefCell<Vec<IntrinsicContext<Tree>>>>,
}
impl<Tree: LayoutTree> NativeProvider<'_, '_, Tree> {
    fn failure(
        &self,
        failure: BlockFragmentationFailure,
    ) -> LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(self.capture.root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::BlockFragmentation(failure),
        )
    }
    fn request(
        &self,
        frame: BlockFragmentainerOf<Tree::Scalar>,
    ) -> LayoutRootRequestOf<Tree::Scalar> {
        let inline = self.axes.logical_size(frame.size).inline;
        let axes = self.axes.physical_size(LogicalSizeOf::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    NonNegativeFiniteOf::new(inline)
                        .expect("bound fragmentainer geometry is validated"),
                ),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ));
        let LayoutRootContextOf::Standalone {
            percentage_parent, ..
        } = self.template.context()
        else {
            unreachable!("native admission qualified the standalone root context")
        };
        let original = self.axes.logical_size(percentage_parent);
        let parent = self.axes.physical_size(LogicalSizeOf::new(
            Some(NonNegativeFiniteOf::new(inline).expect("bound inline constraint is validated")),
            original.block,
        ));
        LayoutRootRequestOf::standalone(
            SizingRequestOf::new(axes, self.template.sizing_request().orthogonal_fallback()),
            self.axes,
            parent,
        )
        .with_rounding_mode(self.template.rounding_mode())
    }
    fn intrinsics(
        &self,
        frame: BlockFragmentainerOf<Tree::Scalar>,
    ) -> NativeResult<Tree, IntrinsicContext<Tree>> {
        let mut request = self.request(frame);
        let fallback = request.sizing_request().orthogonal_fallback();
        let first = self.frames.first().ok_or_else(|| {
            self.failure(BlockFragmentationFailure::MissingFragmentainer { index: 0 })
        })?;
        let icb = first.size.map(|value| {
            Some(NonNegativeFiniteOf::new(value).expect("bound first fragmentainer is validated"))
        });
        request = LayoutRootRequestOf::standalone(
            request
                .sizing_request()
                .with_orthogonal_fallback(OrthogonalFallbackOf::new(
                    icb,
                    fallback.nearest_scrollport(),
                    fallback.containing_limits(),
                )),
            self.axes,
            match request.context() {
                LayoutRootContextOf::Standalone {
                    percentage_parent, ..
                } => percentage_parent,
                _ => unreachable!("native request is standalone"),
            },
        )
        .with_rounding_mode(request.rounding_mode());
        if let Some(context) = self
            .intrinsics
            .borrow()
            .iter()
            .find(|context| context.request == request)
        {
            return Ok(context.clone());
        }
        let context = Arc::new(crate::engine::prepare_block_fragment_intrinsics(
            self.capture.tree,
            self.capture.root,
            request,
            self.target.clone(),
            self.axes,
        )?);
        self.intrinsics.borrow_mut().push(context.clone());
        Ok(context)
    }

    fn normal(
        entry: LayoutOutputEntryOf<Tree::Node, Tree::Scalar>,
    ) -> NormalGeometryOf<Tree::Scalar> {
        match entry.geometry_state() {
            GeometryStateOf::OrdinaryUnrounded { measured_normal } => measured_normal,
            GeometryStateOf::OrdinaryCanonical { normal }
            | GeometryStateOf::CanonicalAtomic { normal, .. } => normal,
        }
    }
}

pub(crate) fn compute<Tree: LayoutTree>(
    capture: &LayoutFlowInput<'_, Tree>,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
    let bound = capture
        .bound
        .as_ref()
        .expect("native dispatch requires its bound request");
    let BoundMode::Block { kind, axes, frames } = bound.mode() else {
        return Err(unsupported(root));
    };
    if !capture.paragraphs.is_empty()
        || !matches!(
            bound.root_request().context(),
            LayoutRootContextOf::Standalone { .. }
        )
        || bound
            .root_request()
            .sizing_request()
            .axes()
            .width
            .operation()
            != SizingOperation::Normal
        || bound
            .root_request()
            .sizing_request()
            .axes()
            .height
            .operation()
            != SizingOperation::Normal
    {
        return Err(unsupported(root));
    }
    let inventory = Inventory::new(capture.tree, &capture.graph, *axes)?;
    let scopes = super::scopes::Scopes::new(inventory, capture.tree, *axes)?;
    let provider = NativeProvider {
        capture,
        inventory: &scopes.source,
        frames,
        axes: *axes,
        kind: *kind,
        target: bound.target_basis(),
        template: bound.root_request(),
        intrinsics: Rc::new(RefCell::new(Vec::new())),
    };
    scoped::compute(
        &provider,
        &scopes,
        capture.graph.invalidation(changed_nodes)?,
    )
}
