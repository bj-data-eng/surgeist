//! Start the next authentic original subject, preserving existing async owners.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{DomParsedHtmlStyleSubjectCompletion, DomParsedHtmlStyleSubjectError};
/// The real Runtime owner or original subject transition refused.
#[derive(Debug)]
pub enum DomStyleParsedSubjectError {
    /// Session, idle state or finite Runtime preparation refused.
    Runtime(DomStyleRuntimeError),
    /// Original ordering, handoff, context or finite admission refused.
    Subject(DomParsedHtmlStyleSubjectError),
}
impl From<DomStyleRuntimeError> for DomStyleParsedSubjectError {
    fn from(e: DomStyleRuntimeError) -> Self {
        Self::Runtime(e)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedSubjectError {
    fn from(e: DomParsedHtmlStyleError) -> Self {
        Self::Subject(e.into())
    }
}
impl From<DomParsedHtmlStyleSubjectError> for DomStyleParsedSubjectError {
    fn from(e: DomParsedHtmlStyleSubjectError) -> Self {
        Self::Subject(e)
    }
}
impl std::fmt::Display for DomStyleParsedSubjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(e) => e.fmt(f),
            Self::Subject(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedSubjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(e) => Some(e),
            Self::Subject(e) => Some(e),
        }
    }
}
impl DomStyleRuntime {
    /// Start only the exact next original distinct Style insertion. Existing
    /// sheets, names, processing rows and tasks keep their actual owners.
    pub fn advance_parsed_style_subject(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        expected: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSubjectCompletion, DomStyleParsedSubjectError> {
        let owner = self.original()?;
        let state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let mut hold = dom
            .raw_mut()
            .begin_parsed_html_style(contribution, limits, cancel)?;
        let steps = Steps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut custody = Custody {
            bytes: size_of::<DomParsedHtmlStyleSubjectCompletion>(),
            shared: 0,
            units: 0,
            current: 0,
            maximum: state.limits.storage_bytes,
            outputs: 1,
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        custody.current = current_storage(&state, &mut hold, &custody, &steps)?;
        custody.check()?;
        steps.prefix(&hold)?;
        Ok(hold.advance_subject(expected, cancel)?)
    }
}
