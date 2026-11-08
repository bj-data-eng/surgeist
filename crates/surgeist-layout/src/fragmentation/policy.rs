use super::*;
use crate::LayoutScalar;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(super) struct Force {
    pub(super) required: FragmentationBreakKinds,
    pub(super) side: Option<FragmentationForcedSide>,
    side_order: usize,
}

impl Force {
    fn empty() -> Self {
        Self {
            required: FragmentationBreakKinds::NONE,
            side: None,
            side_order: 0,
        }
    }
    pub(super) fn merge(&mut self, other: Self) {
        self.required = self.required.union(other.required);
        if other.side.is_some() && (self.side.is_none() || other.side_order >= self.side_order) {
            self.side = other.side;
            self.side_order = other.side_order;
        }
    }
    fn add(&mut self, control: FragmentationBreakControl, available: FragmentationBreakKinds) {
        use FragmentationBreak as B;
        let (kind, side) = match control.value {
            B::Page => (FragmentationBreakKinds::PAGE, None),
            B::Column => (FragmentationBreakKinds::COLUMN, None),
            B::Region => (FragmentationBreakKinds::REGION, None),
            B::Left => (
                FragmentationBreakKinds::PAGE,
                Some(FragmentationForcedSide::Left),
            ),
            B::Right => (
                FragmentationBreakKinds::PAGE,
                Some(FragmentationForcedSide::Right),
            ),
            B::Recto => (
                FragmentationBreakKinds::PAGE,
                Some(FragmentationForcedSide::Recto),
            ),
            B::Verso => (
                FragmentationBreakKinds::PAGE,
                Some(FragmentationForcedSide::Verso),
            ),
            _ => return,
        };
        if control.role == FragmentationRole::Absolute || !available.contains(kind) {
            return;
        }
        self.required = self.required.union(kind);
        if side.is_some() && (self.side.is_none() || control.source_order >= self.side_order) {
            self.side = side;
            self.side_order = control.source_order;
        }
    }
}

pub(super) fn force<S: LayoutScalar>(
    opportunity: &FragmentationOpportunityOf<S>,
    available: FragmentationBreakKinds,
) -> Option<Force> {
    if !matches!(
        opportunity.kind,
        FragmentationOpportunityKindOf::Sibling | FragmentationOpportunityKindOf::ModeSpecific
    ) {
        return None;
    }
    force_controls(&opportunity.before, &opportunity.after, available)
}

pub(super) fn force_controls(
    before: &[FragmentationBreakControl],
    after: &[FragmentationBreakControl],
    available: FragmentationBreakKinds,
) -> Option<Force> {
    let mut result = Force::empty();
    for &control in after.iter().chain(before) {
        result.add(control, available);
    }
    (!result.required.is_empty()).then_some(result)
}

fn effective(value: FragmentationBreak, available: FragmentationBreakKinds) -> FragmentationBreak {
    use FragmentationBreak as B;
    let kind = match value {
        B::AvoidPage | B::Page | B::Left | B::Right | B::Recto | B::Verso => {
            FragmentationBreakKinds::PAGE
        }
        B::AvoidColumn | B::Column => FragmentationBreakKinds::COLUMN,
        B::AvoidRegion | B::Region => FragmentationBreakKinds::REGION,
        _ => return value,
    };
    if available.contains(kind) {
        value
    } else {
        B::Auto
    }
}

fn forbidden(value: FragmentationBreakInside, crossed: FragmentationBreakKinds) -> bool {
    match value {
        FragmentationBreakInside::Auto => false,
        FragmentationBreakInside::Avoid => !crossed.is_empty(),
        FragmentationBreakInside::AvoidPage => crossed.contains(FragmentationBreakKinds::PAGE),
        FragmentationBreakInside::AvoidColumn => crossed.contains(FragmentationBreakKinds::COLUMN),
        FragmentationBreakInside::AvoidRegion => crossed.contains(FragmentationBreakKinds::REGION),
    }
}

pub(super) fn legal<S: LayoutScalar>(
    opportunity: &FragmentationOpportunityOf<S>,
    crossed: FragmentationBreakKinds,
    available: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    legal_view(
        opportunity.kind,
        LegalityFacts::capture(opportunity, available),
        crossed,
        stage,
    )
}

/// Facts owned by the opportunity actually selected by this scheduler. A
/// graphical projection changes its geometry, not its break-policy class.
pub(super) struct OpportunityEvidenceOf<S: LayoutScalar> {
    kind: FragmentationOpportunityKindOf<S>,
    controls: Vec<FragmentationBreakControl>,
    ancestors: Vec<FragmentationBreakInside>,
}

