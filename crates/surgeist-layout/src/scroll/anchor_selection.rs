use core::fmt;
use core::hash::Hash;
use core::slice;
use std::collections::HashSet;

use super::{ScrollRectOf, scroll_rect_axis_interval};
use crate::{DefaultScalar, LayoutScalar, PhysicalAxis};

#[cfg(test)]
mod tests;

/// The local used policy of the element associated with a box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollAnchorPolicy {
    #[default]
    Auto,
    None,
}

/// The actual absolute containing block in this scroller's frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAnchorContainingBlock<Node> {
    /// The containing block is the scroller or an element within it.
    Element(Node),
    /// The containing block is within the scroller and has no element identity.
    Anonymous,
    /// The containing block is an ancestor of the scroller.
    AboveScroller,
}

/// Positioning facts needed for anchor selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAnchorPosition<Node> {
    Ordinary,
    Fixed,
    Absolute(ScrollAnchorContainingBlock<Node>),
}

/// The element's box participation and scrollable overflow in scroller coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollAnchorBoxOf<S: LayoutScalar = DefaultScalar> {
    /// Display suppression excludes the element and its descendants.
    Suppressed,
    /// The element generates no box. Its descendants remain traversable.
    None,
    /// The element generates non-atomic inline boxes. Bounds can prune traversal.
    Inline(Option<ScrollRectOf<S>>),
    /// The element has a box that can serve as an anchor.
    Eligible(ScrollRectOf<S>),
}

pub type ScrollAnchorBox = ScrollAnchorBoxOf<DefaultScalar>;

/// Used box and exclusion facts for one DOM node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollAnchorSubjectOf<Node, S: LayoutScalar = DefaultScalar> {
    NonElement,
    Element {
        box_kind: ScrollAnchorBoxOf<S>,
        policy: ScrollAnchorPolicy,
        position: ScrollAnchorPosition<Node>,
    },
}

pub type ScrollAnchorSubject<Node> = ScrollAnchorSubjectOf<Node, DefaultScalar>;

impl<Node: Copy, S: LayoutScalar> ScrollAnchorSubjectOf<Node, S> {
    fn box_kind(self) -> ScrollAnchorBoxOf<S> {
        match self {
            Self::NonElement => ScrollAnchorBoxOf::None,
            Self::Element { box_kind, .. } => box_kind,
        }
    }

    fn opts_out(self) -> bool {
        matches!(
            self,
            Self::Element {
                policy: ScrollAnchorPolicy::None,
                ..
            }
        )
    }

    fn excluded(self) -> bool {
        self.opts_out()
            || matches!(
                self,
                Self::Element {
                    box_kind: ScrollAnchorBoxOf::Suppressed,
                    ..
                } | Self::Element {
                    position: ScrollAnchorPosition::Fixed
                        | ScrollAnchorPosition::Absolute(
                            ScrollAnchorContainingBlock::AboveScroller
                        ),
                    ..
                }
            )
    }
}

/// One borrowed record from a coherent tree and geometry observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollAnchorNodeOf<'a, Node, S: LayoutScalar = DefaultScalar> {
    pub parent: Option<Node>,
    /// Direct children in DOM order.
    pub children: &'a [Node],
    /// Absolute elements with this containing block and a different DOM parent,
    /// in DOM order.
    pub positioned: &'a [Node],
    pub subject: ScrollAnchorSubjectOf<Node, S>,
}

pub type ScrollAnchorNode<'a, Node> = ScrollAnchorNodeOf<'a, Node, DefaultScalar>;

/// Resolved priority designation from the same observation as the tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAnchorPriority<Node> {
    Absent,
    Candidate(Node),
    Unavailable,
}

/// A selection for one scroller and one optimal viewing region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollAnchorRequestOf<Node, S: LayoutScalar = DefaultScalar> {
    /// The element or document associated with the scrolling box.
    pub scroller: Node,
    /// The scroller's optimal viewing region in the candidate rectangles' frame.
    pub optimal_viewing_region: ScrollRectOf<S>,
    /// The element designated by the focused area's DOM anchor.
    pub focused: ScrollAnchorPriority<Node>,
    /// The first element containing the active find match.
    pub find_match: ScrollAnchorPriority<Node>,
}

pub type ScrollAnchorRequest<Node> = ScrollAnchorRequestOf<Node, DefaultScalar>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAnchorSelectionError<Node> {
    FocusUnavailable,
    FindMatchUnavailable,
    MissingNode(Node),
    InconsistentDomChild { parent: Node, child: Node },
    InconsistentPositionedChild { owner: Node, child: Node },
    Cycle(Node),
}

