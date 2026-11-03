//! Native family identities and borrowed accepted execution observations.
use super::{NativeExecutedEdge, NativeFamilyData, NativeFamilyRow, NativeLineReceipt};
use crate::engine::validation::ValidationGraph;
use crate::inline_wrapping::{edge_plan::AdmittedGeneratedBoxProof, ordering::NativeOrderingData};
use crate::{FlowAxes, GeneratedBoxRef, InlineBoundaryKind, LayoutScalar, WrappingScopeError};
use std::{fmt, ops::Range, sync::Arc};

/// One requested formatter's immutable, complete native inline-box family.
/// Identity binds the actual admitted graph, original proof and ordering profile.
#[derive(Clone)]
pub struct NativeInlineBoxFamily<Node> {
    formatter: Node,
    formatter_index: usize,
    graph: Arc<ValidationGraph<Node>>,
    proof: Arc<AdmittedGeneratedBoxProof<Node>>,
    profile: Arc<NativeOrderingData>,
}
impl<Node: fmt::Debug> fmt::Debug for NativeInlineBoxFamily<Node> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeInlineBoxFamily")
            .field("formatter", &self.formatter)
            .field(
                "fragments",
                &self
                    .profile
                    .family()
                    .map(|data| data.components.components().len()),
            )
            .finish()
    }
}
impl<Node: PartialEq> PartialEq for NativeInlineBoxFamily<Node> {
    fn eq(&self, other: &Self) -> bool {
        self.formatter == other.formatter
            && Arc::ptr_eq(&self.graph, &other.graph)
            && Arc::ptr_eq(&self.proof, &other.proof)
            && Arc::ptr_eq(&self.profile, &other.profile)
    }
}
impl<Node: Eq> Eq for NativeInlineBoxFamily<Node> {}
impl<Node: Copy + Eq> NativeInlineBoxFamily<Node> {
    pub(crate) fn new(
        formatter: Node,
        graph: Arc<ValidationGraph<Node>>,
        proof: Arc<AdmittedGeneratedBoxProof<Node>>,
        profile: Arc<NativeOrderingData>,
    ) -> Self {
        let formatter_index = graph
            .index(formatter)
            .expect("family admission proved the actual formatter");
        Self {
            formatter,
            formatter_index,
            graph,
            proof,
            profile,
        }
    }
    #[must_use]
    pub fn formatter(&self) -> Node {
        self.formatter
    }
    pub(crate) fn profile(&self) -> &Arc<NativeOrderingData> {
        &self.profile
    }
    pub(crate) fn data(&self) -> &NativeFamilyData {
        self.profile
            .family()
            .expect("only demanded admitted profiles issue families")
    }
    pub(crate) fn identity(&self, key: usize) -> GeneratedBoxRef<Node> {
        self.proof.identity(key).clone()
    }
    pub(crate) fn node_at_rank(&self, rank: usize) -> Node {
        self.graph.nodes[self.graph.children[self.formatter_index][rank]]
    }
    pub(crate) fn fragment(&self, index: usize) -> NativeInlineBoxFragment<Node> {
        NativeInlineBoxFragment {
            family: self.clone(),
            index,
        }
    }
    /// Global components of complete bodies, independently of selected lines.
    pub fn fragments(&self) -> impl ExactSizeIterator<Item = NativeInlineBoxFragment<Node>> + '_ {
        (0..self.data().components.components().len()).map(|index| self.fragment(index))
    }
}

/// Stable global fragment identity within one authentic native family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeInlineBoxFragment<Node> {
    family: NativeInlineBoxFamily<Node>,
    index: usize,
}
impl<Node: Copy + Eq> NativeInlineBoxFragment<Node> {
    #[must_use]
    pub fn family(&self) -> &NativeInlineBoxFamily<Node> {
        &self.family
    }
    #[must_use]
    pub fn owner(&self) -> GeneratedBoxRef<Node> {
        self.family.identity(self.component().owner())
    }
    fn component(&self) -> surgeist_text::SuppliedInlineBoxComponent<'_> {
        self.family
            .data()
            .components
            .component(self.index)
            .expect("issued fragment index")
    }
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.component()
            .parent_index()
            .map(|index| self.family.fragment(index))
    }
    #[must_use]
    pub fn body_root(&self) -> Self {
        let mut index = self.index;
        while let Some(parent) = self
            .family
            .data()
            .components
            .component(index)
            .expect("issued component ancestry")
            .parent_index()
        {
            index = parent;
        }
        self.family.fragment(index)
    }
    /// Refuses foreign families and distinct body roots before traversing ancestry.
    pub fn nearest_common_ancestor(&self, other: &Self) -> Result<Self, WrappingScopeError> {
        if self.family != other.family {
            return Err(WrappingScopeError::DifferentFamily);
        }
        if self.component().body() != other.component().body() {
            return Err(WrappingScopeError::DifferentBody);
        }
        let mut left = Some(self.index);
        while let Some(index) = left {
            let mut right = Some(other.index);
            while let Some(candidate) = right {
                if index == candidate {
                    return Ok(self.family.fragment(index));
                }
                right = self
                    .family
                    .data()
                    .components
                    .component(candidate)
                    .expect("admitted component ancestry")
                    .parent_index();
            }
            left = self
                .family
                .data()
                .components
                .component(index)
                .expect("admitted component ancestry")
                .parent_index();
        }
        Err(WrappingScopeError::InconsistentIncidence)
    }
}

