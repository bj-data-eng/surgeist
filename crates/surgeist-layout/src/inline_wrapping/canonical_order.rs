//! One canonical generated-order check shared by both capture phases.

use super::{
    ChildOrderCompleteness,
    context::{Relation, interval_relation},
};
use surgeist_text::{
    CapturedControl, CapturedEventPosition, CapturedParagraph, CapturedScope, ParagraphControlKind,
    ScopeRole,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalBox {
    Scope(CapturedScope),
    NativePair {
        start: CapturedControl,
        end: CapturedControl,
    },
    Occurrence {
        position: CapturedEventPosition,
        domain: CapturedScope,
    },
}
impl CanonicalBox {
    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, Self::NativePair { .. })
            || matches!(self, Self::Scope(scope) if scope.role() == ScopeRole::Inline)
    }

    pub(crate) fn domain(&self) -> CapturedScope {
        match self {
            Self::Scope(scope) => scope.domain(),
            Self::NativePair { start, .. } => start.paragraph().domain(),
            Self::Occurrence { domain, .. } => domain.clone(),
        }
    }
    fn position(&self) -> CapturedEventPosition {
        match self {
            Self::Scope(scope) => scope.enter_position(),
            Self::NativePair { start, .. } => start.event_position(),
            Self::Occurrence { position, .. } => position.clone(),
        }
    }
    fn end(&self) -> CapturedEventPosition {
        match self {
            Self::Scope(scope) => scope.exit_position(),
            Self::NativePair { end, .. } => end.event_position(),
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
    pub(crate) fn native_pair(
        &self,
        start: CapturedControl,
        end: CapturedControl,
    ) -> Result<CanonicalBox, CanonicalOrderError> {
        if *start.kind() != ParagraphControlKind::InlineBoundaryStart
            || *end.kind() != ParagraphControlKind::InlineBoundaryEnd
            || start.paragraph().domain() != end.paragraph().domain()
            || start.event_position().partial_cmp(&end.event_position())
                != Some(std::cmp::Ordering::Less)
        {
            return Err(CanonicalOrderError::Contradiction);
        }
        for scope in &self.scopes {
            let first = scope.entry_control().as_ref() == Some(&start);
            let last = scope.exit_control().as_ref() == Some(&end);
            if first || last {
                if first && last {
                    return Ok(CanonicalBox::Scope(scope.clone()));
                }
                return Err(CanonicalOrderError::Contradiction);
            }
        }
        Ok(CanonicalBox::NativePair { start, end })
    }

    pub(crate) fn validate_native(
        &self,
        native: &[CanonicalBox],
    ) -> Result<(), CanonicalOrderError> {
        for (i, pair) in native.iter().enumerate() {
            if !matches!(pair, CanonicalBox::NativePair { .. }) {
                continue;
            }
            for other in self
                .scopes
                .iter()
                .filter(|scope| scope.role() == ScopeRole::Inline)
                .cloned()
                .map(CanonicalBox::Scope)
                .chain(native[..i].iter().cloned())
            {
                if other.domain() == pair.domain()
                    && other != *pair
                    && interval_relation(
                        &pair.position(),
                        &pair.end(),
                        &other.position(),
                        &other.end(),
                    ) == Relation::Crossing
                {
                    return Err(CanonicalOrderError::Contradiction);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn parent(
        &self,
        child: &CanonicalBox,
        native: &[CanonicalBox],
    ) -> Result<Option<CanonicalBox>, CanonicalOrderError> {
        let domain = child.domain();
        if !child.position().same_capture(&domain.enter_position()) {
            return Err(CanonicalOrderError::Unavailable);
        }
        let mut closest: Option<CanonicalBox> = None;
        for candidate in self
            .scopes
            .iter()
            .filter(|scope| scope.role() == ScopeRole::Inline)
            .cloned()
            .map(CanonicalBox::Scope)
            .chain(native.iter().cloned())
        {
            if candidate == *child || candidate.domain() != domain {
                continue;
            }
            if interval_relation(
                &candidate.position(),
                &candidate.end(),
                &child.position(),
                &child.end(),
            ) != Relation::Contains
            {
                continue;
            }
            if closest.as_ref().is_none_or(|prior| {
                interval_relation(
                    &prior.position(),
                    &prior.end(),
                    &candidate.position(),
                    &candidate.end(),
                ) == Relation::Contains
            }) {
                closest = Some(candidate);
            }
        }
        Ok(closest.or(Some(CanonicalBox::Scope(domain))))
    }

    fn covers(
        &self,
        parent: &CanonicalBox,
        children: &[CanonicalBox],
        prefix: ChildOrderCompleteness,
        suffix: ChildOrderCompleteness,
    ) -> bool {
        let CanonicalCoverage::Paragraphs(paragraphs) = &self.coverage else {
            return true;
        };
        let start = if prefix == ChildOrderCompleteness::Complete {
            parent.position()
        } else {
            children
                .first()
                .map_or_else(|| parent.position(), CanonicalBox::position)
        };
        let end = if suffix == ChildOrderCompleteness::Complete {
            parent.end()
        } else {
            children
                .last()
                .map_or_else(|| parent.end(), CanonicalBox::end)
        };
        let domain = parent.domain();
        if !start.same_capture(&domain.enter_position()) || end < start {
            return false;
        }
        let Some(events) = domain.flow_events() else {
            return false;
        };
        let mut any = false;
        let mut fences = Vec::new();
        for event in events {
            use surgeist_text::CapturedFlowEvent as E;
            match event {
                E::ParagraphStart(paragraph)
                    if paragraph.enter_position() < end && start < paragraph.exit_position() =>
                {
                    if paragraph.domain() != domain || !paragraphs.contains(&paragraph) {
                        return false;
                    }
                    if let CanonicalBox::Scope(scope) = parent
                        && scope.role() == ScopeRole::Inline
                    {
                        let Some(intersection) = paragraph
                            .scope_intersections()
                            .find(|row| row.scope() == *scope)
                        else {
                            return false;
                        };
                        let local_start =
                            std::cmp::max_by(start.clone(), paragraph.enter_position(), |a, b| {
                                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                            });
                        let local_end =
                            std::cmp::min_by(end.clone(), paragraph.exit_position(), |a, b| {
                                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                            });
                        if local_start < intersection.enter_position()
                            || intersection.exit_position() < local_end
                        {
                            return false;
                        }
                    }
                    any = true;
                }
                // These are actual parent-flow fences, not child paragraph
                // coverage. Opaque child content never fills a missing piece
                // of the parent's required paragraph region.
                E::BlockStart(child)
                    if child.enter_position() < end && start < child.exit_position() =>
                {
                    fences.push(child)
                }
                E::BlockEnd(child)
                    if child.enter_position() < end && start < child.exit_position() =>
                {
                    let opened = fences.pop();
                    if opened.as_ref() != Some(&child) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        any && fences.is_empty()
    }

    pub(crate) fn validate_sibling(
        &self,
        parent: &CanonicalBox,
        current: &CanonicalBox,
        sibling: &CanonicalBox,
        side: surgeist_text::SourceBoxSide,
        native: &[CanonicalBox],
    ) -> Result<(), CanonicalOrderError> {
        if self.parent(current, native)?.as_ref() != Some(parent)
            || self.parent(sibling, native)?.as_ref() != Some(parent)
            || current.domain() != sibling.domain()
        {
            return Err(CanonicalOrderError::Contradiction);
        }
        let ordered = match side {
            surgeist_text::SourceBoxSide::Before => sibling.end() < current.position(),
            surgeist_text::SourceBoxSide::After => current.end() < sibling.position(),
        };
        if ordered {
            Ok(())
        } else {
            Err(CanonicalOrderError::Contradiction)
        }
    }

    pub(crate) fn validate(
        &self,
        parent: &CanonicalBox,
        children: &[CanonicalBox],
        prefix: ChildOrderCompleteness,
        suffix: ChildOrderCompleteness,
        native: &[CanonicalBox],
    ) -> Result<(), CanonicalOrderError> {
        for child in children {
            if child.domain() != parent.domain() {
                return Err(CanonicalOrderError::Contradiction);
            }
            if self.parent(child, native)?.as_ref() != Some(parent) {
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
            .chain(
                native
                    .iter()
                    .filter(|box_| matches!(box_, CanonicalBox::NativePair { .. }))
                    .cloned(),
            )
            .chain(self.occurrences.iter().cloned())
        {
            if self.parent(&child, native)?.as_ref() == Some(parent) {
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
