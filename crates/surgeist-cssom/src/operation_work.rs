//! The work of one store or batch operation, charged once to a host's `cssom`
//! scope.
//!
//! An operation of a store that holds a host opens a record on its thread; the
//! census runs and rule formatting the operation does, including those of the
//! operations it calls, add to it. When the outermost operation returns, the
//! record is charged as one batch. A commit inside an operation settles the
//! record before it publishes, so that a refused charge publishes nothing.
//! With no host no record is open, and adding to it does nothing.
use crate::CssomError;
use std::cell::Cell;
use std::sync::Arc;
use surgeist_css::accounting::{
    CssAccount, CssChange, CssFront, CssHostScope, CssResource, CssWork,
};

/// What the census runs and rule formatting of one operation did.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OperationWork {
    runs: usize,
    objects: usize,
    entries: usize,
    steps: usize,
    nodes: usize,
}

impl OperationWork {
    /// The changes of the work done, without the dimensions it did none of.
    fn changes(&self) -> ([CssChange; 5], usize) {
        let mut changes = [CssChange::Work(CssWork::CensusRun, 0); 5];
        let mut length = 0;
        for (dimension, count) in [
            (CssWork::CensusRun, self.runs),
            (CssWork::CensusObject, self.objects),
            (CssWork::CensusEntry, self.entries),
            (CssWork::SourceVisit, self.steps),
            (CssWork::RuleFormatNode, self.nodes),
        ] {
            if count > 0 {
                changes[length] =
                    CssChange::Work(dimension, u64::try_from(count).unwrap_or(u64::MAX));
                length += 1;
            }
        }
        (changes, length)
    }
}

/// The record of the outermost operation of one store on this thread.
#[derive(Clone, Copy, Debug)]
struct Open {
    /// The key of the owner of the store whose operation opened it.
    owner: usize,
    work: OperationWork,
}

thread_local! {
    static OPEN: Cell<Option<Open>> = const { Cell::new(None) };
}

/// Adds to the open record, if any.
fn add(note: impl FnOnce(&mut OperationWork)) {
    OPEN.with(|open| {
        if let Some(mut record) = open.get() {
            note(&mut record.work);
            open.set(Some(record));
        }
    });
}

/// One census run that examined `objects` objects and `entries` entries and
/// took `steps` steps of its source walk.
pub(crate) fn census(objects: usize, entries: usize, steps: usize) {
    add(|work| {
        work.runs = work.runs.saturating_add(1);
        work.objects = work.objects.saturating_add(objects);
        work.entries = work.entries.saturating_add(entries);
        work.steps = work.steps.saturating_add(steps);
    });
}

/// One rule formatting that prepared `nodes` nodes.
pub(crate) fn formatted(nodes: usize) {
    add(|work| work.nodes = work.nodes.saturating_add(nodes));
}

/// The open record of the outermost operation; it is closed on every path.
struct Record;

impl Record {
    fn open(owner: usize) -> Self {
        OPEN.with(|open| {
            open.set(Some(Open {
                owner,
                work: OperationWork::default(),
            }));
        });
        Self
    }
    fn close(self) -> OperationWork {
        OPEN.with(Cell::take)
            .map(|record| record.work)
            .unwrap_or_default()
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        OPEN.with(|open| open.set(None));
    }
}

/// The record of another store's operation, set aside while an operation of
/// this store runs inside it (from a caller's callback), and put back on every
/// path.
struct SetAside(Option<Open>);

impl SetAside {
    fn take() -> Self {
        Self(OPEN.with(Cell::take))
    }
}

impl Drop for SetAside {
    fn drop(&mut self) {
        OPEN.with(|open| open.set(self.0));
    }
}

/// Whether an operation of the store owned by `owner` has a record open.
fn open_for(owner: usize) -> bool {
    OPEN.with(|open| open.get().is_some_and(|record| record.owner == owner))
}