/// Applicability is fixed by the equal context inventories checked at scope
/// registration. These idempotent facts preserve cumulative Rule2 evidence
/// without retaining another ancestor/control vector at every embedding.
#[derive(Clone, Copy, Default)]
struct AvoidanceFacts([bool; 4]);
impl AvoidanceFacts {
    fn add(&mut self, value: FragmentationBreakInside) {
        let index = match value {
            FragmentationBreakInside::Auto => return,
            FragmentationBreakInside::Avoid => 0,
            FragmentationBreakInside::AvoidPage => 1,
            FragmentationBreakInside::AvoidColumn => 2,
            FragmentationBreakInside::AvoidRegion => 3,
        };
        self.0[index] = true;
    }
    fn merge(&mut self, other: Self) {
        for (value, additional) in self.0.iter_mut().zip(other.0) {
            *value |= additional;
        }
    }
    fn forbids(self, crossed: FragmentationBreakKinds) -> bool {
        self.0
            .into_iter()
            .zip([
                FragmentationBreakInside::Avoid,
                FragmentationBreakInside::AvoidPage,
                FragmentationBreakInside::AvoidColumn,
                FragmentationBreakInside::AvoidRegion,
            ])
            .any(|(present, value)| present && forbidden(value, crossed))
    }
}

#[derive(Clone, Copy, Default)]
struct LegalityFacts {
    non_auto: bool,
    controls: AvoidanceFacts,
    ancestors: AvoidanceFacts,
}
impl LegalityFacts {
    fn capture<S: LayoutScalar>(
        opportunity: &FragmentationOpportunityOf<S>,
        available: FragmentationBreakKinds,
    ) -> Self {
        Self::collect(
            opportunity.after.iter().chain(&opportunity.before),
            opportunity.ancestors.iter(),
            available,
        )
    }
    fn collect<'a>(
        controls: impl Iterator<Item = &'a FragmentationBreakControl>,
        ancestors: impl Iterator<Item = &'a FragmentationBreakInside>,
        available: FragmentationBreakKinds,
    ) -> Self {
        let mut facts = Self::default();
        for control in controls {
            if control.role == FragmentationRole::Absolute {
                continue;
            }
            facts.non_auto |= effective(control.value, available) != FragmentationBreak::Auto;
            facts.controls.add(match control.value {
                FragmentationBreak::Avoid => FragmentationBreakInside::Avoid,
                FragmentationBreak::AvoidPage => FragmentationBreakInside::AvoidPage,
                FragmentationBreak::AvoidColumn => FragmentationBreakInside::AvoidColumn,
                FragmentationBreak::AvoidRegion => FragmentationBreakInside::AvoidRegion,
                _ => FragmentationBreakInside::Auto,
            });
        }
        for &value in ancestors {
            facts.ancestors.add(value);
        }
        facts
    }
    fn merge(&mut self, other: Self) {
        self.non_auto |= other.non_auto;
        self.controls.merge(other.controls);
        self.ancestors.merge(other.ancestors);
    }
    fn interior(self) -> Self {
        Self {
            ancestors: self.ancestors,
            ..Self::default()
        }
    }
}

#[derive(Clone)]
pub(super) struct SelectedPolicyOf<S: LayoutScalar> {
    pub(super) origin: Arc<OpportunityEvidenceOf<S>>,
    facts: LegalityFacts,
}
impl<S: LayoutScalar> SelectedPolicyOf<S> {
    pub(super) fn capture(
        opportunity: &FragmentationOpportunityOf<S>,
        available: FragmentationBreakKinds,
    ) -> Result<Self, std::collections::TryReserveError> {
        let origin = Arc::new(OpportunityEvidenceOf::capture(opportunity)?);
        let facts =
            LegalityFacts::collect(origin.controls.iter(), origin.ancestors.iter(), available);
        Ok(Self { origin, facts })
    }
    pub(super) fn project(
        &self,
        parent: &FragmentationOpportunityOf<S>,
        available: FragmentationBreakKinds,
    ) -> Self {
        let mut facts = self.facts;
        facts.merge(LegalityFacts::capture(parent, available));
        Self {
            origin: Arc::clone(&self.origin),
            facts,
        }
    }
    pub(super) fn project_accepted(&self, parent: &Self) -> Self {
        let mut facts = self.facts;
        facts.merge(parent.facts);
        Self {
            origin: Arc::clone(&self.origin),
            facts,
        }
    }
    pub(super) fn project_interior(
        &self,
        parent: &FragmentationOpportunityOf<S>,
        available: FragmentationBreakKinds,
    ) -> Self {
        let mut facts = self.facts;
        facts.merge(LegalityFacts::capture(parent, available).interior());
        Self {
            origin: Arc::clone(&self.origin),
            facts,
        }
    }
    pub(super) fn project_accepted_interior(&self, parent: &Self) -> Self {
        let mut facts = self.facts;
        facts.merge(parent.facts.interior());
        Self {
            origin: Arc::clone(&self.origin),
            facts,
        }
    }
}

