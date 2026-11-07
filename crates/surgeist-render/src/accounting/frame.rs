//! One fresh frame's forward-work authority and three distinct closing domains.
use super::*;
use std::{
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard},
};

/// A completed frame attempt, distinct from last-successful frame statistics.
/// Each domain keeps its actual closing boundary. Failed extraction is typed
/// unavailability; an already closed interval does not include later rollback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFrameReport {
    status: RenderAttemptStatus,
    attempt: std::result::Result<RenderAttemptReport, RenderAccountingError>,
    cpu: std::result::Result<RenderCpuInterval, RenderAccountingError>,
    generation: Option<std::result::Result<RenderGenerationInterval, RenderAccountingError>>,
    diagnostic: Option<RenderAccountingError>,
    cleanup_fault: Option<RenderAccountingError>,
}
impl RenderFrameReport {
    /// Terminal status after inner cleanup and required finalization.
    pub const fn status(&self) -> RenderAttemptStatus {
        self.status
    }
    /// Exact forward-work interval, or its typed extraction fault.
    pub fn attempt(&self) -> std::result::Result<&RenderAttemptReport, RenderAccountingError> {
        self.attempt.as_ref().map_err(|e| *e)
    }
    /// Renderer CPU interval, or its typed extraction fault.
    pub fn renderer_cpu(&self) -> std::result::Result<&RenderCpuInterval, RenderAccountingError> {
        self.cpu.as_ref().map_err(|e| *e)
    }
    /// Selected generation interval. None means selection failed before a device
    /// generation could be associated with this started attempt.
    pub fn generation(
        &self,
    ) -> Option<std::result::Result<&RenderGenerationInterval, RenderAccountingError>> {
        self.generation.as_ref().map(|v| v.as_ref().map_err(|e| *e))
    }
    /// First returned accounting cause, without replacing an earlier backend cause.
    pub const fn accounting_diagnostic(&self) -> Option<RenderAccountingError> {
        self.diagnostic
    }
    /// Mandatory cleanup/finalization fault, separate from the causal outcome.
    pub(crate) fn note_cleanup_fault(&mut self, fault: Option<RenderAccountingError>) {
        self.cleanup_fault = self.cleanup_fault.or(fault);
        if self.cleanup_fault.is_some() && self.status == RenderAttemptStatus::Succeeded {
            self.status = RenderAttemptStatus::Failed(crate::ErrorCode::Accounting);
            if let Ok(attempt) = &mut self.attempt {
                attempt.status = self.status;
            }
        }
    }
    /// First mandatory cleanup or finalization fault, independently of an earlier
    /// causal backend failure or refusal.
    pub const fn cleanup_fault(&self) -> Option<RenderAccountingError> {
        self.cleanup_fault
    }
}

/// Borrowed immutable latest-frame observation. Holding it borrows Renderer and
/// prevents another mutable operation, just like borrowing ordinary renderer data.
pub struct RenderFrameReportRef<'a>(RwLockReadGuard<'a, Option<RenderFrameReport>>);
impl Deref for RenderFrameReportRef<'_> {
    type Target = RenderFrameReport;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("occupied borrowed report")
    }
}
#[derive(Clone, Default)]
pub(crate) struct FrameReportSlot(Arc<RwLock<Option<RenderFrameReport>>>);
impl FrameReportSlot {
    pub(crate) fn read(&self) -> Option<RenderFrameReportRef<'_>> {
        let guard = self
            .0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.is_some() {
            Some(RenderFrameReportRef(guard))
        } else {
            None
        }
    }
    pub(crate) fn take(&self) -> Option<RenderFrameReport> {
        self.0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
    pub(crate) fn store(&self, report: RenderFrameReport) {
        *self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(report)
    }
}

pub(crate) struct FrameAttempt {
    account: Account,
    checkpoint: Option<Checkpoint>,
    cpu: RendererCpu,
    cpu_checkpoint: Option<CpuCheckpoint>,
    generation: Option<Generation>,
    generation_checkpoint: Option<GenerationCheckpoint>,
    generation_setup_fault: Option<RenderAccountingError>,
    slot: FrameReportSlot,
    started: bool,
}
impl FrameAttempt {
    pub(crate) fn begin(
        slot: FrameReportSlot,
        cpu: &RendererCpu,
        options: RenderAccountingOptions,
    ) -> Result<Self> {
        let (account, checkpoint) = build_account(
            options,
            RenderAttemptKind::Frame,
            GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            #[cfg(feature = "render-text")]
            None,
            Some(cpu.at(RenderCpuScope::FrameBacking)),
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
            cpu: cpu.clone(),
            cpu_checkpoint: Some(cpu_checkpoint),
            generation: None,
            generation_checkpoint: None,
            generation_setup_fault: None,
            slot,
            started: true,
        })
    }
    pub(crate) fn started(&self) -> bool {
        self.started
    }
    pub(crate) fn account(&self) -> &Account {
        &self.account
    }
    pub(crate) fn attach_generation(&mut self, generation: &Generation) -> Result<()> {
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
    pub(crate) fn status_for(&self, error: &Error) -> RenderAttemptStatus {
        match error.accounting_diagnostic() {
            Some(RenderAccountingError::Refused(_)) => RenderAttemptStatus::Refused,
            Some(RenderAccountingError::CpuRefused {
                reason: RenderRefusalReason::Limit { .. } | RenderRefusalReason::Overflow,
                ..
            }) if self.cpu.fault().is_none() => RenderAttemptStatus::Refused,
            _ => RenderAttemptStatus::Failed(error.code()),
        }
    }
    pub(crate) fn finish(
        mut self,
        status: RenderAttemptStatus,
        diagnostic: Option<RenderAccountingError>,
    ) -> (RenderFrameReport, Option<RenderAccountingError>) {
        let result = self.extract(status, diagnostic);
        self.started = false;
        result
    }
    fn extract(
        &mut self,
        status: RenderAttemptStatus,
        diagnostic: Option<RenderAccountingError>,
    ) -> (RenderFrameReport, Option<RenderAccountingError>) {
        let mut fault = self.cpu.fault();
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
        if let Err(error) = &interval {
            fault = fault.or(Some(*error));
        }
        let status = if status == RenderAttemptStatus::Succeeded && fault.is_some() {
            RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
        } else {
            status
        };
        let refusal = *self
            .account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let attempt = interval.map(|interval| RenderAttemptReport {
            kind: RenderAttemptKind::Frame,
            status,
            interval,
            refusal,
        });
        (
            RenderFrameReport {
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
impl Drop for FrameAttempt {
    fn drop(&mut self) {
        if self.checkpoint.is_none() {
            return;
        }
        let (report, _) = self.extract(RenderAttemptStatus::Canceled, None);
        if self.started {
            self.slot.store(report);
        }
    }
}

#[cfg(test)]
mod tests;
