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

/// Complete variant dispositions in the actual earlier ordinary inventory.
/// Automatic dispositions discharge variant coverage, never certify consumer
/// fitting and never advance the genuine phantom frontier.
#[derive(Clone, Debug, PartialEq)]
pub enum ConditionalDisposition {
    Phantom(FinalizedLine),
    /// Consumer-owned successive-line policy suppresses this exact automatic
    /// variant before attempting it. Overflow means exhausted finite capacity;
    /// MinContent permits only the static zero limit. This does not suppress
    /// manual or no-alternative siblings, prove fit, or advance the phantom
    /// frontier. Rebuild the query when the committed policy context changes.
    AutoLineLimit {
        alternative: super::PreparedAlternative,
        limit: usize,
    },
    /// Consumer's finite manual-priority decision, backed by an exact
    /// admitted manual portion trial through the shared measurement finalizer.
    AutoManualPriority {
        alternative: super::PreparedAlternative,
        portion: FinalizedLine,
    },
    /// Actual ordinary variant trial; finite fit exhaustion remains consumer-owned.
    AutoFitExhausted(FinalizedLine),
    /// A finite consumer policy has rejected this exact hyphenated ordinary
    /// trial in its current sequence branch. The receipt authenticates the
    /// preparation, unchanged start, variant and actual trial context; Text does
    /// not certify the consumer's completed-tail policy decision. Source,
    /// Control and Automatic alternatives are eligible. Meaningful None,
    /// Forced and Paragraph variants remain barriers. This never advances the
    /// phantom frontier and cannot be reused for an intrinsic query.
    ConsumerHyphenationPolicy(FinalizedLine),
    /// Intrinsic automatic refinement, independent of finite fit failures.
    AutoIntrinsicRefinement(super::PreparedAlternative),
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DispositionIdentity {
    kind: u8,
    opportunity: usize,
    alternative: Option<usize>,
    portion: Option<super::PreparedWordPortion>,
    line_limit: Option<usize>,
    policy_context: Vec<u64>,
    controls: Vec<(usize, bool)>,
    atomics: Vec<(usize, bool)>,
}
impl ConditionalDisposition {
    fn receipt(&self) -> Option<&FinalizedLine> {
        match self {
            Self::Phantom(line)
            | Self::AutoFitExhausted(line)
            | Self::ConsumerHyphenationPolicy(line)
            | Self::AutoManualPriority { portion: line, .. } => Some(line),
            Self::AutoIntrinsicRefinement(_) | Self::AutoLineLimit { .. } => None,
        }
    }
}

/// A genuine eligible cut in one queried ordinary interval. Equality retains
/// query semantics rather than the allocation of a returned vector or proof.
#[derive(Clone, Debug)]
pub struct PreparedConditionalOpportunity {
    prepared: PreparedText,
    start: usize,
    incoming: Option<super::HyphenationContinuation>,
    ceiling: usize,
    purpose: ConditionalPurpose,
    end: usize,
    skipped: Arc<[ConditionalDisposition]>,
    identities: Arc<[DispositionIdentity]>,
}

impl PartialEq for PreparedConditionalOpportunity {
    fn eq(&self, other: &Self) -> bool {
        self.prepared == other.prepared
            && self.start == other.start
            && self.incoming == other.incoming
            && self.ceiling == other.ceiling
            && self.purpose == other.purpose
            && self.end == other.end
            && self.identities == other.identities
    }
}
impl Eq for PreparedConditionalOpportunity {}
impl Hash for PreparedConditionalOpportunity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.prepared.hash(state);
        self.start.hash(state);
        self.incoming.hash(state);
        self.ceiling.hash(state);
        self.purpose.hash(state);
        self.end.hash(state);
        self.identities.hash(state);
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

