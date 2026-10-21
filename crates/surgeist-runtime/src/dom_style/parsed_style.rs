//! Original Style preparation and decisions; retained inputs and transitions belong to DOM/HTML.
use super::*;
use surgeist_dom::{
    DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits, DomParsedHtmlStyleError,
    DomParsedHtmlStyleInitialPolicyRequest, DomParsedHtmlStyleInlinePolicyObservation,
    DomParsedHtmlStylePhase, DomParsedHtmlStylePrefixView, DomParsedHtmlStyleRenderingProgress,
};
use surgeist_html::HtmlCspAssociation;
mod admission;
mod create_sheet;
mod notification;
mod successor;
pub use create_sheet::{DomStyleParsedSheetError, DomStyleParsedSheetOutcome};
pub use notification::{DomStyleParsedNotificationError, DomStyleParsedNotificationProgress};
pub use successor::DomStyleParsedSuccessorError;

/// The actual Session or original parsed Style prefix refused a command.
#[derive(Debug)]
pub enum DomStyleParsedPrefixError {
    /// Runtime owner, phase, cancellation or finite work refused.
    Runtime(DomStyleRuntimeError),
    /// Canonical original-input qualification, phase or admission refused.
    Style(DomParsedHtmlStyleError),
}
impl std::fmt::Display for DomStyleParsedPrefixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "parsed Style Runtime preparation refused: {error}"),
            Self::Style(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for DomStyleParsedPrefixError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Style(error) => Some(error),
        }
    }
}
impl From<DomStyleRuntimeError> for DomStyleParsedPrefixError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedPrefixError {
    fn from(error: DomParsedHtmlStyleError) -> Self {
        Self::Style(error)
    }
}

impl DomStyleRuntime {
    /// Process the retained original rendering actions using HTML's bounded
    /// media classification and original Document context. Unsupported context
    /// remains a precise requirement. A completion owns the actual DOM output
    /// and retirement custody; the whole contribution remains Pending.
    pub fn process_parsed_style_rendering(
        &self,
        dom: &mut DomEventDom<'_>,
        qualification: &DomParsedHtmlStyleInitialPolicyRequest,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleRenderingProgress, DomStyleParsedPrefixError> {
        let owner = self.original()?;
        let state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let mut hold =
            dom.raw_mut()
                .begin_parsed_html_style(qualification.contribution(), limits, cancel)?;
        hold.step(cancel).map_err(DomStyleRuntimeError::from)?;
        state.step(cancel)?;
        Ok(hold.process_rendering(qualification, cancel)?)
    }

    /// Advance the original first influencing Style only as far as its
    /// historical inline-policy request. No decision, sheet or contribution
    /// completion is produced. Original work and phase survive refusal/Drop.
    pub fn prepare_parsed_style(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomStyleParsedPrefixError> {
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
        hold.step(cancel).map_err(DomStyleRuntimeError::from)?;
        state.step(cancel)?;
        Ok(hold.advance(cancel)?)
    }

    /// Bind the exact explicitly unbound original initial-policy slot. Input
    /// remains in its envelope on refusal; accepted custody stays canonical.
    /// A subsequent preparation reaches the actual inline-policy request.
    pub fn bind_parsed_style_initial_policy(
        &self,
        dom: &mut DomEventDom<'_>,
        qualification: &DomParsedHtmlStyleInitialPolicyRequest,
        policy: &mut Option<HtmlCspAssociation>,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleParsedPrefixError> {
        let owner = self.original()?;
        let state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let mut hold =
            dom.raw_mut()
                .begin_parsed_html_style(qualification.contribution(), limits, cancel)?;
        hold.step(cancel).map_err(DomStyleRuntimeError::from)?;
        state.step(cancel)?;
        Ok(hold.bind_initial_policy(qualification, policy, cancel)?)
    }

    /// Accept one externally evaluated decision for the exact retained original
    /// request. Evaluate through the separate immutable Session query, then drop
    /// that view before calling this command. Refusal keeps the actual input;
    /// retry it without evaluating again. Neither outcome creates a sheet or
    /// acknowledges the whole contribution.
    pub fn accept_parsed_style_inline_policy(
        &self,
        dom: &mut DomEventDom<'_>,
        contribution: DomParsedHtmlAssociationId,
        input: &mut Option<DomParsedHtmlStyleInlinePolicyObservation>,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomStyleParsedPrefixError> {
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
        hold.step(cancel).map_err(DomStyleRuntimeError::from)?;
        state.step(cancel)?;
        Ok(hold.accept_inline_policy(input, cancel)?)
    }

    /// Exclude executable-JavaScript bookkeeping for this Session's actual
    /// no-engine execution profile. DOM qualifies the exact original occurrence
    /// and retained cursor; this is not a false script-blocking predicate.
    /// Success reaches only the independent render-context action and leaves
    /// the whole contribution Pending. Refused retries retain original work.
    pub fn exclude_parsed_style_script_bookkeeping(
        &self,
        dom: &mut DomEventDom<'_>,
        qualification: &DomParsedHtmlStyleInitialPolicyRequest,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomStyleParsedPrefixError> {
        let owner = self.original()?;
        let state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let mut hold =
            dom.raw_mut()
                .begin_parsed_html_style(qualification.contribution(), limits, cancel)?;
        hold.step(cancel).map_err(DomStyleRuntimeError::from)?;
        state.step(cancel)?;
        // DomDocumentEnvironment's admitted native owner supplies no scripting
        // engine. Rust listener dispatch does not add executable JavaScript.
        // DOM also checks the actual report/Document and stored profile.
        Ok(hold.exclude_script_bookkeeping(
            qualification,
            surgeist_html::HtmlStyleExecutionProfile::NoExecutableJavaScript,
            cancel,
        )?)
    }
}

impl DomSession {
    /// Borrow the actual canonical original slot/request after a preparation
    /// command has released Runtime custody. This read grants no reply or
    /// publication authority; drop the view before binding or advancing.
    pub fn parsed_html_style_prefix(
        &self,
        contribution: DomParsedHtmlAssociationId,
    ) -> Result<DomParsedHtmlStylePrefixView<'_>, DomParsedHtmlStyleError> {
        self.dom.parsed_html_style_prefix(contribution)
    }
}
