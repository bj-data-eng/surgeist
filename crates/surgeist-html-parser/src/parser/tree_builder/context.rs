//! The actual §13.4 bootstrap and return owner. The capture is immutable
//! evidence; the internal root, parser Document and output are real records.

use super::*;
use crate::construction::RootKind;
use crate::parser::tokenizer::{
    Tokenizer,
    states::{Raw, State},
};
use crate::{HtmlCapturedNodeKind, HtmlFragmentParseRequest, HtmlParserScriptingMode};
use head::{HeadStage, HeadStep, InsertionMode};
use prologue::BeforeHead;

pub(crate) mod ownership;

pub(crate) enum FragmentExecutionError {
    Stop(ParseStop),
    Kernel(KernelError),
    Head(head::HeadError),
    Consumed,
}
impl std::fmt::Debug for FragmentExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop(error) => f.debug_tuple("Stop").field(error).finish(),
            Self::Kernel(error) => f.debug_tuple("Kernel").field(error).finish(),
            Self::Head(error) => f.debug_tuple("Head").field(error).finish(),
            Self::Consumed => f.write_str("Consumed"),
        }
    }
}
impl From<ParseStop> for FragmentExecutionError {
    fn from(value: ParseStop) -> Self {
        Self::Stop(value)
    }
}
impl From<SessionError> for FragmentExecutionError {
    fn from(value: SessionError) -> Self {
        Self::Kernel(value.into())
    }
}
impl From<KernelError> for FragmentExecutionError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}
impl From<head::HeadError> for FragmentExecutionError {
    fn from(value: head::HeadError) -> Self {
        Self::Head(value)
    }
}

pub(crate) struct ContextFragmentDriver<'source> {
    request: &'source HtmlFragmentParseRequest,
    control: ParseControl,
    head: Option<HeadStage<'source>>,
    failed_evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
    failure_usage: Option<super::super::control::ParseUsage>,
}

/// Completion keeps the actual fragment execution owner intact. It neither
/// publishes a Document report nor drains tasks during immutable observation.
pub(crate) struct CompletedContextFragment<'source> {
    driver: Option<ContextFragmentDriver<'source>>,
    frozen: Option<crate::construction::FrozenArena>,
}
/// This adapter borrows qualified captured strings and uses the same charged
/// comparisons as actual stack dispatch. It does not precharge an unpolled scan
/// or derive namespace from a local-name vocabulary.
fn initial_modes(
    request: &HtmlFragmentParseRequest,
    c: &mut ParseControl,
) -> Result<(State, InsertionMode, bool), FragmentExecutionError> {
    c.require_source(request.source())?;
    c.charge(WorkCategory::RecordVisit, 2)?;
    let context = request.context();
    let node = context
        .capture()
        .node(context.context_element().subject())
        .map_err(|_| SessionError::ContextUnavailable)?;
    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
        return Err(SessionError::ContextUnavailable.into());
    };
    let name = element.name();
    use crate::bounded::HtmlFragmentNameTest;
    let (lexical, insertion) = crate::bounded::fragment_initial_modes(
        request.effective_scripting_mode(),
        |test| match test {
            HtmlFragmentNameTest::HtmlNamespace => {
                tag_sets::namespace(name, "http://www.w3.org/1999/xhtml", c)
            }
            HtmlFragmentNameTest::Local(choices) => tag_sets::local_in(name, choices, c),
        },
    )?;
    let template = insertion == crate::HtmlFragmentInsertionMode::InTemplate;
    let lexical = match lexical {
        crate::HtmlFragmentTokenizerState::Data => State::Data,
        crate::HtmlFragmentTokenizerState::Rcdata => State::Raw(Raw::Rcdata),
        crate::HtmlFragmentTokenizerState::Rawtext => State::Raw(Raw::Rawtext),
        crate::HtmlFragmentTokenizerState::ScriptData => State::Raw(Raw::Script),
        crate::HtmlFragmentTokenizerState::Plaintext => State::Plaintext,
    };
    Ok((lexical, parser_insertion_mode(insertion), template))
}

/// Parser-private representation adapter; the HTML rule owner has no dependency
/// on tree-builder state or its controller.
pub(super) fn parser_insertion_mode(mode: crate::HtmlFragmentInsertionMode) -> InsertionMode {
    use crate::HtmlFragmentInsertionMode as Mode;
    match mode {
        Mode::InBody => InsertionMode::InBody,
        Mode::InRow => InsertionMode::InRow,
        Mode::InTableBody => InsertionMode::InTableBody,
        Mode::InCaption => InsertionMode::InCaption,
        Mode::InColumnGroup => InsertionMode::InColumnGroup,
        Mode::InTable => InsertionMode::InTable,
        Mode::InTemplate => InsertionMode::InTemplate,
        Mode::InFrameset => InsertionMode::InFrameset,
        Mode::BeforeHead => InsertionMode::BeforeHead,
    }
}

impl<'source> ContextFragmentDriver<'source> {
    pub(super) fn stop_for_public_failure(&mut self) {
        self.control
            .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
    }
    pub(crate) fn take_completed(
        input: &mut Option<Self>,
    ) -> Result<CompletedContextFragment<'source>, FragmentExecutionError> {
        let driver = input.as_mut().ok_or(FragmentExecutionError::Consumed)?;
        driver.control.require_source(driver.request.source())?;
        let state = driver
            .head
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .state
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        write(
            size_of::<CompletedContextFragment>() + size_of::<Self>() + size_of::<Option<Self>>(),
            2,
            &mut driver.control,
        )?;
        Ok(CompletedContextFragment {
            driver: input.take(),
            frozen: None,
        })
    }
    /// Actual fixture-only borrow; no new public mutation or entry capability.
    #[cfg(test)]
    pub(crate) fn input_fixture_parts(
        &mut self,
    ) -> Option<(
        &mut ParseControl,
        &mut ParserArenaSession,
        &super::ConstructionCapability<'source>,
    )> {
        let (session, capability) = self.head.as_mut()?.standalone_parts()?;
        Some((&mut self.control, session, capability))
    }
    pub(crate) fn step(&mut self) -> Result<HeadStep, FragmentExecutionError> {
        self.control.require_source(self.request.source())?;
        let result = self
            .head
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?
            .step(&mut self.control);
        match result {
            Ok(HeadStep::Boundary(head::HeadBoundary::AfterHead)) => {
                let state = self
                    .head
                    .as_mut()
                    .expect("active driver")
                    .state
                    .as_mut()
                    .ok_or(FragmentExecutionError::Consumed)?;
                write(
                    size_of::<Option<head::HeadBoundary>>(),
                    1,
                    &mut self.control,
                )?;
                state.boundary = None;
                Ok(HeadStep::Consumed)
            }
            Err(error) => {
                self.control
                    .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
                Err(error.into())
            }
            Ok(step) => Ok(step),
        }
    }
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        self.head
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?
            .release(&mut self.control)?;
        self.head = None;
        Ok(())
    }
    /// A qualified fragment root and borrowed canonical view, only after the
    /// real EOF owner has completed. The native view's Document root is P;
    /// this explicit root is F, and never impersonates the captured target.
    #[cfg(test)]
    pub(crate) fn completed_native_view(
        &mut self,
    ) -> Result<(ParserNode, crate::construction::NativeView<'_>), FragmentExecutionError> {
        self.control.require_source(self.request.source())?;
        let state = self
            .head
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .state
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        state.qualify_completed(&mut self.control)?;
        let root = state
            .before
            .kernel
            .session
            .fragment_output(&mut self.control)?;
        write(
            size_of::<(ParserNode, crate::construction::NativeView)>(),
            2,
            &mut self.control,
        )?;
        let view = state.before.kernel.session.native_view(&mut self.control)?;
        Ok((root, view))
    }
}

impl<'source> ContextFragmentDriver<'source> {
    pub(crate) fn take_failed(
        input: &mut Option<Self>,
    ) -> Result<FailedContextFragment<'source>, FragmentExecutionError> {
        let driver = input.as_mut().ok_or(FragmentExecutionError::Consumed)?;
        let stop = driver
            .control
            .stop()
            .ok_or(FragmentExecutionError::Consumed)?;
        if driver.failure_usage.is_none() {
            driver.failure_usage = Some(
                driver
                    .control
                    .first_stop_usage()
                    .expect("sole first stop")
                    .clone(),
            );
            driver.failed_evidence = driver
                .head
                .as_mut()
                .ok_or(FragmentExecutionError::Consumed)?
                .state
                .as_mut()
                .ok_or(FragmentExecutionError::Consumed)?
                .before
                .kernel
                .session
                .take_failed_evidence(&mut driver.control)?;
        }
        driver.release()?;
        let driver = input.take().expect("reclaimed actual fragment execution");
        Ok(FailedContextFragment {
            request: driver.request,
            stop,
            at_stop: driver.failure_usage.expect("retained first stop snapshot"),
            after_cleanup: driver.control.usage().clone(),
            control: driver.control,
            evidence: driver.failed_evidence,
        })
    }
}

pub(crate) struct FailedContextFragment<'source> {
    request: &'source HtmlFragmentParseRequest,
    control: ParseControl,
    stop: ParseStop,
    at_stop: super::super::control::ParseUsage,
    after_cleanup: super::super::control::ParseUsage,
    evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
}
impl FailedContextFragment<'_> {
    #[cfg(test)]
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    #[cfg(test)]
    pub(crate) fn context(&self) -> &crate::HtmlFragmentContext {
        self.request.context()
    }
    #[cfg(test)]
    pub(crate) fn stop(&self) -> ParseStop {
        self.stop
    }
    #[cfg(test)]
    pub(crate) fn at_stop(&self) -> &super::super::control::ParseUsage {
        &self.at_stop
    }
    #[cfg(test)]
    pub(crate) fn after_cleanup(&self) -> &super::super::control::ParseUsage {
        &self.after_cleanup
    }
    #[cfg(test)]
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.evidence
            .as_ref()
            .map(|evidence| evidence.diagnostics())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        if let Some(evidence) = self.evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

