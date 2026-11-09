//! Temporary error reporting for one unscoped source-module execution run.

use super::Context;
use crate::{JsError, JsResult, job::UnwindCleanup};

pub(super) struct ModuleRunFailure {
    first: Option<JsError>,
}

struct ModuleRun<'a> {
    context: &'a mut Context,
}

impl ModuleRun<'_> {
    fn finish<T>(self, result: JsResult<T>) -> JsResult<T> {
        // Remove the entire owner before disposing an overridden result or any
        // error roots. Reentrant work must not inherit this completed run.
        let failure = self.context.module_run_failure.take();
        let first = failure.and_then(|failure| failure.first);
        drop(self);
        match first {
            Some(error) => {
                let mut cleanup = UnwindCleanup::default();
                cleanup.run(move || drop(result));
                Err(cleanup.finish_with(error))
            }
            None => result,
        }
    }
}

impl Drop for ModuleRun<'_> {
    fn drop(&mut self) {
        let failure = self.context.module_run_failure.take();
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(move || drop(failure));
        cleanup.finish();
    }
}

impl Context {
    /// Selects a temporary report owner only when no execution/run already owns it.
    pub(crate) fn with_module_run<T>(
        &mut self,
        run: impl FnOnce(&mut Context) -> JsResult<T>,
    ) -> JsResult<T> {
        if self.execution.is_some() {
            let result = run(self);
            if let Some(stop) = self
                .execution
                .as_ref()
                .and_then(|execution| execution.stop())
                .cloned()
            {
                let mut cleanup = UnwindCleanup::default();
                cleanup.run(move || drop(result));
                return Err(cleanup.finish_with(stop));
            }
            return result;
        }
        if self.module_run_failure.is_some() {
            return run(self);
        }
        self.module_run_failure = Some(ModuleRunFailure { first: None });
        let guard = ModuleRun { context: self };
        let result = run(&mut *guard.context);
        guard.finish(result)
    }

    pub(crate) fn module_run_stop(&self) -> Option<&JsError> {
        self.module_run_failure
            .as_ref()
            .and_then(|failure| failure.first.as_ref())
    }

    /// Retains an actual admitted committed-reaction cause at its selected owner.
    /// False means there is no owner: the caller must still return the error.
    pub(crate) fn retain_committed_reaction_failure(&mut self, error: &mut JsError) -> bool {
        self.retain_module_failure(error)
    }

    /// Retains a failed canonical settlement or required module-parent action.
    /// Remaining owned bookkeeping can close, while new guest actions refuse it.
    pub(crate) fn retain_module_failure(&mut self, error: &mut JsError) -> bool {
        if self.execution.is_none() && self.module_run_failure.is_none() {
            return false;
        }
        if let Err(first) = self.check_execution_stop() {
            let superseded = std::mem::replace(error, first);
            let mut cleanup = UnwindCleanup::default();
            cleanup.run(move || drop(superseded));
            cleanup.finish();
        } else {
            self.capture_error_backtrace(error);
        }
        if let Some(execution) = &mut self.execution {
            execution.retain_stop(error);
        } else if let Some(failure) = &mut self.module_run_failure
            && failure.first.is_none()
        {
            failure.first = Some(error.clone());
        }
        true
    }
}

#[cfg(test)]
mod tests;
