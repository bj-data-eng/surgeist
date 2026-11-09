//! Complete font-independent observations of issued structural source windows.
use super::{
    prepared::{self, ParticipantData, PresentationUnitKind},
    *,
};
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::{InlineBoxKind, Result};
use std::sync::Arc;

/// An actual occurrence's semantic role, without its line-time dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticAtomicAllocation {
    pub occurrence: PreparedAtomic,
    pub kind: InlineBoxKind,
}

/// Actual endpoint participation and source/box custody, without a metric strut.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticControlAllocation {
    pub occurrence: PreparedControl,
    pub boundary: Option<InlineBoundaryAllocation>,
}
impl super::inline_box::BoundaryControl for SemanticControlAllocation {
    fn occurrence(&self) -> &PreparedControl {
        &self.occurrence
    }
    fn boundary(&self) -> Option<&InlineBoundaryAllocation> {
        self.boundary.as_ref()
    }
}

/// One complete issued section, optionally resumed at an authenticated seam.
///
/// First/last facts are fixed owner input. Missing facts remain unresolved;
/// this operation neither resolves nor executes metric-only Phase-II demands.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticWindowRequest {
    pub segment: PreparedSourceSegment,
    pub start: PreparedBoundary,
    pub incoming: Option<HyphenationContinuation>,
    pub completion: PreparedOpportunity,
    pub atomics: Vec<SemanticAtomicAllocation>,
    pub controls: Vec<SemanticControlAllocation>,
    pub boundary_context: Vec<InlineBoundaryContextAllocation>,
    pub inline_boxes: Option<PreparedInlineBoxFamily>,
    pub formatted_line: FormattedLineContext,
}
impl SemanticWindowRequest {
    /// Starts at this exact section's entry and retains its issued completion.
    pub fn new(segment: PreparedSourceSegment) -> Self {
        Self {
            start: segment.start(),
            completion: segment.end_opportunity(),
            segment,
            incoming: None,
            atomics: Vec::new(),
            controls: Vec::new(),
            boundary_context: Vec::new(),
            inline_boxes: None,
            formatted_line: FormattedLineContext::default(),
        }
    }
    pub(crate) fn visit_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness as W;
        visit(W::buffer(&self.atomics, self.atomics.len())?)?;
        visit(W::buffer(&self.controls, self.controls.len())?)?;
        visit(W::buffer(
            &self.boundary_context,
            self.boundary_context.len(),
        )?)?;
        if let Some(family) = &self.inline_boxes {
            family.visit_allocations(visit)?;
        }
        Ok(())
    }
}

/// Ordinary spelling up to one actual mandatory ending, not a fitted line.
#[derive(Clone, Debug)]
pub struct SemanticPartitionEvidence {
    start: PreparedBoundary,
    terminal: PreparedOpportunity,
    content: bool,
}
impl SemanticPartitionEvidence {
    pub fn start(&self) -> PreparedBoundary {
        self.start.clone()
    }
    pub fn terminal(&self) -> PreparedOpportunity {
        self.terminal.clone()
    }
    /// Actual semantic in-flow content with eligible edge text removed.
    /// This is not a CSS line-existence or placement observation.
    pub const fn has_in_flow_content(&self) -> bool {
        self.content
    }
}

/// One preparation-issued alternative's authentic spelling/source provenance.
#[derive(Clone, Debug)]
pub struct SemanticConditionalEvidence {
    alternative: PreparedAlternative,
    partition: usize,
    before_content: bool,
    after_content: Option<bool>,
}
impl SemanticConditionalEvidence {
    pub fn alternative(&self) -> PreparedAlternative {
        self.alternative.clone()
    }
    pub const fn partition(&self) -> usize {
        self.partition
    }
    pub const fn before_has_in_flow_content(&self) -> bool {
        self.before_content
    }
    pub const fn after_has_in_flow_content(&self) -> Option<bool> {
        self.after_content
    }
}

#[derive(Clone, Debug)]
pub struct SemanticWindowEvidence {
    inner: Arc<Window>,
}
#[derive(Debug)]
struct Window {
    request: SemanticWindowRequest,
    partitions: Vec<SemanticPartitionEvidence>,
    conditionals: Vec<SemanticConditionalEvidence>,
    usage: Option<crate::ScopeAlignmentUsage>,
    retention: ScopeAlignmentRetention,
    input_retention: ScopeAlignmentRetention,
}

