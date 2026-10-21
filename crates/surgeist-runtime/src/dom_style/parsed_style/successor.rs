//! Execute the next authentic original Style prefix without changing async custody.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{DomParsedHtmlStyleSuccessorCompletion, DomParsedHtmlStyleSuccessorError};

/// The actual Session or original successor refused before joint publication.
#[derive(Debug)]
pub enum DomStyleParsedSuccessorError {
    /// Runtime ownership, idle state or finite adapter admission refused.
    Runtime(DomStyleRuntimeError),
    /// Original order, input qualification or actual producer preparation refused.
    Successor(DomParsedHtmlStyleSuccessorError),
}
impl From<DomStyleRuntimeError> for DomStyleParsedSuccessorError {
    fn from(value: DomStyleRuntimeError) -> Self {
        Self::Runtime(value)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedSuccessorError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Successor(value.into())
    }
}
impl From<DomParsedHtmlStyleSuccessorError> for DomStyleParsedSuccessorError {
    fn from(value: DomParsedHtmlStyleSuccessorError) -> Self {
        Self::Successor(value)
    }
}
impl std::fmt::Display for DomStyleParsedSuccessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => error.fmt(f),
            Self::Successor(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedSuccessorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Successor(error) => Some(error),
        }
    }
}
impl DomStyleRuntime {
    /// Reach the next authentic ParserPopped inline-policy request, removing
    /// an actually created predecessor sheet when present. The typed removal
    /// view distinguishes genuine publications from known absence of a sheet.
    /// This supplies no whole-history or networking dispatch permission.
    pub fn advance_parsed_style_successor(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSuccessorCompletion, DomStyleParsedSuccessorError> {
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
            bytes: size_of::<DomParsedHtmlStyleSuccessorCompletion>()
                + size_of::<std::rc::Rc<DomStyleCssomRoot>>(),
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
        let document = hold.successor_owner_document(cancel)?;
        let mut cap = None;
        if let Some(document) = document {
            let mut selected = None;
            for root in &state.roots {
                steps.prefix(&hold)?;
                if root.node == document.node() {
                    selected = Some(root);
                    break;
                }
            }
            let root = selected.ok_or(DomStyleRuntimeError::SheetSetRootUnavailable {
                root: document.node(),
            })?;
            steps.prefix(&hold)?;
            cap = Some(root.bound()?.clone());
        }
        let mut prep = hold.prepare_successor(cap.as_deref(), cancel)?;
        prep.step(cancel).map_err(DomStyleRuntimeError::from)?;
        steps.runtime()?;
        prep.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        steps.runtime()?;
        let published = prep.commit(cancel)?;
        // There is no Runtime semantic transition for removal. Names, serials
        // and all previous processing/task rows stay in their existing owners.
        let (completion, retirement) = published.into_parts();
        drop(retirement);
        drop(cap);
        drop(hold);
        Ok(completion)
    }
}
