//! Authenticate the original synchronous end without completing async work.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{DomParsedHtmlFrontierError, DomParsedHtmlSynchronousEnd};

/// Session admission or actual original synchronous-end validation refused.
#[derive(Debug)]
pub enum DomStyleParsedFrontierError {
    /// Runtime owner, idle state or finite adapter admission refused.
    Runtime(DomStyleRuntimeError),
    /// The original owner refused admission or identified remaining execution.
    Frontier(DomParsedHtmlFrontierError),
}
impl From<DomStyleRuntimeError> for DomStyleParsedFrontierError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedFrontierError {
    fn from(error: DomParsedHtmlStyleError) -> Self {
        Self::Frontier(error.into())
    }
}
impl From<DomParsedHtmlFrontierError> for DomStyleParsedFrontierError {
    fn from(error: DomParsedHtmlFrontierError) -> Self {
        Self::Frontier(error)
    }
}
impl std::fmt::Display for DomStyleParsedFrontierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => error.fmt(f),
            Self::Frontier(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedFrontierError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Frontier(error) => Some(error),
        }
    }
}
impl DomStyleRuntime {
    /// Seal the authentic original synchronous execution after the lower owner
    /// validates the whole supported inventory. Async imports and notifications
    /// remain independent; this neither acknowledges nor removes the contribution.
    pub fn complete_parsed_html_synchronous_execution(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlSynchronousEnd, DomStyleParsedFrontierError> {
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
            bytes: size_of::<DomParsedHtmlSynchronousEnd>(),
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
        Ok(hold.complete_synchronous_execution(cancel)?)
    }
}
