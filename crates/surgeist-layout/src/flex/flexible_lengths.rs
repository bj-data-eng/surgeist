use super::distribution_number::DistributionNumber;
use super::items::ResolvedFlexItem;
use super::{Constants, FlexAxes, LayoutScalar};
use crate::layout_math::MaxBeforeMinScalarClampExt;
use core::cmp::Ordering;

#[derive(Clone, Copy)]
enum FlexDistribution {
    Grow,
    Shrink,
}

fn freeze_inflexible_items<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    axes: FlexAxes,
    distribution: FlexDistribution,
) -> Vec<bool> {
    items
        .iter_mut()
        .map(|item| {
            let basis = item.flex_base.outer();
            let frozen = match distribution {
                FlexDistribution::Grow => {
                    item.flex_grow_factor == S::ZERO || basis > item.hypothetical_main_size
                }
                FlexDistribution::Shrink => {
                    item.flex_shrink_factor == S::ZERO || basis < item.hypothetical_main_size
                }
            };
            let target = if frozen {
                item.hypothetical_main_size
            } else {
                basis
            };
            item.target_size = axes.with_main_size(item.target_size, target);
            frozen
        })
        .collect()
}

impl FlexDistribution {
    fn factor<Node, S: LayoutScalar>(self, item: &ResolvedFlexItem<Node, S>) -> S {
        match self {
            Self::Grow => item.flex_grow_factor,
            Self::Shrink => item.flex_shrink_factor,
        }
    }

    fn weight<Node, S: LayoutScalar>(
        self,
        item: &ResolvedFlexItem<Node, S>,
    ) -> Result<DistributionNumber<S>, S> {
        let factor = DistributionNumber::finite(self.factor(item))?;
        match self {
            Self::Grow => Ok(factor),
            Self::Shrink => Ok(factor * DistributionNumber::finite(item.flex_base.inner())?),
        }
    }
}

