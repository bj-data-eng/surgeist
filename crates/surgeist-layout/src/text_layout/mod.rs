//! The concrete immutable Text capture and its Layout-owned consuming adapter.

pub(crate) mod boxes;
pub(crate) mod capture;
pub(crate) mod edge_components;
pub(crate) mod fit;
pub(crate) mod formatted;
mod fragmentation;
mod roles;
pub(crate) mod run;
pub(crate) mod scope_alignment;
pub(crate) mod state;
#[cfg(test)]
mod tests;

pub use scope_alignment::{TextScopeAlignmentInput, TextScopeAlignmentInputOf};
use surgeist_text::{PreparedAtomic, PreparedControl, PreparedOrigin, PreparedScope, PreparedText};

/// One source occurrence participating through its actual Text preparation.
///
/// The issued origin retains its immutable preparation. This value carries no
/// caller-authored width, glyph array, break permission or replacement extent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextParticipation {
    origin: PreparedOrigin,
}

impl TextParticipation {
    #[must_use]
    pub const fn new(origin: PreparedOrigin) -> Self {
        Self { origin }
    }

    #[must_use]
    pub const fn origin(&self) -> &PreparedOrigin {
        &self.origin
    }
}

/// An actual Text-issued semantic control with no Layout strut allocation.
///
/// Its kind, wrapping rule, anchor, and affinity remain Text-owned observations.
/// Structural line breaks and inline boundaries with real Layout metrics use
/// their existing normalized input roles instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextControlParticipation {
    occurrence: PreparedControl,
}

impl TextControlParticipation {
    #[must_use]
    pub const fn new(occurrence: PreparedControl) -> Self {
        Self { occurrence }
    }
    #[must_use]
    pub const fn occurrence(&self) -> &PreparedControl {
        &self.occurrence
    }
}

/// The actual immutable tree node associated with an issued source occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct TextOriginBinding<Node> {
    pub origin: PreparedOrigin,
    pub node: Node,
}

/// The actual atomic box node associated with an issued Text occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct TextAtomicBinding<Node> {
    pub occurrence: PreparedAtomic,
    pub node: Node,
}

/// A control occurrence and the actual tree node supplying its participation.
#[derive(Clone, Debug, PartialEq)]
pub struct TextControlBinding<Node> {
    pub occurrence: PreparedControl,
    pub node: Node,
}

/// The normalized Layout association for one Text-issued contributing scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextScopeAssociation<Node> {
    ParagraphContainer(Node),
    Inline { start: Node, end: Node },
}

/// One exact issued scope and its actual normalized container or boundary pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextScopeBinding<Node> {
    pub scope: PreparedScope,
    pub association: TextScopeAssociation<Node>,
}

/// Whether this actual PreparedText binding requests complete-body inline-box
/// fragment identity. Numeric flow and unrelated bindings make no such claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoxFragmentationDemand {
    NotRequested,
    CompleteBodies,
}

/// The retained checked proof, distinct from the caller's requested capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdmittedInlineBoxes {
    NotRequested,
    CompleteBodies(surgeist_text::PreparedInlineBoxFamily),
}
impl AdmittedInlineBoxes {
    pub(crate) fn demand(&self) -> InlineBoxFragmentationDemand {
        match self {
            Self::NotRequested => InlineBoxFragmentationDemand::NotRequested,
            Self::CompleteBodies(_) => InlineBoxFragmentationDemand::CompleteBodies,
        }
    }
    pub(crate) fn family(&self) -> Option<&surgeist_text::PreparedInlineBoxFamily> {
        match self {
            Self::NotRequested => None,
            Self::CompleteBodies(family) => Some(family),
        }
    }
}