/// Failure of this structural transaction, retaining its completed resource
/// charges. No partial semantic evidence is published. Usage is absent when
/// source admission fails before operation funding or no limits were captured.
#[derive(Debug)]
pub struct SemanticWindowFailure {
    error: crate::Error,
    usage: Option<crate::ScopeAlignmentUsage>,
    input_retention: ScopeAlignmentRetention,
}
impl SemanticWindowFailure {
    pub fn error(&self) -> &crate::Error {
        &self.error
    }
    pub fn into_parts(
        self,
    ) -> (
        crate::Error,
        Option<crate::ScopeAlignmentUsage>,
        ScopeAlignmentRetention,
    ) {
        (self.error, self.usage, self.input_retention)
    }
    pub const fn input_retention(&self) -> ScopeAlignmentRetention {
        self.input_retention
    }
    pub const fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.usage
    }
}
impl std::fmt::Display for SemanticWindowFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for SemanticWindowFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
impl SemanticWindowEvidence {
    pub(crate) fn same_storage(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
    pub(crate) fn visit_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness as W;
        visit(W::object(self.inner.as_ref(), 0))?;
        self.inner.request.visit_allocations(visit)?;
        visit(W::buffer(
            &self.inner.partitions,
            self.inner.partitions.len(),
        )?)?;
        visit(W::buffer(
            &self.inner.conditionals,
            self.inner.conditionals.len(),
        )?)
    }
    pub fn request(&self) -> &SemanticWindowRequest {
        &self.inner.request
    }
    pub fn matches(&self, request: &SemanticWindowRequest) -> bool {
        self.inner.request == *request
    }
    pub fn partitions(&self) -> &[SemanticPartitionEvidence] {
        &self.inner.partitions
    }
    pub fn conditionals(&self) -> &[SemanticConditionalEvidence] {
        &self.inner.conditionals
    }
    pub fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.inner.usage
    }
    /// This proof's own live rows/bytes, excluding shared preparation/family
    /// storage. Use the scope census to admit an actual retained product union.
    pub fn scope_alignment_retention(&self) -> ScopeAlignmentRetention {
        self.inner.retention
    }
    /// Actual retained input admitted by this transaction, before new products.
    pub fn input_retention(&self) -> ScopeAlignmentRetention {
        self.inner.input_retention
    }
    /// Exact original occurrences, including source floats and empty anchors.
    pub fn participants(&self) -> impl Iterator<Item = PreparedParticipant> + '_ {
        let prepared = &self.inner.request.segment.prepared;
        let start = prepared.tables.boundaries[self.inner.request.start.index].participant;
        let end = prepared.tables.boundaries[self.inner.request.segment.end().index].participant;
        prepared.tables.participants[start..end]
            .iter()
            .map(move |value| super::readback::participant(prepared, *value))
    }
}

fn invalid() -> crate::Error {
    protocol_error(ProtocolError::InvalidSemanticWindow)
}

