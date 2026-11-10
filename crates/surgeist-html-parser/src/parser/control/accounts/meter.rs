//! The accounting of a parse with no host and no profiled operation, kept in
//! plain counters.
//!
//! The ledger form of that accounting has two scopes: a charging scope that
//! records every work category and every resource without a cap, and its parent,
//! the cap scope, which carries the parse limits and totals the credits
//! (`accounting::private`). Nothing else observes it, so [`Meter`] holds the
//! state of both scopes in tables and applies the ledger's admission rules to
//! them directly:
//!
//! - A batch is atomic and its items are checked in order, each against the
//!   charging scope and then the cap scope. The first refusal names the scope,
//!   the item, the change in that scope's units, the value the change was applied
//!   to (which includes the items of the batch before it) and the reason.
//! - A resource peak includes the usage an item reaches inside a batch.
//! - A refused batch, and a checked batch, leave usage and peaks as they were.
//!
//! A refusal is the ledger's own [`LedgerError`], so the stop mapping reads a
//! refusal of the meter and a refusal of an operation alike. The equivalence of
//! the two on every outcome and every figure is tested against the ledger.

use crate::HtmlParseLimits;
use crate::accounting::LedgerError;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlChange, HtmlLimits, HtmlResource, HtmlResourceTable, HtmlWork,
    HtmlWorkTable,
};
use surgeist_ledger::{Change, Dimension, Error, Refusal, RefusalReason, Scope};

/// What a parse can read of a scope: its work totals and the live and peak
/// value of each resource.
pub(crate) trait Reading {
    fn work(&self, dimension: HtmlWork) -> u64;
    fn live(&self, dimension: HtmlResource) -> u64;
    fn peak(&self, dimension: HtmlResource) -> u64;
}

impl Reading for surgeist_ledger::Usage<HtmlWork, HtmlResource> {
    fn work(&self, dimension: HtmlWork) -> u64 {
        Self::work(self, dimension)
    }
    fn live(&self, dimension: HtmlResource) -> u64 {
        Self::live(self, dimension)
    }
    fn peak(&self, dimension: HtmlResource) -> u64 {
        Self::peak(self, dimension)
    }
}

/// The values a parse reads of a scope, copied out of it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Readings {
    work: HtmlWorkTable<u64>,
    live: HtmlResourceTable<u64>,
    peak: HtmlResourceTable<u64>,
}

impl Readings {
    pub(crate) fn capture(reading: &dyn Reading) -> Self {
        let mut copy = Self {
            work: HtmlWorkTable::splat(0),
            live: HtmlResourceTable::splat(0),
            peak: HtmlResourceTable::splat(0),
        };
        for &dimension in HtmlWork::ALL {
            *HtmlWork::get_mut(&mut copy.work, dimension) = reading.work(dimension);
        }
        for &dimension in HtmlResource::ALL {
            *HtmlResource::get_mut(&mut copy.live, dimension) = reading.live(dimension);
            *HtmlResource::get_mut(&mut copy.peak, dimension) = reading.peak(dimension);
        }
        copy
    }
    pub(crate) fn work(&self, dimension: HtmlWork) -> u64 {
        *HtmlWork::get(&self.work, dimension)
    }
    pub(crate) fn live(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(&self.live, dimension)
    }
    pub(crate) fn peak(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(&self.peak, dimension)
    }
}

/// One scope of a [`Meter`], read in place.
pub(super) struct View<'a> {
    work: &'a HtmlWorkTable<u64>,
    live: &'a HtmlResourceTable<u64>,
    peak: &'a HtmlResourceTable<u64>,
}

impl Reading for View<'_> {
    fn work(&self, dimension: HtmlWork) -> u64 {
        *HtmlWork::get(self.work, dimension)
    }
    fn live(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(self.live, dimension)
    }
    fn peak(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(self.peak, dimension)
    }
}

