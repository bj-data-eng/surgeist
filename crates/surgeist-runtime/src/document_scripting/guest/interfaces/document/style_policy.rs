use super::*;
use crate::document_scripting::guest::reports::{
    ViolationBacking, ViolationOrigin, ViolationQueue,
};
use crate::trusted_types_csp::{ReportError, ViolationReport, prepare_style_violation};
use crate::{
    DomSourceDriveError, DomSourceDriveProgress, DomSourceUnavailable, DomStyleRuntimeError,
    DomStyleUnavailable,
};
use surgeist_dom::{DomSourceLimits, DomSourceRequest, DomStyleInlineCspObservation};
use surgeist_html::{accounting::HtmlChargedError, security::csp::HtmlCspError};

#[derive(Clone, Copy)]
enum StyleReportDelivery {
    Attached,
    Stopped,
}

struct PreparedStylePolicy {
    observation: DomStyleInlineCspObservation,
    reports: Vec<ViolationReport<ViolationBacking>>,
    origin: Option<ViolationOrigin>,
    _report_bank: Option<surgeist_dom::DomStyleCspRetention>,
    observation_backing: Option<surgeist_dom::DomStyleCspRetention>,
}

impl NativeOwner {
    pub(super) fn finish_guest_tree_policies(
        &self,
        result: &mut Result<DomSourceDriveProgress, DomSourceDriveError>,
        limits: DomSourceLimits,
    ) -> Result<(), DocumentScriptNativeError> {
        self.finish_tree_policies(result, limits, StyleReportDelivery::Attached)
    }

    fn finish_tree_policies(
        &self,
        result: &mut Result<DomSourceDriveProgress, DomSourceDriveError>,
        limits: DomSourceLimits,
        delivery: StyleReportDelivery,
    ) -> Result<(), DocumentScriptNativeError> {
        let cancel = &self.cancellation;
        loop {
            self.finish_style(result, limits, delivery);
            let Ok(DomSourceDriveProgress::Pending {
                request: DomSourceRequest::MetaCsp(request),
                ..
            }) = *result
            else {
                self.reconcile_pending_style();
                return Ok(());
            };
            let mut session = self
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            if let Err(error) = session.service_trusted_types_meta_csp(request, limits, cancel) {
                *result = Err(error.into());
                return Ok(());
            }
            *result = session.source_execution().resume(
                &mut session.dom(),
                request.operation(),
                limits,
                &mut guest_source_services(&mut |_| None),
                cancel,
            );
        }
    }

    pub(super) fn finish_guest_style(
        &self,
        result: &mut Result<DomSourceDriveProgress, DomSourceDriveError>,
        limits: DomSourceLimits,
    ) {
        self.finish_style(result, limits, StyleReportDelivery::Attached);
    }

