//! Finite residual preferences, independent of geometry and source custody.
use std::cmp::Ordering;

/// The finite components' common scale and bounded signed residual. Never
/// materialize their possibly overflowing product as a Layout coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NaturalResidual {
    scale: f64,
    value: f64,
}
impl NaturalResidual {
    pub(crate) fn from_components(capacity: f64, indent: f64, advance: f64) -> Option<Self> {
        if ![capacity, indent, advance].into_iter().all(f64::is_finite) {
            return None;
        }
        let scale = capacity.abs().max(indent.abs()).max(advance.abs());
        if scale == 0.0 {
            return Some(Self {
                scale: 0.0,
                value: 0.0,
            });
        }
        Some(Self::normalized(
            scale,
            capacity / scale - indent / scale - advance / scale,
        ))
    }

    fn normalized(scale: f64, value: f64) -> Self {
        if value == 0.0 {
            Self {
                scale: 0.0,
                value: 0.0,
            }
        } else if value.abs() < 1.0 {
            // Cancellation can leave a tiny residual from huge components.
            // This product cannot overflow, and narrows only the private
            // arithmetic scale; it never becomes a supplied geometry width.
            let scale = scale * value.abs();
            if scale == 0.0 {
                Self {
                    scale: 0.0,
                    value: 0.0,
                }
            } else {
                Self {
                    scale,
                    value: value.signum(),
                }
            }
        } else {
            Self { scale, value }
        }
    }

