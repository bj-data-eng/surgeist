//! Counts of the subgrid computations a layout performs, kept per thread so a
//! test can bound the work of one layout by the depth of its subgrid chain.

use std::cell::Cell;

/// What one layout computed, counted on the current thread.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SubgridWork {
    /// Preparations of a subgrid wrapper for intrinsic traversal.
    pub(crate) wrapper_preparations: usize,
    /// Contextual Grid computations of subgrid items, probes included.
    pub(crate) contextual_grids: usize,
    /// Standalone-minimum probe runs.
    pub(crate) minimum_probes: usize,
    /// Post-placement baseline refresh recomputations.
    pub(crate) baseline_refreshes: usize,
}

/// One kind of counted work.
#[derive(Clone, Copy)]
pub(super) enum Observation {
    WrapperPreparation,
    ContextualGrid,
    MinimumProbe,
    BaselineRefresh,
}

thread_local! {
    static SUBGRID_WORK: Cell<SubgridWork> = const {
        Cell::new(SubgridWork {
            wrapper_preparations: 0,
            contextual_grids: 0,
            minimum_probes: 0,
            baseline_refreshes: 0,
        })
    };
}

pub(super) fn observe(observation: Observation) {
    SUBGRID_WORK.with(|cell| {
        let mut work = cell.get();
        let count = match observation {
            Observation::WrapperPreparation => &mut work.wrapper_preparations,
            Observation::ContextualGrid => &mut work.contextual_grids,
            Observation::MinimumProbe => &mut work.minimum_probes,
            Observation::BaselineRefresh => &mut work.baseline_refreshes,
        };
        *count += 1;
        cell.set(work);
    });
}

/// Returns the work counted since the last call and resets the counts.
pub(crate) fn take_subgrid_work() -> SubgridWork {
    SUBGRID_WORK.with(Cell::take)
}