/// Runs one operation of the store owned by `owner` (the key of its owner
/// identity), whose host is `host`.
///
/// With a host, the outermost operation of the store runs inside one phase of
/// the host's `cssom` scope and then charges that scope once with the work the
/// record holds, whether or not the operation succeeded; an operation of the
/// same store inside it adds its work to the outer one's. An operation of
/// another store, which a caller's callback can run, sets the outer record
/// aside, so each store is charged only its own work and a store without a
/// host is charged nothing. With no host it only runs `run`.
///
/// Every charge of a commit inside the operation comes before the commit
/// publishes (see [`charged_before_publication`]), and no census runs after a
/// publication, so the charge made when the operation returns after one is
/// empty and makes no ledger call.
///
/// # Errors
///
/// The error of `run` when it fails. When it succeeds,
/// [`CssomError::Accounting`] when the host does not admit the charge, and
/// when the phase cannot open, in which case `run` has not been called.
pub(crate) fn charged<T, E: From<CssomError>>(
    owner: usize,
    host: Option<&CssHostScope>,
    run: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if open_for(owner) {
        return run();
    }
    let _outer = SetAside::take();
    let Some(host) = host else {
        return run();
    };
    let record = Record::open(owner);
    let account = CssAccount::host(host);
    let result = account
        .timed(CssFront::Cssom, run)
        .map_err(|fault| E::from(CssomError::Accounting(fault.into())))?;
    let charge = settle(account, record.close(), NO_GAUGES);
    let value = result?;
    charge.map_err(E::from)?;
    Ok(value)
}

/// No gauge changes.
const NO_GAUGES: ([CssChange; 3], usize) = ([CssChange::Work(CssWork::CensusRun, 0); 3], 0);

/// Runs the preparation of a commit of the store owned by `owner`, whose
/// host is `host`, and charges, as one batch with the work, the gauge
/// changes `gauges` gives for the prepared result.
///
/// Outermost, it is [`charged`] with the gauge changes added to its batch
/// when the preparation succeeds. Inside an open operation of the same store,
/// a successful preparation charges the work recorded so far with the gauge
/// changes and empties the record, before the commit publishes; a failed one
/// leaves its work for the outer charge.
///
/// # Errors
///
/// The error of `run`, and [`CssomError::Accounting`] when the host does not
/// admit the batch; nothing of it is then admitted and the prepared commit is
/// dropped unpublished.
pub(crate) fn charged_before_publication<T, E: From<CssomError>>(
    owner: usize,
    host: Option<&CssHostScope>,
    run: impl FnOnce() -> Result<T, E>,
    gauges: impl FnOnce(&T) -> ([CssChange; 3], usize),
) -> Result<T, E> {
    let Some(host) = host else {
        return charged(owner, None, run);
    };
    let account = CssAccount::host(host);
    if open_for(owner) {
        let value = run()?;
        let work = OPEN
            .with(|open| {
                open.replace(Some(Open {
                    owner,
                    work: OperationWork::default(),
                }))
            })
            .map(|record| record.work)
            .unwrap_or_default();
        settle(account, work, gauges(&value)).map_err(E::from)?;
        return Ok(value);
    }
    let _outer = SetAside::take();
    let record = Record::open(owner);
    let result = account
        .timed(CssFront::Cssom, run)
        .map_err(|fault| E::from(CssomError::Accounting(fault.into())))?;
    let changes = result.as_ref().map_or(NO_GAUGES, gauges);
    let charge = settle(account, record.close(), changes);
    let value = result?;
    charge.map_err(E::from)?;
    Ok(value)
}

/// Charges `work` and the gauge changes `gauges` to the `cssom` scope as one
/// batch.
fn settle(
    account: CssAccount<'_>,
    work: OperationWork,
    gauges: ([CssChange; 3], usize),
) -> Result<(), CssomError> {
    let (work, length) = work.changes();
    let mut changes = [CssChange::Work(CssWork::CensusRun, 0); 8];
    changes[..length].copy_from_slice(&work[..length]);
    changes[length..length + gauges.1].copy_from_slice(&gauges.0[..gauges.1]);
    account
        .charge(CssFront::Cssom, &changes[..length + gauges.1])
        .map_err(CssomError::Accounting)
}