/// One genuine selected membership. Empty boxes have no invented operand/node.
#[derive(Clone, Copy)]
pub struct NativeInlineBoxMemberView<'a, Node> {
    family: &'a NativeInlineBoxFamily<Node>,
    row: NativeFamilyRow,
}
impl<Node: Copy + Eq> NativeInlineBoxMemberView<'_, Node> {
    #[must_use]
    pub fn node(&self) -> Option<Node> {
        self.child_rank().map(|rank| self.family.node_at_rank(rank))
    }
    #[must_use]
    pub fn child_rank(&self) -> Option<usize> {
        match self.row {
            NativeFamilyRow::Operand { rank, .. } => Some(rank),
            _ => None,
        }
    }
    /// Logical supplied-operand ordinal in the complete formatter, not visual rank.
    /// Structural metadata and body delimiters do not consume an operand ordinal.
    #[must_use]
    pub fn operand_ordinal(&self) -> Option<usize> {
        match self.row {
            NativeFamilyRow::Operand { ordinal, .. } => Some(ordinal),
            _ => None,
        }
    }
    #[must_use]
    pub fn empty_box(&self) -> Option<GeneratedBoxRef<Node>> {
        match self.row {
            NativeFamilyRow::Empty { owner, .. } => Some(self.family.identity(owner)),
            _ => None,
        }
    }
}
impl<Node: Copy + Eq + fmt::Debug> fmt::Debug for NativeInlineBoxMemberView<'_, Node> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeInlineBoxMemberView")
            .field("node", &self.node())
            .field("child_rank", &self.child_rank())
            .field("operand_ordinal", &self.operand_ordinal())
            .field("empty_box", &self.empty_box())
            .finish()
    }
}

/// An actual source alias or executed BoxEdge association. Scalar coordinates
/// are in the line's executed logical inline frame, including alignment/indent.
#[derive(Clone, Copy)]
pub struct NativeInlineBoxEdgeView<'a, Node, S: LayoutScalar> {
    family: &'a NativeInlineBoxFamily<Node>,
    axes: FlowAxes,
    edge: &'a NativeExecutedEdge<S>,
}
impl<Node: Copy + Eq, S: LayoutScalar> NativeInlineBoxEdgeView<'_, Node, S> {
    #[must_use]
    pub fn node(&self) -> Node {
        self.family.node_at_rank(self.edge.rank)
    }
    #[must_use]
    pub fn child_rank(&self) -> usize {
        self.edge.rank
    }
    #[must_use]
    pub fn owner(&self) -> GeneratedBoxRef<Node> {
        self.family.identity(self.edge.owner)
    }
    #[must_use]
    pub fn kind(&self) -> InlineBoundaryKind {
        self.edge.kind
    }
    #[must_use]
    pub fn is_source_alias(&self) -> bool {
        self.edge.alias
    }
    /// Actual selected source-alias point in the executed logical inline frame.
    #[must_use]
    pub fn source_alias_position(&self) -> Option<S> {
        self.edge.alias.then_some(self.edge.flow.0)
    }
    /// Axes of the executed formatter-logical inline coordinate frame.
    #[must_use]
    pub fn flow_axes(&self) -> FlowAxes {
        self.axes
    }
    #[must_use]
    pub fn flow_interval(&self) -> Option<(S, S)> {
        (!self.edge.alias).then_some(self.edge.flow)
    }
    #[must_use]
    pub fn strip_interval(&self) -> Option<(S, S)> {
        (!self.edge.alias).then_some(self.edge.strip)
    }
}
impl<Node: Copy + Eq + fmt::Debug, S: LayoutScalar> fmt::Debug
    for NativeInlineBoxEdgeView<'_, Node, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeInlineBoxEdgeView")
            .field("node", &self.node())
            .field("kind", &self.kind())
            .field("alias", &self.edge.alias)
            .field("flow", &self.flow_interval())
            .field("strip", &self.strip_interval())
            .finish()
    }
}

