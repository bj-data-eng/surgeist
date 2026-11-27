//! Task progression for an attached Document's original HTML stop parsing.

use super::*;
use guest::ParsingEventFiring;
use surgeist_dom::{hosted::DomHostedHtmlLoadStep, identity::DomStamp};
use surgeist_html_parser::HtmlParseStop;

use crate::internal::{
    dom_event::{DomEventDispatchLimits, DomEventDispatchOutcome, ParsingEvent},
    dom_lifecycle::{
        DocumentTaskCustody, DomHostedHtmlStopParsing, HtmlCompletionTask, NativeTaskRunning,
    },
};

/// Outcome of servicing one task in the attached Document's HTML completion.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentHtmlCompletionProgress {
    /// This attachment has no pending HTML completion task.
    Idle,
    /// The EOF continuation set interactive readiness and finished the parser task.
    Interactive,
    /// The DOMContentLoaded event and its task-end checkpoint completed.
    DomContentLoaded,
    /// The original Window load task awaits an Object task in the native
    /// DOM-manipulation source. Its event loop remains available to that source.
    WaitingForObjectTask(surgeist_dom::task::DomElementTaskId),
    /// The native Window load task and its committed completion tail finished.
    Loaded,
    /// Native dispatch retains the next step of the same task.
    Retained(DomEventDispatchOutcome),
}

/// Refusal in an original hosted HTML completion operation.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentHtmlCompletionError {
    /// The actual script owner or checkpoint refused the requested operation.
    Script(DocumentScriptError),
    /// The hosted identity, task source, resource profile or timing refused it.
    Hosted(DomHostedRuntimeError),
    /// The original parser could not release its open-element stack.
    Parser(HtmlParseStop),
    /// The native event registry or a listener operation refused the event.
    Event(DocumentScriptEventError),
    /// The original element-task bank refused load-delay observation.
    ElementTask(crate::internal::element_tasks::DomElementTaskRuntimeError),
    /// An original EOF continuation is required before completion can begin.
    MissingContinuation,
}

impl fmt::Display for DocumentHtmlCompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Script(error) => error.fmt(f),
            Self::Hosted(error) => error.fmt(f),
            Self::Parser(error) => write!(
                f,
                "the original parser could not finish its stack: {error:?}"
            ),
            Self::Event(error) => error.fmt(f),
            Self::ElementTask(error) => error.fmt(f),
            Self::MissingContinuation => {
                f.write_str("HTML completion requires the original EOF continuation")
            }
        }
    }
}

impl std::error::Error for DocumentHtmlCompletionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Script(error) => Some(error),
            Self::Hosted(error) => Some(error),
            Self::Parser(_) => None,
            Self::Event(error) => Some(error),
            Self::ElementTask(error) => Some(error),
            Self::MissingContinuation => None,
        }
    }
}

impl From<DocumentScriptNativeError> for DocumentHtmlCompletionError {
    fn from(error: DocumentScriptNativeError) -> Self {
        Self::Script(error.into())
    }
}

impl From<DomHostedRuntimeError> for DocumentHtmlCompletionError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

#[derive(Clone, Copy)]
enum Step {
    Publish(DomHostedHtmlLoadStep),
    Dispatch(ParsingEvent),
    PopParserStack,
}

use DomHostedHtmlLoadStep as Load;
const PARSER_END: &[Step] = &[
    Step::Publish(Load::Interactive),
    Step::Dispatch(ParsingEvent::ReadyStateChange),
    Step::PopParserStack,
];
const CONTENT_LOADED: &[Step] = &[
    Step::Publish(Load::ContentLoadedStart),
    Step::Dispatch(ParsingEvent::DomContentLoaded),
    Step::Publish(Load::ContentLoadedEnd),
];
const WINDOW_LOAD: &[Step] = &[
    Step::Publish(Load::Complete),
    Step::Dispatch(ParsingEvent::ReadyStateChange),
    Step::Publish(Load::LoadStart),
    Step::Dispatch(ParsingEvent::WindowLoad),
    Step::Publish(Load::LoadEnd),
    Step::Publish(Load::PageShowing),
    Step::Dispatch(ParsingEvent::PageShow),
    Step::Publish(Load::Finished),
];

pub(super) struct HtmlCompletion {
    original: DomHostedHtmlStopParsing,
    checkpoint: DomStamp,
    task: HtmlCompletionTask,
    next_step: usize,
    custody: Option<DocumentTaskCustody>,
    timing: Option<u64>,
    firing: Option<ParsingEventFiring>,
    _backing: DomHostedRuntimeRetention,
}

impl DocumentScriptHost {
    pub(super) fn has_retained_html_completion_task(
        &self,
    ) -> Result<bool, DocumentScriptNativeError> {
        match &self.html_completion {
            Some(active) => active
                .try_borrow()
                .map(|active| active.custody.is_some())
                .map_err(|_| DocumentScriptNativeError::Busy),
            None => Ok(false),
        }
    }

