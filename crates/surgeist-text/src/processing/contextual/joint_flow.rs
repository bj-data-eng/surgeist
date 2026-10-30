//! Explicit joint authority over existing raw paragraph occurrences.

use super::{
    Capture, ContextualSourceInput, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphConnectionKind, ProcessingBarrier, ProcessingContext, Result, ScopeCapture,
    SourceProcessingConnection, SourceProcessingError, invalid,
};
use crate::processing::{delimiter_crosses, reserve};

/// Producer authorization to analyze complete raw paragraph coverage jointly.
/// All ordinals refer to this same ContextualSourceInput, not issued handles.
/// The scope is an actual domain root. Members describe a contiguous interval
/// in that domain's event order, retaining empty members and child-domain holes.
#[derive(Clone, Debug)]
pub struct SourceProcessingJointFlow {
    pub scope: usize,
    pub paragraphs: Vec<usize>,
    /// One listed existing paragraph; this introduces no direction/strut override.
    pub facts_owner: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceProcessingJointFlowError {
    EmptyMembers {
        declaration: usize,
    },
    InvalidScope {
        declaration: usize,
        scope: usize,
    },
    NotDomainRoot {
        declaration: usize,
        scope: usize,
    },
    InvalidMember {
        declaration: usize,
        paragraph: usize,
    },
    DuplicateMember {
        declaration: usize,
        paragraph: usize,
    },
    MemberAlreadyAssigned {
        declaration: usize,
        paragraph: usize,
        first_declaration: usize,
    },
    WrongDomain {
        declaration: usize,
        paragraph: usize,
    },
    MemberOrder {
        declaration: usize,
        previous: usize,
        paragraph: usize,
    },
    SkippedMember {
        declaration: usize,
        paragraph: usize,
    },
    FactsOwnerNotMember {
        declaration: usize,
        paragraph: usize,
    },
    DirectionMismatch {
        declaration: usize,
        paragraph: usize,
        owner: usize,
    },
    StrutMismatch {
        declaration: usize,
        paragraph: usize,
        owner: usize,
    },
    UndeclaredConnection {
        connection: usize,
    },
    ConnectionDomain {
        connection: usize,
    },
    ConnectionPosition {
        connection: usize,
    },
    ConnectionEndpoints {
        connection: usize,
    },
    DuplicateConnection {
        connection: usize,
        first_connection: usize,
    },
    MissingConnection {
        declaration: usize,
        before_origin: usize,
        after_origin: usize,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct JointMember {
    pub(crate) paragraph: usize,
    pub(crate) record: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedJointFlowData {
    pub(crate) scope: usize,
    pub(crate) domain: usize,
    pub(crate) members: Vec<JointMember>,
    pub(crate) facts_owner: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedConnectionData {
    pub(crate) fact: SourceProcessingConnection,
    pub(crate) order: usize,
    /// A genuine cross-view policy has no invented raw paragraph-local target.
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

fn refusal(reason: SourceProcessingJointFlowError) -> crate::Error {
    invalid(SourceProcessingError::JointFlow { reason })
}

/// All event/topology admission precedes this normalization. No partial joint
/// authority escapes; existing independent raw projections remain unchanged.
pub(super) fn admit(
    input: &ContextualSourceInput,
    topology: &mut ScopeCapture,
    captures: &mut [Capture],
    paragraph_order: &[usize],
    connection_orders: &[usize],
    context: &mut ProcessingContext,
) -> Result<()> {
    reserve(&mut topology.connections, input.connections.len())?;
    for (fact, &order) in input.connections.iter().zip(connection_orders) {
        topology.connections.push(CapturedConnectionData {
            fact: *fact,
            order,
            boundary_policy: None,
        });
    }
    if input.joint_flows.is_empty() {
        return Ok(());
    }
    let mut records = Vec::new();
    reserve(&mut records, input.paragraphs.len())?;
    records.resize(input.paragraphs.len(), 0);
    for (record, &paragraph) in paragraph_order.iter().enumerate() {
        records[paragraph] = record;
    }
    reserve(&mut topology.joint_flows, input.joint_flows.len())?;
    for (declaration, flow) in input.joint_flows.iter().enumerate() {
        let scope = topology.scopes.get(flow.scope).ok_or_else(|| {
            refusal(SourceProcessingJointFlowError::InvalidScope {
                declaration,
                scope: flow.scope,
            })
        })?;
        if scope.definition.is_none() || topology.domain_scopes[scope.domain] != flow.scope {
            return Err(refusal(SourceProcessingJointFlowError::NotDomainRoot {
                declaration,
                scope: flow.scope,
            }));
        }
        let domain = scope.domain;
        if flow.paragraphs.is_empty() {
            return Err(refusal(SourceProcessingJointFlowError::EmptyMembers {
                declaration,
            }));
        }
        let mut members: Vec<JointMember> = Vec::new();
        reserve(&mut members, flow.paragraphs.len())?;
        for &paragraph in &flow.paragraphs {
            let fact = topology.paragraphs.get(paragraph).ok_or_else(|| {
                refusal(SourceProcessingJointFlowError::InvalidMember {
                    declaration,
                    paragraph,
                })
            })?;
            if let Some(first_declaration) = fact.joint_flow {
                return Err(refusal(if first_declaration == declaration {
                    SourceProcessingJointFlowError::DuplicateMember {
                        declaration,
                        paragraph,
                    }
                } else {
                    SourceProcessingJointFlowError::MemberAlreadyAssigned {
                        declaration,
                        paragraph,
                        first_declaration,
                    }
                }));
            }
            if fact.domain != domain {
                return Err(refusal(SourceProcessingJointFlowError::WrongDomain {
                    declaration,
                    paragraph,
                }));
            }
            let record = records[paragraph];
            if let Some(previous) = members.last() {
                if record <= previous.record {
                    return Err(refusal(SourceProcessingJointFlowError::MemberOrder {
                        declaration,
                        previous: previous.paragraph,
                        paragraph,
                    }));
                }
                for &skipped in &paragraph_order[previous.record + 1..record] {
                    if topology.paragraphs[skipped].domain == domain {
                        return Err(refusal(SourceProcessingJointFlowError::SkippedMember {
                            declaration,
                            paragraph: skipped,
                        }));
                    }
                }
            }
            topology.paragraphs[paragraph].joint_flow = Some(declaration);
            members.push(JointMember { paragraph, record });
        }
        if !flow.paragraphs.contains(&flow.facts_owner) {
            return Err(refusal(
                SourceProcessingJointFlowError::FactsOwnerNotMember {
                    declaration,
                    paragraph: flow.facts_owner,
                },
            ));
        }
        let owner = topology.paragraphs[flow.facts_owner].facts;
        for member in &members {
            let facts = topology.paragraphs[member.paragraph].facts;
            if facts.base_direction != owner.base_direction {
                return Err(refusal(SourceProcessingJointFlowError::DirectionMismatch {
                    declaration,
                    paragraph: member.paragraph,
                    owner: flow.facts_owner,
                }));
            }
            if facts.paragraph_strut != owner.paragraph_strut {
                return Err(refusal(SourceProcessingJointFlowError::StrutMismatch {
                    declaration,
                    paragraph: member.paragraph,
                    owner: flow.facts_owner,
                }));
            }
        }
        topology.joint_flows.push(CapturedJointFlowData {
            scope: flow.scope,
            domain,
            members,
            facts_owner: flow.facts_owner,
        });
    }

    let mut next = Vec::new();
    let mut edge_owner = Vec::new();
    let mut seen = Vec::new();
    reserve(&mut next, input.origins.len())?;
    reserve(&mut edge_owner, input.origins.len())?;
    reserve(&mut seen, input.origins.len())?;
    next.resize(input.origins.len(), None);
    edge_owner.resize(input.origins.len(), None);
    seen.resize(input.origins.len(), None);
    for (declaration, flow) in topology.joint_flows.iter().enumerate() {
        let mut previous = None;
        for member in &flow.members {
            for &origin in &captures[member.paragraph].origins {
                if let Some(before) = previous {
                    next[before] = Some(origin);
                    edge_owner[before] = Some(declaration);
                }
                previous = Some(origin);
            }
        }
    }
    for (paragraph, capture) in captures.iter().enumerate() {
        if topology.paragraphs[paragraph].joint_flow.is_none() {
            for pair in capture.origins.windows(2) {
                next[pair[0]] = Some(pair[1]);
            }
        }
    }
    for (connection, row) in topology.connections.iter().enumerate() {
        if row.fact.before_origin >= input.origins.len()
            || row.fact.after_origin >= input.origins.len()
        {
            return Err(refusal(
                SourceProcessingJointFlowError::ConnectionEndpoints { connection },
            ));
        }
        let (before, after) = row.paragraphs(topology);
        if before != after {
            let joint = topology.paragraphs[before].joint_flow;
            if joint.is_none() || joint != topology.paragraphs[after].joint_flow {
                return Err(refusal(
                    SourceProcessingJointFlowError::UndeclaredConnection { connection },
                ));
            }
        }
        let event = &topology.events[row.order];
        let domain = event.before.map(|scope| topology.scopes[scope].domain);
        if domain != Some(topology.paragraphs[before].domain)
            || domain != Some(topology.paragraphs[after].domain)
        {
            return Err(refusal(SourceProcessingJointFlowError::ConnectionDomain {
                connection,
            }));
        }
        if next[row.fact.before_origin] != Some(row.fact.after_origin) {
            return Err(refusal(
                SourceProcessingJointFlowError::ConnectionEndpoints { connection },
            ));
        }
        if row.order <= topology.origin_orders[row.fact.before_origin]
            || row.order >= topology.origin_orders[row.fact.after_origin]
            || event.at != input.origins[row.fact.after_origin].aggregate.start
        {
            return Err(refusal(
                SourceProcessingJointFlowError::ConnectionPosition { connection },
            ));
        }
        if let Some(first_connection) = seen[row.fact.before_origin] {
            return Err(refusal(
                SourceProcessingJointFlowError::DuplicateConnection {
                    connection,
                    first_connection,
                },
            ));
        }
        seen[row.fact.before_origin] = Some(connection);
    }
    for (before_origin, after) in next.iter().enumerate() {
        if let Some(after_origin) = after
            && seen[before_origin].is_none()
        {
            if let Some(declaration) = edge_owner[before_origin] {
                return Err(refusal(SourceProcessingJointFlowError::MissingConnection {
                    declaration,
                    before_origin,
                    after_origin: *after_origin,
                }));
            }
            return Err(invalid(SourceProcessingError::Connection));
        }
    }

    // The raw views keep only their actual local edges. Cross-view edges and
    // their policies remain in the global capture, not a fabricated local row.
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
        if delimiter_crosses(
            &input.segment_breaks,
            input.origins[row.fact.before_origin].aggregate.end,
        ) || delimiter_crosses(
            &input.segment_breaks,
            input.origins[row.fact.after_origin].aggregate.start,
        ) {
            return Err(invalid(SourceProcessingError::DelimiterCrossing));
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
    Ok(())
}