impl PreparedText {
    /// Observe a complete semantic window without fonts, shaping, line fitting,
    /// metrics, float bands or scope finalization. Success is conditional on the
    /// eventual real operation succeeding; it supplies no placement authority.
    pub fn semantic_window(
        &self,
        request: &SemanticWindowRequest,
    ) -> std::result::Result<SemanticWindowEvidence, SemanticWindowFailure> {
        self.semantic_window_with_allowance(request, None)
    }
    /// Observe under a reduced actual structural-resource allowance.
    pub fn semantic_window_with_scope_allowance(
        &self,
        request: &SemanticWindowRequest,
        allowance: ScopeAlignmentAllowance,
    ) -> std::result::Result<SemanticWindowEvidence, SemanticWindowFailure> {
        self.semantic_window_with_allowance(request, Some(allowance))
    }
    fn semantic_window_with_allowance(
        &self,
        request: &SemanticWindowRequest,
        allowance: Option<ScopeAlignmentAllowance>,
    ) -> std::result::Result<SemanticWindowEvidence, SemanticWindowFailure> {
        let mut ledger = None;
        let mut input_retention = ScopeAlignmentRetention::default();
        self.produce_semantic_window(request, allowance, &mut ledger, &mut input_retention)
            .map_err(|error| SemanticWindowFailure {
                error,
                usage: ledger
                    .as_ref()
                    .map(|budget| budget.usage_after_discard(input_retention.payload_bytes())),
                input_retention,
            })
    }
    fn produce_semantic_window(
        &self,
        request: &SemanticWindowRequest,
        allowance: Option<ScopeAlignmentAllowance>,
        ledger: &mut Ledger,
        input_retention: &mut ScopeAlignmentRetention,
    ) -> Result<SemanticWindowEvidence> {
        let section = super::segments::admit(self, &request.segment)?;
        if request.completion != request.segment.end_opportunity() {
            return Err(invalid());
        }
        let end = request.segment.end();
        let (start, finish) = super::selection::admit_window(
            self,
            &request.segment,
            &request.start,
            &end,
            request.incoming.as_ref(),
            request.inline_boxes.as_ref(),
            None,
        )?;
        crate::engine::scope_alignment::census::semantic_window_budget(
            &self.inner,
            request,
            allowance,
            ledger,
            input_retention,
        )?;
        validate_allocations(self, request, start.participant..finish.participant, ledger)?;
        super::inline_box::validate_sources(
            &self.inner,
            &request.controls,
            &request.boundary_context,
        )?;
        if let Some(family) = &request.inline_boxes {
            family
                .inner
                .require_contexts(&request.controls, &request.boundary_context, ledger)?;
        } else {
            super::inline_box::validate_request(
                &self.inner,
                &request.controls,
                &request.boundary_context,
                ledger,
            )?;
        }
        let mut alternatives = Vec::new();
        for i in section.opportunities.clone() {
            account::work(ledger, 1)?;
            for &alternative in &self.inner.tables.opportunities[i].alternatives {
                if self.inner.tables.alternatives[alternative].source_segment
                    != Some(request.segment.index)
                {
                    return Err(invalid());
                }
                push(ledger, &mut alternatives, alternative)?;
            }
        }
        for &alternative in &self.inner.tables.latent_alternatives {
            account::work(ledger, 1)?;
            if self.inner.tables.alternatives[alternative].source_segment
                == Some(request.segment.index)
            {
                push(ledger, &mut alternatives, alternative)?;
            }
        }
        let mut partitions = Vec::new();
        let mut conditionals = Vec::new();
        let mut from = request.start.clone();
        let mut incoming = request.incoming.as_ref();
        for index in section.opportunities.clone() {
            account::work(ledger, 1)?;
            let row = &self.inner.tables.opportunities[index];
            if !matches!(
                row.kind,
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            ) {
                continue;
            }
            let boundary = PreparedBoundary {
                prepared: Arc::clone(&self.inner),
                index: row.boundary,
            };
            if row.kind == PreparedBreakKind::Forced && row.boundary <= request.start.index {
                continue;
            }
            let terminal = PreparedOpportunity {
                prepared: Arc::clone(&self.inner),
                index,
            };
            // A mandatory occurrence never inherits a discretionary alternative.
            if !row.alternatives.is_empty() {
                return Err(invalid());
            }
            let content = observe(self, request, &from, &boundary, incoming, None, ledger)?;
            let partition = partitions.len();
            for &i in &alternatives {
                let alternative = &self.inner.tables.alternatives[i];
                account::work(ledger, 1)?;
                let cut = PreparedBoundary {
                    prepared: Arc::clone(&self.inner),
                    index: alternative.boundary,
                };
                if cut.index <= from.index || cut.index > boundary.index {
                    continue;
                }
                let alternative = PreparedAlternative {
                    prepared: Arc::clone(&self.inner),
                    index: i,
                };
                let before_content = observe(
                    self,
                    request,
                    &from,
                    &cut,
                    incoming,
                    Some(&alternative),
                    ledger,
                )?;
                let after_content = HyphenationContinuation::from_alternative(alternative.clone())
                    .map(|after| {
                        observe(self, request, &cut, &boundary, Some(&after), None, ledger)
                    })
                    .transpose()?;
                push(
                    ledger,
                    &mut conditionals,
                    SemanticConditionalEvidence {
                        alternative,
                        partition,
                        before_content,
                        after_content,
                    },
                )?;
            }
            push(
                ledger,
                &mut partitions,
                SemanticPartitionEvidence {
                    start: from,
                    terminal,
                    content,
                },
            )?;
            from = boundary;
            incoming = None;
        }
        if partitions
            .last()
            .is_none_or(|p| p.terminal != request.completion)
        {
            return Err(invalid());
        }
        account::drop_vec(ledger, alternatives)?;
        account::header::<Window>(ledger)?;
        let request = clone_request(request, ledger)?;
        let retention = retention(&request, &partitions, &conditionals)?;
        let usage = ledger
            .as_ref()
            .map(crate::processing::contextual::alignment::ScopeAlignmentBudget::usage);
        Ok(SemanticWindowEvidence {
            inner: Arc::new(Window {
                request,
                partitions,
                conditionals,
                usage,
                retention,
                input_retention: *input_retention,
            }),
        })
    }
}