    /// Submit the authentic EOF continuation for this attachment.
    /// An admission refusal leaves the continuation with the caller.
    pub fn begin_html_completion(
        &mut self,
        permit: &DocumentScriptPermit,
        continuation: &mut Option<DomHostedHtmlStopParsing>,
    ) -> Result<(), DocumentHtmlCompletionError> {
        self.check_permit(permit)?;
        if self.html_completion.is_some() {
            return Err(DomHostedRuntimeError::OperationPending.into());
        }
        let original = continuation
            .as_ref()
            .ok_or(DocumentHtmlCompletionError::MissingContinuation)?;
        let top = original.top_level();
        if top != self.attachment()?.target {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        self.guest_events()
            .map_err(DocumentScriptEventError::Guest)
            .map_err(DocumentHtmlCompletionError::Event)?;
        self.observe()
            .map_err(DocumentHtmlCompletionError::Script)?;
        let checkpoint = original.load_checkpoint();
        let owner = self.native.clone();
        let cancel = &owner.cancellation;
        let backing = self.with_native(|dom, hosted, _| {
            hosted.qualify_local_html_resources(dom, top, checkpoint, cancel)?;
            hosted.retain_script_payload(dom, size_of::<RefCell<HtmlCompletion>>(), cancel)
        })??;
        let original = continuation
            .take()
            .ok_or(DocumentHtmlCompletionError::MissingContinuation)?;
        self.html_completion = Some(Rc::new(RefCell::new(HtmlCompletion {
            original,
            checkpoint,
            task: HtmlCompletionTask::ParserEnd,
            next_step: 0,
            custody: None,
            timing: None,
            firing: None,
            _backing: backing,
        })));
        Ok(())
    }

    /// Service the next original HTML completion task on the attached Window.
    /// Native dispatch and checkpoint refusals leave the current task resumable.
    pub fn service_html_completion(
        &mut self,
        permit: &DocumentScriptPermit,
        limits: DomEventDispatchLimits,
    ) -> Result<DocumentHtmlCompletionProgress, DocumentHtmlCompletionError> {
        self.check_permit(permit)?;
        let Some(active) = self.html_completion.clone() else {
            return Ok(DocumentHtmlCompletionProgress::Idle);
        };
        let mut active = active
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let result = active.service(self, limits)?;
        if matches!(result, DocumentHtmlCompletionProgress::Loaded) {
            self.html_completion.take();
        }
        Ok(result)
    }
}

impl HtmlCompletion {
    fn service(
        &mut self,
        host: &mut DocumentScriptHost,
        limits: DomEventDispatchLimits,
    ) -> Result<DocumentHtmlCompletionProgress, DocumentHtmlCompletionError> {
        let owner = host.native.clone();
        let cancel = &owner.cancellation;
        let top = self.original.top_level();
        if self.task == HtmlCompletionTask::WindowLoad
            && self.custody.is_none()
            && let Some(task) = host
                .with_native(|dom, hosted, _| {
                    hosted.object_task_delaying_local_html_load(dom, top, cancel)
                })?
                .map_err(DocumentHtmlCompletionError::ElementTask)?
        {
            return Ok(DocumentHtmlCompletionProgress::WaitingForObjectTask(task));
        }
        let mut running = host.with_native(|dom, hosted, _| {
            hosted.enter_html_completion_task(dom, top, self.task, &mut self.custody, cancel)
        })??;
        let steps = match self.task {
            HtmlCompletionTask::ParserEnd => PARSER_END,
            HtmlCompletionTask::DomContentLoaded => CONTENT_LOADED,
            HtmlCompletionTask::WindowLoad => WINDOW_LOAD,
        };
        while let Some(step) = steps.get(self.next_step).copied() {
            match step {
                Step::Publish(step) => self.publish(host, &running, step, cancel)?,
                Step::PopParserStack => self
                    .original
                    .finish_open_elements()
                    .map_err(DocumentHtmlCompletionError::Parser)?,
                Step::Dispatch(event) => {
                    let firing = self.firing.get_or_insert_with(|| {
                        ParsingEventFiring::new(event, top.global(), top.document())
                    });
                    let outcome = host
                        .advance_parsing_event(firing, &running, limits)
                        .map_err(DocumentHtmlCompletionError::Event)?;
                    if !matches!(outcome, DomEventDispatchOutcome::Complete(_)) {
                        return Ok(DocumentHtmlCompletionProgress::Retained(outcome));
                    }
                    self.firing.take();
                }
            }
            self.next_step += 1;
        }
        host.with_native(|dom, _, _| running.checkpoint(dom, cancel))??;
        host.checkpoint()
            .map_err(DocumentHtmlCompletionError::Script)?;
        running.complete();
        drop(running);
        self.custody.take();
        self.next_step = 0;
        Ok(match self.task {
            HtmlCompletionTask::ParserEnd => {
                self.task = HtmlCompletionTask::DomContentLoaded;
                DocumentHtmlCompletionProgress::Interactive
            }
            HtmlCompletionTask::DomContentLoaded => {
                self.task = HtmlCompletionTask::WindowLoad;
                DocumentHtmlCompletionProgress::DomContentLoaded
            }
            HtmlCompletionTask::WindowLoad => DocumentHtmlCompletionProgress::Loaded,
        })
    }

    fn publish(
        &mut self,
        host: &mut DocumentScriptHost,
        running: &NativeTaskRunning,
        step: DomHostedHtmlLoadStep,
        cancel: &AtomicBool,
    ) -> Result<(), DocumentHtmlCompletionError> {
        let top = self.original.top_level();
        host.with_native(|dom, hosted, _| {
            if matches!(
                step,
                Load::Interactive
                    | Load::ContentLoadedStart
                    | Load::ContentLoadedEnd
                    | Load::Complete
            ) {
                hosted.qualify_local_html_resources(dom, top, self.checkpoint, cancel)?;
            }
            let time = match self.timing {
                Some(time) => time,
                None => *self
                    .timing
                    .insert(running.sample_html_load_time(dom, cancel)?),
            };
            hosted.with_current_top_level(dom, top, cancel, |source, dom| {
                source
                    .publish_local_html_load(
                        dom.raw_mut(),
                        top.document(),
                        top.global(),
                        step,
                        time,
                        cancel,
                    )
                    .map_err(DomHostedRuntimeError::from)
            })?;
            self.checkpoint = dom.snapshot().stamp();
            self.timing.take();
            Ok::<_, DomHostedRuntimeError>(())
        })??;
        Ok(())
    }
}
