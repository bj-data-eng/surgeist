//! Original own-domain cut equivalence shared by admission and observation.

use super::{ScopeCapture, SourceBoxSide, SourceProcessingEvent};
use std::sync::{Arc, Mutex};

/// Only complete original-admission facts are published. Failed construction
/// leaves the cache empty, so another analyzed core can retry independently.
#[derive(Debug, Default)]
pub(crate) struct CanonicalCache(Mutex<Option<Arc<CanonicalFlow>>>);

impl Clone for CanonicalCache {
    fn clone(&self) -> Self {
        // A private deep topology copy may still be changed before admission.
        // Ordinary captured handles clone their Arc, not this topology/cache.
        Self::default()
    }
}

impl ScopeCapture {
    pub(crate) fn canonical_flow(&self) -> crate::Result<Arc<CanonicalFlow>> {
        // Poison recovery is safe: the guard protects publication alone, and
        // no partially built or mutable canonical facts enter this Option.
        let mut value = self
            .canonical
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(flow) = &*value {
            return Ok(Arc::clone(flow));
        }
        // The original topology and endpoint validation precede computed
        // admission. Later computed-row appends are not dependencies here.
        let flow = Arc::new(CanonicalFlow::new(self)?);
        *value = Some(Arc::clone(&flow));
        Ok(flow)
    }
}

#[cfg(test)]
impl ScopeCapture {
    pub(crate) fn canonical_payload(
        &self,
        census: &mut crate::engine::analysis::scope::IndexPayloadCensus,
    ) {
        let value = self
            .canonical
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(flow) = value {
            flow.payload(census);
        }
    }
}

#[derive(Debug)]
pub(crate) struct CanonicalFlow {
    orders: Vec<Vec<usize>>,
    paragraphs: Vec<Vec<(usize, usize, usize)>>,
    scopes: Vec<Option<(usize, usize)>>,
    edges: Vec<Vec<usize>>,
    transitions: Vec<Vec<usize>>,
    endpoints: Vec<Option<(usize, SourceBoxSide)>>,
}

#[cfg(test)]
impl CanonicalFlow {
    pub(crate) fn payload(
        self: &Arc<Self>,
        census: &mut crate::engine::analysis::scope::IndexPayloadCensus,
    ) {
        if !census.shared(self) {
            return;
        }
        census.buffer(&self.orders);
        for values in &self.orders {
            census.buffer(values);
        }
        census.buffer(&self.paragraphs);
        for values in &self.paragraphs {
            census.buffer(values);
        }
        census.buffer(&self.scopes);
        census.buffer(&self.edges);
        for values in &self.edges {
            census.buffer(values);
        }
        census.buffer(&self.transitions);
        for values in &self.transitions {
            census.buffer(values);
        }
        census.buffer(&self.endpoints);
    }
}

