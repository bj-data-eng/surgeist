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
    /// Actual exhausted staged candidate; numerical fit remains consumer-owned.
    Exhausted(FinalizedLine),
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
            | Self::Exhausted(line)
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
    stage: ConditionalStage,
    alternative: Option<usize>,
    search: Option<SearchIdentity>,
    decisions: Vec<PhraseDecisionIdentity>,
    history: Vec<DispositionIdentity>,
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
            && self.stage == other.stage
            && self.alternative == other.alternative
            && self.search == other.search
            && self.decisions == other.decisions
            && self.history == other.history
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
        self.stage.hash(state);
        self.alternative.hash(state);
        self.search.hash(state);
        self.decisions.hash(state);
        self.history.hash(state);
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

    /// Actual terminating mixed boundary, before the ordinary ceiling.
    /// A Hyphens-stage capability may authorize its exact latent alternative at
    /// the ceiling; no same-ceiling nonhyphenating cut is introduced.
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
            || request
                .alternative
                .as_ref()
                .map(|alternative| alternative.index)
                != self.alternative
            || request
                .alternative
                .as_ref()
                .is_some_and(|alternative| !Arc::ptr_eq(&prepared.inner, &alternative.prepared))
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
        if let Some(search) = &self.search
            && !search.selection_context.is_empty()
            && selection_context(&request.context) != search.selection_context
        {
            return Err(invalid());
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
    fn arbitrary_opportunities(
        &self,
        start: &PreparedBoundary,
        incoming: Option<&super::HyphenationContinuation>,
        ceiling: &PreparedOpportunity,
        purpose: ConditionalPurpose,
        skipped_ordinary: &[ConditionalDisposition],
    ) -> crate::Result<Vec<PreparedConditionalOpportunity>> {
        super::continuation::validate(self, start, incoming)?;
        let (frontier, identities) =
            validate_query(self, start, incoming, ceiling, purpose, skipped_ordinary)?;
        let tables = &self.inner.tables;
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
                stage: ConditionalStage::Arbitrary,
                alternative: None,
                search: None,
                decisions: Vec::new(),
                history: Vec::new(),
            })
            .collect())
    }
}