fn retention(
    request: &SemanticWindowRequest,
    partitions: &Vec<SemanticPartitionEvidence>,
    conditionals: &Vec<SemanticConditionalEvidence>,
) -> Result<ScopeAlignmentRetention> {
    let records = [
        request.atomics.len(),
        request.controls.len(),
        request.boundary_context.len(),
        partitions.len(),
        conditionals.len(),
    ]
    .into_iter()
    .try_fold(0usize, |sum, count| {
        sum.checked_add(count)
            .ok_or_else(|| count_overflow(crate::ScopeAlignmentLimitKind::Records))
    })?;
    let payload_bytes = [
        account::bytes(&request.atomics)?,
        account::bytes(&request.controls)?,
        account::bytes(&request.boundary_context)?,
        account::bytes(partitions)?,
        account::bytes(conditionals)?,
    ]
    .into_iter()
    .try_fold(std::mem::size_of::<Window>(), |sum, count| {
        sum.checked_add(count)
            .ok_or_else(|| count_overflow(crate::ScopeAlignmentLimitKind::PayloadBytes))
    })?;
    Ok(ScopeAlignmentRetention {
        records,
        payload_bytes,
    })
}
fn structural_overflow() -> crate::Error {
    count_overflow(crate::ScopeAlignmentLimitKind::StructuralWork)
}
fn count_overflow(kind: crate::ScopeAlignmentLimitKind) -> crate::Error {
    crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow { kind })
}

fn work(ledger: &mut Ledger, counts: &[usize]) -> Result<()> {
    let count = counts.iter().try_fold(0usize, |sum, count| {
        sum.checked_add(*count).ok_or_else(structural_overflow)
    })?;
    account::work(ledger, count)
}

fn push<T>(ledger: &mut Ledger, values: &mut Vec<T>, value: T) -> Result<()> {
    if ledger.is_some() {
        return account::push(ledger, values, value);
    }
    values.try_reserve(1).map_err(|_| {
        crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Allocation {
            kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
        })
    })?;
    values.push(value);
    Ok(())
}
fn collect<T>(ledger: &mut Ledger, values: impl IntoIterator<Item = T>) -> Result<Vec<T>> {
    let mut result = Vec::new();
    for value in values {
        push(ledger, &mut result, value)?;
    }
    Ok(result)
}

fn clone_request(
    request: &SemanticWindowRequest,
    ledger: &mut Ledger,
) -> Result<SemanticWindowRequest> {
    Ok(SemanticWindowRequest {
        segment: request.segment.clone(),
        start: request.start.clone(),
        incoming: request.incoming.clone(),
        completion: request.completion.clone(),
        atomics: collect(ledger, request.atomics.iter().cloned())?,
        controls: collect(ledger, request.controls.iter().cloned())?,
        boundary_context: collect(ledger, request.boundary_context.iter().cloned())?,
        inline_boxes: request.inline_boxes.clone(),
        formatted_line: request.formatted_line,
    })
}

