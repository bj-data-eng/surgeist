//! Resolve authentic capture facts against one immutable Layout graph.

use std::collections::{HashMap, HashSet};

use super::{
    TextBindingError as E, TextParagraphBinding, TextScopeAssociation, TextScopeBinding,
    binding_error,
};
use crate::engine::validation::ValidationGraph;
use crate::inline_wrapping::{
    CanonicalBox, CanonicalCoverage, CanonicalOrderBindings, CanonicalOrderInventory,
    CanonicalScopeProof, WrapInside, WrappingScopeError, WrappingScopeInput, WrappingScopeProof,
};
use surgeist_text::{
    CapturedControl, CapturedDomainDefinition, CapturedEventPosition, CapturedFlowEvent as Event,
    CapturedParagraph, CapturedScope, PreparedScope, PreparedText, ScopeRole,
};

type ResultOf<Tree, T> = crate::LayoutResultOf<
    <Tree as crate::Traverse>::Node,
    T,
    <Tree as crate::Traverse>::Scalar,
    <Tree as crate::LayoutTree>::MeasureError,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParagraphClass {
    Flow,
    SemanticFlow,
    AssociationOnly,
}

pub(crate) fn classify<Node>(paragraph: &TextParagraphBinding<Node>) -> Result<ParagraphClass, E> {
    if !paragraph.nodes.is_empty() {
        return Ok(ParagraphClass::Flow);
    }
    let computed_only = paragraph.prepared.captured_owner().is_some()
        && paragraph.prepared.origin_views().next().is_none()
        && paragraph.prepared.control_views().next().is_none()
        && paragraph.prepared.atomic_views().next().is_none()
        && paragraph.prepared.virtual_separators().next().is_none()
        && paragraph.origins.is_empty()
        && paragraph.controls.is_empty()
        && paragraph.atomics.is_empty();
    if computed_only {
        let mut computed = false;
        let mut forced = paragraph
            .prepared
            .opportunity_views()
            .filter(|offer| offer.kind() == surgeist_text::PreparedBreakKind::Forced)
            .map(|offer| offer.computed_line_boundary());
        let all_computed = paragraph.prepared.participants().all(|participant| {
            let surgeist_text::PreparedParticipant::ComputedLineBoundary(boundary) = participant
            else {
                return false;
            };
            computed = true;
            forced.next().flatten().as_ref() == Some(&boundary)
        });
        if computed && all_computed && forced.next().is_none() {
            return Ok(ParagraphClass::SemanticFlow);
        }
    }
    if paragraph.prepared.captured_owner().is_some()
        && paragraph.prepared.origin_views().next().is_none()
        && paragraph.prepared.control_views().next().is_none()
        && paragraph.prepared.atomic_views().next().is_none()
        && paragraph.prepared.participants().next().is_none()
        && paragraph.prepared.scopes().next().is_none()
        && paragraph.prepared.virtual_separators().next().is_none()
        && paragraph.origins.is_empty()
        && paragraph.controls.is_empty()
        && paragraph.atomics.is_empty()
        && paragraph.line_strut.is_none()
    {
        Ok(ParagraphClass::AssociationOnly)
    } else {
        Err(E::EmptyRun)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopeAssociations<Node> {
    inline: Vec<InlineScope<Node>>,
    paragraphs: Vec<ParagraphResolution<Node>>,
    inline_boxes: super::fragmentation::InlineBoxFamilies,
    domains: Vec<Domain<Node>>,
    regions: Vec<CapturedScope>,
    structural: Vec<CapturedScope>,
    computed: Vec<ComputedIncidence>,
    slots: Vec<SemanticParagraphSlot<Node>>,
    runs: Vec<NativeRun>,
}

/// A normalized Text-issued structural segment and its actual native window.
/// An empty window is a real source segment at the proved native entry gap.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeRun {
    pub(crate) paragraph: usize,
    pub(crate) nodes: std::ops::Range<usize>,
    pub(crate) children: std::ops::Range<usize>,
    pub(crate) segment: surgeist_text::PreparedSourceSegment,
}

#[derive(Clone)]
pub(crate) struct NativeRunSlot {
    pub(crate) gap: usize,
    pub(crate) segment: surgeist_text::PreparedSourceSegment,
}

/// A checked native gap is a coordinate, never a paragraph identity. Actual
/// capture/preparation handles retain distinct semantic occurrences at that gap.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SemanticParagraphSlot<Node> {
    pub(crate) captured: CapturedParagraph,
    pub(crate) prepared: PreparedText,
    pub(crate) container: Node,
    pub(crate) gap: usize,
    previous: Option<CapturedEventPosition>,
    next: Option<CapturedEventPosition>,
    key: (usize, u8),
}

#[derive(Clone, Debug, PartialEq)]
struct ComputedIncidence {
    boundary: surgeist_text::PreparedComputedLineBoundary,
    cut: ComputedNativeCut,
}

/// Inclusive bounds on possible gaps between actual run nodes. They are not a
/// computed-group identity; distinct groups can have the same native bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ComputedNativeCut {
    lower: usize,
    upper: usize,
}

impl ComputedNativeCut {
    pub(crate) fn enclosing(gaps: &[usize]) -> Option<Self> {
        Some(Self {
            lower: *gaps.iter().min()?,
            upper: *gaps.iter().max()?,
        })
    }

    pub(crate) fn relative_to(self, run: std::ops::Range<usize>) -> Option<Self> {
        let lower = self.lower.max(run.start);
        let upper = self.upper.min(run.end);
        (lower <= upper).then(|| Self {
            lower: lower - run.start,
            upper: upper - run.start,
        })
    }

    pub(crate) fn side_of_node(self, offset: usize) -> Option<std::cmp::Ordering> {
        if offset < self.lower {
            Some(std::cmp::Ordering::Less)
        } else if offset >= self.upper {
            Some(std::cmp::Ordering::Greater)
        } else {
            // In particular, a node ON the lower bound of a nonexact gap is
            // ambiguous: the cut may be immediately before or after that node.
            None
        }
    }
}

#[cfg(test)]
mod computed_cut_tests {
    use super::ComputedNativeCut;
    use std::cmp::Ordering::{Greater, Less};

    #[test]
    fn a_float_on_the_first_possible_gap_has_no_proven_side() {
        // Native nodes [left, float, right]: after-left gap1 and before-right
        // gap2 are both possible. The float at node1 is not strictly interior
        // to those bounds, but its side is still genuinely ambiguous.
        let uncertain = ComputedNativeCut { lower: 1, upper: 2 };
        assert_eq!(uncertain.side_of_node(0), Some(Less));
        assert_eq!(uncertain.side_of_node(1), None);
        assert_eq!(uncertain.side_of_node(2), Some(Greater));
        let after_left = ComputedNativeCut { lower: 1, upper: 1 };
        let before_right = ComputedNativeCut { lower: 2, upper: 2 };
        assert_eq!(after_left.side_of_node(1), Some(Greater));
        assert_eq!(before_right.side_of_node(1), Some(Less));
    }
}
#[derive(Clone, Debug, PartialEq)]
struct InlineScope<Node> {
    scope: CapturedScope,
    start: Node,
    end: Node,
}
#[derive(Clone, Debug, PartialEq)]
struct Domain<Node> {
    scope: CapturedScope,
    container: Node,
}
#[derive(Clone, Debug, PartialEq)]
struct ParagraphResolution<Node> {
    prepared: PreparedText,
    container: Node,
    class: ParagraphClass,
    scopes: Vec<(PreparedScope, Association)>,
}
#[derive(Clone, Debug, PartialEq)]
enum Association {
    Domain,
    Inline(usize),
}
impl<Node> Default for ScopeAssociations<Node> {
    fn default() -> Self {
        Self {
            inline: Vec::new(),
            paragraphs: Vec::new(),
            inline_boxes: super::fragmentation::InlineBoxFamilies::default(),
            domains: Vec::new(),
            regions: Vec::new(),
            structural: Vec::new(),
            computed: Vec::new(),
            slots: Vec::new(),
            runs: Vec::new(),
        }
    }
}
fn owns_member(prepared: &PreparedText, member: &CapturedParagraph) -> bool {
    prepared
        .captured_owner()
        .is_some_and(|owner| owner.members().any(|actual| actual == *member))
}

fn occurrence_member(
    prepared: &PreparedText,
    position: &CapturedEventPosition,
    atomic: bool,
) -> Option<CapturedParagraph> {
    let owner = prepared.captured_owner()?;
    let domain = owner.domain();
    let member = domain.flow_events()?.find_map(|event| match event {
        Event::Atomic {
            paragraph,
            position: actual,
        } if atomic && actual == *position => Some(paragraph),
        Event::Origin {
            paragraph,
            position: actual,
        } if !atomic && actual == *position => Some(paragraph),
        _ => None,
    })?;
    owner
        .members()
        .any(|actual| actual == member)
        .then_some(member)
}

impl<Node> ScopeAssociations<Node> {
    pub(crate) fn inline_boxes(&self, binding: usize) -> super::AdmittedInlineBoxes {
        self.inline_boxes.state(binding)
    }

    pub(crate) fn inline_box_families(&self) -> &super::fragmentation::InlineBoxFamilies {
        &self.inline_boxes
    }

    pub(crate) fn set_inline_box_families(
        &mut self,
        families: super::fragmentation::InlineBoxFamilies,
    ) {
        self.inline_boxes = families;
    }

    pub(crate) fn same_inline_box_families(&self, other: &Self) -> bool {
        self.inline_boxes.same_owner(&other.inline_boxes)
    }

    pub(crate) fn admit_inline_box_comparison<S: crate::LayoutScalar>(
        &self,
        other: &Self,
        limits: surgeist_text::ScopeAlignmentLimits,
        resources: &super::scope_alignment::ScopeAlignmentResources<S>,
    ) -> surgeist_text::Result<()> {
        self.inline_boxes
            .admit_comparison(&other.inline_boxes, limits, resources)
    }
}

impl<Node: Copy + Eq> ScopeAssociations<Node> {
    /// Exact canonical descriptor bound for a fallible caller to admit before
    /// the pure association search. No second lookup index is retained.
    pub(crate) fn captured_binding_work(&self, scope: &CapturedScope) -> usize {
        if scope.role() == ScopeRole::Domain {
            self.domains.len()
        } else {
            self.inline.len()
        }
    }

    pub(crate) fn captured_binding(
        &self,
        scope: &CapturedScope,
    ) -> Option<TextScopeAssociation<Node>> {
        if scope.role() == ScopeRole::Domain {
            self.domains
                .iter()
                .find(|row| &row.scope == scope)
                .map(|row| TextScopeAssociation::ParagraphContainer(row.container))
        } else {
            self.inline
                .iter()
                .find(|row| &row.scope == scope)
                .map(|row| TextScopeAssociation::Inline {
                    start: row.start,
                    end: row.end,
                })
        }
    }
    pub(crate) fn runs(&self) -> &[NativeRun] {
        &self.runs
    }

