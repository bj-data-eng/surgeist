//! Shared checkpoint custody and extraction for renderer-owned attempts.
use super::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct RendererReport {
    status: RenderAttemptStatus,
    attempt: std::result::Result<RenderAttemptReport, RenderAccountingError>,
    cpu: std::result::Result<RenderCpuInterval, RenderAccountingError>,
    generation: Option<std::result::Result<RenderGenerationInterval, RenderAccountingError>>,
    diagnostic: Option<RenderAccountingError>,
    cleanup_fault: Option<RenderAccountingError>,
}
impl RendererReport {
    #[cfg(test)]
    pub(super) fn snapshot_for_test(&self) -> Self {
        Self {
            status: self.status,
            attempt: self
                .attempt
                .as_ref()
                .map(RenderAttemptReport::snapshot_for_test)
                .map_err(|error| *error),
            cpu: self
                .cpu
                .as_ref()
                .map(RenderCpuInterval::snapshot_for_test)
                .map_err(|error| *error),
            generation: self.generation.as_ref().map(|interval| {
                interval
                    .as_ref()
                    .map(RenderGenerationInterval::snapshot_for_test)
                    .map_err(|error| *error)
            }),
            diagnostic: self.diagnostic,
            cleanup_fault: self.cleanup_fault,
        }
    }
    pub(super) fn fmt(
        &self,
        name: &str,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        formatter
            .debug_struct(name)
            .field("status", &self.status)
            .field("attempt", &self.attempt)
            .field("cpu", &self.cpu)
            .field("generation", &self.generation)
            .field("diagnostic", &self.diagnostic)
            .field("cleanup_fault", &self.cleanup_fault)
            .finish()
    }
    /// Terminal status after inner cleanup and required finalization.
    pub(super) const fn status(&self) -> RenderAttemptStatus {
        self.status
    }
    /// Exact forward-work interval, or its typed extraction fault.
    pub(super) fn attempt(
        &self,
    ) -> std::result::Result<&RenderAttemptReport, RenderAccountingError> {
        self.attempt.as_ref().map_err(|e| *e)
    }
    /// Renderer CPU interval, or its typed extraction fault.
    pub(super) fn renderer_cpu(
        &self,
    ) -> std::result::Result<&RenderCpuInterval, RenderAccountingError> {
        self.cpu.as_ref().map_err(|e| *e)
    }
    /// Selected generation interval, when one was associated with this attempt.
    pub(super) fn generation(
        &self,
    ) -> Option<std::result::Result<&RenderGenerationInterval, RenderAccountingError>> {
        self.generation.as_ref().map(|v| v.as_ref().map_err(|e| *e))
    }
    /// First returned accounting cause, without replacing an earlier backend cause.
    pub(super) const fn accounting_diagnostic(&self) -> Option<RenderAccountingError> {
        self.diagnostic
    }
    /// Mandatory cleanup/finalization fault, separate from the causal outcome.
    pub(super) fn note_cleanup_fault(&mut self, fault: Option<RenderAccountingError>) {
        self.cleanup_fault = self.cleanup_fault.or(fault);
        if self.cleanup_fault.is_some() && self.status == RenderAttemptStatus::Succeeded {
            self.status = RenderAttemptStatus::Failed(crate::ErrorCode::Accounting);
            if let Ok(attempt) = &mut self.attempt {
                attempt.status = self.status;
            }
        }
    }
    /// Record a failed final output transfer without changing any closed interval.
    pub(super) fn fail_after_close(
        &mut self,
        error: &Error,
        cleanup_fault: Option<RenderAccountingError>,
    ) {
        self.status = RenderAttemptStatus::Failed(crate::ErrorCode::Accounting);
        if let Ok(attempt) = &mut self.attempt {
            attempt.status = self.status;
        }
        self.diagnostic = self.diagnostic.or(error.accounting_diagnostic());
        self.cleanup_fault = self.cleanup_fault.or(cleanup_fault);
    }
    /// First mandatory cleanup or finalization fault, independently of an earlier
    /// causal backend failure or refusal.
    pub(super) const fn cleanup_fault(&self) -> Option<RenderAccountingError> {
        self.cleanup_fault
    }
}

/// A bounded, preallocated report destination. Replacement and take never allocate.
pub(super) struct ReportSlot<T>(Arc<RwLock<Option<T>>>);
impl<T> Clone for ReportSlot<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Default for ReportSlot<T> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }
}
impl<T> ReportSlot<T> {
    pub(super) fn read(&self) -> Option<RwLockReadGuard<'_, Option<T>>> {
        let guard = self
            .0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.is_some().then_some(guard)
    }
    pub(super) fn take(&self) -> Option<T> {
        self.0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
    pub(super) fn store(&self, report: T) {
        *self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(report);
    }
}

