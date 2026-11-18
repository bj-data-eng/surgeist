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

thread_local! {
    static OPEN: Cell<Option<OperationWork>> = const { Cell::new(None) };
}

/// Adds to the open record, if any.
fn add(note: impl FnOnce(&mut OperationWork)) {
    OPEN.with(|open| {
        if let Some(mut work) = open.get() {
            note(&mut work);
            open.set(Some(work));
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
    fn open() -> Self {
        OPEN.with(|open| open.set(Some(OperationWork::default())));
        Self
    }
    fn close(self) -> OperationWork {
        OPEN.with(Cell::take).unwrap_or_default()
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        OPEN.with(|open| open.set(None));
    }
}

/// Runs one operation of a store whose host is `host`.
///
/// With a host, the outermost operation on the thread runs inside one phase of
/// the host's `cssom` scope and then charges that scope once with the work the
/// record holds, whether or not the operation succeeded; an operation inside
/// another adds its work to the outer one's. With no host it only runs
/// `run`.
///
/// # Errors
///
/// The error of `run` when it fails. When it succeeds,
/// [`CssomError::Accounting`] when the host does not admit the charge, and
/// when the phase cannot open, in which case `run` has not been called.
pub(crate) fn charged<T, E: From<CssomError>>(
    host: Option<&CssHostScope>,
    run: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let Some(host) = host else {
        return run();
    };
    if OPEN.with(|open| open.get().is_some()) {
        return run();
    }
    let record = Record::open();
    let account = CssAccount::host(host);
    let result = account
        .timed(CssFront::Cssom, run)
        .map_err(|fault| E::from(CssomError::Accounting(fault.into())))?;
    let charge = settle(account, record.close());
    let value = result?;
    charge.map_err(E::from)?;
    Ok(value)
}

/// Runs the preparation of a commit of a store whose host is `host`.
///
/// It is [`charged`], except that inside an open operation a successful
/// preparation charges the work recorded so far and empties the record,
/// before the commit publishes.
///
/// # Errors
///
/// The error of `run`, and [`CssomError::Accounting`] when the host does not
/// admit the charge; the prepared commit is then dropped unpublished.
pub(crate) fn charged_before_publication<T, E: From<CssomError>>(
    host: Option<&CssHostScope>,
    run: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let Some(host) = host else {
        return run();
    };
    if OPEN.with(|open| open.get().is_none()) {
        return charged(Some(host), run);
    }
    let value = run()?;
    let work = OPEN
        .with(|open| open.replace(Some(OperationWork::default())))
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