/// The value an item had before it was applied, to restore it when a later item
/// of the batch is refused.
#[derive(Clone, Copy)]
enum Undo {
    Work {
        charging: HtmlWork,
        charging_total: u64,
        cap: HtmlWork,
        cap_total: u64,
    },
    Live {
        resource: HtmlResource,
        live: u64,
        peak: u64,
    },
}

/// The work dimension of the cap scope that a work dimension of the charging
/// scope counts in, at rate one: the parser categories and the credits count as
/// credits, and a semantic step as a semantic step
/// (`HtmlWork::charging_tariff`).
const fn cap_work(dimension: HtmlWork) -> HtmlWork {
    match dimension.credits_per_unit() {
        Some(_) => HtmlWork::Credits,
        None => HtmlWork::SemanticStep,
    }
}

/// The state of both scopes of a parse's own operation.
pub(crate) struct Meter {
    cap: Scope,
    charging: Scope,
    /// The caps of the cap scope. The charging scope has none.
    limits: HtmlLimits,
    charging_work: HtmlWorkTable<u64>,
    cap_work: HtmlWorkTable<u64>,
    /// The live value of each resource, which both scopes hold alike: the
    /// tariff from the charging scope to the cap scope is the identity.
    live: HtmlResourceTable<u64>,
    peak: HtmlResourceTable<u64>,
    undo: Vec<Undo>,
}

impl Meter {
    /// The meter of a parse under `limits`, with the scopes of a private
    /// operation: it names refusals by them, and nothing else reads them.
    pub(super) fn new(limits: &HtmlParseLimits) -> Result<Self, HtmlAccountingFault> {
        Self::with_limits(limits.ledger_limits())
    }

    pub(super) fn with_limits(limits: HtmlLimits) -> Result<Self, HtmlAccountingFault> {
        let (_, cap, charging) = crate::accounting::private(limits)?;
        Ok(Self {
            cap,
            charging,
            limits,
            charging_work: HtmlWorkTable::splat(0),
            cap_work: HtmlWorkTable::splat(0),
            live: HtmlResourceTable::splat(0),
            peak: HtmlResourceTable::splat(0),
            undo: Vec::new(),
        })
    }

    pub(super) const fn cap(&self) -> Scope {
        self.cap
    }

    pub(super) const fn charging(&self) -> Scope {
        self.charging
    }

    /// Admits a batch to the charging scope, all or nothing.
    pub(super) fn apply(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        self.run(changes, false)
    }

    /// Whether [`Self::apply`] would admit the batch. Nothing changes.
    pub(super) fn check(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        self.run(changes, true)
    }

