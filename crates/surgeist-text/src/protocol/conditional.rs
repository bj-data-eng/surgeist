//! Preparation-issued conditional cuts, without a claim about consumer fit.

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use super::{
    FinalizedLine, LineSelectionRequest, PreparedBoundary, PreparedOpportunity, PreparedText,
    ProtocolError, ProtocolHandleKind, protocol_error,
};

/// The consumer operation for which additional cuts may be selected.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConditionalPurpose {
    /// Finite fitting, after the consumer exhausted actual ordinary candidates.
    Overflow,
    /// Unconstrained min-content pieces; only overflow-wrap:anywhere contributes.
    MinContent,
}

/// A genuine eligible cut in one queried ordinary interval. Equality retains
/// query semantics rather than the allocation of a returned vector or proof.
#[derive(Clone, Debug)]
pub struct PreparedConditionalOpportunity {
    prepared: PreparedText,
    start: usize,
    ceiling: usize,
    purpose: ConditionalPurpose,
    end: usize,
    skipped: Arc<[FinalizedLine]>,
}

impl PartialEq for PreparedConditionalOpportunity {
    fn eq(&self, other: &Self) -> bool {
        self.prepared == other.prepared
            && self.start == other.start
            && self.ceiling == other.ceiling
            && self.purpose == other.purpose
            && self.end == other.end
    }
}
impl Eq for PreparedConditionalOpportunity {}
impl Hash for PreparedConditionalOpportunity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.prepared.hash(state);
        self.start.hash(state);
        self.ceiling.hash(state);
        self.purpose.hash(state);
        self.end.hash(state);
    }
}

impl PreparedConditionalOpportunity {
    #[must_use]
    pub fn prepared_text(&self) -> PreparedText {
        self.prepared.clone()
    }

