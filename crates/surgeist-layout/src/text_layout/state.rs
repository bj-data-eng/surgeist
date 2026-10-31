//! Off-side Text progress retained by the existing Layout computation session.

use super::TextParagraphBinding;
use crate::{LayoutScalar, NodeOutputOf, Point};
use surgeist_text::{PlacementCoverage, PlacementEntry, PlacementRequest};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalFrame<Node, S: LayoutScalar> {
    pub node: Node,
    pub origin: Point<f64>,
    pub output: NodeOutputOf<S>,
}

#[derive(Clone)]
pub(crate) struct PendingParagraph<Node> {
    pub binding_index: usize,
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
            for (line, other) in self.lines.iter().zip(&other.lines) {
                super::scope_alignment::admit_key_comparison(
                    limits,
                    line.scopes.comparison_rows(),
                    other.scopes.comparison_rows(),
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
    pub frames: std::sync::Arc<[CanonicalFrame<Node, S>]>,
    pub pending: Vec<PendingParagraph<Node>>,
    pub boxes: Vec<super::boxes::BoxRunUse<Node, S>>,
    pub reusable: std::sync::Arc<[super::fit::ReusableLine]>,
    pub atomic_origins: Vec<(Node, Point<f64>)>,
    pub absolute_baselines: Vec<(Node, f64)>,
    pub scope_fragments: Vec<ScopeFragmentGroup<Node, S>>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> TextComputation<'input, Node, S> {
    pub(crate) fn formatted_active(&self) -> bool {
        self.formatted
            .as_ref()
            .is_some_and(|state| !state.demanded.is_empty())
    }

    pub(crate) fn formatted_valid(&self) -> bool {
        self.formatted
            .as_ref()
            .is_none_or(super::formatted::FormattedState::valid)
    }

    pub(crate) fn formatted_snapshot(
        &self,
        container: Node,
        nodes: &[Node],
        segment: &surgeist_text::PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: super::fit::Available,
    ) -> super::formatted::FormattedTailSnapshot {
        if let Some(state) = &self.formatted {
            state.snapshot(&state.binding(container, nodes, segment, input, mode))
        } else {
            // A checked Layout run has actual complete tree authority. No tail
            // collection or sidecar lookup is performed until a reached demand.
            super::formatted::FormattedTailSnapshot::known(segment.clone(), None)
        }
    }

    pub(crate) fn formatted_merge_run(
        &mut self,
        container: Node,
        nodes: &[Node],
        segment: &surgeist_text::PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: super::fit::Available,
        uses: super::formatted::FormattedTailUses,
    ) -> Option<super::formatted::FormattedFlowItem<Node, S>> {
        if self.formatted.is_none() && uses.demands.is_empty() && uses.roles.is_empty() {
            return None;
        }
        let state = self
            .formatted
            .get_or_insert_with(super::formatted::FormattedState::new);
        state.frames = self.frames.clone();
        state.wrapping = self.wrapping.clone();
        let binding = state.binding(container, nodes, segment, input, mode);
        let item =
            super::formatted::FormattedFlowItem::Text(binding.clone(), uses.accepted.clone());
        state.merge(binding, uses);
        self.formatted_active().then_some(item)
    }

    pub(crate) fn formatted_begin_block(
        &mut self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
    ) -> bool {
        if !self.formatted_active() {
            return false;
        }
        self.formatted
            .as_mut()
            .unwrap()
            .stack
            .push(super::formatted::ComputeScope {
                node,
                input,
                anonymous,
                frame: None,
            });
        true
    }

    pub(crate) fn formatted_end_block(
        &mut self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
        success: bool,
    ) {
        if let Some(state) = &mut self.formatted {
            state.finish_block(
                super::formatted::ComputeScope {
                    node,
                    input,
                    anonymous,
                    frame: None,
                },
                success,
            );
        }
    }

    pub(crate) fn formatted_new_flow(&self) -> Option<super::formatted::FormattedFlow<Node, S>> {
        self.formatted_active()
            .then(|| super::formatted::FormattedFlow {
                frame: self
                    .formatted
                    .as_ref()
                    .and_then(|state| state.stack.last())
                    .and_then(|scope| scope.frame),
                items: Vec::new(),
            })
    }

    pub(crate) fn formatted_begin_flow(
        &mut self,
        input: crate::ComputeInputOf<S>,
        available: crate::AvailableOf<S>,
        publication: bool,
    ) {
        if self.formatted_active()
            && let Some(scope) = self
                .formatted
                .as_mut()
                .and_then(|state| state.stack.last_mut())
        {
            scope.frame = Some(super::formatted::FlowFrame {
                input,
                available,
                publication,
            });
        }
    }

    pub(crate) fn formatted_record_traversal(
        &mut self,
        flow: &Option<super::formatted::FormattedFlow<Node, S>>,
    ) {
        if let (Some(state), Some(flow)) = (&mut self.formatted, flow) {
            state.record(flow.clone());
        }
    }

    pub(crate) fn formatted_record_flow(
        &mut self,
        flow: Option<super::formatted::FormattedFlow<Node, S>>,
    ) {
        if let (Some(state), Some(flow)) = (&mut self.formatted, flow) {
            state.select(flow);
        }
    }

    pub(crate) fn formatted_child(
        &self,
        node: Node,
        input: crate::ComputeInputOf<S>,
        anonymous: bool,
    ) -> Option<super::formatted::FormattedFlowItem<Node, S>> {
        if !self.formatted_active() {
            return None;
        }
        let mut scope = self.formatted.as_ref()?.stack.clone();
        scope.push(super::formatted::ComputeScope {
            node,
            input,
            anonymous,
            frame: None,
        });
        Some(super::formatted::FormattedFlowItem::Child(scope))
    }

    /// Required ancestors must execute to reach the demanded owner. Descendant
    /// summaries are needed only inside that owner; unrelated sibling caches
    /// retain the ordinary geometry-only path.
    pub(crate) fn formatted_cache_required(&self, node: Node) -> bool {
        if !self.formatted_active() {
            return false;
        }
        let Some(graph) = self.graph else {
            return true;
        };
        let Some(subject) = graph.index(node) else {
            return true;
        };
        self.formatted
            .as_ref()
            .unwrap()
            .demanded
            .iter()
            .any(|owner| {
                let Some(owner) = graph.index(*owner) else {
                    return true;
                };
                let reaches = |start: usize, end: usize| {
                    let mut pending = vec![start];
                    let mut seen = Vec::new();
                    while let Some(at) = pending.pop() {
                        if at == end {
                            return true;
                        }
                        if !seen.contains(&at) {
                            seen.push(at);
                            pending.extend(&graph.children[at]);
                        }
                    }
                    false
                };
                reaches(owner, subject) || reaches(subject, owner)
            })
    }

    pub(crate) fn formatted_cache_available(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> bool {
        !self.formatted_cache_required(node)
            || self
                .formatted
                .as_ref()
                .and_then(|state| state.receipt(node, input))
                .is_some_and(|receipt| receipt.output.is_some())
    }

    pub(crate) fn formatted_cache_matches(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) -> bool {
        !self.formatted_cache_required(node)
            || self
                .formatted
                .as_ref()
                .is_some_and(|state| state.cache_matches(node, input, context, output))
    }

    pub(crate) fn formatted_restore_cache(
        &mut self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> bool {
        if !self.formatted_cache_required(node) {
            return true;
        }
        self.formatted
            .as_mut()
            .is_some_and(|state| state.restore(node, input))
    }

    pub(crate) fn formatted_next_pass(&mut self) {
        if let Some(state) = &mut self.formatted {
            state.next_pass();
        }
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
            frames: std::sync::Arc::from([]),
            pending: Vec::new(),
            boxes: Vec::new(),
            reusable: std::sync::Arc::from([]),
            atomic_origins: Vec::new(),
            absolute_baselines: Vec::new(),
            scope_fragments: Vec::new(),
        }
    }

    pub(crate) fn run(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<super::AdmittedRun<'input, Node>> {
        self.associations.runs().iter().find_map(|row| {
            let binding = &self.paragraphs[row.paragraph];
            (binding.container == container
                && !row.nodes.is_empty()
                && binding.nodes[row.nodes.clone()] == *nodes)
                .then(|| self.associations.admitted_run(self.paragraphs, row))
        })
    }

    pub(crate) fn run_starting(
        &self,
        container: Node,
        nodes: &[Node],
    ) -> Option<super::AdmittedRun<'input, Node>> {
        self.associations.runs().iter().find_map(|row| {
            let binding = &self.paragraphs[row.paragraph];
            (binding.container == container
                && !row.nodes.is_empty()
                && nodes.starts_with(&binding.nodes[row.nodes.clone()]))
            .then(|| self.associations.admitted_run(self.paragraphs, row))
        })
    }

    pub(crate) fn semantic_slots<M>(
        &self,
        container: Node,
    ) -> crate::LayoutResultOf<Node, Vec<super::capture::NativeRunSlot>, S, M> {
        let count = self
            .associations
            .runs()
            .iter()
            .filter(|row| {
                row.nodes.is_empty() && self.paragraphs[row.paragraph].container == container
            })
            .count();
        let mut slots = Vec::new();
        slots.try_reserve_exact(count).map_err(|_| {
            crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(container),
                crate::LayoutOperation::TextCapture,
                crate::LayoutErrorKindOf::Allocation,
            )
        })?;
        slots.extend(
            self.associations
                .runs()
                .iter()
                .filter(|row| {
                    row.nodes.is_empty() && self.paragraphs[row.paragraph].container == container
                })
                .map(|row| super::capture::NativeRunSlot {
                    gap: row.children.start,
                    segment: row.segment.clone(),
                }),
        );
        slots.sort_unstable_by_key(|slot| slot.gap);
        let ordering_error =
            || super::binding_error(container, super::TextBindingError::ScopeAssociation);
        let mut first = 0;
        while first < slots.len() {
            let gap = slots[first].gap;
            let end = first + slots[first..].partition_point(|slot| slot.gap == gap);
            if end - first > 1 {
                // A native gap alone cannot order distinct source operations.
                // Admit the actual issued positions before the pure comparison;
                // a sole scheduled slot has no such ordering dependency.
                let position = slots[first]
                    .segment
                    .entry_position()
                    .ok_or_else(ordering_error)?;
                for slot in &slots[first + 1..end] {
                    let other = slot.segment.entry_position().ok_or_else(ordering_error)?;
                    if !position.same_capture(&other) {
                        return Err(ordering_error());
                    }
                }
                slots[first..end].sort_unstable_by(|a, b| {
                    a.segment
                        .entry_position()
                        .expect("admitted source ordering position")
                        .partial_cmp(
                            &b.segment
                                .entry_position()
                                .expect("admitted source ordering position"),
                        )
                        .expect("same immutable capture establishes comparison")
                });
                if slots[first..end].windows(2).any(|pair| {
                    pair[0].segment.entry_position() == pair[1].segment.entry_position()
                }) {
                    return Err(ordering_error());
                }
            }
            first = end;
        }
        Ok(slots)
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
        self.associations
            .runs()
            .iter()
            .find(|row| {
                self.paragraphs[row.paragraph].container == container
                    && row.nodes.is_empty()
                    && row.segment == *segment
            })
            .map(|row| self.associations.admitted_run(self.paragraphs, row))
    }
    pub fn frame(&self, node: Node) -> Option<CanonicalFrame<Node, S>> {
        self.frames.iter().find(|frame| frame.node == node).copied()
    }

    pub(crate) fn record_boxes(&mut self, uses: super::boxes::BoxRunUse<Node, S>) {
        if !uses.trials.is_empty() && !self.boxes.contains(&uses) {
            self.boxes.push(uses);
        }
    }

    pub fn record(&mut self, paragraph: PendingParagraph<Node>) {
        if let Some(old) = self.pending.iter_mut().find(|old| {
            old.binding_index == paragraph.binding_index
                && old.segment == paragraph.segment
                && old.container == paragraph.container
                && old.nodes == paragraph.nodes
                && old.prepared == paragraph.prepared
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

    pub fn record_atomic_origins(&mut self, origins: Vec<(Node, Point<f64>)>) {
        for (node, origin) in origins {
            if let Some((_, current)) = self
                .atomic_origins
                .iter_mut()
                .find(|(subject, _)| *subject == node)
            {
                *current = origin;
            } else {
                self.atomic_origins.push((node, origin));
            }
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
                .map(super::formatted::FormattedState::basis),
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
                            flow: *flow,
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
            let allocation = || {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(binding.container),
                    crate::LayoutOperation::TextPlacement,
                    crate::LayoutErrorKindOf::Allocation,
                )
            };
            let mut entries = Vec::new();
            // The Text-issued order, not raw member numbers or current packet
            // insertion order, owns whole-output coverage. Move genuine entries;
            // an absent segment remains absent and the Text receiver refuses it.
            for segment in binding.prepared.source_segments() {
                for paragraph in pending.iter_mut().filter(|paragraph| {
                    paragraph.binding_index == index && paragraph.segment == segment
                }) {
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
