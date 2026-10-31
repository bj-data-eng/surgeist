//! Positive separator and character-boundary allocation over actual Text response.
use super::super::scope_alignment::ScopeAlignmentFacts;
use super::{FinalizedLine, FitError};
use surgeist_text::{
    SpacingBoundaryGeometry, SpacingSlotDisposition, SpacingSlotGeometry, SpacingStretchability,
};

pub(super) struct Allocation {
    pub finalized: FinalizedLine,
    pub amounts: Vec<f64>,
}

/// None is a proved unexpandable line, requiring a fresh fallback selection.
/// Numeric or unknown capability failures remain their actual owning errors.
pub(super) fn allocate(
    natural: &FinalizedLine,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<Option<Allocation>, FitError> {
    let selected = natural.selected_line();
    let context = &selected.request().context;
    let rows = natural.spacing_slots().collect::<Vec<_>>();
    let mut amounts = vec![0.0; rows.len()];
    let Some(width) = context.available_inline else {
        return Ok(Some(Allocation {
            finalized: natural.clone(),
            amounts,
        }));
    };
    let capacity = crate::group_alignment::canonical_capacity::<f64>(
        width,
        context.alignment_insets,
        context.indent,
    )
    .map_err(|error| FitError::Numeric(error.value(), error.reason()))?;
    let advance = alignment(natural);
    let extra = add(capacity, -advance)?;
    if extra == 0.0 {
        return Ok(Some(Allocation {
            finalized: natural.clone(),
            amounts,
        }));
    }
    if extra < 0.0 {
        return Ok(None);
    }
    let direct = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            (row.can_expand() && row.stretchability == SpacingStretchability::Direct)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let unresolved = rows.iter().position(|row| {
        matches!(row.disposition, SpacingSlotDisposition::Retained { .. })
            && row.stretchability != SpacingStretchability::Excluded
            && match row.geometry {
                SpacingSlotGeometry::Boundary(
                    SpacingBoundaryGeometry::ProtectedRequired
                    | SpacingBoundaryGeometry::ProtectedCursive,
                ) => false,
                SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Constrained)
                | SpacingSlotGeometry::SharedProviderGroup => true,
                SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::Independent) => {
                    matches!(row.stretchability, SpacingStretchability::Deferred(_))
                }
                SpacingSlotGeometry::Independent { intrinsic_advance } => {
                    intrinsic_advance != 0.0
                        && matches!(row.stretchability, SpacingStretchability::Deferred(_))
                }
            }
    });
    if !direct.is_empty() {
        return match direct_allocation(natural, capacity, extra, amounts.clone(), &direct, facts) {
            Ok(allocation) => Ok(Some(allocation)),
            Err(error @ FitError::SpacingBudget { .. }) => {
                if let Some(index) = unresolved {
                    demand(natural, amounts, index, capacity, extra, facts).map(Some)
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        };
    }
    if let Some(index) = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            row.can_expand() && row.stretchability == SpacingStretchability::TabDependent
        })
        .max_by_key(|(_, row)| row.traversal_order)
        .map(|(index, _)| index)
    {
        // The last admitted spacing target in Text's real grid traversal can vary
        // the incoming pen of its next included tab continuously. Earlier
        // allocations supply only a subset of those pen values. Fixed signed
        // advances and each forward next-stop/one-threshold-bump map preserve
        // monotonicity, including genuine differing periods and thresholds.
        let result = search(
            natural,
            &mut amounts,
            index,
            capacity,
            extra,
            natural.clone(),
            facts,
        )?;
        if result.exact {
            return Ok(Some(Allocation {
                finalized: result.finalized,
                amounts,
            }));
        }
    }
    // Shared geometry and neutral terminal dependencies cannot prove that no
    // expansion is available. Demand the material operation through Text, using
    // genuine handles and a complete table; never construct its diagnostic here.
    if let Some(index) = unresolved {
        return demand(natural, amounts, index, capacity, extra, facts).map(Some);
    }
    Ok(None)
}