    /// Borrow one canonical admitted row with its original immutable binding
    /// inventory. LayoutFlowInput establishes this coupling before publication;
    /// the computation and its forks keep borrowing that same input family.
    pub(crate) fn admitted_run<'input>(
        &self,
        paragraphs: &'input [TextParagraphBinding<Node>],
        row: &NativeRun,
    ) -> super::AdmittedRun<'input, Node> {
        assert!(
            self.runs.iter().any(|actual| std::ptr::eq(actual, row)),
            "a native run is borrowed from this admitted inventory"
        );
        let binding = &paragraphs[row.paragraph];
        assert!(
            binding.prepared == self.paragraphs[row.paragraph].prepared
                && binding.container == self.paragraphs[row.paragraph].container
                && binding.inline_box_fragments == self.inline_boxes.state(row.paragraph).demand(),
            "the native run uses its original binding family"
        );
        super::AdmittedRun {
            binding_index: row.paragraph,
            binding,
            inventory: row.nodes.clone(),
            native: row.children.clone(),
            segment: row.segment.clone(),
            inline_boxes: self.inline_boxes.state(row.paragraph),
        }
    }

    pub(crate) fn semantic_slot(
        &self,
        prepared: &PreparedText,
    ) -> Option<&SemanticParagraphSlot<Node>> {
        self.slots.iter().find(|slot| &slot.prepared == prepared)
    }
    pub(crate) fn box_for_endpoint(
        &self,
        node: Node,
        mut inspect: impl FnMut() -> surgeist_text::Result<()>,
    ) -> surgeist_text::Result<Option<CapturedScope>> {
        for row in &self.inline {
            inspect()?;
            if row.start == node || row.end == node {
                return Ok(Some(row.scope.clone()));
            }
        }
        Ok(None)
    }

    pub(crate) fn native_pairs(&self) -> impl Iterator<Item = (&CapturedScope, (Node, Node))> {
        self.inline
            .iter()
            .map(|row| (&row.scope, (row.start, row.end)))
    }

    pub(crate) fn computed_cut(
        &self,
        boundary: &surgeist_text::PreparedComputedLineBoundary,
    ) -> Option<ComputedNativeCut> {
        self.computed
            .iter()
            .find(|row| &row.boundary == boundary)
            .map(|row| row.cut)
    }
    pub(crate) fn captured_scope_relation(
        &self,
        prepared: &PreparedText,
        scope: &CapturedScope,
    ) -> CapturedScopeRelation {
        let Some(paragraph) = self
            .paragraphs
            .iter()
            .find(|p| &p.prepared == prepared)
            .and_then(|p| p.prepared.captured_owner())
        else {
            return CapturedScopeRelation::Unavailable;
        };
        if paragraph.domain() != scope.domain()
            || !paragraph
                .domain()
                .enter_position()
                .same_capture(&scope.enter_position())
        {
            return CapturedScopeRelation::Unavailable;
        }
        if paragraph
            .members()
            .any(|member| member.scope_intersections().any(|i| i.scope() == *scope))
        {
            CapturedScopeRelation::Local
        } else {
            CapturedScopeRelation::Outside
        }
    }
    pub fn binding(&self, scope: &PreparedScope) -> Option<TextScopeBinding<Node>> {
        for paragraph in &self.paragraphs {
            if let Some((_, association)) = paragraph.scopes.iter().find(|(s, _)| s == scope) {
                let association = match association {
                    Association::Domain => {
                        TextScopeAssociation::ParagraphContainer(paragraph.container)
                    }
                    Association::Inline(index) => {
                        let row = &self.inline[*index];
                        TextScopeAssociation::Inline {
                            start: row.start,
                            end: row.end,
                        }
                    }
                };
                return Some(TextScopeBinding {
                    scope: scope.clone(),
                    association,
                });
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapturedScopeRelation {
    Local,
    Outside,
    Unavailable,
}

pub(super) struct CaptureResolution<Node> {
    pub associations: ScopeAssociations<Node>,
    pub wrapping: Vec<WrappingScopeProof<Node>>,
}
#[derive(Clone)]
struct NativeRegion<Node> {
    flow: Node,
    pair: (Node, Node),
    start: CapturedControl,
    end: CapturedControl,
    owner: usize,
    first: usize,
    last: usize,
    active: bool,
}
impl<Node> NativeRegion<Node> {
    fn domain(&self) -> CapturedScope {
        self.start.paragraph().domain()
    }
}
struct Control<Node> {
    captured: CapturedControl,
    node: Node,
    paragraph: usize,
}
struct Atomic<Node> {
    position: CapturedEventPosition,
    member: CapturedParagraph,
    node: Node,
    paragraph: usize,
    kind: surgeist_text::InlineBoxKind,
}
struct Origin<Node> {
    position: CapturedEventPosition,
    member: CapturedParagraph,
    node: Node,
    paragraph: usize,
}
enum ActualOwner<Node> {
    Physical(Node),
    Semantic {
        container: Node,
        paragraph: CapturedParagraph,
    },
}
struct Actual<Node> {
    event: Event,
    owner: ActualOwner<Node>,
    key: (usize, u8),
}
impl<Node: Copy> Actual<Node> {
    fn subject(&self) -> Node {
        match &self.owner {
            ActualOwner::Physical(node) => *node,
            ActualOwner::Semantic { container, .. } => *container,
        }
    }
}

#[derive(Default)]
struct NodeEvents {
    start: Vec<(CapturedScope, Event)>,
    body: Vec<(CapturedScope, Event)>,
    end: Vec<(CapturedScope, Event)>,
}
struct PositionedEvent {
    position: CapturedEventPosition,
    event: Event,
}
struct ExpectedFlow {
    events: Vec<PositionedEvent>,
    paragraphs: HashMap<CapturedParagraph, (usize, usize)>,
    previous: Vec<Option<usize>>,
    next: Vec<Option<usize>>,
}
#[derive(Clone, Copy)]
struct GapScan<'a> {
    previous: Option<&'a Event>,
    next: Option<&'a Event>,
    inside: bool,
    complete: bool,
}

struct Proof<'a, Tree: crate::LayoutTree> {
    tree: &'a Tree,
    graph: &'a ValidationGraph<Tree::Node>,
    paragraphs: &'a [TextParagraphBinding<Tree::Node>],
    result: ScopeAssociations<Tree::Node>,
    controls: Vec<Control<Tree::Node>>,
    atomics: Vec<Atomic<Tree::Node>>,
    origins: Vec<Origin<Tree::Node>>,
    scopes: Vec<CapturedScope>,
    domains: Vec<CapturedScope>,
    paths: Vec<u8>,
    enter: Vec<usize>,
    exit: Vec<usize>,
    depth: Vec<usize>,
    node_events: Vec<NodeEvents>,
    node_domains: Vec<Vec<usize>>,
    expected: HashMap<CapturedScope, ExpectedFlow>,
    paragraph_indices: HashMap<CapturedParagraph, usize>,
    metadata: HashSet<CapturedParagraph>,
    nearest: HashMap<CapturedScope, Option<CapturedScope>>,
    enclosing: HashMap<CapturedScope, Option<CapturedScope>>,
    semantic: Vec<CapturedScope>,
    known: Vec<CapturedScope>,
    wrapping: Vec<CapturedScope>,
    native: Vec<NativeRegion<Tree::Node>>,
    // Constructor demand only; actual Source/native body proof still follows.
    inline_box_bodies: bool,
    computed: Vec<(usize, surgeist_text::PreparedComputedLineBoundary)>,
    activation: Option<&'a mut super::scope_alignment::AdmissionWork>,
    activation_storage: ActivationStorage,
}

#[derive(Default)]
struct ActivationStorage {
    // Each receipt follows this actual canonical vector, never a parallel fact
    // index. Known/scopes and proof copies are scratch. Domains moves into the
    // retained structural inventory; regions/inline already belong to the
    // retained associations. Native graph, ExpectedFlow and slot-order buffers
    // are not transferred to this feature owner.
    known: super::scope_alignment::ScopeCensus,
    scopes: super::scope_alignment::ScopeCensus,
    domains: super::scope_alignment::ScopeCensus,
    regions: super::scope_alignment::ScopeCensus,
    inline: super::scope_alignment::ScopeCensus,
}

pub(super) fn activation_error<Tree: crate::LayoutTree>(
    node: Tree::Node,
    error: surgeist_text::Error,
) -> crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
    crate::LayoutErrorOf::new(
        crate::LayoutErrorSiteOf::Node(node),
        crate::LayoutOperation::TextCapture,
        crate::LayoutErrorKindOf::Text(error),
    )
}

fn activation_unique<Tree: crate::LayoutTree, T: PartialEq>(
    work: Option<&mut super::scope_alignment::AdmissionWork>,
    held: &mut super::scope_alignment::ScopeCensus,
    values: &mut Vec<T>,
    value: T,
    retained: bool,
    node: Tree::Node,
) -> ResultOf<Tree, ()> {
    let Some(work) = work else {
        push_unique(values, value);
        return Ok(());
    };
    for old in values.iter() {
        work.visits(1)
            .map_err(|error| activation_error::<Tree>(node, error))?;
        if old == &value {
            return Ok(());
        }
    }
    work.push(values, value, held, retained)
        .map_err(|error| activation_error::<Tree>(node, error))
}

pub(super) fn resolve<Tree: crate::LayoutTree>(
    tree: &Tree,
    graph: &ValidationGraph<Tree::Node>,
    paragraphs: &[TextParagraphBinding<Tree::Node>],
    wrapping_input: &crate::WrappingInput<Tree::Node>,
    inventory_scopes: &[CapturedScope],
    inline_box_bodies: bool,
    alignment: &mut super::scope_alignment::ScopeAlignmentAdmission<Tree::Scalar>,
) -> ResultOf<Tree, CaptureResolution<Tree::Node>> {
    if paragraphs.is_empty() {
        return Ok(CaptureResolution {
            associations: ScopeAssociations::default(),
            wrapping: Vec::new(),
        });
    }
    let wrapping_scopes = &wrapping_input.scopes;
    let canonical_orders = !wrapping_input.child_order.is_empty();
    let (alignment_inputs, activation) = alignment.capture_parts();
    let (enter, exit) = graph_order(graph);
    let mut proof = Proof {
        tree,
        graph,
        paragraphs,
        result: ScopeAssociations::default(),
        controls: Vec::new(),
        atomics: Vec::new(),
        origins: Vec::new(),
        scopes: Vec::new(),
        domains: Vec::new(),
        paths: graph.path_counts(),
        enter,
        exit,
        depth: vec![0; graph.nodes.len()],
        node_events: (0..graph.nodes.len())
            .map(|_| NodeEvents::default())
            .collect(),
        node_domains: vec![Vec::new(); graph.nodes.len()],
        expected: HashMap::new(),
        paragraph_indices: HashMap::new(),
        metadata: HashSet::new(),
        nearest: HashMap::new(),
        enclosing: HashMap::new(),
        semantic: Vec::new(),
        known: Vec::new(),
        wrapping: Vec::new(),
        native: Vec::new(),
        inline_box_bodies,
        computed: Vec::new(),
        activation: None,
        activation_storage: ActivationStorage::default(),
    };
    proof.index()?;
    proof.activation = activation;
    let box_edges = paragraphs.iter().flat_map(|paragraph| &paragraph.controls).any(|binding| {
        matches!(tree.layout_input(binding.node), crate::LayoutInputViewOf::InlineBoundary(input) if input.parent_edge_axes().is_some())
    });
    if box_edges
        || !wrapping_scopes.is_empty()
        || !inventory_scopes.is_empty()
        || !alignment_inputs.is_empty()
    {
        proof.index_scope_inventory(wrapping_scopes.is_empty() && inventory_scopes.is_empty())?;
    }
    proof.activate_alignments(alignment_inputs)?;
    proof.activate_computed()?;
    proof.activate_wrapping(wrapping_scopes, inventory_scopes)?;
    if box_edges {
        proof.activate_box_edges()?;
    }
    proof.prepare_observations()?;
    // A is selected from producer topology before resolving its supplied rows.
    let (regions, region_copy) = if let Some(work) = proof.activation.as_deref_mut() {
        work.copy(&proof.result.regions)
            .map_err(|error| activation_error::<Tree>(graph.nodes[0], error))?
    } else {
        (
            proof.result.regions.clone(),
            super::scope_alignment::ScopeCensus::default(),
        )
    };
    if let Some(work) = proof.activation.as_deref_mut() {
        let passes = regions.len().checked_mul(3).ok_or_else(|| {
            activation_error::<Tree>(
                graph.nodes[0],
                super::scope_alignment::overflow(
                    surgeist_text::ScopeAlignmentLimitKind::StructuralWork,
                ),
            )
        })?;
        work.visits(passes)
            .map_err(|error| activation_error::<Tree>(graph.nodes[0], error))?;
    }
    for region in &regions {
        let (events, event_copy) = proof.activation_events(region)?;
        for event in events.iter() {
            proof.require_expected(event)?;
            if let Event::BlockStart(child) | Event::BlockEnd(child) = event {
                proof.activation_domain(child.clone())?;
            }
        }
        drop(events);
        if let Some(work) = proof.activation.as_deref_mut() {
            work.release(event_copy);
        }
    }
    let native = proof.active_native_regions();
    for region in &native {
        for event in proof
            .native_selected(region)
            .iter()
            .map(|entry| entry.event.clone())
            .collect::<Vec<_>>()
        {
            proof.require_expected(&event)?;
            if let Event::BlockStart(child) | Event::BlockEnd(child) = event {
                proof.activation_domain(child)?;
            }
        }
    }
    proof.close_associations()?;
    proof.semantic_slots()?;
    // Unexpected real observations inside a region are retained too. Their
    // structural identities need validation but never activate another A region.
    for region in &regions {
        for child in proof.region_domains(region)? {
            proof.activation_domain(child)?;
        }
    }
    for region in &native {
        for child in proof.domains_in_roots(&region.domain(), proof.native_roots(region))? {
            proof.activation_domain(child)?;
        }
    }
    proof.close_associations()?;
    proof.validate_pairs()?;
    proof.validate_metadata()?;
    for region in &regions {
        proof.compare_region(region)?;
    }
    for region in &native {
        proof.compare_native_region(region)?;
    }
    proof.project_computed()?;
    proof.prepare_runs()?;
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if proof.result.paragraphs[index].class == ParagraphClass::AssociationOnly {
            proof.metadata_slot(index)?;
        }
        for scope in paragraph.prepared.scopes() {
            let canonical = scope.captured_scope();
            let association = match canonical.role() {
                ScopeRole::Domain => Association::Domain,
                ScopeRole::Inline => Association::Inline(
                    proof
                        .result
                        .inline
                        .iter()
                        .position(|row| row.scope == canonical)
                        .ok_or_else(|| binding_error(paragraph.container, E::ScopeCoverage))?,
                ),
            };
            proof.result.paragraphs[index]
                .scopes
                .push((scope, association));
        }
        validate_virtuals(paragraph)
            .map_err(|reason| binding_error(paragraph.container, reason))?;
    }
    let wrapping = proof.wrapping_proofs(canonical_orders)?;
    proof.result.structural = proof.domains;
    drop(regions);
    if let Some(work) = proof.activation.as_deref_mut() {
        work.release(region_copy);
        work.release(proof.activation_storage.known);
        work.release(proof.activation_storage.scopes);
        work.retain_existing(proof.activation_storage.domains)
            .map_err(|error| activation_error::<Tree>(graph.nodes[0], error))?;
    }
    Ok(CaptureResolution {
        associations: proof.result,
        wrapping,
    })
}