    fn finish_style(
        &self,
        result: &mut Result<DomSourceDriveProgress, DomSourceDriveError>,
        limits: DomSourceLimits,
        delivery: StyleReportDelivery,
    ) {
        self.record_reached_style();
        let cancel = &self.cancellation;
        while let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Style(request),
            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
        }) = result
        {
            let operation = request.source().operation();
            let accepted = (|| {
                let Some(mut prepared) = self.evaluate_guest_style(request, delivery)? else {
                    return Ok(false);
                };
                let mut session = self
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DomStyleRuntimeError::Binding)?;
                let mut dom = session.dom();
                let mut lease = dom.resume_source_edit(operation, limits, cancel)?;
                let Some(origin) = prepared.origin else {
                    drop(prepared.observation_backing.take());
                    lease.accept_inline_csp(request, &prepared.observation, cancel)?;
                    return Ok(true);
                };
                let mut queue = self
                    .trusted_types_reports
                    .try_borrow_mut()
                    .map_err(|_| DomStyleRuntimeError::Binding)?;
                let budget = lease.inline_csp_budget(request)?;
                let prepared_queue = ViolationQueue::prepare_batch(
                    &mut queue,
                    origin,
                    prepared.reports.len(),
                    &mut |bytes, units| {
                        Ok(ViolationBacking::Style {
                            _backing: budget.retain(bytes, units, cancel)?,
                        })
                    },
                    |units| {
                        for _ in 0..units {
                            budget.step(cancel)?;
                        }
                        Ok(())
                    },
                )
                .map_err(|error| match error {
                    DocumentGuestEventError::Document(DocumentScriptError::Native(
                        DocumentScriptNativeError::Dom(error),
                    )) => DomStyleRuntimeError::Dom(error),
                    DocumentGuestEventError::Document(DocumentScriptError::Native(
                        DocumentScriptNativeError::ForeignPermit,
                    )) => DomStyleRuntimeError::Protocol,
                    _ => DomStyleRuntimeError::Admission,
                })?;
                // The canonical receipt admits this same bank. No other
                // allocation occurs between the two ownership checks.
                drop(prepared.observation_backing.take());
                lease.accept_inline_csp(request, &prepared.observation, cancel)?;
                prepared_queue
                    .install()
                    .push_style(request.source().element(), prepared.reports);
                Ok::<_, DomStyleRuntimeError>(true)
            })();
            match accepted {
                Ok(false) => return,
                Err(error) => {
                    *result = Err(error.into());
                    return;
                }
                Ok(true) => {}
            };
            *result = match self.drive_pending_style(operation, limits) {
                Ok(progress) => progress,
                Err(error) => Err(error.into()),
            };
            self.reconcile_pending_style();
        }
    }

    fn record_reached_style(&self) {
        if let Ok(session) = self.session.try_borrow()
            && let Some(operation) = session.read_dom().pending_style_source_edit()
        {
            self.pending_style.set(Some(operation));
        }
    }

    fn evaluate_guest_style(
        &self,
        request: &surgeist_dom::DomStyleRequest,
        delivery: StyleReportDelivery,
    ) -> Result<Option<PreparedStylePolicy>, DomStyleRuntimeError> {
        let cancel = &self.cancellation;
        let session = self
            .session
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let budget = session.read_dom().inline_csp_budget(request)?;
        let input = request.source();
        let csp = input.document_metadata().policy_container().csp();
        let step = || budget.step(cancel);
        step()?;
        if matches!(
            csp,
            surgeist_dom::DomDocumentCspState::ConstructorInitialEmpty
        ) {
            return Ok(Some(PreparedStylePolicy {
                observation: DomStyleInlineCspObservation::complete_empty(input, 0),
                reports: Vec::new(),
                origin: None,
                _report_bank: None,
                observation_backing: None,
            }));
        }
        let Some(processor) = &session.trusted_types_csp else {
            return Ok(None);
        };
        let processor = processor
            .processor
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let snapshot = session.snapshot();
        let policies = processor
            .document_policies_with(csp, &mut |layout| {
                budget.admit_bytes(layout.atomic_bytes(), cancel)?;
                layout
                    .charge_work(&mut CspWork(|units| {
                        for _ in 0..units {
                            step()?;
                        }
                        Ok::<(), DocumentScriptNativeError>(())
                    }))
                    .map_err(|error| match error {
                        HtmlChargedError::Stopped(DocumentScriptNativeError::Dom(error)) => {
                            DomStyleRuntimeError::Dom(error)
                        }
                        _ => DomStyleRuntimeError::Admission,
                    })
            })
            .map_err(|error| match error {
                HtmlCspError::Interrupted(error) => error,
                HtmlCspError::Protocol(_) => DomStyleRuntimeError::Protocol,
            })?;
        let Some(policies) = policies else {
            return Ok(None);
        };
        let revision = input.stamp().revision().get();
        if policies.is_empty() {
            return Ok(Some(PreparedStylePolicy {
                observation: DomStyleInlineCspObservation::complete_empty(input, revision),
                reports: Vec::new(),
                origin: None,
                _report_bank: None,
                observation_backing: None,
            }));
        }
        let mut staged_backing = None;
        let mut reports = Vec::new();
        let mut origin = None;
        let mut allowed = true;
        let mut hashes = crate::trusted_types_csp::StyleHashes::new(input.text());
        for (index, policy) in policies.iter().enumerate() {
            step()?;
            let Some((matches, report_sample)) =
                policy.inline_style(input.nonce_content(), &mut hashes, &mut || step())?
            else {
                return Ok(None);
            };
            if matches {
                continue;
            }
            allowed &= !policy.delivery().enforces();
            if matches!(delivery, StyleReportDelivery::Stopped) {
                continue;
            }
            if reports.is_empty() {
                let document = snapshot.node_with(input.document().node(), &mut || step())?;
                let Some(global) = document.relevant_global()?.filter(|global| {
                    self.attachment.get().is_some_and(|attached| {
                        attached.target.global() == *global
                            && attached.target.document() == input.document()
                    })
                }) else {
                    return Ok(None);
                };
                origin = Some(ViolationOrigin {
                    global,
                    document: input.document(),
                });
                let bytes = policies
                    .len()
                    .checked_mul(size_of::<ViolationReport<ViolationBacking>>())
                    .ok_or(DomError::RepresentationExhausted)?;
                staged_backing = Some(budget.retain(bytes, 0, cancel)?);
                reports
                    .try_reserve_exact(policies.len())
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
                if reports.capacity() != policies.len() {
                    return Err(DomStyleRuntimeError::Admission);
                }
            }
            let sample = if report_sample {
                input.text().as_slice()
            } else {
                &[]
            };
            reports.push(
                prepare_style_violation(index, policy, sample, &mut || step(), |bytes| {
                    let retained = ViolationQueue::record_bytes(bytes)
                        .map_err(|_| DomError::RepresentationExhausted)?;
                    Ok(ViolationBacking::Style {
                        _backing: budget.retain(retained, bytes / size_of::<u16>(), cancel)?,
                    })
                })
                .map_err(|error| match error {
                    ReportError::Stopped(error) => DomStyleRuntimeError::Dom(error),
                    ReportError::RepresentationExhausted => {
                        DomError::RepresentationExhausted.into()
                    }
                    ReportError::Allocation => DomStyleRuntimeError::Admission,
                })?,
            );
        }
        let bytes = policies.iter().try_fold(
            size_of::<Vec<DocumentString>>() + 2 * size_of::<usize>(),
            |bytes, policy| {
                step()?;
                policy
                    .serialized()
                    .retained_units()
                    .checked_mul(size_of::<u16>())
                    .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                    .and_then(|n| n.checked_add(size_of::<DocumentString>()))
                    .and_then(|n| n.checked_add(bytes))
                    .ok_or(DomError::RepresentationExhausted)
            },
        )?;
        let observation_backing = budget.retain(bytes, 0, cancel)?;
        let mut originals = Vec::new();
        originals
            .try_reserve_exact(policies.len())
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        if originals.capacity() != policies.len() {
            return Err(DomStyleRuntimeError::Admission);
        }
        for policy in policies {
            step()?;
            originals.push(policy.serialized().clone());
        }
        Ok(Some(PreparedStylePolicy {
            observation: DomStyleInlineCspObservation::evaluated(
                input,
                Arc::new(originals),
                revision,
                allowed,
            ),
            reports,
            origin,
            _report_bank: staged_backing,
            observation_backing: Some(observation_backing),
        }))
    }

    fn drive_pending_style(
        &self,
        operation: surgeist_dom::DomSourceOperationId,
        limits: DomSourceLimits,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DomStyleRuntimeError> {
        let mut session = self
            .session
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        Ok(session.source_execution().resume(
            &mut session.dom(),
            operation,
            limits,
            &mut guest_source_services(&mut |_| None),
            &self.cancellation,
        ))
    }

    fn reconcile_pending_style(&self) {
        if let Some(operation) = self.pending_style.get()
            && let Ok(session) = self.session.try_borrow()
            && session
                .read_dom()
                .pending_source_edit()
                .is_none_or(|status| status.operation() != operation)
        {
            self.pending_style.set(None);
        }
    }
}

