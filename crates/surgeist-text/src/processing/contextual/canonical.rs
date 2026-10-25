//! Original own-domain cut equivalence shared by admission and observation.

use super::{ScopeCapture, SourceBoxSide, SourceProcessingEvent};

pub(crate) struct CanonicalFlow {
    orders: Vec<Vec<usize>>,
    paragraphs: Vec<Vec<(usize, usize, usize)>>,
}

impl CanonicalFlow {
    pub(crate) fn new(capture: &ScopeCapture) -> crate::Result<Self> {
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
        Ok(Self { orders, paragraphs })
    }

    pub(crate) fn rank(&self, domain: usize, cut: usize) -> usize {
        self.orders[domain].partition_point(|&event| event < cut)
    }

    pub(crate) fn len(&self, domain: usize) -> usize {
        self.orders[domain].len()
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