/// The size of a published state in the units of the `cssom` gauges: its
/// objects, the state itself included, its entries and its host and live
/// string bytes, as its census counts them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Retained([u64; 3]);

impl Retained {
    const RESOURCES: [CssResource; 3] = [
        CssResource::RetainedObject,
        CssResource::RetainedEntry,
        CssResource::RetainedStringByte,
    ];

    pub(crate) fn new(objects: usize, entries: usize, strings: usize) -> Self {
        Self([objects, entries, strings].map(|count| u64::try_from(count).unwrap_or(u64::MAX)))
    }

    /// The changes that take a holding of `self` to `to`: the increases as
    /// acquisitions and the decreases as releases, without the equal ones.
    pub(crate) fn changes(self, to: Self, increases: bool) -> ([CssChange; 3], usize) {
        let mut changes = [CssChange::Work(CssWork::CensusRun, 0); 3];
        let mut length = 0;
        for ((resource, held), new) in Self::RESOURCES.into_iter().zip(self.0).zip(to.0) {
            let change = match (increases, new.checked_sub(held), held.checked_sub(new)) {
                (true, Some(more), _) if more > 0 => CssChange::Acquire(resource, more),
                (false, _, Some(less)) if less > 0 => CssChange::Release(resource, less),
                _ => continue,
            };
            changes[length] = change;
            length += 1;
        }
        (changes, length)
    }
}

/// The holding of a store in its host's `cssom` gauges as a commit changes
/// it: the increases acquired before the commit publishes, released again if
/// it never does.
pub(crate) struct Acquisition {
    host: Option<Arc<CssHostScope>>,
    held: Retained,
    to: Retained,
}

impl Acquisition {
    /// The holding of a commit whose preparation's batch acquired the
    /// increases from `held` to `to` below `host`.
    pub(crate) fn acquired(host: Option<&Arc<CssHostScope>>, held: Retained, to: Retained) -> Self {
        Self {
            host: host.cloned(),
            held,
            to,
        }
    }

    /// Releases the decreases, and sets `held` to the new holding.
    ///
    /// A release is never refused by a cap. One the ledger cannot apply, for
    /// a defect it has latched, is not reported: the ledger reports the defect
    /// on its next call.
    pub(crate) fn publish(mut self, held: &mut Retained) {
        // Taking the host hands the increases to the store's holding, so the
        // drop releases nothing.
        if let Some(host) = self.host.take() {
            let (changes, length) = self.held.changes(self.to, false);
            let _ = CssAccount::host(&host).charge(CssFront::Cssom, &changes[..length]);
        }
        *held = self.to;
    }
}

impl Drop for Acquisition {
    /// Releases the increases of a commit that never published.
    fn drop(&mut self) {
        if let Some(host) = &self.host {
            let (changes, length) = self.held.changes(self.to, true);
            let releases = changes.map(|change| match change {
                CssChange::Acquire(resource, amount) => CssChange::Release(resource, amount),
                other => other,
            });
            let _ = CssAccount::host(host).charge(CssFront::Cssom, &releases[..length]);
        }
    }
}

/// Releases everything a store holds in its host's gauges, when it drops.
pub(crate) fn release_all(host: Option<&CssHostScope>, held: Retained) {
    if let Some(host) = host {
        let (changes, length) = held.changes(Retained::default(), false);
        let _ = CssAccount::host(host).charge(CssFront::Cssom, &changes[..length]);
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support {
    pub(super) use super::tests::{
        SOURCE, guard, host, make, open_host, smallest, version, with_sheet,
    };
    pub(super) use crate::accounting::{CssChange, CssLedger, CssLimits, CssResourceTable};
    pub(super) use surgeist_ledger::ProfileMode;

    pub(super) fn constructed() -> crate::CssomSheetInputs {
        crate::CssomSheetInputs::constructed(version("document"), "document".into(), None)
    }
}
#[cfg(test)]
mod gauge_tests;