/// Supplied association facts, checked together against the immutable tree by
/// `LayoutFlowInput::try_new` before any layout computation can consume them.
#[derive(Clone, Debug, PartialEq)]
pub struct TextParagraphBinding<Node> {
    pub prepared: PreparedText,
    pub inline_box_fragments: InlineBoxFragmentationDemand,
    pub container: Node,
    /// Complete native participating-node inventory in actual container order.
    /// A structural child block may separate its admitted native run windows;
    /// this does not split the one analyzed output or its preparation.
    pub nodes: Vec<Node>,
    pub origins: Vec<TextOriginBinding<Node>>,
    pub atomics: Vec<TextAtomicBinding<Node>>,
    pub controls: Vec<TextControlBinding<Node>>,
    /// Genuine normalized line allocation, when the caller supplies one.
    /// Absence never guesses an empty/default font or a zero-size line.
    pub line_strut: Option<surgeist_text::LineStrut>,
}

/// One admitted native use of the same output-level binding and preparation.
/// The segment is issued by Text; the native slice is derived from the checked
/// immutable tree. This view declares no source or paragraph authority.
#[derive(Clone)]
pub(crate) struct AdmittedRun<'input, Node> {
    pub(crate) binding_index: usize,
    pub(crate) binding: &'input TextParagraphBinding<Node>,
    pub(crate) inventory: std::ops::Range<usize>,
    pub(crate) native: std::ops::Range<usize>,
    pub(crate) segment: surgeist_text::PreparedSourceSegment,
    pub(crate) inline_boxes: AdmittedInlineBoxes,
}

impl<Node> AdmittedRun<'_, Node> {
    pub(crate) fn nodes(&self) -> &[Node] {
        &self.binding.nodes[self.inventory.clone()]
    }
}

/// A rejected association between immutable Layout and Text inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TextBindingError {
    EmptyRun,
    RunMembership,
    ParticipantOrder,
    DuplicateRunNode,
    OverlappingParagraphs,
    OriginCoverage,
    AtomicCoverage,
    ControlCoverage,
    ScopeCoverage,
    ScopeAssociation,
    ScopeBoundaryCapture,
    ScopeBoundaryOrder,
    ScopeOccurrenceMembership,
    ScopeTreeAmbiguity,
    ParticipantRole,
    MissingParagraph,
    RootMismatch,
    TabReferenceDirection,
    TerminalBlockEdge,
    ComputedBoundaryAssociation,
    InlineBoxFragmentsRequireFlow,
}

/// Complete, checked inline flow over one borrowed tree.
///
/// The tree borrow prevents mutable replacement of its normalized inputs while
/// the input is being consumed. Text paragraphs retain their exact preparation.
/// Wrapping scopes describe actual normalized inline boxes; numeric-only flow
/// needs no Text preparation.
pub struct LayoutFlowInput<'tree, Tree: crate::LayoutTree> {
    pub(crate) tree: &'tree Tree,
    pub(crate) root: Tree::Node,
    pub(crate) paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
    pub(crate) invalidated: Vec<Tree::Node>,
    pub(crate) graph: crate::engine::validation::ValidationGraph<Tree::Node>,
    pub(crate) associations: std::sync::Arc<capture::ScopeAssociations<Tree::Node>>,
    pub(crate) wrapping: std::sync::Arc<crate::inline_wrapping::WrappingContext<Tree::Node>>,
    pub(crate) scope_alignments:
        std::sync::Arc<scope_alignment::ScopeAlignmentSnapshot<Tree::Scalar>>,
}

impl<'tree, Tree: crate::LayoutTree> LayoutFlowInput<'tree, Tree> {
    /// Validates paragraph participation and declared wrapping scopes at this root.
    ///
    /// This proves association to supplied immutable normalized inputs. It does
    /// not authenticate live DOM or computed-style facts absent from the tree.
    /// Empty paragraphs admit box/control-only flow. Empty scopes retain the
    /// ordinary automatic wrapping policy.
    pub fn try_new(
        tree: &'tree Tree,
        root: Tree::Node,
        paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
        wrapping_input: crate::WrappingInput<Tree::Node>,
        scope_alignments: Vec<TextScopeAlignmentInputOf<Tree::Scalar>>,
    ) -> crate::LayoutResultOf<Tree::Node, Self, Tree::Scalar, Tree::MeasureError> {
        Self::validate_run_nodes(&paragraphs)?;
        Self::try_new_inner(
            tree,
            root,
            paragraphs,
            wrapping_input,
            scope_alignments,
            None,
        )
    }

