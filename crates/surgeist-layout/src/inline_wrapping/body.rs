//! Pretty's fixed-reference body score over genuine natural formatted lines.

use super::quality::NaturalLine;
use crate::LayoutScalar;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BodyScore {
    count: usize,
    mean: f64,
    last: NaturalLine,
    pair_minimum: f64,
    minimum: f64,
    maximum: f64,
    mean_fullness: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BodyAdmission {
    CountMismatch,
    TerminalAdvanceRegression,
    TerminalFullnessRegression,
    PairRegression,
    NoBodyImprovement,
    Improved,
}

impl BodyScore {
    pub(crate) fn from_lines(lines: impl IntoIterator<Item = Option<NaturalLine>>) -> Option<Self> {
        let mut lines = lines.into_iter();
        let first = lines.next()??;
        let mut last = lines.next()??;
        let mut previous = first;
        let mut count = 2_usize;
        let mut mean = (1.0 - first.fullness()).powi(2);
        let mut minimum = first.fullness();
        let mut maximum = first.fullness();
        let mut mean_fullness = first.fullness();
        for line in lines {
            let line = line?;
            count = count.checked_add(1)?;
            let divisor = f64::from_usize(count - 1);
            // Both the squared term and running mean are in [0,1]. Their
            // difference is bounded; no huge products or pairwise sum occurs.
            let square = (1.0 - last.fullness()).powi(2);
            mean += (square - mean) / divisor;
            mean_fullness += (last.fullness() - mean_fullness) / divisor;
            minimum = minimum.min(last.fullness());
            maximum = maximum.max(last.fullness());
            previous = last;
            last = line;
        }
        Some(Self {
            count,
            mean,
            last,
            pair_minimum: previous.fullness().min(last.fullness()),
            minimum,
            maximum,
            mean_fullness,
        })
    }

    pub(crate) fn mean(self) -> f64 {
        self.mean
    }

    pub(crate) fn admission(self, reference: Self) -> BodyAdmission {
        if self.count != reference.count {
            BodyAdmission::CountMismatch
        } else if self.last.advance() < reference.last.advance() {
            BodyAdmission::TerminalAdvanceRegression
        } else if self.last.fullness() < reference.last.fullness() {
            BodyAdmission::TerminalFullnessRegression
        } else if self.pair_minimum < reference.pair_minimum {
            BodyAdmission::PairRegression
        } else if self.mean >= reference.mean {
            BodyAdmission::NoBodyImprovement
        } else {
            BodyAdmission::Improved
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BodyGoal(f64);

impl BodyGoal {
    pub(crate) fn compare_distance(
        self,
        left: Option<NaturalLine>,
        right: Option<NaturalLine>,
    ) -> Ordering {
        match (left, right) {
            (Some(left), Some(right)) => (left.fullness() - self.0)
                .abs()
                .total_cmp(&(right.fullness() - self.0).abs()),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BodyTrial<T> {
    pub(crate) score: BodyScore,
    pub(crate) value: T,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BodyWinner<T> {
    pub(crate) goal: BodyGoal,
    pub(crate) trial: BodyTrial<T>,
}

pub(crate) fn choose_body<T, E>(
    reference: BodyScore,
    mut run: impl FnMut(BodyGoal) -> Result<Option<BodyTrial<T>>, E>,
) -> Result<Option<BodyWinner<T>>, E> {
    let mut winner: Option<BodyWinner<T>> = None;
    let mut previous = None;
    for value in (0..=16)
        .map(|step| {
            let fraction = f64::from(step) / 16.0;
            (1.0 - fraction) * reference.minimum + fraction * reference.maximum
        })
        .chain(std::iter::once(reference.mean_fullness))
    {
        let goal = BodyGoal(value);
        if previous == Some(goal) {
            continue;
        }
        previous = Some(goal);
        let Some(trial) = run(goal)? else { continue };
        // All guards use the same post-Tail phase reference. The incumbent is
        // used only to rank the Q of an already-admitted alternative.
        if trial.score.admission(reference) == BodyAdmission::Improved
            && winner
                .as_ref()
                .is_none_or(|prior| trial.score.mean() < prior.trial.score.mean())
        {
            winner = Some(BodyWinner { goal, trial });
        }
    }
    Ok(winner)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(capacities: &[f64], advances: &[f64]) -> BodyScore {
        BodyScore::from_lines(
            capacities
                .iter()
                .zip(advances)
                .map(|(&c, &a)| NaturalLine::from_components(c, 0.0, a)),
        )
        .unwrap()
    }

    #[test]
    fn pretty_body_is_the_nonterminal_mean_including_long_groups() {
        let auto = score(&[60.0; 3], &[60.0, 10.0, 50.0]);
        let improved = score(&[60.0; 3], &[30.0, 40.0, 50.0]);
        assert!((auto.mean() - 25.0 / 72.0).abs() <= f64::EPSILON);
        assert!((improved.mean() - 13.0 / 72.0).abs() <= f64::EPSILON);
        assert_eq!(improved.admission(auto), BodyAdmission::Improved);
        let mut a = vec![60.0; 9];
        a.extend([60.0, 10.0, 50.0]);
        let mut b = vec![60.0; 9];
        b.extend([30.0, 40.0, 50.0]);
        let auto = score(&[60.0; 12], &a);
        let improved = score(&[60.0; 12], &b);
        assert!((auto.mean() - 25.0 / 396.0).abs() <= f64::EPSILON);
        assert!((improved.mean() - 13.0 / 396.0).abs() <= f64::EPSILON);
        assert_eq!(improved.admission(auto), BodyAdmission::Improved);
    }

    #[test]
    fn pretty_body_guards_are_independent_and_equality_is_admissible() {
        let reference = score(&[60.0; 3], &[30.0, 10.0, 40.0]);
        assert_eq!(
            score(&[60.0; 3], &[60.0, 10.0, 40.0]).admission(reference),
            BodyAdmission::Improved
        );
        assert_eq!(
            score(&[60.0; 2], &[30.0, 40.0]).admission(reference),
            BodyAdmission::CountMismatch
        );
        assert_eq!(
            score(&[60.0; 3], &[60.0, 50.0, 30.0]).admission(reference),
            BodyAdmission::TerminalAdvanceRegression
        );
        assert_eq!(
            score(&[60.0, 60.0, 80.0], &[60.0, 50.0, 40.0]).admission(reference),
            BodyAdmission::TerminalFullnessRegression
        );
        assert_eq!(
            score(&[60.0; 3], &[60.0, 5.0, 40.0]).admission(reference),
            BodyAdmission::PairRegression
        );
        assert_eq!(
            reference.admission(reference),
            BodyAdmission::NoBodyImprovement
        );
    }

    #[test]
    fn pretty_body_uses_one_fixed_guard_not_the_previous_candidate() {
        let reference = score(&[60.0; 4], &[60.0, 10.0, 10.0, 50.0]);
        let first = score(&[60.0; 4], &[20.0, 20.0, 50.0, 60.0]);
        let second = score(&[60.0; 4], &[50.0, 50.0, 30.0, 50.0]);
        assert_eq!(first.admission(reference), BodyAdmission::Improved);
        assert_eq!(second.admission(reference), BodyAdmission::Improved);
        assert_eq!(
            second.admission(first),
            BodyAdmission::TerminalAdvanceRegression
        );
        let mut calls = 0;
        let winner = choose_body::<_, ()>(reference, |_| {
            calls += 1;
            Ok(Some(BodyTrial {
                score: if calls == 1 { first } else { second },
                value: calls,
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, 2);
    }

    #[test]
    fn pretty_body_domain_ties_and_owner_errors_are_not_reinterpreted() {
        assert!(BodyScore::from_lines([NaturalLine::from_components(60.0, 0.0, 20.0)]).is_none());
        assert!(
            BodyScore::from_lines([NaturalLine::from_components(60.0, 0.0, 20.0), None]).is_none()
        );
        let reference = score(&[60.0; 3], &[60.0, 10.0, 50.0]);
        assert!(
            choose_body::<(), ()>(reference, |_| Ok(Some(BodyTrial {
                score: reference,
                value: ()
            })))
            .unwrap()
            .is_none()
        );
        let improved = score(&[60.0; 3], &[30.0, 40.0, 50.0]);
        let mut calls = 0;
        let winner = choose_body::<_, ()>(reference, |_| {
            calls += 1;
            Ok(Some(BodyTrial {
                score: improved,
                value: calls,
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, 1);
        assert_eq!(
            choose_body::<(), _>(reference, |_| Err("provider")),
            Err("provider")
        );
    }
}