fn demand(
    natural: &FinalizedLine,
    mut amounts: Vec<f64>,
    index: usize,
    capacity: f64,
    extra: f64,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<Allocation, FitError> {
    amounts.fill(0.0);
    amounts[index] = extra;
    let finalized = probe(natural, &amounts, facts)?;
    if alignment(&finalized) == capacity {
        Ok(Allocation { finalized, amounts })
    } else {
        Err(budget(capacity, &finalized))
    }
}

fn direct_allocation(
    natural: &FinalizedLine,
    capacity: f64,
    extra: f64,
    mut amounts: Vec<f64>,
    direct: &[usize],
    facts: &dyn ScopeAlignmentFacts,
) -> Result<Allocation, FitError> {
    let share = super::finite(extra / direct.len() as f64)?;
    if share == 0.0 {
        return Err(FitError::Numeric(
            extra,
            crate::LayoutTextNumericError::NonZeroLost,
        ));
    }
    for &index in direct {
        amounts[index] = share;
    }
    let finalized = probe(natural, &amounts, facts)?;
    if alignment(&finalized) == capacity {
        return Ok(Allocation { finalized, amounts });
    }
    // Equal native priority supplies the first table. Correct its actual Text
    // residual at the last direct owner. If its zero lower bound still exceeds
    // capacity, remove that owner's positive amount and continue to the prior
    // owner; this progress is bounded by the actual candidate inventory.
    for &index in direct.iter().rev() {
        let previous = amounts[index];
        amounts[index] = 0.0;
        let lower = probe(natural, &amounts, facts)?;
        if alignment(&lower) > capacity {
            continue;
        }
        if alignment(&lower) == capacity {
            return Ok(Allocation {
                finalized: lower,
                amounts,
            });
        }
        let result = search(
            natural,
            &mut amounts,
            index,
            capacity,
            previous,
            lower,
            facts,
        )?;
        return if result.exact {
            Ok(Allocation {
                finalized: result.finalized,
                amounts,
            })
        } else {
            // Direct content that cannot represent the required exact measure
            // is a numeric budget failure, never a justified silent underfill.
            Err(budget(capacity, &result.finalized))
        };
    }
    Err(budget(capacity, natural))
}

struct SearchResult {
    finalized: FinalizedLine,
    exact: bool,
}
struct Probe {
    amount: f64,
    finalized: FinalizedLine,
}

/// Search one nonnegative finite amount against the real monotone Text response.
/// Every accepted update strictly advances an endpoint in the finite binary64
/// domain. No retry quota, assumed continuous tab response or failed-probe
/// fallback supplies termination or reachability evidence.
fn search(
    natural: &FinalizedLine,
    amounts: &mut [f64],
    index: usize,
    capacity: f64,
    initial: f64,
    lower: FinalizedLine,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<SearchResult, FitError> {
    let mut low = Probe {
        amount: 0.0,
        finalized: lower,
    };
    let mut upper = initial;
    let mut high = loop {
        amounts[index] = upper;
        let finalized = probe(natural, amounts, facts)?;
        let measured = alignment(&finalized);
        if measured == capacity {
            return Ok(SearchResult {
                finalized,
                exact: true,
            });
        }
        if measured > capacity {
            break Probe {
                amount: upper,
                finalized,
            };
        }
        low = Probe {
            amount: upper,
            finalized,
        };
        if upper == f64::MAX {
            // This successfully executed maximum is the upper bound of every
            // finite nonnegative amount, rather than a refused-probe guess.
            return Ok(SearchResult {
                finalized: low.finalized,
                exact: false,
            });
        }
        upper = if upper > f64::MAX * 0.5 {
            f64::MAX
        } else {
            upper * 2.0
        };
    };
    loop {
        if low.amount.next_up() >= high.amount {
            amounts[index] = low.amount;
            return Ok(SearchResult {
                finalized: low.finalized,
                exact: false,
            });
        }
        // Arithmetic midpoint [0,60] probes30, not a gratuitous subnormal-scale
        // bit midpoint. At the final rounded endpoints choose the next actual
        // representable interior amount; adjacency was checked above.
        let middle = low.amount + (high.amount - low.amount) * 0.5;
        let middle = if middle <= low.amount || middle >= high.amount {
            low.amount.next_up()
        } else {
            middle
        };
        amounts[index] = middle;
        let finalized = probe(natural, amounts, facts)?;
        let measured = alignment(&finalized);
        if measured == capacity {
            return Ok(SearchResult {
                finalized,
                exact: true,
            });
        }
        if measured < capacity {
            low = Probe {
                amount: middle,
                finalized,
            };
        } else {
            high = Probe {
                amount: middle,
                finalized,
            };
        }
    }
}

fn probe(
    natural: &FinalizedLine,
    amounts: &[f64],
    facts: &dyn ScopeAlignmentFacts,
) -> Result<FinalizedLine, FitError> {
    super::finalize_spacing(natural, amounts, facts)
}

fn alignment(line: &FinalizedLine) -> f64 {
    line.measurements().alignment().advance()
}
fn add(base: f64, delta: f64) -> Result<f64, FitError> {
    crate::group_alignment::add::<f64>(base, delta)
        .map_err(|error| FitError::Numeric(error.value(), error.reason()))
}
fn budget(capacity: f64, line: &FinalizedLine) -> FitError {
    FitError::SpacingBudget {
        available: capacity,
        advance: alignment(line),
    }
}
