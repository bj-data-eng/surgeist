//! The reader of the optional parse detail a request selected.

use crate::{
    DetailSet, HtmlParseReport,
    construction::{
        evidence::HtmlParseDiagnostics,
        history::FrozenHistory,
        pending_association::HtmlParseAssociationOccurrence,
        read::{
            HtmlParseAssociations, HtmlParseCause, HtmlParseHistory, HtmlParseLexicalOccurrence,
            HtmlParseLexicalPart, HtmlParseOmission,
        },
        report::{
            HtmlParseDialogMissing, HtmlParseHyperlink, HtmlParseImage, HtmlParseMetaMissing,
            HtmlParseStyle,
        },
    },
};
use std::{error::Error, fmt};

/// The detail a parse selected, read from its report.
///
/// A parse that selected none has no detail: [`HtmlParseReport::detail`] is
/// `None`. Each reader answers only for the kind the request selected.
#[derive(Clone, Copy)]
pub struct HtmlParseDetail<'a> {
    history: &'a FrozenHistory,
    selected: DetailSet,
}

/// The origin and cause graph was not recorded, because the request selected
/// none of the source map, the lexical records and the history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseCauseUnselected;

impl fmt::Display for HtmlParseCauseUnselected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the parse did not select the detail that records construction causes")
    }
}

impl Error for HtmlParseCauseUnselected {}

/// A value of a report that records the construction cause it was made under.
/// [`HtmlParseDetail::cause_of`] reads it.
pub trait HtmlParseCaused<'a>: sealed::Sealed {
    #[doc(hidden)]
    fn recorded_cause(&self) -> Option<HtmlParseCause<'a>>;
}

mod sealed {
    pub trait Sealed {}
}

macro_rules! caused {
    ($($subject:ident),* $(,)?) => {$(
        impl sealed::Sealed for $subject<'_> {}
        impl<'a> HtmlParseCaused<'a> for $subject<'a> {
            fn recorded_cause(&self) -> Option<HtmlParseCause<'a>> {
                Self::recorded_cause(self)
            }
        }
    )*};
}
caused!(
    HtmlParseAssociationOccurrence,
    HtmlParseDialogMissing,
    HtmlParseHyperlink,
    HtmlParseImage,
    HtmlParseMetaMissing,
    HtmlParseStyle,
);

impl<'a> HtmlParseDetail<'a> {
    pub(crate) fn new(report: &'a HtmlParseReport, selected: DetailSet) -> Self {
        Self {
            history: report.history(),
            selected,
        }
    }
    /// The kinds of detail the request selected.
    pub fn selected(&self) -> DetailSet {
        self.selected
    }
    /// The association inventory, when the request selected the source map or
    /// the lexical records. The source map holds the rows of retained fields;
    /// the lexical records hold the rows of lexical occurrences and omissions.
    pub fn associations(&self) -> Option<HtmlParseAssociations<'a>> {
        self.selected
            .intersects(DetailSet::SOURCE_MAP.union(DetailSet::LEXICAL))
            .then(|| HtmlParseAssociations::new(self.history))
    }
    /// The lexical occurrences and their parts, when the request selected them.
    pub fn lexical(&self) -> Option<HtmlParseLexical<'a>> {
        self.selected
            .contains(DetailSet::LEXICAL)
            .then_some(HtmlParseLexical {
                history: self.history,
            })
    }
    /// The omission records, when the request selected the lexical records.
    pub fn omissions(&self) -> Option<HtmlParseOmissions<'a>> {
        self.selected
            .contains(DetailSet::LEXICAL)
            .then_some(HtmlParseOmissions {
                history: self.history,
            })
    }
    /// The construction history, when the request selected it.
    pub fn history(&self) -> Option<HtmlParseHistory<'a>> {
        self.selected
            .contains(DetailSet::HISTORY)
            .then(|| HtmlParseHistory::new(self.history))
    }
    /// The parse-error diagnostics, when the request selected them.
    pub fn diagnostics(&self) -> Option<HtmlParseDiagnostics<'a>> {
        self.selected
            .contains(DetailSet::DIAGNOSTICS)
            .then(|| self.history.evidence().diagnostics())
    }
    /// The construction cause of `subject`, or `None` when it has none. The
    /// request must have selected the source map, the lexical records or the
    /// history, which record the causes.
    pub fn cause_of<S: HtmlParseCaused<'a>>(
        &self,
        subject: &S,
    ) -> Result<Option<HtmlParseCause<'a>>, HtmlParseCauseUnselected> {
        if !self.selected.records_graph() {
            return Err(HtmlParseCauseUnselected);
        }
        Ok(subject.recorded_cause())
    }
}

/// The lexical occurrences and parts of a parse that selected them.
#[derive(Clone, Copy)]
pub struct HtmlParseLexical<'a> {
    history: &'a FrozenHistory,
}

impl<'a> HtmlParseLexical<'a> {
    /// The number of lexical occurrences.
    pub fn len(&self) -> usize {
        self.history.associations().lexical_occurrences_len()
    }
    /// Whether the parse recorded no lexical occurrence.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The occurrence at `index`, or `None` past the last one.
    pub fn occurrence(&self, index: usize) -> Option<HtmlParseLexicalOccurrence<'a>> {
        self.history
            .associations()
            .lexical_occurrence(index)
            .map(|value| HtmlParseLexicalOccurrence::new(self.history, value))
    }
    /// The number of lexical parts.
    pub fn parts_len(&self) -> usize {
        self.history.associations().lexical_parts_len()
    }
    /// The part at `index`, or `None` past the last one.
    pub fn part(&self, index: usize) -> Option<HtmlParseLexicalPart<'a>> {
        self.history
            .associations()
            .lexical_part(index)
            .map(|value| HtmlParseLexicalPart::new(self.history, value))
    }
}

/// The omission records of a parse that selected the lexical records.
#[derive(Clone, Copy)]
pub struct HtmlParseOmissions<'a> {
    history: &'a FrozenHistory,
}

impl<'a> HtmlParseOmissions<'a> {
    /// The number of omissions.
    pub fn len(&self) -> usize {
        self.history.associations().omissions_len()
    }
    /// Whether the parse recorded no omission.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The omission at `index`, or `None` past the last one.
    pub fn get(&self, index: usize) -> Option<HtmlParseOmission<'a>> {
        self.history
            .associations()
            .omission(index)
            .map(HtmlParseOmission::new)
    }
}
