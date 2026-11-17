//! The work one accounted parse call counts.
//!
//! Token documents are built several calls below the parse fronts. An
//! accounted front collects for the length of its call: every document built
//! on its thread while it collects adds its metrics to the call's record,
//! which the front charges once when the call returns. The scoped parser
//! thread of deep input collects into a record of its own, which is added to
//! the caller's when the thread is joined.
//!
//! A plain front does not collect, so a built document finds no record and
//! adds nothing.

use std::cell::Cell;

use surgeist_css_parser::document::Metrics;

/// Sums over the token documents one accounted parse call built.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ParseWork {
    documents: u64,
    components: u64,
    spelling_bytes: u64,
}

impl ParseWork {
    fn add_document(&mut self, metrics: Metrics) {
        self.documents = self.documents.saturating_add(1);
        self.components = self.components.saturating_add(amount(metrics.components()));
        self.spelling_bytes = self
            .spelling_bytes
            .saturating_add(amount(metrics.spelling_bytes()));
    }

    const fn add(&mut self, other: Self) {
        self.documents = self.documents.saturating_add(other.documents);
        self.components = self.components.saturating_add(other.components);
        self.spelling_bytes = self.spelling_bytes.saturating_add(other.spelling_bytes);
    }
}

/// A count as a ledger amount. A count beyond `u64` is charged as `u64::MAX`.
fn amount(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

thread_local! {
    /// The record of the accounted call collecting on this thread, if any.
    static COLLECTING: Cell<Option<ParseWork>> = const { Cell::new(None) };
}

/// Whether an accounted call is collecting on this thread.
pub(crate) fn is_collecting() -> bool {
    COLLECTING
        .try_with(|record| record.get().is_some())
        .unwrap_or(false)
}

/// Adds a built document to the record of the collecting call, if any.
pub(crate) fn note(metrics: Metrics) {
    let _ = COLLECTING.try_with(|record| {
        if let Some(mut work) = record.get() {
            work.add_document(metrics);
            record.set(Some(work));
        }
    });
}

/// Adds the work a parser thread collected to the record of the collecting
/// call, if any.
pub(crate) fn add(collected: ParseWork) {
    let _ = COLLECTING.try_with(|record| {
        if let Some(mut work) = record.get() {
            work.add(collected);
            record.set(Some(work));
        }
    });
}

/// Runs `run` and returns it with the work of the documents it built.
///
/// The work is `None` when a call further out on this thread is already
/// collecting: the documents then count toward that call, which charges them.
pub(crate) fn collect<T>(run: impl FnOnce() -> T) -> (T, Option<ParseWork>) {
    if is_collecting() {
        return (run(), None);
    }
    let collecting = Collecting::begin();
    let value = run();
    (value, Some(collecting.finish()))
}

/// The collection of one call. Dropping it ends the collection, so a call
/// that unwinds leaves no record behind.
struct Collecting;

impl Collecting {
    fn begin() -> Self {
        let _ = COLLECTING.try_with(|record| record.set(Some(ParseWork::default())));
        Self
    }

    fn finish(self) -> ParseWork {
        COLLECTING
            .try_with(Cell::take)
            .ok()
            .flatten()
            .unwrap_or_default()
    }
}

impl Drop for Collecting {
    fn drop(&mut self) {
        let _ = COLLECTING.try_with(|record| record.set(None));
    }
}
