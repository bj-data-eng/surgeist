//! One canonical generated-order check shared by both capture phases.

use super::ChildOrderCompleteness;
use surgeist_text::{CapturedEventPosition, CapturedParagraph, CapturedScope, ScopeRole};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalBox {
    Scope(CapturedScope),
    Occurrence {
        position: CapturedEventPosition,
        domain: CapturedScope,
    },
}
impl CanonicalBox {
    pub(crate) fn domain(&self) -> CapturedScope {
        match self {
            Self::Scope(scope) => scope.domain(),
            Self::Occurrence { domain, .. } => domain.clone(),
        }
    }
    fn position(&self) -> CapturedEventPosition {
        match self {
            Self::Scope(scope) => scope.enter_position(),
            Self::Occurrence { position, .. } => position.clone(),
        }
    }
    fn end(&self) -> CapturedEventPosition {
        match self {
            Self::Scope(scope) => scope.exit_position(),
            Self::Occurrence { position, .. } => position.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CanonicalCoverage {
    /// The source adapter owns the complete result of its one consumed input.
    WholeInput,
    /// Normal admission owns precisely these actual captured paragraphs.
    Paragraphs(Vec<CapturedParagraph>),
}
#[derive(Clone, Debug)]
pub(crate) struct CanonicalOrderInventory {
    pub(crate) scopes: Vec<CapturedScope>,
    pub(crate) occurrences: Vec<CanonicalBox>,
    pub(crate) coverage: CanonicalCoverage,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum CanonicalOrderError {
    Unavailable,
    Contradiction,
}

impl CanonicalOrderInventory {
    fn parent(&self, child: &CanonicalBox) -> Result<Option<CapturedScope>, CanonicalOrderError> {
        if let CanonicalBox::Scope(scope) = child {
            if scope.role() != ScopeRole::Inline {
                return Err(CanonicalOrderError::Contradiction);
            }
            return Ok(scope.parent());
        }
        let position = child.position();
        let domain = child.domain();
        if !position.same_capture(&domain.enter_position()) {
            return Err(CanonicalOrderError::Unavailable);
        }
        Ok(self
            .scopes
            .iter()
            .filter(|scope| {
                scope.role() == ScopeRole::Inline
                    && scope.domain() == domain
                    && scope.enter_position() < position
                    && position < scope.exit_position()
            })
            .max_by(|a, b| {
                a.enter_position()
                    .partial_cmp(&b.enter_position())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .or(Some(domain)))
    }

    fn covers(
        &self,
        parent: &CapturedScope,
        children: &[CanonicalBox],
        prefix: ChildOrderCompleteness,
        suffix: ChildOrderCompleteness,
    ) -> bool {
        let CanonicalCoverage::Paragraphs(paragraphs) = &self.coverage else {
            return true;
        };
        if parent.role() == ScopeRole::Inline {
            let start = if prefix == ChildOrderCompleteness::Complete {
                parent.enter_position()
            } else {
                children
                    .first()
                    .map_or_else(|| parent.enter_position(), CanonicalBox::position)
            };
            let end = if suffix == ChildOrderCompleteness::Complete {
                parent.exit_position()
            } else {
                children
                    .last()
                    .map_or_else(|| parent.exit_position(), CanonicalBox::end)
            };
            return paragraphs.iter().any(|paragraph| {
                paragraph.domain() == parent.domain()
                    && paragraph.enter_position() < start
                    && end < paragraph.exit_position()
            });
        }
        let Some(events) = parent.flow_events() else {
            return false;
        };
        let mut any = false;
        for event in events {
            if let surgeist_text::CapturedFlowEvent::ParagraphStart(paragraph) = event {
                any = true;
                if !paragraphs.contains(&paragraph) {
                    return false;
                }
            }
        }
        any
    }

    pub(crate) fn validate(
        &self,
        parent: &CapturedScope,
        children: &[CanonicalBox],
        prefix: ChildOrderCompleteness,
        suffix: ChildOrderCompleteness,
    ) -> Result<(), CanonicalOrderError> {
        for child in children {
            if child.domain() != parent.domain() {
                return Err(CanonicalOrderError::Contradiction);
            }
            if self.parent(child)?.as_ref() != Some(parent) {
                return Err(CanonicalOrderError::Contradiction);
            }
        }
        let mut actual = Vec::new();
        for child in self
            .scopes
            .iter()
            .filter(|scope| scope.role() == ScopeRole::Inline)
            .cloned()
            .map(CanonicalBox::Scope)
            .chain(self.occurrences.iter().cloned())
        {
            if self.parent(&child)?.as_ref() == Some(parent) {
                actual.push(child);
            }
        }
        actual.sort_by(|a, b| {
            a.position()
                .partial_cmp(&b.position())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let (Some(first), Some(last)) = (children.first(), children.last()) {
            let start = actual
                .iter()
                .position(|child| child == first)
                .ok_or(CanonicalOrderError::Unavailable)?;
            let end = actual
                .iter()
                .position(|child| child == last)
                .ok_or(CanonicalOrderError::Unavailable)?;
            if start > end
                || actual[start..=end] != *children
                || prefix == ChildOrderCompleteness::Complete && start != 0
                || suffix == ChildOrderCompleteness::Complete && end + 1 != actual.len()
            {
                return Err(CanonicalOrderError::Contradiction);
            }
        } else if !actual.is_empty()
            && (prefix == ChildOrderCompleteness::Complete
                || suffix == ChildOrderCompleteness::Complete)
        {
            return Err(CanonicalOrderError::Contradiction);
        }
        if (!children.is_empty()
            || prefix == ChildOrderCompleteness::Complete
            || suffix == ChildOrderCompleteness::Complete)
            && !self.covers(parent, children, prefix, suffix)
        {
            return Err(CanonicalOrderError::Unavailable);
        }
        Ok(())
    }
}

/// Built by the normal association owner before its checked observations drop.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalOrderBindings<Node> {
    pub(crate) inventory: CanonicalOrderInventory,
    pub(crate) nodes: Vec<(Node, CanonicalBox)>,
    pub(crate) flows: Vec<(Node, CapturedScope)>,
}
