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

impl DomSession {
    pub(super) fn finish_guest_style(
        &mut self,
        result: &mut Result<DomSourceDriveProgress, DomSourceDriveError>,
        owner: &NativeOwner,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) {
        let Some(processor) = self.trusted_types_csp.clone() else {
            return;
        };
        while let Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Style(request),
            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
        }) = result
        {
            let operation = request.source().operation();
            let mut source = self.source_execution();
            let mut dom = self.dom();
            let accepted = (|| {
                let mut lease = dom.resume_source_edit(operation, limits, cancel)?;
                let mut queue = owner
                    .trusted_types_reports
                    .try_borrow_mut()
                    .map_err(|_| DomStyleRuntimeError::Binding)?;
                let mut staged_backing = None;
                let mut reports = Vec::new();
                let mut origin = None;
                let observation = {
                    let processor = processor
                        .processor
                        .try_borrow()
                        .map_err(|_| DomStyleRuntimeError::Binding)?;
                    lease.evaluate_inline_csp(request, cancel, |input, step, admit| {
                        let policies = processor
                            .document_policies_with(
                                input.document_metadata().policy_container().csp(),
                                &mut |layout| {
                                    admit(layout.atomic_bytes())?;
                                    layout
                                        .charge_work(&mut CspWork(|units| {
                                            for _ in 0..units {
                                                step()?;
                                            }
                                            Ok::<(), DocumentScriptNativeError>(())
                                        }))
                                        .map_err(|error| match error {
                                            HtmlChargedError::Stopped(
                                                DocumentScriptNativeError::Dom(error),
                                            ) => DomStyleRuntimeError::Dom(error),
                                            _ => DomStyleRuntimeError::Admission,
                                        })
                                },
                            )
                            .map_err(|error| match error {
                                HtmlCspError::Interrupted(error) => error,
                                HtmlCspError::Protocol(_) => DomStyleRuntimeError::Protocol,
                            })?;
                        let Some(policies) = policies else {
                            return Ok(None);
                        };
                        let revision = input.stamp().revision().get();
                        if policies.is_empty() {
                            return Ok(Some(DomStyleInlineCspObservation::complete_empty(
                                input, revision,
                            )));
                        }
                        let mut allowed = true;
                        let mut hashes = crate::trusted_types_csp::StyleHashes::new(input.text());
                        for (index, policy) in policies.iter().enumerate() {
                            step()?;
                            let Some((matches, report_sample)) = policy.inline_style(
                                input.nonce_content(),
                                &mut hashes,
                                &mut || step(),
                            )?
                            else {
                                return Ok(None);
                            };
                            if matches {
                                continue;
                            }
                            if reports.is_empty() {
                                let snapshot = lease.snapshot();
                                let document =
                                    snapshot.node_with(input.document().node(), &mut || step())?;
                                let Some(global) = document.relevant_global()?.filter(|global| {
                                    owner.attachment.get().is_some_and(|attached| {
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
                                staged_backing = Some(
                                    lease.retain_inline_csp_payload(request, bytes, 0, cancel)?,
                                );
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
                                prepare_style_violation(
                                    index,
                                    policy,
                                    sample,
                                    &mut || step(),
                                    |bytes| {
                                        let retained = ViolationQueue::record_bytes(bytes)
                                            .map_err(|_| DomError::RepresentationExhausted)?;
                                        Ok(ViolationBacking::Style {
                                            _backing: lease.retain_inline_csp_payload(
                                                request,
                                                retained,
                                                bytes / size_of::<u16>(),
                                                cancel,
                                            )?,
                                        })
                                    },
                                )
                                .map_err(|error| match error {
                                    ReportError::Stopped(error) => DomStyleRuntimeError::Dom(error),
                                    ReportError::RepresentationExhausted => {
                                        DomError::RepresentationExhausted.into()
                                    }
                                    ReportError::Allocation => DomStyleRuntimeError::Admission,
                                })?,
                            );
                            allowed &= !policy.delivery().enforces();
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
                        admit(bytes)?;
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
                        Ok::<_, DomStyleRuntimeError>(Some(
                            DomStyleInlineCspObservation::evaluated(
                                input,
                                Arc::new(originals),
                                revision,
                                allowed,
                            ),
                        ))
                    })?
                };
                if let Some(observation) = observation {
                    let was_empty = queue.is_none();
                    let admit = if let Some(origin) = origin {
                        ViolationQueue::admit_batch(
                            &mut queue,
                            origin,
                            reports.len(),
                            &mut |bytes, units| {
                                Ok(ViolationBacking::Style {
                                    _backing: lease
                                        .retain_inline_csp_payload(request, bytes, units, cancel)?,
                                })
                            },
                            |units| {
                                lease.evaluate_inline_csp(request, cancel, |_, step, _| {
                                    for _ in 0..units {
                                        step()?;
                                    }
                                    Ok::<_, DomError>(())
                                })?;
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
                        })
                    } else {
                        Ok(())
                    };
                    if let Err(error) = admit.and_then(|()| {
                        lease
                            .accept_inline_csp(request, &observation, cancel)
                            .map_err(Into::into)
                    }) {
                        if was_empty {
                            queue.take();
                        }
                        return Err(error);
                    }
                    if let Some(queue) = queue.as_mut() {
                        queue.push_style(request.source().element(), reports);
                    }
                    drop(staged_backing);
                    Ok::<_, DomStyleRuntimeError>(true)
                } else {
                    Ok(false)
                }
            })();
            match accepted {
                Ok(false) => return,
                Err(error) => {
                    *result = Err(error.into());
                    return;
                }
                Ok(true) => {}
            };
            *result = source.resume(
                &mut dom,
                operation,
                limits,
                &mut guest_source_services(&mut initial_inline_style_policy),
                cancel,
            );
        }
    }
}