/// Lifetime-free F construction. Captured dependencies and completed attachment
/// effects remain in the same frozen session; no live DOM operation is executed.
pub(crate) struct FrozenContextFragmentParts {
    pub(super) control: ParseControl,
    pub(super) arena: Option<crate::construction::FrozenArena>,
    source: (crate::HtmlSourceId, crate::HtmlSourceCoordinates, usize),
    final_usage: super::super::control::ParseUsage,
    requested: HtmlParserScriptingMode,
    effective: HtmlParserScriptingMode,
    allow_shadow: bool,
    failed_evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
    after_cleanup: Option<super::super::control::ParseUsage>,
}
impl FrozenContextFragmentParts {
    pub(super) fn at_stop(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.control.first_stop_usage()
    }
    pub(super) fn after_cleanup(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.after_cleanup.as_ref()
    }
    pub(super) fn prepare_report_request(
        &mut self,
        request: &crate::request::FragmentRequestParts,
    ) -> Result<(), SessionError> {
        self.control.require_source(request.source())?;
        self.control.charge(WorkCategory::ProviderCall, 1)?;
        self.control.charge(WorkCategory::RecordVisit, 5)?;
        self.control.charge(
            WorkCategory::InspectByte,
            (2 * size_of::<crate::HtmlSourceId>()
                + 2 * size_of::<crate::HtmlSourceCoordinates>()
                + 2 * size_of::<usize>()
                + 2 * size_of::<crate::HtmlParseCancellation>()) as u64,
        )?;
        if !self.control.owns_request(request.cancellation())
            || self.source_binding()
                != (
                    request.source().identity(),
                    request.source().coordinates(),
                    request.source().len(),
                )
            || self.failed_evidence.is_some()
            || self.after_cleanup.is_some()
            || self.arena.as_ref().and_then(|a| a.fragment()).is_none()
        {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    fn prepare_seal(
        &mut self,
        request: &HtmlFragmentParseRequest,
    ) -> Result<(), FragmentExecutionError> {
        self.control.charge(WorkCategory::ProviderCall, 1)?;
        if !self.control.owns_request(request.cancellation()) {
            return Err(FragmentExecutionError::Consumed);
        }
        self.control.require_source(request.source())?;
        self.control.charge(WorkCategory::RecordVisit, 11)?;
        self.control.charge(
            WorkCategory::InspectByte,
            (2 * size_of::<crate::HtmlFragmentContext>()
                + 2 * size_of::<crate::HtmlSourceId>()
                + 2 * size_of::<crate::HtmlSourceCoordinates>()
                + 2 * size_of::<usize>()
                + 4 * size_of::<HtmlParserScriptingMode>()
                + 2 * size_of::<bool>()) as u64,
        )?;
        let arena = self
            .arena
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        let actual = arena.frozen_fragment_context()?;
        let expected = request.context();
        if self.failed_evidence.is_some()
            || self.after_cleanup.is_some()
            || arena.fragment().is_none()
            || self.source_binding()
                != (
                    request.source().identity(),
                    request.source().coordinates(),
                    request.source().len(),
                )
            || actual.capture().identity() != expected.capture().identity()
            || actual.capture().host_facts_revision() != expected.capture().host_facts_revision()
            || actual.target() != expected.target()
            || actual.context_element() != expected.context_element()
            || actual.target_document() != expected.target_document()
            || actual.context_document() != expected.context_document()
            || actual.nearest_form() != expected.nearest_form()
            || self.requested != request.requested_scripting_mode()
            || self.effective != request.effective_scripting_mode()
            || self.allow_shadow != request.allow_declarative_shadow_roots()
        {
            return Err(FragmentExecutionError::Consumed);
        }
        arena.retained_evidence()?;
        arena.prepare_fragment_effects(&mut self.control)?;
        write(
            size_of::<ownership::SealedFragmentConstruction>()
                + size_of::<crate::request::FragmentRequestParts>()
                + size_of::<Option<HtmlFragmentParseRequest>>()
                + size_of::<Option<Self>>()
                + size_of::<super::super::control::ParseUsage>(),
            12,
            &mut self.control,
        )?;
        self.control.checkpoint()?;
        self.final_usage = self.control.usage().clone();
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn fragment(&mut self) -> Result<&crate::HtmlFragment, FragmentExecutionError> {
        self.control.checkpoint()?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.arena
            .as_ref()
            .and_then(|arena| arena.fragment())
            .ok_or(FragmentExecutionError::Consumed)
    }
    #[cfg(test)]
    pub(crate) fn context(&self) -> Result<&crate::HtmlFragmentContext, FragmentExecutionError> {
        Ok(self
            .arena
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .frozen_fragment_context()?)
    }
    pub(crate) fn source_binding(
        &self,
    ) -> (&crate::HtmlSourceId, crate::HtmlSourceCoordinates, usize) {
        (&self.source.0, self.source.1, self.source.2)
    }
    #[cfg(test)]
    pub(crate) fn final_usage(&self) -> &super::super::control::ParseUsage {
        &self.final_usage
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Result<crate::construction::evidence::DiagnosticEvidenceView<'_>, FragmentExecutionError>
    {
        if let Some(evidence) = &self.failed_evidence {
            return Ok(evidence.diagnostics());
        }
        Ok(self
            .arena
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .retained_evidence()?
            .diagnostics())
    }
    pub(crate) fn cleanup_failure(&mut self) -> Result<(), FragmentExecutionError> {
        if self.control.stop().is_none() {
            return Err(FragmentExecutionError::Consumed);
        }
        if self.after_cleanup.is_some() {
            return Ok(());
        }
        if let Some(arena) = &mut self.arena {
            if self.failed_evidence.is_none() {
                self.failed_evidence = arena.take_stopped_evidence(&mut self.control)?;
            }
            arena.release(&mut self.control)?;
            self.arena = None;
        }
        self.after_cleanup = Some(self.control.usage().clone());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        if self.control.stop().is_some() {
            self.cleanup_failure()?;
        } else if let Some(arena) = &mut self.arena {
            arena.release(&mut self.control)?;
            self.arena = None;
            self.after_cleanup = Some(self.control.usage().clone());
        }
        if let Some(evidence) = self.failed_evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

impl<'source> CompletedContextFragment<'source> {
    pub(super) fn stop_for_public_failure(&mut self) {
        if let Some(d) = &mut self.driver {
            d.stop_for_public_failure();
        }
    }
    pub(crate) fn take_failed(
        &mut self,
    ) -> Result<FailedContextFragment<'source>, FragmentExecutionError> {
        if self.frozen.is_none() {
            return ContextFragmentDriver::take_failed(&mut self.driver);
        }
        let driver = self
            .driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        let stop = driver
            .control
            .stop()
            .ok_or(FragmentExecutionError::Consumed)?;
        if driver.failure_usage.is_none() {
            driver.failure_usage = Some(
                driver
                    .control
                    .first_stop_usage()
                    .expect("sole first stop")
                    .clone(),
            );
            driver.failed_evidence = self
                .frozen
                .as_mut()
                .expect("retained frozen arena")
                .take_stopped_evidence(&mut driver.control)?;
        }
        if let Some(head) = &mut driver.head {
            head.release_frozen_execution(&mut driver.control)?;
            driver.head = None;
        }
        self.frozen
            .as_mut()
            .expect("retained frozen arena")
            .release(&mut driver.control)?;
        self.frozen = None;
        let driver = self
            .driver
            .take()
            .expect("all construction owners reclaimed");
        Ok(FailedContextFragment {
            request: driver.request,
            stop,
            at_stop: driver.failure_usage.expect("retained first stop"),
            after_cleanup: driver.control.usage().clone(),
            control: driver.control,
            evidence: driver.failed_evidence,
        })
    }
    pub(crate) fn prepare_native_associations(&mut self) -> Result<(), FragmentExecutionError> {
        let driver = self
            .driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        driver.control.require_source(driver.request.source())?;
        let state = driver
            .head
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?
            .state
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        state
            .before
            .kernel
            .session
            .prepare_native_associations(&mut driver.control, driver.request.source())?;
        Ok(())
    }
    pub(crate) fn freeze_parts(
        &mut self,
    ) -> Result<FrozenContextFragmentParts, FragmentExecutionError> {
        if self.frozen.is_some() {
            return Err(self
                .driver
                .as_ref()
                .and_then(|driver| driver.control.stop())
                .map_or(
                    FragmentExecutionError::Consumed,
                    FragmentExecutionError::Stop,
                ));
        }
        let driver = self
            .driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        driver.control.require_source(driver.request.source())?;
        let state = driver
            .head
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .state
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        state
            .before
            .kernel
            .session
            .preflight_completed_freeze(&mut driver.control)?;
        write(
            size_of::<FrozenContextFragmentParts>()
                + size_of::<Option<crate::construction::FrozenArena>>()
                + size_of::<Option<ContextFragmentDriver>>()
                + size_of::<ParseControl>()
                + ownership::frozen_failure_layout().0,
            13 + ownership::frozen_failure_layout().1,
            &mut driver.control,
        )?;
        driver.control.charge(WorkCategory::RecordVisit, 3)?;
        driver.control.charge(WorkCategory::ProviderCall, 1)?;
        let source = (
            driver.request.source().identity().clone(),
            driver.request.source().coordinates(),
            driver.request.source().len(),
        );
        let arena = driver
            .head
            .as_mut()
            .expect("qualified head")
            .state
            .as_mut()
            .expect("qualified state")
            .before
            .kernel
            .session
            .freeze(&mut driver.control)?;
        self.frozen = Some(arena);
        driver
            .head
            .as_mut()
            .expect("retained execution")
            .release_frozen_execution(&mut driver.control)?;
        driver.head = None;
        driver.control.checkpoint()?;
        let requested = driver.request.requested_scripting_mode();
        let effective = driver.request.effective_scripting_mode();
        let allow_shadow = driver.request.allow_declarative_shadow_roots();
        let driver = self
            .driver
            .take()
            .expect("reclaimed execution retains controller");
        Ok(FrozenContextFragmentParts {
            final_usage: driver.control.usage().clone(),
            control: driver.control,
            arena: self.frozen.take(),
            source,
            requested,
            effective,
            allow_shadow,
            failed_evidence: None,
            after_cleanup: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn native_view(
        &mut self,
    ) -> Result<(ParserNode, crate::construction::NativeView<'_>), FragmentExecutionError> {
        self.driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?
            .completed_native_view()
    }
    #[cfg(test)]
    pub(crate) fn context(&self) -> Result<&crate::HtmlFragmentContext, FragmentExecutionError> {
        Ok(self
            .driver
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .request
            .context())
    }
    #[cfg(test)]
    pub(crate) fn diagnostics(
        &self,
    ) -> Result<&crate::construction::evidence::ParseEvidence, FragmentExecutionError> {
        let driver = self
            .driver
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        let state = driver
            .head
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .state
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?;
        Ok(state.before.kernel.session.retained_evidence()?)
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        let driver = self
            .driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        if driver.control.stop().is_none() {
            write(
                size_of::<Option<ContextFragmentDriver>>(),
                1,
                &mut driver.control,
            )?;
        }
        if let Some(arena) = &mut self.frozen {
            if let Some(head) = &mut driver.head {
                head.release_frozen_execution(&mut driver.control)?;
                driver.head = None;
            }
            arena.release(&mut driver.control)?;
            self.frozen = None;
        } else {
            driver.release()?;
        }
        self.driver = None;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn initialized_driver(request: &HtmlFragmentParseRequest) -> ContextFragmentDriver<'_> {
    let mut entry =
        ownership::FragmentEntry::new(request, super::parse::InitializationTarget::Native);
    match entry.take_driver() {
        Ok(driver) => driver,
        Err(_) => {
            let error = format!("{:?}", entry.reason());
            entry
                .release_failure()
                .expect("actual fragment fixture cleanup");
            panic!("fragment initialization failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::native_view::NativeKind;
    use crate::*;

    pub(super) fn request(
        source: &str,
        context_name: &str,
        scripting: HtmlScriptingStatus,
    ) -> HtmlFragmentParseRequest {
        let mut capture = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = capture
            .add_document(
                HtmlDocumentMode::LimitedQuirks,
                scripting,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = capture
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, context_name),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture.establish_root(&context).unwrap();
        let capture = capture.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(source),
            context,
            super::super::tests::limits(100),
        )
        .unwrap()
    }
    pub(super) fn complete(driver: &mut ContextFragmentDriver<'_>) {
        for _ in 0..1000 {
            match driver.step().unwrap() {
                HeadStep::NeedInput => panic!("complete fragment cannot starve"),
                HeadStep::Consumed => {}
                HeadStep::Complete => return,
                HeadStep::Boundary(boundary) => panic!("unexpected real boundary {boundary:?}"),
            }
        }
        panic!("finite literal did not complete");
    }
    fn initialization_context(
        namespace: Namespace,
        local: impl Into<DocumentString>,
        scripting: HtmlScriptingStatus,
    ) -> HtmlFragmentContext {
        let mut capture = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = capture
            .add_document(
                HtmlDocumentMode::NoQuirks,
                scripting,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let element = capture
            .add_element(
                &document,
                QualifiedName::new(namespace, None, local),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture.establish_root(&element).unwrap();
        let capture = capture.finish().unwrap();
        HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap()
    }

    #[test]
    fn initialization_context_table_has_pinned_tokenizer_and_insertion_modes() {
        use HtmlFragmentInsertionMode as I;
        use HtmlFragmentTokenizerState as T;
        // HTML 877d §13.4 tokenizer selection and reset's last-node rules.
        // In particular last-node td/th/head are InBody, not InCell/InHead.
        for (local, lexical, insertion) in [
            ("title", T::Rcdata, I::InBody),
            ("textarea", T::Rcdata, I::InBody),
            ("style", T::Rawtext, I::InBody),
            ("xmp", T::Rawtext, I::InBody),
            ("iframe", T::Rawtext, I::InBody),
            ("noembed", T::Rawtext, I::InBody),
            ("noframes", T::Rawtext, I::InBody),
            ("noscript", T::Rawtext, I::InBody),
            ("script", T::ScriptData, I::InBody),
            ("plaintext", T::Plaintext, I::InBody),
            ("tr", T::Data, I::InRow),
            ("tbody", T::Data, I::InTableBody),
            ("thead", T::Data, I::InTableBody),
            ("tfoot", T::Data, I::InTableBody),
            ("caption", T::Data, I::InCaption),
            ("colgroup", T::Data, I::InColumnGroup),
            ("table", T::Data, I::InTable),
            ("template", T::Data, I::InTemplate),
            ("frameset", T::Data, I::InFrameset),
            ("html", T::Data, I::BeforeHead),
            ("td", T::Data, I::InBody),
            ("th", T::Data, I::InBody),
            ("head", T::Data, I::InBody),
            ("body", T::Data, I::InBody),
            ("select", T::Data, I::InBody),
            ("TITLE", T::Data, I::InBody),
            ("unknown", T::Data, I::InBody),
        ] {
            for namespace in [
                Namespace::html(),
                Namespace::svg(),
                Namespace::mathml(),
                Namespace::none(),
                Namespace::from_uri("urn:context"),
            ] {
                for scripting in [HtmlScriptingStatus::Disabled, HtmlScriptingStatus::Enabled] {
                    let html = namespace == Namespace::html();
                    let lexical = if !html
                        || (local == "noscript" && scripting == HtmlScriptingStatus::Disabled)
                    {
                        T::Data
                    } else {
                        lexical
                    };
                    let insertion = if html { insertion } else { I::InBody };
                    let context = initialization_context(namespace.clone(), local, scripting);
                    assert_eq!(
                        context
                            .initial_tokenizer_state(HtmlParserScriptingMode::Inert)
                            .unwrap(),
                        lexical,
                        "{local} {namespace:?} {scripting:?}"
                    );
                    assert_eq!(
                        context.initial_insertion_mode(),
                        insertion,
                        "{local} {namespace:?}"
                    );
                    let request = HtmlFragmentParseRequest::new(
                        HtmlSource::utf8(""),
                        context,
                        super::super::tests::limits(100),
                    )
                    .unwrap();
                    let mut c =
                        ParseControl::new(*request.limits(), request.cancellation().clone());
                    c.admit_source(request.source()).unwrap();
                    let (actual_lexical, actual_insertion, template) =
                        initial_modes(&request, &mut c).unwrap();
                    let expected_lexical = match lexical {
                        T::Data => State::Data,
                        T::Rcdata => State::Raw(Raw::Rcdata),
                        T::Rawtext => State::Raw(Raw::Rawtext),
                        T::ScriptData => State::Raw(Raw::Script),
                        T::Plaintext => State::Plaintext,
                    };
                    let expected_insertion = match insertion {
                        I::InBody => InsertionMode::InBody,
                        I::InRow => InsertionMode::InRow,
                        I::InTableBody => InsertionMode::InTableBody,
                        I::InCaption => InsertionMode::InCaption,
                        I::InColumnGroup => InsertionMode::InColumnGroup,
                        I::InTable => InsertionMode::InTable,
                        I::InTemplate => InsertionMode::InTemplate,
                        I::InFrameset => InsertionMode::InFrameset,
                        I::BeforeHead => InsertionMode::BeforeHead,
                    };
                    assert_eq!(
                        (actual_lexical, actual_insertion, template),
                        (
                            expected_lexical,
                            expected_insertion,
                            insertion == I::InTemplate
                        ),
                        "{local} {namespace:?} {scripting:?}"
                    );
                    super::super::tests::assert_released(&c);
                }
            }
        }
        let context = initialization_context(
            Namespace::html(),
            DocumentString::from_utf16(vec![0xd800]),
            HtmlScriptingStatus::Disabled,
        );
        assert_eq!(
            context
                .initial_tokenizer_state(HtmlParserScriptingMode::Inert)
                .unwrap(),
            T::Data
        );
        assert_eq!(context.initial_insertion_mode(), I::InBody);
    }

    #[test]
    fn initialization_context_rejects_missing_scripting_invalid_modes_and_foreign_targets() {
        let missing = initialization_context(
            Namespace::html(),
            "noscript",
            HtmlScriptingStatus::Unavailable,
        );
        assert!(matches!(
            missing.initial_tokenizer_state(HtmlParserScriptingMode::Inert),
            Err(HtmlCaptureError::MissingFact {
                fact: HtmlCaptureFact::ScriptingStatus,
                ..
            })
        ));
        // The insertion-only helper does not read an unavailable scripting fact.
        assert_eq!(
            missing.initial_insertion_mode(),
            HtmlFragmentInsertionMode::InBody
        );
        for mode in [
            HtmlParserScriptingMode::Normal,
            HtmlParserScriptingMode::Disabled,
        ] {
            assert_eq!(
                missing.initial_tokenizer_state(mode),
                Err(HtmlCaptureError::InvalidFragmentScriptingMode(mode))
            );
        }
        let enabled =
            initialization_context(Namespace::html(), "noscript", HtmlScriptingStatus::Enabled);
        assert_eq!(
            enabled.initial_tokenizer_state(HtmlParserScriptingMode::Fragment),
            Err(HtmlCaptureError::UnsupportedExecution(
                HtmlParserScriptingMode::Fragment
            ))
        );
        let disabled =
            initialization_context(Namespace::html(), "noscript", HtmlScriptingStatus::Disabled);
        assert_eq!(
            disabled
                .initial_tokenizer_state(HtmlParserScriptingMode::Fragment)
                .unwrap(),
            HtmlFragmentTokenizerState::Data
        );
        assert!(
            HtmlFragmentContext::element(enabled.capture(), disabled.context_element().subject())
                .is_err()
        );
        assert!(
            HtmlFragmentContext::element(enabled.capture(), enabled.context_document().subject())
                .is_err()
        );
    }

    #[test]
    fn initialization_context_one_short_and_cancel_stop_without_allocating_parser_records() {
        let request = request("", "template", HtmlScriptingStatus::Disabled);
        let mut measured = ParseControl::new(*request.limits(), request.cancellation().clone());
        measured.admit_source(request.source()).unwrap();
        initial_modes(&request, &mut measured).unwrap();
        let needed = measured
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let limits = request
            .limits()
            .into_builder()
            .work_credits(needed - 1)
            .finish()
            .unwrap();
        let mut short = ParseControl::new(limits, request.cancellation().clone());
        short.admit_source(request.source()).unwrap();
        assert!(matches!(
            initial_modes(&request, &mut short),
            Err(FragmentExecutionError::Stop(_))
        ));
        assert_eq!(short.usage().work(WorkCategory::RecordWrite), 0);
        super::super::tests::assert_released(&short);
        let cancellation = request.cancellation().clone();
        let mut cancelled = ParseControl::new(*request.limits(), cancellation.clone());
        cancelled.admit_source(request.source()).unwrap();
        let before = cancelled.usage().clone();
        cancellation.cancel();
        assert!(matches!(
            initial_modes(&request, &mut cancelled),
            Err(FragmentExecutionError::Stop(ParseStop::Cancelled))
        ));
        assert_eq!(
            cancelled.usage().work(WorkCategory::RecordVisit),
            before.work(WorkCategory::RecordVisit)
        );
        assert_eq!(
            cancelled.usage().work(WorkCategory::InspectByte),
            before.work(WorkCategory::InspectByte)
        );
        super::super::tests::assert_released(&cancelled);
    }

    #[test]
    fn initialization_context_reset_uses_real_stack_template_modes_and_head_pointer() {
        // A single root uses the captured context as the last node: td/th/head
        // do not acquire the modes of a real non-last stack element.
        for (local, expected) in [
            ("td", InsertionMode::InBody),
            ("th", InsertionMode::InBody),
            ("head", InsertionMode::InBody),
            ("html", InsertionMode::BeforeHead),
        ] {
            let request = request("", local, HtmlScriptingStatus::Disabled);
            let mut driver = initialized_driver(&request);
            let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
            state.reset_table_mode(&mut driver.control).unwrap();
            assert_eq!(state.mode, expected, "{local}");
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
        for (context, source, expected) in [
            ("div", "<table><tbody><tr><td>", InsertionMode::InCell),
            ("div", "<table><tbody><tr><th>", InsertionMode::InCell),
            ("html", "<head>", InsertionMode::InHead),
            ("html", "<head></head>", InsertionMode::AfterHead),
            ("template", "<col>", InsertionMode::InColumnGroup),
        ] {
            let request = request(source, context, HtmlScriptingStatus::Disabled);
            let mut driver = initialized_driver(&request);
            for _ in 0..100 {
                if driver.head.as_ref().unwrap().state.as_ref().unwrap().mode == expected {
                    break;
                }
                assert!(!matches!(
                    driver.step().unwrap(),
                    HeadStep::Complete | HeadStep::NeedInput
                ));
            }
            let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
            assert_eq!(state.mode, expected, "reached {source}");
            state.reset_table_mode(&mut driver.control).unwrap();
            assert_eq!(state.mode, expected, "reset {source}");
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }

    #[test]
    fn captured_bootstrap_cause_binds_actual_internal_root_and_f_without_lexical_range() {
        use crate::construction::provenance::{CapturedCauseRole, ConstructionOperation};
        let request = request("<p>A</p>", "div", HtmlScriptingStatus::Disabled);
        let mut driver = initialized_driver(&request);
        let root = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .at(0, &mut driver.control)
            .unwrap();
        complete(&mut driver);
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        let output = state
            .before
            .kernel
            .session
            .fragment_output(&mut driver.control)
            .unwrap();
        state
            .before
            .kernel
            .session
            .prepare_native_associations(&mut driver.control, request.source())
            .unwrap();
        {
            let mut rows = state
                .before
                .kernel
                .session
                .native_associations(&mut driver.control)
                .unwrap();
            let mut found_root = false;
            let mut found_output = false;
            let mut cursor = 0;
            loop {
                let is_root = rows.node_for(cursor, &root).unwrap();
                let is_output = rows.node_for(cursor, &output).unwrap();
                let Some((row, cause)) = rows.row(cursor).unwrap() else {
                    break;
                };
                if is_root || is_output {
                    assert!(row.source.is_none());
                    assert_eq!(cause.operation, ConstructionOperation::Captured);
                    assert!(cause.source.is_none());
                    let captured = cause.captured.as_ref().unwrap();
                    assert_eq!(captured.role, CapturedCauseRole::FragmentBootstrap);
                    assert_eq!(&captured.capture, request.context().capture().identity());
                    assert_eq!(
                        &captured.target_document,
                        request.context().target_document()
                    );
                    assert_eq!(&captured.context, request.context().context_element());
                    if is_root {
                        found_root = true;
                    } else {
                        found_output = true;
                    }
                }
                cursor += 1;
            }
            assert!(found_root && found_output);
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn copied_captured_namespace_does_not_replace_actual_lexical_node_trigger() {
        use crate::construction::StringLocation;
        use crate::construction::provenance::{
            CapturedCauseRole, ConstructionOperation, NativeDestination,
        };
        let request = namespace_request("<x></x>", Namespace::from_uri("urn:actual:context"));
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let x = {
            let (root, mut view) = driver.completed_native_view().unwrap();
            view.child_at(&root, 0).unwrap().unwrap()
        };
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        state
            .before
            .kernel
            .session
            .prepare_native_associations(&mut driver.control, request.source())
            .unwrap();
        let mut node_trigger = None;
        let mut captured_namespace = false;
        {
            let mut rows = state
                .before
                .kernel
                .session
                .native_associations(&mut driver.control)
                .unwrap();
            for cursor in 0..rows.len() {
                let is_x = rows.node_for(cursor, &x).unwrap();
                let (row, cause) = rows.row(cursor).unwrap().unwrap();
                if is_x {
                    node_trigger = Some((
                        cause.operation,
                        cause.source.as_ref().map(|range| {
                            (
                                range.source() == request.source().identity(),
                                range.start(),
                                range.end(),
                            )
                        }),
                    ));
                }
                if matches!(
                    row.destination,
                    NativeDestination::Field(StringLocation::Namespace(_))
                ) && cause
                    .captured
                    .as_ref()
                    .is_some_and(|captured| captured.role == CapturedCauseRole::ContextNamespace)
                {
                    captured_namespace = row.source.is_none()
                        && cause.source.is_none()
                        && cause.captured.as_ref().unwrap().capture
                            == *request.context().capture().identity()
                        && row.start == 0
                        && row.end == "urn:actual:context".len();
                }
            }
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert!(captured_namespace);
        assert_eq!(
            node_trigger,
            Some((ConstructionOperation::Token, Some((true, 0, 3))))
        );
    }

    #[test]
    fn frozen_fragment_parts_keep_actual_f_capture_and_source_after_execution_reclamation() {
        let request = request("<p>A&amp;B</p>", "div", HtmlScriptingStatus::Disabled);
        let mut input = Some(initialized_driver(&request));
        complete(input.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
        assert!(input.is_none());
        completed.prepare_native_associations().unwrap();
        let mut parts = completed.freeze_parts().unwrap();
        assert!(completed.driver.is_none());
        assert!(completed.frozen.is_none());
        assert_eq!(parts.source_binding().0, request.source().identity());
        assert_eq!(parts.source_binding().1, request.source().coordinates());
        assert_eq!(parts.source_binding().2, request.source().len());
        assert_eq!(
            parts.context().unwrap().context_element(),
            request.context().context_element()
        );
        assert!(parts.final_usage().work(WorkCategory::RecordVisit) > 0);
        assert_eq!(parts.diagnostics().unwrap().len(), 0);
        {
            let fragment = parts.fragment().unwrap();
            let p = fragment.root().children().next().unwrap();
            let crate::HtmlNodeKind::Element(element) = fragment.node(&p).unwrap().kind() else {
                panic!()
            };
            assert_eq!(element.name().local_name().code_units(), &[112]);
            let text = fragment.node(&p).unwrap().children().next().unwrap();
            let crate::HtmlNodeKind::Text(value) = fragment.node(&text).unwrap().kind() else {
                panic!()
            };
            assert_eq!(value.code_units(), &[65, 38, 66]);
        }
        let alias = parts.fragment().unwrap().clone();
        assert!(parts.release().is_err());
        assert!(parts.arena.is_some());
        drop(alias);
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }

    #[test]
    fn frozen_fragment_retains_completed_captured_shadow_effect_and_second_template_fallback() {
        let request = shadow_request(
            "<template shadowrootmode=closed><p>A</p></template><template id=second>B</template>Z",
            true,
        );
        let mut input = Some(initialized_driver(&request));
        complete(input.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
        let mut parts = completed.freeze_parts().unwrap();
        {
            let arena = parts.arena.as_ref().unwrap();
            let effect = arena
                .declarative_shadow(&mut parts.control, 0)
                .unwrap()
                .unwrap();
            let crate::construction::context::QualifiedFragmentHost::Captured(host) = &effect.host
            else {
                panic!()
            };
            assert_eq!(host, request.context().context_element());
            assert_eq!(effect.source.source(), request.source().identity());
            assert_eq!(
                (effect.source.start(), effect.source.end()),
                (0, "<template shadowrootmode=closed>".len())
            );
            assert_eq!(effect.properties.mode(), HtmlShadowRootMode::Closed);
            assert!(effect.declarative && effect.available_to_internals);
            assert!(
                arena
                    .declarative_shadow(&mut parts.control, 1)
                    .unwrap()
                    .is_none()
            );
            let fragment = arena.fragment().unwrap();
            assert_ne!(effect.contents, fragment.root().id());
            let p = fragment
                .node(&effect.contents)
                .unwrap()
                .children()
                .next()
                .unwrap();
            let text = fragment.node(&p).unwrap().children().next().unwrap();
            let crate::HtmlNodeKind::Text(value) = fragment.node(&text).unwrap().kind() else {
                panic!()
            };
            assert_eq!(value.code_units(), &[65]);
            let ordinary: Vec<_> = fragment.root().children().collect();
            assert_eq!(ordinary.len(), 2);
            assert_ne!(ordinary[0], effect.template);
            let crate::HtmlNodeKind::Element(element) = fragment.node(&ordinary[0]).unwrap().kind()
            else {
                panic!()
            };
            assert_eq!(
                element.name().local_name().code_units(),
                &"template".encode_utf16().collect::<Vec<_>>()
            );
            let crate::HtmlNodeKind::Text(value) = fragment.node(&ordinary[1]).unwrap().kind()
            else {
                panic!()
            };
            assert_eq!(value.code_units(), &[90]);
        }
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }

    #[test]
    fn fragment_freeze_work_cuts_keep_active_or_frozen_owner_until_same_issuer_failure_cleanup() {
        let request = request(
            "<!doctype html><p>A</p>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut extent = None;
        let mut credit = None;
        let mut stopped_after_arena_transfer = false;
        loop {
            let mut input = Some(initialized_driver(&request));
            complete(input.as_mut().unwrap());
            let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
            let driver = completed.driver.as_mut().unwrap();
            let before = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            if let Some(credit) = credit {
                let remaining = request.limits().work_credits() - before;
                driver
                    .control
                    .charge(WorkCategory::StateStep, remaining - credit)
                    .unwrap();
            }
            match completed.freeze_parts() {
                Ok(mut parts) => {
                    if extent.is_none() {
                        let work = parts
                            .control
                            .usage()
                            .resource(HtmlParseResource::WorkCredits)
                            .current
                            - before;
                        assert!(work < 100_000, "finite coverage extent");
                        extent = Some(work);
                    }
                    assert_eq!(
                        parts
                            .arena
                            .as_ref()
                            .unwrap()
                            .fragment()
                            .unwrap()
                            .root()
                            .children()
                            .count(),
                        1
                    );
                    if parts.control.stop().is_none() {
                        // Successful cuts can leave zero active cleanup credits.
                        if parts.release().is_err() {
                            parts.release().unwrap();
                        }
                    } else {
                        parts.release().unwrap();
                    }
                    super::super::tests::assert_released(&parts.control);
                }
                Err(_) => {
                    let first = completed.driver.as_ref().unwrap().control.stop().unwrap();
                    assert!(matches!(
                        first,
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        }
                    ));
                    stopped_after_arena_transfer |= completed.frozen.is_some();
                    assert!(completed.driver.is_some());
                    assert!(completed.freeze_parts().is_err());
                    assert_eq!(
                        completed.driver.as_ref().unwrap().control.stop(),
                        Some(first)
                    );
                    let mut failed = completed.take_failed().unwrap();
                    assert_eq!(failed.stop(), first);
                    assert_eq!(failed.diagnostics().unwrap().len(), 1);
                    failed.release().unwrap();
                    super::super::tests::assert_released(&failed.control);
                }
            }
            credit = Some(match credit {
                None => 0,
                Some(value) => value + 32,
            });
            if credit.unwrap() > extent.unwrap() {
                break;
            }
        }
        assert!(
            stopped_after_arena_transfer,
            "real reclaimed-execution tail was interrupted"
        );
    }

    #[test]
    fn cancelled_frozen_fragment_partition_drops_native_effects_but_retains_committed_diagnostics()
    {
        let request = request(
            "<!doctype html><p>A</p>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut input = Some(initialized_driver(&request));
        complete(input.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
        let mut parts = completed.freeze_parts().unwrap();
        request.cancellation().cancel();
        assert!(parts.fragment().is_err());
        let first = parts.control.stop().unwrap();
        parts.cleanup_failure().unwrap();
        assert!(parts.arena.is_none());
        assert_eq!(parts.diagnostics().unwrap().len(), 1);
        assert_eq!(parts.control.stop(), Some(first));
        assert!(parts.after_cleanup.is_some());
        assert!(parts.fragment().is_err());
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }

    #[test]
    fn completed_fragment_failure_extraction_preserves_cancelled_freeze_owner_and_diagnostics() {
        let request = request(
            "<!doctype html><p>A</p>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut input = Some(initialized_driver(&request));
        complete(input.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
        request.cancellation().cancel();
        assert!(completed.freeze_parts().is_err());
        assert!(completed.driver.is_some());
        let first = completed.driver.as_ref().unwrap().control.stop().unwrap();
        let mut failure = completed.take_failed().unwrap();
        assert!(completed.driver.is_none());
        assert!(completed.frozen.is_none());
        assert_eq!(failure.stop(), first);
        assert_eq!(failure.diagnostics().unwrap().len(), 1);
        failure.release().unwrap();
        super::super::tests::assert_released(&failure.control);
    }

    #[test]
    fn failed_fragment_partition_keeps_original_stop_and_only_committed_diagnostics() {
        let request = request(
            "<!doctype html><p>A</p>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut input = Some(initialized_driver(&request));
        loop {
            let driver = input.as_mut().unwrap();
            driver.step().unwrap();
            let evidence = driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session
                .retained_evidence()
                .unwrap();
            if !evidence.diagnostics().is_empty() {
                break;
            }
        }
        request.cancellation().cancel();
        assert!(input.as_mut().unwrap().step().is_err());
        let first = input.as_ref().unwrap().control.stop().unwrap();
        let mut failure = ContextFragmentDriver::take_failed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(failure.stop(), first);
        assert_eq!(failure.source().identity(), request.source().identity());
        assert_eq!(
            failure.context().context_element(),
            request.context().context_element()
        );
        assert!(failure.at_stop().work(WorkCategory::RecordVisit) > 0);
        assert!(
            failure.after_cleanup().work(WorkCategory::RecordVisit)
                >= failure.at_stop().work(WorkCategory::RecordVisit)
        );
        assert_eq!(failure.diagnostics().unwrap().len(), 1);
        failure.release().unwrap();
        assert_eq!(failure.control.stop(), Some(first));
        super::super::tests::assert_released(&failure.control);
    }

    #[test]
    fn completed_fragment_moves_only_after_true_return_and_borrows_actual_fragment_evidence() {
        let request = request("<p>A</p>", "div", HtmlScriptingStatus::Disabled);
        let mut input = Some(initialized_driver(&request));
        assert!(matches!(
            ContextFragmentDriver::take_completed(&mut input),
            Err(FragmentExecutionError::Head(
                head::HeadError::IncompleteCompletion
            ))
        ));
        assert!(input.is_some());
        assert!(input.as_ref().unwrap().control.stop().is_none());
        complete(input.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(
            completed.context().unwrap().context_element(),
            request.context().context_element()
        );
        let evidence = completed.diagnostics().unwrap();
        assert_eq!(evidence.len(), 0);
        {
            let (fragment, mut view) = completed.native_view().unwrap();
            assert!(matches!(
                view.kind(&fragment).unwrap(),
                NativeKind::Fragment
            ));
            assert_eq!(view.child_count(&fragment).unwrap(), 1);
            let parser = view.root().unwrap();
            assert!(!view.same_node(&parser, &fragment).unwrap());
        }
        completed.release().unwrap();
        assert!(completed.driver.is_none());
        assert!(matches!(
            completed.native_view(),
            Err(FragmentExecutionError::Consumed)
        ));
    }

    #[test]
    fn real_div_fragment_returns_only_parsed_children_and_inherits_parser_document_mode() {
        let request = request(
            "<p>A&amp;B</p><p>Z</p>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert!(matches!(view.kind(&root).unwrap(), NativeKind::Fragment));
            assert_eq!(view.child_count(&root).unwrap(), 2);
            let first = view.child_at(&root, 0).unwrap().unwrap();
            let second = view.child_at(&root, 1).unwrap().unwrap();
            assert!(!view.same_node(&first, &second).unwrap());
            let NativeKind::Element(name) = view.kind(&first).unwrap() else {
                panic!("actual p")
            };
            assert_eq!(name.local.len(), 1);
            assert_eq!(view.unit(&name.local, 0).unwrap(), Some(u16::from(b'p')));
            let text = view.child_at(&first, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("actual text")
            };
            let units: Vec<_> = (0..value.len())
                .map(|i| view.unit(&value, i).unwrap().unwrap())
                .collect();
            assert_eq!(units, [65, 38, 66]);
            let document = view.root().unwrap();
            assert!(matches!(
                view.kind(&document).unwrap(),
                NativeKind::Document(HtmlDocumentMode::LimitedQuirks)
            ));
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn real_table_context_selects_table_mode_without_context_wrapper_or_implied_table() {
        let request = request(
            "<tr><td>A</td><td>B</td></tr><p>Z</p>",
            "table",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 2);
            let tbody = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&tbody).unwrap() else {
                panic!("actual tbody")
            };
            let name: Vec<_> = (0..name.local.len())
                .map(|i| view.unit(&name.local, i).unwrap().unwrap())
                .collect();
            assert_eq!(name, "tbody".encode_utf16().collect::<Vec<_>>());
            let tr = view.child_at(&tbody, 0).unwrap().unwrap();
            assert_eq!(view.child_count(&tr).unwrap(), 2);
            let a = view.child_at(&tr, 0).unwrap().unwrap();
            let b = view.child_at(&tr, 1).unwrap().unwrap();
            assert!(!view.same_node(&a, &b).unwrap());
            assert_eq!(view.child_count(&a).unwrap(), 1);
            assert_eq!(view.child_count(&b).unwrap(), 1);
            let p = view.child_at(&root, 1).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&p).unwrap() else {
                panic!("actual p")
            };
            assert_eq!(name.local.len(), 1);
            assert_eq!(view.unit(&name.local, 0).unwrap(), Some(112));
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn textarea_context_rcdata_has_no_fabricated_appropriate_end_tag_or_first_lf_rule() {
        let request = request(
            "\nA&amp;B</textarea>Z",
            "textarea",
            HtmlScriptingStatus::Enabled,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 1);
            let text = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("real text")
            };
            let units: Vec<_> = (0..value.len())
                .map(|i| view.unit(&value, i).unwrap().unwrap())
                .collect();
            assert_eq!(
                units,
                "\nA&B</textarea>Z".encode_utf16().collect::<Vec<_>>()
            );
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn context_tokenizer_initialization_uses_actual_mode_without_invented_end_tag() {
        for (context, scripting, source, expected) in [
            (
                "title",
                HtmlScriptingStatus::Enabled,
                "A&amp;B</title>Z",
                "A&B</title>Z",
            ),
            (
                "style",
                HtmlScriptingStatus::Enabled,
                "A&amp;B</style>Z",
                "A&amp;B</style>Z",
            ),
            (
                "script",
                HtmlScriptingStatus::Enabled,
                "A&amp;B</script>Z",
                "A&amp;B</script>Z",
            ),
            (
                "plaintext",
                HtmlScriptingStatus::Disabled,
                "A&amp;B<b>Z",
                "A&amp;B<b>Z",
            ),
            (
                "noscript",
                HtmlScriptingStatus::Enabled,
                "<b>A&amp;B</b>",
                "<b>A&amp;B</b>",
            ),
        ] {
            let request = request(source, context, scripting);
            let mut driver = initialized_driver(&request);
            complete(&mut driver);
            {
                let (root, mut view) = driver.completed_native_view().unwrap();
                assert_eq!(view.child_count(&root).unwrap(), 1);
                let child = view.child_at(&root, 0).unwrap().unwrap();
                let NativeKind::Text(value) = view.kind(&child).unwrap() else {
                    panic!("context produces literal text, without a context wrapper")
                };
                let units: Vec<_> = (0..value.len())
                    .map(|i| view.unit(&value, i).unwrap().unwrap())
                    .collect();
                assert_eq!(
                    units,
                    expected.encode_utf16().collect::<Vec<_>>(),
                    "{context}"
                );
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
        let request = request("<b>A&amp;B</b>", "noscript", HtmlScriptingStatus::Disabled);
        assert_eq!(
            request.requested_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        assert_eq!(
            request.effective_scripting_mode(),
            HtmlParserScriptingMode::Disabled
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 1);
            let child = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&child).unwrap() else {
                panic!("disabled noscript uses actual Data, not Inert rawtext")
            };
            assert_eq!(name.local.len(), 1);
            assert_eq!(view.unit(&name.local, 0).unwrap(), Some(98));
            let text = view.child_at(&child, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("literal A&B")
            };
            assert_eq!(
                (0..value.len())
                    .map(|i| view.unit(&value, i).unwrap().unwrap())
                    .collect::<Vec<_>>(),
                [65, 38, 66]
            );
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn real_utf16_rcdata_context_preserves_isolates_entity_cr_and_literal_end_like_units() {
        let seed = request("", "textarea", HtmlScriptingStatus::Enabled);
        let mut source = vec![65, 0xd83d, 0xde00, 0xd800, 13, 10];
        source.extend("&#13;\0</textarea>Z".encode_utf16());
        let request = HtmlFragmentParseRequest::new(
            HtmlSource::utf16(DocumentString::from_utf16(source)),
            seed.context().clone(),
            super::super::tests::limits(100),
        )
        .unwrap();
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 1);
            let child = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&child).unwrap() else {
                panic!("actual RCDATA")
            };
            let mut expected = vec![65, 0xd83d, 0xde00, 0xd800, 10, 13, 0xfffd];
            expected.extend("</textarea>Z".encode_utf16());
            assert_eq!(units(&mut view, &value), expected);
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn inert_fragment_scripts_retain_parser_document_and_started_state_without_disabled_preparation()
     {
        let request = request("<script>A</script>", "div", HtmlScriptingStatus::Enabled);
        assert_eq!(
            request.effective_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        let root = state
            .before
            .kernel
            .session
            .fragment_output(&mut driver.control)
            .unwrap();
        let script = {
            let mut view = state
                .before
                .kernel
                .session
                .native_view(&mut driver.control)
                .unwrap();
            view.child_at(&root, 0).unwrap().unwrap()
        };
        let metadata = state
            .before
            .kernel
            .session
            .script_metadata(&mut driver.control, &script)
            .unwrap();
        assert!(metadata.flags.already_started());
        assert!(!metadata.flags.force_async());
        assert_eq!(metadata.script_type, None);
        assert!(
            state
                .before
                .kernel
                .session
                .script_preparation_document(&mut driver.control, &script)
                .unwrap()
                .is_none()
        );
        let document = state
            .before
            .kernel
            .session
            .script_parser_document(&mut driver.control, &script)
            .unwrap()
            .unwrap();
        let parser = state
            .before
            .kernel
            .session
            .root(&mut driver.control)
            .unwrap();
        assert!(
            state
                .before
                .kernel
                .session
                .same_node(&mut driver.control, &document, &parser)
                .unwrap()
        );
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    pub(super) fn template_request(source: &str, known: bool) -> HtmlFragmentParseRequest {
        let mut capture = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = capture
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Enabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        if known {
            capture
                .establish_document_template_facts(
                    &document,
                    HtmlDocumentClass::Html,
                    HtmlTemplateDocumentRole::Ordinary,
                    HtmlCaptureTemplateCacheInput::Absent,
                )
                .unwrap();
        }
        let context = capture
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture.establish_root(&context).unwrap();
        let capture = capture.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(source),
            context,
            super::super::tests::limits(100),
        )
        .unwrap()
    }
    #[test]
    fn real_template_fragment_keeps_hosted_contents_and_captured_target_document_distinct() {
        let request = template_request(
            "<template id=t><tr><td>A</td><td>B</td></tr></template><p>Z</p>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let c = &mut driver.control;
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        let session = &state.before.kernel.session;
        let output = session.fragment_output(c).unwrap();
        let mut view = session.native_view(c).unwrap();
        assert_eq!(view.child_count(&output).unwrap(), 2);
        let element = view.child_at(&output, 0).unwrap().unwrap();
        assert_eq!(view.child_count(&element).unwrap(), 0);

        let contents = session
            .unpatched_template_contents(c, &element)
            .unwrap()
            .unwrap();
        let target = session.qualified_node_document(c, &element).unwrap();
        let crate::construction::ParserDocumentReference::Captured(target) = target else {
            panic!("actual captured target")
        };
        assert_eq!(&target, request.context().target_document());
        let crate::construction::ParserDocumentReference::Owned(inert) =
            session.qualified_node_document(c, &contents).unwrap()
        else {
            panic!("actual local inert owner")
        };
        let mut view = session.native_view(c).unwrap();
        assert!(matches!(
            view.kind(&inert).unwrap(),
            NativeKind::Document(HtmlDocumentMode::NoQuirks)
        ));
        assert!(matches!(
            view.kind(&contents).unwrap(),
            NativeKind::Fragment
        ));
        let row = view.child_at(&contents, 0).unwrap().unwrap();
        assert_eq!(view.child_count(&row).unwrap(), 2);
        let a = view.child_at(&row, 0).unwrap().unwrap();
        let b = view.child_at(&row, 1).unwrap().unwrap();
        assert!(!view.same_node(&a, &b).unwrap());

        drop(target);
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn unavailable_template_document_facts_are_conditional_and_latch_first_stop() {
        let request = template_request("<template>A</template>", false);
        let mut driver = initialized_driver(&request);
        let error = loop {
            match driver.step() {
                Ok(HeadStep::Consumed) => {}
                Err(error) => break error,
                Ok(other) => panic!("unexpected completion {other:?}"),
            }
        };
        assert!(
            matches!(
                error,
                FragmentExecutionError::Head(head::HeadError::Kernel(KernelError::Session(
                    SessionError::ContextFactUnavailable(HtmlCaptureFact::TemplateDocumentRole)
                )))
            ),
            "{error:?}"
        );
        let first = driver.control.stop().unwrap();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.stop(), Some(first));
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    pub(super) fn shadow_request(source: &str, allow: bool) -> HtmlFragmentParseRequest {
        shadow_request_with_presence(source, allow, true)
    }
    pub(super) fn shadow_request_with_presence(
        source: &str,
        allow: bool,
        known_absent: bool,
    ) -> HtmlFragmentParseRequest {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        if known_absent {
            builder.establish_no_shadow_root(&context).unwrap();
        }
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(source),
            context,
            super::super::tests::limits(100),
            HtmlParserScriptingMode::Inert,
            allow,
        )
        .unwrap()
    }
    #[test]
    fn captured_shadow_attachment_retains_actual_root_and_second_template_uses_planned_presence() {
        let request = shadow_request(
            "<template shadowrootmode=closed shadowrootdelegatesfocus shadowrootserializable shadowrootclonable shadowrootslotassignment=manual shadowrootcustomelementregistry><p>A</p></template><template shadowrootmode=open><p>B</p></template><p>Z</p>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let effect = session
            .declarative_shadow(&mut driver.control, 0)
            .unwrap()
            .unwrap();
        let crate::construction::context::QualifiedFragmentHost::Captured(host) = &effect.host
        else {
            panic!("actual retained captured host")
        };
        assert_eq!(host, request.context().context_element());
        assert_eq!(effect.properties.mode(), HtmlShadowRootMode::Closed);
        assert_eq!(
            effect.properties.slot_assignment(),
            HtmlSlotAssignment::Manual
        );
        assert!(
            effect.properties.delegates_focus()
                && effect.properties.serializable()
                && effect.properties.clonable()
        );
        assert!(effect.keep_registry_null && effect.declarative && effect.available_to_internals);
        assert_eq!(effect.registry, HtmlCapturedRegistry::Null);
        assert_eq!(effect.source.start(), 0);
        assert!(
            session
                .declarative_shadow(&mut driver.control, 1)
                .unwrap()
                .is_none()
        );
        let shadow = session
            .template_state(&mut driver.control, &effect.template)
            .unwrap()
            .contents;
        let mut view = session.native_view(&mut driver.control).unwrap();
        assert!(view.same_node(&shadow, &effect.contents).unwrap());
        let Some(crate::construction::native_view::NativeFragmentOwner::CapturedShadowHost(actual)) =
            view.fragment_owner(&effect.contents).unwrap()
        else {
            panic!("actual retained capture host")
        };
        assert_eq!(&actual, request.context().context_element());
        drop(actual);
        assert_eq!(view.child_count(&output).unwrap(), 2);
        let fallback = view.child_at(&output, 0).unwrap().unwrap();
        assert!(!view.same_node(&fallback, &effect.template).unwrap());
        assert_eq!(view.child_count(&fallback).unwrap(), 0);
        let p = view.child_at(&effect.contents, 0).unwrap().unwrap();
        let text = view.child_at(&p, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text).unwrap() else {
            panic!("literal A")
        };
        assert_eq!(view.unit(&text, 0).unwrap(), Some(65));
        drop(effect);
        let fallback_contents = session
            .template_state(&mut driver.control, &fallback)
            .unwrap()
            .contents;
        let mut view = session.native_view(&mut driver.control).unwrap();
        let p = view.child_at(&fallback_contents, 0).unwrap().unwrap();
        let text = view.child_at(&p, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text).unwrap() else {
            panic!("literal B")
        };
        assert_eq!(view.unit(&text, 0).unwrap(), Some(66));

        let node = request
            .context()
            .capture()
            .node(request.context().context_element().subject())
            .unwrap();
        assert_eq!(node.shadow_root(), Some(HtmlCapturedShadowRoot::Absent));
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn shadow_presence_is_required_only_after_permission_mode_and_host_eligibility() {
        for (source, allow) in [
            ("<template shadowrootmode=open><p>A</p></template>", false),
            ("<template shadowrootmode=bogus><p>A</p></template>", true),
        ] {
            let request = shadow_request_with_presence(source, allow, false);
            let mut driver = initialized_driver(&request);
            complete(&mut driver);
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            assert!(
                session
                    .declarative_shadow(&mut driver.control, 0)
                    .unwrap()
                    .is_none()
            );
            let (root, mut view) = driver.completed_native_view().unwrap();
            let template = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&template).unwrap() else {
                panic!("fallback actual E")
            };
            assert_eq!(name.local.len(), 8);
            assert_eq!(view.child_count(&template).unwrap(), 0);

            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
        let request = shadow_request_with_presence(
            "<template shadowrootmode=open><p>A</p></template>",
            true,
            false,
        );
        let mut driver = initialized_driver(&request);
        let error = loop {
            if let Err(error) = driver.step() {
                break error;
            }
        };
        assert!(matches!(
            error,
            FragmentExecutionError::Head(head::HeadError::Kernel(KernelError::Session(
                SessionError::ContextFactUnavailable(HtmlCaptureFact::ShadowRoot)
            )))
        ));
        let first = driver.control.stop().unwrap();
        assert_eq!(
            first,
            ParseStop::UnsupportedStandaloneState(
                crate::parser::control::StandaloneUnsupported::Provenance
            )
        );
        assert!(driver.completed_native_view().is_err());
        assert_eq!(driver.control.stop(), Some(first));
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn shadow_context_effect_exact_and_one_short_preserve_cleanup() {
        // E is created for pre-redirect P, whose inert cache is actual owned
        // Document metadata. Only the captured-host attachment is a retained
        // ContextEffects slot; no captured target cache is consulted here.
        for cap in [0, 1] {
            let base = shadow_request("<template shadowrootmode=open><p>A</p></template>", true);
            let limits = base
                .limits()
                .into_builder()
                .context_effects(cap)
                .finish()
                .unwrap();
            let request = HtmlFragmentParseRequest::with_options(
                base.source().clone(),
                base.context().clone(),
                limits,
                HtmlParserScriptingMode::Inert,
                true,
            )
            .unwrap();
            let mut driver = initialized_driver(&request);
            let result = loop {
                match driver.step() {
                    Ok(HeadStep::Complete) => break Ok(()),
                    Ok(_) => {}
                    Err(error) => break Err(error),
                }
            };
            if cap == 1 {
                result.unwrap();
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::ContextEffects)
                        .current,
                    1
                );
                let session = &driver
                    .head
                    .as_ref()
                    .unwrap()
                    .state
                    .as_ref()
                    .unwrap()
                    .before
                    .kernel
                    .session;
                assert!(
                    session
                        .declarative_shadow(&mut driver.control, 0)
                        .unwrap()
                        .is_some()
                );
            } else {
                assert!(result.is_err());
                let first = driver.control.stop().unwrap();
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ContextEffects,
                        ..
                    }
                ));
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::ContextEffects)
                        .current,
                    0
                );
                assert!(driver.completed_native_view().is_err());
                assert_eq!(driver.control.stop(), Some(first));
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }

    #[test]
    fn selectedcontent_clones_local_clonable_shadow_after_light_children() {
        let request = shadow_request(
            "<select><button><selectedcontent></selectedcontent></button><option selected><div><template shadowrootmode=closed shadowrootclonable shadowrootcustomelementregistry><i>S</i></template><b>L</b></div></option></select>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let (root, mut view) = driver.completed_native_view().unwrap();
        let select = view.child_at(&root, 0).unwrap().unwrap();
        let button = view.child_at(&select, 0).unwrap().unwrap();
        let selected = view.child_at(&button, 0).unwrap().unwrap();
        let option = view.child_at(&select, 1).unwrap().unwrap();
        let original = view.child_at(&option, 0).unwrap().unwrap();
        let copy = view.child_at(&selected, 0).unwrap().unwrap();
        assert!(!view.same_node(&original, &copy).unwrap());
        assert_eq!(view.child_count(&original).unwrap(), 1);
        assert_eq!(view.child_count(&copy).unwrap(), 1);
        let original_light = view.child_at(&original, 0).unwrap().unwrap();
        let copied_light = view.child_at(&copy, 0).unwrap().unwrap();
        assert!(!view.same_node(&original_light, &copied_light).unwrap());
        let shadow = view.shadow(&original).unwrap().unwrap();
        let cloned = view.shadow(&copy).unwrap().unwrap();
        assert!(!view.same_node(&shadow.contents, &cloned.contents).unwrap());
        assert_eq!(cloned.properties.mode(), HtmlShadowRootMode::Closed);
        assert!(cloned.properties.clonable());
        assert_eq!(cloned.declarative, Some(true));
        assert_eq!(cloned.keep_registry_null, Some(true));
        assert_eq!(shadow.available_to_internals, Some(true));
        assert_eq!(cloned.available_to_internals, Some(false));
        let original_i = view.child_at(&shadow.contents, 0).unwrap().unwrap();
        let copied_i = view.child_at(&cloned.contents, 0).unwrap().unwrap();
        assert!(!view.same_node(&original_i, &copied_i).unwrap());
        for (node, name, text) in [(&copied_light, "b", 76), (&copied_i, "i", 83)] {
            let NativeKind::Element(actual) = view.kind(node).unwrap() else {
                panic!("literal cloned element")
            };
            assert_eq!(actual.local.len(), 1);
            assert_eq!(
                view.unit(&actual.local, 0).unwrap(),
                Some(u16::from(name.as_bytes()[0]))
            );
            let child = view.child_at(node, 0).unwrap().unwrap();
            let NativeKind::Text(actual) = view.kind(&child).unwrap() else {
                panic!("literal cloned text")
            };
            assert_eq!(actual.len(), 1);
            assert_eq!(view.unit(&actual, 0).unwrap(), Some(text));
        }
        drop(shadow);
        drop(cloned);
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn local_shadow_host_keeps_light_children_and_qualified_root_separate() {
        let request = shadow_request(
            "<div id=h><template shadowrootmode=open shadowrootclonable><span>S</span></template><b>L</b><template shadowrootmode=closed><i>F</i></template></div>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let effect = session
            .declarative_shadow(&mut driver.control, 0)
            .unwrap()
            .unwrap();
        let crate::construction::context::QualifiedFragmentHost::Owned(host) = &effect.host else {
            panic!("actual newly constructed host")
        };
        let mut view = session.native_view(&mut driver.control).unwrap();
        assert_eq!(view.child_count(&output).unwrap(), 1);
        let div = view.child_at(&output, 0).unwrap().unwrap();
        assert!(view.same_node(&div, host).unwrap());
        let shadow = view.shadow(&div).unwrap().unwrap();
        assert!(view.same_node(&shadow.host, &div).unwrap());
        assert!(view.same_node(&shadow.contents, &effect.contents).unwrap());
        assert_eq!(shadow.properties.mode(), HtmlShadowRootMode::Open);
        assert!(shadow.properties.clonable());
        let Some(crate::construction::native_view::NativeFragmentOwner::ShadowHost(owner)) =
            view.fragment_owner(&effect.contents).unwrap()
        else {
            panic!("actual shadow backlink")
        };
        assert!(view.same_node(&owner, &div).unwrap());
        assert_eq!(view.child_count(&div).unwrap(), 2);
        let light = view.child_at(&div, 0).unwrap().unwrap();
        let fallback = view.child_at(&div, 1).unwrap().unwrap();
        for (node, name) in [(&light, "b"), (&fallback, "template")] {
            let NativeKind::Element(actual) = view.kind(node).unwrap() else {
                panic!("literal light child")
            };
            assert_eq!(actual.local.len(), name.len());
            for (i, expected) in name.bytes().enumerate() {
                assert_eq!(
                    view.unit(&actual.local, i).unwrap(),
                    Some(u16::from(expected))
                );
            }
        }
        assert_eq!(view.child_count(&fallback).unwrap(), 0);
        let span = view.child_at(&effect.contents, 0).unwrap().unwrap();
        let text = view.child_at(&span, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text).unwrap() else {
            panic!("shadow text")
        };
        assert_eq!(view.unit(&text, 0).unwrap(), Some(83));
        drop(effect);
        drop(shadow);
        let fallback_contents = session
            .template_state(&mut driver.control, &fallback)
            .unwrap()
            .contents;
        let mut view = session.native_view(&mut driver.control).unwrap();
        let i = view.child_at(&fallback_contents, 0).unwrap().unwrap();
        let text = view.child_at(&i, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text).unwrap() else {
            panic!("fallback text")
        };
        assert_eq!(view.unit(&text, 0).unwrap(), Some(70));

        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn fragment_for_patching_uses_real_output_scope_after_root_redirect() {
        let request = template_request(
            "<?start name='x'?><p>old</p><?end?><span>K</span><template for=x><p>N</p></template><p>Z</p>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let (root, mut view) = driver.completed_native_view().unwrap();
        assert_eq!(view.child_count(&root).unwrap(), 3);
        for (ordinal, name, unit) in [(0, "p", 78), (1, "span", 75), (2, "p", 90)] {
            let node = view.child_at(&root, ordinal).unwrap().unwrap();
            let NativeKind::Element(actual) = view.kind(&node).unwrap() else {
                panic!("literal element")
            };
            assert_eq!(actual.local.len(), name.len());
            for (offset, expected) in name.bytes().enumerate() {
                assert_eq!(
                    view.unit(&actual.local, offset).unwrap(),
                    Some(u16::from(expected))
                );
            }
            let text = view.child_at(&node, 0).unwrap().unwrap();
            let NativeKind::Text(text) = view.kind(&text).unwrap() else {
                panic!("literal text")
            };
            assert_eq!(view.unit(&text, 0).unwrap(), Some(unit));
        }

        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    fn namespace_request(source: &str, namespace: Namespace) -> HtmlFragmentParseRequest {
        let mut capture = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = capture
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = capture
            .add_element(
                &document,
                QualifiedName::new(namespace, None, "context"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        capture.establish_root(&context).unwrap();
        let capture = capture.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(source),
            context,
            super::super::tests::limits(100),
        )
        .unwrap()
    }
    fn units(
        view: &mut crate::construction::NativeView<'_>,
        value: &crate::construction::native_view::NativeString<'_>,
    ) -> Vec<u16> {
        (0..value.len())
            .map(|i| view.unit(value, i).unwrap().unwrap())
            .collect()
    }

    fn integration_request(
        source: &str,
        namespace: Namespace,
        local: &str,
        attributes: OrderedAttributes,
    ) -> HtmlFragmentParseRequest {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 20_000_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(namespace, None, local),
                attributes.clone(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let HtmlCapturedNodeKind::Element(element) = context
            .capture()
            .node(context.context_element().subject())
            .unwrap()
            .kind()
        else {
            panic!("captured context element")
        };
        assert_eq!(
            element.attributes(),
            &attributes,
            "exact raw records survive"
        );
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(source),
            context,
            super::super::tests::limits(100)
                .into_builder()
                .work_credits(1_000_000)
                .finish()
                .unwrap(),
        )
        .unwrap()
    }

    fn encoding_attribute(
        namespace: Namespace,
        prefix: Option<&str>,
        local: &str,
        value: &str,
    ) -> OrderedAttributes {
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(namespace, prefix.map(DocumentString::from), local),
            value,
        )])
        .unwrap()
    }

    fn integration_observation(
        namespace: Namespace,
        local: &str,
        attributes: OrderedAttributes,
    ) -> (bool, Option<Vec<u16>>) {
        let request = integration_request("<x>A</x>", namespace, local, attributes);
        let classified = request.context().is_html_integration_point();
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let namespace = {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 1);
            let child = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&child).unwrap() else {
                panic!("actual parsed child")
            };
            assert_eq!(units(&mut view, &name.local), [120]);
            name.namespace.as_ref().map(|value| units(&mut view, value))
        };
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        (classified, namespace)
    }

    fn assert_encoding_prefix(prefix: Option<&str>) {
        // HTML877d §13.2.6 and §13.4 step 19, extended to the accepted raw
        // domain by #1175: namespace/local-name identity ignores prefix metadata.
        // These literal expectations do not come from either classifier.
        let actual: Vec<_> = [
            "text/html",
            "TeXt/HtMl",
            "application/xhtml+xml",
            "ApPlIcAtIoN/XhTmL+XmL",
        ]
        .into_iter()
        .map(|value| {
            integration_observation(
                Namespace::mathml(),
                "annotation-xml",
                encoding_attribute(Namespace::none(), prefix, "encoding", value),
            )
        })
        .collect();
        let expected = vec![
            (
                true,
                Some(
                    "http://www.w3.org/1999/xhtml"
                        .encode_utf16()
                        .collect::<Vec<_>>()
                ),
            );
            4
        ];
        assert_eq!(actual, expected, "retained prefix {prefix:?}");
    }

    #[test]
    fn annotation_xml_absent_encoding_prefix_qualifies_both_mime_values() {
        assert_encoding_prefix(None);
    }

    #[test]
    fn annotation_xml_empty_encoding_prefix_qualifies_both_mime_values() {
        assert_encoding_prefix(Some(""));
    }

    #[test]
    fn annotation_xml_nonempty_encoding_prefix_qualifies_both_mime_values() {
        assert_encoding_prefix(Some("p"));
    }

    #[test]
    fn annotation_xml_exact_attribute_identity_and_unpadded_values_control_integration() {
        for attributes in [
            OrderedAttributes::default(),
            encoding_attribute(Namespace::svg(), None, "encoding", "text/html"),
            encoding_attribute(
                Namespace::from_uri("urn:encoding"),
                Some("p"),
                "encoding",
                "text/html",
            ),
            encoding_attribute(Namespace::none(), None, "Encoding", "text/html"),
            encoding_attribute(Namespace::none(), None, "p:encoding", "text/html"),
            encoding_attribute(Namespace::none(), None, ":encoding", "text/html"),
            encoding_attribute(Namespace::none(), None, "other", "text/html"),
            encoding_attribute(Namespace::none(), None, "encoding", " text/html"),
            encoding_attribute(Namespace::none(), None, "encoding", "text/html "),
            encoding_attribute(
                Namespace::none(),
                None,
                "encoding",
                "text/html;charset=utf-8",
            ),
            encoding_attribute(Namespace::none(), None, "encoding", "text/xml"),
            encoding_attribute(Namespace::none(), None, "encoding", ""),
        ] {
            assert_eq!(
                integration_observation(Namespace::mathml(), "annotation-xml", attributes.clone()),
                (
                    false,
                    Some(
                        "http://www.w3.org/1998/Math/MathML"
                            .encode_utf16()
                            .collect()
                    )
                ),
                "attribute {attributes:?}",
            );
        }
        for (namespace, local, child_namespace) in [
            (Namespace::none(), "annotation-xml", None),
            (
                Namespace::svg(),
                "annotation-xml",
                Some("http://www.w3.org/2000/svg"),
            ),
            (
                Namespace::html(),
                "annotation-xml",
                Some("http://www.w3.org/1999/xhtml"),
            ),
            (
                Namespace::mathml(),
                "Annotation-xml",
                Some("http://www.w3.org/1998/Math/MathML"),
            ),
            (
                Namespace::mathml(),
                "other",
                Some("http://www.w3.org/1998/Math/MathML"),
            ),
        ] {
            assert_eq!(
                integration_observation(
                    namespace,
                    local,
                    encoding_attribute(Namespace::none(), None, "encoding", "text/html")
                ),
                (
                    false,
                    child_namespace.map(|value| value.encode_utf16().collect())
                ),
                "element {local}",
            );
        }
    }

    #[test]
    fn annotation_xml_actual_nodes_override_opposite_adjusted_contexts() {
        // §13.2.4.2 selects the captured context only while the open stack has
        // its synthetic root alone. Later dispatch uses the actual current node.
        for (value, source, path, child_namespace, sibling_namespace) in [
            (
                "text/html",
                "<math><annotation-xml encoding='text/xml'><x>A</x></annotation-xml></math><x>B</x>",
                &[0, 0, 0][..],
                "http://www.w3.org/1998/Math/MathML",
                "http://www.w3.org/1999/xhtml",
            ),
            (
                "text/xml",
                "<annotation-xml encoding='text/html'><x>A</x></annotation-xml><x>B</x>",
                &[0, 0][..],
                "http://www.w3.org/1999/xhtml",
                "http://www.w3.org/1998/Math/MathML",
            ),
            (
                "text/html",
                "<math><annotation-xml p:encoding='text/html'><x>A</x></annotation-xml></math><x>B</x>",
                &[0, 0, 0][..],
                "http://www.w3.org/1998/Math/MathML",
                "http://www.w3.org/1999/xhtml",
            ),
            (
                "text/html",
                "<math><annotation-xml :encoding='text/html'><x>A</x></annotation-xml></math><x>B</x>",
                &[0, 0, 0][..],
                "http://www.w3.org/1998/Math/MathML",
                "http://www.w3.org/1999/xhtml",
            ),
        ] {
            let request = integration_request(
                source,
                Namespace::mathml(),
                "annotation-xml",
                encoding_attribute(Namespace::none(), None, "encoding", value),
            );
            let mut driver = initialized_driver(&request);
            complete(&mut driver);
            {
                let (root, mut view) = driver.completed_native_view().unwrap();
                assert_eq!(view.child_count(&root).unwrap(), 2);
                let mut child = root.clone();
                for ordinal in path {
                    child = view.child_at(&child, *ordinal).unwrap().unwrap();
                }
                for (child, namespace) in [
                    (child, child_namespace),
                    (view.child_at(&root, 1).unwrap().unwrap(), sibling_namespace),
                ] {
                    let NativeKind::Element(name) = view.kind(&child).unwrap() else {
                        panic!("actual x")
                    };
                    assert_eq!(units(&mut view, &name.local), [120]);
                    assert_eq!(
                        units(&mut view, &name.namespace.unwrap()),
                        namespace.encode_utf16().collect::<Vec<_>>()
                    );
                }
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }

    #[test]
    fn annotation_xml_irrelevant_attribute_payloads_remain_lazy_under_work_limit() {
        // Ten million units cannot be inspected within a million work credits.
        // Wrong element/attribute roles and impossible MIME lengths must finish
        // without scanning the value. The source remains the same small literal.
        let large = "z".repeat(10_000_000);
        for (namespace, local, attribute_local, classified, child_namespace) in [
            (
                Namespace::svg(),
                "foreignObject",
                "encoding",
                true,
                "http://www.w3.org/1999/xhtml",
            ),
            (
                Namespace::mathml(),
                "other",
                "encoding",
                false,
                "http://www.w3.org/1998/Math/MathML",
            ),
            (
                Namespace::mathml(),
                "annotation-xml",
                "other",
                false,
                "http://www.w3.org/1998/Math/MathML",
            ),
            (
                Namespace::mathml(),
                "annotation-xml",
                "encoding",
                false,
                "http://www.w3.org/1998/Math/MathML",
            ),
        ] {
            assert_eq!(
                integration_observation(
                    namespace,
                    local,
                    encoding_attribute(Namespace::none(), None, attribute_local, &large)
                ),
                (classified, Some(child_namespace.encode_utf16().collect())),
                "element {local}, attribute {attribute_local}",
            );
        }
    }
    #[test]
    fn captured_integration_points_use_actual_qualified_encoding_and_math_text_exceptions() {
        for (namespace, local, attrs, source, expected) in [
            (
                Namespace::mathml(),
                "mi",
                OrderedAttributes::default(),
                "<mglyph/><malignmark/><x>A</x>",
                vec![
                    ("mglyph", "http://www.w3.org/1998/Math/MathML"),
                    ("malignmark", "http://www.w3.org/1998/Math/MathML"),
                    ("x", "http://www.w3.org/1999/xhtml"),
                ],
            ),
            (
                Namespace::mathml(),
                "annotation-xml",
                OrderedAttributes::new([crate::Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "encoding"),
                    "TeXt/HtMl",
                )])
                .unwrap(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1999/xhtml")],
            ),
            (
                Namespace::mathml(),
                "annotation-xml",
                OrderedAttributes::new([crate::Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:encoding"), None, "encoding"),
                    "text/html",
                )])
                .unwrap(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1998/Math/MathML")],
            ),
            (
                Namespace::mathml(),
                "annotation-xml",
                OrderedAttributes::new([crate::Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "Encoding"),
                    "text/html",
                )])
                .unwrap(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1998/Math/MathML")],
            ),
            (
                Namespace::svg(),
                "foreignObject",
                OrderedAttributes::default(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1999/xhtml")],
            ),
            (
                Namespace::svg(),
                "desc",
                OrderedAttributes::default(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1999/xhtml")],
            ),
            (
                Namespace::svg(),
                "title",
                OrderedAttributes::default(),
                "<x>A</x>",
                vec![("x", "http://www.w3.org/1999/xhtml")],
            ),
        ] {
            let mut builder = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
                HtmlHostFactsRevision::fresh(),
            );
            let document = builder
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            let context = builder
                .add_element(
                    &document,
                    QualifiedName::new(namespace, None, local),
                    attrs,
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            builder.establish_root(&context).unwrap();
            let capture = builder.finish().unwrap();
            let context =
                HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap())
                    .unwrap();
            let request = HtmlFragmentParseRequest::new(
                HtmlSource::utf8(source),
                context,
                super::super::tests::limits(100),
            )
            .unwrap();
            let mut driver = initialized_driver(&request);
            complete(&mut driver);
            {
                let (root, mut view) = driver.completed_native_view().unwrap();
                assert_eq!(view.child_count(&root).unwrap(), expected.len());
                for (ordinal, (local, namespace)) in expected.iter().enumerate() {
                    let child = view.child_at(&root, ordinal).unwrap().unwrap();
                    let NativeKind::Element(name) = view.kind(&child).unwrap() else {
                        panic!("actual source element")
                    };
                    assert_eq!(
                        units(&mut view, &name.local),
                        local.encode_utf16().collect::<Vec<_>>()
                    );
                    assert_eq!(
                        units(&mut view, &name.namespace.unwrap()),
                        namespace.encode_utf16().collect::<Vec<_>>()
                    );
                }
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }
    #[test]
    fn arbitrary_captured_namespace_is_copied_exactly_and_cdata_uses_adjusted_context() {
        let request = namespace_request(
            "<![CDATA[A<B]]><x xlink:href=u></x><p>Z</p>",
            Namespace::from_uri("urn:actual:context"),
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            assert_eq!(view.child_count(&root).unwrap(), 3);
            let text = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("real CDATA")
            };
            assert_eq!(units(&mut view, &value), [65, 60, 66]);
            let x = view.child_at(&root, 1).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&x).unwrap() else {
                panic!("real foreign x")
            };
            assert_eq!(
                units(&mut view, &name.namespace.unwrap()),
                "urn:actual:context".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(units(&mut view, &name.local), [120]);
            let attr = view.attribute_at(&x, 0).unwrap().unwrap();
            assert_eq!(
                units(&mut view, &attr.name.namespace.unwrap()),
                "http://www.w3.org/1999/xlink"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, &attr.name.prefix.unwrap()),
                "xlink".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, &attr.name.local),
                "href".encode_utf16().collect::<Vec<_>>()
            );
            let p = view.child_at(&root, 2).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&p).unwrap() else {
                panic!("HTML breakout p")
            };
            assert_eq!(
                units(&mut view, &name.namespace.unwrap()),
                "http://www.w3.org/1999/xhtml"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn actual_absent_context_namespace_stays_absent_on_created_foreign_elements() {
        let request = namespace_request("<X>A</X>", Namespace::none());
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        {
            let (root, mut view) = driver.completed_native_view().unwrap();
            let x = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&x).unwrap() else {
                panic!("real foreign x")
            };
            assert!(name.namespace.is_none());
            assert_eq!(units(&mut view, &name.local), [120]);
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn captured_nearest_form_controls_parser_rows_without_fabricated_parser_association() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let form = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "form"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&form).unwrap();
        builder.establish_parent(&context, &form).unwrap();
        let capture = builder.finish().unwrap();
        let form = capture
            .element_id(&capture.subject_id(&form).unwrap())
            .unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        assert_eq!(context.nearest_form(), Some(&form));
        let request = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<form id=x><img id=a></form><form id=y><img id=b></form>"),
            context,
            super::super::tests::limits(100),
        )
        .unwrap();
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert!(state.current_form.is_none());
        let session = &state.before.kernel.session;
        let root = session.fragment_output(&mut driver.control).unwrap();
        let mut view = session.native_view(&mut driver.control).unwrap();
        assert_eq!(view.child_count(&root).unwrap(), 2);
        let first = view.child_at(&root, 0).unwrap().unwrap();
        let actual_form = view.child_at(&root, 1).unwrap().unwrap();
        let second = view.child_at(&actual_form, 0).unwrap().unwrap();
        for (node, name) in [(&first, "img"), (&actual_form, "form"), (&second, "img")] {
            let NativeKind::Element(actual) = view.kind(node).unwrap() else {
                panic!("literal element")
            };
            assert_eq!(
                units(&mut view, &actual.local),
                name.encode_utf16().collect::<Vec<_>>()
            );
        }

        for (node, expected) in [(&first, None), (&second, Some(&actual_form))] {
            let facts = session.image_form_state(&mut driver.control, node).unwrap();
            assert!(!facts.parser_inserted);
            assert_ne!(
                facts.evidence,
                crate::construction::FormAssociationEvidence::ParserAssociated
            );
            let owner = session.image_form_owner(&mut driver.control, node).unwrap();
            match (owner, expected) {
                (None, None) => {}
                (Some(owner), Some(expected)) => assert!(
                    session
                        .same_node(&mut driver.control, &owner, expected)
                        .unwrap()
                ),
                _ => panic!("actual ordinary insertion association differs"),
            }
        }
        let evidence = session.retained_evidence().unwrap();
        assert_eq!(evidence.len(), 2);
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn svg_context_script_wrapper_preserves_disconnected_fragment_processing_and_namespace() {
        let request = namespace_request("<script>A</script><g/>", Namespace::svg());
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session;
        let root = session.fragment_output(&mut driver.control).unwrap();
        let mut view = session.native_view(&mut driver.control).unwrap();
        assert_eq!(view.child_count(&root).unwrap(), 2);
        let script = view.child_at(&root, 0).unwrap().unwrap();
        let g = view.child_at(&root, 1).unwrap().unwrap();
        for (node, name) in [(&script, "script"), (&g, "g")] {
            let NativeKind::Element(actual) = view.kind(node).unwrap() else {
                panic!("literal SVG element")
            };
            assert_eq!(
                units(&mut view, &actual.local),
                name.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, actual.namespace.as_ref().unwrap()),
                "http://www.w3.org/2000/svg"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
        }
        let text = view.child_at(&script, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text).unwrap() else {
            panic!("script Text")
        };
        assert_eq!(units(&mut view, &text), [65]);

        let facts = session
            .svg_script_state(&mut driver.control, &script)
            .unwrap();
        assert!(!facts.flags.already_processed());
        assert_eq!(facts.origin, crate::construction::SvgScriptOrigin::Parser);
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn local_shadow_root_exact_node_cap_and_postattachment_cancellation_keep_real_owners() {
        // P, internal root, F, div, target inert Document, E and its original
        // contents consume seven nodes. The separately attached shadow root is
        // the eighth; closing tokens create no further native record.
        for cap in [7, 8] {
            let base = shadow_request("<div><template shadowrootmode=open></template></div>", true);
            let limits = base
                .limits()
                .into_builder()
                .managed_nodes(cap)
                .finish()
                .unwrap();
            let request = HtmlFragmentParseRequest::with_options(
                base.source().clone(),
                base.context().clone(),
                limits,
                HtmlParserScriptingMode::Inert,
                true,
            )
            .unwrap();
            let mut driver = initialized_driver(&request);
            let result = loop {
                match driver.step() {
                    Ok(HeadStep::Complete) => break Ok(()),
                    Ok(_) => {}
                    Err(error) => break Err(error),
                }
            };
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                cap
            );
            if cap == 8 {
                result.unwrap();
                let session = &driver
                    .head
                    .as_ref()
                    .unwrap()
                    .state
                    .as_ref()
                    .unwrap()
                    .before
                    .kernel
                    .session;
                let effect = session
                    .declarative_shadow(&mut driver.control, 0)
                    .unwrap()
                    .unwrap();
                let crate::construction::context::QualifiedFragmentHost::Owned(host) = &effect.host
                else {
                    panic!("local native host")
                };
                let mut view = session.native_view(&mut driver.control).unwrap();
                let shadow = view.shadow(host).unwrap().unwrap();
                assert!(view.same_node(&effect.contents, &shadow.contents).unwrap());
                assert_eq!(view.child_count(&effect.contents).unwrap(), 0);
            } else {
                assert!(result.is_err());
                let first = driver.control.stop().unwrap();
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedNodes,
                        ..
                    }
                ));
                assert!(driver.completed_native_view().is_err());
                assert_eq!(driver.control.stop(), Some(first));
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
        let request = shadow_request("<div><template shadowrootmode=open></template></div>", true);
        let mut driver = initialized_driver(&request);
        assert!(matches!(driver.step().unwrap(), HeadStep::Consumed));
        assert!(matches!(driver.step().unwrap(), HeadStep::Consumed));
        {
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            let effect = session
                .declarative_shadow(&mut driver.control, 0)
                .unwrap()
                .unwrap();
            let crate::construction::context::QualifiedFragmentHost::Owned(host) = &effect.host
            else {
                panic!("actual attachment before cancellation")
            };
            let mut view = session.native_view(&mut driver.control).unwrap();
            assert!(view.shadow(host).unwrap().is_some());
        }
        request.cancellation().cancel();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        assert!(driver.completed_native_view().is_err());
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn ready_attachment_registry_follows_actual_shadow_host_adoption_from_inert_contents() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let global = builder.add_registry(Some(Vec::new())).unwrap();
        builder
            .establish_registry_scope(&global, HtmlRegistryScope::Global)
            .unwrap();
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Registry(global.clone()),
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Registry(global.clone()),
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let global = capture.registry_id(&global).unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template><div><template shadowrootmode=open></template></div></template>",
            ),
            context,
            super::super::tests::limits(100),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap();
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        let session = &mut state.before.kernel.session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let effect = session
            .declarative_shadow(&mut driver.control, 0)
            .unwrap()
            .unwrap();
        assert_eq!(effect.registry, HtmlCapturedRegistry::Null);
        assert!(!effect.keep_registry_null);
        let crate::construction::context::QualifiedFragmentHost::Owned(host) = effect.host else {
            panic!("local host created in actual inert contents")
        };
        let shadow = effect.contents;
        session
            .insert(
                &mut driver.control,
                &output,
                &host,
                None,
                Some(&state.script.as_ref().unwrap().capability),
            )
            .unwrap();
        assert_eq!(
            session
                .lookup_registry(&mut driver.control, &shadow)
                .unwrap(),
            HtmlCapturedRegistry::Registry(global.clone())
        );
        let actual = session
            .declarative_shadow(&mut driver.control, 0)
            .unwrap()
            .unwrap()
            .registry;
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert_eq!(actual, HtmlCapturedRegistry::Registry(global));
    }

    #[test]
    fn no_definition_creation_retains_saved_is_and_clones_independent_backing() {
        let request = request(
            "<html is='late'><x-a></x-a><div is=''></div><annotation-xml></annotation-xml><svg is='foreign'/><select><button><selectedcontent></selectedcontent></button><option selected><x-b is='saved'>T</x-b></option></select>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session;
        let root = session.fragment_output(&mut driver.control).unwrap();
        let document = session.root(&mut driver.control).unwrap();
        let mut view = session.native_view(&mut driver.control).unwrap();
        assert_eq!(view.child_count(&root).unwrap(), 5);
        let html = view.child_at(&document, 0).unwrap().unwrap();
        assert_eq!(view.attribute_count(&html).unwrap(), 1);
        let late = view.attribute_at(&html, 0).unwrap().unwrap();
        assert_eq!(late.value.len(), 4);
        let custom = view.child_at(&root, 0).unwrap().unwrap();
        let empty_is = view.child_at(&root, 1).unwrap().unwrap();
        let reserved = view.child_at(&root, 2).unwrap().unwrap();
        let foreign = view.child_at(&root, 3).unwrap().unwrap();
        let select = view.child_at(&root, 4).unwrap().unwrap();
        let button = view.child_at(&select, 0).unwrap().unwrap();
        let selected = view.child_at(&button, 0).unwrap().unwrap();
        let option = view.child_at(&select, 1).unwrap().unwrap();
        let original = view.child_at(&option, 0).unwrap().unwrap();
        let copy = view.child_at(&selected, 0).unwrap().unwrap();
        assert!(!view.same_node(&original, &copy).unwrap());
        let (root_state, root_is) = session
            .element_creation(&mut driver.control, &html)
            .unwrap();
        assert_eq!(root_state, HtmlCustomElementState::Uncustomized);
        assert!(root_is.is_none());
        let (state, saved) = session
            .element_creation(&mut driver.control, &custom)
            .unwrap();
        assert_eq!(state, HtmlCustomElementState::Undefined);
        assert!(saved.is_none());
        let (state, saved) = session
            .element_creation(&mut driver.control, &empty_is)
            .unwrap();
        assert_eq!(state, HtmlCustomElementState::Undefined);
        assert!(saved.unwrap().is_empty());
        let (state, saved) = session
            .element_creation(&mut driver.control, &reserved)
            .unwrap();
        assert_eq!(state, HtmlCustomElementState::Uncustomized);
        assert!(saved.is_none());
        let (state, saved) = session
            .element_creation(&mut driver.control, &foreign)
            .unwrap();
        assert_eq!(state, HtmlCustomElementState::Uncustomized);
        assert_eq!(
            saved.unwrap().code_units(),
            &[102, 111, 114, 101, 105, 103, 110]
        );
        let (state, first) = session
            .element_creation(&mut driver.control, &original)
            .unwrap();
        let (copied, second) = session
            .element_creation(&mut driver.control, &copy)
            .unwrap();
        assert_eq!(state, HtmlCustomElementState::Undefined);
        assert_eq!(copied, state);
        assert_eq!(first.unwrap().code_units(), &[115, 97, 118, 101, 100]);
        assert_eq!(second.unwrap().code_units(), first.unwrap().code_units());
        assert_ne!(
            first.unwrap().code_units().as_ptr(),
            second.unwrap().code_units().as_ptr()
        );
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn creation_asks_unavailable_definitions_only_when_a_definition_can_match() {
        for (source, definitions, expected) in [
            ("<div>X</div>", None, None),
            (
                "<x-a>X</x-a>",
                None,
                Some(SessionError::ContextFactUnavailable(
                    HtmlCaptureFact::Definitions,
                )),
            ),
            (
                "<div is=''>X</div>",
                None,
                Some(SessionError::ContextFactUnavailable(
                    HtmlCaptureFact::Definitions,
                )),
            ),
            (
                "<x-a>X</x-a>",
                Some(vec![HtmlCustomElementDefinition::new(
                    "x-a".into(),
                    "x-a".into(),
                    false,
                    false,
                )]),
                Some(SessionError::ContextReactionUnavailable),
            ),
            (
                "<div is='x-a'>X</div>",
                Some(vec![HtmlCustomElementDefinition::new(
                    "x-a".into(),
                    "div".into(),
                    false,
                    false,
                )]),
                Some(SessionError::ContextReactionUnavailable),
            ),
        ] {
            let mut builder = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
                HtmlHostFactsRevision::fresh(),
            );
            let registry = builder.add_registry(definitions).unwrap();
            builder
                .establish_registry_scope(&registry, HtmlRegistryScope::Global)
                .unwrap();
            let document = builder
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Registry(registry.clone()),
                )
                .unwrap();
            let context = builder
                .add_element(
                    &document,
                    QualifiedName::new(Namespace::html(), None, "div"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Registry(registry),
                )
                .unwrap();
            builder.establish_root(&context).unwrap();
            let capture = builder.finish().unwrap();
            let context =
                HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap())
                    .unwrap();
            let request = HtmlFragmentParseRequest::new(
                HtmlSource::utf8(source),
                context,
                super::super::tests::limits(100),
            )
            .unwrap();
            let mut driver = initialized_driver(&request);
            let actual = if expected.is_some() {
                let error = driver
                    .step()
                    .expect_err("actual definition qualification before creation");
                assert!(driver.completed_native_view().is_err());
                let first = driver.control.stop().unwrap();
                assert_eq!(
                    first,
                    ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
                );
                let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
                assert!(matches!(
                    &state.pending.as_ref().unwrap().payload,
                    prologue::Payload::Tag(Some(_))
                ));
                let session = &state.before.kernel.session;
                let root = session.fragment_output(&mut driver.control);
                assert!(root.is_err());
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::ManagedNodes)
                        .current,
                    3
                );
                Some(error)
            } else {
                complete(&mut driver);
                let (root, mut view) = driver.completed_native_view().unwrap();
                assert_eq!(view.child_count(&root).unwrap(), 1);
                None
            };
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
            match (actual, expected) {
                (
                    Some(FragmentExecutionError::Head(head::HeadError::Kernel(
                        KernelError::Session(actual),
                    ))),
                    Some(expected),
                ) => assert_eq!(actual, expected),
                (None, None) => {}
                (actual, _) => panic!("wrong qualified error {actual:?}"),
            }
        }
    }
    #[test]
    fn adopted_local_host_uses_current_registry_disable_shadow_before_attachment() {
        use crate::parser::tokenizer::token::{Event, TokenKind};
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let registry = builder
            .add_registry(Some(vec![HtmlCustomElementDefinition::new(
                "x-host".into(),
                "x-host".into(),
                false,
                true,
            )]))
            .unwrap();
        builder
            .establish_registry_scope(&registry, HtmlRegistryScope::Global)
            .unwrap();
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Registry(registry.clone()),
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Registry(registry),
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request = HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template><x-host></x-host></template><template shadowrootmode=open>",
            ),
            context,
            super::super::tests::limits(100),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap();
        let mut driver = initialized_driver(&request);
        // Four literal tokens end the real outer template before the later
        // template is pulled. x-host was created in the real Null-registry IT.
        for _ in 0..4 {
            assert!(matches!(driver.step().unwrap(), HeadStep::Consumed));
        }
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        assert_eq!(state.before.kernel.open.as_ref().unwrap().len(), 1);
        let output = state
            .before
            .kernel
            .session
            .fragment_output(&mut driver.control)
            .unwrap();
        let mut view = state
            .before
            .kernel
            .session
            .native_view(&mut driver.control)
            .unwrap();
        let outer = view.child_at(&output, 0).unwrap().unwrap();
        let contents = view.template_contents(&outer).unwrap().unwrap();
        let host = view.child_at(&contents, 0).unwrap().unwrap();
        assert_eq!(
            state
                .before
                .kernel
                .session
                .lookup_registry(&mut driver.control, &host)
                .unwrap(),
            HtmlCapturedRegistry::Null
        );
        let capability = &state.script.as_ref().unwrap().capability;
        state
            .before
            .kernel
            .session
            .insert(&mut driver.control, &output, &host, None, Some(capability))
            .unwrap();
        // Consume the actual next source-qualified start tag with the same
        // tokenizer/evidence owner; no authored tag, fake source or new issuer.
        let token = loop {
            let evidence = state
                .before
                .kernel
                .session
                .evidence(&mut driver.control)
                .unwrap();
            let crate::parser::tokenizer::TokenizerPull::Event(event) = state
                .tokenizer
                .as_mut()
                .unwrap()
                .next(&mut driver.control, false, evidence)
                .unwrap()
            else {
                panic!("complete fixture requires actual event");
            };
            match event {
                Event::Token(token) => break token.into_token(&mut driver.control).unwrap(),
                event => state.lexical.try_push(&mut driver.control, event).unwrap(),
            }
        };
        let source = token.source;
        let TokenKind::Tag(tag) = token.kind else {
            panic!("literal second template")
        };
        let mut input = Some(tag);
        let template = state
            .before
            .kernel
            .session
            .allocate_html_tag_for_creation(
                &mut driver.control,
                &mut input,
                &host,
                None,
                crate::construction::CreationContext::Ordinary,
            )
            .unwrap();
        let attached = state
            .before
            .kernel
            .session
            .attach_declarative_shadow(
                &mut driver.control,
                &template,
                crate::construction::context::DeclarativeShadowHost::Owned(&host),
                &source,
                capability,
            )
            .unwrap();
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert!(
            !attached,
            "current captured definition disables shadow after actual adoption"
        );
    }

    #[test]
    fn cloning_adopted_custom_host_refuses_actual_new_upgrade_reaction_domain() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
            HtmlHostFactsRevision::fresh(),
        );
        let registry = builder
            .add_registry(Some(vec![HtmlCustomElementDefinition::new(
                "x-host".into(),
                "x-host".into(),
                false,
                false,
            )]))
            .unwrap();
        builder
            .establish_registry_scope(&registry, HtmlRegistryScope::Global)
            .unwrap();
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Registry(registry.clone()),
            )
            .unwrap();
        builder
            .establish_document_template_facts(
                &document,
                HtmlDocumentClass::Html,
                HtmlTemplateDocumentRole::Ordinary,
                HtmlCaptureTemplateCacheInput::Absent,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Registry(registry),
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request=HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<template><x-host></x-host></template><select><button><selectedcontent></selectedcontent></button><option selected></option></select>"),
            context,super::super::tests::limits(100)).unwrap();
        let mut driver = initialized_driver(&request);
        // Stop with the literal option still on the real stack; its actual
        // next end token performs the required parser-pop selectedcontent update.
        for _ in 0..10 {
            assert!(matches!(driver.step().unwrap(), HeadStep::Consumed));
        }
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        let output = state
            .before
            .kernel
            .session
            .fragment_output(&mut driver.control)
            .unwrap();
        let mut view = state
            .before
            .kernel
            .session
            .native_view(&mut driver.control)
            .unwrap();
        let outer = view.child_at(&output, 0).unwrap().unwrap();
        let contents = view.template_contents(&outer).unwrap().unwrap();
        let host = view.child_at(&contents, 0).unwrap().unwrap();
        let select = view.child_at(&output, 1).unwrap().unwrap();
        let option = view.child_at(&select, 1).unwrap().unwrap();
        let button = view.child_at(&select, 0).unwrap().unwrap();
        let selected = view.child_at(&button, 0).unwrap().unwrap();
        assert_eq!(
            state
                .before
                .kernel
                .session
                .lookup_registry(&mut driver.control, &host)
                .unwrap(),
            HtmlCapturedRegistry::Null
        );
        state
            .before
            .kernel
            .session
            .insert(
                &mut driver.control,
                &option,
                &host,
                None,
                Some(&state.script.as_ref().unwrap().capability),
            )
            .unwrap();
        let result = driver.step();
        let first = driver.control.stop();
        if result.is_ok() {
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            let mut view = session.native_view(&mut driver.control).unwrap();
            let copy = view
                .child_at(&selected, 0)
                .unwrap()
                .expect("actual option pop cloned the newly inserted host");
            assert!(!view.same_node(&host, &copy).unwrap());
        }
        if result.is_err() {
            assert!(driver.completed_native_view().is_err());
        }
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        match result.err() {
            Some(FragmentExecutionError::Head(head::HeadError::Kernel(KernelError::Session(
                actual,
            )))) => assert_eq!(actual, SessionError::ContextReactionUnavailable),
            actual => panic!("actual new clone must refuse the reaction domain: {actual:?}"),
        }
        assert_eq!(
            first,
            Some(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            ))
        );
    }

    #[test]
    fn details_fragment_closed_and_name_only_preserve_supported_creation() {
        for source in [
            "<details></details>",
            "<details name=g><summary>A</summary></details>",
        ] {
            let request = request(source, "div", HtmlScriptingStatus::Disabled);
            let mut driver = initialized_driver(&request);
            complete(&mut driver);
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }
    #[test]
    fn details_fragment_open_requires_actual_notification_capability_before_take() {
        super::owned_fragment_details_effect_tests::unproved_node_refuses_before_take();
    }
    #[test]
    fn radio_fragment_detached_groups_normalize_without_inventing_document_connection() {
        let request = request(
            "<input type=radio name=g checked><input type=radio name=g checked>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &mut driver
            .head
            .as_mut()
            .unwrap()
            .state
            .as_mut()
            .unwrap()
            .before
            .kernel
            .session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let (a, b) = {
            let mut view = session.native_view(&mut driver.control).unwrap();
            (
                view.child_at(&output, 0).unwrap().unwrap(),
                view.child_at(&output, 1).unwrap().unwrap(),
            )
        };
        // Both fresh checked flags were set while their roots were singletons;
        // disconnected insertion is not the becomes-connected trigger.
        assert!(
            session
                .input_state(&mut driver.control, &a)
                .unwrap()
                .flags
                .checkedness()
        );
        assert!(
            session
                .input_state(&mut driver.control, &b)
                .unwrap()
                .flags
                .checkedness()
        );
        session
            .set_input_checked(&mut driver.control, &b, true)
            .unwrap();
        assert!(
            !session
                .input_state(&mut driver.control, &a)
                .unwrap()
                .flags
                .checkedness()
        );
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn radio_captured_shadow_connection_requires_domain_only_for_actual_checked_trigger() {
        for checked in [false, true] {
            let source = if checked {
                "<template shadowrootmode=open><input type=radio name=g checked></template>"
            } else {
                "<template shadowrootmode=open><input type=radio name=g></template>"
            };
            let request = shadow_request(source, true);
            let mut driver = initialized_driver(&request);
            if checked {
                let error = loop {
                    match driver.step() {
                        Ok(HeadStep::Consumed) => {}
                        Err(error) => break error,
                        Ok(other) => {
                            panic!("expected actual captured-domain refusal, got {other:?}")
                        }
                    }
                };
                assert!(
                    matches!(
                        error,
                        FragmentExecutionError::Head(head::HeadError::Kernel(
                            KernelError::Session(SessionError::Stop(
                                ParseStop::UnsupportedStandaloneState(
                                    crate::parser::control::StandaloneUnsupported::Provenance
                                )
                            ))
                        ))
                    ),
                    "{error:?}"
                );
                let first = driver.control.stop().unwrap();
                assert_eq!(
                    first,
                    ParseStop::UnsupportedStandaloneState(
                        crate::parser::control::StandaloneUnsupported::Provenance
                    )
                );
                assert!(driver.step().is_err());
                assert_eq!(driver.control.stop(), Some(first));
                assert!(driver.completed_native_view().is_err());
            } else {
                complete(&mut driver);
            }
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
        }
    }
    #[test]
    fn radio_owned_shadow_connection_normalizes_only_its_actual_ordinary_tree() {
        use crate::construction::context::QualifiedFragmentHost;
        let request = shadow_request(
            "<div><template shadowrootmode=open><input type=radio name=g checked></template></div><input type=radio name=g checked>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let session = &mut driver
            .head
            .as_mut()
            .unwrap()
            .state
            .as_mut()
            .unwrap()
            .before
            .kernel
            .session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let effect = session
            .declarative_shadow(&mut driver.control, 0)
            .unwrap()
            .unwrap();
        let QualifiedFragmentHost::Owned(host) = effect.host else {
            panic!("actual owned host")
        };
        let contents = effect.contents;
        let (inner, outer) = {
            let mut view = session.native_view(&mut driver.control).unwrap();
            (
                view.child_at(&contents, 0).unwrap().unwrap(),
                view.child_at(&output, 1).unwrap().unwrap(),
            )
        };
        assert!(
            session
                .input_state(&mut driver.control, &inner)
                .unwrap()
                .flags
                .checkedness()
        );
        assert!(
            session
                .input_state(&mut driver.control, &outer)
                .unwrap()
                .flags
                .checkedness()
        );
        // Actual private Document P is retained by this fragment driver. Moving
        // the owned host beneath P makes its existing shadow-including root P.
        let document = session.root(&mut driver.control).unwrap();
        session
            .insert(&mut driver.control, &document, &host, None, None)
            .unwrap();
        session
            .insert(&mut driver.control, &contents, &outer, None, None)
            .unwrap();
        assert!(
            !session
                .input_state(&mut driver.control, &inner)
                .unwrap()
                .flags
                .checkedness()
        );
        assert!(
            session
                .input_state(&mut driver.control, &outer)
                .unwrap()
                .flags
                .checkedness()
        );
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
    #[test]
    fn radio_dirty_checked_attribute_removal_does_not_trigger_group_normalization() {
        let request = request(
            "<input type=radio name=g checked><input type=radio name=g checked>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = initialized_driver(&request);
        let mut first = None;
        for _ in 0..128 {
            assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            let output = session.fragment_output(&mut driver.control).unwrap();
            let mut view = session.native_view(&mut driver.control).unwrap();
            if let Some(node) = view.child_at(&output, 0).unwrap() {
                first = Some(node);
                break;
            }
        }
        let first = first.expect("actual first void input before second token");
        let session = &mut driver
            .head
            .as_mut()
            .unwrap()
            .state
            .as_mut()
            .unwrap()
            .before
            .kernel
            .session;
        session
            .set_input_checked(&mut driver.control, &first, true)
            .unwrap();
        complete(&mut driver);
        let session = &mut driver
            .head
            .as_mut()
            .unwrap()
            .state
            .as_mut()
            .unwrap()
            .before
            .kernel
            .session;
        let output = session.fragment_output(&mut driver.control).unwrap();
        let second = {
            let mut view = session.native_view(&mut driver.control).unwrap();
            view.child_at(&output, 1).unwrap().unwrap()
        };
        assert!(
            session
                .input_state(&mut driver.control, &first)
                .unwrap()
                .flags
                .dirty_checkedness()
        );
        assert!(
            session
                .input_state(&mut driver.control, &second)
                .unwrap()
                .flags
                .checkedness()
        );
        session
            .remove_input_attribute(
                &mut driver.control,
                &first,
                crate::construction::InputGroupAttribute::Checked,
            )
            .unwrap();
        assert!(
            session
                .input_state(&mut driver.control, &first)
                .unwrap()
                .flags
                .checkedness()
        );
        assert!(
            session
                .input_state(&mut driver.control, &second)
                .unwrap()
                .flags
                .checkedness()
        );
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn connected_owned_shadow_checkbox_resolves_nonancestor_explicit_form_in_its_tree() {
        // HTML877d reset-form-owner step4 resolves form= in the ordinary tree
        // when DOM3071 shadow-including-root proves the element connected.
        // The form is a preceding sibling, so ancestor fallback cannot satisfy it.
        let request = shadow_request(
            "<div><template shadowrootmode=open><form id=f></form></template></div><input type=checkbox form=f>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let (initially_unassociated, resolved_expected_form, final_reset, final_tree) = {
            let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
            let session = &mut state.before.kernel.session;
            let document = session.root(&mut driver.control).unwrap();
            let output = session.fragment_output(&mut driver.control).unwrap();
            let effect = session
                .declarative_shadow(&mut driver.control, 0)
                .unwrap()
                .expect("actual declarative shadow attachment");
            let crate::construction::context::QualifiedFragmentHost::Owned(host) = effect.host
            else {
                panic!("literal div must issue an owned host")
            };
            let contents = effect.contents;
            let (checkbox, form) = {
                let mut view = session.native_view(&mut driver.control).unwrap();
                assert!(matches!(
                    view.kind(&document).unwrap(),
                    NativeKind::Document(_)
                ));
                let actual_host = view.child_at(&output, 0).unwrap().unwrap();
                assert!(view.same_node(&host, &actual_host).unwrap());
                let shadow = view.shadow(&host).unwrap().unwrap();
                assert!(view.same_node(&contents, &shadow.contents).unwrap());
                assert_eq!(view.child_count(&contents).unwrap(), 1);
                (
                    view.child_at(&output, 1).unwrap().unwrap(),
                    view.child_at(&contents, 0).unwrap().unwrap(),
                )
            };
            let initially_unassociated = session
                .form_association_owner(&mut driver.control, &checkbox)
                .unwrap()
                .is_none();
            // This real host now has the actual internal Document as parent.
            session
                .insert(
                    &mut driver.control,
                    &document,
                    &host,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            // The same existing Checkbox enters the connected owned shadow tree.
            session
                .insert(
                    &mut driver.control,
                    &contents,
                    &checkbox,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            let owner = session
                .form_association_owner(&mut driver.control, &checkbox)
                .unwrap();
            let association = session
                .form_association_state(&mut driver.control, &checkbox)
                .unwrap();
            let final_reset = !association.parser_inserted
                && association.evidence == crate::construction::FormAssociationEvidence::Reset;
            let (resolved_expected_form, final_tree) = {
                let mut view = session.native_view(&mut driver.control).unwrap();
                let resolved_expected_form = owner
                    .as_ref()
                    .map(|owner| view.same_node(owner, &form).unwrap())
                    .unwrap_or(false);
                let first = view.child_at(&contents, 0).unwrap().unwrap();
                let second = view.child_at(&contents, 1).unwrap().unwrap();
                let final_tree = view.child_count(&contents).unwrap() == 2
                    && view.same_node(&first, &form).unwrap()
                    && view.same_node(&second, &checkbox).unwrap();
                (resolved_expected_form, final_tree)
            };
            (
                initially_unassociated,
                resolved_expected_form,
                final_reset,
                final_tree,
            )
        };
        // Complete both real owner cleanup and zero-storage verification BEFORE
        // the discriminatory form-owner assertion; no cloned payload observers.
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert_eq!(driver.control.stop(), None);
        assert!(initially_unassociated);
        assert!(final_reset);
        assert!(final_tree);
        assert!(
            resolved_expected_form,
            "connected owned-shadow Checkbox must resolve sibling form id=f"
        );
    }

    #[test]
    fn radio_unchecked_explicit_form_follows_nested_owned_shadow_connection_only() {
        use crate::construction::context::QualifiedFragmentHost;
        let request = shadow_request(
            "<form id=f></form><div><template shadowrootmode=open><div><template shadowrootmode=open><form id=f></form></template></div></template></div><input type=radio name=g form=f>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let (associated, no_light_tree_match, disconnected_null, unchecked) = {
            let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
            let session = &mut state.before.kernel.session;
            let document = session.root(&mut driver.control).unwrap();
            let output = session.fragment_output(&mut driver.control).unwrap();
            let outer = session
                .declarative_shadow(&mut driver.control, 0)
                .unwrap()
                .unwrap();
            let inner = session
                .declarative_shadow(&mut driver.control, 1)
                .unwrap()
                .unwrap();
            let QualifiedFragmentHost::Owned(outer_host) = outer.host else {
                panic!("actual outer owned host")
            };
            let QualifiedFragmentHost::Owned(inner_host) = inner.host else {
                panic!("actual inner owned host")
            };
            let outer_contents = outer.contents;
            let inner_contents = inner.contents;
            let (radio, light_form, shadow_form) = {
                let mut view = session.native_view(&mut driver.control).unwrap();
                let nested_host = view.child_at(&outer_contents, 0).unwrap().unwrap();
                assert!(view.same_node(&nested_host, &inner_host).unwrap());
                (
                    view.child_at(&output, 2).unwrap().unwrap(),
                    view.child_at(&output, 0).unwrap().unwrap(),
                    view.child_at(&inner_contents, 0).unwrap().unwrap(),
                )
            };
            session
                .insert(
                    &mut driver.control,
                    &document,
                    &outer_host,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            session
                .insert(
                    &mut driver.control,
                    &inner_contents,
                    &radio,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            let owner = session
                .form_association_owner(&mut driver.control, &radio)
                .unwrap()
                .unwrap();
            let (associated, no_light_tree_match) = {
                let mut view = session.native_view(&mut driver.control).unwrap();
                (
                    view.same_node(&owner, &shadow_form).unwrap(),
                    !view.same_node(&owner, &light_form).unwrap(),
                )
            };
            // Actual disconnection then another actual insertion resets ownership.
            // The ordinary sibling form alone cannot satisfy the connected branch.
            session
                .insert(
                    &mut driver.control,
                    &output,
                    &outer_host,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            session
                .insert(
                    &mut driver.control,
                    &output,
                    &radio,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            session
                .insert(
                    &mut driver.control,
                    &inner_contents,
                    &radio,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            let disconnected_null = session
                .form_association_owner(&mut driver.control, &radio)
                .unwrap()
                .is_none();
            let unchecked = !session
                .input_state(&mut driver.control, &radio)
                .unwrap()
                .flags
                .checkedness();
            (
                associated,
                no_light_tree_match,
                disconnected_null,
                unchecked,
            )
        };
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert_eq!(driver.control.stop(), None);
        assert!(associated && no_light_tree_match && disconnected_null && unchecked);
    }

    #[test]
    fn radio_unchecked_explicit_form_requires_influencing_captured_shadow_host() {
        for kind in ["checkbox", "radio"] {
            let source =
                format!("<template shadowrootmode=open><input type={kind} form=f></template>");
            let request = shadow_request(&source, true);
            let mut driver = initialized_driver(&request);
            let mut refused = false;
            for _ in 0..128 {
                if driver.step().is_err() {
                    refused = true;
                    break;
                }
            }
            let first = driver.control.stop();
            assert!(driver.step().is_err());
            assert_eq!(driver.control.stop(), first);
            assert!(driver.completed_native_view().is_err());
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
            assert!(refused);
            assert_eq!(
                first,
                Some(ParseStop::UnsupportedStandaloneState(
                    crate::parser::control::StandaloneUnsupported::Provenance
                ))
            );
        }
    }

    #[test]
    fn radio_shadow_form_insertion_cancel_preserves_first_stop_and_cleanup() {
        let request = shadow_request(
            "<div><template shadowrootmode=open><form id=f></form></template></div><input type=radio form=f>",
            true,
        );
        let mut driver = initialized_driver(&request);
        complete(&mut driver);
        let result = {
            let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
            let session = &mut state.before.kernel.session;
            let document = session.root(&mut driver.control).unwrap();
            let output = session.fragment_output(&mut driver.control).unwrap();
            let effect = session
                .declarative_shadow(&mut driver.control, 0)
                .unwrap()
                .unwrap();
            let crate::construction::context::QualifiedFragmentHost::Owned(host) = effect.host
            else {
                panic!("actual owned host")
            };
            let contents = effect.contents;
            let radio = {
                let mut view = session.native_view(&mut driver.control).unwrap();
                view.child_at(&output, 1).unwrap().unwrap()
            };
            session
                .insert(
                    &mut driver.control,
                    &document,
                    &host,
                    None,
                    Some(&state.script.as_ref().unwrap().capability),
                )
                .unwrap();
            request.cancellation().cancel();
            session.insert(
                &mut driver.control,
                &contents,
                &radio,
                None,
                Some(&state.script.as_ref().unwrap().capability),
            )
        };
        assert!(driver.completed_native_view().is_err());
        let first = driver.control.stop();
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        assert_eq!(result, Err(SessionError::Stop(ParseStop::Cancelled)));
        assert_eq!(first, Some(ParseStop::Cancelled));
        assert_eq!(driver.control.stop(), first);
    }
}

#[cfg(test)]
mod owned_fragment_details_tests {
    use super::*;
    use crate::construction::native_view::{NativeKind, NativeString, NativeView};
    use crate::*;

    #[derive(Debug, PartialEq)]
    struct Detail {
        attributes: Vec<(Vec<u16>, Vec<u16>)>,
    }
    #[derive(Debug, PartialEq)]
    struct Snapshot {
        details: Vec<Detail>,
        texts: Vec<Vec<u16>>,
        templates: usize,
        shadows: usize,
        distinct_details: bool,
    }
    struct Case {
        label: &'static str,
        source: &'static str,
        expected: Snapshot,
    }
    fn units(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }
    fn detail(attributes: &[(&str, &str)]) -> Detail {
        Detail {
            attributes: attributes
                .iter()
                .map(|(n, v)| (units(n), units(v)))
                .collect(),
        }
    }
    fn expected(
        details: Vec<Detail>,
        texts: &[&str],
        templates: usize,
        shadows: usize,
    ) -> Snapshot {
        Snapshot {
            details,
            texts: texts.iter().map(|v| units(v)).collect(),
            templates,
            shadows,
            distinct_details: true,
        }
    }
    fn read_units(view: &mut NativeView<'_>, value: NativeString<'_>) -> Vec<u16> {
        (0..value.len())
            .map(|i| view.unit(&value, i).unwrap().unwrap())
            .collect()
    }
    fn snapshot(driver: &mut ContextFragmentDriver<'_>) -> Snapshot {
        let (root, mut view) = driver.completed_native_view().unwrap();
        let mut result = Snapshot {
            details: Vec::new(),
            texts: Vec::new(),
            templates: 0,
            shadows: 0,
            distinct_details: true,
        };
        let mut pending = vec![root];
        let mut detail_nodes = Vec::new();
        // The fixtures are finite. Read only the actual owned tree and its
        // template/shadow contents; captured hosts are never copied or traversed.
        for _ in 0..256 {
            let Some(node) = pending.pop() else {
                return result;
            };
            match view.kind(&node).unwrap() {
                NativeKind::Element(name) => {
                    let local = read_units(&mut view, name.local);
                    if local == units("details") {
                        assert_eq!(
                            read_units(&mut view, name.namespace.unwrap()),
                            units("http://www.w3.org/1999/xhtml")
                        );
                        for previous in &detail_nodes {
                            result.distinct_details &= !view.same_node(previous, &node).unwrap();
                        }
                        detail_nodes.push(node.clone());
                        let attributes = (0..view.attribute_count(&node).unwrap())
                            .map(|position| {
                                let attribute =
                                    view.attribute_at(&node, position).unwrap().unwrap();
                                assert!(attribute.name.namespace.is_none());
                                assert!(attribute.name.prefix.is_none());
                                (
                                    read_units(&mut view, attribute.name.local),
                                    read_units(&mut view, attribute.value),
                                )
                            })
                            .collect();
                        result.details.push(Detail { attributes });
                    }
                }
                NativeKind::Text(value) => result.texts.push(read_units(&mut view, value)),
                _ => {}
            }
            if let Some(contents) = view.template_contents(&node).unwrap() {
                result.templates += 1;
                pending.push(contents);
            }
            if let Some(shadow) = view.shadow(&node).unwrap() {
                result.shadows += 1;
                assert_eq!(shadow.properties.mode(), HtmlShadowRootMode::Closed);
                assert!(shadow.properties.clonable());
                pending.push(shadow.contents);
            }
            for position in (0..view.child_count(&node).unwrap()).rev() {
                pending.push(view.child_at(&node, position).unwrap().unwrap());
            }
        }
        panic!("finite literal owned tree exceeded its observation bound");
    }

    #[test]
    fn owned_fragment_details_open_peers_templates_and_real_clones_complete() {
        // HTML877d Details attribute/insertion hooks: initial open is added
        // while detached; insertion closes the given second grouped element.
        // Empty/absent/different names do not group. Templates and shadows
        // have independent ordinary trees. Actual selectedcontent cloning
        // retains attributes until insertion into the source's ordinary tree.
        let cases = [
            Case {
                label: "name then open; first peer survives",
                source: "<details id=a name=g open=v><summary>A</summary></details><details id=b name=g open=w><summary>B</summary></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "g")]),
                    ],
                    &["A", "B", "Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "open then name; removal reindexes the later name",
                source: "<details id=a open=v name=g></details><details id=b open=w name=g></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("open", "v"), ("name", "g")]),
                        detail(&[("id", "b"), ("name", "g")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "empty names do not group",
                source: "<details id=a name='' open=v></details><details id=b name='' open=w></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", ""), ("open", "v")]),
                        detail(&[("id", "b"), ("name", ""), ("open", "w")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "absent names do not group",
                source: "<details id=a open=v></details><details id=b open=w></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("open", "v")]),
                        detail(&[("id", "b"), ("open", "w")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "ASCII name values are case sensitive",
                source: "<details id=a name=g open=v></details><details id=b name=G open=w></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "G"), ("open", "w")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "nonASCII name values are exact",
                source: "<details id=a name=é open=v></details><details id=b name=É open=w></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "é"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "É"), ("open", "w")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "duplicate open keeps first value and unrelated spelling",
                source: "<details id=a name=g open=v open=w x:open=q></details><details id=b name=g open=x></details><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v"), ("x:open", "q")]),
                        detail(&[("id", "b"), ("name", "g")]),
                    ],
                    &["Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "template contents are a separate group tree",
                source: "<details id=a name=g open=v></details><template><details id=b name=g open=w></details><details id=c name=g open=x></details></template><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "g"), ("open", "w")]),
                        detail(&[("id", "c"), ("name", "g")]),
                    ],
                    &["Z"],
                    1,
                    0,
                ),
            },
            Case {
                label: "owned shadow contents are a separate group tree",
                source: "<details id=a name=g open=v></details><div><template shadowrootmode=closed shadowrootclonable shadowrootcustomelementregistry><details id=b name=g open=w></details><details id=c name=g open=x></details></template></div><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "g"), ("open", "w")]),
                        detail(&[("id", "c"), ("name", "g")]),
                    ],
                    &["Z"],
                    0,
                    1,
                ),
            },
            Case {
                label: "selectedcontent clone closes on actual insertion",
                source: "<select><button><selectedcontent></selectedcontent></button><option selected><details id=a name=g open=v><summary>A</summary></details></option></select><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g")]),
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                    ],
                    &["A", "A", "Z"],
                    0,
                    0,
                ),
            },
            Case {
                label: "selectedcontent clones independent owned shadow groups",
                source: "<select><button><selectedcontent></selectedcontent></button><option selected><div><template shadowrootmode=closed shadowrootclonable shadowrootcustomelementregistry><details id=a name=g open=v></details><details id=b name=g open=w></details></template></div></option></select><p>Z</p>",
                expected: expected(
                    vec![
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "g")]),
                        detail(&[("id", "a"), ("name", "g"), ("open", "v")]),
                        detail(&[("id", "b"), ("name", "g")]),
                    ],
                    &["Z"],
                    0,
                    2,
                ),
            },
        ];
        let mut completed = Vec::new();
        let mut stopped = Vec::new();
        let mut observed = Vec::new();
        let mut expected_snapshots = Vec::new();
        for case in cases {
            let request = tests::shadow_request(case.source, true);
            let mut driver = initialized_driver(&request);
            let mut outcome = None;
            for _ in 0..20_000 {
                match driver.step() {
                    Ok(HeadStep::Consumed) => {}
                    terminal => {
                        outcome = Some(terminal);
                        break;
                    }
                }
            }
            let outcome = outcome.expect("finite actual fragment reached a terminal result");
            let complete = matches!(outcome, Ok(HeadStep::Complete));
            let first = driver.control.stop();
            if complete {
                observed.push((case.label, snapshot(&mut driver)));
            } else {
                assert!(
                    matches!(
                        outcome,
                        Err(FragmentExecutionError::Head(head::HeadError::Kernel(
                            KernelError::Session(SessionError::Stop(
                                ParseStop::UnsupportedStandaloneState(
                                    StandaloneUnsupported::DetailsNotification
                                )
                            ))
                        )))
                    ),
                    "{}: {outcome:?}",
                    case.label
                );
                assert_eq!(
                    first,
                    Some(ParseStop::UnsupportedStandaloneState(
                        StandaloneUnsupported::DetailsNotification
                    ))
                );
                let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
                assert!(matches!(&state.pending.as_ref().unwrap().payload,
                        super::super::prologue::Payload::Tag(Some(tag))
                        if tag.attributes.as_slice().iter().any(|a|
                            a.name.value.text().code_units() == [111, 112, 101, 110])));
                stopped.push((case.label, first));
            }
            // Every stimulus runs and real cleanup succeeds before the grouped
            // completion assertion. Native snapshots own UTF16 copies only.
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
            assert!(driver.head.is_none());
            assert_eq!(driver.control.stop(), first);
            completed.push((case.label, complete));
            expected_snapshots.push((case.label, case.expected));
        }
        assert!(
            completed.iter().all(|(_, complete)| *complete),
            "owned fragment Details did not complete after all fixtures and cleanup: {completed:?}; typed stops: {stopped:?}"
        );
        assert_eq!(observed, expected_snapshots);
    }
}

#[cfg(test)]
mod owned_fragment_details_effect_tests {
    use super::*;
    use crate::construction::{CreationContext, CreationLocation, NativeView, ParserArenaSession};
    use crate::parser::tokenizer::{
        tests::LexerFixture,
        token::{Attribute, Event, Tag, TokenKind},
    };
    use crate::*;

    fn owner(
        request: &HtmlFragmentParseRequest,
    ) -> (ParseControl, ParserArenaSession, ConstructionCapability<'_>) {
        let mut driver = initialized_driver(request);
        let stage = driver.head.as_mut().unwrap();
        // Reuse the existing extraction's real controlled lexer/stack disposal;
        // the session and source/controller/document-bound issuer survive.
        stage
            .prepare_standalone_fixture(&mut driver.control)
            .unwrap();
        let (session, capability) = driver.head.take().unwrap().into_standalone_fixture();
        assert!(driver.failed_evidence.is_none());
        (driver.control, session, capability)
    }
    fn tag(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> (Tag, HtmlSourceRange) {
        let Event::Token(event) = lexer.next(c, false).unwrap().unwrap() else {
            panic!("literal starts with a real tag");
        };
        let token = event.into_token(c).unwrap();
        let TokenKind::Tag(tag) = token.kind else {
            panic!("actual tag")
        };
        (tag, token.source)
    }
    fn attribute(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> Option<Attribute> {
        let (mut tag, _) = tag(lexer, c);
        assert_eq!(tag.attributes.len(), 1);
        let attribute = tag.attributes.try_pop(c).unwrap();
        tag.name.release(c).unwrap();
        tag.attributes.release(c).unwrap();
        attribute
    }
    fn release_attribute(input: &mut Option<Attribute>, c: &mut ParseControl) {
        if let Some(attribute) = input.take() {
            attribute.name.release(c).unwrap();
            attribute.value.release(c).unwrap();
        }
    }
    fn detail(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
        proof: &ConstructionCapability<'_>,
        parent: &ParserNode,
    ) -> ParserNode {
        let (tag, source) = tag(lexer, c);
        session
            .allocate_details_for_creation(
                c,
                &mut Some(tag),
                CreationLocation {
                    intended_parent: parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &source,
                proof,
            )
            .unwrap()
    }
    fn attributes(
        session: &ParserArenaSession,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Vec<(Vec<u16>, Vec<u16>)> {
        fn units(
            view: &mut NativeView<'_>,
            text: crate::construction::native_view::NativeString<'_>,
        ) -> Vec<u16> {
            (0..text.len())
                .map(|i| view.unit(&text, i).unwrap().unwrap())
                .collect()
        }
        let mut view = session.native_view(c).unwrap();
        (0..view.attribute_count(node).unwrap())
            .map(|i| {
                let attribute = view.attribute_at(node, i).unwrap().unwrap();
                assert!(attribute.name.namespace.is_none());
                (
                    units(&mut view, attribute.name.local),
                    units(&mut view, attribute.value),
                )
            })
            .collect()
    }
    fn literals(input: &[(&str, &str)]) -> Vec<(Vec<u16>, Vec<u16>)> {
        input
            .iter()
            .map(|(n, v)| (n.encode_utf16().collect(), v.encode_utf16().collect()))
            .collect()
    }
    pub(super) fn unproved_node_refuses_before_take() {
        let request = tests::request(
            "<details name=g><i open=new>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let (mut c, mut session, proof) = owner(&request);
        let parent = session.fragment_output(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let (tag, _) = tag(&mut lexer, &mut c);
        // A real generic lexical creation did not issue/bind Details eligibility.
        // Matching local name, inert profile and null registry do not grant it.
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &parent,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let mut input = attribute(&mut lexer, &mut c);
        let pointer = input
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .shared_code_units()
            .as_ptr();
        let before = attributes(&session, &mut c, &node);
        assert_eq!(before, literals(&[("name", "g")]));
        let expected = ParseStop::UnsupportedStandaloneState(
            crate::parser::control::StandaloneUnsupported::DetailsNotification,
        );
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::Stop(expected))
        );
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr(),
            pointer
        );
        let usage = c.usage().clone();
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::Stop(expected))
        );
        assert_eq!(c.usage(), &usage);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(expected))
        );
        release_attribute(&mut input, &mut c);
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::assert_released(&c);
        assert_eq!(c.stop(), Some(expected));
    }
    #[test]
    fn owned_fragment_details_managed_name_open_and_insertion_use_distinct_rules() {
        let request = tests::request(
            "<details name=g open=v><details name=h open=w><details name=g open=z><i name=g><i open=x><i open=y>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let (mut c, mut session, proof) = owner(&request);
        let parent = session.fragment_output(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let first = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &first, None, Some(&proof))
            .unwrap();
        let second = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &second, Some(&first), Some(&proof))
            .unwrap();
        let third = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        session
            .insert(&mut c, &parent, &third, Some(&first), Some(&proof))
            .unwrap();
        assert_eq!(
            attributes(&session, &mut c, &third),
            literals(&[("name", "g")])
        );
        assert_eq!(
            attributes(&session, &mut c, &first),
            literals(&[("name", "g"), ("open", "v")])
        );
        let mut name = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut name, &proof)
            .unwrap();
        assert!(name.is_none());
        assert_eq!(
            attributes(&session, &mut c, &first),
            literals(&[("name", "g"), ("open", "v")])
        );
        assert_eq!(
            attributes(&session, &mut c, &second),
            literals(&[("name", "g")])
        );
        let mut open = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut open, &proof)
            .unwrap();
        assert!(open.is_none());
        assert_eq!(
            attributes(&session, &mut c, &first),
            literals(&[("name", "g")])
        );
        assert_eq!(
            attributes(&session, &mut c, &second),
            literals(&[("name", "g"), ("open", "x")])
        );
        let mut replacement = attribute(&mut lexer, &mut c);
        session
            .set_details_attribute(&mut c, &second, &mut replacement, &proof)
            .unwrap();
        assert_eq!(
            attributes(&session, &mut c, &second),
            literals(&[("name", "g"), ("open", "y")])
        );
        session
            .remove_details_attribute(&mut c, &second, HtmlDetailsAttribute::Open)
            .unwrap();
        assert_eq!(
            attributes(&session, &mut c, &second),
            literals(&[("name", "g")])
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::assert_released(&c);
        assert_eq!(c.stop(), None);
    }
    #[test]
    fn owned_fragment_details_aliased_incoming_attribute_refuses_then_real_retry_succeeds() {
        let request = tests::request(
            "<details name=g><i open=v>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let (mut c, mut session, proof) = owner(&request);
        let parent = session.fragment_output(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        let mut input = attribute(&mut lexer, &mut c);
        let alias = input.as_ref().unwrap().value.value.text().clone();
        let pointer = alias.shared_code_units().as_ptr();
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::AliasedPayload)
        );
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr(),
            pointer
        );
        assert_eq!(
            attributes(&session, &mut c, &node),
            literals(&[("name", "g")])
        );
        assert_eq!(c.stop(), None);
        drop(alias);
        session
            .set_details_attribute(&mut c, &node, &mut input, &proof)
            .unwrap();
        assert!(input.is_none());
        assert_eq!(
            attributes(&session, &mut c, &node),
            literals(&[("name", "g"), ("open", "v")])
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::assert_released(&c);
    }
    #[test]
    fn owned_fragment_details_foreign_capability_and_cancel_refuse_before_take() {
        for cancelled in [false, true] {
            let request = tests::request(
                "<details name=g><i open=v>",
                "div",
                HtmlScriptingStatus::Disabled,
            );
            let (mut c, mut session, proof) = owner(&request);
            let parent = session.fragment_output(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
            let mut input = attribute(&mut lexer, &mut c);
            let pointer = input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr();
            let expected = if cancelled {
                request.cancellation().cancel();
                let expected = ParseStop::Cancelled;
                assert_eq!(
                    session.set_details_attribute(&mut c, &node, &mut input, &proof),
                    Err(SessionError::Stop(expected))
                );
                expected
            } else {
                let foreign_request = tests::request("", "div", HtmlScriptingStatus::Disabled);
                let (mut foreign, mut other, foreign_proof) = owner(&foreign_request);
                let unchanged = foreign.usage().clone();
                let expected =
                    ParseStop::InternalInvariant(crate::parser::control::Invariant::ForeignControl);
                assert_eq!(
                    session.set_details_attribute(&mut c, &node, &mut input, &foreign_proof),
                    Err(SessionError::Stop(expected))
                );
                assert_eq!(foreign.usage(), &unchanged);
                other.release(&mut foreign).unwrap();
                super::super::tests::assert_released(&foreign);
                expected
            };
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .value
                    .value
                    .text()
                    .shared_code_units()
                    .as_ptr(),
                pointer
            );
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(expected))
            );
            let usage = c.usage().clone();
            assert_eq!(
                session.set_details_attribute(&mut c, &node, &mut input, &proof),
                Err(SessionError::Stop(expected))
            );
            assert_eq!(c.usage(), &usage);
            release_attribute(&mut input, &mut c);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::assert_released(&c);
            assert_eq!(c.stop(), Some(expected));
        }
    }
    #[test]
    fn owned_fragment_details_foreign_source_refuses_and_preserves_both_actual_owners() {
        let request = tests::request("<details name=g>", "div", HtmlScriptingStatus::Disabled);
        let (mut c, mut session, proof) = owner(&request);
        let parent = session.fragment_output(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
        let foreign_request = tests::request("<i open=v>", "div", HtmlScriptingStatus::Disabled);
        let (mut foreign, mut other, _) = owner(&foreign_request);
        let mut foreign_lexer =
            LexerFixture::new(foreign_request.source(), State::Data, None, &mut foreign).unwrap();
        let mut input = attribute(&mut foreign_lexer, &mut foreign);
        let pointer = input
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .shared_code_units()
            .as_ptr();
        let foreign_usage = foreign.usage().clone();
        let expected =
            ParseStop::InternalInvariant(crate::parser::control::Invariant::ForeignSource);
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::Stop(expected))
        );
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr(),
            pointer
        );
        assert_eq!(foreign.usage(), &foreign_usage);
        let usage = c.usage().clone();
        assert_eq!(
            session.set_details_attribute(&mut c, &node, &mut input, &proof),
            Err(SessionError::Stop(expected))
        );
        assert_eq!(c.usage(), &usage);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(expected))
        );
        release_attribute(&mut input, &mut foreign);
        foreign_lexer.release(&mut foreign).unwrap();
        other.release(&mut foreign).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::assert_released(&foreign);
        super::super::tests::assert_released(&c);
        assert_eq!(c.stop(), Some(expected));
        assert_eq!(foreign.stop(), None);
    }
    #[test]
    fn owned_fragment_details_managed_slot_exact_and_one_short_retain_cleanup() {
        for cap in [1, 2] {
            let base = tests::request(
                "<details name=g><i open=v>",
                "div",
                HtmlScriptingStatus::Disabled,
            );
            let request = HtmlFragmentParseRequest::new(
                base.source().clone(),
                base.context().clone(),
                base.limits()
                    .into_builder()
                    .managed_attributes(cap)
                    .finish()
                    .unwrap(),
            )
            .unwrap();
            let (mut c, mut session, proof) = owner(&request);
            let parent = session.fragment_output(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = detail(&mut session, &mut c, &mut lexer, &proof, &parent);
            let mut input = attribute(&mut lexer, &mut c);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedAttributes)
                    .current,
                1
            );
            let result = session.set_details_attribute(&mut c, &node, &mut input, &proof);
            if cap == 2 {
                result.unwrap();
                assert!(input.is_none());
                assert_eq!(
                    attributes(&session, &mut c, &node),
                    literals(&[("name", "g"), ("open", "v")])
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedAttributes)
                        .current,
                    2
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::ManagedAttributes,
                        ..
                    }))
                ));
                let first = c.stop().unwrap();
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
            }
            release_attribute(&mut input, &mut c);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            super::super::tests::assert_released(&c);
        }
    }
}

#[cfg(test)]
mod owned_fragment_details_completion_tests {
    use super::*;
    use crate::construction::context::QualifiedFragmentHost;
    use crate::*;

    #[test]
    fn owned_fragment_details_captured_attachment_has_local_members_and_unchanged_capture() {
        let request = tests::shadow_request(
            "<template shadowrootmode=open><details name=g open=v></details><details name=g open=w></details></template>",
            true,
        );
        let identity = request.context().capture().identity().clone();
        let usage = request.context().capture().usage();
        let target = request.context().context_element().clone();
        let target_snapshot = format!(
            "{:?}",
            request.context().capture().node(target.subject()).unwrap()
        );
        for _ in 0..2 {
            let mut driver = initialized_driver(&request);
            tests::complete(&mut driver);
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            let effect = session
                .declarative_shadow(&mut driver.control, 0)
                .unwrap()
                .unwrap();
            let QualifiedFragmentHost::Captured(host) = &effect.host else {
                panic!("actual captured attachment")
            };
            assert_eq!(host, &target);
            let mut view = session.native_view(&mut driver.control).unwrap();
            assert_eq!(view.child_count(&effect.contents).unwrap(), 2);
            let a = view.child_at(&effect.contents, 0).unwrap().unwrap();
            let b = view.child_at(&effect.contents, 1).unwrap().unwrap();
            assert!(!view.same_node(&a, &b).unwrap());
            assert_eq!(view.attribute_count(&a).unwrap(), 2);
            assert_eq!(view.attribute_count(&b).unwrap(), 1);
            let open = view.attribute_at(&a, 1).unwrap().unwrap();
            assert_eq!(
                (0..open.name.local.len())
                    .map(|i| view.unit(&open.name.local, i).unwrap().unwrap())
                    .collect::<Vec<_>>(),
                vec![111, 112, 101, 110]
            );
            assert_eq!(view.unit(&open.value, 0).unwrap(), Some(118));
            let output = session.fragment_output(&mut driver.control).unwrap();
            let mut view = session.native_view(&mut driver.control).unwrap();
            assert_eq!(view.child_count(&output).unwrap(), 0);
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
            assert_eq!(driver.control.stop(), None);
            assert_eq!(request.context().capture().identity(), &identity);
            assert_eq!(request.context().capture().usage(), usage);
            assert_eq!(
                format!(
                    "{:?}",
                    request.context().capture().node(target.subject()).unwrap()
                ),
                target_snapshot
            );
        }
    }
    #[test]
    fn owned_fragment_details_retirement_preserves_actual_original_ranges_and_frozen_output() {
        let request = tests::request(
            "<details name=g open></details><details name=g open></details>",
            "div",
            HtmlScriptingStatus::Disabled,
        );
        let mut driver = Some(initialized_driver(&request));
        tests::complete(driver.as_mut().unwrap());
        let owner = driver.as_mut().unwrap();
        let state = owner.head.as_mut().unwrap().state.as_mut().unwrap();
        let session = &mut state.before.kernel.session;
        session
            .prepare_native_associations(&mut owner.control, request.source())
            .unwrap();
        let mut retired = Vec::new();
        {
            let mut rows = session.native_associations(&mut owner.control).unwrap();
            let mut group = 0;
            while let Some(header) = rows.group(group).unwrap() {
                if header.retirement.is_some() {
                    let mut value = Some(rows.materialize_group(group).unwrap());
                    let units = value.as_ref().unwrap().text().code_units().to_vec();
                    let mut cursor = Some(rows.group_origin_cursor(group).unwrap());
                    let mut origins = Vec::new();
                    while let Some(origin) =
                        rows.next_group_origin(cursor.as_mut().unwrap()).unwrap()
                    {
                        assert_eq!(origin.source.source(), request.source().identity());
                        origins.push((origin.source.start(), origin.source.end()));
                    }
                    rows.release_group_origin_cursor(&mut cursor).unwrap();
                    rows.release_materialized_group(&mut value).unwrap();
                    retired.push((units, origins));
                }
                group += 1;
            }
        }
        assert_eq!(
            retired,
            vec![
                (
                    vec![111, 112, 101, 110],
                    vec![(47, 48), (48, 49), (49, 50), (50, 51)]
                ),
                (vec![], vec![])
            ]
        );
        let mut completed = ContextFragmentDriver::take_completed(&mut driver).unwrap();
        assert!(driver.is_none());
        completed.prepare_native_associations().unwrap();
        let mut parts = completed.freeze_parts().unwrap();
        {
            let fragment = parts.fragment().unwrap();
            let root = fragment.root();
            let nodes = root.children().collect::<Vec<_>>();
            assert_eq!(nodes.len(), 2);
            let HtmlNodeKind::Element(a) = fragment.node(&nodes[0]).unwrap().kind() else {
                panic!("first Details")
            };
            let HtmlNodeKind::Element(b) = fragment.node(&nodes[1]).unwrap().kind() else {
                panic!("second Details")
            };
            assert_eq!(
                a.attributes()
                    .iter()
                    .map(|a| a.name().local_name().code_units().to_vec())
                    .collect::<Vec<_>>(),
                vec![vec![110, 97, 109, 101], vec![111, 112, 101, 110]]
            );
            assert_eq!(
                b.attributes()
                    .iter()
                    .map(|a| a.name().local_name().code_units().to_vec())
                    .collect::<Vec<_>>(),
                vec![vec![110, 97, 109, 101]]
            );
        }
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn owned_fragment_details_every_actual_step_cancel_keeps_stop_and_cleanup_owner() {
        const SOURCE: &str =
            "<details name=g open></details><details name=g open></details><p>Z</p>";
        let baseline = tests::request(SOURCE, "div", HtmlScriptingStatus::Disabled);
        let mut driver = initialized_driver(&baseline);
        let mut steps = 0;
        for _ in 0..100 {
            steps += 1;
            if driver.step().unwrap() == HeadStep::Complete {
                break;
            }
        }
        assert!(steps > 4 && steps < 100);
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
        for cut in 0..steps {
            let request = tests::request(SOURCE, "div", HtmlScriptingStatus::Disabled);
            let mut driver = initialized_driver(&request);
            for _ in 0..cut {
                assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
            }
            request.cancellation().cancel();
            assert!(driver.step().is_err());
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            let usage = driver.control.usage().clone();
            assert!(driver.step().is_err());
            assert_eq!(driver.control.usage(), &usage);
            assert!(driver.completed_native_view().is_err());
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            driver.release().unwrap();
            super::super::tests::assert_released(&driver.control);
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        }
    }
}

#[cfg(test)]
mod shadow_reference_alias_characterization {
    use super::*;

    #[test]
    fn local_and_captured_shadow_reference_alias_refusal_retains_original_frozen_owner() {
        for source in [
            "<template shadowrootmode=open><p>A</p></template>",
            "<div><template shadowrootmode=open><p>A</p></template></div>",
        ] {
            let request = tests::shadow_request(source, true);
            let mut input = Some(initialized_driver(&request));
            tests::complete(input.as_mut().unwrap());
            let mut completed = ContextFragmentDriver::take_completed(&mut input).unwrap();
            let mut parts = completed.freeze_parts().unwrap();
            let (template, shadow) = {
                let arena = parts.arena.as_ref().unwrap();
                let effect = arena
                    .declarative_shadow(&mut parts.control, 0)
                    .unwrap()
                    .unwrap();
                assert_eq!(effect.source.source(), request.source().identity());
                assert_eq!(
                    request.source().text(effect.source).unwrap(),
                    crate::DocumentString::from("<template shadowrootmode=open>")
                );
                (effect.template, effect.contents)
            };
            let original_root = parts.fragment().unwrap().root().id();
            let alias = parts.fragment().unwrap().clone();
            let capacity = parts.control.usage().actual_capacity_bytes;
            assert!(parts.release().is_err());
            let first_stop = parts.control.stop();
            assert_eq!(
                first_stop,
                Some(ParseStop::InternalInvariant(
                    crate::parser::control::Invariant::SharedAllocationAliased
                ))
            );
            assert!(parts.arena.is_some());
            assert_eq!(parts.control.usage().actual_capacity_bytes, capacity);
            assert_eq!(parts.source_binding().0, request.source().identity());
            assert!(parts.fragment().is_err());
            let fragment = parts.arena.as_ref().unwrap().fragment().unwrap();
            assert_eq!(fragment.root().id(), original_root);
            // Pinned HTML877d template step7.11.6 readback remains exact even
            // while recovery is refused by an independently retained arena alias.
            assert_eq!(
                fragment.node(&template).unwrap().template_contents(),
                Some(shadow.clone())
            );
            let p = fragment.node(&shadow).unwrap().children().next().unwrap();
            let text = fragment.node(&p).unwrap().children().next().unwrap();
            assert!(matches!(fragment.node(&text).unwrap().kind(),
                crate::HtmlNodeKind::Text(value) if value.code_units() == [65]));
            drop(alias);
            parts.release().unwrap();
            crate::parser::tree_builder::tests::assert_released(&parts.control);
            assert_eq!(parts.control.stop(), first_stop);
        }
    }
}
