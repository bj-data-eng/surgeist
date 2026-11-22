//! Qualification and timing for the original HTML completion tasks.

use super::*;
use surgeist_dom::identity::DomStamp;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlCompletionTask {
    ParserEnd,
    DomContentLoaded,
    WindowLoad,
}

impl HostedRuntime {
    pub(crate) fn enter_html_completion_task(
        &self,
        dom: &mut DomEventDom<'_>,
        top: DomHostedTopLevel,
        task: HtmlCompletionTask,
        custody: &mut Option<DocumentTaskCustody>,
        cancel: &AtomicBool,
    ) -> Result<NativeTaskRunning, DomHostedRuntimeError> {
        self.with_current_top_level(dom, top, cancel, |_, _| Ok::<(), DomHostedRuntimeError>(()))?;
        self.enter_retained_document_task(
            dom,
            (top.global(), top.document()),
            records::CurrentTask::HtmlCompletion {
                task,
                document: top.document(),
            },
            custody,
            cancel,
        )
    }

    pub(crate) fn qualify_local_html_resources(
        &self,
        dom: &mut DomEventDom<'_>,
        top: DomHostedTopLevel,
        checkpoint: DomStamp,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        self.with_current_top_level(dom, top, cancel, |source, dom| {
            if source.local_html_resource_free_changes_since(
                dom.raw(),
                top.document(),
                top.global(),
                checkpoint,
                cancel,
            )? {
                Ok(())
            } else {
                Err(DomHostedError::PendingDocumentObligations.into())
            }
        })
    }
}

impl NativeTaskRunning {
    pub(crate) fn sample_html_load_time(
        &self,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<u64, DomHostedRuntimeError> {
        let owner = self
            .state
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let sample = state
            .sample_admitted_clock(dom, cancel)?
            .map_err(clock_error)?;
        let sample = sample.coarsen().map_err(clock_error)?;
        if sample.source != self.time_origin.source {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(sample.microseconds)
    }
}
