//! Optional paragraph-tail quality over actual natural line measurements.

use super::quality::NaturalLine;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TailPair {
    penultimate: NaturalLine,
    last: NaturalLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TailAdmission {
    NoLastFullnessGain,
    NoPairGain,
    NoNaturalTailGain,
    Improved,
}

impl TailPair {
    pub(crate) fn new(penultimate: NaturalLine, last: NaturalLine) -> Self {
        Self { penultimate, last }
    }

    pub(crate) fn eligible_auto(self) -> bool {
        self.last.fullness() < self.penultimate.fullness()
    }

    fn minimum(self) -> f64 {
        self.penultimate.fullness().min(self.last.fullness())
    }

    pub(crate) fn admission(self, reference: Self) -> TailAdmission {
        if self.last.fullness() <= reference.last.fullness() {
            TailAdmission::NoLastFullnessGain
        } else if self.minimum() <= reference.minimum() {
            TailAdmission::NoPairGain
        } else if self.last.advance() <= reference.last.advance() {
            TailAdmission::NoNaturalTailGain
        } else {
            TailAdmission::Improved
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TailGoal(f64);

impl TailGoal {
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
pub(crate) struct TailTrial<T> {
    pub(crate) pair: TailPair,
    pub(crate) value: T,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TailWinner<T> {
    pub(crate) goal: TailGoal,
    pub(crate) trial: TailTrial<T>,
}

pub(crate) fn choose_tail<T, E>(
    reference: TailPair,
    mut run: impl FnMut(TailGoal) -> Result<Option<TailTrial<T>>, E>,
) -> Result<Option<TailWinner<T>>, E> {
    let Some(mut selection) = TailSelection::new(reference) else {
        return Ok(None);
    };
    while let Some(goal) = selection.next_goal() {
        selection.accept(run(goal)?);
    }
    Ok(selection.finish())
}

/// Finite Tail goals and ranking shared by suspended and synchronous owners.
/// Only adjacent equal goals are skipped; equal scores keep the first trial.
pub(crate) struct TailSelection<T> {
    reference: TailPair,
    step: u8,
    previous: Option<TailGoal>,
    pending: Option<TailGoal>,
    winner: Option<TailWinner<T>>,
}

impl<T> TailSelection<T> {
    pub(crate) fn new(reference: TailPair) -> Option<Self> {
        reference.eligible_auto().then_some(Self {
            reference,
            step: 0,
            previous: None,
            pending: None,
            winner: None,
        })
    }

    pub(crate) fn next_goal(&mut self) -> Option<TailGoal> {
        assert!(
            self.pending.is_none(),
            "the previous Tail goal owns a trial receipt"
        );
        while self.step <= 16 {
            let fraction = f64::from(self.step) / 16.0;
            self.step += 1;
            let goal = TailGoal(
                (1.0 - fraction) * self.reference.last.fullness()
                    + fraction * self.reference.penultimate.fullness(),
            );
            if self.previous == Some(goal) {
                continue;
            }
            self.previous = Some(goal);
            self.pending = Some(goal);
            return Some(goal);
        }
        None
    }

    pub(crate) fn accept(&mut self, trial: Option<TailTrial<T>>) {
        let goal = self
            .pending
            .take()
            .expect("a Tail trial answers the pending goal");
        let Some(trial) = trial else { return };
        if trial.pair.admission(self.reference) == TailAdmission::Improved
            && self
                .winner
                .as_ref()
                .is_none_or(|prior| trial.pair.minimum() > prior.trial.pair.minimum())
        {
            self.winner = Some(TailWinner { goal, trial });
        }
    }

    pub(crate) fn finish(self) -> Option<TailWinner<T>> {
        assert!(
            self.pending.is_none() && self.step > 16,
            "all Tail goals were answered"
        );
        self.winner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(widths: [f64; 2], advances: [f64; 2]) -> TailPair {
        TailPair::new(
            NaturalLine::from_components(widths[0], 0.0, advances[0]).unwrap(),
            NaturalLine::from_components(widths[1], 0.0, advances[1]).unwrap(),
        )
    }

    #[test]
    fn natural_tail_and_weaker_pair_both_improve_without_transferring_the_defect() {
        let reference = pair([60.0; 2], [60.0, 10.0]);
        let winner = choose_tail::<_, ()>(reference, |_| {
            Ok(Some(TailTrial {
                pair: pair([60.0; 2], [30.0, 40.0]),
                value: "repair",
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, "repair");
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0; 2], [10.0, 60.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn a_narrower_band_with_the_same_natural_tail_is_not_a_repair() {
        let reference = pair([60.0, 80.0], [60.0, 10.0]);
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0; 2], [60.0, 10.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
        assert!(
            choose_tail::<_, ()>(reference, |_| Ok(Some(TailTrial {
                pair: pair([60.0, 20.0], [30.0, 9.0]),
                value: (),
            })))
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn optional_domain_is_finite_positive_capacity_and_a_fitting_natural_measure() {
        for components in [
            [f64::NAN, 0.0, 0.0],
            [60.0, f64::INFINITY, 10.0],
            [60.0, 0.0, f64::INFINITY],
            [f64::MAX, -f64::MAX, 10.0],
            [60.0, 60.0, 0.0],
            [60.0, 70.0, 0.0],
            [60.0, 0.0, -1.0],
            [60.0, 0.0, 61.0],
        ] {
            assert!(
                NaturalLine::from_components(components[0], components[1], components[2]).is_none()
            );
        }
        assert_eq!(
            NaturalLine::from_components(80.0, 20.0, 30.0)
                .unwrap()
                .fullness(),
            0.5
        );
        assert_eq!(
            NaturalLine::from_components(f64::MAX, 0.0, f64::MAX)
                .unwrap()
                .fullness(),
            1.0
        );
    }

    #[test]
    fn equal_improved_scores_keep_the_first_trial_and_owner_errors_propagate() {
        let reference = pair([60.0; 2], [60.0, 10.0]);
        let mut index = 0;
        let winner = choose_tail::<_, ()>(reference, |_| {
            index += 1;
            Ok(Some(TailTrial {
                pair: pair([60.0; 2], [30.0, 40.0]),
                value: index,
            }))
        })
        .unwrap()
        .unwrap();
        assert_eq!(winner.trial.value, 1);
        assert_eq!(
            choose_tail::<(), _>(reference, |_| Err("owner")),
            Err("owner")
        );
        let mut called = false;
        assert!(
            choose_tail::<(), ()>(pair([60.0; 2], [30.0, 40.0]), |_| {
                called = true;
                Ok(None)
            })
            .unwrap()
            .is_none()
        );
        assert!(!called);
    }
}