pub(super) struct RendererAttempt {
    account: Account,
    checkpoint: Option<Checkpoint>,
    report_backing: Option<ReportBacking<RenderPhase, IntervalNumbers>>,
    cpu: RendererCpu,
    pub(super) cpu_checkpoint: Option<CpuCheckpoint>,
    generation: Option<Generation>,
    generation_checkpoint: Option<GenerationCheckpoint>,
    generation_setup_fault: Option<RenderAccountingError>,
    kind: RenderAttemptKind,
    started: bool,
}
impl RendererAttempt {
    pub(super) fn begin(
        kind: RenderAttemptKind,
        cpu: &RendererCpu,
        options: RenderAccountingOptions,
    ) -> Result<Self> {
        let scope = match kind {
            RenderAttemptKind::Frame => RenderCpuScope::FrameBacking,
            RenderAttemptKind::Readback => RenderCpuScope::ReadbackOutput,
            _ => return Err(RenderAccountingError::Invariant.into()),
        };
        let (account, checkpoint, report_backing) = build_account(
            options,
            kind,
            GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            #[cfg(feature = "render-text")]
            None,
            Some(cpu.at(scope)),
        )?;
        let cpu_checkpoint = match cpu.begin_interval() {
            Ok(value) => value,
            Err(error) => {
                let _ = account.ledger.end_checkpoint(&checkpoint);
                return Err(error);
            }
        };
        Ok(Self {
            account,
            checkpoint: Some(checkpoint),
            report_backing: Some(report_backing),
            cpu: cpu.clone(),
            cpu_checkpoint: Some(cpu_checkpoint),
            generation: None,
            generation_checkpoint: None,
            generation_setup_fault: None,
            kind,
            started: true,
        })
    }
    pub(super) fn started(&self) -> bool {
        self.started
    }
    pub(super) fn account(&self) -> &Account {
        &self.account
    }
    pub(super) fn attach_generation(&mut self, generation: &Generation) -> Result<()> {
        if self.generation.is_some() {
            return Err(RenderAccountingError::Invariant.into());
        }
        match generation.begin_interval() {
            Ok(checkpoint) => {
                self.generation = Some(generation.clone());
                self.generation_checkpoint = Some(checkpoint);
                Ok(())
            }
            Err(error) => {
                let diagnostic = error
                    .accounting_diagnostic()
                    .unwrap_or(RenderAccountingError::Invariant);
                if matches!(diagnostic, RenderAccountingError::Setup(_)) {
                    self.started = false;
                }
                self.generation_setup_fault = Some(diagnostic);
                Err(error)
            }
        }
    }
    pub(super) fn status_for(&self, error: &Error) -> RenderAttemptStatus {
        match error.accounting_diagnostic() {
            Some(RenderAccountingError::Refused(_)) => RenderAttemptStatus::Refused,
            Some(RenderAccountingError::CpuRefused {
                reason: RenderRefusalReason::Limit { .. } | RenderRefusalReason::Overflow,
                ..
            }) if self.cpu.fault().is_none() => RenderAttemptStatus::Refused,
            _ => RenderAttemptStatus::Failed(error.code()),
        }
    }
    pub(super) fn finish(
        &mut self,
        status: RenderAttemptStatus,
        diagnostic: Option<RenderAccountingError>,
    ) -> (RendererReport, Option<RenderAccountingError>) {
        let result = self.extract(status, diagnostic);
        self.started = false;
        result
    }
    fn extract(
        &mut self,
        status: RenderAttemptStatus,
        diagnostic: Option<RenderAccountingError>,
    ) -> (RendererReport, Option<RenderAccountingError>) {
        let mut fault = self.cpu.fault().or(self.account.latched_fault());
        if let Some(generation) = &self.generation
            && let Err(error) = generation.ensure_healthy()
        {
            fault = fault.or(error.accounting_diagnostic());
        }
        let cpu = self
            .cpu_checkpoint
            .take()
            .ok_or(RenderAccountingError::Invariant)
            .and_then(|checkpoint| {
                checkpoint.finish().map_err(|e| {
                    e.accounting_diagnostic()
                        .unwrap_or(RenderAccountingError::Invariant)
                })
            });
        if let Err(error) = &cpu {
            fault = fault.or(Some(*error));
        }
        let generation = if let Some(checkpoint) = self.generation_checkpoint.take() {
            let interval = checkpoint.finish().map_err(|error| {
                error
                    .accounting_diagnostic()
                    .unwrap_or(RenderAccountingError::Invariant)
            });
            if let Err(error) = &interval {
                fault = fault.or(Some(*error));
            }
            Some(interval)
        } else {
            self.generation_setup_fault.map(Err)
        };
        let interval = self
            .checkpoint
            .take()
            .ok_or(RenderAccountingError::Invariant)
            .and_then(|checkpoint| {
                self.account
                    .ledger
                    .end_checkpoint(&checkpoint)
                    .map_err(map_ledger_failure)
            });
        let refusal = *self
            .account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut attempt = interval.and_then(|interval| {
            let backing = self
                .report_backing
                .take()
                .ok_or(RenderAccountingError::Invariant)?;
            RenderAttemptReport::new(
                self.kind,
                status,
                interval,
                refusal,
                self.account.scopes.as_ref(),
                backing,
            )
        });
        if let Err(error) = &attempt {
            fault = fault.or(Some(*error));
        }
        let status = if status == RenderAttemptStatus::Succeeded && fault.is_some() {
            RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
        } else {
            status
        };
        if let Ok(attempt) = &mut attempt {
            attempt.status = status;
        }
        (
            RendererReport {
                status,
                attempt,
                cpu,
                generation,
                diagnostic: diagnostic.or(fault),
                cleanup_fault: fault,
            },
            fault,
        )
    }
}
impl RendererAttempt {
    /// Close owned checkpoints even when setup failed before a started attempt.
    pub(super) fn cancel(&mut self) -> Option<RendererReport> {
        self.checkpoint.as_ref()?;
        let started = self.started;
        let (report, _) = self.finish(RenderAttemptStatus::Canceled, None);
        started.then_some(report)
    }
}