impl<Node: fmt::Debug> fmt::Display for ScrollAnchorSelectionError<Node> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FocusUnavailable => f.write_str("focused-area anchor is unavailable"),
            Self::FindMatchUnavailable => f.write_str("active find-match element is unavailable"),
            Self::MissingNode(node) => write!(f, "scroll anchor node {node:?} is unavailable"),
            Self::InconsistentDomChild { parent, child } => {
                write!(
                    f,
                    "scroll anchor node {child:?} is not a DOM child of {parent:?}"
                )
            }
            Self::InconsistentPositionedChild { owner, child } => {
                write!(
                    f,
                    "scroll anchor node {child:?} has no positioned edge from {owner:?}"
                )
            }
            Self::Cycle(node) => write!(f, "scroll anchor traversal repeats ancestor {node:?}"),
        }
    }
}

impl<Node: fmt::Debug> std::error::Error for ScrollAnchorSelectionError<Node> {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Clipped,
    Partial,
    Full,
}

fn visibility<S: LayoutScalar>(rect: ScrollRectOf<S>, region: ScrollRectOf<S>) -> Visibility {
    let mut fully_visible = true;
    for axis in [PhysicalAxis::Horizontal, PhysicalAxis::Vertical] {
        let (start, end) = scroll_rect_axis_interval(rect, axis);
        let (region_start, region_end) = scroll_rect_axis_interval(region, axis);
        if start.max(region_start) >= end.min(region_end) {
            return Visibility::Clipped;
        }
        fully_visible &= start >= region_start && end <= region_end;
    }
    if fully_visible {
        Visibility::Full
    } else {
        Visibility::Partial
    }
}

struct AnchorWalk<'a, Node> {
    owner: Node,
    children: slice::Iter<'a, Node>,
    positioned: slice::Iter<'a, Node>,
    fallback: bool,
}

impl<'a, Node: Copy> AnchorWalk<'a, Node> {
    fn new<S: LayoutScalar>(
        owner: Node,
        record: ScrollAnchorNodeOf<'a, Node, S>,
        fallback: bool,
    ) -> Self {
        Self {
            owner,
            children: record.children.iter(),
            positioned: record.positioned.iter(),
            fallback,
        }
    }
}

type AnchorSelection<Node> = Result<Option<Node>, ScrollAnchorSelectionError<Node>>;

/// Select an anchor according to the CSS Scroll Anchoring 1 selection rules.
///
/// The lookup borrows DOM relations and resolved box facts from one immutable
/// observation. All candidate rectangles use the same unrounded coordinate
/// frame as the optimal viewing region. An empty result is a resolved
/// selection. A required unavailable designation or node is an error.
pub fn select_scroll_anchor<'a, Node, S>(
    request: ScrollAnchorRequestOf<Node, S>,
    read: impl Fn(Node) -> Option<ScrollAnchorNodeOf<'a, Node, S>>,
) -> Result<Option<Node>, ScrollAnchorSelectionError<Node>>
where
    Node: Copy + Eq + Hash + 'a,
    S: LayoutScalar,
{
    let read = |node| read(node).ok_or(ScrollAnchorSelectionError::MissingNode(node));
    let scroller = read(request.scroller)?;
    if scroller.subject.opts_out() {
        return Ok(None);
    }
    for (priority, unavailable) in [
        (
            request.focused,
            ScrollAnchorSelectionError::FocusUnavailable,
        ),
        (
            request.find_match,
            ScrollAnchorSelectionError::FindMatchUnavailable,
        ),
    ] {
        match priority {
            ScrollAnchorPriority::Absent => {}
            ScrollAnchorPriority::Unavailable => return Err(unavailable),
            ScrollAnchorPriority::Candidate(node) => {
                if let Some(anchor) = select_priority(request, node, &read)? {
                    return Ok(Some(anchor));
                }
            }
        }
    }
    walk_dom(request, scroller, &read)
}

