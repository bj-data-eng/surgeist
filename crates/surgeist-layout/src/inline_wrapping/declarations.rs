use super::{Ancestry, AvoidClass, PreferenceFrontier, WrappingScopeError};
use std::sync::Arc;
use surgeist_text::CapturedScope;

/// Noninherited wrapping behavior at one actual box edge.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WrapEdge {
    #[default]
    Auto,
    Avoid,
    AvoidLine,
    AvoidFlex,
    Line,
    Flex,
}
/// Original normalized declarations, independent of effective propagation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WrapEdges {
    pub before: WrapEdge,
    pub after: WrapEdge,
}
impl WrapEdges {
    pub const AUTO: Self = Self {
        before: WrapEdge::Auto,
        after: WrapEdge::Auto,
    };
    pub(crate) fn has_effect(self, mode: WrappingMode) -> bool {
        [self.before, self.after].iter().any(|d| {
            let e = d.effect(mode);
            e.forced || e.avoided
        })
    }
    pub(crate) fn is_auto(self) -> bool {
        self == Self::AUTO
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WrappingMode {
    Inline,
    Flex { multiline: bool },
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EdgeEffect {
    pub(crate) forced: bool,
    pub(crate) avoided: bool,
}
impl WrapEdge {
    pub(crate) fn effect(self, mode: WrappingMode) -> EdgeEffect {
        let (forced, avoided) = match mode {
            WrappingMode::Inline => (
                self == Self::Line,
                matches!(self, Self::Avoid | Self::AvoidLine),
            ),
            WrappingMode::Flex { multiline } => (
                multiline && self == Self::Flex,
                matches!(self, Self::Avoid | Self::AvoidFlex),
            ),
        };
        EdgeEffect { forced, avoided }
    }
}
/// A genuine non-atomic inline box, with phase-specific scope authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InlineBoxRef<Node, Scope = CapturedScope> {
    NativePair { start: Node, end: Node },
    Scope(Scope),
}
/// Generated box identity; physical boundary endpoints are not boxes themselves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedBoxRef<Node, Scope = CapturedScope> {
    Node(Node),
    Inline(InlineBoxRef<Node, Scope>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildOrderCompleteness {
    Complete,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedBoxParticipation {
    InFlow,
    OutOfFlow,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedChildBoxInput<Node, Scope = CapturedScope> {
    pub box_: GeneratedBoxRef<Node, Scope>,
    pub participation: GeneratedBoxParticipation,
}
/// Actual generated children and side-specific completeness within one context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InlineChildOrderInput<Node, Scope = CapturedScope> {
    pub parent: GeneratedBoxRef<Node, Scope>,
    pub children: Vec<GeneratedChildBoxInput<Node, Scope>>,
    pub prefix: ChildOrderCompleteness,
    pub suffix: ChildOrderCompleteness,
    pub formatting_context: Node,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrappingInput<Node> {
    pub scopes: Vec<super::WrappingScopeInput<Node>>,
    pub child_order: Vec<InlineChildOrderInput<Node>>,
}
impl<Node> Default for WrappingInput<Node> {
    fn default() -> Self {
        Self {
            scopes: Vec::new(),
            child_order: Vec::new(),
        }
    }
}
/// Shared comparison authority for actual flex item-side occurrences.
pub(crate) struct EdgePreferences {
    ancestry: Arc<Ancestry>,
    count: usize,
}
impl EdgePreferences {
    pub(crate) fn new(edge_count: usize) -> Self {
        Self {
            ancestry: Arc::default(),
            count: edge_count,
        }
    }
    pub(crate) fn class(&self, edges: &[usize]) -> Result<AvoidClass, WrappingScopeError> {
        if edges.iter().any(|&edge| edge >= self.count) {
            return Err(WrappingScopeError::InconsistentIncidence);
        }
        let mut edges = edges.to_vec();
        edges.sort_unstable();
        edges.dedup();
        Ok(AvoidClass {
            ancestry: self.ancestry.clone(),
            scope: None,
            edges,
        })
    }
    pub(crate) fn frontier<T>(&self) -> PreferenceFrontier<T> {
        AvoidClass {
            ancestry: self.ancestry.clone(),
            scope: None,
            edges: Vec::new(),
        }
        .frontier()
    }
}
