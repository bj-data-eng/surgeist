use super::*;
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
                        for policy in policies {
                            step()?;
                            let Some(matches) =
                                policy.inline_style(input.nonce_content(), &mut || step())?
                            else {
                                return Ok(None);
                            };
                            if !matches
                                && policy.delivery()
                                    == crate::trusted_types_csp::PolicyDelivery::ReportOnlyHeader
                            {
                                return Ok(None);
                            }
                            allowed &= matches;
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
                    lease.accept_inline_csp(request, &observation, cancel)?;
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