fn validate_allocations(
    prepared: &PreparedText,
    request: &SemanticWindowRequest,
    range: std::ops::Range<usize>,
    ledger: &mut Ledger,
) -> Result<()> {
    let tables = &prepared.inner.tables;
    for (i, allocation) in request.boundary_context.iter().enumerate() {
        work(ledger, &[1, i, request.controls.len()])?;
        super::selection::require_prepared(
            prepared,
            &allocation.occurrence.prepared,
            ProtocolHandleKind::Control,
            allocation.occurrence.index,
        )?;
        if request
            .controls
            .iter()
            .any(|row| row.occurrence == allocation.occurrence)
            || request.boundary_context[..i]
                .iter()
                .any(|row| row.occurrence == allocation.occurrence)
        {
            return Err(protocol_error(ProtocolError::DuplicateBoundaryContext {
                occurrence: allocation.occurrence.index,
            }));
        }
        let occurrence = tables.controls[allocation.occurrence.index].occurrence;
        if !super::boundary::is_endpoint(
            &prepared.inner.analyzed.paragraph.controls[occurrence].kind,
        ) {
            return Err(protocol_error(ProtocolError::UnexpectedBoundaryContext {
                occurrence: allocation.occurrence.index,
            }));
        }
    }
    for (i, allocation) in request.atomics.iter().enumerate() {
        work(ledger, &[1, i, range.len()])?;
        super::selection::require_prepared(
            prepared,
            &allocation.occurrence.prepared,
            ProtocolHandleKind::Atomic,
            allocation.occurrence.index,
        )?;
        if !tables.participants[range.clone()]
            .contains(&ParticipantData::Atomic(allocation.occurrence.index))
        {
            return Err(protocol_error(ProtocolError::UnexpectedAtomic {
                occurrence: allocation.occurrence.index,
            }));
        }
        if request.atomics[..i]
            .iter()
            .any(|prior| prior.occurrence == allocation.occurrence)
        {
            return Err(protocol_error(ProtocolError::DuplicateAtomic {
                occurrence: allocation.occurrence.index,
            }));
        }
        let occurrence = tables.atomics[allocation.occurrence.index].occurrence;
        if prepared.inner.analyzed.processed.source().boxes()[occurrence].kind() != allocation.kind
        {
            return Err(protocol_error(ProtocolError::InvalidSemanticAtomic {
                occurrence: allocation.occurrence.index,
            }));
        }
    }
    for (i, allocation) in request.controls.iter().enumerate() {
        work(ledger, &[1, i, range.len()])?;
        super::selection::require_prepared(
            prepared,
            &allocation.occurrence.prepared,
            ProtocolHandleKind::Control,
            allocation.occurrence.index,
        )?;
        if !tables.participants[range.clone()]
            .contains(&ParticipantData::Control(allocation.occurrence.index))
        {
            return Err(protocol_error(ProtocolError::UnexpectedControl {
                occurrence: allocation.occurrence.index,
            }));
        }
        if request.controls[..i]
            .iter()
            .any(|prior| prior.occurrence == allocation.occurrence)
        {
            return Err(protocol_error(ProtocolError::DuplicateControl {
                occurrence: allocation.occurrence.index,
            }));
        }
        let occurrence = tables.controls[allocation.occurrence.index].occurrence;
        if super::boundary::is_endpoint(
            &prepared.inner.analyzed.paragraph.controls[occurrence].kind,
        ) != allocation.boundary.is_some()
        {
            return Err(protocol_error(
                ProtocolError::MissingControlBoundaryGeometry {
                    occurrence: allocation.occurrence.index,
                },
            ));
        }
    }
    for participant in &tables.participants[range] {
        work(ledger, &[1, request.atomics.len(), request.controls.len()])?;
        match *participant {
            ParticipantData::Atomic(index)
                if !request.atomics.iter().any(|a| a.occurrence.index == index) =>
            {
                return Err(protocol_error(ProtocolError::MissingAtomic {
                    occurrence: index,
                }));
            }
            ParticipantData::Control(index)
                if !request.controls.iter().any(|a| a.occurrence.index == index) =>
            {
                return Err(protocol_error(ProtocolError::MissingControl {
                    occurrence: index,
                }));
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Row {
    edge: crate::SuppliedInlineParticipant,
    content: super::semantic_content::Content,
}

fn observe(
    prepared: &PreparedText,
    request: &SemanticWindowRequest,
    start: &PreparedBoundary,
    end: &PreparedBoundary,
    incoming: Option<&HyphenationContinuation>,
    alternative: Option<&PreparedAlternative>,
    ledger: &mut Ledger,
) -> Result<bool> {
    use super::semantic_content::Content;
    let data = &prepared.inner;
    let start = data.tables.boundaries[start.index].participant;
    let end = data.tables.boundaries[end.index].participant;
    let mut rows = Vec::new();
    let mut anchor_content = false;
    let mut incoming = incoming;
    for emission in prepared::semantic::Overlay::new(data, start..end, alternative.map(|a| a.index))
    {
        account::work(ledger, 1)?;
        let unit = match emission? {
            prepared::semantic::Emission::Indication(index) => {
                emit_indication(data, index, &mut rows, ledger)?;
                continue;
            }
            prepared::semantic::Emission::Original(unit) => unit,
        };
        let kind = unit.capture.kind;
        if prepared::semantic::receives_incoming(
            data,
            kind,
            !unit.atoms.is_empty(),
            unit.capture.bidi_class.is_some(),
        ) && let Some(after) = incoming.take()
        {
            emit_indication(data, after.alternative.index, &mut rows, ledger)?;
        }
        let level = crate::BidiLevel::try_new(unit.capture.level).map_err(|_| invalid())?;
        match kind {
            PresentationUnitKind::Cell { .. } => push(
                ledger,
                &mut rows,
                Row {
                    edge: crate::SuppliedInlineParticipant::Cell { level },
                    content: Content::Cell,
                },
            )?,
            PresentationUnitKind::Atomic { occurrence } => {
                let in_flow = data.analyzed.processed.source().boxes()[occurrence].kind()
                    == InlineBoxKind::InFlow;
                if in_flow {
                    push(
                        ledger,
                        &mut rows,
                        Row {
                            edge: crate::SuppliedInlineParticipant::Atomic { level },
                            content: Content::Atomic { in_flow },
                        },
                    )?;
                }
            }
            PresentationUnitKind::Control { occurrence } => {
                account::work(ledger, request.controls.len())?;
                anchor_content |= Content::Boundary {
                    participates: request
                        .controls
                        .iter()
                        .find(|a| data.tables.controls[a.occurrence.index].occurrence == occurrence)
                        .and_then(|a| a.boundary.as_ref())
                        .is_some_and(InlineBoundaryAllocation::participates),
                }
                .in_flow();
            }
            _ => {
                for atom in unit.atoms {
                    emit_atom(
                        data,
                        unit.projection.atoms[atom].code_point,
                        unit.analysis.infos[atom].1,
                        level,
                        &mut rows,
                        ledger,
                    )?;
                }
            }
        }
    }
    if incoming.is_some() {
        return Err(crate::hyphenation::error(
            crate::HyphenationError::MissingReceivingBidiAuthority,
        ));
    }
    let projected = collect(ledger, rows.iter().map(|row| row.edge))?;
    work(ledger, &[projected.len(), projected.len()])?;
    let edges = crate::SuppliedLine::new(&projected).edges();
    let mut result = anchor_content;
    for (i, row) in rows.iter().enumerate() {
        account::work(ledger, 1)?;
        let content = match row.content {
            Content::Text { hard_separator, .. } => Content::Text {
                retained: edges.is_discarded(i) != Some(true),
                hard_separator,
            },
            other => other,
        };
        result |= content.in_flow();
    }
    account::drop_vec(ledger, rows)?;
    account::drop_vec(ledger, projected)?;
    Ok(result)
}

fn emit_indication(
    prepared: &crate::engine::PreparedData,
    index: usize,
    rows: &mut Vec<Row>,
    ledger: &mut Ledger,
) -> Result<()> {
    let semantic = &prepared.indications[index].semantic;
    for atom in semantic.inserted_atoms.clone() {
        let level =
            crate::BidiLevel::try_new(semantic.analysis.levels[atom]).map_err(|_| invalid())?;
        emit_atom(
            prepared,
            semantic.projection.atoms[atom].code_point,
            semantic.analysis.infos[atom].1,
            level,
            rows,
            ledger,
        )?;
    }
    Ok(())
}
fn emit_atom(
    prepared: &crate::engine::PreparedData,
    code_point: u32,
    style: usize,
    level: crate::BidiLevel,
    rows: &mut Vec<Row>,
    ledger: &mut Ledger,
) -> Result<()> {
    account::work(ledger, 1)?;
    // Native hard separators are delimiters, not edge-ordering slots.
    if crate::engine::analysis::is_hard_separator(code_point) {
        return Ok(());
    }
    let edge = crate::engine::analysis::edge::whitespace_edge(
        code_point == 0x20,
        code_point == 0x1680,
        prepared.analyzed.styles[style].style.authored().white_space,
        true,
    );
    push(
        ledger,
        rows,
        Row {
            edge: crate::SuppliedInlineParticipant::Text { level, edge },
            content: super::semantic_content::Content::Text {
                retained: true,
                hard_separator: false,
            },
        },
    )
}
