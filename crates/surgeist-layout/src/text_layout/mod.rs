//! The concrete immutable Text capture and its Layout-owned consuming adapter.

pub(crate) mod fit;
pub(crate) mod run;
pub(crate) mod state;
#[cfg(test)]
mod tests;

use surgeist_text::{PreparedAtomic, PreparedControl, PreparedOrigin, PreparedText};

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

/// Supplied association facts, checked together against the immutable tree by
/// `TextLayoutInput::try_new` before any layout computation can consume them.
#[derive(Clone, Debug, PartialEq)]
pub struct TextParagraphBinding<Node> {
    pub prepared: PreparedText,
    pub container: Node,
    pub nodes: Vec<Node>,
    pub origins: Vec<TextOriginBinding<Node>>,
    pub atomics: Vec<TextAtomicBinding<Node>>,
    pub controls: Vec<TextControlBinding<Node>>,
    /// Genuine normalized line allocation, when the caller supplies one.
    /// Absence never guesses an empty/default font or a zero-size line.
    pub line_strut: Option<surgeist_text::LineStrut>,
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
    ParticipantRole,
    MissingParagraph,
    RootMismatch,
    TabReferenceDirection,
}

/// A complete, checked Text participation capture over one borrowed tree.
///
/// The tree borrow prevents mutable replacement of its normalized inputs while
/// the capture is being consumed. Issued Text handles retain the exact original
/// preparation; independently prepared equal text cannot replace that basis.
pub struct TextLayoutInput<'tree, Tree: crate::LayoutTree> {
    pub(crate) tree: &'tree Tree,
    pub(crate) root: Tree::Node,
    pub(crate) paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
    pub(crate) invalidated: Vec<Tree::Node>,
}

impl<'tree, Tree: crate::LayoutTree> TextLayoutInput<'tree, Tree> {
    /// Validates the complete captured paragraph inventory against this root.
    ///
    /// This proves association to supplied immutable normalized inputs. It does
    /// not authenticate live DOM or computed-style facts absent from the tree.
    pub fn try_new(
        tree: &'tree Tree,
        root: Tree::Node,
        paragraphs: Vec<TextParagraphBinding<Tree::Node>>,
    ) -> crate::LayoutResultOf<Tree::Node, Self, Tree::Scalar, Tree::MeasureError> {
        let mut influencing = Vec::new();
        let mut occupied = Vec::new();
        for paragraph in &paragraphs {
            let error = |reason| binding_error(paragraph.container, reason);
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
            if paragraph.nodes.is_empty() {
                return Err(error(TextBindingError::EmptyRun));
            }
            let children: Vec<_> = tree.children(paragraph.container).collect();
            let ordinary = crate::inline::run_ranges(&children, |node| tree.layout_input(node))
                .iter()
                .any(|range| children[range.clone()] == paragraph.nodes);
            let window = children
                .windows(paragraph.nodes.len())
                .position(|nodes| nodes == paragraph.nodes);
            let complete_window = window.is_some_and(|start| {
                let end = start + paragraph.nodes.len();
                let transparent_float = |node| {
                    tree.layout_input(node).as_box().is_some_and(|input| {
                        !input.float.is_none()
                            && input.position != crate::Position::Absolute
                            && input.display != crate::Display::None
                    })
                };
                paragraph.nodes.iter().all(|&node| {
                    crate::inline::continues_run(tree.layout_input(node)) || transparent_float(node)
                }) && (start == 0
                    || !crate::inline::continues_run(tree.layout_input(children[start - 1])))
                    && (end == children.len()
                        || !crate::inline::continues_run(tree.layout_input(children[end])))
            });
            if !ordinary && !complete_window {
                return Err(error(TextBindingError::RunMembership));
            }
            for (index, node) in paragraph.nodes.iter().copied().enumerate() {
                if paragraph.nodes[..index].contains(&node) {
                    return Err(error(TextBindingError::DuplicateRunNode));
                }
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
        let invalidated = crate::engine::validation::validate_layout_request_with_text(
            tree,
            root,
            &influencing,
            &paragraphs,
        )?;
        // Every actual prepared-text node under this root must be covered. A
        // partial inventory cannot silently use the box-only compute entry.
        let mut pending = vec![root];
        let mut visited = Vec::new();
        while let Some(node) = pending.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if (tree.layout_input(node).as_inline_text().is_some()
                || tree.layout_input(node).as_text_control().is_some())
                && !occupied.contains(&node)
            {
                return Err(binding_error(node, TextBindingError::MissingParagraph));
            }
            pending.extend(tree.children(node));
        }
        Ok(Self {
            tree,
            root,
            paragraphs,
            invalidated,
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
