//! The occupancy tree against a plain array of end rows, with seeded random
//! operations and widenings.

use super::*;
use std::convert::Infallible;

struct Rng(u64);
impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        usize::try_from((self.0 >> 33) % u64::try_from(n).unwrap()).unwrap()
    }
}

fn free(_: usize) -> Result<(), Infallible> {
    Ok(())
}

#[test]
fn raising_clearing_and_finding_agree_with_a_plain_array() {
    for seed in 0..40 {
        let mut rng = Rng(seed);
        let mut tree = Occupancy::default();
        let mut naive: Vec<usize> = Vec::new();
        for _ in 0..200 {
            match rng.below(5) {
                0 => {
                    // Widen by a few columns, as a table does.
                    let columns = naive.len() + 1 + rng.below(9);
                    naive.resize(columns, 0);
                    let leaves = Occupancy::leaves_for(columns);
                    if leaves > tree.leaves() {
                        tree.widen(leaves);
                    }
                }
                1 | 2 if !naive.is_empty() => {
                    let first = rng.below(naive.len());
                    let end = first + 1 + rng.below(naive.len() - first);
                    let to = if rng.below(4) == 0 {
                        usize::MAX
                    } else {
                        rng.below(30)
                    };
                    tree.raise(Columns { first, end }, to, &mut free).unwrap();
                    for column in &mut naive[first..end] {
                        *column = (*column).max(to);
                    }
                }
                3 if !naive.is_empty() => {
                    let first = rng.below(naive.len());
                    let end = first + 1 + rng.below(naive.len() - first);
                    tree.clear(Columns { first, end }, &mut free).unwrap();
                    for column in &mut naive[first..end] {
                        *column = 0;
                    }
                }
                _ => {
                    let from = rng.below(naive.len() + 2);
                    let most = rng.below(30);
                    let expected = (from..naive.len()).find(|c| naive[*c] <= most);
                    let found = tree.first_at_most(from, most, &mut free).unwrap();
                    // Columns past the table are free: the tree may report one.
                    let found = found.filter(|c| *c < naive.len());
                    assert_eq!(found, expected, "seed {seed} from {from} most {most}");
                }
            }
        }
    }
}

#[test]
fn a_search_costs_logarithmic_steps_and_pays_before_each() {
    let mut tree = Occupancy::default();
    tree.widen(1024);
    tree.raise(
        Columns {
            first: 0,
            end: 1000,
        },
        7,
        &mut free,
    )
    .unwrap();
    let mut steps = 0;
    let found = tree
        .first_at_most(0, 6, &mut |n| {
            steps += n;
            Ok::<(), Infallible>(())
        })
        .unwrap();
    assert_eq!(found, Some(1000));
    assert!(steps <= 4 * 11, "{steps}");
    // A refusal after two units stops the descent.
    let mut paid = 0;
    let refused = tree.first_at_most(0, 6, &mut |n| {
        if paid + n > 2 {
            return Err(());
        }
        paid += n;
        Ok(())
    });
    assert_eq!(refused, Err(()));
    assert_eq!(paid, 2);
}

#[test]
fn widening_keeps_every_value_and_frees_the_new_columns() {
    let mut tree = Occupancy::default();
    tree.widen(4);
    tree.raise(Columns { first: 1, end: 3 }, 9, &mut free)
        .unwrap();
    tree.widen(16);
    assert_eq!(tree.first_at_most(1, 8, &mut free).unwrap(), Some(3));
    assert_eq!(tree.first_at_most(0, 0, &mut free).unwrap(), Some(0));
    assert_eq!(tree.first_at_most(1, 9, &mut free).unwrap(), Some(1));
}