fn select_priority<'a, Node, S>(
    request: ScrollAnchorRequestOf<Node, S>,
    priority: Node,
    read: &impl Fn(Node) -> Result<ScrollAnchorNodeOf<'a, Node, S>, ScrollAnchorSelectionError<Node>>,
) -> AnchorSelection<Node>
where
    Node: Copy + Eq + Hash + 'a,
    S: LayoutScalar,
{
    let mut path = HashSet::new();
    let mut candidate = priority;
    while candidate != request.scroller {
        if !path.insert(candidate) {
            return Err(ScrollAnchorSelectionError::Cycle(candidate));
        }
        let record = read(candidate)?;
        match record.subject.box_kind() {
            ScrollAnchorBoxOf::Inline(_) => match record.parent {
                Some(parent) => candidate = parent,
                None => break,
            },
            ScrollAnchorBoxOf::Eligible(bounds) => {
                if visibility(bounds, request.optimal_viewing_region) != Visibility::Clipped
                    && reaches_scroller(candidate, request.scroller, read)?
                {
                    return Ok(Some(candidate));
                }
                break;
            }
            ScrollAnchorBoxOf::None | ScrollAnchorBoxOf::Suppressed => break,
        }
    }
    Ok(None)
}

fn reaches_scroller<'a, Node, S>(
    mut candidate: Node,
    scroller: Node,
    read: &impl Fn(Node) -> Result<ScrollAnchorNodeOf<'a, Node, S>, ScrollAnchorSelectionError<Node>>,
) -> Result<bool, ScrollAnchorSelectionError<Node>>
where
    Node: Copy + Eq + Hash + 'a,
    S: LayoutScalar,
{
    let mut path = HashSet::new();
    while candidate != scroller {
        if !path.insert(candidate) {
            return Err(ScrollAnchorSelectionError::Cycle(candidate));
        }
        let record = read(candidate)?;
        let Some(parent) = record.parent else {
            return Ok(false);
        };
        if record.subject.excluded() {
            return Ok(false);
        }
        candidate = parent;
    }
    Ok(true)
}

fn walk_dom<'a, Node, S>(
    request: ScrollAnchorRequestOf<Node, S>,
    scroller: ScrollAnchorNodeOf<'a, Node, S>,
    read: &impl Fn(Node) -> Result<ScrollAnchorNodeOf<'a, Node, S>, ScrollAnchorSelectionError<Node>>,
) -> AnchorSelection<Node>
where
    Node: Copy + Eq + Hash + 'a,
    S: LayoutScalar,
{
    let root = ScrollAnchorNodeOf {
        positioned: &[],
        ..scroller
    };
    let mut stack = vec![AnchorWalk::new(request.scroller, root, false)];
    let mut active = HashSet::from([request.scroller]);
    while let Some(frame) = stack.last_mut() {
        let (candidate, record) = if let Some(&child) = frame.children.next() {
            let record = read(child)?;
            if record.parent != Some(frame.owner) {
                return Err(ScrollAnchorSelectionError::InconsistentDomChild {
                    parent: frame.owner,
                    child,
                });
            }
            (child, record)
        } else if let Some(&child) = frame.positioned.next() {
            let record = read(child)?;
            if record.parent == Some(frame.owner)
                || !matches!(
                    record.subject,
                    ScrollAnchorSubjectOf::Element {
                        position: ScrollAnchorPosition::Absolute(
                            ScrollAnchorContainingBlock::Element(owner),
                        ),
                        ..
                    } if owner == frame.owner
                )
            {
                return Err(ScrollAnchorSelectionError::InconsistentPositionedChild {
                    owner: frame.owner,
                    child,
                });
            }
            if !reaches_scroller(child, request.scroller, read)? {
                continue;
            }
            (child, record)
        } else {
            let owner = frame.owner;
            if frame.fallback {
                return Ok(Some(owner));
            }
            stack.pop();
            active.remove(&owner);
            continue;
        };
        if active.contains(&candidate) {
            return Err(ScrollAnchorSelectionError::Cycle(candidate));
        }
        if record.subject.excluded() {
            continue;
        }
        let fallback = match record.subject.box_kind() {
            ScrollAnchorBoxOf::Eligible(bounds) => {
                match visibility(bounds, request.optimal_viewing_region) {
                    Visibility::Clipped => continue,
                    Visibility::Full => return Ok(Some(candidate)),
                    Visibility::Partial => true,
                }
            }
            ScrollAnchorBoxOf::Inline(bounds) => {
                if bounds.is_some_and(|bounds| {
                    visibility(bounds, request.optimal_viewing_region) == Visibility::Clipped
                }) {
                    continue;
                }
                false
            }
            ScrollAnchorBoxOf::None => false,
            ScrollAnchorBoxOf::Suppressed => continue,
        };
        active.insert(candidate);
        stack.push(AnchorWalk::new(candidate, record, fallback));
    }
    Ok(None)
}
