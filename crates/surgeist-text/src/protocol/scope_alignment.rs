//! Preparation-bound font context and exact selected scope allocation.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(crate) use super::conditional::{ScopeProofVisit, visit_request_scope_frames};

use super::{LineBoxAlignment, PreparedText, SelectedLine, SpacingAllocations};
use crate::{
    BaselineShift, CapturedScope, EffectiveFontInstance, FontBaseline, FontBaselineOutcome, Result,
    ScopeFontReference, UprightMetricProvenance, UprightMetricReference,
};

/// Retained same-instance metric context; querying this view never resolves a font.
#[derive(Clone, Copy)]
pub struct PreparedScopeMetrics<'a> {
    pub(crate) prepared: &'a crate::engine::PreparedData,
    pub(crate) index: usize,
}
impl fmt::Debug for PreparedScopeMetrics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedScopeMetrics")
            .field("scope", &self.captured_scope())
            .finish_non_exhaustive()
    }
}
impl PreparedScopeMetrics<'_> {
    fn data(&self) -> &crate::engine::scope_alignment::ScopeMetricsData {
        &self.prepared.scope_alignment.metrics[self.index]
    }
    pub fn captured_scope(&self) -> CapturedScope {
        self.data().scope.clone()
    }
    pub fn font_reference(&self) -> ScopeFontReference {
        self.data().reference
    }
    pub fn common_baseline_reference(&self) -> Option<FontBaseline> {
        self.prepared.options.resolved_baseline_reference()
    }
    pub fn orientation(&self) -> crate::LineOrientation {
        self.prepared.options.orientation()
    }
    pub fn effective_instance(&self) -> Result<EffectiveFontInstance<'_>> {
        let data = self.data();
        EffectiveFontInstance::qualify_synthesized(
            data.font.face(),
            data.font_size,
            &data.coords,
            data.synthesis,
        )
    }
    pub fn font_size(&self) -> f32 {
        self.data().font_size
    }
    pub fn line_height(&self) -> f64 {
        self.data().line_height
    }
    pub fn over(&self) -> f64 {
        self.data().over
    }
    pub fn under(&self) -> f64 {
        self.data().under
    }
    pub fn native_reference_coordinate(&self) -> f64 {
        self.data().native_coordinate
    }
    /// A demanded named coordinate relative to the retained native reference,
    /// positive toward line-over. None means the name was not demanded.
    pub fn matching_coordinate(&self, name: FontBaseline) -> Option<f64> {
        self.data().matching[crate::engine::scope_alignment::baseline_index(name)]
    }
    pub fn baseline_outcome(&self, name: FontBaseline) -> Option<FontBaselineOutcome> {
        self.data().baselines[crate::engine::scope_alignment::baseline_index(name)]
    }
    pub fn upright_reference(&self) -> Option<UprightMetricReference> {
        self.data().upright_reference
    }
    pub fn upright_provenance(&self) -> Option<UprightMetricProvenance> {
        self.data().upright_provenance
    }
}

impl PreparedText {
    pub fn scope_alignment_retention(&self) -> Option<ScopeAlignmentRetention> {
        self.inner.scope_alignment.retention
    }
    pub fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.inner.scope_alignment.usage
    }
    pub fn scope_metrics(&self, scope: &CapturedScope) -> Result<PreparedScopeMetrics<'_>> {
        let index = crate::engine::scope_alignment::metric_index(&self.inner, scope, None)?;
        Ok(PreparedScopeMetrics {
            prepared: &self.inner,
            index,
        })
    }
}