impl CanonicalFlow {
    fn new(capture: &ScopeCapture) -> crate::Result<Self> {
        let mut orders = Vec::new();
        let mut paragraphs = Vec::new();
        super::super::reserve(&mut orders, capture.domain_scopes.len())?;
        super::super::reserve(&mut paragraphs, capture.domain_scopes.len())?;
        for _ in &capture.domain_scopes {
            orders.push(Vec::new());
            paragraphs.push(Vec::new());
        }
        for (index, event) in capture.events.iter().enumerate() {
            let domain = match event.event {
                SourceProcessingEvent::EnterScope(i) | SourceProcessingEvent::ExitScope(i) => {
                    Some(capture.scopes[i].domain)
                }
                SourceProcessingEvent::Connection(_) => None,
                _ => event.paragraph.map(|p| capture.paragraphs[p].domain),
            };
            if let Some(domain) = domain {
                super::super::reserve(&mut orders[domain], 1)?;
                orders[domain].push(index);
            }
        }
        // Plain child-block boundaries are real parent-flow fences. Atomic and
        // out-of-flow content scheduling supplies no additional parent slots.
        for (domain, flow) in capture.flow_orders.iter().enumerate() {
            super::super::reserve(&mut orders[domain], flow.len())?;
            orders[domain].extend(flow);
            orders[domain].sort_unstable();
            orders[domain].dedup();
        }
        for (index, paragraph) in capture.paragraphs.iter().enumerate() {
            super::super::reserve(&mut paragraphs[paragraph.domain], 1)?;
            paragraphs[paragraph.domain].push((paragraph.enter, paragraph.exit, index));
        }
        for domain in &mut paragraphs {
            domain.sort_unstable_by_key(|p| p.0);
        }
        let mut scopes = Vec::new();
        super::super::reserve(&mut scopes, capture.scopes.len())?;
        scopes.resize(capture.scopes.len(), None);
        let mut endpoints = Vec::new();
        super::super::reserve(&mut endpoints, capture.controls.len())?;
        endpoints.resize(capture.controls.len(), None);
        let mut edges = Vec::new();
        super::super::reserve(&mut edges, capture.paragraphs.len())?;
        edges.resize(capture.paragraphs.len(), Vec::new());
        let mut transitions = Vec::new();
        super::super::reserve(&mut transitions, capture.domain_scopes.len())?;
        transitions.resize(capture.domain_scopes.len(), Vec::new());
        let mut flow = Self {
            orders,
            paragraphs,
            scopes,
            edges,
            transitions,
            endpoints,
        };
        for (index, scope) in capture.scopes.iter().enumerate() {
            if scope.definition.is_some() {
                continue;
            }
            let before = flow.rank(scope.domain, scope.enter);
            let after = flow.rank(scope.domain, scope.exit + 1);
            flow.scopes[index] = Some((before, after));
            super::super::reserve(&mut flow.transitions[scope.domain], 2)?;
            flow.transitions[scope.domain].extend([before, after]);
            for (side, target) in [
                (SourceBoxSide::Before, before),
                (SourceBoxSide::After, after),
            ] {
                if let Some(paragraph) = flow.scope_paragraph(capture, index, side) {
                    super::super::reserve(&mut flow.edges[paragraph], 1)?;
                    flow.edges[paragraph].push(target);
                }
            }
            if let Some(pair) = scope.boundaries {
                flow.endpoints[pair.start] = Some((before, SourceBoxSide::Before));
                flow.endpoints[pair.end] = Some((after, SourceBoxSide::After));
            }
        }
        for ranks in flow.edges.iter_mut().chain(&mut flow.transitions) {
            ranks.sort_unstable();
            ranks.dedup();
        }
        Ok(flow)
    }

    pub(crate) fn rank(&self, domain: usize, cut: usize) -> usize {
        self.orders[domain].partition_point(|&event| event < cut)
    }

    pub(crate) fn len(&self, domain: usize) -> usize {
        self.orders[domain].len()
    }

    pub(crate) fn edges(&self, paragraph: usize) -> &[usize] {
        &self.edges[paragraph]
    }

    pub(crate) fn transitions(&self, domain: usize) -> &[usize] {
        &self.transitions[domain]
    }

    pub(crate) fn scope_ranks(&self, scope: usize) -> Option<(usize, usize)> {
        self.scopes.get(scope).copied().flatten()
    }

    pub(crate) fn endpoint(&self, control: usize) -> Option<(usize, SourceBoxSide)> {
        self.endpoints.get(control).copied().flatten()
    }

    /// The exact locality rule used by computed scope-edge admission. A scope
    /// may enclose a paragraph whose Enter/Exit lies inside its original edge.
    pub(crate) fn scope_paragraph(
        &self,
        capture: &ScopeCapture,
        scope: usize,
        side: SourceBoxSide,
    ) -> Option<usize> {
        let fact = &capture.scopes[scope];
        let event = match side {
            SourceBoxSide::Before => fact.enter,
            SourceBoxSide::After => fact.exit,
        };
        let paragraph = capture.events[event].paragraph.or_else(|| {
            let paragraphs = &self.paragraphs[fact.domain];
            let candidate = match side {
                SourceBoxSide::Before => {
                    paragraphs.get(paragraphs.partition_point(|p| p.0 <= fact.enter))
                }
                SourceBoxSide::After => paragraphs
                    .partition_point(|p| p.1 < fact.exit)
                    .checked_sub(1)
                    .and_then(|i| paragraphs.get(i)),
            }?;
            (fact.enter < candidate.0 && candidate.1 < fact.exit).then_some(candidate.2)
        })?;
        let p = capture.paragraphs[paragraph];
        if capture.events[event].paragraph.is_none()
            && capture.events[event].at
                != capture.events[match side {
                    SourceBoxSide::Before => p.enter,
                    SourceBoxSide::After => p.exit,
                }]
                .at
        {
            return None;
        }
        Some(paragraph)
    }
}

#[cfg(test)]
mod tests;
