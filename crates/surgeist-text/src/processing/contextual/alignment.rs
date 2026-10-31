//! Captured alignment facts and feature-local, off-side resource accounting.

pub(crate) mod operations;
#[cfg(test)]
mod operations_tests;

use std::mem::size_of;

use super::{ScopeCapture, SourceProcessingScope};
use crate::{FontBaseline, FontBaselineAxis, FontBaselineScript, Result};

/// Effective formatting classification at a genuine captured inline boundary.
///
/// This is supplied before shaping. It is not a displacement: an explicit zero
/// shift is NonInitial, while absent producer evidence is represented by None.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScopeAlignmentBoundary {
    /// The actual initial alignment formatting value.
    Initial,
    /// Non-initial formatting, including an explicit geometrically zero shift.
    NonInitial,
}

/// Actual scope-font query context, without caller-invented font coordinates.
///
/// The scope's captured Style remains its sole font/style input. A reference on
/// a domain or otherwise dormant scope does not create an alignment target or a
/// font demand. The actual demanded instance and measurements are prepared later.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeFontReference {
    /// Supplied canonical script of this scope's font-reference context.
    pub script: FontBaselineScript,
    /// Font-design axis before any physical line rotation.
    pub axis: FontBaselineAxis,
    /// Actual resolved dominant baseline, not caller-invented metric coordinates.
    pub baseline: FontBaseline,
}

/// Explicit feature-local admission limits, including meaningful zero limits.
///
/// Payload comprises owned feature headers, allocated buffer capacities and
/// unique pinned resource data, including live input/output/scratch coexistence.
/// Existing source/topology/style storage, fixed transaction bookkeeping,
/// failure diagnostics, provider internals, allocator metadata, process RSS and
/// unrelated caller-retained results are outside this payload contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeAlignmentLimits {
    records: usize,
    font_queries: usize,
    structural_work: usize,
    payload_bytes: usize,
}

impl ScopeAlignmentLimits {
    /// Bounds logical rows materialized within one transaction, actual font
    /// resolver invocations, documented structural work and live payload bytes.
    /// Rows remain charged when scratch is dropped; byte custody can be released.
    #[must_use]
    pub const fn new(
        records: usize,
        font_queries: usize,
        structural_work: usize,
        payload_bytes: usize,
    ) -> Self {
        Self {
            records,
            font_queries,
            structural_work,
            payload_bytes,
        }
    }

    #[must_use]
    pub const fn records(self) -> usize {
        self.records
    }

    #[must_use]
    pub const fn font_queries(self) -> usize {
        self.font_queries
    }

    #[must_use]
    pub const fn structural_work(self) -> usize {
        self.structural_work
    }

    #[must_use]
    pub const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }
}

/// The exact feature resource whose admission failed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScopeAlignmentLimitKind {
    Records,
    FontQueries,
    StructuralWork,
    PayloadBytes,
}

/// Typed scope-alignment admission, resource or occurrence-custody refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeAlignmentError {
    /// New source facts were supplied without early feature limits.
    LimitsRequired,
    /// An explicit resource bound cannot admit the actual requirement.
    Limit {
        kind: ScopeAlignmentLimitKind,
        required: usize,
        limit: usize,
    },
    /// Checked count/byte arithmetic overflowed, or a custody debit underflowed.
    Overflow { kind: ScopeAlignmentLimitKind },
    /// Fallible feature-payload reservation was refused.
    Allocation { kind: ScopeAlignmentLimitKind },
    /// The requested operation has no authentic contextual capture.
    MissingCapture,
    /// A supplied captured handle belongs to another immutable capture.
    ForeignCapture,
    /// A supplied scope does not belong to the required actual content domain.
    WrongDomain,
    /// An active scope or required inline ancestor lacks its early formatting
    /// classification; absent evidence cannot stand in for Initial alignment.
    MissingBoundary { scope: usize },
    /// An actual demanded scope has no supplied effective Style.
    MissingStyle { scope: usize },
    /// An actual demanded scope lacks its supplied script/axis/baseline facts.
    MissingFontReference { scope: usize },
    /// This preparation did not retain demanded metrics for the actual scope.
    MissingMetrics { scope: usize },
    /// The supplied reference cannot be used by the actual operation's frame.
    UnsupportedReference { scope: usize },
    /// Actual content custody cannot represent the requested independent frame.
    UnrepresentedOwnership { scope: usize },
    /// An independently demanded scope boundary divides an indivisible unit.
    UnpartitionableUnit { scope: usize },
    /// An allocated handle belongs to a different selected presentation.
    ForeignSelectedScope,
    /// One actual selected scope received more than one allocation.
    DuplicateAllocation { scope: usize },
    /// A demanded selected scope has no complete allocation.
    MissingAllocation { scope: usize },
    /// Jointly owned native output was assigned inconsistent composed frames.
    IncompatibleFrames { first: usize, second: usize },
    /// Late alignment disagrees with the captured early formatting class.
    BoundaryMismatch { scope: usize },
}