    /// Exact unchanged queried start, including its mixed occurrence position.
    #[must_use]
    pub fn start(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.prepared.inner),
            index: self.start,
        }
    }

    /// First ordinary opportunity not proved phantom by the supplied receipts.
    #[must_use]
    pub fn ceiling(&self) -> PreparedOpportunity {
        PreparedOpportunity {
            prepared: Arc::clone(&self.prepared.inner),
            index: self.ceiling,
        }
    }

    #[must_use]
    pub fn purpose(&self) -> ConditionalPurpose {
        self.purpose
    }

    /// Actual terminating mixed boundary, strictly before the ceiling.
    #[must_use]
    pub fn boundary(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.prepared.inner),
            index: self.end,
        }
    }

    pub(super) fn validate(
        &self,
        prepared: &PreparedText,
        request: &LineSelectionRequest,
    ) -> crate::Result<()> {
        if prepared != &self.prepared {
            return Err(protocol_error(ProtocolError::ForeignPreparation {
                kind: ProtocolHandleKind::ConditionalOpportunity,
                occurrence: self.end,
            }));
        }
        let invalid = || {
            protocol_error(ProtocolError::InvalidConditionalSelection {
                start: request.start.index,
                end: request.end.index,
            })
        };
        if request.start.index != self.start
            || request.end.index != self.end
            || request.alternative.is_some()
        {
            return Err(invalid());
        }
        if request.context.available_inline.is_some()
            != (self.purpose == ConditionalPurpose::Overflow)
        {
            return Err(protocol_error(ProtocolError::InvalidConditionalContext {
                purpose: self.purpose,
            }));
        }
        // Geometry can converge differently, but a proof without retained
        // participation cannot authorize a changed semantic allocation role.
        for receipt in self.skipped.iter() {
            let prior = receipt.selected_line();
            for control in &prior.request().controls {
                if request
                    .controls
                    .iter()
                    .find(|actual| actual.occurrence == control.occurrence)
                    .is_none_or(|actual| actual.strut.is_some() != control.strut.is_some())
                {
                    return Err(invalid());
                }
            }
            for atomic in &prior.request().atomics {
                if request
                    .atomics
                    .iter()
                    .find(|actual| actual.occurrence == atomic.occurrence)
                    .is_none_or(|actual| {
                        std::mem::discriminant(&actual.allocation)
                            != std::mem::discriminant(&atomic.allocation)
                    })
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}

impl PreparedText {
    /// Issues content-ordered, eligible text EGC cuts before ceiling, retaining
    /// unchanged start. Earlier ordinary endpoints require actual finalized
    /// phantom receipts in inventory order: Allowed's no-alternative variant
    /// followed by every issued alternative; Discretionary's every issued
    /// alternative. Forced/Paragraph endings cannot be skipped. Each receipt
    /// must prove no retained flow with the finalized prefix observation,
    /// not merely zero width, empty ink or font-only metrics.
    ///
    /// Overflow receipts and selections have a finite available span;
    /// MinContent receipts and selections remain unconstrained. This query
    /// proves neither fit nor complete ordinary fit exhaustion. The consumer
    /// owns those operations and must also reject phantom conditional trials.
    /// All cuts retain their distinct occurrence custody. Equivalent repeated
    /// queries issue semantically equal capabilities independent of proof
    /// allocation and numerical width trials.
    ///
    /// Raw source analysis has a distinct characterized ordinary inventory and
    /// receives no additional cuts from this native operation. Common handle,
    /// interval and receipt validation still applies to those queries.
    pub fn conditional_opportunities(
        &self,
        start: &PreparedBoundary,
        ceiling: &PreparedOpportunity,
        purpose: ConditionalPurpose,
        skipped_ordinary: &[FinalizedLine],
    ) -> crate::Result<Vec<PreparedConditionalOpportunity>> {
        for (owner, kind, occurrence) in [
            (&start.prepared, ProtocolHandleKind::Boundary, start.index),
            (
                &ceiling.prepared,
                ProtocolHandleKind::Opportunity,
                ceiling.index,
            ),
        ] {
            if !Arc::ptr_eq(&self.inner, owner) {
                return Err(protocol_error(ProtocolError::ForeignPreparation {
                    kind,
                    occurrence,
                }));
            }
        }
        let tables = &self.inner.tables;
        let invalid = || {
            protocol_error(ProtocolError::InvalidConditionalQuery {
                start: start.index,
                ceiling: ceiling.index,
            })
        };
        let from = tables.boundaries.get(start.index).ok_or_else(invalid)?;
        if tables
            .opportunities
            .get(ceiling.index)
            .is_none_or(|row| tables.boundaries[row.boundary].participant <= from.participant)
        {
            return Err(invalid());
        }
        let frontier = validate_frontier(self, start, ceiling, purpose, skipped_ordinary)?;
        let end = tables.opportunities[ceiling.index].boundary;
        let boundaries = super::prepared::conditional_boundaries(
            &self.inner.analyzed,
            tables,
            start.index,
            end,
            purpose,
        )?;
        let skipped: Arc<[FinalizedLine]> = skipped_ordinary.to_vec().into();
        Ok(boundaries
            .into_iter()
            .filter(|end| *end > frontier)
            .map(|end| PreparedConditionalOpportunity {
                prepared: self.clone(),
                start: start.index,
                ceiling: ceiling.index,
                purpose,
                end,
                skipped: Arc::clone(&skipped),
            })
            .collect())
    }
}

fn validate_frontier(
    prepared: &PreparedText,
    start: &PreparedBoundary,
    ceiling: &PreparedOpportunity,
    purpose: ConditionalPurpose,
    receipts: &[FinalizedLine],
) -> crate::Result<usize> {
    use super::{LineEnd, PreparedBreakKind, PreparedTermination};
    let tables = &prepared.inner.tables;
    let mut cursor = 0;
    let mut frontier = start.index;
    for (index, row) in tables
        .opportunities
        .iter()
        .enumerate()
        .take(ceiling.index)
        .filter(|(_, row)| {
            tables.boundaries[row.boundary].participant > tables.boundaries[start.index].participant
        })
    {
        if matches!(
            row.kind,
            PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
        ) || (row.kind == PreparedBreakKind::Discretionary && row.alternatives.is_empty())
        {
            return Err(protocol_error(ProtocolError::InvalidConditionalQuery {
                start: start.index,
                ceiling: ceiling.index,
            }));
        }
        let variants = (row.kind == PreparedBreakKind::Allowed)
            .then_some(None)
            .into_iter()
            .chain(row.alternatives.iter().copied().map(Some));
        for variant in variants {
            let invalid = || {
                protocol_error(ProtocolError::InvalidConditionalReceipt {
                    receipt: cursor,
                    opportunity: index,
                })
            };
            let receipt = receipts.get(cursor).ok_or_else(invalid)?;
            if receipt.prepared_text() != *prepared {
                return Err(protocol_error(ProtocolError::ForeignFinalizedLine {
                    line: cursor,
                }));
            }
            let selected = receipt.selected_line();
            let request = selected.request();
            if request.start != *start
                || request.end.index != row.boundary
                || !matches!(&request.termination, PreparedTermination::Ordinary(offer) if offer.index == index)
                || request
                    .alternative
                    .as_ref()
                    .map(|alternative| alternative.index)
                    != variant
                || request.context.end != LineEnd::Soft
            {
                return Err(invalid());
            }
            if request.context.available_inline.is_some()
                != (purpose == ConditionalPurpose::Overflow)
            {
                return Err(protocol_error(ProtocolError::InvalidConditionalContext {
                    purpose,
                }));
            }
            let last = request.participants.last().ok_or_else(invalid)?;
            if receipt
                .prefix_at(&crate::TextSeam::AfterParticipant(last.clone()))?
                .has_retained_flow_participation()
            {
                return Err(protocol_error(ProtocolError::RetainedConditionalPrefix {
                    opportunity: index,
                    alternative: variant,
                }));
            }
            cursor += 1;
        }
        frontier = row.boundary;
    }
    if cursor != receipts.len() {
        return Err(protocol_error(ProtocolError::InvalidConditionalReceipt {
            receipt: cursor,
            opportunity: ceiling.index,
        }));
    }
    Ok(frontier)
}
