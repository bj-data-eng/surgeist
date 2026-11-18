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
use surgeist_css::accounting::{CssAccount, CssChange, CssFront, CssHostScope, CssWork};

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
    let charge = settle(account, record.close());
    let value = result?;
    charge.map_err(E::from)?;
    Ok(value)
}

/// Runs the preparation of a commit of the store owned by `owner`, whose
/// host is `host`.
///
/// It is [`charged`], except that inside an open operation of the same store
/// a successful preparation charges the work recorded so far and empties the
/// record, before the commit publishes.
///
/// # Errors
///
/// The error of `run`, and [`CssomError::Accounting`] when the host does not
/// admit the charge; the prepared commit is then dropped unpublished.
pub(crate) fn charged_before_publication<T, E: From<CssomError>>(
    owner: usize,
    host: Option<&CssHostScope>,
    run: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let Some(host) = host.filter(|_| open_for(owner)) else {
        return charged(owner, host, run);
    };
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
    settle(CssAccount::host(host), work).map_err(E::from)?;
    Ok(value)
}

/// Charges `work` to the `cssom` scope as one batch.
fn settle(account: CssAccount<'_>, work: OperationWork) -> Result<(), CssomError> {
    let (changes, length) = work.changes();
    account
        .charge(CssFront::Cssom, &changes[..length])
        .map_err(CssomError::Accounting)
}

#[cfg(test)]
mod tests;