    /// Revalidates an early source receipt against this current host projection,
    /// then performs the ordinary immutable Text and native association checks.
    pub fn try_new_with_edges(
        tree: &'tree Tree,
        root: Tree::Node,
        paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
        wrapping_input: crate::WrappingInput<Tree::Node>,
        scope_alignments: Vec<TextScopeAlignmentInputOf<Tree::Scalar>>,
        receipt: &crate::ResolvedWrapEdges<Tree::Node>,
    ) -> crate::LayoutResultOf<Tree::Node, Self, Tree::Scalar, Tree::MeasureError>
    where
        Tree: crate::WrapEdgeSource,
    {
        Self::validate_run_nodes(&paragraphs)?;
        let graph = receipt.validate_final(tree, root, &paragraphs, &wrapping_input)?;
        Self::try_new_inner(
            tree,
            root,
            paragraphs,
            wrapping_input,
            scope_alignments,
            Some(graph),
        )
    }

    /// Node occurrence uniqueness is an input-only guard. It precedes every
    /// capture receipt, native tree observation and run-window projection.
    fn validate_run_nodes(
        paragraphs: &[TextParagraphBinding<Tree::Node>],
    ) -> crate::LayoutResultOf<Tree::Node, (), Tree::Scalar, Tree::MeasureError> {
        for paragraph in paragraphs {
            for (index, node) in paragraph.nodes.iter().enumerate() {
                if paragraph.nodes[..index].contains(node) {
                    return Err(binding_error(
                        paragraph.container,
                        TextBindingError::DuplicateRunNode,
                    ));
                }
            }
        }
        Ok(())
    }