    fn at_scale(self, scale: f64) -> f64 {
        if scale == 0.0 {
            0.0
        } else {
            self.value * (self.scale / scale)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BalanceGoal(NaturalResidual);
impl BalanceGoal {
    /// Less means the left candidate is closer. One common scale covers both
    /// candidates and the goal; unrelated normalized magnitudes cannot rank.
    pub(crate) fn compare_distance(
        self,
        left: NaturalResidual,
        right: NaturalResidual,
    ) -> Ordering {
        let scale = self.0.scale.max(left.scale).max(right.scale);
        let goal = self.0.at_scale(scale);
        (left.at_scale(scale) - goal)
            .abs()
            .total_cmp(&(right.at_scale(scale) - goal).abs())
    }
}

/// Compare fixed-count eligible groups. Invalid count/profile is a caller-basis
/// mismatch, not a reason to reject ordinary zero/one/long Auto groups.
pub(crate) fn variance_order(
    left: &[NaturalResidual],
    right: &[NaturalResidual],
) -> Option<Ordering> {
    if left.len() != right.len() || left.len() > 10 {
        return None;
    }
    let scale = left
        .iter()
        .chain(right)
        .fold(0.0_f64, |scale, r| scale.max(r.scale));
    let score = |values: &[NaturalResidual]| {
        let mut sum = 0.0;
        for (i, a) in values.iter().enumerate() {
            for b in &values[i + 1..] {
                let difference = a.at_scale(scale) - b.at_scale(scale);
                sum += difference * difference;
            }
        }
        sum
    };
    // Same count cancels the variance divisor; <=45 bounded terms per side.
    Some(score(left).total_cmp(&score(right)))
}

fn goals(values: &[NaturalResidual]) -> Vec<BalanceGoal> {
    let scale = values.iter().fold(0.0_f64, |scale, r| scale.max(r.scale));
    let normalized = values.iter().map(|r| r.at_scale(scale)).collect::<Vec<_>>();
    let minimum = normalized.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = normalized.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = normalized.iter().sum::<f64>()
        / f64::from(u32::try_from(values.len()).expect("eligible group has at most ten lines"));
    let mut result: Vec<BalanceGoal> = Vec::with_capacity(18);
    for value in (0..=16)
        .map(|j| {
            let fraction = f64::from(j) / 16.0;
            (1.0 - fraction) * minimum + fraction * maximum
        })
        .chain(std::iter::once(mean))
    {
        let goal = BalanceGoal(NaturalResidual::normalized(scale, value));
        if !result.iter().any(|prior| {
            let common = prior.0.scale.max(goal.0.scale);
            prior.0.at_scale(common) == goal.0.at_scale(common)
        }) {
            result.push(goal);
        }
    }
    result
}

/// Some from the owner authenticates the real terminal and frozen prefix. The
/// controller additionally checks the original formatted count and variance.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupTrial<T> {
    pub(crate) residuals: Vec<NaturalResidual>,
    pub(crate) value: T,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GroupWinner<T> {
    pub(crate) goal: Option<BalanceGoal>,
    pub(crate) trial: GroupTrial<T>,
}

/// Eligible groups only. The owning formatter restores its genuine group-entry
/// checkpoint before each invocation, stops at its actual terminal/count, and
/// subsequently replays the winner's complete value/basis. Errors propagate;
/// None means a genuinely inadmissible path, never a fabricated empty group.
pub(crate) fn choose_group<T, E>(
    reference: &[NaturalResidual],
    auto: Option<GroupTrial<T>>,
    mut run: impl FnMut(BalanceGoal) -> Result<Option<GroupTrial<T>>, E>,
) -> Result<Option<GroupWinner<T>>, E> {
    if !(2..=10).contains(&reference.len()) {
        return Ok(None);
    }
    let targets = goals(
        auto.as_ref()
            .filter(|trial| trial.residuals.len() == reference.len())
            .map_or(reference, |trial| trial.residuals.as_slice()),
    );
    let admissible = |trial: &GroupTrial<T>| {
        variance_order(&trial.residuals, reference).is_some_and(|order| order != Ordering::Greater)
    };
    let mut winner = auto
        .filter(admissible)
        .map(|trial| GroupWinner { goal: None, trial });
    for goal in targets {
        let Some(trial) = run(goal)? else { continue };
        if admissible(&trial)
            && winner.as_ref().is_none_or(|prior| {
                variance_order(&trial.residuals, &prior.trial.residuals) == Some(Ordering::Less)
            })
        {
            winner = Some(GroupWinner {
                goal: Some(goal),
                trial,
            });
        }
    }
    Ok(winner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline_wrapping::EdgePreferences;

    fn r(value: f64) -> NaturalResidual {
        NaturalResidual::from_components(value, 0.0, 0.0).unwrap()
    }
    fn trial<T>(values: &[f64], value: T) -> GroupTrial<T> {
        GroupTrial {
            residuals: values.iter().copied().map(r).collect(),
            value,
        }
    }

    #[test]
    fn actual_components_reject_nonfinite_without_forming_overflowing_residuals() {
        for triple in [
            (f64::INFINITY, 0.0, 0.0),
            (0.0, f64::NAN, 0.0),
            (0.0, 0.0, f64::NEG_INFINITY),
        ] {
            assert!(NaturalResidual::from_components(triple.0, triple.1, triple.2).is_none());
        }
        let extreme = NaturalResidual::from_components(f64::MAX, -f64::MAX, -f64::MAX).unwrap();
        assert_eq!(extreme.scale, f64::MAX);
        assert_eq!(extreme.value, 3.0);
        let cancelled = NaturalResidual::from_components(f64::MAX, f64::MAX, 0.0).unwrap();
        assert_eq!(cancelled, r(0.0));
    }

    #[test]
    fn pairwise_variance_uses_literal_signed_and_extreme_relations() {
        assert_eq!(
            variance_order(&[r(30.0), r(20.0)], &[r(0.0), r(50.0)]),
            Some(Ordering::Less)
        );
        assert_eq!(
            variance_order(&[r(-1.0), r(1.0)], &[r(1.0), r(-1.0)]),
            Some(Ordering::Equal)
        );
        assert_eq!(
            variance_order(&[r(0.0), r(0.0)], &[r(f64::MAX), r(f64::MAX)]),
            Some(Ordering::Equal)
        );
        assert_eq!(
            variance_order(
                &[r(-f64::MAX), r(f64::MAX)],
                &[r(-f64::MAX / 2.0), r(f64::MAX / 2.0)]
            ),
            Some(Ordering::Greater)
        );
        let cancelled = NaturalResidual::from_components(f64::MAX, f64::MAX, 1.0).unwrap();
        assert_eq!(
            variance_order(&[r(0.0), cancelled], &[r(0.0), r(2.0)]),
            Some(Ordering::Less)
        );
        assert_eq!(variance_order(&[], &[]), Some(Ordering::Equal));
        assert_eq!(variance_order(&[r(1.0)], &[]), None);
        assert_eq!(variance_order(&[r(1.0); 11], &[r(1.0); 11]), None);
    }

    #[test]
    fn distance_uses_common_magnitude_and_retains_real_near_ties() {
        let goal = BalanceGoal(r(10.0));
        assert_eq!(goal.compare_distance(r(9.0), r(1000.0)), Ordering::Less);
        assert_eq!(
            BalanceGoal(r(1.0)).compare_distance(r(0.0), r(2.0)),
            Ordering::Equal
        );
        let tiny = 2.0_f64.powi(-500);
        let next = f64::from_bits(tiny.to_bits() + 1);
        assert_eq!(
            BalanceGoal(r(0.0)).compare_distance(r(tiny), r(next)),
            Ordering::Less
        );
        assert_eq!(
            BalanceGoal(r(-f64::MAX)).compare_distance(r(0.0), r(f64::MAX)),
            Ordering::Less
        );
    }

    #[test]
    fn finite_lattice_and_mean_are_once_derived_and_deduplicated() {
        let targets = goals(&[r(0.0), r(0.0), r(1.0)]);
        assert_eq!(targets.len(), 18);
        assert_eq!(targets[0].0, r(0.0));
        assert_eq!(targets[16].compare_distance(r(1.0), r(0.0)), Ordering::Less);
        assert_eq!(
            targets[17].compare_distance(r(1.0 / 3.0), r(0.0)),
            Ordering::Less
        );
        assert_eq!(goals(&[r(0.0), r(1.0)]).len(), 17);
        assert_eq!(goals(&[r(3.0), r(3.0)]).len(), 1);
        let extreme = goals(&[r(-f64::MAX), r(f64::MAX)]);
        assert_eq!(extreme.len(), 17);
        assert_eq!(extreme[8].0, r(0.0));
        assert!(
            extreme
                .iter()
                .all(|goal| goal.0.scale.is_finite() && goal.0.value.is_finite())
        );
    }

    #[test]
    fn controller_preserves_auto_ties_and_bounds_portfolio_without_resampling() {
        let reference = [r(0.0), r(0.0), r(1.0)];
        let mut calls = 0;
        let winner = choose_group(&reference, Some(trial(&[0.0, 0.0, 1.0], 0)), |_| {
            calls += 1;
            Ok::<_, ()>(Some(trial(&[0.5, 0.5, 0.5], calls)))
        })
        .unwrap()
        .unwrap();
        assert_eq!(calls, 18);
        assert_eq!(winner.trial.value, 1);
        assert!(winner.goal.is_some());
        let winner = choose_group(&[r(0.0), r(1.0)], Some(trial(&[0.0, 1.0], "auto")), |_| {
            Ok::<_, ()>(Some(trial(&[1.0, 0.0], "trial")))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, "auto");
        assert_eq!(winner.goal, None);
    }

    #[test]
    fn controller_rejects_count_and_reference_regression_without_hiding_errors() {
        let mut calls = 0;
        let winner = choose_group(&[r(0.0), r(1.0)], Some(trial(&[0.0, 1.0], "auto")), |_| {
            calls += 1;
            Ok::<_, ()>(match calls {
                1 => Some(trial(&[0.0], "wrong-count")),
                2 => Some(trial(&[-1.0, 2.0], "worse")),
                _ => None,
            })
        })
        .unwrap()
        .unwrap();
        assert_eq!(calls, 17);
        assert_eq!(winner.trial.value, "auto");
        let error = choose_group::<(), _>(&[r(0.0), r(1.0)], None, |_| Err("provider"));
        assert_eq!(error, Err("provider"));
        let absent = choose_group::<(), ()>(&[r(0.0), r(1.0)], None, |_| Ok(None)).unwrap();
        assert_eq!(absent, None);
        assert_eq!(
            choose_group::<(), ()>(&[r(0.0)], None, |_| panic!("not eligible")).unwrap(),
            None
        );
    }

    #[test]
    fn controller_samples_rolling_auto_once_even_when_reference_guard_rejects_it() {
        let mut calls = 0;
        let winner = choose_group(
            &[r(0.0), r(1.0), r(2.0)],
            Some(trial(&[0.0, 0.0, 100.0], "worse-local-auto")),
            |goal| {
                calls += 1;
                if calls == 1 {
                    assert_eq!(goal.compare_distance(r(0.0), r(100.0)), Ordering::Less);
                }
                Ok::<_, ()>(Some(trial(&[1.0, 1.0, 1.0], "balanced")))
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(calls, 18);
        assert_eq!(winner.trial.value, "balanced");
    }

    #[test]
    fn goal_rank_never_overrides_dominance_and_auto_keeps_its_normal_ties() {
        let preferences = EdgePreferences::new(2);
        let closer = |a: &(f64, &str), b: &(f64, &str)| a.0.total_cmp(&b.0);
        let mut frontier = preferences.frontier();
        frontier
            .consider_ranked(
                preferences.class(&[0, 1]).unwrap(),
                (0.0, "near-dominated"),
                closer,
            )
            .unwrap();
        frontier
            .consider_ranked(
                preferences.class(&[0]).unwrap(),
                (100.0, "dominates"),
                closer,
            )
            .unwrap();
        assert_eq!(
            frontier.into_preferred_ranked(closer).unwrap().1,
            "dominates"
        );
        let mut frontier = preferences.frontier();
        frontier
            .consider(preferences.class(&[0]).unwrap(), "early")
            .unwrap();
        frontier
            .consider(preferences.class(&[1]).unwrap(), "normal-last")
            .unwrap();
        assert_eq!(frontier.into_preferred(), Some("normal-last"));
    }

    #[test]
    fn ranked_equal_and_incomparable_classes_keep_distance_then_normal_serial() {
        let preferences = EdgePreferences::new(2);
        let closer = |a: &(f64, &str), b: &(f64, &str)| a.0.total_cmp(&b.0);
        for edges in [&[][..], &[0][..]] {
            let mut frontier = preferences.frontier();
            frontier
                .consider_ranked(preferences.class(edges).unwrap(), (1.0, "near"), closer)
                .unwrap();
            frontier
                .consider_ranked(
                    preferences.class(edges).unwrap(),
                    (2.0, "later-far"),
                    closer,
                )
                .unwrap();
            assert_eq!(frontier.into_preferred_ranked(closer).unwrap().1, "near");
        }
        let mut frontier = preferences.frontier();
        frontier
            .consider_ranked(
                preferences.class(&[0]).unwrap(),
                (1.0, "early-near"),
                closer,
            )
            .unwrap();
        frontier
            .consider_ranked(preferences.class(&[1]).unwrap(), (2.0, "later-far"), closer)
            .unwrap();
        assert_eq!(
            frontier.into_preferred_ranked(closer).unwrap().1,
            "early-near"
        );
        let mut frontier = preferences.frontier();
        frontier
            .consider_ranked(preferences.class(&[0]).unwrap(), (1.0, "early-tie"), closer)
            .unwrap();
        frontier
            .consider_ranked(
                preferences.class(&[1]).unwrap(),
                (1.0, "normal-last"),
                closer,
            )
            .unwrap();
        assert_eq!(
            frontier.into_preferred_ranked(closer).unwrap().1,
            "normal-last"
        );
    }
}