impl<S: LayoutScalar> OpportunityEvidenceOf<S> {
    pub(super) fn capture(
        opportunity: &FragmentationOpportunityOf<S>,
    ) -> Result<Self, std::collections::TryReserveError> {
        let mut controls = Vec::new();
        controls.try_reserve(
            opportunity
                .after
                .len()
                .saturating_add(opportunity.before.len()),
        )?;
        controls.extend(opportunity.after.iter().chain(&opportunity.before).copied());
        let mut ancestors = Vec::new();
        ancestors.try_reserve(opportunity.ancestors.len())?;
        ancestors.extend_from_slice(&opportunity.ancestors);
        Ok(Self {
            kind: opportunity.kind,
            controls,
            ancestors,
        })
    }
}

pub(super) fn legal_projected<S: LayoutScalar>(
    selected: &SelectedPolicyOf<S>,
    parent: &FragmentationOpportunityOf<S>,
    crossed: FragmentationBreakKinds,
    available: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    let mut facts = selected.facts;
    facts.merge(LegalityFacts::capture(parent, available));
    legal_view(selected.origin.kind, facts, crossed, stage)
}

pub(super) fn legal_projected_accepted<S: LayoutScalar>(
    selected: &SelectedPolicyOf<S>,
    parent: &SelectedPolicyOf<S>,
    crossed: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    let mut facts = selected.facts;
    facts.merge(parent.facts);
    legal_view(selected.origin.kind, facts, crossed, stage)
}

pub(super) fn legal_projected_interior<S: LayoutScalar>(
    selected: &SelectedPolicyOf<S>,
    parent: &FragmentationOpportunityOf<S>,
    crossed: FragmentationBreakKinds,
    available: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    let mut facts = selected.facts;
    facts.merge(LegalityFacts::capture(parent, available).interior());
    legal_view(selected.origin.kind, facts, crossed, stage)
}
pub(super) fn legal_projected_accepted_interior<S: LayoutScalar>(
    selected: &SelectedPolicyOf<S>,
    parent: &SelectedPolicyOf<S>,
    crossed: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    let mut facts = selected.facts;
    facts.merge(parent.facts.interior());
    legal_view(selected.origin.kind, facts, crossed, stage)
}

fn legal_view<S: LayoutScalar>(
    kind: FragmentationOpportunityKindOf<S>,
    facts: LegalityFacts,
    crossed: FragmentationBreakKinds,
    stage: FragmentationStage,
) -> bool {
    if let FragmentationOpportunityKindOf::Lines {
        before,
        after,
        orphans,
        widows,
    } = kind
        && stage == FragmentationStage::Strict
        && (before < orphans || after < widows)
    {
        return false;
    }
    if let FragmentationOpportunityKindOf::ContentEdge { gap } = kind
        && gap <= S::ZERO
    {
        return false;
    }
    if matches!(
        stage,
        FragmentationStage::Strict | FragmentationStage::RelaxLineCounts
    ) {
        let sibling = matches!(kind, FragmentationOpportunityKindOf::Sibling);
        if (!sibling || !facts.non_auto) && facts.ancestors.forbids(crossed) {
            return false;
        }
        if facts.controls.forbids(crossed) {
            return false;
        }
    }
    true
}

pub(super) fn edges<S: LayoutScalar>(
    window: &FragmentationWindowOf<S>,
    body: S,
    available: S,
    stage: FragmentationStage,
) -> Option<(S, S)> {
    let start = window.cloned_block_start;
    let end = window.cloned_block_end;
    let pair = match stage {
        FragmentationStage::TruncateBlockEnd => {
            if body + start > available {
                return None;
            }
            (start, end.min((available - body - start).max(S::ZERO)))
        }
        FragmentationStage::TruncateBlockStart => {
            if body > available {
                return None;
            }
            (start.min((available - body).max(S::ZERO)), S::ZERO)
        }
        FragmentationStage::Emergency => (S::ZERO, S::ZERO),
        _ => (start, end),
    };
    let occupied = body + pair.0 + pair.1;
    (occupied.is_finite() && occupied <= available).then_some(pair)
}
