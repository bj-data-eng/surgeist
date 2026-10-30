//! Source relationships are independent of joint analysis permission.

use super::{
    Capture, ContextualSourceInput, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphConnectionKind, ProcessingBarrier, ProcessingContext, Result, ScopeCapture,
    SourceProcessingConnection, SourceProcessingError, SourceProcessingEvent, invalid,
};
use crate::processing::{delimiter_crosses, reserve};

/// Invalid evidence for an explicitly supplied source connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceProcessingConnectionError {
    Endpoints {
        connection: usize,
    },
    Domain {
        connection: usize,
    },
    Position {
        connection: usize,
    },
    Duplicate {
        connection: usize,
        first_connection: usize,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedConnectionData {
    pub(crate) fact: SourceProcessingConnection,
    pub(crate) order: usize,
    /// A cross-view policy retains its actual global target, never a fake local row.
    pub(crate) boundary_policy: Option<ParagraphBoundaryPolicy>,
}

impl CapturedConnectionData {
    pub(super) fn paragraphs(&self, capture: &ScopeCapture) -> (usize, usize) {
        let paragraph = |origin: usize| {
            capture.events[capture.origin_orders[origin]]
                .paragraph
                .expect("connection endpoints are admitted original occurrences")
        };
        (
            paragraph(self.fact.before_origin),
            paragraph(self.fact.after_origin),
        )
    }
}

fn refusal(reason: SourceProcessingConnectionError) -> crate::Error {
    invalid(SourceProcessingError::SourceConnection { reason })
}

/// Walk the sole capture's actual per-domain original sequence. Empty
/// occurrences participate; child-domain originals never replace parent ones.
pub(in crate::processing) fn visit_source_neighbors(
    capture: &ScopeCapture,
    mut visit: impl FnMut(usize, usize, usize) -> Result<()>,
) -> Result<()> {
    let mut previous = Vec::new();
    reserve(&mut previous, capture.domain_scopes.len())?;
    previous.resize(capture.domain_scopes.len(), None);
    for event in &capture.events {
        if let SourceProcessingEvent::Origin { origin, .. } = event.event {
            let paragraph = event.paragraph.expect("admitted original has a paragraph");
            let domain = capture.paragraphs[paragraph].domain;
            if let Some(before) = previous[domain] {
                visit(domain, before, origin)?;
            }
            previous[domain] = Some(origin);
        }
    }
    Ok(())
}

/// Validate every supplied edge against the full original event stream. The
/// returned scratch index contains only supplied, validated relationships:
/// absence is unknown and does not infer continuity or a disconnected fence.
pub(super) fn admit(
    input: &ContextualSourceInput,
    topology: &mut ScopeCapture,
    captures: &mut [Capture],
    connection_orders: &[usize],
    context: &mut ProcessingContext,
) -> Result<Vec<Option<usize>>> {
    let mut next: Vec<Option<usize>> = Vec::new();
    let mut provided: Vec<Option<usize>> = Vec::new();
    reserve(&mut next, input.origins.len())?;
    reserve(&mut provided, input.origins.len())?;
    next.resize(input.origins.len(), None);
    provided.resize(input.origins.len(), None);
    // This is allowable endpoint adjacency, not a source-connection graph.
    // Empty origins remain occurrences; child-domain originals occupy their
    // own flow and do not replace the actual parent-domain neighbors.
    visit_source_neighbors(topology, |_, before, after| {
        next[before] = Some(after);
        Ok(())
    })?;

    reserve(&mut topology.connections, input.connections.len())?;
    for (connection, (fact, &order)) in input.connections.iter().zip(connection_orders).enumerate()
    {
        if fact.before_origin >= input.origins.len() || fact.after_origin >= input.origins.len() {
            return Err(refusal(SourceProcessingConnectionError::Endpoints {
                connection,
            }));
        }
        let row = CapturedConnectionData {
            fact: *fact,
            order,
            boundary_policy: None,
        };
        let (before, after) = row.paragraphs(topology);
        let event = &topology.events[order];
        let domain = event.before.map(|scope| topology.scopes[scope].domain);
        if domain != Some(topology.paragraphs[before].domain)
            || domain != Some(topology.paragraphs[after].domain)
        {
            return Err(refusal(SourceProcessingConnectionError::Domain {
                connection,
            }));
        }
        if next[fact.before_origin] != Some(fact.after_origin) {
            return Err(refusal(SourceProcessingConnectionError::Endpoints {
                connection,
            }));
        }
        if order <= topology.origin_orders[fact.before_origin]
            || order >= topology.origin_orders[fact.after_origin]
            || event.at != input.origins[fact.after_origin].aggregate.start
        {
            return Err(refusal(SourceProcessingConnectionError::Position {
                connection,
            }));
        }
        if let Some(first_connection) = provided[fact.before_origin] {
            return Err(refusal(SourceProcessingConnectionError::Duplicate {
                connection,
                first_connection,
            }));
        }
        // Local delimiter checks retain their original position after computed
        // request admission, preserving its request-owned precise diagnostics.
        if before != after
            && (delimiter_crosses(
                &input.segment_breaks,
                input.origins[fact.before_origin].aggregate.end,
            ) || delimiter_crosses(
                &input.segment_breaks,
                input.origins[fact.after_origin].aggregate.start,
            ))
        {
            return Err(invalid(SourceProcessingError::DelimiterCrossing));
        }
        provided[fact.before_origin] = Some(connection);
        topology.connections.push(row);
    }

    // Raw paragraph projections keep local relationships only, whether or not
    // any joint declarations exist. Cross relationships remain globally owned.
    for capture in captures {
        let mut kept = 0;
        for index in 0..capture.connections.len() {
            let connection = capture.connections[index];
            let (before, after) = topology.connections[connection].paragraphs(topology);
            if before == after {
                capture.connections[kept] = connection;
                capture.connection_orders[kept] = capture.connection_orders[index];
                capture.connection_anchor_cuts[kept] = capture.connection_anchor_cuts[index];
                kept += 1;
            }
        }
        capture.connections.truncate(kept);
        capture.connection_orders.truncate(kept);
        capture.connection_anchor_cuts.truncate(kept);
    }
    for connection in 0..topology.connections.len() {
        let row = &topology.connections[connection];
        let (before, after) = row.paragraphs(topology);
        if before == after {
            continue;
        }
        if row.fact.kind == ParagraphConnectionKind::Disconnected {
            reserve(&mut context.barriers, 1)?;
            context.barriers.push(ProcessingBarrier {
                at: input.origins[row.fact.after_origin].aggregate.start,
                domain: topology.paragraphs[before].domain,
                order: row.order,
                plaintext_reset: false,
                atomic: None,
                scope: None,
            });
        }
        for policy in &input.boundary_policies {
            if policy.definition == ParagraphBoundaryDefinition::Connection(connection) {
                if topology.connections[connection].boundary_policy.is_some() {
                    return Err(invalid(SourceProcessingError::BoundaryPolicy));
                }
                topology.connections[connection].boundary_policy = Some(*policy);
            }
        }
        if topology.connections[connection].fact.kind == ParagraphConnectionKind::Continuous
            && topology.connections[connection].boundary_policy.is_none()
        {
            return Err(invalid(SourceProcessingError::BoundaryPolicy));
        }
    }
    Ok(provided)
}