impl<Tree: crate::LayoutTree> Proof<'_, Tree> {
    // Only activation inventories and scope-owned copies enter this ledger.
    // Graph arrays, ExpectedFlow maps, native topology validation and the
    // preexisting ordinary wrapping inventory retain their native owners.
    fn activation_events(
        &mut self,
        scope: &CapturedScope,
    ) -> ResultOf<Tree, (Vec<Event>, super::scope_alignment::ScopeCensus)> {
        if let Some(work) = self.activation.as_deref_mut() {
            let events = &self.expected[&scope.domain()].events;
            let start = scope.enter_position();
            let end = scope.exit_position();
            let first = events.partition_point(|event| event.position <= start);
            let last = events.partition_point(|event| event.position < end);
            return work
                .copy_map(&events[first..last], |entry| entry.event.clone())
                .map_err(|error| activation_error::<Tree>(self.graph.nodes[0], error));
        }
        Ok((
            self.selected(scope)
                .iter()
                .map(|entry| entry.event.clone())
                .collect(),
            super::scope_alignment::ScopeCensus::default(),
        ))
    }

    fn activation_scope(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        activation_unique::<Tree, _>(
            self.activation.as_deref_mut(),
            &mut self.activation_storage.scopes,
            &mut self.scopes,
            scope,
            false,
            self.graph.nodes[0],
        )
    }

    fn activation_domain(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        activation_unique::<Tree, _>(
            self.activation.as_deref_mut(),
            &mut self.activation_storage.domains,
            &mut self.domains,
            scope,
            false,
            self.graph.nodes[0],
        )
    }

    fn activation_region(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        let node = self.graph.nodes[0];
        let Some(work) = self.activation.as_deref_mut() else {
            add_region(&mut self.result.regions, scope);
            return Ok(());
        };
        for region in &self.result.regions {
            work.visits(1)
                .map_err(|error| activation_error::<Tree>(node, error))?;
            if region.domain() == scope.domain()
                && region.enter_position() <= scope.enter_position()
                && scope.exit_position() <= region.exit_position()
            {
                return Ok(());
            }
        }
        work.visits(self.result.regions.len())
            .map_err(|error| activation_error::<Tree>(node, error))?;
        work.pin(
            &self.result.regions,
            &mut self.activation_storage.regions,
            true,
        )
        .map_err(|error| activation_error::<Tree>(node, error))?;
        self.result.regions.retain(|region| {
            region.domain() != scope.domain()
                || !(scope.enter_position() <= region.enter_position()
                    && region.exit_position() <= scope.exit_position())
        });
        work.pin(
            &self.result.regions,
            &mut self.activation_storage.regions,
            true,
        )
        .map_err(|error| activation_error::<Tree>(node, error))?;
        work.push(
            &mut self.result.regions,
            scope,
            &mut self.activation_storage.regions,
            true,
        )
        .map_err(|error| activation_error::<Tree>(node, error))
    }

    fn index_scope_inventory(&mut self, feature_only: bool) -> ResultOf<Tree, ()> {
        for paragraph in &self.result.paragraphs {
            if let Some(owner) = paragraph.prepared.captured_owner() {
                for captured in owner.members() {
                    for intersection in captured.scope_intersections() {
                        let scope = intersection.scope();
                        if scope.role() == ScopeRole::Inline {
                            activation_unique::<Tree, _>(
                                if feature_only {
                                    self.activation.as_deref_mut()
                                } else {
                                    None
                                },
                                &mut self.activation_storage.known,
                                &mut self.known,
                                scope,
                                false,
                                self.graph.nodes[0],
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn activate_alignments(
        &mut self,
        inputs: &[super::TextScopeAlignmentInputOf<Tree::Scalar>],
    ) -> ResultOf<Tree, ()> {
        for input in inputs {
            let scope = &input.scope;
            if let Some(work) = self.activation.as_deref_mut() {
                work.visits(self.known.len().checked_add(1).ok_or_else(|| {
                    activation_error::<Tree>(
                        self.graph.nodes[0],
                        super::scope_alignment::overflow(
                            surgeist_text::ScopeAlignmentLimitKind::StructuralWork,
                        ),
                    )
                })?)
                .map_err(|error| activation_error::<Tree>(self.graph.nodes[0], error))?;
            }
            if !self.known.contains(scope) {
                return Err(binding_error(
                    self.graph.nodes[0],
                    E::ScopeOccurrenceMembership,
                ));
            }
            // An actual endpoint pair proves this inline box in the host tree.
            // Semantic parent-font context does not fabricate such a pair.
            if scope.entry_control().is_none() || scope.exit_control().is_none() {
                return Err(binding_error(self.graph.nodes[0], E::ScopeBoundaryCapture));
            }
            self.activation_scope(scope.clone())?;
            self.activation_domain(scope.domain())?;
            self.activation_region(scope.clone())?;
        }
        Ok(())
    }
    fn computed_error(
        &self,
        paragraph: usize,
    ) -> crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        binding_error(
            self.paragraphs[paragraph].container,
            E::ComputedBoundaryAssociation,
        )
    }

    fn activate_computed(&mut self) -> ResultOf<Tree, ()> {
        for (index, binding) in self.paragraphs.iter().enumerate() {
            let boundaries = binding
                .prepared
                .participants()
                .filter_map(|part| match part {
                    surgeist_text::PreparedParticipant::ComputedLineBoundary(boundary) => {
                        Some(boundary)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            for boundary in boundaries {
                let view = boundary.view();
                let captured = binding
                    .prepared
                    .captured_owner()
                    .ok_or_else(|| self.computed_error(index))?;
                if boundary.prepared_text() != binding.prepared
                    || view.domain() != captured.domain()
                    || view.causes().len() == 0
                {
                    return Err(self.computed_error(index));
                }
                for position in [view.preceding_event(), view.following_event()]
                    .into_iter()
                    .flatten()
                {
                    if !position.same_capture(&captured.domain().enter_position()) {
                        return Err(self.computed_error(index));
                    }
                }
                for cause in view.causes() {
                    if !owns_member(&binding.prepared, &cause.paragraph())
                        || cause.domain() != captured.domain()
                    {
                        return Err(self.computed_error(index));
                    }
                    match cause.target() {
                        surgeist_text::CapturedComputedLineBoundaryTarget::Scope {
                            scope, ..
                        } => {
                            if scope.role() != ScopeRole::Inline
                                || scope.domain() != captured.domain()
                                || !captured.members().any(|member| {
                                    member.scope_intersections().any(|row| row.scope() == scope)
                                })
                            {
                                return Err(self.computed_error(index));
                            }
                            if scope.entry_control().is_some() || scope.exit_control().is_some() {
                                self.activation_scope(scope.clone())?;
                            } else {
                                push_unique(&mut self.semantic, scope.clone());
                            }
                            add_region(&mut self.result.regions, scope);
                        }
                        surgeist_text::CapturedComputedLineBoundaryTarget::Endpoint(control) => {
                            if !self
                                .controls
                                .iter()
                                .any(|row| row.paragraph == index && row.captured == control)
                            {
                                return Err(self.computed_error(index));
                            }
                        }
                        surgeist_text::CapturedComputedLineBoundaryTarget::Atomic {
                            position,
                            ..
                        } => {
                            if !self.atomics.iter().any(|row| {
                                row.paragraph == index
                                    && row.position == position
                                    && row.kind == surgeist_text::InlineBoxKind::InFlow
                            }) {
                                return Err(self.computed_error(index));
                            }
                        }
                    }
                }
                self.activation_domain(captured.domain())?;
                self.computed.push((index, boundary));
            }
        }
        Ok(())
    }

    fn computed_node_gap(
        &self,
        paragraph: usize,
        node: Tree::Node,
        after: bool,
    ) -> ResultOf<Tree, usize> {
        let binding = &self.paragraphs[paragraph];
        if binding.nodes.is_empty() {
            if self.result.semantic_slot(&binding.prepared).is_none() {
                return Err(self.computed_error(paragraph));
            }
            let owner = self.require_node(binding.container)?;
            let mut branch = self.require_node(node)?;
            while self.graph.parents[branch].first().copied() != Some(owner) {
                branch = *self.graph.parents[branch]
                    .first()
                    .ok_or_else(|| self.computed_error(paragraph))?;
            }
            return Ok(0);
        }
        if let Some(index) = binding.nodes.iter().position(|&n| n == node) {
            return Ok(index + usize::from(after));
        }
        let owner = self.require_node(binding.container)?;
        let mut branch = self.require_node(node)?;
        while self.graph.parents[branch].first().copied() != Some(owner) {
            branch = *self.graph.parents[branch]
                .first()
                .ok_or_else(|| self.computed_error(paragraph))?;
        }
        let rank = self.graph.children[owner]
            .iter()
            .position(|&i| i == branch)
            .ok_or_else(|| self.computed_error(paragraph))?;
        let mut gap = 0;
        for &node in &binding.nodes {
            let node = self.require_node(node)?;
            let actual = self.graph.children[owner]
                .iter()
                .position(|&child| child == node)
                .ok_or_else(|| self.computed_error(paragraph))?;
            if actual == rank {
                return Err(self.computed_error(paragraph));
            }
            gap += usize::from(actual < rank);
        }
        Ok(gap)
    }

    fn computed_event_gap(
        &self,
        paragraph: usize,
        event: &Event,
        consumed: bool,
    ) -> ResultOf<Tree, usize> {
        let binding = &self.paragraphs[paragraph];
        match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => {
                let index = self
                    .paragraph(p)
                    .ok_or_else(|| self.computed_error(paragraph))?;
                match self.member_class(p) {
                    ParagraphClass::AssociationOnly => Err(self.computed_error(paragraph)),
                    ParagraphClass::SemanticFlow => {
                        let slot = self
                            .result
                            .slots
                            .iter()
                            .find(|slot| slot.captured == *p)
                            .ok_or_else(|| self.computed_error(paragraph))?;
                        let owner = self.require_node(binding.container)?;
                        Ok(binding
                            .nodes
                            .iter()
                            .take_while(|&&node| {
                                let index = self.graph.index(node).expect("validated native node");
                                self.graph.children[owner]
                                    .iter()
                                    .position(|&child| child == index)
                                    .expect("direct native participant")
                                    < slot.gap
                            })
                            .count())
                    }
                    ParagraphClass::Flow => {
                        let after = matches!(event, Event::ParagraphEnd(_));
                        let node = self
                            .member_node(p, after)
                            .ok_or_else(|| self.computed_error(index))?;
                        self.computed_node_gap(paragraph, node, after)
                    }
                }
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope)
                if scope
                    == &binding
                        .prepared
                        .captured_owner()
                        .expect("computed captured paragraph")
                        .domain() =>
            {
                Ok(if matches!(event, Event::BlockEnd(_)) {
                    binding.nodes.len()
                } else {
                    0
                })
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope) => self.computed_node_gap(
                paragraph,
                self.domain(scope)?.container,
                matches!(event, Event::BlockEnd(_)),
            ),
            _ => self.computed_node_gap(paragraph, self.event_node(event)?, consumed),
        }
    }

    fn project_computed(&mut self) -> ResultOf<Tree, ()> {
        for (index, boundary) in &self.computed {
            let view = boundary.view();
            let flow = &self.expected[&view.domain()];
            let before = view.preceding_event();
            let cut = before.as_ref().map_or(0, |position| {
                flow.events.partition_point(|row| row.position <= *position)
            });
            let previous = cut.checked_sub(1).and_then(|i| {
                if self.visible(&flow.events[i].event) {
                    Some(i)
                } else {
                    flow.previous[i]
                }
            });
            let next = (cut < flow.events.len()).then_some(cut).and_then(|i| {
                if self.visible(&flow.events[i].event) {
                    Some(i)
                } else {
                    flow.next[i]
                }
            });
            let mut lower = previous
                .map(|i| self.computed_event_gap(*index, &flow.events[i].event, true))
                .transpose()?
                .unwrap_or(0);
            let mut upper = next
                .map(|i| self.computed_event_gap(*index, &flow.events[i].event, false))
                .transpose()?
                .unwrap_or(self.paragraphs[*index].nodes.len());
            // All actual targets constrain the one group cut. Several distinct
            // groups with coincident bounds remain distinct issued capabilities.
            for cause in view.causes() {
                let target = match cause.target() {
                    surgeist_text::CapturedComputedLineBoundaryTarget::Scope { scope, side } => {
                        self.result
                            .inline
                            .iter()
                            .find(|row| row.scope == scope)
                            .map(|row| {
                                (
                                    if side == surgeist_text::SourceBoxSide::After {
                                        row.end
                                    } else {
                                        row.start
                                    },
                                    side == surgeist_text::SourceBoxSide::After,
                                )
                            })
                    }
                    surgeist_text::CapturedComputedLineBoundaryTarget::Endpoint(control) => self
                        .controls
                        .iter()
                        .find(|row| row.captured == control)
                        .map(|row| {
                            (
                                row.node,
                                matches!(
                                    control.kind(),
                                    surgeist_text::ParagraphControlKind::InlineBoundaryEnd
                                ),
                            )
                        }),
                    surgeist_text::CapturedComputedLineBoundaryTarget::Atomic {
                        position,
                        side,
                    } => self
                        .atomics
                        .iter()
                        .find(|row| row.position == position)
                        .map(|row| (row.node, side == surgeist_text::SourceBoxSide::After)),
                };
                if let Some((node, after)) = target {
                    let exact = self.computed_node_gap(*index, node, after)?;
                    lower = lower.max(exact);
                    upper = upper.min(exact);
                }
            }
            if lower > upper {
                return Err(self.computed_error(*index));
            }
            self.result.computed.push(ComputedIncidence {
                boundary: boundary.clone(),
                cut: ComputedNativeCut { lower, upper },
            });
        }
        Ok(())
    }

    fn activate_box_edges(&mut self) -> ResultOf<Tree, ()> {
        for index in 0..self.controls.len() {
            let node = self.controls[index].node;
            let crate::LayoutInputViewOf::InlineBoundary(input) = self.tree.layout_input(node)
            else {
                continue;
            };
            let Some(parent_axes) = input.parent_edge_axes() else {
                continue;
            };
            let captured = self.controls[index].captured.clone();
            if let Some(work) = self.activation.as_deref_mut() {
                work.visits(self.known.len())
                    .map_err(|error| activation_error::<Tree>(node, error))?;
            }
            let owner = self
                .known
                .iter()
                .find(|scope| {
                    scope.entry_control().as_ref() == Some(&captured)
                        || scope.exit_control().as_ref() == Some(&captured)
                })
                .cloned();
            let pair = if let Some(scope) = &owner {
                let (start, end) = (
                    scope
                        .entry_control()
                        .ok_or_else(|| binding_error(node, E::ScopeBoundaryCapture))?,
                    scope
                        .exit_control()
                        .ok_or_else(|| binding_error(node, E::ScopeBoundaryCapture))?,
                );
                if let Some(work) = self.activation.as_deref_mut() {
                    work.visits(self.controls.len())
                        .map_err(|error| activation_error::<Tree>(node, error))?;
                }
                let start = self
                    .controls
                    .iter()
                    .find(|row| row.captured == start)
                    .map(|row| row.node)
                    .ok_or_else(|| binding_error(node, E::ScopeAssociation))?;
                if let Some(work) = self.activation.as_deref_mut() {
                    work.visits(self.controls.len())
                        .map_err(|error| activation_error::<Tree>(node, error))?;
                }
                let end = self
                    .controls
                    .iter()
                    .find(|row| row.captured == end)
                    .map(|row| row.node)
                    .ok_or_else(|| binding_error(node, E::ScopeAssociation))?;
                (start, end)
            } else {
                if let Some(work) = self.activation.as_deref_mut() {
                    work.visits(self.native.len())
                        .map_err(|error| activation_error::<Tree>(node, error))?;
                }
                self.native
                    .iter()
                    .find(|row| row.pair.0 == node || row.pair.1 == node)
                    .map(|row| row.pair)
                    .ok_or_else(|| binding_error(node, E::ScopeAssociation))?
            };
            for peer in [pair.0, pair.1] {
                let actual = match self.tree.layout_input(peer) {
                    crate::LayoutInputViewOf::InlineBoundary(peer) => peer.parent_edge_axes(),
                    _ => None,
                };
                if actual != Some(parent_axes) {
                    return Err(crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(peer),
                        crate::LayoutOperation::TextCapture,
                        crate::LayoutErrorKindOf::InvalidInput(
                            crate::LayoutInvalidInputOf::InlineBoundaryParentMismatch {
                                expected: Some(parent_axes),
                                actual,
                            },
                        ),
                    ));
                }
            }
            if let Some(scope) = owner {
                self.activate_scope(&scope, true)?;
            }
        }
        Ok(())
    }

    fn activate_wrapping(
        &mut self,
        inputs: &[WrappingScopeInput<Tree::Node>],
        inventory_scopes: &[CapturedScope],
    ) -> ResultOf<Tree, ()> {
        if inputs.is_empty() && inventory_scopes.is_empty() {
            return Ok(());
        }
        let mut native_observations = Vec::new();
        let mut forest_scopes = Vec::new();
        for input in inputs {
            if let WrappingScopeInput::NativePair { start, end, .. } = input {
                let complete = self.controls.iter().any(|row| {
                    (row.node == *start || row.node == *end)
                        && row.captured.paragraph().domain().domain_topology()
                            == surgeist_text::SourceProcessingTopology::Complete
                });
                let source = self
                    .controls
                    .iter()
                    .find(|row| row.node == *start)
                    .zip(self.controls.iter().find(|row| row.node == *end));
                let observation = source
                    .filter(|(a, b)| {
                        a.captured.paragraph().domain() == b.captured.paragraph().domain()
                    })
                    .map(|(a, b)| {
                        CanonicalOrderInventory::native_pair(
                            &self.known,
                            a.captured.clone(),
                            b.captured.clone(),
                        )
                    })
                    .transpose()
                    .map_err(|_| {
                        crate::inline_wrapping::wrapping_error(
                            self.graph.nodes[0],
                            WrappingScopeError::CaptureAssociation,
                        )
                    })?;
                if complete && observation.is_none() {
                    return Err(crate::inline_wrapping::wrapping_error(
                        self.graph.nodes[0],
                        WrappingScopeError::CaptureAssociation,
                    ));
                }
                let matching = matches!(observation, Some(CanonicalBox::Scope(_)));
                if let Some(CanonicalBox::Scope(scope)) = &observation {
                    push_unique(&mut forest_scopes, scope.clone());
                }
                if let Some(observation @ CanonicalBox::NativePair { .. }) = observation
                    && !native_observations.contains(&observation)
                {
                    native_observations.push(observation);
                }
                if let Some((a, b)) = source.filter(|(a, b)| {
                    a.captured.paragraph().domain() == b.captured.paragraph().domain()
                }) {
                    let (flow, span) = crate::inline_wrapping::native_span(
                        self.tree,
                        self.graph,
                        *start,
                        *end,
                        &self.paths,
                    )
                    .map_err(|reason| {
                        crate::inline_wrapping::wrapping_error(self.graph.nodes[0], reason)
                    })?;
                    let domain = a.captured.paragraph().domain();
                    if self.domain(&domain)?.container != flow
                        || self.paragraphs[a.paragraph].container != flow
                        || self.paragraphs[b.paragraph].container != flow
                        || a.captured
                            .event_position()
                            .partial_cmp(&b.captured.event_position())
                            != Some(std::cmp::Ordering::Less)
                    {
                        return Err(crate::inline_wrapping::wrapping_error(
                            flow,
                            WrappingScopeError::CaptureAssociation,
                        ));
                    }
                    // A real native parent may surround an owned complete body
                    // while its endpoint raw owners belong to other bindings.
                    // Demand follows the actual checked flow/domain, not only
                    // the endpoint binding indices or selected allocations.
                    let mut demanded = false;
                    if self.inline_box_bodies {
                        for paragraph in self.paragraphs {
                            if let Some(work) = self.activation.as_deref_mut() {
                                work.visits(1)
                                    .map_err(|error| activation_error::<Tree>(flow, error))?;
                            }
                            if paragraph.inline_box_fragments
                                == super::InlineBoxFragmentationDemand::CompleteBodies
                                && paragraph.container == flow
                                && paragraph
                                    .prepared
                                    .captured_owner()
                                    .is_some_and(|owner| owner.domain() == domain)
                            {
                                demanded = true;
                                break;
                            }
                        }
                    }
                    let active = (demanded || match input {
                        WrappingScopeInput::NativePair {
                            wrap_inside,
                            edges,
                            metric_contribution,
                            ..
                        } => {
                            [*start, *end].iter().any(|node| matches!(self.tree.layout_input(*node), crate::LayoutInputViewOf::InlineBoundary(input) if input.parent_edge_axes().is_some()))
                                || *wrap_inside == WrapInside::Avoid
                                || metric_contribution.is_some()
                                || edges.has_effect(crate::inline_wrapping::WrappingMode::Inline)
                        }
                        _ => false,
                    }) && !matching;
                    if let Some(prior) = self
                        .native
                        .iter_mut()
                        .find(|row| row.pair == (*start, *end))
                    {
                        prior.active |= active;
                    } else {
                        self.native.push(NativeRegion {
                            flow,
                            pair: (*start, *end),
                            start: a.captured.clone(),
                            end: b.captured.clone(),
                            owner: self.graph.index(flow).expect("proved native flow"),
                            first: span.first,
                            last: span.last,
                            active,
                        });
                    }
                    self.activation_domain(domain)?;
                }
            }
            if let WrappingScopeInput::Captured {
                scope,
                wrap_inside,
                edges,
                metric_contribution,
            } = input
            {
                self.activate_scope(
                    scope,
                    *wrap_inside == WrapInside::Avoid
                        || metric_contribution.is_some()
                        || edges.has_effect(crate::inline_wrapping::WrappingMode::Inline),
                )?;
                push_unique(&mut forest_scopes, scope.clone());
            }
        }
        let active_inventory = inputs.iter().any(|input| match input {
            WrappingScopeInput::Captured {
                wrap_inside,
                edges,
                metric_contribution,
                ..
            }
            | WrappingScopeInput::NativePair {
                wrap_inside,
                edges,
                metric_contribution,
                ..
            } => {
                *wrap_inside == WrapInside::Avoid
                    || metric_contribution.is_some()
                    || edges.has_effect(crate::inline_wrapping::WrappingMode::Inline)
            }
        }) || self.graph.nodes.iter().any(|node| {
            self.tree
                .node_input(*node)
                .wrap_edges
                .has_effect(crate::inline_wrapping::WrappingMode::Inline)
        });
        for scope in inventory_scopes {
            self.activate_scope(scope, active_inventory)?;
            push_unique(&mut forest_scopes, scope.clone());
        }
        // A source box not supplied as a normalized forest row still
        // constrains a claimed native pair. Declared rows are checked by the
        // forest itself, preserving its specific CrossingScopes diagnostic.
        let unrepresented = self
            .known
            .iter()
            .filter(|scope| !forest_scopes.contains(scope))
            .cloned()
            .collect::<Vec<_>>();
        for observation in &native_observations {
            CanonicalOrderInventory::validate_native(
                &unrepresented,
                std::slice::from_ref(observation),
            )
            .map_err(|_| {
                crate::inline_wrapping::wrapping_error(
                    self.graph.nodes[0],
                    WrappingScopeError::CaptureAssociation,
                )
            })?;
        }
        // Known physical pairing is metadata, not a request to validate every
        // unrelated interior. Resolve only actual supplied endpoint bindings.
        let known_count = self.known.len();
        for index in 0..known_count {
            let scope = self.known[index].clone();
            let start = scope.entry_control().and_then(|c| {
                self.controls
                    .iter()
                    .find(|r| r.captured == c)
                    .map(|r| r.node)
            });
            let end = scope.exit_control().and_then(|c| {
                self.controls
                    .iter()
                    .find(|r| r.captured == c)
                    .map(|r| r.node)
            });
            match (start, end) {
                (Some(start), Some(end)) => {
                    self.activation_scope(scope.clone())?;
                    self.activation_domain(scope.domain())?;
                    push_unique(&mut self.wrapping, scope.clone());
                    if inputs.iter().any(|input| {
                        matches!(input, WrappingScopeInput::NativePair {
                            start:a, end:b, wrap_inside, edges, metric_contribution,
                        } if start == *a && end == *b && (*wrap_inside==WrapInside::Avoid || metric_contribution.is_some() || edges.has_effect(crate::inline_wrapping::WrappingMode::Inline)))
                    }) {
                        self.activation_region(scope.clone())?;
                    }
                }
                (start, end) => {
                    if inputs.iter().any(|input| {
                        matches!(input, WrappingScopeInput::NativePair {start:a,end:b,..}
                        if start.is_some_and(|node|node==*a) || end.is_some_and(|node|node==*b))
                    }) {
                        return Err(crate::inline_wrapping::wrapping_error(
                            self.graph.nodes[0],
                            WrappingScopeError::CaptureAssociation,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate a genuine captured actual-box reference, independently of declarations.
    fn activate_scope(&mut self, scope: &CapturedScope, active: bool) -> ResultOf<Tree, ()> {
        if scope.role() != ScopeRole::Inline {
            return Err(crate::inline_wrapping::wrapping_error(
                self.graph.nodes[0],
                WrappingScopeError::NonInlineScope,
            ));
        }
        if !self.known.contains(scope) {
            let present = self
                .result
                .paragraphs
                .iter()
                .any(|p| p.prepared.captured_owner().is_some());
            let same = self
                .result
                .paragraphs
                .iter()
                .filter_map(|p| p.prepared.captured_owner())
                .any(|p| {
                    p.domain()
                        .enter_position()
                        .same_capture(&scope.enter_position())
                });
            let reason = if present && !same {
                WrappingScopeError::ForeignCapture
            } else {
                WrappingScopeError::MissingCapture
            };
            return Err(crate::inline_wrapping::wrapping_error(
                self.graph.nodes[0],
                reason,
            ));
        }
        let mut current = Some(scope.clone());
        while let Some(scope) = current.filter(|s| s.role() == ScopeRole::Inline) {
            push_unique(&mut self.wrapping, scope.clone());
            current = scope.parent();
        }
        self.activation_domain(scope.domain())?;
        if scope.entry_control().is_some() || scope.exit_control().is_some() {
            self.activation_scope(scope.clone())?;
        } else {
            push_unique(&mut self.semantic, scope.clone());
        }
        if active {
            self.activation_region(scope.clone())?;
        }
        Ok(())
    }

    /// An enclosing requested pair already proves its own-flow interior. Keep
    /// only maximal such work items; identities and the common forest are not
    /// merged, and canonical semantic/physical proof remains independent.
    fn active_native_regions(&self) -> Vec<NativeRegion<Tree::Node>> {
        self.native
            .iter()
            .filter(|region| region.active)
            .filter(|region| {
                !self.native.iter().any(|outer| {
                    outer.active
                        && outer.owner == region.owner
                        && outer.domain() == region.domain()
                        && outer.first < region.first
                        && region.last < outer.last
                        && outer.start.event_position() < region.start.event_position()
                        && region.end.event_position() < outer.end.event_position()
                })
            })
            .cloned()
            .collect()
    }
    fn wrapping_proofs(
        &self,
        canonical_orders: bool,
    ) -> ResultOf<Tree, Vec<WrappingScopeProof<Tree::Node>>> {
        let order = if canonical_orders {
            let mut nodes = Vec::new();
            for (node, position, paragraph) in self
                .origins
                .iter()
                .map(|row| (row.node, &row.position, row.paragraph))
                .chain(
                    self.atomics
                        .iter()
                        .map(|row| (row.node, &row.position, row.paragraph)),
                )
            {
                let captured = self.result.paragraphs[paragraph]
                    .prepared
                    .captured_owner()
                    .ok_or_else(|| binding_error(node, E::ScopeAssociation))?;
                nodes.push((
                    node,
                    CanonicalBox::Occurrence {
                        position: position.clone(),
                        domain: captured.domain(),
                    },
                ));
            }
            let mut coverage = Vec::new();
            for row in &self.result.paragraphs {
                if let Some(owner) = row.prepared.captured_owner() {
                    coverage.extend(owner.members());
                }
            }
            Some(std::sync::Arc::new(CanonicalOrderBindings {
                inventory: CanonicalOrderInventory {
                    scopes: self.known.clone(),
                    occurrences: nodes.iter().map(|(_, fact)| fact.clone()).collect(),
                    coverage: CanonicalCoverage::Paragraphs(coverage),
                },
                nodes,
                flows: self
                    .result
                    .domains
                    .iter()
                    .map(|row| (row.container, row.scope.clone()))
                    .collect(),
            }))
        } else {
            None
        };
        let mut result = self
            .wrapping
            .iter()
            .map(|scope| {
                let pair = self
                    .result
                    .inline
                    .iter()
                    .find(|row| row.scope == *scope)
                    .map(|row| (row.start, row.end));
                Ok(WrappingScopeProof::Captured(CanonicalScopeProof {
                    scope: scope.clone(),
                    flow: self.domain(&scope.domain())?.container,
                    pair,
                    order: order.clone(),
                }))
            })
            .collect::<ResultOf<Tree, Vec<_>>>()?;
        result.extend(
            self.native
                .iter()
                .map(|row| WrappingScopeProof::NativePair {
                    flow: row.flow,
                    pair: row.pair,
                    start: row.start.clone(),
                    end: row.end.clone(),
                    order: order.clone(),
                }),
        );
        Ok(result)
    }
    fn index(&mut self) -> ResultOf<Tree, ()> {
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            let captured = paragraph.prepared.captured_owner();
            let class =
                classify(paragraph).map_err(|reason| binding_error(paragraph.container, reason))?;
            if let Some(captured) = &captured {
                for member in captured.members() {
                    if self.paragraph_indices.insert(member, index).is_some() {
                        return Err(binding_error(paragraph.container, E::OverlappingParagraphs));
                    }
                }
                let domain = captured.domain();
                if let Some(old) = self.result.domains.iter().find(|d| d.scope == domain) {
                    if old.container != paragraph.container {
                        return Err(binding_error(paragraph.container, E::ScopeAssociation));
                    }
                } else {
                    if self.result.domains.iter().any(|d| {
                        d.container == paragraph.container && same_capture(&d.scope, &domain)
                    }) {
                        return Err(binding_error(paragraph.container, E::ScopeAssociation));
                    }
                    self.result.domains.push(Domain {
                        scope: domain.clone(),
                        container: paragraph.container,
                    });
                }
                if class != ParagraphClass::Flow {
                    push_unique(&mut self.domains, domain);
                }
            }
            for row in &paragraph.controls {
                if let Some(control) = row.occurrence.captured_control() {
                    if !owns_member(&paragraph.prepared, &control.paragraph())
                        || self.controls.iter().any(|old| old.captured == control)
                    {
                        return Err(binding_error(row.node, E::ControlCoverage));
                    }
                    self.controls.push(Control {
                        captured: control,
                        node: row.node,
                        paragraph: index,
                    });
                }
            }
            for row in &paragraph.atomics {
                if let Some(position) = row.occurrence.view().capture_position() {
                    let member = occurrence_member(&paragraph.prepared, &position, true)
                        .ok_or_else(|| binding_error(row.node, E::AtomicCoverage))?;
                    if self.atomics.iter().any(|old| old.position == position) {
                        return Err(binding_error(row.node, E::AtomicCoverage));
                    }
                    self.atomics.push(Atomic {
                        position,
                        member,
                        node: row.node,
                        paragraph: index,
                        kind: row.occurrence.view().original_box().kind(),
                    });
                }
            }
            for row in &paragraph.origins {
                if let Some(position) = row.origin.view().capture_position() {
                    let member = occurrence_member(&paragraph.prepared, &position, false)
                        .ok_or_else(|| binding_error(row.node, E::OriginCoverage))?;
                    if self.origins.iter().any(|old| old.position == position) {
                        return Err(binding_error(row.node, E::OriginCoverage));
                    }
                    self.origins.push(Origin {
                        position,
                        member,
                        node: row.node,
                        paragraph: index,
                    });
                }
            }
            for scope in paragraph.prepared.scopes() {
                let canonical = scope.captured_scope();
                let Some(captured) = &captured else {
                    return Err(binding_error(paragraph.container, E::ScopeCoverage));
                };
                if canonical.domain() != captured.domain()
                    || !captured
                        .members()
                        .any(|member| member.scope_intersections().any(|i| i.scope() == canonical))
                {
                    return Err(binding_error(
                        paragraph.container,
                        E::ScopeOccurrenceMembership,
                    ));
                }
                push_unique(&mut self.scopes, canonical.clone());
                push_unique(&mut self.domains, canonical.domain());
                add_region(&mut self.result.regions, canonical);
            }
            self.result.paragraphs.push(ParagraphResolution {
                prepared: paragraph.prepared.clone(),
                container: paragraph.container,
                class,
                scopes: Vec::new(),
            });
        }
        Ok(())
    }
    fn member_class(&self, captured: &CapturedParagraph) -> ParagraphClass {
        if self.origins.iter().any(|row| row.member == *captured)
            || self.atomics.iter().any(|row| row.member == *captured)
            || self
                .controls
                .iter()
                .any(|row| row.captured.paragraph() == *captured)
        {
            ParagraphClass::Flow
        } else if self.computed.iter().any(|(_, boundary)| {
            boundary
                .view()
                .causes()
                .any(|cause| cause.paragraph() == *captured)
        }) {
            ParagraphClass::SemanticFlow
        } else {
            ParagraphClass::AssociationOnly
        }
    }

    fn member_node(&self, member: &CapturedParagraph, last: bool) -> Option<Tree::Node> {
        self.origins
            .iter()
            .filter(|row| row.member == *member)
            .map(|row| (row.position.clone(), row.node))
            .chain(
                self.atomics
                    .iter()
                    .filter(|row| row.member == *member)
                    .map(|row| (row.position.clone(), row.node)),
            )
            .chain(
                self.controls
                    .iter()
                    .filter(|row| row.captured.paragraph() == *member)
                    .map(|row| (row.captured.event_position(), row.node)),
            )
            .reduce(|a, b| {
                let order =
                    a.0.partial_cmp(&b.0)
                        .expect("one authentic captured member");
                if (last && order.is_lt()) || (!last && order.is_gt()) {
                    b
                } else {
                    a
                }
            })
            .map(|(_, node)| node)
    }

    fn paragraph(&self, captured: &CapturedParagraph) -> Option<usize> {
        self.paragraph_indices.get(captured).copied()
    }
    fn domain(&self, scope: &CapturedScope) -> ResultOf<Tree, Domain<Tree::Node>> {
        self.result
            .domains
            .iter()
            .find(|d| &d.scope == scope)
            .cloned()
            .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))
    }
    fn require_node(&self, node: Tree::Node) -> ResultOf<Tree, usize> {
        let index = self
            .graph
            .index(node)
            .ok_or_else(|| binding_error(node, E::ScopeAssociation))?;
        if self.paths[index] != 1 {
            return Err(binding_error(node, E::ScopeTreeAmbiguity));
        }
        Ok(index)
    }
    fn require_expected(&self, event: &Event) -> ResultOf<Tree, ()> {
        let (node, paragraph) = match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => {
                let index = self
                    .paragraph(p)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                (self.paragraphs[index].container, Some(index))
            }
            Event::Origin {
                paragraph,
                position,
            } => {
                let row = self
                    .origins
                    .iter()
                    .find(|o| &o.position == position)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if &row.member != paragraph
                    || !owns_member(&self.result.paragraphs[row.paragraph].prepared, paragraph)
                {
                    return Err(binding_error(row.node, E::OriginCoverage));
                }
                (row.node, Some(row.paragraph))
            }
            Event::Atomic {
                paragraph,
                position,
            } => {
                let row = self
                    .atomics
                    .iter()
                    .find(|a| &a.position == position)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if &row.member != paragraph
                    || !owns_member(&self.result.paragraphs[row.paragraph].prepared, paragraph)
                {
                    return Err(binding_error(row.node, E::AtomicCoverage));
                }
                (row.node, Some(row.paragraph))
            }
            Event::Control(control) => {
                let row = self
                    .controls
                    .iter()
                    .find(|c| &c.captured == control)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                (row.node, Some(row.paragraph))
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope) => {
                (self.domain(scope)?.container, None)
            }
        };
        self.require_node(node)?;
        if let Some(index) = paragraph {
            self.require_node(self.paragraphs[index].container)?;
        }
        Ok(())
    }
    fn close_associations(&mut self) -> ResultOf<Tree, ()> {
        let mut b = 0;
        let mut c = 0;
        while b < self.scopes.len() || c < self.domains.len() {
            while b < self.scopes.len() {
                let scope = self.scopes[b].clone();
                b += 1;
                self.scope_association(scope)?;
            }
            while c < self.domains.len() {
                let domain = self.domains[c].clone();
                c += 1;
                self.domain_association(domain)?;
            }
        }
        Ok(())
    }
    fn ancestors(
        &mut self,
        mut scope: Option<CapturedScope>,
        domain: &CapturedScope,
        nominee: Option<&CapturedEventPosition>,
        site: Tree::Node,
    ) -> ResultOf<Tree, Option<CapturedScope>> {
        // Capture construction proves nested canonical extents. Containment in
        // the immediate retained inline therefore proves all its ancestors too.
        if let (Some(parent), Some(position)) = (&scope, nominee)
            && parent.domain() == *domain
            && parent.role() == ScopeRole::Inline
            && !(parent.enter_position() < *position && *position < parent.exit_position())
        {
            return Err(binding_error(site, E::ScopeAssociation));
        }
        let mut trail = Vec::new();
        let nearest = loop {
            let Some(parent) = scope else {
                break None;
            };
            if &parent.domain() != domain || parent.role() == ScopeRole::Domain {
                break None;
            }
            if parent.entry_control().is_some() && parent.exit_control().is_some() {
                break Some(parent);
            }
            if let Some(known) = self.nearest.get(&parent) {
                break known.clone();
            }
            scope = parent.parent();
            trail.push(parent);
        };
        for parent in trail {
            self.nearest.insert(parent, nearest.clone());
        }
        if let Some(parent) = &nearest {
            self.activation_scope(parent.clone())?;
        }
        // Processing that B work item closes the remaining complete ancestors.
        Ok(nearest)
    }
    fn scope_association(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        if let Some(work) = self.activation.as_deref_mut() {
            work.visits(1)
                .map_err(|error| activation_error::<Tree>(self.graph.nodes[0], error))?;
        }
        let domain = self.domain(&scope.domain())?;
        self.activation_domain(scope.domain())?;
        let parent = self.ancestors(scope.parent(), &scope.domain(), None, domain.container)?;
        self.enclosing.insert(scope.clone(), parent);
        if scope.role() == ScopeRole::Domain || self.result.inline.iter().any(|s| s.scope == scope)
        {
            return Ok(());
        }
        let (Some(entry), Some(exit)) = (scope.entry_control(), scope.exit_control()) else {
            return Err(binding_error(domain.container, E::ScopeBoundaryCapture));
        };
        let start = self
            .controls
            .iter()
            .find(|c| c.captured == entry)
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        let end = self
            .controls
            .iter()
            .find(|c| c.captured == exit)
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        let exact = |c: &Control<Tree::Node>, kind| {
            c.captured.paragraph().domain() == scope.domain()
                && self.paragraphs[c.paragraph].container == domain.container
                && matches!(self.tree.layout_input(c.node), crate::LayoutInputViewOf::InlineBoundary(input) if input.kind() == kind)
        };
        if start.node == end.node
            || !exact(start, crate::InlineBoundaryKind::Start)
            || !exact(end, crate::InlineBoundaryKind::End)
        {
            return Err(binding_error(domain.container, E::ScopeAssociation));
        }
        self.require_node(start.node)?;
        self.require_node(end.node)?;
        let row = InlineScope {
            scope,
            start: start.node,
            end: end.node,
        };
        if let Some(work) = self.activation.as_deref_mut() {
            work.push(
                &mut self.result.inline,
                row,
                &mut self.activation_storage.inline,
                true,
            )
            .map_err(|error| activation_error::<Tree>(self.graph.nodes[0], error))?;
        } else {
            self.result.inline.push(row);
        }
        Ok(())
    }
    fn domain_association(&mut self, scope: CapturedScope) -> ResultOf<Tree, ()> {
        let domain = self.domain(&scope)?;
        self.require_node(domain.container)?;
        if self.tree.layout_input(domain.container).as_box().is_none() {
            return Err(binding_error(domain.container, E::ScopeAssociation));
        }
        let parent = scope.parent().map(|p| p.domain());
        let definition = scope
            .domain_definition()
            .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
        match definition {
            CapturedDomainDefinition::Block => {
                if self.atomics.iter().any(|a| {
                    a.node == domain.container && a.position.same_capture(&scope.enter_position())
                }) {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                if domain.container == self.graph.nodes[0] {
                    return Ok(());
                }
                if let Some(parent) = parent {
                    let actual_parent = self.domain(&parent)?;
                    self.require_node(actual_parent.container)?;
                    if !descendant(self.graph, actual_parent.container, domain.container) {
                        return Err(binding_error(domain.container, E::ScopeAssociation));
                    }
                    self.no_intervening(&scope, actual_parent.container, domain.container)?;
                    let enclosing =
                        self.ancestors(scope.parent(), &parent, None, domain.container)?;
                    self.enclosing.insert(scope.clone(), enclosing);
                    self.activation_domain(parent)?;
                }
            }
            CapturedDomainDefinition::AtomicBlock { atomic }
            | CapturedDomainDefinition::OutOfFlowBlock { atomic } => {
                let row = self
                    .atomics
                    .iter()
                    .find(|a| a.position == atomic)
                    .ok_or_else(|| binding_error(domain.container, E::ScopeAssociation))?;
                let expected = if matches!(
                    scope.domain_definition(),
                    Some(CapturedDomainDefinition::AtomicBlock { .. })
                ) {
                    surgeist_text::InlineBoxKind::InFlow
                } else {
                    surgeist_text::InlineBoxKind::OutOfFlow
                };
                if row.node != domain.container || row.kind != expected {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                let captured = self.result.paragraphs[row.paragraph]
                    .prepared
                    .captured_owner()
                    .ok_or_else(|| binding_error(row.node, E::ScopeAssociation))?;
                let parent = parent.ok_or_else(|| binding_error(row.node, E::ScopeAssociation))?;
                if captured.domain() != parent {
                    return Err(binding_error(row.node, E::ScopeAssociation));
                }
                let actual_parent = self.domain(&parent)?;
                if !descendant(self.graph, actual_parent.container, domain.container) {
                    return Err(binding_error(domain.container, E::ScopeAssociation));
                }
                self.no_intervening(&scope, actual_parent.container, domain.container)?;
                let enclosing =
                    self.ancestors(scope.parent(), &parent, Some(&atomic), domain.container)?;
                self.enclosing.insert(scope.clone(), enclosing);
                self.activation_domain(parent)?;
            }
        }
        Ok(())
    }
    fn no_intervening(
        &self,
        scope: &CapturedScope,
        parent: Tree::Node,
        node: Tree::Node,
    ) -> ResultOf<Tree, ()> {
        for other in &self.result.domains {
            if other.container != parent
                && other.container != node
                && same_capture(scope, &other.scope)
                && descendant(self.graph, parent, other.container)
                && descendant(self.graph, other.container, node)
                && !scope_ancestor(&other.scope, scope)
            {
                return Err(binding_error(node, E::ScopeAssociation));
            }
        }
        Ok(())
    }
    fn validate_pairs(&self) -> ResultOf<Tree, ()> {
        let positions: Vec<_> = self
            .result
            .inline
            .iter()
            .map(|row| {
                Ok((
                    self.enter[self.require_node(row.start)?],
                    self.enter[self.require_node(row.end)?],
                ))
            })
            .collect::<ResultOf<Tree, _>>()?;
        for (index, row) in self.result.inline.iter().enumerate() {
            let (start, end) = positions[index];
            if start >= end {
                return Err(binding_error(row.start, E::ScopeBoundaryOrder));
            }
            for (other_index, other) in self.result.inline[..index].iter().enumerate() {
                if row.scope.domain() != other.scope.domain() {
                    continue;
                }
                let (first, last) = positions[other_index];
                if interval_relation(
                    &row.scope.enter_position(),
                    &row.scope.exit_position(),
                    &other.scope.enter_position(),
                    &other.scope.exit_position(),
                ) != interval_relation(&start, &end, &first, &last)
                {
                    return Err(binding_error(row.start, E::ScopeBoundaryOrder));
                }
            }
        }
        for scope in &self.domains {
            let Some(Some(parent)) = self.enclosing.get(scope) else {
                continue;
            };
            let index = self
                .result
                .inline
                .iter()
                .position(|s| &s.scope == parent)
                .expect("closed enclosing B association");
            let (first, last) = positions[index];
            let domain = self.domain(scope)?;
            let node = self.require_node(domain.container)?;
            if !(first < self.enter[node] && self.exit[node] < last) {
                return Err(binding_error(domain.container, E::ScopeAssociation));
            }
        }
        Ok(())
    }
    fn roots(&self, region: &CapturedScope) -> ResultOf<Tree, Vec<usize>> {
        let domain = self.domain(&region.domain())?;
        let node = self.require_node(domain.container)?;
        if region.role() == ScopeRole::Domain {
            return Ok(self.graph.children[node].clone());
        }
        if self.semantic.contains(region) && !self.result.inline.iter().any(|s| &s.scope == region)
        {
            let (first, last) = self.semantic_neighbors(region);
            return self.gap_roots(&region.domain(), first, last);
        }
        let pair = self
            .result
            .inline
            .iter()
            .find(|s| &s.scope == region)
            .ok_or_else(|| binding_error(domain.container, E::ScopeCoverage))?;
        let start = self.graph.index(pair.start).expect("resolved boundary");
        let end = self.graph.index(pair.end).expect("resolved boundary");
        let first = self.graph.children[node]
            .iter()
            .position(|&n| n == start)
            .ok_or_else(|| binding_error(pair.start, E::ScopeAssociation))?;
        let last = self.graph.children[node]
            .iter()
            .position(|&n| n == end)
            .ok_or_else(|| binding_error(pair.end, E::ScopeAssociation))?;
        if first > last {
            return Err(binding_error(pair.start, E::ScopeBoundaryOrder));
        }
        Ok(self.graph.children[node][first..=last].to_vec())
    }
    fn region_domains(&self, region: &CapturedScope) -> ResultOf<Tree, Vec<CapturedScope>> {
        self.domains_in_roots(&region.domain(), self.roots(region)?)
    }
    fn domains_in_roots(
        &self,
        domain: &CapturedScope,
        roots: Vec<usize>,
    ) -> ResultOf<Tree, Vec<CapturedScope>> {
        let mut result = Vec::new();
        let mut seen = vec![false; self.graph.nodes.len()];
        let mut pending = roots;
        while let Some(index) = pending.pop() {
            if seen[index] {
                continue;
            }
            seen[index] = true;
            if let Some(domain) = self.node_domains[index]
                .iter()
                .map(|&i| &self.result.domains[i])
                .find(|d| &d.scope != domain && same_capture(&d.scope, domain))
            {
                self.require_node(domain.container)?;
                push_unique(&mut result, domain.scope.clone());
            } else {
                pending.extend(self.graph.children[index].iter().copied());
            }
        }
        Ok(result)
    }
    fn actual(
        &self,
        domain: &CapturedScope,
        roots: Vec<usize>,
        strict: bool,
        gap: Option<GapScan<'_>>,
        region: Option<&CapturedScope>,
    ) -> ResultOf<Tree, Vec<Actual<Tree::Node>>> {
        let bounds = roots
            .first()
            .zip(roots.last())
            .map(|(&first, &last)| ((self.enter[first], 0), (self.exit[last], 5)));
        let mut actual = self.actual_physical(domain, roots, strict, gap)?;
        if gap.is_some() {
            return Ok(actual);
        }
        for slot in &self.result.slots {
            if slot.captured.domain() != *domain {
                continue;
            }
            if let Some(region) = region
                && region.role() == ScopeRole::Inline
                && !(slot.captured.enter_position() < region.exit_position()
                    && region.enter_position() < slot.captured.exit_position())
            {
                continue;
            }
            let included = bounds.is_none_or(|(first, last)| first <= slot.key && slot.key <= last);
            if !included {
                continue;
            }
            for event in [
                Event::ParagraphStart(slot.captured.clone()),
                Event::ParagraphEnd(slot.captured.clone()),
            ] {
                self.require_expected(&event)?;
                let at = actual
                    .iter()
                    .position(|row| {
                        row.key > slot.key
                            || row.key == slot.key
                                && match &row.owner {
                                    ActualOwner::Physical(_) => true,
                                    ActualOwner::Semantic { paragraph, .. } => {
                                        paragraph.domain() != slot.captured.domain()
                                            || event_position(&row.event) > event_position(&event)
                                    }
                                }
                    })
                    .unwrap_or(actual.len());
                actual.insert(
                    at,
                    Actual {
                        event,
                        owner: ActualOwner::Semantic {
                            container: slot.container,
                            paragraph: slot.captured.clone(),
                        },
                        key: slot.key,
                    },
                );
            }
        }
        Ok(actual)
    }

    fn actual_physical(
        &self,
        domain: &CapturedScope,
        roots: Vec<usize>,
        strict: bool,
        mut gap: Option<GapScan<'_>>,
    ) -> ResultOf<Tree, Vec<Actual<Tree::Node>>> {
        let mut result = Vec::new();
        // A region has one bounded traversal. For an empty slot, a shared node
        // seen before its authentic left anchor must be revisited if encountered
        // inside the candidate gap. Two epochs remain bounded by graph size.
        let mut seen = vec![false; self.graph.nodes.len() * 2];
        let mut pending: Vec<_> = roots.into_iter().rev().map(|i| (i, false, false)).collect();
        while let Some((index, exiting, opaque)) = pending.pop() {
            let node = self.graph.nodes[index];
            if exiting {
                for (_, event) in self.node_events[index]
                    .end
                    .iter()
                    .filter(|(d, _)| d == domain)
                {
                    let item = Actual {
                        event: event.clone(),
                        owner: ActualOwner::Physical(node),
                        key: (
                            if opaque {
                                self.enter[index]
                            } else {
                                self.exit[index]
                            },
                            4,
                        ),
                    };
                    if self.observe(item, strict, &mut gap, &mut result)? {
                        return Ok(result);
                    }
                }
                continue;
            }
            let epoch = usize::from(gap.as_ref().is_some_and(|g| g.inside));
            let slot = index + epoch * self.graph.nodes.len();
            if seen[slot] {
                continue;
            }
            seen[slot] = true;
            for (_, event) in self.node_events[index]
                .start
                .iter()
                .filter(|(d, _)| d == domain)
            {
                let item = Actual {
                    event: event.clone(),
                    owner: ActualOwner::Physical(node),
                    key: (self.enter[index], 0),
                };
                if self.observe(item, strict, &mut gap, &mut result)? {
                    return Ok(result);
                }
            }
            for (_, event) in self.node_events[index]
                .body
                .iter()
                .filter(|(d, _)| d == domain)
            {
                let item = Actual {
                    event: event.clone(),
                    owner: ActualOwner::Physical(node),
                    key: (self.enter[index], 1),
                };
                if self.observe(item, strict, &mut gap, &mut result)? {
                    return Ok(result);
                }
            }
            let child = self.node_domains[index]
                .iter()
                .map(|&i| &self.result.domains[i])
                .find(|d| &d.scope != domain && same_capture(&d.scope, domain));
            if let Some(child) = child {
                if strict {
                    self.require_node(node)?;
                }
                if matches!(
                    child.scope.domain_definition(),
                    Some(CapturedDomainDefinition::Block)
                ) && child.scope.parent().is_some_and(|p| &p.domain() == domain)
                {
                    for (event, key) in [
                        (
                            Event::BlockStart(child.scope.clone()),
                            (self.enter[index], 2),
                        ),
                        (Event::BlockEnd(child.scope.clone()), (self.exit[index], 0)),
                    ] {
                        if self.observe(
                            Actual {
                                event,
                                owner: ActualOwner::Physical(node),
                                key,
                            },
                            strict,
                            &mut gap,
                            &mut result,
                        )? {
                            return Ok(result);
                        }
                    }
                }
                pending.push((index, true, true));
            } else {
                pending.push((index, true, false));
                pending.extend(
                    self.graph.children[index]
                        .iter()
                        .rev()
                        .map(|&i| (i, false, false)),
                );
            }
        }
        if gap
            .as_ref()
            .is_some_and(|g| !g.inside || g.next.is_some() && !g.complete)
        {
            return Err(binding_error(
                self.domain(domain)?.container,
                E::ScopeAssociation,
            ));
        }
        Ok(result)
    }
    fn observe(
        &self,
        item: Actual<Tree::Node>,
        strict: bool,
        gap: &mut Option<GapScan<'_>>,
        result: &mut Vec<Actual<Tree::Node>>,
    ) -> ResultOf<Tree, bool> {
        if let Some(gap) = gap {
            if !gap.inside {
                if gap.previous == Some(&item.event) {
                    self.require_node(item.subject())?;
                    gap.inside = true;
                    result.push(item);
                }
            } else {
                self.require_node(item.subject())?;
                if gap.next != Some(&item.event) {
                    return Err(binding_error(item.subject(), E::ScopeAssociation));
                }
                gap.complete = true;
                result.push(item);
                return Ok(true);
            }
        } else {
            if strict {
                self.require_node(item.subject())?;
            }
            result.push(item);
        }
        Ok(false)
    }
    fn expected_events(&self, entries: &[PositionedEvent]) -> ResultOf<Tree, Vec<Event>> {
        let mut expected = Vec::new();
        for entry in entries {
            let event = entry.event.clone();
            if let Event::ParagraphStart(p) | Event::ParagraphEnd(p) = &event {
                let index = self
                    .paragraph(p)
                    .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?;
                if self.member_class(p) == ParagraphClass::AssociationOnly {
                    if !self.metadata.contains(p) {
                        return Err(binding_error(
                            self.paragraphs[index].container,
                            E::ScopeAssociation,
                        ));
                    }
                    continue;
                }
            }
            expected.push(event);
        }
        Ok(expected)
    }
    fn native_roots(&self, region: &NativeRegion<Tree::Node>) -> Vec<usize> {
        self.graph.children[region.owner][region.first..=region.last].to_vec()
    }
    fn native_selected(&self, region: &NativeRegion<Tree::Node>) -> &[PositionedEvent] {
        let events = &self.expected[&region.domain()].events;
        let first = region.start.event_position();
        let last = region.end.event_position();
        let start = events.partition_point(|row| row.position < first);
        let end = events.partition_point(|row| row.position <= last);
        &events[start..end]
    }
    fn compare_native_region(&self, region: &NativeRegion<Tree::Node>) -> ResultOf<Tree, ()> {
        let expected = self.expected_events(self.native_selected(region))?;
        let mut actual = self.actual(
            &region.domain(),
            self.native_roots(region),
            true,
            None,
            None,
        )?;
        let lower = (self.enter[self.require_node(region.pair.0)?], 1);
        let upper = (self.enter[self.require_node(region.pair.1)?], 1);
        actual.retain(|row| lower <= row.key && row.key <= upper);
        if expected.iter().ne(actual.iter().map(|row| &row.event)) {
            return Err(binding_error(region.flow, E::ParticipantOrder));
        }
        Ok(())
    }
    fn compare_region(&self, region: &CapturedScope) -> ResultOf<Tree, ()> {
        let mut expected = self.expected_events(self.selected(region))?;
        let semantic = self.semantic.contains(region)
            && !self.result.inline.iter().any(|s| &s.scope == region);
        let mut actual = self.actual(
            &region.domain(),
            self.roots(region)?,
            true,
            None,
            Some(region),
        )?;
        if semantic {
            // Without physical edges, authentic adjacent flow events close
            // both semantic edges. They prove order, not allocated BP extent.
            let (before, after) = self.semantic_neighbors(region);
            if let Some(before) = before {
                expected.insert(0, before.clone());
            }
            if let Some(after) = after {
                expected.push(after.clone());
            }
            let first = before.map_or(Some(0), |event| {
                actual.iter().position(|a| &a.event == event)
            });
            let last = after.map_or(Some(actual.len()), |event| {
                actual
                    .iter()
                    .rposition(|a| &a.event == event)
                    .map(|i| i + 1)
            });
            let (Some(first), Some(last)) = (first, last) else {
                return Err(binding_error(
                    self.domain(&region.domain())?.container,
                    E::ScopeAssociation,
                ));
            };
            if first > last {
                return Err(binding_error(
                    self.domain(&region.domain())?.container,
                    E::ParticipantOrder,
                ));
            }
            actual = actual.drain(first..last).collect();
        }
        if region.role() == ScopeRole::Inline && !semantic {
            let pair = self
                .result
                .inline
                .iter()
                .find(|s| &s.scope == region)
                .expect("resolved region endpoints");
            let lower = (self.enter[self.require_node(pair.start)?], 1);
            let upper = (self.enter[self.require_node(pair.end)?], 1);
            actual.retain(|observation| lower <= observation.key && observation.key <= upper);
        }
        if expected.iter().ne(actual.iter().map(|a| &a.event)) {
            return Err(binding_error(
                self.domain(&region.domain())?.container,
                E::ParticipantOrder,
            ));
        }
        Ok(())
    }
    fn structural_neighbors(
        &self,
        domain: &CapturedScope,
        entry: &CapturedEventPosition,
        exit: &CapturedEventPosition,
    ) -> ResultOf<Tree, (Option<CapturedScope>, Option<CapturedScope>)> {
        let container = self.domain(domain)?.container;
        // These issued positions are capture CUTS, not identities of the event
        // which follows them. Raw paragraph markers supply no native boundary.
        if !entry.same_capture(&domain.enter_position())
            || !exit.same_capture(&domain.enter_position())
            || entry <= &domain.enter_position()
            || exit < entry
            || &domain.exit_position() < exit
        {
            return Err(binding_error(container, E::RunMembership));
        }
        let mut before = None;
        let mut after = None;
        for event in domain
            .flow_events()
            .ok_or_else(|| binding_error(container, E::RunMembership))?
        {
            let Event::BlockStart(scope) = event else {
                continue;
            };
            if scope == *domain
                || scope.domain_definition() != Some(CapturedDomainDefinition::Block)
                || scope.parent().map(|parent| parent.domain()).as_ref() != Some(domain)
            {
                continue;
            }
            if exit <= &scope.enter_position() {
                // Canonical flow order makes the first such block the actual
                // right neighbour. Merely seeing it creates no native demand.
                after.get_or_insert(scope);
            } else if &scope.exit_position() < entry {
                before = Some(scope);
            } else {
                return Err(binding_error(container, E::RunMembership));
            }
        }
        Ok((before, after))
    }

    fn structural_window(
        &self,
        domain: &CapturedScope,
        before: Option<&CapturedScope>,
        after: Option<&CapturedScope>,
    ) -> ResultOf<Tree, std::ops::Range<usize>> {
        let container = self.domain(domain)?.container;
        let owner = self.require_node(container)?;
        let rank = |scope: &CapturedScope| {
            // Only a material partition/empty slot needs the source neighbour's
            // actual native container and unique containing child branch.
            let rank = self.native_event_gap(domain, &Event::BlockStart(scope.clone()), false)?;
            let child = self.graph.children[owner][rank];
            if !self
                .tree
                .layout_input(self.graph.nodes[child])
                .as_box()
                .is_some_and(|input| {
                    input.display != crate::Display::None
                        && input.position != crate::Position::Absolute
                        && input.float.is_none()
                        && !input.display.is_inline_level()
                })
            {
                return Err(binding_error(container, E::RunMembership));
            }
            Ok(rank)
        };
        let start = before.map(&rank).transpose()?.map_or(0, |rank| rank + 1);
        let end = after
            .map(rank)
            .transpose()?
            .unwrap_or(self.graph.children[owner].len());
        if start > end {
            return Err(binding_error(container, E::RunMembership));
        }
        Ok(start..end)
    }

    fn prepare_runs(&mut self) -> ResultOf<Tree, ()> {
        for (index, binding) in self.paragraphs.iter().enumerate() {
            let scheduled = self.result.paragraphs[index].class != ParagraphClass::AssociationOnly
                || binding
                    .prepared
                    .captured_owner()
                    .is_some_and(|owner| owner.joint_flow().is_some());
            // Raw source-only association metadata has no line/run operation.
            // Its genuine markers are still checked by validate_metadata.
            if !scheduled {
                continue;
            }
            let owner = self
                .graph
                .index(binding.container)
                .ok_or_else(|| binding_error(binding.container, E::ScopeAssociation))?;
            let children = &self.graph.children[owner];
            let mut covered = 0;
            let mut horizon = None;
            for segment in binding.prepared.source_segments() {
                let (lower, upper) = if let Some(captured) = binding.prepared.captured_owner() {
                    let (Some(entry), Some(exit)) =
                        (segment.entry_position(), segment.exit_position())
                    else {
                        return Err(binding_error(binding.container, E::RunMembership));
                    };
                    let domain = captured.domain();
                    let (before, after) = self.structural_neighbors(&domain, &entry, &exit)?;
                    let window = if binding.prepared.source_segments().len() == 1
                        && !binding.nodes.is_empty()
                    {
                        // This binding already passed complete host-run admission.
                        // No child-domain association or path to a dormant remote
                        // occurrence is needed to borrow its one real node window.
                        let first = children
                            .iter()
                            .position(|&node| self.graph.nodes[node] == binding.nodes[0])
                            .ok_or_else(|| binding_error(binding.container, E::RunMembership))?;
                        let last = children
                            .iter()
                            .position(|&node| {
                                self.graph.nodes[node]
                                    == *binding.nodes.last().expect("nonempty native inventory")
                            })
                            .ok_or_else(|| binding_error(binding.container, E::RunMembership))?;
                        first..last + 1
                    } else {
                        self.structural_window(&domain, before.as_ref(), after.as_ref())?
                    };
                    // Each represented occurrence must fall in the SAME issued
                    // source interval and actual native window. Numeric offsets
                    // or a designated facts owner cannot substitute for custody.
                    let incidence = |node, position: &CapturedEventPosition| {
                        let rank = children
                            .iter()
                            .position(|&child| self.graph.nodes[child] == node)
                            .ok_or_else(|| binding_error(binding.container, E::RunMembership))?;
                        let source_inside = &entry <= position && position < &exit;
                        if source_inside != window.contains(&rank) {
                            return Err(binding_error(binding.container, E::RunMembership));
                        }
                        Ok(())
                    };
                    for row in self.origins.iter().filter(|row| row.paragraph == index) {
                        incidence(row.node, &row.position)?;
                    }
                    for row in self.atomics.iter().filter(|row| row.paragraph == index) {
                        incidence(row.node, &row.position)?;
                    }
                    for row in self.controls.iter().filter(|row| row.paragraph == index) {
                        incidence(row.node, &row.captured.event_position())?;
                    }
                    (window.start, window.end)
                } else {
                    if binding.prepared.source_segments().len() != 1 || binding.nodes.is_empty() {
                        return Err(binding_error(binding.container, E::RunMembership));
                    }
                    let lower = children
                        .iter()
                        .position(|&node| self.graph.nodes[node] == binding.nodes[0])
                        .ok_or_else(|| binding_error(binding.container, E::RunMembership))?;
                    (lower, lower + binding.nodes.len())
                };
                horizon = Some(upper);
                if lower > upper || upper > children.len() {
                    return Err(binding_error(binding.container, E::RunMembership));
                }
                // An issued segment cannot silently cross a real ordinary
                // child block in the host. Float/absolute domains are distinct.
                for &child in &children[lower..upper] {
                    if self
                        .tree
                        .layout_input(self.graph.nodes[child])
                        .as_box()
                        .is_some_and(|input| {
                            input.display != crate::Display::None
                                && input.position != crate::Position::Absolute
                                && input.float.is_none()
                                && !input.display.is_inline_level()
                        })
                    {
                        return Err(binding_error(binding.container, E::RunMembership));
                    }
                }
                let first = covered;
                while let Some(node) = binding.nodes.get(covered) {
                    let rank = children
                        .iter()
                        .position(|&child| self.graph.nodes[child] == *node)
                        .ok_or_else(|| binding_error(binding.container, E::RunMembership))?;
                    if rank < lower {
                        return Err(binding_error(binding.container, E::RunMembership));
                    }
                    if rank >= upper {
                        break;
                    }
                    covered += 1;
                }
                let native = if first < covered {
                    let start = children
                        .iter()
                        .position(|&child| self.graph.nodes[child] == binding.nodes[first])
                        .expect("validated native inventory");
                    let end = children
                        .iter()
                        .position(|&child| self.graph.nodes[child] == binding.nodes[covered - 1])
                        .expect("validated native inventory")
                        + 1;
                    start..end
                } else {
                    lower..lower
                };
                self.result.runs.try_reserve(1).map_err(|_| {
                    crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(binding.container),
                        crate::LayoutOperation::TextCapture,
                        crate::LayoutErrorKindOf::Allocation,
                    )
                })?;
                self.result.runs.push(NativeRun {
                    paragraph: index,
                    nodes: first..covered,
                    children: native,
                    segment,
                });
            }
            if binding.prepared.terminal_edge() == surgeist_text::ParagraphTerminalEdge::BlockEnd
                && horizon.is_some_and(|end| {
                    children[end..].iter().any(|&node| {
                        self.tree
                            .layout_input(self.graph.nodes[node])
                            .as_box()
                            .is_none_or(|input| {
                                input.display != crate::Display::None
                                    && input.position != crate::Position::Absolute
                            })
                    })
                })
            {
                return Err(binding_error(binding.container, E::TerminalBlockEdge));
            }
            if covered != binding.nodes.len() {
                return Err(binding_error(binding.container, E::RunMembership));
            }
        }
        Ok(())
    }

    fn native_event_gap(
        &self,
        domain: &CapturedScope,
        event: &Event,
        after: bool,
    ) -> ResultOf<Tree, usize> {
        let owner = self.require_node(self.domain(domain)?.container)?;
        if matches!(event, Event::BlockStart(scope) | Event::BlockEnd(scope) if scope == domain) {
            return Ok(if matches!(event, Event::BlockEnd(_)) {
                self.graph.children[owner].len()
            } else {
                0
            });
        }
        let mut branch = self.require_node(self.event_node(event)?)?;
        while self.graph.parents[branch].first().copied() != Some(owner) {
            branch = *self.graph.parents[branch]
                .first()
                .ok_or_else(|| binding_error(self.graph.nodes[branch], E::ScopeAssociation))?;
        }
        let rank = self.graph.children[owner]
            .iter()
            .position(|&node| node == branch)
            .ok_or_else(|| binding_error(self.graph.nodes[owner], E::ScopeAssociation))?;
        Ok(rank + usize::from(after))
    }

    fn semantic_slots(&mut self) -> ResultOf<Tree, ()> {
        for (index, paragraph) in self.result.paragraphs.iter().enumerate() {
            let Some(owner) = paragraph.prepared.captured_owner() else {
                continue;
            };
            for member in owner.members() {
                if self.member_class(&member) != ParagraphClass::SemanticFlow {
                    continue;
                }
                let captured = &member;
                let domain = captured.domain();
                let flow = &self.expected[&domain];
                let &(first, last) = flow
                    .paragraphs
                    .get(captured)
                    .ok_or_else(|| binding_error(paragraph.container, E::ScopeAssociation))?;
                if last != first + 1 {
                    return Err(binding_error(paragraph.container, E::ScopeAssociation));
                }
                let before = flow.previous[first].map(|at| &flow.events[at].event);
                let after = flow.next[last].map(|at| &flow.events[at].event);
                for event in [before, after].into_iter().flatten() {
                    self.require_expected(event)?;
                }
                self.actual_physical(
                    &domain,
                    self.gap_roots(&domain, before, after)?,
                    false,
                    Some(GapScan {
                        previous: before,
                        next: after,
                        inside: before.is_none(),
                        complete: false,
                    }),
                )?;
                let owner = self.require_node(paragraph.container)?;
                let lower = before
                    .map(|event| self.native_event_gap(&domain, event, true))
                    .transpose()?
                    .unwrap_or(0);
                let upper = after
                    .map(|event| self.native_event_gap(&domain, event, false))
                    .transpose()?
                    .unwrap_or(self.graph.children[owner].len());
                if lower != upper {
                    return Err(binding_error(paragraph.container, E::ScopeAssociation));
                }
                for other in self.paragraphs.iter().filter(|other| {
                    other.container == paragraph.container && !other.nodes.is_empty()
                }) {
                    let start = self.graph.children[owner]
                        .iter()
                        .position(|&node| self.graph.nodes[node] == other.nodes[0])
                        .ok_or_else(|| binding_error(paragraph.container, E::RunMembership))?;
                    if start < lower && lower < start + other.nodes.len() {
                        return Err(binding_error(paragraph.container, E::RunMembership));
                    }
                }
                if let Some(other) = self
                    .result
                    .slots
                    .iter()
                    .find(|slot| slot.container == paragraph.container && slot.gap == lower)
                    && (other.captured.domain() != domain
                        || !other
                            .captured
                            .enter_position()
                            .same_capture(&captured.enter_position()))
                {
                    return Err(binding_error(paragraph.container, E::ScopeAssociation));
                }
                let key = if let Some(&child) = self.graph.children[owner].get(lower) {
                    (self.enter[child], 0)
                } else if let Some(&child) = self.graph.children[owner].last() {
                    (self.exit[child], 5)
                } else {
                    (self.enter[owner], 5)
                };
                self.result.slots.push(SemanticParagraphSlot {
                    captured: captured.clone(),
                    prepared: self.paragraphs[index].prepared.clone(),
                    container: paragraph.container,
                    gap: lower,
                    previous: before.map(event_position),
                    next: after.map(event_position),
                    key,
                });
            }
        }
        self.result.slots.sort_by(|a, b| {
            self.graph
                .index(a.container)
                .cmp(&self.graph.index(b.container))
                .then(a.gap.cmp(&b.gap))
                .then_with(|| {
                    a.captured
                        .enter_position()
                        .partial_cmp(&b.captured.enter_position())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        Ok(())
    }

    fn metadata_slot(&self, index: usize) -> ResultOf<Tree, ()> {
        let owner = self.result.paragraphs[index]
            .prepared
            .captured_owner()
            .expect("classified metadata is captured");
        if !owner
            .members()
            .all(|member| self.metadata.contains(&member))
        {
            return Err(binding_error(
                self.paragraphs[index].container,
                E::ScopeAssociation,
            ));
        }
        Ok(())
    }
    fn validate_metadata(&mut self) -> ResultOf<Tree, ()> {
        #[expect(
            clippy::mutable_key_type,
            reason = "Text handle hashing uses immutable capture identity and occurrence index; lazy cache contents never affect the key"
        )]
        let mut proven = HashSet::new();
        for (index, paragraph) in self.result.paragraphs.iter().enumerate() {
            let Some(owner) = paragraph.prepared.captured_owner() else {
                continue;
            };
            for member in owner.members() {
                if self.member_class(&member) != ParagraphClass::AssociationOnly {
                    continue;
                }
                let captured = &member;
                let domain = captured.domain();
                let flow = &self.expected[&domain];
                let &(first, last) = flow
                    .paragraphs
                    .get(captured)
                    .ok_or_else(|| binding_error(paragraph.container, E::ScopeAssociation))?;
                if last != first + 1 {
                    return Err(binding_error(paragraph.container, E::ScopeAssociation));
                }
                let previous = flow.previous[first];
                let next = flow.next[last];
                if proven.insert((domain.clone(), previous, next)) {
                    let before = previous.map(|i| &flow.events[i].event);
                    let after = next.map(|i| &flow.events[i].event);
                    let roots = self.gap_roots(&domain, before, after)?;
                    self.actual(
                        &domain,
                        roots,
                        false,
                        Some(GapScan {
                            previous: before,
                            next: after,
                            inside: before.is_none(),
                            complete: false,
                        }),
                        None,
                    )?;
                }
                self.metadata.insert(captured.clone());
                // Every supplied identity is checked even when several share a slot.
                self.require_node(self.paragraphs[index].container)?;
            }
        }
        Ok(())
    }
    fn event_node(&self, event: &Event) -> ResultOf<Tree, Tree::Node> {
        self.require_expected(event)?;
        let node = match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => self
                .member_node(p, matches!(event, Event::ParagraphEnd(_)))
                .ok_or_else(|| binding_error(self.graph.nodes[0], E::ScopeAssociation))?,
            Event::Origin { position, .. } => {
                self.origins
                    .iter()
                    .find(|o| &o.position == position)
                    .expect("required origin")
                    .node
            }
            Event::Atomic { position, .. } => {
                self.atomics
                    .iter()
                    .find(|a| &a.position == position)
                    .expect("required atomic")
                    .node
            }
            Event::Control(control) => {
                self.controls
                    .iter()
                    .find(|c| &c.captured == control)
                    .expect("required control")
                    .node
            }
            Event::BlockStart(scope) | Event::BlockEnd(scope) => self.domain(scope)?.container,
        };
        self.require_node(node)?;
        Ok(node)
    }
    fn gap_roots(
        &self,
        domain: &CapturedScope,
        previous: Option<&Event>,
        next: Option<&Event>,
    ) -> ResultOf<Tree, Vec<usize>> {
        let container = self.require_node(self.domain(domain)?.container)?;
        let semantic_gap = |event: Option<&Event>| match event {
            Some(Event::ParagraphStart(paragraph) | Event::ParagraphEnd(paragraph)) => self
                .result
                .slots
                .iter()
                .find(|slot| &slot.captured == paragraph)
                .map(|slot| slot.gap),
            _ => None,
        };
        let left_gap = semantic_gap(previous);
        let right_gap = semantic_gap(next);
        if left_gap.is_some() || right_gap.is_some() {
            let first = left_gap.map_or_else(
                || {
                    previous
                        .map(|event| self.native_event_gap(domain, event, false))
                        .transpose()
                        .map(|gap| gap.unwrap_or(0))
                },
                Ok,
            )?;
            let last = right_gap.map_or_else(
                || {
                    next.map(|event| self.native_event_gap(domain, event, true))
                        .transpose()
                        .map(|gap| gap.unwrap_or(self.graph.children[container].len()))
                },
                Ok,
            )?;
            if first > last {
                return Err(binding_error(
                    self.graph.nodes[container],
                    E::ScopeAssociation,
                ));
            }
            return Ok(self.graph.children[container][first..last].to_vec());
        }

        let left = previous
            .map(|e| self.event_node(e).and_then(|n| self.require_node(n)))
            .transpose()?;
        let right = next
            .map(|e| self.event_node(e).and_then(|n| self.require_node(n)))
            .transpose()?;
        let (Some(mut first), Some(mut last)) = (left, right) else {
            let anchor = left.or(right);
            if let Some(mut node) = anchor {
                while self.graph.parents[node].first().copied() != Some(container) {
                    node = *self.graph.parents[node].first().ok_or_else(|| {
                        binding_error(self.graph.nodes[node], E::ScopeAssociation)
                    })?;
                }
                let position = self.graph.children[container]
                    .iter()
                    .position(|&i| i == node)
                    .expect("unique child edge");
                return Ok(if left.is_some() {
                    self.graph.children[container][position..].to_vec()
                } else {
                    self.graph.children[container][..=position].to_vec()
                });
            }
            return Ok(self.graph.children[container].clone());
        };
        if first == last {
            return Ok(vec![first]);
        }
        while self.depth[first] > self.depth[last] {
            first = self.graph.parents[first][0];
        }
        while self.depth[last] > self.depth[first] {
            last = self.graph.parents[last][0];
        }
        while first != last {
            first = self.graph.parents[first][0];
            last = self.graph.parents[last][0];
        }
        let ancestor = first;
        let branch = |mut node: usize| {
            if node == ancestor {
                return None;
            }
            while self.graph.parents[node][0] != ancestor {
                node = self.graph.parents[node][0];
            }
            Some(node)
        };
        let (Some(first), Some(last)) = (
            branch(left.expect("left anchor")),
            branch(right.expect("right anchor")),
        ) else {
            return Ok(vec![ancestor]);
        };
        let first = self.graph.children[ancestor]
            .iter()
            .position(|&i| i == first)
            .expect("left branch");
        let last = self.graph.children[ancestor]
            .iter()
            .position(|&i| i == last)
            .expect("right branch");
        if first > last {
            return Err(binding_error(
                self.graph.nodes[ancestor],
                E::ScopeAssociation,
            ));
        }
        Ok(self.graph.children[ancestor][first..=last].to_vec())
    }
    fn selected(&self, scope: &CapturedScope) -> &[PositionedEvent] {
        let events = &self.expected[&scope.domain()].events;
        let start = scope.enter_position();
        let end = scope.exit_position();
        let first = events.partition_point(|event| event.position <= start);
        let last = events.partition_point(|event| event.position < end);
        &events[first..last]
    }

    fn visible(&self, event: &Event) -> bool {
        match event {
            Event::ParagraphStart(p) | Event::ParagraphEnd(p) => self.paragraph(p).map_or_else(
                || {
                    let (first, last) = self.expected[&p.domain()].paragraphs[p];
                    last != first + 1
                },
                |_| self.member_class(p) != ParagraphClass::AssociationOnly,
            ),
            _ => true,
        }
    }

    fn semantic_neighbors(&self, scope: &CapturedScope) -> (Option<&Event>, Option<&Event>) {
        let flow = &self.expected[&scope.domain()];
        let first = flow
            .events
            .partition_point(|row| row.position <= scope.enter_position());
        let last = flow
            .events
            .partition_point(|row| row.position < scope.exit_position());
        let before = first.checked_sub(1).and_then(|i| {
            if self.visible(&flow.events[i].event) {
                Some(i)
            } else {
                flow.previous[i]
            }
        });
        let after = (last < flow.events.len()).then_some(last).and_then(|i| {
            if self.visible(&flow.events[i].event) {
                Some(i)
            } else {
                flow.next[i]
            }
        });
        (
            before.map(|i| &flow.events[i].event),
            after.map(|i| &flow.events[i].event),
        )
    }
    fn prepare_observations(&mut self) -> ResultOf<Tree, ()> {
        for &node in self.graph.finish.iter().rev() {
            for &child in &self.graph.children[node] {
                if self.paths[child] == 1 {
                    self.depth[child] = self.depth[node] + 1;
                }
            }
        }
        for paragraph in &self.result.paragraphs {
            let Some(owner) = paragraph.prepared.captured_owner() else {
                continue;
            };
            for member in owner.members() {
                let (Some(first), Some(last)) = (
                    self.member_node(&member, false),
                    self.member_node(&member, true),
                ) else {
                    continue;
                };
                // Original raw markers retain their own incidence even when a
                // joint glyph/native node contains several members. They do not
                // become scheduling or paragraph-frontier capabilities.
                self.node_events[self.graph.index(first).expect("validated member node")]
                    .start
                    .push((member.domain(), Event::ParagraphStart(member.clone())));
                self.node_events[self.graph.index(last).expect("validated member node")]
                    .end
                    .push((member.domain(), Event::ParagraphEnd(member)));
            }
        }
        for row in &self.origins {
            self.node_events[self.graph.index(row.node).expect("validated origin")]
                .body
                .push((
                    row.member.domain(),
                    Event::Origin {
                        paragraph: row.member.clone(),
                        position: row.position.clone(),
                    },
                ));
        }
        for row in &self.controls {
            self.node_events[self.graph.index(row.node).expect("validated control")]
                .body
                .push((
                    row.captured.paragraph().domain(),
                    Event::Control(row.captured.clone()),
                ));
        }
        for row in &self.atomics {
            self.node_events[self.graph.index(row.node).expect("validated atomic")]
                .body
                .push((
                    row.member.domain(),
                    Event::Atomic {
                        paragraph: row.member.clone(),
                        position: row.position.clone(),
                    },
                ));
        }
        let domains = &self.result.domains;
        for events in &mut self.node_events {
            events
                .body
                .sort_unstable_by(|(a_domain, a), (b_domain, b)| {
                    let a_rank = domains
                        .iter()
                        .position(|domain| domain.scope == *a_domain)
                        .expect("indexed source domain");
                    let b_rank = domains
                        .iter()
                        .position(|domain| domain.scope == *b_domain)
                        .expect("indexed source domain");
                    a_rank.cmp(&b_rank).then_with(|| {
                        event_position(a)
                            .partial_cmp(&event_position(b))
                            .expect("one source domain has one capture")
                    })
                });
        }
        for (index, domain) in self.result.domains.iter().enumerate() {
            self.node_domains[self
                .graph
                .index(domain.container)
                .expect("validated container")]
            .push(index);
        }
        let mut domains = Vec::new();
        for (_, boundary) in &self.computed {
            push_unique(&mut domains, boundary.view().domain());
        }
        for region in &self.result.regions {
            push_unique(&mut domains, region.domain());
        }
        for region in self.native.iter().filter(|region| region.active) {
            push_unique(&mut domains, region.domain());
        }
        for paragraph in &self.result.paragraphs {
            if let Some(owner) = paragraph.prepared.captured_owner()
                && owner
                    .members()
                    .any(|member| self.member_class(&member) != ParagraphClass::Flow)
            {
                push_unique(&mut domains, owner.domain());
            }
        }
        for domain in domains {
            let events: Vec<_> = domain
                .flow_events()
                .expect("domain flow")
                .map(|event| PositionedEvent {
                    position: event_position(&event),
                    event,
                })
                .collect();
            #[expect(
                clippy::mutable_key_type,
                reason = "Text handle hashing uses immutable capture identity and occurrence index; lazy cache contents never affect the key"
            )]
            let mut paragraphs = HashMap::new();
            for (index, event) in events.iter().enumerate() {
                match &event.event {
                    Event::ParagraphStart(p) => {
                        paragraphs.insert(p.clone(), (index, index));
                    }
                    Event::ParagraphEnd(p) => {
                        paragraphs.get_mut(p).expect("owner paragraph nesting").1 = index;
                    }
                    _ => {}
                }
            }
            let visible = |event: &Event| match event {
                Event::ParagraphStart(p) | Event::ParagraphEnd(p) => self.paragraph(p).map_or_else(
                    || {
                        let (first, last) = paragraphs[p];
                        last != first + 1
                    },
                    |_| self.member_class(p) == ParagraphClass::Flow,
                ),
                _ => true,
            };
            let mut previous = vec![None; events.len()];
            let mut next = previous.clone();
            let mut last = None;
            for (index, event) in events.iter().enumerate() {
                previous[index] = last;
                if visible(&event.event) {
                    last = Some(index);
                }
            }
            last = None;
            for (index, event) in events.iter().enumerate().rev() {
                next[index] = last;
                if visible(&event.event) {
                    last = Some(index);
                }
            }
            self.expected.insert(
                domain,
                ExpectedFlow {
                    events,
                    paragraphs,
                    previous,
                    next,
                },
            );
        }
        // Source-neighbor Block slots used to locate supplied empty incidence
        // are C obligations, not new A regions.
        for paragraph in &self.result.paragraphs {
            let Some(owner) = paragraph.prepared.captured_owner() else {
                continue;
            };
            for member in owner.members() {
                if self.member_class(&member) == ParagraphClass::Flow {
                    continue;
                }
                let captured = &member;
                let flow = &self.expected[&captured.domain()];
                let (first, last) = flow.paragraphs[captured];
                for i in [flow.previous[first], flow.next[last]]
                    .into_iter()
                    .flatten()
                {
                    if let Event::BlockStart(child) | Event::BlockEnd(child) = &flow.events[i].event
                    {
                        activation_unique::<Tree, _>(
                            self.activation.as_deref_mut(),
                            &mut self.activation_storage.domains,
                            &mut self.domains,
                            child.clone(),
                            false,
                            self.graph.nodes[0],
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}
fn same_capture(first: &CapturedScope, second: &CapturedScope) -> bool {
    first
        .enter_position()
        .same_capture(&second.enter_position())
}
fn scope_ancestor(ancestor: &CapturedScope, scope: &CapturedScope) -> bool {
    let mut parent = scope.parent();
    while let Some(p) = parent {
        if &p == ancestor {
            return true;
        }
        parent = p.parent();
    }
    false
}
fn add_region(regions: &mut Vec<CapturedScope>, scope: CapturedScope) {
    if regions.iter().any(|r| {
        r.domain() == scope.domain()
            && r.enter_position() <= scope.enter_position()
            && scope.exit_position() <= r.exit_position()
    }) {
        return;
    }
    regions.retain(|r| {
        r.domain() != scope.domain()
            || !(scope.enter_position() <= r.enter_position()
                && r.exit_position() <= scope.exit_position())
    });
    regions.push(scope);
}
fn event_position(event: &Event) -> CapturedEventPosition {
    match event {
        Event::ParagraphStart(p) => p.enter_position(),
        Event::ParagraphEnd(p) => p.exit_position(),
        Event::Origin { position, .. } | Event::Atomic { position, .. } => position.clone(),
        Event::Control(c) => c.event_position(),
        Event::BlockStart(s) => s.enter_position(),
        Event::BlockEnd(s) => s.exit_position(),
    }
}
fn descendant<Node: Copy + Eq>(graph: &ValidationGraph<Node>, parent: Node, child: Node) -> bool {
    let (Some(parent), Some(child)) = (graph.index(parent), graph.index(child)) else {
        return false;
    };
    if parent == child {
        return false;
    }
    let mut seen = vec![false; graph.nodes.len()];
    let mut pending = graph.parents[child].clone();
    while let Some(node) = pending.pop() {
        if node == parent {
            return true;
        }
        if !seen[node] {
            seen[node] = true;
            pending.extend(graph.parents[node].iter().copied());
        }
    }
    false
}
fn graph_order<Node: Copy + Eq>(graph: &ValidationGraph<Node>) -> (Vec<usize>, Vec<usize>) {
    let mut enter = vec![0; graph.nodes.len()];
    let mut exit = enter.clone();
    let mut seen = vec![false; graph.nodes.len()];
    let mut pending = vec![(0, false)];
    let mut clock = 0;
    while let Some((node, done)) = pending.pop() {
        if done {
            exit[node] = clock;
            clock += 1;
            continue;
        }
        if seen[node] {
            continue;
        }
        seen[node] = true;
        enter[node] = clock;
        clock += 1;
        pending.push((node, true));
        pending.extend(graph.children[node].iter().rev().map(|&i| (i, false)));
    }
    (enter, exit)
}
fn interval_relation<T: PartialOrd>(start: &T, end: &T, first: &T, last: &T) -> u8 {
    if end < first {
        1
    } else if last < start {
        2
    } else if start < first && last < end {
        3
    } else if first < start && end < last {
        4
    } else {
        0
    }
}

fn validate_virtuals<Node: Copy + Eq>(paragraph: &TextParagraphBinding<Node>) -> Result<(), E> {
    for occurrence in paragraph.prepared.virtual_separators() {
        let view = occurrence.view();
        let scope = view.scope().captured_scope();
        let Some(captured) = paragraph.prepared.captured_owner() else {
            return Err(E::ScopeOccurrenceMembership);
        };
        if captured.domain() != scope.domain()
            || !captured.members().any(|member| {
                member
                    .scope_intersections()
                    .any(|intersection| intersection.scope() == scope)
            })
        {
            return Err(E::ScopeOccurrenceMembership);
        }
        let preceding: Vec<_> = view.preceding_controls().collect();
        let following: Vec<_> = view.following_controls().collect();
        let mut previous = None;
        for control in preceding.iter().chain(&following) {
            let Some(current) = paragraph
                .controls
                .iter()
                .find(|row| &row.occurrence == control)
                .and_then(|row| paragraph.nodes.iter().position(|node| *node == row.node))
            else {
                return Err(E::ScopeOccurrenceMembership);
            };
            if previous.is_some_and(|p| p >= current) {
                return Err(E::ScopeOccurrenceMembership);
            }
            previous = Some(current);
        }
        for (endpoint, local) in [
            (scope.entry_control(), preceding),
            (scope.exit_control(), following),
        ] {
            if let Some(endpoint) = endpoint
                && owns_member(&paragraph.prepared, &endpoint.paragraph())
                && !local
                    .iter()
                    .any(|control| control.captured_control().as_ref() == Some(&endpoint))
            {
                return Err(E::ScopeOccurrenceMembership);
            }
        }
    }
    Ok(())
}