pub(super) fn resolve_flexible_lengths<Node, S: LayoutScalar>(
    items: &mut [ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
) -> Result<(), S> {
    let Some(container_main_size) = flex_main_size(constants) else {
        return Ok(());
    };
    let container_main_size = DistributionNumber::finite(container_main_size)?;
    let hypothetical_free_space = container_main_size - flex_used_space(items, constants, None)?;
    let distribution = match hypothetical_free_space.sign() {
        Ordering::Equal => return Ok(()),
        Ordering::Greater => FlexDistribution::Grow,
        Ordering::Less => FlexDistribution::Shrink,
    };
    let mut frozen = freeze_inflexible_items(items, constants.axes, distribution);
    let initial_free_space =
        container_main_size - flex_used_space(items, constants, Some(&frozen))?;
    let mut proposals = vec![(DistributionNumber::ZERO, Ordering::Equal); items.len()];

    while frozen.iter().any(|frozen| !*frozen) {
        let mut free_space =
            container_main_size - flex_used_space(items, constants, Some(&frozen))?;
        let mut factor_sum = S::ZERO;
        let mut weight_sum = DistributionNumber::ZERO;
        for (item, frozen) in items.iter().zip(&frozen) {
            if !*frozen {
                // Only a raw factor sum below one affects partial fill.
                factor_sum = (factor_sum + distribution.factor(item)).min(S::ONE);
                weight_sum = weight_sum + distribution.weight(item)?;
            }
        }
        let can_distribute = weight_sum.sign() != Ordering::Equal;
        if factor_sum < S::ONE {
            let partial_space = initial_free_space * DistributionNumber::finite(factor_sum)?;
            if partial_space.magnitude_less(free_space) {
                free_space = partial_space;
            }
        }
        if matches!(distribution, FlexDistribution::Shrink) {
            free_space = -free_space.abs();
        }

        let mut total_violation = DistributionNumber::ZERO;
        for ((item, frozen), proposal) in items.iter().zip(&frozen).zip(&mut proposals) {
            if *frozen {
                continue;
            }
            let base = DistributionNumber::finite(item.flex_base.outer())?;
            let share = if can_distribute {
                free_space * distribution.weight(item)? / weight_sum
            } else {
                DistributionNumber::ZERO
            };
            let proposed = base + share;
            let clamped = clamp_distribution_main_size(item, constants.axes, proposed)?;
            let violation = clamped - proposed;
            *proposal = (clamped, violation.sign());
            total_violation = total_violation + violation;
        }

        // A nonzero finite sum has at least one contributing term of its sign.
        // A zero sum freezes every remaining item.
        let total = total_violation.sign();
        for ((item, frozen), &(proposal, violation)) in
            items.iter_mut().zip(&mut frozen).zip(&proposals)
        {
            if !*frozen && (total == Ordering::Equal || violation == total) {
                // An unfrozen proposal may be recomputed after a peer freezes.
                let target = proposal.narrow();
                if !target.is_finite() {
                    return Err(target);
                }
                item.target_size = constants.axes.with_main_size(item.target_size, target);
                *frozen = true;
            }
        }
    }
    Ok(())
}

pub(super) fn flex_used_space<Node, S: LayoutScalar>(
    items: &[ResolvedFlexItem<Node, S>],
    constants: &Constants<S>,
    frozen: Option<&[bool]>,
) -> Result<DistributionNumber<S>, S> {
    let axes = constants.axes;
    let mut used = DistributionNumber::ZERO;
    for (index, item) in items.iter().enumerate() {
        let gap = if index == 0 {
            S::ZERO
        } else {
            axes.main_size(constants.gap)
        };
        let main_size = if frozen.is_none_or(|frozen| frozen[index]) {
            axes.main_size(item.target_size)
        } else {
            item.flex_base.outer()
        };
        let margin = DistributionNumber::finite(axes.main_start_edge(item.margin))?
            + DistributionNumber::finite(axes.main_end_edge(item.margin))?;
        used = used
            + (DistributionNumber::finite(gap)? + DistributionNumber::finite(main_size)? + margin);
    }
    Ok(used)
}

fn clamp_distribution_main_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    axes: FlexAxes,
    mut value: DistributionNumber<S>,
) -> Result<DistributionNumber<S>, S> {
    if let Some(maximum) = axes.main_size(item.max_size) {
        value = value.min(DistributionNumber::finite(maximum)?);
    }
    let automatic_minimum = item.automatic_min_main_size.unwrap_or(S::ZERO);
    let minimum = axes.main_size(item.min_size).unwrap_or(S::ZERO);
    let padding_border = axes.main_size((item.padding + item.border).sum_axes());
    let floor = DistributionNumber::finite(automatic_minimum.max(minimum).max(padding_border))?;
    Ok(value.max(floor))
}

pub(super) fn clamp_cross_size<Node, S: LayoutScalar>(
    item: &ResolvedFlexItem<Node, S>,
    value: S,
) -> S {
    value.clamp_max_before_min_optional(item.min_cross_size, item.max_cross_size)
}

pub(super) fn clamp_main_size_axes<S: LayoutScalar>(
    value: S,
    automatic_min: Option<S>,
    min: Option<S>,
    max: Option<S>,
    padding_border: S,
) -> S {
    let value = max.map_or(value, |max| value.min(max));
    let value = automatic_min.map_or(value, |min| value.max(min));
    let value = min.map_or(value, |min| value.max(min));
    // Hypothetical and final used main sizes floor the content box at zero.
    // Raw flex-base arithmetic is a separate phase. Keep non-finite failures
    // available to the existing checked publication boundary.
    if !value.is_finite() {
        value
    } else if !padding_border.is_finite() {
        padding_border
    } else {
        value.max(padding_border)
    }
}

pub(super) fn flex_main_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants.axes.main_size(constants.node_inner_size)
}