impl DocumentScriptHost {
    /// Resume a parked Style edit with the same Session processor and report
    /// queue. A refusal leaves the original Source operation available.
    pub fn resume_pending_style_edit(
        &mut self,
        permit: &DocumentScriptPermit,
        operation: surgeist_dom::DomSourceOperationId,
        limits: DomSourceLimits,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentScriptNativeError>
    {
        self.check_permit(permit)?;
        self.native
            .resume_style(operation, limits, StyleReportDelivery::Attached)
    }
}

impl StoppedDocumentScriptHost {
    /// Settle the original native Style edit after disposing of the engine.
    /// The selected processor still decides the policy and the original
    /// synchronous effects complete. Retired reports have no delivery owner.
    pub fn settle_pending_style_edit(
        &mut self,
        operation: surgeist_dom::DomSourceOperationId,
        limits: DomSourceLimits,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentScriptNativeError>
    {
        self.native
            .resume_style(operation, limits, StyleReportDelivery::Stopped)
    }
}

impl NativeOwner {
    fn resume_style(
        &self,
        operation: surgeist_dom::DomSourceOperationId,
        limits: DomSourceLimits,
        delivery: StyleReportDelivery,
    ) -> Result<Result<DomSourceDriveProgress, DomSourceDriveError>, DocumentScriptNativeError>
    {
        if self.pending_style.get() != Some(operation) {
            return Err(DocumentScriptNativeError::SourceAccess);
        }
        let mut result = self
            .drive_pending_style(operation, limits)
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        self.finish_tree_policies(&mut result, limits, delivery)?;
        self.reconcile_pending_style();
        self.reconcile_pending_attribute()?;
        Ok(result)
    }
}