    /// Authentic ordinary ceiling after the completely discharged earlier variants.
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
        if request.incoming != self.incoming
            || request.start.index != self.start
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
        for receipt in self
            .skipped
            .iter()
            .filter_map(ConditionalDisposition::receipt)
        {
            let prior = receipt.selected_line();
            for control in &prior.request().controls {
                if request
                    .controls
                    .iter()
                    .find(|actual| actual.occurrence == control.occurrence)
                    .is_some_and(|actual| actual.strut.is_some() != control.strut.is_some())
                {
                    return Err(invalid());
                }
            }
            for atomic in &prior.request().atomics {
                if request
                    .atomics
                    .iter()
                    .find(|actual| actual.occurrence == atomic.occurrence)
                    .is_some_and(|actual| {
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
    /// Issues content-ordered, eligible text EGC cuts strictly before `ceiling`,
    /// retaining the exact mixed-occurrence `start`.
    ///
    /// Earlier ordinary variants require complete dispositions in inventory
    /// order: Allowed's no-alternative variant followed by each alternative,
    /// or every Discretionary alternative. Forced and Paragraph endpoints are
    /// barriers. A phantom receipt must prove no retained flow through its
    /// finalized prefix; zero width, empty ink, or font-only metrics do not.
    /// Only entirely phantom endpoints advance the phantom frontier.
    ///
    /// Automatic variants may instead carry a finite manual-priority portion
    /// receipt, an actual finite ordinary trial, or an intrinsic refinement
    /// disposition. These discharge variant coverage without asserting that
    /// Text verified consumer fit or advancing the phantom frontier. None and
    /// authored manual variants cannot use automatic dispositions. Measurement
    /// portions cannot substitute ordinary trial or phantom receipts.
    ///
    /// Overflow receipts and selections have finite available spans;
    /// MinContent is unconstrained and uses intrinsic dispositions. The consumer
    /// owns complete ordinary search, fit comparison, and rejection of phantom
    /// conditional trials. It must rebuild dispositions after material frame or
    /// participation changes. Selection checks the retained participation roles.
    ///
    /// Cuts retain distinct occurrence custody. Semantic equality includes the
    /// query, purpose and normalized variant/partition/participation identities,
    /// independently of receipt allocation. Ordinary automatic dispositions do
    /// not identify numerical trial widths. A finite `ConsumerHyphenationPolicy`
    /// discharge additionally retains its actual trial context, including the
    /// available-inline width and supplied allocation geometry, in that semantic
    /// identity; it is a consumer policy decision for that branch/context.
    ///
    /// Raw source analysis has its separately characterized ordinary inventory
    /// and receives no additional cuts from this native operation. Common owner,
    /// interval and disposition validation still applies.
    pub fn conditional_opportunities(
        &self,
        start: &PreparedBoundary,
        incoming: Option<&super::HyphenationContinuation>,
        ceiling: &PreparedOpportunity,
        purpose: ConditionalPurpose,
        skipped_ordinary: &[ConditionalDisposition],
    ) -> crate::Result<Vec<PreparedConditionalOpportunity>> {
        super::continuation::validate(self, start, incoming)?;
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
        let (frontier, identities) =
            validate_frontier(self, start, incoming, ceiling, purpose, skipped_ordinary)?;
        let end = tables.opportunities[ceiling.index].boundary;
        let boundaries = super::prepared::conditional_boundaries(
            &self.inner.analyzed,
            tables,
            start.index,
            end,
            purpose,
        )?;
        let skipped: Arc<[ConditionalDisposition]> = skipped_ordinary.to_vec().into();
        let identities: Arc<[DispositionIdentity]> = identities.into();
        Ok(boundaries
            .into_iter()
            .filter(|end| *end > frontier)
            .map(|end| PreparedConditionalOpportunity {
                prepared: self.clone(),
                start: start.index,
                incoming: incoming.cloned(),
                ceiling: ceiling.index,
                purpose,
                end,
                skipped: Arc::clone(&skipped),
                identities: Arc::clone(&identities),
            })
            .collect())
    }
}

fn validate_frontier(
    prepared: &PreparedText,
    start: &PreparedBoundary,
    incoming: Option<&super::HyphenationContinuation>,
    ceiling: &PreparedOpportunity,
    purpose: ConditionalPurpose,
    dispositions: &[ConditionalDisposition],
) -> crate::Result<(usize, Vec<DispositionIdentity>)> {
    use super::{LineEnd, PreparedBreakKind, PreparedTermination};
    let tables = &prepared.inner.tables;
    let mut cursor = 0;
    let mut frontier = start.index;
    let mut identities = Vec::new();
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
        let mut all_phantom = true;
        for variant in variants {
            let invalid = || {
                protocol_error(ProtocolError::InvalidConditionalReceipt {
                    receipt: cursor,
                    opportunity: index,
                })
            };
            let disposition = dispositions.get(cursor).ok_or_else(invalid)?;
            let automatic = variant.and_then(|i| match tables.alternatives[i].definition {
                super::prepared::AlternativeDefinition::Automatic { cut, .. } => Some(cut),
                _ => None,
            });
            let mut identity = DispositionIdentity {
                kind: 0,
                opportunity: index,
                alternative: variant,
                portion: None,
                line_limit: None,
                policy_context: Vec::new(),
                controls: Vec::new(),
                atomics: Vec::new(),
            };
            if let Some(receipt) = disposition.receipt() {
                if receipt.prepared_text() != *prepared {
                    return Err(protocol_error(ProtocolError::ForeignFinalizedLine {
                        line: cursor,
                    }));
                }
                let selected = receipt.selected_line();
                let request = selected.request();
                if request.context.available_inline.is_some()
                    != (purpose == ConditionalPurpose::Overflow)
                {
                    return Err(protocol_error(ProtocolError::InvalidConditionalContext {
                        purpose,
                    }));
                }
                identity.controls = request
                    .controls
                    .iter()
                    .map(|row| (row.occurrence.index, row.strut.is_some()))
                    .collect();
                identity.atomics = request
                    .atomics
                    .iter()
                    .map(|row| {
                        (
                            row.occurrence.index,
                            matches!(row.allocation, super::AtomicLineAllocation::InFlow { .. }),
                        )
                    })
                    .collect();
                identity.controls.sort_unstable();
                identity.atomics.sort_unstable();
            }
            match disposition {
                ConditionalDisposition::Phantom(receipt)
                | ConditionalDisposition::AutoFitExhausted(receipt)
                | ConditionalDisposition::ConsumerHyphenationPolicy(receipt) => {
                    let selected = receipt.selected_line();
                    let request = selected.request();
                    if request.incoming.as_ref() != incoming
                        || request.start != *start
                        || request.end.index != row.boundary
                        || !matches!(&request.termination, PreparedTermination::Ordinary(offer) if offer.index == index)
                        || request.alternative.as_ref().map(|alt| alt.index) != variant
                        || request.context.end != LineEnd::Soft
                    {
                        return Err(invalid());
                    }
                    if matches!(disposition, ConditionalDisposition::Phantom(_)) {
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
                    } else {
                        if purpose != ConditionalPurpose::Overflow
                            || if matches!(
                                disposition,
                                ConditionalDisposition::ConsumerHyphenationPolicy(_)
                            ) {
                                variant.is_none()
                            } else {
                                automatic.is_none()
                            }
                        {
                            return Err(invalid());
                        }
                        if matches!(
                            disposition,
                            ConditionalDisposition::ConsumerHyphenationPolicy(_)
                        ) {
                            identity.kind = 5;
                            identity.policy_context = policy_context(receipt);
                        } else {
                            identity.kind = 2;
                        }
                        all_phantom = false;
                    }
                }
                ConditionalDisposition::AutoManualPriority {
                    alternative,
                    portion,
                } => {
                    let Some(cut) = automatic else {
                        return Err(invalid());
                    };
                    if !Arc::ptr_eq(&prepared.inner, &alternative.prepared)
                        || Some(alternative.index) != variant
                        || purpose != ConditionalPurpose::Overflow
                    {
                        return Err(invalid());
                    }
                    let selected = portion.selected_line();
                    let PreparedTermination::Measurement(part) = &selected.request().termination
                    else {
                        return Err(invalid());
                    };
                    part.validate(prepared, selected.request())?;
                    if !part.has_manual_partition()
                        || tables.hyphenation_cuts[cut].word != Some(part.word.index)
                        || row.boundary <= part.start
                        || row.boundary > part.end
                    {
                        return Err(invalid());
                    }
                    identity.kind = 1;
                    identity.portion = Some(part.clone());
                    all_phantom = false;
                }
                ConditionalDisposition::AutoLineLimit { alternative, limit } => {
                    if automatic.is_none()
                        || (purpose == ConditionalPurpose::MinContent && *limit != 0)
                        || !Arc::ptr_eq(&prepared.inner, &alternative.prepared)
                        || Some(alternative.index) != variant
                    {
                        return Err(invalid());
                    }
                    identity.kind = 4;
                    identity.line_limit = Some(*limit);
                    all_phantom = false;
                }
                ConditionalDisposition::AutoIntrinsicRefinement(alternative) => {
                    if automatic.is_none()
                        || purpose != ConditionalPurpose::MinContent
                        || !Arc::ptr_eq(&prepared.inner, &alternative.prepared)
                        || Some(alternative.index) != variant
                    {
                        return Err(invalid());
                    }
                    identity.kind = 3;
                    all_phantom = false;
                }
            }
            identities.push(identity);
            cursor += 1;
        }
        if all_phantom {
            frontier = row.boundary;
        }
    }
    if cursor != dispositions.len() {
        return Err(protocol_error(ProtocolError::InvalidConditionalReceipt {
            receipt: cursor,
            opportunity: ceiling.index,
        }));
    }
    Ok((frontier, identities))
}

// Retain the actual finite trial's semantic geometry and participation, not its
// receipt allocation. All these numbers crossed successful finalization and are
// finite; canonical signed zero has the same semantic identity.
fn policy_context(line: &FinalizedLine) -> Vec<u64> {
    fn number(values: &mut Vec<u64>, value: f64) {
        values.push(if value == 0.0 { 0 } else { value.to_bits() });
    }
    let selected = line.selected_line();
    let request = selected.request();
    let context = &request.context;
    let mut values = vec![
        context.base_direction as u64,
        context.orientation as u64,
        context.start as u64,
        context.end as u64,
        context.alignment as u64,
        context.spacing as u64,
    ];
    number(
        &mut values,
        context.available_inline.expect("finite policy receipt"),
    );
    number(&mut values, context.indent);
    match context.tabs {
        super::TabContext::NoTabs => values.push(0),
        super::TabContext::FontAdvance => values.push(1),
        super::TabContext::IntrinsicCssStops => values.push(4),
        super::TabContext::CssStops { origin } => {
            values.push(if origin.is_some() { 3 } else { 2 });
            if let Some(origin) = origin {
                number(&mut values, origin);
            }
        }
    }
    values.push(request.atomics.len() as u64);
    for allocation in &request.atomics {
        values.push(allocation.occurrence.index as u64);
        match allocation.allocation {
            super::AtomicLineAllocation::OutOfFlow => values.push(0),
            super::AtomicLineAllocation::InFlow {
                flow,
                box_geometry,
                alignment,
            } => {
                values.extend([1, alignment as u64]);
                for value in [
                    flow.advance,
                    flow.before_baseline,
                    flow.after_baseline,
                    flow.occupied_start,
                    flow.occupied_end,
                    box_geometry.border.origin.x,
                    box_geometry.border.origin.y,
                    box_geometry.border.size.width,
                    box_geometry.border.size.height,
                    box_geometry.content.origin.x,
                    box_geometry.content.origin.y,
                    box_geometry.content.size.width,
                    box_geometry.content.size.height,
                    box_geometry.baseline,
                ] {
                    number(&mut values, value);
                }
            }
        }
    }
    values.push(request.controls.len() as u64);
    for allocation in &request.controls {
        values.push(allocation.occurrence.index as u64);
        values.push(u64::from(allocation.strut.is_some()));
        if let Some(strut) = allocation.strut {
            values.push(strut.alignment as u64);
            number(&mut values, strut.baseline);
            number(&mut values, strut.line_extent);
        }
    }
    values.push(u64::from(request.line_strut.is_some()));
    if let Some(strut) = request.line_strut {
        number(&mut values, strut.baseline);
        number(&mut values, strut.line_extent);
    }
    number(&mut values, line.feedback().advance);
    for slot in line.spacing_slots() {
        number(&mut values, slot.advance);
    }
    values
}