    /// The state of one of the two scopes.
    pub(super) fn view(&self, scope: Scope) -> Result<View<'_>, LedgerError> {
        let work = if scope == self.charging {
            &self.charging_work
        } else if scope == self.cap {
            &self.cap_work
        } else {
            return Err(Error::ForeignScope);
        };
        Ok(View {
            work,
            live: &self.live,
            peak: &self.peak,
        })
    }

    /// The caps configured for one of the two scopes.
    pub(super) fn limits(&self, scope: Scope) -> Result<HtmlLimits, LedgerError> {
        if scope == self.cap {
            Ok(self.limits)
        } else if scope == self.charging {
            Ok(HtmlLimits::unlimited())
        } else {
            Err(Error::ForeignScope)
        }
    }

    fn run(&mut self, changes: &[HtmlChange], check_only: bool) -> Result<(), LedgerError> {
        if let ([change], false) = (changes, check_only) {
            // A change is checked before it changes anything, so a batch of
            // one needs no record of what to restore.
            return self.step(0, *change, false);
        }
        self.undo.clear();
        self.undo
            .try_reserve(changes.len())
            .map_err(|_| Error::Allocation)?;
        let mut outcome = Ok(());
        for (item, &change) in changes.iter().enumerate() {
            outcome = self.step(item, change, true);
            if outcome.is_err() {
                break;
            }
        }
        if outcome.is_err() || check_only {
            self.rollback();
        }
        outcome
    }

    fn rollback(&mut self) {
        while let Some(undo) = self.undo.pop() {
            match undo {
                Undo::Work {
                    charging,
                    charging_total,
                    cap,
                    cap_total,
                } => {
                    *HtmlWork::get_mut(&mut self.charging_work, charging) = charging_total;
                    *HtmlWork::get_mut(&mut self.cap_work, cap) = cap_total;
                }
                Undo::Live {
                    resource,
                    live,
                    peak,
                } => {
                    *HtmlResource::get_mut(&mut self.live, resource) = live;
                    *HtmlResource::get_mut(&mut self.peak, resource) = peak;
                }
            }
        }
    }

    fn refused(
        scope: Scope,
        item: usize,
        change: HtmlChange,
        current: u64,
        reason: RefusalReason,
    ) -> LedgerError {
        Error::Refused(Refusal {
            scope,
            item,
            change,
            current,
            reason,
        })
    }

    /// Applies one change of a batch, checking the charging scope and then the
    /// cap scope before anything changes. With `journal` it records the values
    /// it replaces, for [`Self::rollback`].
    fn step(&mut self, item: usize, change: HtmlChange, journal: bool) -> Result<(), LedgerError> {
        match change {
            Change::Work(dimension, amount) => {
                let charging_total = *HtmlWork::get(&self.charging_work, dimension);
                let Some(charging_next) = charging_total.checked_add(amount) else {
                    return Err(Self::refused(
                        self.charging,
                        item,
                        change,
                        charging_total,
                        RefusalReason::Overflow,
                    ));
                };
                let cap = cap_work(dimension);
                let at_cap = Change::Work(cap, amount);
                let cap_total = *HtmlWork::get(&self.cap_work, cap);
                let Some(cap_next) = cap_total.checked_add(amount) else {
                    return Err(Self::refused(
                        self.cap,
                        item,
                        at_cap,
                        cap_total,
                        RefusalReason::Overflow,
                    ));
                };
                let limit = self.limits.work(cap);
                if cap_next > limit {
                    return Err(Self::refused(
                        self.cap,
                        item,
                        at_cap,
                        cap_total,
                        RefusalReason::Limit { limit },
                    ));
                }
                if journal {
                    self.undo.push(Undo::Work {
                        charging: dimension,
                        charging_total,
                        cap,
                        cap_total,
                    });
                }
                *HtmlWork::get_mut(&mut self.charging_work, dimension) = charging_next;
                *HtmlWork::get_mut(&mut self.cap_work, cap) = cap_next;
            }
            Change::Acquire(resource, amount) => {
                let live = *HtmlResource::get(&self.live, resource);
                let Some(next) = live.checked_add(amount) else {
                    return Err(Self::refused(
                        self.charging,
                        item,
                        change,
                        live,
                        RefusalReason::Overflow,
                    ));
                };
                let limit = self.limits.live(resource);
                if next > limit {
                    return Err(Self::refused(
                        self.cap,
                        item,
                        change,
                        live,
                        RefusalReason::Limit { limit },
                    ));
                }
                let peak = HtmlResource::get_mut(&mut self.peak, resource);
                if journal {
                    self.undo.push(Undo::Live {
                        resource,
                        live,
                        peak: *peak,
                    });
                }
                *peak = (*peak).max(next);
                *HtmlResource::get_mut(&mut self.live, resource) = next;
            }
            Change::Release(resource, amount) => {
                let live = *HtmlResource::get(&self.live, resource);
                let Some(next) = live.checked_sub(amount) else {
                    return Err(Self::refused(
                        self.charging,
                        item,
                        change,
                        live,
                        RefusalReason::Underflow,
                    ));
                };
                if journal {
                    self.undo.push(Undo::Live {
                        resource,
                        live,
                        peak: *HtmlResource::get(&self.peak, resource),
                    });
                }
                *HtmlResource::get_mut(&mut self.live, resource) = next;
            }
        }
        Ok(())
    }
}