/// Executed feature-local accounting for one complete transaction.
///
/// Records include retained input rows and every newly materialized row, even
/// when scratch rows are later dropped. Font queries and structural work count
/// this transaction's operations. Payload bytes describe actual live custody;
/// peak includes simultaneous input/output/scratch and old/new buffer storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeAlignmentUsage {
    records: usize,
    font_queries: usize,
    structural_work: usize,
    payload_bytes: usize,
    peak_payload_bytes: usize,
}

impl ScopeAlignmentUsage {
    #[must_use]
    pub const fn records(self) -> usize {
        self.records
    }

    #[must_use]
    pub const fn font_queries(self) -> usize {
        self.font_queries
    }

    #[must_use]
    pub const fn structural_work(self) -> usize {
        self.structural_work
    }

    #[must_use]
    pub const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }

    #[must_use]
    pub const fn peak_payload_bytes(self) -> usize {
        self.peak_payload_bytes
    }
}

fn refusal(reason: ScopeAlignmentError) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::ScopeAlignmentRefused,
        "scope alignment resource admission refused",
    )
    .with_detail(crate::ErrorDetail::ScopeAlignment { reason })
}

fn overflow(kind: ScopeAlignmentLimitKind) -> crate::Error {
    refusal(ScopeAlignmentError::Overflow { kind })
}

fn check(kind: ScopeAlignmentLimitKind, required: usize, limit: usize) -> Result<()> {
    if required > limit {
        return Err(refusal(ScopeAlignmentError::Limit {
            kind,
            required,
            limit,
        }));
    }
    Ok(())
}

fn sum(kind: ScopeAlignmentLimitKind, current: usize, additional: usize) -> Result<usize> {
    current
        .checked_add(additional)
        .ok_or_else(|| overflow(kind))
}

fn buffer_bytes<T>(capacity: usize) -> Result<usize> {
    let bytes = capacity
        .checked_mul(size_of::<T>())
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
    if bytes > isize::MAX as usize {
        return Err(overflow(ScopeAlignmentLimitKind::PayloadBytes));
    }
    Ok(bytes)
}

/// One private transaction over immutable limits and actual retained input.
///
/// Owners seed their actual retained row/byte census, not a previous phase's
/// monotonic usage (which may include discarded scratch). They charge real
/// resolver calls before execution, documented event/edge/operand/metric work,
/// new rows, buffer capacities and unique pinned resources. Shared allocations
/// are counted once by their actual owning identity; this ledger does not guess
/// resource identity or measure opaque fontique/provider internals.
#[derive(Debug)]
pub(crate) struct ScopeAlignmentBudget {
    limits: ScopeAlignmentLimits,
    usage: ScopeAlignmentUsage,
}

