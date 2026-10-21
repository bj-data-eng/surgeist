//! Whole original settlement joins canonical proof with actual Runtime tails.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{DomParsedHtmlContributionCompletion, DomParsedHtmlTerminalError};
/// Actual Session admission, canonical settlement or a retained same-contribution tail refused.
#[derive(Debug)]
pub enum DomStyleParsedTerminalError {
    /// Session ownership, idle state or finite Runtime admission refused.
    Runtime(DomStyleRuntimeError),
    /// Canonical source, End, readiness or obligation qualification refused.
    Terminal(DomParsedHtmlTerminalError),
    /// An actual same-contribution processing/delivery/cleanup row remains.
    TailPending {
        /// Exact contribution whose actual Runtime tail is retained.
        contribution: DomParsedHtmlAssociationId,
    },
}
impl From<DomStyleRuntimeError> for DomStyleParsedTerminalError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedTerminalError {
    fn from(error: DomParsedHtmlStyleError) -> Self {
        Self::Terminal(error.into())
    }
}
impl From<DomParsedHtmlTerminalError> for DomStyleParsedTerminalError {
    fn from(error: DomParsedHtmlTerminalError) -> Self {
        Self::Terminal(error)
    }
}
impl std::fmt::Display for DomStyleParsedTerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original contribution settlement refused: {self:?}")
    }
}
impl std::error::Error for DomStyleParsedTerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Terminal(error) => Some(error),
            _ => None,
        }
    }
}
impl DomStyleRuntime {
    /// Explicitly settle an already-ended no-import contribution after every
    /// real notification and Runtime tail is complete. This executes no End,
    /// resource or event work and changes no CSSOM or preference owner.
    pub fn complete_parsed_html_contribution(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlContributionCompletion, DomStyleParsedTerminalError> {
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
        // Lower owns the actual completion frame and its original output debit.
        // Runtime retains no second result or mirrored semantic receipt.
        let mut custody = Custody {
            bytes: 0,
            shared: 0,
            units: 0,
            current: 0,
            maximum: state.limits.storage_bytes,
            outputs: 0,
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        custody.current = current_storage(&state, &mut hold, &custody, &steps)?;
        custody.check()?;
        for completion in &state.completions {
            steps.prefix(&hold)?;
            let parsed = match completion {
                Completion::ParsedProcessing(record) => Some(record),
                Completion::Queued(task) | Completion::Waiting(task) => task.parsed.as_ref(),
            };
            if parsed.is_some_and(|record| record.qualification.contribution() == contribution) {
                return Err(DomStyleParsedTerminalError::TailPending { contribution });
            }
        }
        steps.prefix(&hold)?;
        Ok(hold.complete_contribution(cancel)?)
    }
}
