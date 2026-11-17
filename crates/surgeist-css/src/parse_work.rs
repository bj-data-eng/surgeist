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
//!
//! The workers of the component constructors (`query/when.rs`,
//! `query/supports/construction.rs`, `query/container/construction.rs` and
//! `rule/imports.rs`) start threads of their own, which do not hand a record
//! across. No parse front reaches them: their callers are the public
//! `try_from_components` constructors and normalization. A front that came to
//! collect across them would hand the record over as
//! `parser::bounded_execution` does.

use std::cell::Cell;

use surgeist_css_parser::accounting::{CssChange, CssWork};
use surgeist_css_parser::document::Metrics;

/// Sums over the token documents one accounted parse call built.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ParseWork {
    documents: u64,
    components: u64,
    spelling_bytes: u64,
}

impl ParseWork {
    /// The changes that charge this work to the parse front.
    pub(crate) const fn changes(self) -> [CssChange; 3] {
        [
            CssChange::Work(CssWork::Document, self.documents),
            CssChange::Work(CssWork::Component, self.components),
            CssChange::Work(CssWork::SpellingByte, self.spelling_bytes),
        ]
    }

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
pub(crate) fn amount(count: usize) -> u64 {
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

#[cfg(test)]
mod tests {
    //! The record of one collecting call. The expected sums are those of the
    //! documents each test builds: `a b` is three components (two idents and
    //! the whitespace between them) and three bytes.

    use std::panic::{AssertUnwindSafe, catch_unwind};

    use surgeist_css_parser::Document;
    use surgeist_css_parser::lexer::Options;
    use surgeist_css_parser::limits::{DepthExcess, Limits};

    use super::*;

    fn metrics(text: &str) -> Metrics {
        let document: Document<'_> = Document::parse(
            text,
            &Options::default(),
            Limits::default(),
            0,
            DepthExcess::Fail,
        )
        .unwrap();
        document.metrics()
    }

    fn work(documents: u64, components: u64, spelling_bytes: u64) -> ParseWork {
        ParseWork {
            documents,
            components,
            spelling_bytes,
        }
    }

    #[test]
    fn a_document_built_while_nothing_collects_is_not_recorded() {
        note(metrics("a b"));
        add(work(5, 5, 5));
        assert!(!is_collecting());
        let ((), collected) = collect(|| {});
        assert_eq!(collected, Some(work(0, 0, 0)));
    }

    #[test]
    fn a_collecting_call_sums_the_documents_built_during_it() {
        let (value, collected) = collect(|| {
            assert!(is_collecting());
            note(metrics("a b"));
            note(metrics("{}"));
            note(metrics(""));
            7
        });
        assert_eq!(value, 7);
        // `{}` is one component of two bytes; the empty text has none.
        assert_eq!(collected, Some(work(3, 4, 5)));
        assert!(!is_collecting());
    }

    #[test]
    fn a_call_inside_a_collecting_call_adds_to_it_and_returns_no_work() {
        let ((), outer) = collect(|| {
            note(metrics("a"));
            let ((), inner) = collect(|| note(metrics("a b")));
            assert_eq!(inner, None);
            assert!(is_collecting());
            note(metrics("b"));
        });
        assert_eq!(outer, Some(work(3, 5, 5)));
    }

    #[test]
    fn work_collected_elsewhere_joins_the_collecting_call() {
        let ((), collected) = collect(|| {
            note(metrics("a"));
            add(work(2, 10, 20));
        });
        assert_eq!(collected, Some(work(3, 11, 21)));
    }

    #[test]
    fn a_call_that_unwinds_leaves_no_record() {
        let unwound = catch_unwind(AssertUnwindSafe(|| {
            collect(|| {
                note(metrics("a"));
                std::panic::resume_unwind(Box::new("stop"));
            })
        }));
        assert!(unwound.is_err());
        assert!(!is_collecting());
        let ((), collected) = collect(|| note(metrics("a")));
        assert_eq!(collected, Some(work(1, 1, 1)));
    }

    #[test]
    fn the_work_charges_documents_components_and_spelling_bytes() {
        assert_eq!(
            work(1, 2, 3).changes(),
            [
                CssChange::Work(CssWork::Document, 1),
                CssChange::Work(CssWork::Component, 2),
                CssChange::Work(CssWork::SpellingByte, 3),
            ]
        );
    }
}
