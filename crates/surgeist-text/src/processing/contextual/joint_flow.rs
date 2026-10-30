//! Explicit joint authority over existing raw paragraph occurrences.

use super::{Capture, ContextualSourceInput, Result, ScopeCapture, SourceProcessingError, invalid};
use crate::processing::reserve;

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

fn refusal(reason: SourceProcessingJointFlowError) -> crate::Error {
    invalid(SourceProcessingError::JointFlow { reason })
}

/// All event/topology admission precedes this normalization. No partial joint
/// authority escapes; existing independent raw projections remain unchanged.
pub(super) fn admit(
    input: &ContextualSourceInput,
    topology: &mut ScopeCapture,
    paragraph_order: &[usize],
) -> Result<()> {
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

    Ok(())
}

/// An explicit joint must have every source relationship required by its
/// complete origin sequence. Independent cross-view seams have no such demand.
pub(super) fn validate_connections(
    topology: &ScopeCapture,
    captures: &[Capture],
    provided: &[Option<usize>],
) -> Result<()> {
    for (declaration, flow) in topology.joint_flows.iter().enumerate() {
        let mut previous: Option<usize> = None;
        for member in &flow.members {
            for &origin in &captures[member.paragraph].origins {
                if let Some(before_origin) = previous {
                    let present = provided[before_origin].is_some_and(|connection| {
                        topology.connections[connection].fact.after_origin == origin
                    });
                    if !present {
                        return Err(refusal(SourceProcessingJointFlowError::MissingConnection {
                            declaration,
                            before_origin,
                            after_origin: origin,
                        }));
                    }
                }
                previous = Some(origin);
            }
        }
    }
    Ok(())
}