/// A selected line piece borrowing one global family and its actual execution.
/// This is native incidence, not a relabeled Text fragment or a paint rectangle.
#[derive(Clone, Copy)]
pub struct NativeInlineBoxPieceView<'a, Node, S: LayoutScalar> {
    family: &'a NativeInlineBoxFamily<Node>,
    run_source: &'a Range<usize>,
    line: &'a NativeLineReceipt<S>,
    component: usize,
    members: &'a [usize],
    edge_indices: &'a [usize],
}
impl<'a, Node: Copy + Eq, S: LayoutScalar> NativeInlineBoxPieceView<'a, Node, S> {
    #[must_use]
    pub fn family(&self) -> &'a NativeInlineBoxFamily<Node> {
        self.family
    }
    #[must_use]
    pub fn fragment(&self) -> NativeInlineBoxFragment<Node> {
        self.family.fragment(self.component)
    }
    #[must_use]
    /// Actual selected-line ordinal within this fitting run.
    pub fn line_index(&self) -> usize {
        self.line.line
    }
    /// Actual fitting-run fences; line_index() is local to this run.
    #[must_use]
    pub fn run_source_range(&self) -> Range<usize> {
        self.run_source.clone()
    }
    /// Actual raw formatter-child fences retained by the issued run projection.
    #[must_use]
    pub fn selected_source_range(&self) -> Range<usize> {
        self.line.source.clone()
    }
    #[must_use]
    pub fn flow_axes(&self) -> FlowAxes {
        self.line.axes
    }
    pub fn selected_members(
        &self,
    ) -> impl Iterator<Item = NativeInlineBoxMemberView<'a, Node>> + 'a {
        let family = self.family;
        self.members
            .iter()
            .copied()
            .map(move |row| NativeInlineBoxMemberView {
                family,
                row: family.data().rows[row],
            })
    }
    pub fn source_aliases(
        &self,
    ) -> impl Iterator<Item = NativeInlineBoxEdgeView<'a, Node, S>> + 'a {
        self.edges(true)
    }
    pub fn executed_edges(
        &self,
    ) -> impl Iterator<Item = NativeInlineBoxEdgeView<'a, Node, S>> + 'a {
        self.edges(false)
    }
    fn edges(
        &self,
        alias: bool,
    ) -> impl Iterator<Item = NativeInlineBoxEdgeView<'a, Node, S>> + 'a {
        let family = self.family;
        let line = self.line;
        self.edge_indices
            .iter()
            .map(move |&index| &line.edges[index])
            .filter(move |edge| edge.alias == alias)
            .map(move |edge| NativeInlineBoxEdgeView {
                family,
                axes: line.axes,
                edge,
            })
    }
}
impl<Node: Copy + Eq + fmt::Debug, S: LayoutScalar> fmt::Debug
    for NativeInlineBoxPieceView<'_, Node, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeInlineBoxPieceView")
            .field("formatter", &self.family.formatter)
            .field("component", &self.component)
            .field("line", &self.line.line)
            .field("source", &self.line.source)
            .finish()
    }
}

