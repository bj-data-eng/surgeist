//! Off-side Text progress retained by the existing Layout computation session.

use super::TextParagraphBinding;
use super::scope_alignment::{EntryBacking, EntryFunding, ScopeAlignmentFacts, ScopeLease};
use crate::{LayoutScalar, NodeOutputOf, Point};
use surgeist_text::{PlacementCoverage, PlacementEntry, PlacementRequest};

enum RunLookup<'a, Node> {
    Exact(&'a [Node]),
    Starting(&'a [Node]),
    Segment(&'a surgeist_text::PreparedSourceSegment),
    EmptySegment(&'a surgeist_text::PreparedSourceSegment),
}

/// In-place sorting with fallible comparisons and moves. A refused inspection
/// stops before that source operation; no uncharged comparison follows it.
fn checked_sort<T, E>(
    values: &mut [T],
    mut compare: impl FnMut(&T, &T) -> Result<std::cmp::Ordering, E>,
    mut moving: impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    fn sift<T, E>(
        values: &mut [T],
        mut root: usize,
        end: usize,
        compare: &mut impl FnMut(&T, &T) -> Result<std::cmp::Ordering, E>,
        moving: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), E> {
        while let Some(mut child) = root.checked_mul(2).and_then(|value| value.checked_add(1)) {
            if child >= end {
                break;
            }
            if child + 1 < end && compare(&values[child], &values[child + 1])?.is_lt() {
                child += 1;
            }
            if !compare(&values[root], &values[child])?.is_lt() {
                break;
            }
            moving()?;
            values.swap(root, child);
            root = child;
        }
        Ok(())
    }
    let len = values.len();
    for root in (0..len / 2).rev() {
        sift(values, root, len, &mut compare, &mut moving)?;
    }
    for end in (1..values.len()).rev() {
        moving()?;
        values.swap(0, end);
        sift(values, 0, end, &mut compare, &mut moving)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalFrame<Node, S: LayoutScalar> {
    pub node: Node,
    pub origin: Point<f64>,
    pub normal_origin: Point<f64>,
    pub normal: crate::output::NormalGeometryOf<S>,
    pub measured_inset_basis: Option<crate::output::MeasuredInsetBasisOf<S>>,
    pub output: NodeOutputOf<S>,
}

#[derive(Clone)]
pub(crate) struct PendingParagraph<Node> {
    pub binding_index: usize,
    pub inline_boxes: super::AdmittedInlineBoxes,
    pub segment: surgeist_text::PreparedSourceSegment,
    pub line_box_start: crate::line_box::LineBoxContext,
    pub line_box_end: crate::line_box::LineBoxContext,
    pub container: Node,
    pub nodes: Vec<Node>,
    pub prepared: surgeist_text::PreparedText,
    pub entries: Vec<PlacementEntry>,
    pub wrap_style: crate::TextWrapStyle,
    pub program: Option<std::sync::Arc<super::fit::SelectionProgram>>,
    pub natural_advances: Vec<f64>,
    pub boxes: Vec<Option<super::boxes::BoxAssignment>>,
    pub bands: Vec<super::fit::LineBands>,
    pub flows: Vec<super::fit::LineFlow>,
    pub amounts: Vec<Vec<f64>>,
    pub scopes: super::scope_alignment::ScopeBases,
    pub encounters: Vec<Vec<super::fit::EncounterBasis>>,
    pub zones: Vec<super::fit::ZoneBasis>,
    pub phrases: Vec<Vec<super::fit::PhraseBasis>>,
    pub outflows: Vec<(usize, surgeist_text::PreparedAtomic, Node)>,
    pub indent: crate::text_indent::ResolvedTextIndent<f64>,
    pub indent_start: crate::text_indent::IndentLineState,
    pub indent_end: crate::text_indent::IndentLineState,
    pub hyphenation_start: crate::hyphenation::LineHyphenationState,
    pub hyphenation_end: crate::hyphenation::LineHyphenationState,
    pub tail: super::fit::TailContext,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LineBasis {
    natural_advance: f64,
    disposition: super::fit::LineDisposition,
    boxes: Option<super::boxes::BoxAssignment>,
    zone: super::fit::ZoneBasis,
    phrases: Vec<super::fit::PhraseBasis>,
    encounters: Vec<super::fit::EncounterBasis>,
    request: surgeist_text::LineSelectionRequest,
    amounts: Vec<f64>,
    scopes: super::scope_alignment::ScopeBasis,
    anchor: surgeist_text::LineAnchor,
    orientation: surgeist_text::LineOrientation,
    atomics: Vec<surgeist_text::AtomicPlacement>,
    bands: super::fit::LineBands,
    flow: super::fit::LineFlow,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PassBasis<Node, S: LayoutScalar> {
    pub wrapping: Option<std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    pub formatted: Option<super::formatted::FormattedPassBasis<Node, S>>,
    pub boxes: Vec<super::boxes::BoxRunUse<Node, S>>,
    pub line_boxes: Vec<(
        Node,
        crate::line_box::LineBoxContext,
        crate::line_box::LineBoxContext,
    )>,
    pub tail: Vec<(Node, super::fit::TailContext)>,
    pub frames: Vec<CanonicalFrame<Node, S>>,
    pub lines: Vec<LineBasis>,
    pub selections: Vec<(
        Node,
        crate::TextWrapStyle,
        Option<std::sync::Arc<super::fit::SelectionProgram>>,
    )>,
    pub absolute_baselines: Vec<(Node, f64)>,
    pub associations: std::sync::Arc<super::capture::ScopeAssociations<Node>>,
    pub scope_alignments: std::sync::Arc<super::scope_alignment::ScopeAlignmentSnapshot<S>>,
    pub hyphenation: Vec<(
        Node,
        crate::hyphenation::LineHyphenationState,
        crate::hyphenation::LineHyphenationState,
    )>,
    pub indentation: Vec<(
        Node,
        crate::text_indent::ResolvedTextIndent<f64>,
        crate::text_indent::IndentLineState,
        crate::text_indent::IndentLineState,
    )>,
}

impl<Node, S: LayoutScalar> PassBasis<Node, S> {
    pub(crate) fn admit_scope_comparison(
        &self,
        other: &Self,
        resources: &super::scope_alignment::ScopeAlignmentResources<S>,
    ) -> surgeist_text::Result<()> {
        let Some(limits) = resources.key_limits()? else {
            return Ok(());
        };
        self.associations
            .admit_inline_box_comparison(&other.associations, limits, resources)?;
        super::scope_alignment::admit_key_comparison(
            limits,
            self.scope_alignments.rows().len(),
            other.scope_alignments.rows().len(),
            resources,
        )?;
        if self.lines.len() == other.lines.len() {
            // Admit the descriptor preflight before walking the native line
            // keys. Existing non-scope payload/equality remains native.
            super::scope_alignment::ScopeAlignmentFacts::visit(
                resources,
                limits,
                self.lines.len(),
            )?;
            for (line, other_line) in self.lines.iter().zip(&other.lines) {
                if !self
                    .associations
                    .same_inline_box_families(&other.associations)
                    && let (Some(left), Some(right)) =
                        (&line.request.inline_boxes, &other_line.request.inline_boxes)
                {
                    let left = left
                        .scope_alignment_retention()
                        .map_or(0, |value| value.records());
                    let right = right
                        .scope_alignment_retention()
                        .map_or(0, |value| value.records());
                    super::scope_alignment::ScopeAlignmentFacts::visit(
                        resources,
                        limits,
                        left.min(right),
                    )?;
                }
                super::scope_alignment::admit_key_comparison(
                    limits,
                    line.scopes.comparison_rows(),
                    other_line.scopes.comparison_rows(),
                    resources,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct ScopeFragmentGroup<Node, S: LayoutScalar> {
    pub container: Node,
    pub prepared: surgeist_text::PreparedText,
    pub nodes: Vec<Node>,
    pub fragments: Vec<crate::InlineFragmentOutputEntryOf<Node, S>>,
}

/// This invocation-local state is part of the ordinary unpublished session.
/// An initial measurement pass supplies no canonical-frame proof. Later passes
/// use actual preceding canonical outputs and must reproduce their full basis
/// before any placed Text or completed batch can escape.
pub(crate) struct TextComputation<'input, Node, S: LayoutScalar> {
    pub wrapping: Option<std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    pub formatted: Option<super::formatted::FormattedState<Node, S>>,
    pub graph: Option<&'input crate::engine::validation::ValidationGraph<Node>>,
    pub paragraphs: &'input [TextParagraphBinding<Node>],
    pub associations: std::sync::Arc<super::capture::ScopeAssociations<Node>>,
    pub scope_resources: super::scope_alignment::ScopeAlignmentResources<S>,
    entry_funding: Option<EntryFunding<S>>,
    pub frames: std::sync::Arc<[CanonicalFrame<Node, S>]>,
    pub pending: Vec<PendingParagraph<Node>>,
    pub boxes: Vec<super::boxes::BoxRunUse<Node, S>>,
    pub reusable: std::sync::Arc<[super::fit::ReusableLine]>,
    pub absolute_baselines: Vec<(Node, f64)>,
    pub scope_fragments: Vec<ScopeFragmentGroup<Node, S>>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> TextComputation<'input, Node, S> {
    pub(crate) fn formatted_active(&self) -> bool {
        self.formatted
            .as_ref()
            .is_some_and(|state| !state.demanded.is_empty())
    }

    pub(crate) fn formatted_valid(&self) -> surgeist_text::Result<bool> {
        self.formatted
            .as_ref()
            .map_or(Ok(true), super::formatted::FormattedState::valid)
    }

    fn formatted_state(
        &mut self,
    ) -> surgeist_text::Result<&mut super::formatted::FormattedState<Node, S>> {
        if self.formatted.is_none() {
            self.formatted = Some(super::formatted::FormattedState::new_entry(
                self.entry_funding.as_ref(),
                self.frames.clone(),
                self.wrapping.clone(),
            )?);
        }
        let state = self
            .formatted
            .as_mut()
            .expect("installed actual Formatted state");
        state.frames = self.frames.clone();
        state.wrapping = self.wrapping.clone();
        Ok(state)
    }

    pub(crate) fn formatted_accept(
        &mut self,
        observed: &super::formatted::FormattedState<Node, S>,
    ) -> surgeist_text::Result<()> {
        self.formatted_state()?.accept(observed)
    }

    pub(crate) fn formatted_snapshot(
        &self,
        container: Node,
        nodes: &[Node],
        segment: &surgeist_text::PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: crate::AxisSizingRequestOf<f64>,
    ) -> surgeist_text::Result<super::formatted::FormattedTailSnapshot> {
        if let Some(state) = &self.formatted {
            state.snapshot_run(container, nodes, segment, input, mode)
        } else {
            // A checked Layout run has actual complete tree authority. No tail
            // collection or sidecar lookup is performed until a reached demand.
            Ok(super::formatted::FormattedTailSnapshot::known(
                segment.clone(),
                None,
            ))
        }
    }

    pub(crate) fn formatted_merge_run(
        &mut self,
        container: Node,
        nodes: &[Node],
        segment: &surgeist_text::PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: crate::AxisSizingRequestOf<f64>,
        uses: super::formatted::FormattedTailUses,
    ) -> surgeist_text::Result<Option<super::formatted::FormattedFlowItem<Node, S>>> {
        if self.formatted.is_none() && uses.demands.is_empty() && uses.roles.is_empty() {
            return Ok(None);
        }
        let item = self
            .formatted_state()?
            .merge_run(container, nodes, segment, input, mode, uses)?;
        Ok(self.formatted_active().then_some(item))
    }

    pub(crate) fn formatted_begin_block(
        &mut self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
    ) -> surgeist_text::Result<bool> {
        if !self.formatted_active() {
            return Ok(false);
        }
        self.formatted
            .as_mut()
            .unwrap()
            .push_scope(super::formatted::ComputeScope {
                node,
                input,
                anonymous,
                frame: None,
            })?;
        Ok(true)
    }

    pub(crate) fn formatted_end_block(
        &mut self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
        success: bool,
    ) -> surgeist_text::Result<()> {
        if let Some(state) = &mut self.formatted {
            state.finish_block(
                super::formatted::ComputeScope {
                    node,
                    input,
                    anonymous,
                    frame: None,
                },
                success,
            )?;
        }
        Ok(())
    }

    pub(crate) fn formatted_new_flow(&self) -> Option<super::formatted::FormattedFlow<Node, S>> {
        self.formatted_active().then(|| {
            super::formatted::FormattedFlow::new(
                self.formatted
                    .as_ref()
                    .and_then(|state| state.stack.last())
                    .and_then(|scope| scope.frame),
                self.entry_funding.as_ref(),
            )
        })
    }

    pub(crate) fn formatted_begin_flow(
        &mut self,
        input: crate::ComputeInputOf<S>,
        available: crate::AxisSizingRequestOf<S>,
        publication: bool,
    ) -> surgeist_text::Result<()> {
        if self.formatted_active() {
            self.formatted
                .as_mut()
                .expect("active Formatted owner")
                .begin_flow(super::formatted::FlowFrame {
                    input,
                    available,
                    publication,
                })?;
        }
        Ok(())
    }

    pub(crate) fn formatted_record_traversal(
        &mut self,
        flow: &Option<super::formatted::FormattedFlow<Node, S>>,
    ) -> surgeist_text::Result<()> {
        if let (Some(state), Some(flow)) = (&mut self.formatted, flow) {
            state.record(flow.try_clone(self.entry_funding.as_ref())?)?;
        }
        Ok(())
    }

    pub(crate) fn formatted_record_flow(
        &mut self,
        flow: &Option<super::formatted::FormattedFlow<Node, S>>,
    ) -> surgeist_text::Result<()> {
        if let (Some(state), Some(flow)) = (&mut self.formatted, flow) {
            state.select(flow.try_clone(self.entry_funding.as_ref())?)?;
        }
        Ok(())
    }

    pub(crate) fn formatted_child(
        &self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
    ) -> surgeist_text::Result<Option<super::formatted::FormattedFlowItem<Node, S>>> {
        if !self.formatted_active() {
            return Ok(None);
        }
        self.formatted
            .as_ref()
            .expect("active Formatted owner")
            .child(super::formatted::ComputeScope {
                node,
                input,
                anonymous,
                frame: None,
            })
            .map(Some)
    }

    /// Required ancestors must execute to reach the demanded owner. Descendant
    /// summaries are needed only inside that owner; unrelated sibling caches
    /// retain the ordinary geometry-only path.
    pub(crate) fn formatted_cache_required(&self, node: Node) -> surgeist_text::Result<bool> {
        if !self.formatted_active() {
            return Ok(false);
        }
        let Some(graph) = self.graph else {
            return Ok(true);
        };
        let funding = self.entry_funding.as_ref();
        let inspect = || funding.map_or(Ok(()), |funding| funding.visit(1));
        let index = |node| {
            for (index, &candidate) in graph.nodes.iter().enumerate() {
                inspect()?;
                if candidate == node {
                    return Ok::<_, surgeist_text::Error>(Some(index));
                }
            }
            Ok(None)
        };
        let Some(subject) = index(node)? else {
            return Ok(true);
        };
        let reaches = |start: usize, end: usize| {
            let mut pending_backing = EntryBacking::default();
            let mut pending = Vec::new();
            let mut seen_backing = EntryBacking::default();
            let mut seen = Vec::new();
            pending_backing.reserve_push(&mut pending, funding)?;
            pending.push(start);
            while !pending.is_empty() {
                inspect()?;
                let at = pending.pop().expect("nonempty pending ancestry");
                if at == end {
                    return Ok::<_, surgeist_text::Error>(true);
                }
                let mut visited = false;
                for &previous in &seen {
                    inspect()?;
                    if previous == at {
                        visited = true;
                        break;
                    }
                }
                if !visited {
                    seen_backing.reserve_push(&mut seen, funding)?;
                    seen.push(at);
                    for &child in &graph.children[at] {
                        inspect()?;
                        pending_backing.reserve_push(&mut pending, funding)?;
                        pending.push(child);
                    }
                }
            }
            Ok(false)
        };
        for &owner in &self.formatted.as_ref().unwrap().demanded {
            inspect()?;
            let Some(owner) = index(owner)? else {
                return Ok(true);
            };
            if reaches(owner, subject)? || reaches(subject, owner)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn formatted_cache_available(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> surgeist_text::Result<bool> {
        if !self.formatted_cache_required(node)? {
            return Ok(true);
        }
        self.formatted
            .as_ref()
            .map_or(Ok(false), |state| state.cache_available(node, input))
    }

    pub(crate) fn formatted_cache_matches(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) -> surgeist_text::Result<bool> {
        if !self.formatted_cache_required(node)? {
            return Ok(true);
        }
        self.formatted.as_ref().map_or(Ok(false), |state| {
            state.cache_matches(node, input, context, output)
        })
    }

    pub(crate) fn formatted_restore_cache(
        &mut self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> surgeist_text::Result<bool> {
        if !self.formatted_cache_required(node)? {
            return Ok(true);
        }
        self.formatted
            .as_mut()
            .map_or(Ok(false), |state| state.restore(node, input))
    }

    pub(crate) fn formatted_next_pass(&mut self) -> surgeist_text::Result<()> {
        if let Some(state) = &mut self.formatted {
            state.next_pass()?;
        }
        Ok(())
    }

    pub fn new(
        paragraphs: &'input [TextParagraphBinding<Node>],
        associations: std::sync::Arc<super::capture::ScopeAssociations<Node>>,
        scope_resources: super::scope_alignment::ScopeAlignmentResources<S>,
    ) -> Self {
        Self {
            wrapping: None,
            formatted: None,
            graph: None,
            paragraphs,
            associations,
            scope_resources,
            entry_funding: None,
            frames: std::sync::Arc::from([]),
            pending: Vec::new(),
            boxes: Vec::new(),
            reusable: std::sync::Arc::from([]),
            absolute_baselines: Vec::new(),
            scope_fragments: Vec::new(),
        }
    }

    /// Share the existing immutable preparation/frame owners directly instead
    /// of allocating empty Arc owners and immediately overwriting them.
    pub(crate) fn staging(
        &self,
        formatted: Option<super::formatted::FormattedState<Node, S>>,
        entry_funding: Option<&EntryFunding<S>>,
    ) -> Self {
        Self {
            wrapping: self.wrapping.clone(),
            formatted,
            graph: self.graph,
            paragraphs: self.paragraphs,
            associations: self.associations.clone(),
            scope_resources: self.scope_resources.clone(),
            frames: self.frames.clone(),
            entry_funding: entry_funding.cloned(),
            pending: Vec::new(),
            boxes: Vec::new(),
            reusable: self.reusable.clone(),
            absolute_baselines: Vec::new(),
            scope_fragments: Vec::new(),
        }
    }

    fn entry_anchor(&self) -> surgeist_text::Result<Option<&'input surgeist_text::PreparedText>> {
        let limits = self.scope_resources.key_limits()?;
        for binding in self.paragraphs {
            if let Some(limits) = limits {
                self.scope_resources.visit(limits, 1)?;
            }
            if binding.prepared.scope_alignment_retention().is_some() {
                return Ok(Some(&binding.prepared));
            }
        }
        if limits.is_some() {
            return Err(super::scope_alignment::refusal(
                surgeist_text::ScopeAlignmentError::MissingCapture,
            ));
        }
        Ok(self.paragraphs.first().map(|binding| &binding.prepared))
    }

    pub(crate) fn prepare_entry(&mut self) -> surgeist_text::Result<()> {
        if let Some(prepared) = self.entry_anchor()? {
            self.scope_resources = self.scope_resources.entry_view(prepared)?;
        }
        Ok(())
    }

    pub(crate) fn entry_funding(&self) -> surgeist_text::Result<Option<EntryFunding<S>>> {
        self.entry_anchor()?.map_or(Ok(None), |prepared| {
            EntryFunding::new(self.scope_resources.clone(), prepared.clone())
        })
    }

    pub(crate) fn entry_case_funding(&self) -> Option<&EntryFunding<S>> {
        self.entry_funding.as_ref()
    }

    fn lookup_row<E>(
        &self,
        container: Node,
        lookup: RunLookup<'_, Node>,
        mut inspect: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&super::capture::NativeRun>, E> {
        for row in self.associations.runs() {
            inspect()?;
            inspect()?;
            let binding = &self.paragraphs[row.paragraph];
            if binding.container != container {
                continue;
            }
            let matches = match &lookup {
                RunLookup::Exact(nodes) | RunLookup::Starting(nodes) => {
                    if row.nodes.is_empty() {
                        false
                    } else {
                        let expected = &binding.nodes[row.nodes.clone()];
                        let length_matches = match &lookup {
                            RunLookup::Exact(_) => expected.len() == nodes.len(),
                            _ => expected.len() <= nodes.len(),
                        };
                        let mut equal = length_matches;
                        if equal {
                            for (actual, expected) in nodes.iter().zip(expected) {
                                inspect()?;
                                if actual != expected {
                                    equal = false;
                                    break;
                                }
                            }
                        }
                        equal
                    }
                }
                RunLookup::Segment(segment) => row.segment == **segment,
                RunLookup::EmptySegment(segment) => {
                    row.nodes.is_empty() && row.segment == **segment
                }
            };
            if matches {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn ordinary_run(
        &self,
        container: Node,
        lookup: RunLookup<'_, Node>,
    ) -> Option<super::AdmittedRun<'input, Node>> {
        match self.lookup_row(
            container,
            lookup,
            || Ok::<(), core::convert::Infallible>(()),
        ) {
            Ok(row) => row.map(|row| self.associations.admitted_run(self.paragraphs, row)),
            Err(never) => match never {},
        }
    }

    fn entry_lookup(
        &self,
        container: Node,
        lookup: RunLookup<'_, Node>,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<super::AdmittedRun<'input, Node>>> {
        let mut inspect = || funding.map_or(Ok(()), |funding| funding.visit(1));
        let Some(row) = self.lookup_row(container, lookup, &mut inspect)? else {
            return Ok(None);
        };
        self.associations
            .try_admitted_run(self.paragraphs, row, inspect)
            .map(Some)
    }

    pub(crate) fn entry_run(
        &self,
        container: Node,
        nodes: &[Node],
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<super::AdmittedRun<'input, Node>>> {
        self.entry_lookup(container, RunLookup::Exact(nodes), funding)
    }

    pub(crate) fn entry_run_starting(
        &self,
        container: Node,
        nodes: &[Node],
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<super::AdmittedRun<'input, Node>>> {
        self.entry_lookup(container, RunLookup::Starting(nodes), funding)
    }

    /// The issued section is the authority even for an empty native window.
    pub(crate) fn entry_run_from_segment(
        &self,
        container: Node,
        segment: &surgeist_text::PreparedSourceSegment,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<super::AdmittedRun<'input, Node>>> {
        self.entry_lookup(container, RunLookup::Segment(segment), funding)
    }

    pub(crate) fn entry_semantic_run(
        &self,
        container: Node,
        segment: &surgeist_text::PreparedSourceSegment,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<super::AdmittedRun<'input, Node>>> {
        self.entry_lookup(container, RunLookup::EmptySegment(segment), funding)
    }

    pub(crate) fn run(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<super::AdmittedRun<'input, Node>> {
        self.ordinary_run(container, RunLookup::Exact(nodes))
    }

    pub(crate) fn run_starting(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<super::AdmittedRun<'input, Node>> {
        self.ordinary_run(container, RunLookup::Starting(nodes))
    }

    pub(crate) fn semantic_slots<M>(
        &self,
        container: Node,
        funding: Option<&EntryFunding<S>>,
    ) -> crate::LayoutResultOf<Node, (Vec<super::capture::NativeRunSlot>, Option<ScopeLease>), S, M>
    {
        let inspect = |count| -> crate::LayoutResultOf<Node, (), S, M> {
            funding
                .map_or(Ok(()), |funding| funding.visit(count))
                .map_err(|error| {
                    crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(container),
                        crate::LayoutOperation::TextCapture,
                        crate::LayoutErrorKindOf::Text(error),
                    )
                })
        };
        let mut count = 0;
        for row in self.associations.runs() {
            inspect(1)?;
            if row.nodes.is_empty() {
                inspect(1)?;
                if self.paragraphs[row.paragraph].container == container {
                    count += 1;
                }
            }
        }
        let lease;
        let mut slots = if let Some(funding) = funding.filter(|_| count != 0) {
            let (slots, storage) = funding.buffer(count).map_err(|error| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(container),
                    crate::LayoutOperation::TextCapture,
                    crate::LayoutErrorKindOf::Text(error),
                )
            })?;
            lease = Some(storage);
            slots
        } else {
            let mut slots = Vec::new();
            slots.try_reserve_exact(count).map_err(|_| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(container),
                    crate::LayoutOperation::TextCapture,
                    crate::LayoutErrorKindOf::Allocation,
                )
            })?;
            lease = None;
            slots
        };
        for row in self.associations.runs() {
            inspect(1)?;
            if row.nodes.is_empty() {
                inspect(1)?;
                if self.paragraphs[row.paragraph].container == container {
                    inspect(1)?;
                    slots.push(super::capture::NativeRunSlot {
                        gap: row.children.start,
                        segment: row.segment.clone(),
                    });
                }
            }
        }
        checked_sort(
            &mut slots,
            |a, b| {
                inspect(2)?;
                Ok(a.gap.cmp(&b.gap))
            },
            || inspect(2),
        )?;
        let ordering_error =
            || super::binding_error(container, super::TextBindingError::ScopeAssociation);
        let mut first = 0;
        while first < slots.len() {
            inspect(1)?;
            let gap = slots[first].gap;
            let mut end = first + 1;
            while end < slots.len() {
                inspect(1)?;
                if slots[end].gap != gap {
                    break;
                }
                end += 1;
            }
            if end - first > 1 {
                // A native gap alone cannot order distinct source operations.
                // Admit the actual issued positions before the pure comparison;
                // a sole scheduled slot has no such ordering dependency.
                inspect(1)?;
                let position = slots[first]
                    .segment
                    .entry_position()
                    .ok_or_else(ordering_error)?;
                for slot in &slots[first + 1..end] {
                    inspect(1)?;
                    let other = slot.segment.entry_position().ok_or_else(ordering_error)?;
                    if !position.same_capture(&other) {
                        return Err(ordering_error());
                    }
                }
                checked_sort(
                    &mut slots[first..end],
                    |a, b| {
                        inspect(2)?;
                        Ok(a.segment
                            .entry_position()
                            .expect("admitted source ordering position")
                            .partial_cmp(
                                &b.segment
                                    .entry_position()
                                    .expect("admitted source ordering position"),
                            )
                            .expect("same immutable capture establishes comparison"))
                    },
                    || inspect(2),
                )?;
                for pair in slots[first..end].windows(2) {
                    inspect(2)?;
                    if pair[0].segment.entry_position() == pair[1].segment.entry_position() {
                        return Err(ordering_error());
                    }
                }
            }
            first = end;
        }
        Ok((slots, lease))
    }

    pub(crate) fn has_semantic(&self, container: Node) -> bool {
        self.associations.runs().iter().any(|row| {
            row.nodes.is_empty() && self.paragraphs[row.paragraph].container == container
        })
    }

    pub(crate) fn semantic_run(
        &self,
        container: Node,
        segment: &surgeist_text::PreparedSourceSegment,
    ) -> Option<super::AdmittedRun<'input, Node>> {
        self.ordinary_run(container, RunLookup::EmptySegment(segment))
    }
    pub fn frame(&self, node: Node) -> Option<CanonicalFrame<Node, S>> {
        self.frames.iter().find(|frame| frame.node == node).copied()
    }

    pub(crate) fn record_boxes(&mut self, uses: super::boxes::BoxRunUse<Node, S>) {
        if !uses.trials.is_empty() && !self.boxes.contains(&uses) {
            self.boxes.push(uses);
        }
    }

    pub(crate) fn discard_placement(&mut self, node: Node) {
        self.pending.retain(|paragraph| paragraph.container != node);
        self.scope_fragments.retain(|group| group.container != node);
        self.absolute_baselines
            .retain(|(subject, _)| *subject != node);
    }

    pub fn record(&mut self, paragraph: PendingParagraph<Node>) {
        if let Some(old) = self.pending.iter_mut().find(|old| {
            old.binding_index == paragraph.binding_index
                && old.segment == paragraph.segment
                && old.container == paragraph.container
                && old.nodes == paragraph.nodes
                && old.prepared == paragraph.prepared
                && old.inline_boxes == paragraph.inline_boxes
        }) {
            *old = paragraph;
        } else {
            self.pending.push(paragraph);
        }
    }

    pub fn record_scope_fragments(
        &mut self,
        container: Node,
        prepared: surgeist_text::PreparedText,
        nodes: Vec<Node>,
        fragments: Vec<crate::InlineFragmentOutputEntryOf<Node, S>>,
    ) {
        if let Some(old) = self.scope_fragments.iter_mut().find(|group| {
            group.container == container && group.nodes == nodes && group.prepared == prepared
        }) {
            old.fragments = fragments;
        } else {
            self.scope_fragments.push(ScopeFragmentGroup {
                container,
                prepared,
                nodes,
                fragments,
            });
        }
    }

    pub fn record_absolute_baseline(&mut self, node: Node, baseline: f64) {
        if let Some((_, current)) = self
            .absolute_baselines
            .iter_mut()
            .find(|(subject, _)| *subject == node)
        {
            *current = baseline;
        } else {
            self.absolute_baselines.push((node, baseline));
        }
    }

    pub fn basis(
        &self,
        frames: Vec<CanonicalFrame<Node, S>>,
    ) -> surgeist_text::Result<PassBasis<Node, S>> {
        if let Some(limits) = self.scope_resources.key_limits()? {
            for paragraph in &self.pending {
                super::scope_alignment::ScopeAlignmentFacts::visit(
                    &self.scope_resources,
                    limits,
                    1,
                )?;
                super::scope_alignment::ScopeAlignmentFacts::visit(
                    &self.scope_resources,
                    limits,
                    paragraph.entries.len(),
                )?;
            }
        }
        Ok(PassBasis {
            boxes: self.boxes.clone(),
            wrapping: self.wrapping.clone(),
            formatted: self
                .formatted
                .as_ref()
                .map(super::formatted::FormattedState::basis)
                .transpose()?,
            line_boxes: self
                .pending
                .iter()
                .map(|p| (p.container, p.line_box_start, p.line_box_end))
                .collect(),
            tail: self
                .pending
                .iter()
                .filter(|p| p.tail != super::fit::TailContext::Unrestricted)
                .map(|p| (p.container, p.tail))
                .collect(),
            frames,
            selections: self
                .pending
                .iter()
                .map(|p| (p.container, p.wrap_style, p.program.clone()))
                .collect(),
            lines: self
                .pending
                .iter()
                .flat_map(|paragraph| {
                    paragraph
                        .entries
                        .iter()
                        .zip(&paragraph.bands)
                        .zip(&paragraph.flows)
                        .zip(&paragraph.amounts)
                        .zip(&paragraph.encounters)
                        .zip(&paragraph.zones)
                        .zip(&paragraph.phrases)
                        .zip(&paragraph.boxes)
                        .zip(&paragraph.natural_advances)
                        .zip(paragraph.scopes.iter(paragraph.entries.len()))
                })
                .map(
                    |(
                        (
                            (
                                (
                                    (((((placement, band), flow), amounts), encounters), zone),
                                    phrases,
                                ),
                                boxes,
                            ),
                            natural_advance,
                        ),
                        scopes,
                    )| {
                        LineBasis {
                            natural_advance: *natural_advance,
                            boxes: boxes.clone(),
                            zone: zone.clone(),
                            phrases: phrases.clone(),
                            encounters: encounters.clone(),
                            amounts: amounts.clone(),
                            scopes,
                            request: placement.finalized_line().selected_line().request().clone(),
                            disposition: match placement {
                                PlacementEntry::Formatted(_) => {
                                    super::fit::LineDisposition::Formatted
                                }
                                PlacementEntry::NonFormatted(line) => {
                                    super::fit::LineDisposition::NonFormatted(line.line.clone())
                                }
                            },
                            anchor: match placement {
                                PlacementEntry::Formatted(line) => line.anchor,
                                PlacementEntry::NonFormatted(line) => match line.orientation {
                                    surgeist_text::LineOrientation::Horizontal => {
                                        surgeist_text::LineAnchor {
                                            inline_origin: line.origin.x,
                                            baseline: line.origin.y,
                                        }
                                    }
                                    surgeist_text::LineOrientation::SidewaysClockwise
                                    | surgeist_text::LineOrientation::SidewaysCounterclockwise
                                    | surgeist_text::LineOrientation::Vertical => {
                                        surgeist_text::LineAnchor {
                                            inline_origin: line.origin.y,
                                            baseline: line.origin.x,
                                        }
                                    }
                                },
                            },
                            orientation: match placement {
                                PlacementEntry::Formatted(line) => line.orientation,
                                PlacementEntry::NonFormatted(line) => line.orientation,
                            },
                            atomics: match placement {
                                PlacementEntry::Formatted(line) => line.atomics.clone(),
                                PlacementEntry::NonFormatted(line) => line.atomics.clone(),
                            },
                            bands: *band,
                            flow: flow.clone(),
                        }
                    },
                )
                .collect(),
            absolute_baselines: self.absolute_baselines.clone(),
            associations: self.associations.clone(),
            scope_alignments: self.scope_resources.snapshot().clone(),
            hyphenation: self
                .pending
                .iter()
                .map(|paragraph| {
                    (
                        paragraph.container,
                        paragraph.hyphenation_start,
                        paragraph.hyphenation_end,
                    )
                })
                .collect(),
            indentation: self
                .pending
                .iter()
                .map(|paragraph| {
                    (
                        paragraph.container,
                        paragraph.indent,
                        paragraph.indent_start,
                        paragraph.indent_end,
                    )
                })
                .collect(),
        })
    }

    pub fn finish<M>(
        &mut self,
    ) -> crate::LayoutResultOf<Node, Vec<crate::PlacedTextOutput<Node>>, S, M> {
        let frames = &self.frames;
        let baselines = &self.absolute_baselines;
        let mut pending = core::mem::take(&mut self.pending);
        for paragraph in &mut pending {
            for (line_index, occurrence, node) in core::mem::take(&mut paragraph.outflows) {
                let frame = frames
                    .iter()
                    .find(|frame| frame.node == node)
                    .ok_or_else(|| {
                        crate::LayoutErrorOf::new(
                            crate::LayoutErrorSiteOf::Node(node),
                            crate::LayoutOperation::TextPlacement,
                            crate::LayoutErrorKindOf::MissingContext(
                                crate::LayoutMissingContext::TextOutOfFlowGeometry,
                            ),
                        )
                    })?;
                let baseline = baselines
                    .iter()
                    .find(|(subject, _)| *subject == node)
                    .map(|(_, value)| *value)
                    .ok_or_else(|| {
                        crate::LayoutErrorOf::new(
                            crate::LayoutErrorSiteOf::Node(node),
                            crate::LayoutOperation::TextPlacement,
                            crate::LayoutErrorKindOf::MissingContext(
                                crate::LayoutMissingContext::TextOutOfFlowGeometry,
                            ),
                        )
                    })?;
                let content = frame
                    .output
                    .scroll_geometry
                    .map(|geometry| geometry.content_box())
                    .ok_or_else(|| {
                        crate::LayoutErrorOf::new(
                            crate::LayoutErrorSiteOf::Node(node),
                            crate::LayoutOperation::TextPlacement,
                            crate::LayoutErrorKindOf::MissingContext(
                                crate::LayoutMissingContext::TextOutOfFlowGeometry,
                            ),
                        )
                    })?;
                let border = surgeist_text::Rect::new(
                    frame.origin.x,
                    frame.origin.y,
                    frame.output.size.width.to_f64(),
                    frame.output.size.height.to_f64(),
                );
                let content = surgeist_text::Rect::new(
                    placement_add(node, frame.origin.x, content.origin().x.to_f64())?,
                    placement_add(node, frame.origin.y, content.origin().y.to_f64())?,
                    content.size().width.to_f64(),
                    content.size().height.to_f64(),
                );
                // Absolute layout records an already-physical block-axis
                // coordinate within the child; translate without reflection.
                let cross_origin = match paragraph.prepared.orientation() {
                    surgeist_text::LineOrientation::Horizontal => frame.origin.y,
                    surgeist_text::LineOrientation::Vertical
                    | surgeist_text::LineOrientation::SidewaysClockwise
                    | surgeist_text::LineOrientation::SidewaysCounterclockwise => frame.origin.x,
                };
                let baseline = placement_add(node, cross_origin, baseline)?;
                let baseline = (baseline + 0.5).floor();
                let atomics = match &mut paragraph.entries[line_index] {
                    PlacementEntry::Formatted(line) => &mut line.atomics,
                    PlacementEntry::NonFormatted(line) => &mut line.atomics,
                };
                atomics.push(surgeist_text::AtomicPlacement {
                    occurrence,
                    border,
                    content,
                    baseline,
                });
            }
        }
        let mut completed = Vec::new();
        while let Some(first) = pending.first() {
            let index = first.binding_index;
            let binding = &self.paragraphs[index];
            let inline_boxes = self.associations.inline_boxes(index);
            let allocation = || {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(binding.container),
                    crate::LayoutOperation::TextPlacement,
                    crate::LayoutErrorKindOf::Allocation,
                )
            };
            let mut entries = Vec::new();
            // The Text-issued order, not raw member numbers or current packet
            // insertion order, owns whole-output coverage. Move entries;
            // an absent segment remains absent and the Text receiver refuses it.
            for segment in binding.prepared.source_segments() {
                for paragraph in pending.iter_mut().filter(|paragraph| {
                    paragraph.binding_index == index && paragraph.segment == segment
                }) {
                    if paragraph.prepared != binding.prepared
                        || paragraph.inline_boxes != inline_boxes
                    {
                        return Err(crate::LayoutErrorOf::new(
                            crate::LayoutErrorSiteOf::Node(binding.container),
                            crate::LayoutOperation::TextPlacement,
                            crate::LayoutErrorKindOf::Text(super::fragmentation::foreign_family()),
                        ));
                    }
                    for entry in &paragraph.entries {
                        super::fragmentation::InlineBoxFamilies::validate_line(
                            &inline_boxes,
                            entry.finalized_line(),
                        )
                        .map_err(|error| {
                            crate::LayoutErrorOf::new(
                                crate::LayoutErrorSiteOf::Node(binding.container),
                                crate::LayoutOperation::TextPlacement,
                                crate::LayoutErrorKindOf::Text(error),
                            )
                        })?;
                    }
                    entries
                        .try_reserve(paragraph.entries.len())
                        .map_err(|_| allocation())?;
                    entries.append(&mut paragraph.entries);
                }
            }
            pending.retain(|paragraph| paragraph.binding_index != index);
            let layout = surgeist_text::Layout::from_finalized_lines(PlacementRequest {
                prepared: binding.prepared.clone(),
                coverage: PlacementCoverage::WholeParagraph,
                entries,
            })
            .map_err(|error| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(binding.container),
                    crate::LayoutOperation::TextPlacement,
                    crate::LayoutErrorKindOf::Text(error),
                )
            })?;
            let mut nodes = Vec::new();
            nodes
                .try_reserve_exact(binding.nodes.len())
                .map_err(|_| allocation())?;
            nodes.extend_from_slice(&binding.nodes);
            completed.try_reserve(1).map_err(|_| allocation())?;
            completed.push(crate::PlacedTextOutput::new(
                binding.container,
                nodes,
                layout,
                inline_boxes,
            ));
        }
        Ok(completed)
    }
}

fn placement_add<Node, S: LayoutScalar, M>(
    node: Node,
    base: f64,
    delta: f64,
) -> crate::LayoutResultOf<Node, f64, S, M> {
    let value = base + delta;
    let reason = if !value.is_finite() {
        Some(crate::LayoutTextNumericError::NonFinite)
    } else if delta != 0.0 && value == base {
        Some(crate::LayoutTextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::TextPlacement,
            crate::LayoutErrorKindOf::TextNumeric {
                value: if value.is_finite() { delta } else { value },
                reason,
            },
        ));
    }
    Ok(value)
}
