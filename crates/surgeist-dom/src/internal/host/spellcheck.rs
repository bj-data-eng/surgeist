//! External per-Element spellcheck defaults, admitted against one DOM basis.

use super::*;
use crate::internal::snapshot::DomNode;
use std::{
    cmp::Ordering,
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
};
use surgeist_html_semantics::HtmlSpellcheckDefaultBehavior;

/// Bounds simultaneously live policy rows at publication and the work to qualify them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostSpellcheckLimits {
    capacity: usize,
    work: usize,
}
impl DomHostSpellcheckLimits {
    /// Capacity covers the current and proposed policies during replacement.
    pub const fn new(capacity: usize, work: usize) -> Self {
        Self { capacity, work }
    }
}

type Row = (NodeId, HtmlSpellcheckDefaultBehavior);

/// Immutable, qualified host policy. Omitted Elements have unavailable defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostSpellcheckDefaults(Arc<Vec<Row>>);
impl DomHostSpellcheckDefaults {
    /// Read the admitted default for this exact Element identity.
    pub fn default_for(&self, element: NodeId) -> Option<HtmlSpellcheckDefaultBehavior> {
        self.default_for_with(element, || Ok::<_, Infallible>(()))
            .unwrap_or_else(|never| match never {})
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn default_for_with<E>(
        &self,
        element: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlSpellcheckDefaultBehavior>, E> {
        let mut start = 0;
        let mut end = self.len();
        while start < end {
            step()?;
            let mid = start + (end - start) / 2;
            let (node, behavior) = self.0[mid];
            match key(node).cmp(&key(element)) {
                Ordering::Less => start = mid + 1,
                Ordering::Greater => end = mid,
                Ordering::Equal => return Ok((node == element).then_some(behavior)),
            }
        }
        Ok(None)
    }
}

fn key(node: NodeId) -> (usize, u64) {
    (node.slot, node.generation)
}

/// Refusal to admit a proposed spellcheck policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostSpellcheckError {
    RequiresBoundedPublication,
    Capacity { limit: usize },
    Work { limit: usize },
    AllocationFailed,
    Cancelled,
    DuplicateSubject(NodeId),
    InheritWithoutElementParent(NodeId),
}
impl fmt::Display for DomHostSpellcheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiresBoundedPublication => f.write_str("spellcheck defaults require a bounded policy publication"),
            Self::Capacity { limit } => write!(f, "spellcheck policy exceeds its capacity of {limit}"),
            Self::Work { limit } => write!(f, "spellcheck policy exceeds its work limit of {limit}"),
            Self::AllocationFailed => f.write_str("allocation of the spellcheck policy failed"),
            Self::Cancelled => f.write_str("spellcheck policy publication was cancelled"),
            Self::DuplicateSubject(_) => f.write_str("spellcheck policy names an Element more than once"),
            Self::InheritWithoutElementParent(_) => f.write_str("an Element without an ordinary Element parent cannot inherit its spellcheck default"),
        }
    }
}
impl std::error::Error for DomHostSpellcheckError {}

impl From<DomHostSpellcheckError> for DomHostError {
    fn from(error: DomHostSpellcheckError) -> Self {
        Self::Spellcheck(error)
    }
}

struct Work<'a> {
    limit: usize,
    remaining: usize,
    cancel: &'a AtomicBool,
}
impl Work<'_> {
    fn charge(&mut self, amount: usize) -> Result<(), DomHostSpellcheckError> {
        if self.cancel.load(AtomicOrdering::Acquire) {
            return Err(DomHostSpellcheckError::Cancelled);
        }
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(DomHostSpellcheckError::Work { limit: self.limit })?;
        Ok(())
    }
    fn node<'a>(
        &mut self,
        document: DomDocument<'a>,
        node: NodeId,
    ) -> Result<DomNode<'a>, DomHostError> {
        document
            .node_with(node, || self.charge(1).map_err(NodeLookupError::Work))
            .map_err(|error| match error {
                NodeLookupError::Dom(error) => DomHostError::Subject { node, error },
                NodeLookupError::Work(error) => error.into(),
            })
    }
}
enum NodeLookupError {
    Dom(DomError),
    Work(DomHostSpellcheckError),
}
impl From<DomError> for NodeLookupError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