impl ScopeAlignmentBudget {
    pub(crate) fn new(
        limits: ScopeAlignmentLimits,
        retained_records: usize,
        retained_payload_bytes: usize,
    ) -> Result<Self> {
        check(
            ScopeAlignmentLimitKind::Records,
            retained_records,
            limits.records,
        )?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            retained_payload_bytes,
            limits.payload_bytes,
        )?;
        Ok(Self {
            limits,
            usage: ScopeAlignmentUsage {
                records: retained_records,
                font_queries: 0,
                structural_work: 0,
                payload_bytes: retained_payload_bytes,
                peak_payload_bytes: retained_payload_bytes,
            },
        })
    }

    pub(crate) const fn usage(&self) -> ScopeAlignmentUsage {
        self.usage
    }

    pub(crate) fn add_records(&mut self, count: usize) -> Result<()> {
        let kind = ScopeAlignmentLimitKind::Records;
        let required = sum(kind, self.usage.records, count)?;
        check(kind, required, self.limits.records)?;
        self.usage.records = required;
        Ok(())
    }

    /// Charges one actual resolver invocation, not its opaque internal work.
    pub(crate) fn font_query(&mut self) -> Result<()> {
        let kind = ScopeAlignmentLimitKind::FontQueries;
        let required = sum(kind, self.usage.font_queries, 1)?;
        check(kind, required, self.limits.font_queries)?;
        self.usage.font_queries = required;
        Ok(())
    }

    pub(crate) fn work(&mut self, count: usize) -> Result<()> {
        let kind = ScopeAlignmentLimitKind::StructuralWork;
        let required = sum(kind, self.usage.structural_work, count)?;
        check(kind, required, self.limits.structural_work)?;
        self.usage.structural_work = required;
        Ok(())
    }

    pub(crate) fn retain_bytes(&mut self, count: usize) -> Result<()> {
        let kind = ScopeAlignmentLimitKind::PayloadBytes;
        let required = sum(kind, self.usage.payload_bytes, count)?;
        check(kind, required, self.limits.payload_bytes)?;
        self.usage.payload_bytes = required;
        self.usage.peak_payload_bytes = self.usage.peak_payload_bytes.max(required);
        Ok(())
    }

    /// Releases already dropped custody; an uncharged debit cannot wrap.
    pub(crate) fn release_bytes(&mut self, count: usize) -> Result<()> {
        let remaining = self
            .usage
            .payload_bytes
            .checked_sub(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        self.usage.payload_bytes = remaining;
        Ok(())
    }

    /// Reserves an off-side feature buffer, accounting actual capacities and
    /// explicit old/new coexistence. Existing buffer custody must be charged.
    /// One structural operation is charged per moved element. Failure leaves
    /// both the previous buffer and ledger unchanged.
    pub(crate) fn reserve<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<()> {
        let kind = ScopeAlignmentLimitKind::PayloadBytes;
        let required = sum(kind, values.len(), additional)?;
        if required <= values.capacity() {
            return Ok(());
        }
        let requested_bytes = buffer_bytes::<T>(required)?;
        let old_bytes = buffer_bytes::<T>(values.capacity())?;
        self.usage
            .payload_bytes
            .checked_sub(old_bytes)
            .ok_or_else(|| overflow(kind))?;
        let mut next = Self {
            limits: self.limits,
            usage: self.usage,
        };
        next.work(values.len())?;
        check(
            kind,
            sum(kind, next.usage.payload_bytes, requested_bytes)?,
            next.limits.payload_bytes,
        )?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(required)
            .map_err(|_| refusal(ScopeAlignmentError::Allocation { kind }))?;
        next.retain_bytes(buffer_bytes::<T>(replacement.capacity())?)?;
        replacement.append(values);
        std::mem::swap(values, &mut replacement);
        drop(replacement);
        next.release_bytes(old_bytes)?;
        *self = next;
        Ok(())
    }
}

/// Sparse facts refer into the sole capture's actual scope inventory. They do
/// not repeat parent/domain/style/events or infer contribution/font demand.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScopeAlignmentFact {
    pub(crate) scope: usize,
    pub(crate) alignment_boundary: Option<ScopeAlignmentBoundary>,
    pub(crate) font_reference: Option<ScopeFontReference>,
}

#[derive(Clone, Debug)]
pub(crate) struct ScopeAlignmentCapture {
    limits: ScopeAlignmentLimits,
    usage: ScopeAlignmentUsage,
    facts: Vec<ScopeAlignmentFact>,
}