/// An actual captured scope fragment in this exact selected presentation.
#[derive(Clone)]
pub struct SelectedScope {
    pub(crate) selected: Arc<super::selection::SelectedData>,
    pub(crate) index: usize,
}
impl PartialEq for SelectedScope {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.selected, &other.selected)
    }
}
impl Eq for SelectedScope {}
impl Hash for SelectedScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.selected).hash(state);
        self.index.hash(state);
    }
}
impl fmt::Debug for SelectedScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedScope")
            .field("scope", &self.captured_scope())
            .finish_non_exhaustive()
    }
}
impl SelectedScope {
    pub fn selected_line(&self) -> SelectedLine {
        SelectedLine {
            inner: Arc::clone(&self.selected),
        }
    }
    pub fn captured_scope(&self) -> CapturedScope {
        self.selected.scopes.rows[self.index].scope.clone()
    }
    pub fn metrics(&self) -> PreparedScopeMetrics<'_> {
        PreparedScopeMetrics {
            prepared: &self.selected.prepared,
            index: self.selected.scopes.rows[self.index].metrics,
        }
    }
    pub fn parent_metrics(&self) -> Option<PreparedScopeMetrics<'_>> {
        self.selected.scopes.rows[self.index]
            .parent_metrics
            .map(|index| PreparedScopeMetrics {
                prepared: &self.selected.prepared,
                index,
            })
    }
    /// Actual local contributions are available before allocation/finalization.
    /// Ancestor and descendant views can observe the same contribution.
    pub fn metric_contributions(&self) -> impl Iterator<Item = ScopeMetricContribution> + '_ {
        self.selected
            .scopes
            .contributions
            .iter()
            .filter(|row| self.selected.scopes.contains(self.index, row.owner))
            .map(|row| row.metrics)
    }
    pub fn direct_metric_contributions(
        &self,
    ) -> impl Iterator<Item = ScopeMetricContribution> + '_ {
        self.selected
            .scopes
            .contribution_chain(self.selected.scopes.rows[self.index].direct.first)
    }
    /// Actual shared custody; ancestor sharing does not imply a frame conflict.
    pub fn shares_operands_with(&self, other: &Self) -> Result<bool> {
        crate::engine::scope_alignment::shares_operands(self, other)
    }
}
impl SelectedLine {
    pub fn scope_alignment_retention(&self) -> Option<ScopeAlignmentRetention> {
        self.inner.scopes.retention
    }
    pub fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.inner.scopes.usage
    }
    pub fn scopes(&self) -> impl ExactSizeIterator<Item = SelectedScope> + '_ {
        (0..self.inner.scopes.rows.len()).map(|index| SelectedScope {
            selected: Arc::clone(&self.inner),
            index,
        })
    }
    /// Complete common-line contributions in the active scope-alignment inventory.
    /// With no feature retention this iterator is empty, not proof that the
    /// ordinary line has no metrics; its existing finalizer remains authoritative.
    pub fn unscoped_metric_contributions(
        &self,
    ) -> impl Iterator<Item = ScopeMetricContribution> + '_ {
        self.inner
            .scopes
            .contribution_chain(self.inner.scopes.unscoped.first)
    }
}

/// One genuine signed local metric contribution, before scope displacement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScopeMetricContribution {
    pub before: f64,
    pub after: f64,
    pub alignment: LineBoxAlignment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeAllocation {
    pub scope: SelectedScope,
    /// Own positive-over contribution, excluding all ancestor contributions.
    pub displacement: BaselineShift,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ScopeAllocations {
    #[default]
    None,
    Complete(Vec<ScopeAllocation>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct LineAllocations {
    pub spacing: SpacingAllocations,
    pub scopes: ScopeAllocations,
}

/// Caller-reduced feature allowance for one operation, including its actual
/// retained Text input. The immutable capture limits remain the upper bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeAlignmentAllowance {
    records: usize,
    payload_bytes: usize,
}
impl ScopeAlignmentAllowance {
    pub const fn new(records: usize, payload_bytes: usize) -> Self {
        Self {
            records,
            payload_bytes,
        }
    }
    pub const fn records(self) -> usize {
        self.records
    }
    pub const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }
}

/// Deterministic executed allocation facts, without selected-Arc identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ScopeFrameAllocation {
    pub scope: CapturedScope,
    pub displacement: BaselineShift,
    pub cumulative_displacement: BaselineShift,
}

/// Actual feature records and live payload retained by this immutable result.
/// Transaction materialization/work/query counts are separate usage observations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScopeAlignmentRetention {
    pub(crate) records: usize,
    pub(crate) payload_bytes: usize,
}
impl ScopeAlignmentRetention {
    pub const fn records(self) -> usize {
        self.records
    }
    pub const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }
}

impl super::FinalizedLine {
    pub fn scope_allocations(&self) -> &[ScopeFrameAllocation] {
        &self.inner.data.scope_allocations
    }
    pub fn scope_alignment_retention(&self) -> Option<ScopeAlignmentRetention> {
        self.inner.data.scope_retention
    }
    pub fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.inner.data.scope_usage
    }
    /// Applied inherited component of this actual canonical request row.
    pub fn line_box_strut_scope_displacement(&self, index: usize) -> Option<BaselineShift> {
        let strut = self.inner.selected.request.line_box_struts.get(index)?;
        let owner = self
            .inner
            .selected
            .scopes
            .strut_owners
            .get(index)
            .copied()
            .flatten();
        let value = if matches!(
            strut.alignment,
            LineBoxAlignment::Top | LineBoxAlignment::Bottom
        ) {
            0.0
        } else {
            owner.map_or(0.0, |owner| {
                self.inner.data.scope_allocations[owner]
                    .cumulative_displacement
                    .get()
            })
        };
        Some(BaselineShift::try_new(value).expect("qualified applied scope frame"))
    }
}