/// A run's accepted immutable receipt storage; indices borrow the batch's one
/// family-slot vector. There is no strong family owner per piece/member/edge.
#[derive(Clone)]
pub(crate) struct NativePublication<S: LayoutScalar> {
    pub(crate) family: usize,
    pub(crate) source: Range<usize>,
    pub(crate) completion: Arc<crate::inline_wrapping::ordering::NativeOrderingProjection>,
    pub(crate) lines: Vec<NativeLineReceipt<S>>,
    pieces: Vec<NativePieceRecord>,
    members: Vec<usize>,
    edges: Vec<usize>,
}
impl<S: LayoutScalar> fmt::Debug for NativePublication<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativePublication")
            .field("family", &self.family)
            .field("source", &self.source)
            .field("lines", &self.lines.len())
            .finish()
    }
}
#[derive(Clone, Debug)]
struct NativePieceRecord {
    line: usize,
    component: usize,
    members: Range<usize>,
    edges: Range<usize>,
}
impl<S: LayoutScalar> NativePublication<S> {
    pub(crate) fn new(
        family: usize,
        source: Range<usize>,
        completion: Arc<crate::inline_wrapping::NativeOrderingProjection>,
        lines: Vec<NativeLineReceipt<S>>,
    ) -> Result<Self, WrappingScopeError> {
        let mut pieces = Vec::new();
        let mut members = Vec::new();
        let mut edges = Vec::new();
        // One transient association buffer, reused per line. It carries only
        // canonical indices; no source, forest or geometry is derived/copied.
        let mut associations = Vec::new();
        let data = completion
            .profile
            .family()
            .ok_or(WrappingScopeError::InconsistentIncidence)?;
        for (line_index, line) in lines.iter().enumerate() {
            associations.clear();
            for (ordinal, &row) in line.rows.iter().enumerate() {
                let fact = data
                    .rows
                    .get(row)
                    .ok_or(WrappingScopeError::InconsistentIncidence)?;
                match fact {
                    NativeFamilyRow::Metadata => continue,
                    NativeFamilyRow::Operand { body, .. } | NativeFamilyRow::Empty { body, .. }
                        if *body == line.body => {}
                    _ => return Err(WrappingScopeError::InconsistentIncidence),
                }
                for component in data.components.row_components(row) {
                    super::push(&mut associations, (component, 0u8, ordinal, row))?;
                }
            }
            for (index, edge) in line.edges.iter().enumerate() {
                let component = data
                    .components
                    .component(edge.component)
                    .ok_or(WrappingScopeError::InconsistentIncidence)?;
                if component.body() != line.body || component.owner() != edge.owner {
                    return Err(WrappingScopeError::InconsistentIncidence);
                }
                super::push(&mut associations, (edge.component, 1u8, index, index))?;
            }
            // Unstable ordering is in-place: no hidden heap sort scratch.
            // The ordinal preserves actual executed member/edge order.
            associations
                .sort_unstable_by_key(|&(component, kind, ordinal, _)| (component, kind, ordinal));
            let mut at = 0;
            while at < associations.len() {
                let component = associations[at].0;
                let member_start = members.len();
                let edge_start = edges.len();
                while at < associations.len() && associations[at].0 == component {
                    let (_, kind, _, value) = associations[at];
                    if kind == 0 {
                        super::push(&mut members, value)?;
                    } else {
                        super::push(&mut edges, value)?;
                    }
                    at += 1;
                }
                super::push(
                    &mut pieces,
                    NativePieceRecord {
                        line: line_index,
                        component,
                        members: member_start..members.len(),
                        edges: edge_start..edges.len(),
                    },
                )?;
            }
        }
        Ok(Self {
            family,
            source,
            completion,
            lines,
            pieces,
            members,
            edges,
        })
    }
    pub(crate) fn try_clone(&self) -> Result<Self, WrappingScopeError> {
        let mut pieces = Vec::new();
        pieces
            .try_reserve_exact(self.pieces.len())
            .map_err(|_| WrappingScopeError::Allocation)?;
        pieces.extend_from_slice(&self.pieces);
        Ok(Self {
            family: self.family,
            source: self.source.clone(),
            completion: self.completion.clone(),
            lines: clone_native_lines(&self.lines)?,
            pieces,
            members: copy_buffer(&self.members)?,
            edges: copy_buffer(&self.edges)?,
        })
    }
}
pub(crate) fn pieces<'a, Node: Copy + Eq, S: LayoutScalar>(
    families: &'a [NativeInlineBoxFamily<Node>],
    runs: &'a [NativePublication<S>],
) -> impl Iterator<Item = NativeInlineBoxPieceView<'a, Node, S>> + 'a {
    runs.iter().flat_map(move |run| {
        let family = &families[run.family];
        run.pieces
            .iter()
            .map(move |piece| NativeInlineBoxPieceView {
                family,
                run_source: &run.source,
                line: &run.lines[piece.line],
                component: piece.component,
                members: &run.members[piece.members.clone()],
                edge_indices: &run.edges[piece.edges.clone()],
            })
    })
}
fn copy_buffer<T: Copy>(source: &[T]) -> Result<Vec<T>, WrappingScopeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(source.len())
        .map_err(|_| WrappingScopeError::Allocation)?;
    result.extend_from_slice(source);
    Ok(result)
}
fn clone_native_lines<S: LayoutScalar>(
    lines: &[NativeLineReceipt<S>],
) -> Result<Vec<NativeLineReceipt<S>>, WrappingScopeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(lines.len())
        .map_err(|_| WrappingScopeError::Allocation)?;
    for line in lines {
        result.push(NativeLineReceipt {
            profile: line.profile.clone(),
            source: line.source.clone(),
            body: line.body,
            line: line.line,
            axes: line.axes,
            receipt: line.receipt.clone(),
            ranks: copy_buffer(&line.ranks)?,
            rows: copy_buffer(&line.rows)?,
            edges: copy_buffer(&line.edges)?,
        });
    }
    Ok(result)
}

/// Actual successful formatter computation, including a genuine empty formatter
/// that establishes no line/run. Admission alone never creates this receipt.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeFormatterCompletion<S: LayoutScalar> {
    pub(crate) family: usize,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) output: crate::ComputeOutputOf<S>,
}