impl ScopeAlignmentCapture {
    pub(super) fn capture(
        scopes: &[SourceProcessingScope],
        limits: Option<ScopeAlignmentLimits>,
    ) -> Result<Option<Self>> {
        let Some(limits) = limits else {
            if scopes.iter().any(has_facts) {
                return Err(refusal(ScopeAlignmentError::LimitsRequired));
            }
            return Ok(None);
        };
        // Charge the actual optional feature field, including its header, and
        // the sparse buffer's actual capacity. Existing source/topology storage
        // is not charged again. Even an empty explicit feature has this header.
        let mut budget = ScopeAlignmentBudget::new(limits, 0, size_of::<Option<Self>>())?;
        // Two bounded input-scope inspections: count, then materialize. Their
        // budget is admitted before each pass; no events/fonts are queried here.
        budget.work(scopes.len())?;
        let count = scopes.iter().filter(|scope| has_facts(scope)).count();
        budget.add_records(count)?;
        let mut facts = Vec::new();
        budget.reserve(&mut facts, count)?;
        budget.work(scopes.len())?;
        for (scope, input) in scopes.iter().enumerate() {
            let alignment_boundary = input.alignment_boundary();
            let font_reference = input.font_reference();
            if alignment_boundary.is_some() || font_reference.is_some() {
                facts.push(ScopeAlignmentFact {
                    scope,
                    alignment_boundary,
                    font_reference,
                });
            }
        }
        Ok(Some(Self {
            limits,
            usage: budget.usage(),
            facts,
        }))
    }

    pub(crate) fn fact(&self, scope: usize) -> Option<&ScopeAlignmentFact> {
        self.find_fact(scope, None)
            .expect("borrowed fact lookup has no fallible budget")
    }

    fn find_fact(
        &self,
        scope: usize,
        budget: Option<&mut ScopeAlignmentBudget>,
    ) -> Result<Option<&ScopeAlignmentFact>> {
        Ok(
            operations::binary_search_by(&self.facts, budget, |fact| fact.scope.cmp(&scope))?
                .ok()
                .map(|index| &self.facts[index]),
        )
    }

    pub(crate) const fn limits(&self) -> ScopeAlignmentLimits {
        self.limits
    }

    pub(crate) const fn usage(&self) -> ScopeAlignmentUsage {
        self.usage
    }
}

fn has_facts(scope: &SourceProcessingScope) -> bool {
    scope.alignment_boundary().is_some() || scope.font_reference().is_some()
}

impl ScopeCapture {
    /// Describes only this capture's actual feature-owned storage. Allocation
    /// witnesses are private census keys, never authority to associate scopes.
    /// The driver meters traversal and deduplicates shared retained owners.
    pub(crate) fn visit_scope_alignment_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness;
        if let Some(capture) = &self.scope_alignment {
            visit(AllocationWitness::object(&self.scope_alignment, 0))?;
            visit(AllocationWitness::buffer(
                &capture.facts,
                capture.facts.len(),
            )?)?;
        }
        Ok(())
    }

    pub(crate) fn scope_alignment_fact(&self, scope: usize) -> Option<&ScopeAlignmentFact> {
        self.scope_alignment.as_ref()?.fact(scope)
    }

    pub(crate) fn scope_alignment_fact_with_budget(
        &self,
        scope: usize,
        budget: &mut ScopeAlignmentBudget,
    ) -> Result<Option<&ScopeAlignmentFact>> {
        match &self.scope_alignment {
            Some(capture) => capture.find_fact(scope, Some(budget)),
            None => Ok(None),
        }
    }

    /// Starts a new transaction over this capture's actual retained feature
    /// rows/bytes, resetting operation counters rather than replaying old work.
    pub(crate) fn scope_alignment_budget(&self) -> Result<Option<ScopeAlignmentBudget>> {
        self.scope_alignment
            .as_ref()
            .map(|capture| {
                ScopeAlignmentBudget::new(
                    capture.limits,
                    capture.facts.len(),
                    capture.usage.payload_bytes,
                )
            })
            .transpose()
    }
}