// In-place heap formation checks work before traversing, comparing, or moving
// rows. Its order is independent of the submitting host's input order.
fn sort_rows(rows: &mut [Row], work: &mut Work<'_>) -> Result<(), DomHostSpellcheckError> {
    let len = rows.len();
    for start in (0..len / 2).rev() {
        sift_rows(rows, start, len, work)?;
    }
    for end in (1..len).rev() {
        work.charge(1)?;
        rows.swap(0, end);
        sift_rows(rows, 0, end, work)?;
    }
    Ok(())
}

fn sift_rows(
    rows: &mut [Row],
    mut root: usize,
    end: usize,
    work: &mut Work<'_>,
) -> Result<(), DomHostSpellcheckError> {
    while root < end / 2 {
        work.charge(1)?;
        let mut child = root * 2 + 1;
        if child + 1 < end {
            work.charge(1)?;
            if key(rows[child].0) < key(rows[child + 1].0) {
                child += 1;
            }
        }
        work.charge(1)?;
        if key(rows[root].0) >= key(rows[child].0) {
            break;
        }
        rows.swap(root, child);
        root = child;
    }
    Ok(())
}

impl DomHostPublisher {
    /// Publish host-authored defaults for the exact HTML Elements in this Document.
    ///
    /// An omitted Element remains unavailable. Equal tables preserve the host
    /// revision independently of input order. Inheriting requires an ordinary
    /// Element parent. Work covers backing formation, lookups, sorting and comparison.
    ///
    /// Returns a basis, subject, limit or cancellation error before publication.
    pub fn publish_spellcheck_defaults(
        &mut self,
        document: DomDocument<'_>,
        observed_basis: DomDocumentStamp,
        defaults: &[(NodeId, HtmlSpellcheckDefaultBehavior)],
        limits: DomHostSpellcheckLimits,
        cancel: &AtomicBool,
    ) -> Result<DomHostPublication, DomHostError> {
        let basis = self.validate_publication(
            document,
            observed_basis,
            DomHostDomains::SpellcheckDefaults,
        )?;
        let previous = self
            .latest()
            .and_then(|capture| match capture.observations() {
                DomHostObservations::SpellcheckDefaults(defaults) => Some(defaults),
                _ => None,
            });
        let retained = previous.map_or(0, |policy| policy.0.capacity());
        let capacity = limits
            .capacity
            .checked_sub(retained)
            .filter(|capacity| defaults.len() <= *capacity)
            .ok_or(DomHostSpellcheckError::Capacity {
                limit: limits.capacity,
            })?;
        let mut work = Work {
            limit: limits.work,
            remaining: limits.work,
            cancel,
        };
        work.charge(defaults.len())?;
        let mut current = Vec::new();
        current
            .try_reserve_exact(defaults.len())
            .map_err(|_| DomHostSpellcheckError::AllocationFailed)?;
        if current.capacity() > capacity {
            return Err(DomHostSpellcheckError::Capacity {
                limit: limits.capacity,
            }
            .into());
        }
        for &(node, behavior) in defaults {
            let subject = work.node(document, node)?;
            if subject
                .as_element()
                .is_none_or(|element| element.common_state().html().is_none())
            {
                return Err(DomHostError::Subject {
                    node,
                    error: DomError::WrongNodeKind,
                });
            }
            if behavior == HtmlSpellcheckDefaultBehavior::InheritByDefault {
                let parent = subject
                    .parent()
                    .ok_or(DomHostSpellcheckError::InheritWithoutElementParent(node))?;
                if work.node(document, parent)?.as_element().is_none() {
                    return Err(DomHostSpellcheckError::InheritWithoutElementParent(node).into());
                }
            }
            work.charge(1)?;
            current.push((node, behavior));
        }
        sort_rows(&mut current, &mut work)?;
        for adjacent in current.windows(2) {
            work.charge(1)?;
            if adjacent[0].0 == adjacent[1].0 {
                return Err(DomHostSpellcheckError::DuplicateSubject(adjacent[0].0).into());
            }
        }
        let mut changed = true;
        if let Some(previous) = previous
            && previous.len() == current.len()
        {
            changed = false;
            for (old, new) in previous.0.iter().zip(&current) {
                work.charge(1)?;
                if old != new {
                    changed = true;
                    break;
                }
            }
        }
        let observations =
            DomHostObservations::SpellcheckDefaults(DomHostSpellcheckDefaults(Arc::new(current)));
        work.charge(0)?;
        self.record_publication(basis, observations, changed)
    }
}