fn validate_query(
    prepared: &PreparedText,
    start: &PreparedBoundary,
    incoming: Option<&super::HyphenationContinuation>,
    ceiling: &PreparedOpportunity,
    purpose: ConditionalPurpose,
    skipped_ordinary: &[ConditionalDisposition],
) -> crate::Result<(usize, Vec<DispositionIdentity>)> {
    for (owner, kind, occurrence) in [
        (&start.prepared, ProtocolHandleKind::Boundary, start.index),
        (
            &ceiling.prepared,
            ProtocolHandleKind::Opportunity,
            ceiling.index,
        ),
    ] {
        if !Arc::ptr_eq(&prepared.inner, owner) {
            return Err(protocol_error(ProtocolError::ForeignPreparation {
                kind,
                occurrence,
            }));
        }
    }
    let tables = &prepared.inner.tables;
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
    validate_frontier(
        prepared,
        start,
        incoming,
        ceiling,
        purpose,
        skipped_ordinary,
    )
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
        .skip(tables.opportunities.partition_point(|row| {
            tables.boundaries[row.boundary].participant
                <= tables.boundaries[start.index].participant
        }))
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
                        if receipt.nonformatted().is_err() {
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
                ConditionalDisposition::Exhausted(_) => return Err(invalid()),
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
fn role(value: Option<bool>) -> u64 {
    match value {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    }
}
fn number(values: &mut Vec<u64>, value: f64) {
    values.push(if value == 0.0 { 0 } else { value.to_bits() });
}
pub(crate) fn request_context(request: super::input::RequestInput<'_>) -> Vec<u64> {
    let context = request.context;
    let mut values = vec![
        context.base_direction as u64,
        context.line_box_direction as u64,
        context.orientation as u64,
        context.start as u64,
        context.end as u64,
        context.spacing as u64,
        role(context.formatted_line.first()),
        role(context.formatted_line.last()),
    ];
    values.push(u64::from(context.available_inline.is_some()));
    if let Some(width) = context.available_inline {
        number(&mut values, width);
    }
    number(&mut values, context.indent);
    number(&mut values, context.alignment_insets.line_left());
    number(&mut values, context.alignment_insets.line_right());
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
    for allocation in request.atomics {
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
    for allocation in request.controls {
        values.push(allocation.occurrence.index as u64);
        values.push(u64::from(allocation.boundary.is_some()));
        if let Some(boundary) = allocation.boundary {
            number(&mut values, boundary.advance());
        }
        values.push(u64::from(allocation.strut.is_some()));
        if let Some(strut) = allocation.strut {
            values.push(strut.alignment as u64);
            number(&mut values, strut.baseline);
            number(&mut values, strut.line_extent);
        }
    }
    values.push(request.boundary_context.len() as u64);
    for allocation in request.boundary_context {
        values.push(allocation.occurrence.index as u64);
        number(&mut values, allocation.boundary.advance());
    }
    values.push(request.line_box_struts.len() as u64);
    for allocation in request.line_box_struts {
        values.push(allocation.alignment as u64);
        number(&mut values, allocation.metrics.baseline);
        number(&mut values, allocation.metrics.line_extent);
    }
    values.push(u64::from(request.line_strut.is_some()));
    if let Some(strut) = request.line_strut {
        number(&mut values, strut.baseline);
        number(&mut values, strut.line_extent);
    }
    values
}
fn policy_context(line: &FinalizedLine) -> Vec<u64> {
    let selected = line.selected_line();
    let mut values = request_context(selected.request().into());
    number(&mut values, line.feedback().advance);
    for &amount in &line.inner.data.spacing_amounts {
        number(&mut values, amount);
    }
    values
}

/// Ordered finite relaxation tiers; intrinsic queries use only Arbitrary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConditionalStage {
    Normal,
    Hyphens,
    Arbitrary,
}

/// Consumer-owned fit decision backed by an authentic otherwise-empty trial.
#[derive(Clone, Debug, PartialEq)]
pub enum PhraseOverflowDecision {
    Fits(FinalizedLine),
    Overflows(FinalizedLine),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PhraseDecisionIdentity {
    portion: super::PreparedPhrasePortion,
    overflows: bool,
    context: Vec<u64>,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct SearchIdentity {
    prepared: PreparedText,
    start: usize,
    incoming: Option<super::HyphenationContinuation>,
    ceiling: usize,
    purpose: ConditionalPurpose,
    ordinary: Vec<DispositionIdentity>,
    frame: Vec<u64>,
    selection_context: Vec<u64>,
    auto_phrase: bool,
}
/// Same-preparation query semantics; repeated equal queries have equal identity.
#[derive(Clone, Debug)]
pub struct PreparedConditionalSearch {
    identity: SearchIdentity,
    skipped: Arc<[ConditionalDisposition]>,
    frontier: usize,
}
impl PartialEq for PreparedConditionalSearch {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
impl Eq for PreparedConditionalSearch {}
impl Hash for PreparedConditionalSearch {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.identity.hash(h)
    }
}

/// One issued tier's complete ordered candidate variants.
#[derive(Clone, Debug)]
pub struct PreparedConditionalBatch {
    search: PreparedConditionalSearch,
    stage: ConditionalStage,
    decisions: Vec<PhraseDecisionIdentity>,
    history: Vec<DispositionIdentity>,
    opportunities: Vec<PreparedConditionalOpportunity>,
    receipts: Arc<[ConditionalDisposition]>,
}
impl PartialEq for PreparedConditionalBatch {
    fn eq(&self, other: &Self) -> bool {
        self.search == other.search
            && self.stage == other.stage
            && self.decisions == other.decisions
            && self.history == other.history
            && self.opportunities == other.opportunities
    }
}
impl Eq for PreparedConditionalBatch {}
impl Hash for PreparedConditionalBatch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.search.hash(state);
        self.stage.hash(state);
        self.decisions.hash(state);
        self.history.hash(state);
        self.opportunities.hash(state);
    }
}
impl PreparedText {
    /// Captures one ordinary interval and its complete earlier variant dispositions.
    /// Finite AutoPhrase searches require an actual failed ordinary `frame` from
    /// this preparation/start/incoming. Consumers own fit and frame freshness;
    /// Text retains its visible context and authenticates all issued capabilities.
    /// Min-content requires no finite frame and independently issues only the
    /// arbitrary opportunities permitted by overflow-wrap:anywhere.
    pub fn conditional_search(
        &self,
        start: &PreparedBoundary,
        incoming: Option<&super::HyphenationContinuation>,
        ceiling: &PreparedOpportunity,
        purpose: ConditionalPurpose,
        skipped: &[ConditionalDisposition],
        frame: Option<&FinalizedLine>,
    ) -> crate::Result<PreparedConditionalSearch> {
        let (frontier, ordinary) =
            validate_query(self, start, incoming, ceiling, purpose, skipped)?;
        let auto_phrase = self.inner.tables.auto_phrase_between(
            start.index,
            self.inner.tables.opportunities[ceiling.index].boundary,
        );
        if purpose == ConditionalPurpose::Overflow && auto_phrase && frame.is_none() {
            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
        }
        let mut selection = Vec::new();
        let context = if let Some(frame) = frame {
            let selected = frame.selected_line();
            let request = selected.request();
            if frame.prepared_text() != *self
                || request.start != *start
                || request.incoming.as_ref() != incoming
                || request.context.available_inline.is_none()
                || purpose != ConditionalPurpose::Overflow
                || !matches!(request.termination, super::PreparedTermination::Ordinary(_))
            {
                return Err(protocol_error(ProtocolError::InvalidConditionalStage));
            }
            selection = selection_context(&request.context);
            policy_context(frame)
        } else {
            Vec::new()
        };
        Ok(PreparedConditionalSearch {
            identity: SearchIdentity {
                prepared: self.clone(),
                start: start.index,
                incoming: incoming.cloned(),
                ceiling: ceiling.index,
                purpose,
                ordinary,
                frame: context,
                selection_context: selection,
                auto_phrase,
            },
            skipped: skipped.to_vec().into(),
            frontier,
        })
    }
}
impl PreparedConditionalSearch {
    pub(super) fn matches_flow_query(
        &self,
        prepared: &PreparedText,
        start: usize,
        incoming: Option<&super::HyphenationContinuation>,
    ) -> bool {
        self.identity.prepared == *prepared
            && self.identity.start == start
            && self.identity.incoming.as_ref() == incoming
    }
    /// Whole remaining affected phrase pieces, clipped to the queried start.
    /// Measure each through `PhraseMeasurement`, including its exact incoming
    /// continuation. A fresh-line measurement may use a different numerical
    /// frame from the failed ordinary trial; its actual context is retained.
    pub fn phrase_portions(&self) -> Vec<super::PreparedPhrasePortion> {
        if self.identity.purpose != ConditionalPurpose::Overflow || !self.identity.auto_phrase {
            return Vec::new();
        }
        let prepared = &self.identity.prepared;
        let tables = &prepared.inner.tables;
        let ceiling = tables.opportunities[self.identity.ceiling].boundary;
        let mut result = Vec::new();
        let start_atom = tables.boundaries[self.identity.start].atom;
        let ceiling_atom = tables.boundaries[ceiling].atom;
        let phrases = &prepared.inner.analyzed.analysis.phrases.phrases;
        let first = phrases.partition_point(|phrase| phrase.atoms.end <= start_atom);
        let last = phrases.partition_point(|phrase| phrase.atoms.start < ceiling_atom);
        for (offset, phrase) in phrases[first..last].iter().enumerate() {
            let index = first + offset;
            let start = tables
                .boundaries
                .partition_point(|boundary| boundary.atom < phrase.atoms.start)
                .max(self.identity.start);
            let mut end = tables
                .boundaries
                .partition_point(|boundary| boundary.atom <= phrase.atoms.end)
                .saturating_sub(1);
            // Independent surviving offers divide the affected phrase, while the
            // original phrase identity remains whole across a restored line end.
            let next = tables
                .opportunities
                .partition_point(|row| row.boundary <= start);
            if let Some(ordinary) = tables.opportunities.get(next) {
                end = end.min(ordinary.boundary);
            }
            if start < end {
                result.push(super::PreparedPhrasePortion {
                    phrase: super::PreparedPhrase {
                        prepared: prepared.clone(),
                        index,
                    },
                    start,
                    end,
                    incoming: if start == self.identity.start {
                        self.identity.incoming.clone()
                    } else {
                        None
                    },
                    query: self.clone(),
                });
            }
        }
        result
    }
    /// Starts with exactly one decision per issued phrase portion, in order.
    /// Supported overflowing pieces unlock their suppressed Normal cuts;
    /// subsequent finite tiers restore eligible hyphens, then arbitrary cuts.
    /// Receipt identity uses normalized values rather than allocation identity.
    pub fn begin(
        &self,
        decisions: &[PhraseOverflowDecision],
    ) -> crate::Result<PreparedConditionalBatch> {
        let portions = self.phrase_portions();
        if portions.len() != decisions.len() {
            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
        }
        let mut identities = Vec::new();
        for (expected, decision) in portions.into_iter().zip(decisions) {
            let (trial, overflows) = match decision {
                PhraseOverflowDecision::Fits(line) => (line, false),
                PhraseOverflowDecision::Overflows(line) => (line, true),
            };
            let selected = trial.selected_line();
            let super::PreparedTermination::PhraseMeasurement(portion) =
                &selected.request().termination
            else {
                return Err(protocol_error(ProtocolError::InvalidPhrasePortion));
            };
            if portion != &expected {
                return Err(protocol_error(ProtocolError::InvalidPhrasePortion));
            }
            portion.validate(&self.identity.prepared, selected.request())?;
            identities.push(PhraseDecisionIdentity {
                portion: portion.clone(),
                overflows,
                context: policy_context(trial),
            });
        }
        let stage =
            if self.identity.purpose == ConditionalPurpose::Overflow && self.identity.auto_phrase {
                ConditionalStage::Normal
            } else {
                ConditionalStage::Arbitrary
            };
        self.batch(stage, identities, Vec::new(), Arc::from([]))
    }
    fn batch(
        &self,
        stage: ConditionalStage,
        decisions: Vec<PhraseDecisionIdentity>,
        history: Vec<DispositionIdentity>,
        receipts: Arc<[ConditionalDisposition]>,
    ) -> crate::Result<PreparedConditionalBatch> {
        let prepared = &self.identity.prepared;
        let tables = &prepared.inner.tables;
        let ceiling = tables.opportunities[self.identity.ceiling].boundary;
        let mut candidates = Vec::new();
        match stage {
            ConditionalStage::Normal => {
                let first = tables
                    .suppressed
                    .partition_point(|row| row.boundary <= self.frontier);
                let last = tables
                    .suppressed
                    .partition_point(|row| row.boundary < ceiling);
                for row in &tables.suppressed[first.min(last)..last] {
                    if decisions
                        .binary_search_by_key(&row.phrase, |decision| decision.portion.phrase.index)
                        .ok()
                        .is_some_and(|index| {
                            let decision = &decisions[index];
                            decision.overflows
                                && decision.portion.start < row.boundary
                                && row.boundary < decision.portion.end
                        })
                    {
                        candidates.push((row.boundary, None));
                    }
                }
            }
            ConditionalStage::Hyphens => {
                let rows = &tables.latent_alternatives;
                let first = rows
                    .partition_point(|index| tables.alternatives[*index].boundary <= self.frontier);
                let last =
                    rows.partition_point(|index| tables.alternatives[*index].boundary <= ceiling);
                candidates.extend(
                    rows[first.min(last)..last]
                        .iter()
                        .map(|index| (tables.alternatives[*index].boundary, Some(*index))),
                );
            }
            ConditionalStage::Arbitrary => {
                let start = PreparedBoundary {
                    prepared: Arc::clone(&prepared.inner),
                    index: self.identity.start,
                };
                let ceiling = PreparedOpportunity {
                    prepared: Arc::clone(&prepared.inner),
                    index: self.identity.ceiling,
                };
                candidates.extend(
                    prepared
                        .arbitrary_opportunities(
                            &start,
                            self.identity.incoming.as_ref(),
                            &ceiling,
                            self.identity.purpose,
                            &self.skipped,
                        )?
                        .into_iter()
                        .map(|cut| (cut.end, None)),
                );
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        let retained: Arc<[ConditionalDisposition]> = self
            .skipped
            .iter()
            .chain(receipts.iter())
            .cloned()
            .collect::<Vec<_>>()
            .into();
        let opportunities = candidates
            .into_iter()
            .map(|(end, alternative)| PreparedConditionalOpportunity {
                prepared: prepared.clone(),
                start: self.identity.start,
                incoming: self.identity.incoming.clone(),
                ceiling: self.identity.ceiling,
                purpose: self.identity.purpose,
                end,
                skipped: Arc::clone(&retained),
                identities: self.identity.ordinary.clone().into(),
                stage,
                alternative,
                search: Some(self.identity.clone()),
                decisions: decisions.clone(),
                history: history.clone(),
            })
            .collect();
        Ok(PreparedConditionalBatch {
            search: self.clone(),
            stage,
            decisions,
            history,
            opportunities,
            receipts,
        })
    }
}
impl PreparedConditionalBatch {
    pub fn stage(&self) -> ConditionalStage {
        self.stage
    }
    pub fn opportunities(&self) -> &[PreparedConditionalOpportunity] {
        &self.opportunities
    }
    /// Discharges every variant of this tier in issuance order and returns the
    /// next tier. Actual finalized receipts retain context and allocation roles;
    /// numerical fit and consumer hyphen policies remain consumer decisions.
    /// Arbitrary is terminal and returns `None`.
    pub fn advance(&self, dispositions: &[ConditionalDisposition]) -> crate::Result<Option<Self>> {
        if self.stage == ConditionalStage::Arbitrary {
            return Ok(None);
        }
        if dispositions.len() != self.opportunities.len() {
            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
        }
        let mut history = self.history.clone();
        for (index, (candidate, disposition)) in
            self.opportunities.iter().zip(dispositions).enumerate()
        {
            let mut identity = DispositionIdentity {
                kind: 6,
                opportunity: index,
                alternative: candidate.alternative,
                portion: None,
                line_limit: None,
                policy_context: Vec::new(),
                controls: Vec::new(),
                atomics: Vec::new(),
            };
            let automatic = candidate.alternative.is_some_and(|index| {
                matches!(
                    self.search.identity.prepared.inner.tables.alternatives[index].definition,
                    super::prepared::AlternativeDefinition::Automatic { .. }
                )
            });
            if let Some(receipt) = disposition.receipt().filter(|_| {
                !matches!(
                    disposition,
                    ConditionalDisposition::AutoManualPriority { .. }
                )
            }) {
                let selected = receipt.selected_line();
                let request = selected.request();
                if receipt.prepared_text() != self.search.identity.prepared
                    || !matches!(&request.termination,super::PreparedTermination::Conditional(cut) if cut==candidate)
                {
                    return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                }
                if matches!(
                    disposition,
                    ConditionalDisposition::ConsumerHyphenationPolicy(_)
                ) && candidate.alternative.is_none()
                {
                    return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                }
                match disposition {
                    ConditionalDisposition::Phantom(_) => {
                        if receipt.nonformatted().is_err() {
                            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                        }
                        identity.kind = 0;
                    }
                    ConditionalDisposition::AutoFitExhausted(_) if !automatic => {
                        return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                    }
                    ConditionalDisposition::AutoFitExhausted(_) => identity.kind = 2,
                    ConditionalDisposition::ConsumerHyphenationPolicy(_) => identity.kind = 5,
                    ConditionalDisposition::Exhausted(_) => {}
                    _ => return Err(protocol_error(ProtocolError::InvalidConditionalStage)),
                }
                identity.policy_context = policy_context(receipt);
            } else {
                match disposition {
                    ConditionalDisposition::AutoManualPriority {
                        alternative,
                        portion,
                    } if automatic
                        && candidate.alternative == Some(alternative.index)
                        && Arc::ptr_eq(
                            &alternative.prepared,
                            &self.search.identity.prepared.inner,
                        ) =>
                    {
                        let selected = portion.selected_line();
                        let super::PreparedTermination::Measurement(part) =
                            &selected.request().termination
                        else {
                            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                        };
                        part.validate(&self.search.identity.prepared, selected.request())?;
                        if selected.request().context.available_inline.is_none()
                            || !part.has_manual_partition()
                            || candidate.end <= part.start
                            || candidate.end > part.end
                            || !candidate.manual_portions()?.contains(part)
                        {
                            return Err(protocol_error(ProtocolError::InvalidConditionalStage));
                        }
                        identity.kind = 1;
                        identity.portion = Some(part.clone());
                        identity.policy_context = policy_context(portion);
                    }
                    ConditionalDisposition::AutoLineLimit { alternative, limit }
                        if automatic
                            && candidate.alternative == Some(alternative.index)
                            && Arc::ptr_eq(
                                &alternative.prepared,
                                &self.search.identity.prepared.inner,
                            ) =>
                    {
                        identity.kind = 4;
                        identity.line_limit = Some(*limit)
                    }
                    _ => return Err(protocol_error(ProtocolError::InvalidConditionalStage)),
                }
            }
            history.push(identity);
        }
        let next = match self.stage {
            ConditionalStage::Normal => ConditionalStage::Hyphens,
            ConditionalStage::Hyphens => ConditionalStage::Arbitrary,
            ConditionalStage::Arbitrary => unreachable!(),
        };
        let receipts = self
            .receipts
            .iter()
            .chain(dispositions)
            .cloned()
            .collect::<Vec<_>>()
            .into();
        self.search
            .batch(next, self.decisions.clone(), history, receipts)
            .map(Some)
    }
}
impl PreparedConditionalOpportunity {
    pub fn stage(&self) -> ConditionalStage {
        self.stage
    }
    pub fn alternative(&self) -> Option<super::PreparedAlternative> {
        self.alternative.map(|index| super::PreparedAlternative {
            prepared: Arc::clone(&self.prepared.inner),
            index,
        })
    }
    /// Same-preparation admitted manual partitions for this restored variant's
    /// lexical word. These measurement capabilities do not create ordinary cuts.
    pub fn manual_portions(&self) -> crate::Result<Vec<super::PreparedWordPortion>> {
        let Some(alternative) = self.alternative else {
            return Ok(Vec::new());
        };
        let cut = self.prepared.inner.tables.alternatives[alternative].hyphenation_cut;
        let Some(word) = self.prepared.inner.tables.hyphenation_cuts[cut].word else {
            return Ok(Vec::new());
        };
        super::hyphenation::restored_portions(&self.prepared, word)
    }
}

// These semantic line choices stay stable across candidate evaluation. Width,
// indent, alignment insets, effective alignment, line-box direction and tab coordinates come from
// each converged frame; the query retains its own frame identity without
// certifying Layout staleness.
fn selection_context(context: &super::LineContext) -> Vec<u64> {
    vec![
        context.base_direction as u64,
        context.orientation as u64,
        context.spacing as u64,
    ]
}
