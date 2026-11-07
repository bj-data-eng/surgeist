use super::*;
use crate::LayoutScalar;

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
    let mut result = Force::empty();
    for &control in opportunity.after.iter().chain(&opportunity.before) {
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
    if let FragmentationOpportunityKindOf::Lines {
        before,
        after,
        orphans,
        widows,
    } = opportunity.kind
        && stage == FragmentationStage::Strict
        && (before < orphans || after < widows)
    {
        return false;
    }
    if let FragmentationOpportunityKindOf::ContentEdge { gap } = opportunity.kind
        && gap <= S::ZERO
    {
        return false;
    }
    if matches!(
        stage,
        FragmentationStage::Strict | FragmentationStage::RelaxLineCounts
    ) {
        let sibling = matches!(opportunity.kind, FragmentationOpportunityKindOf::Sibling);
        let all_auto = opportunity
            .after
            .iter()
            .chain(&opportunity.before)
            .filter(|control| control.role != FragmentationRole::Absolute)
            .all(|control| effective(control.value, available) == FragmentationBreak::Auto);
        if (!sibling || all_auto)
            && opportunity
                .ancestors
                .iter()
                .any(|&value| forbidden(value, crossed))
        {
            return false;
        }
        for control in opportunity.after.iter().chain(&opportunity.before) {
            if control.role == FragmentationRole::Absolute {
                continue;
            }
            let value = match control.value {
                FragmentationBreak::Avoid => FragmentationBreakInside::Avoid,
                FragmentationBreak::AvoidPage => FragmentationBreakInside::AvoidPage,
                FragmentationBreak::AvoidColumn => FragmentationBreakInside::AvoidColumn,
                FragmentationBreak::AvoidRegion => FragmentationBreakInside::AvoidRegion,
                _ => FragmentationBreakInside::Auto,
            };
            if forbidden(value, crossed) {
                return false;
            }
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