    fn try_new_inner(
        tree: &'tree Tree,
        root: Tree::Node,
        paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
        wrapping_input: crate::WrappingInput<Tree::Node>,
        scope_alignments: Vec<TextScopeAlignmentInputOf<Tree::Scalar>>,
        source_graph: Option<crate::engine::validation::ValidationGraph<Tree::Node>>,
    ) -> crate::LayoutResultOf<Tree::Node, Self, Tree::Scalar, Tree::MeasureError> {
        let mut influencing = Vec::new();
        let mut inline_boxes_requested = false;
        let mut occupied = Vec::new();
        for paragraph in &paragraphs {
            let error = |reason| binding_error(paragraph.container, reason);
            let class = capture::classify(paragraph).map_err(error)?;
            inline_boxes_requested |=
                paragraph.inline_box_fragments == InlineBoxFragmentationDemand::CompleteBodies;
            if class == capture::ParagraphClass::AssociationOnly {
                if paragraph.inline_box_fragments == InlineBoxFragmentationDemand::CompleteBodies {
                    return Err(error(TextBindingError::InlineBoxFragmentsRequireFlow));
                }
                if tree.layout_input(paragraph.container).as_box().is_none() {
                    return Err(error(TextBindingError::ScopeAssociation));
                }
                influencing.push(paragraph.container);
                continue;
            }
            if paragraph.prepared.preparation_options().tabs()
                == surgeist_text::TabPreparation::CssStops
                && let Some(reference) = paragraph.prepared.tab_reference()
            {
                let actual =
                    tree.layout_input(paragraph.container)
                        .as_box()
                        .map(|input| match input.direction {
                            crate::Direction::Ltr => {
                                surgeist_text::ParagraphBaseDirection::LeftToRight
                            }
                            crate::Direction::Rtl => {
                                surgeist_text::ParagraphBaseDirection::RightToLeft
                            }
                        });
                if actual != Some(reference.direction()) {
                    return Err(error(TextBindingError::TabReferenceDirection));
                }
            }
            if class == capture::ParagraphClass::SemanticFlow {
                if tree.layout_input(paragraph.container).as_box().is_none() {
                    return Err(error(TextBindingError::ScopeAssociation));
                }
                influencing.push(paragraph.container);
                continue;
            }
            if paragraph.nodes.is_empty() {
                return Err(error(TextBindingError::EmptyRun));
            }
            let children: Vec<_> = tree.children(paragraph.container).collect();
            let ordinary = crate::inline::run_ranges(&children, |node| tree.layout_input(node));
            let transparent_float = |node| {
                tree.layout_input(node).as_box().is_some_and(|input| {
                    !input.float.is_none()
                        && input.position != crate::Position::Absolute
                        && input.display != crate::Display::None
                })
            };
            let structural_block = |node| {
                tree.layout_input(node).as_box().is_some_and(|input| {
                    input.display != crate::Display::None
                        && input.position != crate::Position::Absolute
                        && input.float.is_none()
                        && !input.display.is_inline_level()
                })
            };
            let complete_window = |start: usize, end: usize| {
                ordinary
                    .iter()
                    .any(|range| range.start == start && range.end == end)
                    || children[start..end].iter().all(|&node| {
                        crate::inline::continues_run(tree.layout_input(node))
                            || transparent_float(node)
                    }) && (start == 0
                        || !crate::inline::continues_run(tree.layout_input(children[start - 1])))
                        && (end == children.len()
                            || !crate::inline::continues_run(tree.layout_input(children[end])))
            };
            let first = children
                .iter()
                .position(|node| paragraph.nodes.first() == Some(node))
                .ok_or_else(|| error(TextBindingError::RunMembership))?;
            let mut start = first;
            let mut previous = first;
            for node in paragraph.nodes.iter().skip(1) {
                let position = children
                    .iter()
                    .position(|child| child == node)
                    .ok_or_else(|| error(TextBindingError::RunMembership))?;
                if position <= previous {
                    return Err(error(TextBindingError::RunMembership));
                }
                if position != previous + 1 {
                    if !complete_window(start, previous + 1)
                        || !children[previous + 1..position]
                            .iter()
                            .all(|&node| structural_block(node))
                    {
                        return Err(error(TextBindingError::RunMembership));
                    }
                    start = position;
                }
                previous = position;
            }
            if !complete_window(start, previous + 1) {
                return Err(error(TextBindingError::RunMembership));
            }
            for node in paragraph.nodes.iter().copied() {
                if occupied.contains(&node) {
                    return Err(error(TextBindingError::OverlappingParagraphs));
                }
                occupied.push(node);
            }
            influencing.push(paragraph.container);
            influencing.extend(paragraph.origins.iter().map(|binding| binding.node));
            influencing.extend(paragraph.controls.iter().map(|binding| binding.node));

            let origins: Vec<_> = paragraph
                .prepared
                .origin_views()
                .map(|view| view.handle())
                .collect();
            if origins.len() != paragraph.origins.len()
                || origins.iter().any(|origin| {
                    paragraph
                        .origins
                        .iter()
                        .filter(|binding| &binding.origin == origin)
                        .count()
                        != 1
                })
            {
                return Err(error(TextBindingError::OriginCoverage));
            }
            for binding in &paragraph.origins {
                if !paragraph.nodes.contains(&binding.node) {
                    return Err(error(TextBindingError::RunMembership));
                }
                // The normalized occurrence and paragraph inventory must retain
                // the same issued handle, not merely equal source characters.
                match tree.layout_input(binding.node) {
                    crate::LayoutInputViewOf::InlineText(input)
                        if input.origin() == &binding.origin => {}
                    _ => return Err(error(TextBindingError::ParticipantRole)),
                }
            }
            let atomics: Vec<_> = paragraph
                .prepared
                .atomic_views()
                .map(|view| view.handle())
                .collect();
            if atomics.len() != paragraph.atomics.len()
                || atomics.iter().any(|atomic| {
                    paragraph
                        .atomics
                        .iter()
                        .filter(|binding| &binding.occurrence == atomic)
                        .count()
                        != 1
                })
            {
                return Err(error(TextBindingError::AtomicCoverage));
            }
            for binding in &paragraph.atomics {
                if paragraph
                    .atomics
                    .iter()
                    .filter(|row| row.node == binding.node)
                    .count()
                    != 1
                {
                    return Err(error(TextBindingError::AtomicCoverage));
                }
                let Some(input) = tree.layout_input(binding.node).as_box() else {
                    return Err(error(TextBindingError::ParticipantRole));
                };
                let applicable = match binding.occurrence.view().original_box().kind() {
                    surgeist_text::InlineBoxKind::InFlow => {
                        input.display.is_inline_level()
                            && input.position != crate::Position::Absolute
                            && input.float.is_none()
                    }
                    surgeist_text::InlineBoxKind::OutOfFlow => {
                        input.position == crate::Position::Absolute
                            && input.display != crate::Display::None
                    }
                };
                if !paragraph.nodes.contains(&binding.node) || !applicable {
                    return Err(error(TextBindingError::ParticipantRole));
                }
            }
            let controls: Vec<_> = paragraph
                .prepared
                .control_views()
                .map(|view| view.handle())
                .collect();
            if controls.len() != paragraph.controls.len()
                || controls.iter().any(|control| {
                    paragraph
                        .controls
                        .iter()
                        .filter(|binding| &binding.occurrence == control)
                        .count()
                        != 1
                })
            {
                return Err(error(TextBindingError::ControlCoverage));
            }
            for binding in &paragraph.controls {
                let occurrence = binding.occurrence.view();
                let kind = occurrence.kind();
                let applicable = match tree.layout_input(binding.node) {
                    crate::LayoutInputViewOf::TextControl(input) => {
                        input.occurrence() == &binding.occurrence
                    }
                    crate::LayoutInputViewOf::LineBreak(input) => {
                        !input.display().is_none()
                            && kind == &surgeist_text::ParagraphControlKind::ForcedBreak
                    }
                    crate::LayoutInputViewOf::InlineBoundary(input) => match input.kind() {
                        crate::InlineBoundaryKind::Start => {
                            kind == &surgeist_text::ParagraphControlKind::InlineBoundaryStart
                        }
                        crate::InlineBoundaryKind::End => {
                            kind == &surgeist_text::ParagraphControlKind::InlineBoundaryEnd
                        }
                    },
                    _ => false,
                };
                if !paragraph.nodes.contains(&binding.node) || !applicable {
                    return Err(error(TextBindingError::ParticipantRole));
                }
            }
            let position = |node| {
                paragraph
                    .nodes
                    .iter()
                    .position(|subject| *subject == node)
                    .expect("inventory role checks established actual run membership")
            };
            let mut last_origin = None;
            for origin in &origins {
                let row = paragraph
                    .origins
                    .iter()
                    .find(|row| &row.origin == origin)
                    .expect("complete origin inventory");
                let current = position(row.node);
                if last_origin.is_some_and(|previous| previous >= current) {
                    return Err(error(TextBindingError::ParticipantOrder));
                }
                last_origin = Some(current);
            }
            let control_position = |control: &PreparedControl| {
                position(
                    paragraph
                        .controls
                        .iter()
                        .find(|row| &row.occurrence == control)
                        .expect("complete actual control inventory")
                        .node,
                )
            };
            let mut lower = 0;
            for participant in paragraph.prepared.participants() {
                let positions = match participant {
                    surgeist_text::PreparedParticipant::Text(unit) => unit
                        .view()
                        .origins()
                        .map(|view| {
                            let origin = view.origin();
                            position(
                                paragraph
                                    .origins
                                    .iter()
                                    .find(|row| row.origin == origin)
                                    .expect("complete unit origin inventory")
                                    .node,
                            )
                        })
                        .chain(
                            unit.view()
                                .controls()
                                .map(|control| control_position(&control)),
                        )
                        .collect::<Vec<_>>(),
                    surgeist_text::PreparedParticipant::Cell(cell) => cell
                        .view()
                        .source_cell()
                        .source_pieces()
                        .map(|piece| {
                            let captured = piece.origin_position();
                            position(
                                paragraph
                                    .origins
                                    .iter()
                                    .find(|row| {
                                        row.origin.view().capture_position().as_ref()
                                            == Some(&captured)
                                    })
                                    .expect("complete cell original occurrence inventory")
                                    .node,
                            )
                        })
                        .collect::<Vec<_>>(),
                    surgeist_text::PreparedParticipant::Atomic(atomic) => vec![position(
                        paragraph
                            .atomics
                            .iter()
                            .find(|row| row.occurrence == atomic)
                            .expect("complete atomic inventory")
                            .node,
                    )],
                    surgeist_text::PreparedParticipant::Control(control) => vec![position(
                        paragraph
                            .controls
                            .iter()
                            .find(|row| row.occurrence == control)
                            .expect("complete control inventory")
                            .node,
                    )],
                    surgeist_text::PreparedParticipant::ComputedLineBoundary(_) => {
                        // This semantic incidence has no host node. The source
                        // registry authenticates its actual group/targets/cut
                        // after graph validation, before this input is issued.
                        Vec::new()
                    }
                };
                if let (Some(minimum), Some(maximum)) =
                    (positions.iter().min(), positions.iter().max())
                {
                    if *maximum < lower {
                        return Err(error(TextBindingError::ParticipantOrder));
                    }
                    lower = lower.max(*minimum);
                }
            }
            for node in &paragraph.nodes {
                match tree.layout_input(*node) {
                    crate::LayoutInputViewOf::InlineText(input) => {
                        if paragraph
                            .origins
                            .iter()
                            .filter(|binding| {
                                binding.node == *node && &binding.origin == input.origin()
                            })
                            .count()
                            != 1
                        {
                            return Err(error(TextBindingError::OriginCoverage));
                        }
                    }
                    crate::LayoutInputViewOf::TextControl(input) => {
                        if paragraph
                            .controls
                            .iter()
                            .filter(|binding| {
                                binding.node == *node && &binding.occurrence == input.occurrence()
                            })
                            .count()
                            != 1
                        {
                            return Err(error(TextBindingError::ControlCoverage));
                        }
                    }
                    crate::LayoutInputViewOf::LineBreak(input) if !input.display().is_none() => {
                        if paragraph
                            .controls
                            .iter()
                            .filter(|binding| binding.node == *node)
                            .count()
                            != 1
                        {
                            return Err(error(TextBindingError::ControlCoverage));
                        }
                    }
                    crate::LayoutInputViewOf::InlineBoundary(_) => {
                        if paragraph
                            .controls
                            .iter()
                            .filter(|binding| binding.node == *node)
                            .count()
                            != 1
                        {
                            return Err(error(TextBindingError::ControlCoverage));
                        }
                    }
                    crate::LayoutInputViewOf::Box(input)
                        if input.display.is_inline_level()
                            && input.float.is_none()
                            && input.position != crate::Position::Absolute
                            && paragraph
                                .atomics
                                .iter()
                                .filter(|binding| binding.node == *node)
                                .count()
                                != 1 =>
                    {
                        return Err(error(TextBindingError::AtomicCoverage));
                    }
                    _ => {}
                }
            }
        }
        let source_validated = source_graph.is_some();
        let (graph, _) = if let Some(graph) = source_graph {
            crate::engine::validation::validate_layout_request_with_graph(
                tree,
                root,
                &influencing,
                &paragraphs,
                graph,
            )?
        } else {
            crate::engine::validation::validate_layout_request_with_text_graph(
                tree,
                root,
                &influencing,
                &paragraphs,
            )?
        };
        // Every actual prepared-text node under this root must be covered. A
        // partial inventory cannot silently use the box-only compute entry.
        for &node in &graph.nodes {
            if (tree.layout_input(node).as_inline_text().is_some()
                || tree.layout_input(node).as_text_control().is_some())
                && !occupied.contains(&node)
            {
                return Err(binding_error(node, TextBindingError::MissingParagraph));
            }
        }
        #[expect(
            clippy::mutable_key_type,
            reason = "Text handle hashing uses immutable capture identity and occurrence index; lazy cache contents never affect the key"
        )]
        let mut seen_scopes = std::collections::HashSet::new();
        let inventory_scopes: Vec<_> = wrapping_input
            .child_order
            .iter()
            .flat_map(|order| {
                std::iter::once(&order.parent).chain(order.children.iter().map(|child| &child.box_))
            })
            .filter_map(|box_| {
                let crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope)) = box_ else {
                    return None;
                };
                seen_scopes.insert(scope.clone()).then(|| scope.clone())
            })
            .collect();
        let mut scope_alignments = scope_alignment::ScopeAlignmentSnapshot::admit(
            scope_alignments,
            &paragraphs,
            inline_boxes_requested,
        )
        .map_err(|error| {
            crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(root),
                crate::LayoutOperation::TextCapture,
                crate::LayoutErrorKindOf::Text(error),
            )
        })?;
        let mut resolved = capture::resolve(
            tree,
            &graph,
            &paragraphs,
            &wrapping_input,
            &inventory_scopes,
            inline_boxes_requested,
            &mut scope_alignments,
        )?;
        let wrapping = crate::inline_wrapping::WrappingContext::new(
            tree,
            &graph,
            &wrapping_input.scopes,
            &resolved.wrapping,
            &wrapping_input.child_order,
            &wrapping_input.gaps,
        )
        .map_err(|reason| crate::inline_wrapping::wrapping_error(root, reason))?;
        if !source_validated
            && paragraphs
                .iter()
                .any(|p| wrapping.requires_source(p.container))
        {
            return Err(crate::inline_wrapping::wrapping_error(
                root,
                crate::WrappingScopeError::SourceProcessingRequired,
            ));
        }
        influencing.extend(wrapping.dependencies());
        let mut influence_census = scope_alignment::ScopeCensus::default();
        {
            let (rows, mut admission) = scope_alignments.capture_parts();
            for row in rows {
                if let Some(work) = admission.as_deref_mut() {
                    work.visits(resolved.associations.captured_binding_work(&row.scope))
                        .map_err(|error| capture::activation_error::<Tree>(root, error))?;
                }
                if let Some(TextScopeAssociation::Inline { start, end }) =
                    resolved.associations.captured_binding(&row.scope)
                {
                    if let Some(work) = admission.as_deref_mut() {
                        for node in [start, end] {
                            work.push(&mut influencing, node, &mut influence_census, false)
                                .map_err(|error| capture::activation_error::<Tree>(root, error))?;
                        }
                    } else {
                        influencing.extend([start, end]);
                    }
                }
            }
        }
        let invalidated = graph.invalidation(&influencing)?;
        drop(influencing);
        scope_alignments.work().release(influence_census);
        let family_slots = if inline_boxes_requested {
            Some(fragmentation::reserve_slots::<Tree>(
                root,
                &paragraphs,
                &mut scope_alignments,
            )?)
        } else {
            None
        };
        let scope_alignments = scope_alignments.freeze();
        if let Some(slots) = family_slots {
            let resources = scope_alignment::ScopeAlignmentResources::new(
                scope_alignments.clone(),
                Default::default(),
            );
            let families = fragmentation::prepare_families::<Tree>(
                &paragraphs,
                &resolved.wrapping,
                &wrapping_input.child_order,
                &resources,
                slots,
            )?;
            resolved.associations.set_inline_box_families(families);
        }
        let associations = std::sync::Arc::new(resolved.associations);
        Ok(Self {
            tree,
            root,
            paragraphs,
            invalidated,
            graph,
            associations,
            wrapping: std::sync::Arc::new(wrapping),
            scope_alignments,
        })
    }

    /// Borrows the exact checked immutable paragraph associations.
    pub fn paragraphs(&self) -> &[TextParagraphBinding<Tree::Node>] {
        &self.paragraphs
    }

    /// The immutable tree whose normalized facts were checked by construction.
    pub fn tree(&self) -> &'tree Tree {
        self.tree
    }
}

fn binding_error<Node: Copy, S: crate::LayoutScalar, M>(
    node: Node,
    reason: TextBindingError,
) -> crate::LayoutErrorOf<Node, S, M> {
    crate::LayoutErrorOf::new(
        crate::LayoutErrorSiteOf::Node(node),
        crate::LayoutOperation::TextCapture,
        crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::TextBinding(reason)),
    )
}
