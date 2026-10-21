//! Original notification authority and its canonical synchronous dispatch gate.
use super::*;
mod cleanup;
use crate::parsed_style::DomParsedHtmlStyleNotificationError as Error;
use crate::{
    DomStyleCssomRoot, DomStyleNotificationOrigin, DomStyleNotificationToken, DomStyleSheet,
};
pub use cleanup::DomParsedHtmlStyleNotificationCompletion;
use std::sync::{Mutex, MutexGuard};
use surgeist_cssom::CssomStore;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store::pending_associations) enum Issuance {
    Unissued,
    Issued,
    OwnerRemoved,
}
/// Actual original receipt: historical identities and fixed frame disappear
/// before its Meter. Mutex preserves token Send/Sync without an async loan.
pub(crate) struct NotificationCustody {
    qualification: DomParsedHtmlStyleInitialPolicyRequest,
    meter: Mutex<Meter>,
}
pub struct DomParsedHtmlStyleNotificationPreparation<'hold, 'dom, 'caps> {
    graph: MutexGuard<'hold, Stored>,
    completed: MutexGuard<'hold, Option<CompletedInvocation>>,
    historical: bool,
    _dom: &'hold mut &'dom mut Dom,
    runtime: &'hold Meter,
    meter: Meter,
    store: MutexGuard<'caps, CssomStore>,
    disposition: Option<DomParsedHtmlStyleNotificationDisposition>,
}
/// Retains the genuine root lock while the composing owner installs its
/// already-admitted queue row. Taking once neither unlocks nor allocates.
pub struct DomParsedHtmlStyleNotificationPublished<'caps> {
    _store: MutexGuard<'caps, CssomStore>,
    disposition: Option<DomParsedHtmlStyleNotificationDisposition>,
}
impl DomParsedHtmlStyleNotificationPublished<'_> {
    pub fn take_disposition(&mut self) -> Option<DomParsedHtmlStyleNotificationDisposition> {
        self.disposition.take()
    }
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    pub fn prepare_notification<'hold, 'caps>(
        &'hold mut self,
        qualification: &DomParsedHtmlStyleInitialPolicyRequest,
        expected: &DomStyleSheet,
        expected_owner_label: &str,
        cap: &'caps DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleNotificationPreparation<'hold, 'dom, 'caps>, Error> {
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let value = graph
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?
            .value();
        let completed = self
            .meter
            .entry()
            .completed_style
            .try_lock()
            .map_err(|e| guard_error(e, self.meter.entry().id))?;
        let historical = completed
            .as_ref()
            .is_some_and(|old| &old.qualification == qualification);
        let (notification, profile, sheet) = if historical {
            let old = completed.as_ref().expect("qualified predecessor");
            let CompletedOutcome::Created {
                sheet,
                notification,
            } = &old.outcome
            else {
                return Err(DomParsedHtmlStyleError::WrongPhase {
                    phase: DomParsedHtmlStylePhase::InlinePolicyBlocked,
                }
                .into());
            };
            (*notification, old.profile, sheet)
        } else {
            if value.qualification() != qualification {
                return Err(DomParsedHtmlStyleError::WrongRequest.into());
            }
            let Graph::Rendered {
                request,
                notification,
            } = value.as_ref()
            else {
                return Err(DomParsedHtmlStyleError::WrongPhase {
                    phase: value.phase(),
                }
                .into());
            };
            (
                *notification,
                request.cursor.profile(),
                request
                    .created_sheet
                    .as_ref()
                    .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?,
            )
        };
        if notification != Issuance::Unissued {
            return Err(Error::AlreadyIssued {
                contribution: qualification.contribution(),
            });
        }
        if profile != HtmlStyleExecutionProfile::NoExecutableJavaScript {
            return Err(DomParsedHtmlStyleError::WrongExecutionProfile { profile }.into());
        }
        let root = qualification.document().node();
        self.meter.step(cancel)?;
        if cap.root() != root
            || expected.root() != root
            || self
                .dom
                .style_cssom_roots
                .get(root)
                .is_none_or(|stored| !stored.same_owner(cap))
        {
            return Err(Error::MismatchedOwner { root });
        }
        if self
            .dom
            .pending_associations
            .document(qualification.document())
            != Some(qualification.contribution())
        {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        let mut mapped = None;
        for binding in self.meter.entry().bindings.iter() {
            self.meter.step(cancel)?;
            if &binding.source == qualification.subject() {
                mapped = Some(binding.node);
                break;
            }
        }
        let target = mapped.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if expected.association().element() != target {
            return Err(Error::MismatchedAssociation { target });
        }
        let snapshot = self.dom.snapshot();
        let node = match snapshot.node(target) {
            Ok(value) => value,
            Err(DomError::StaleNode) => return Err(Error::TargetDestroyed { target }),
            Err(error) => return Err(error.into()),
        };
        let current = node
            .as_style()
            .ok_or(Error::MismatchedAssociation { target })?;
        let canonical_matches = current.state().sheet.as_ref().is_some_and(|sheet| {
            sheet.id() == expected.id()
                && sheet.association() == expected.association()
                && sheet.root() == expected.root()
        }) && node.node.document() == qualification.document();
        let store = cap.lock().map_err(|error| match error {
            crate::DomStyleCssomRootError::Busy => Error::OwnerBusy { root },
            crate::DomStyleCssomRootError::Poisoned => Error::OwnerPoisoned { root },
            _ => Error::MismatchedOwner { root },
        })?;
        let meter = self.meter.fork();
        meter.retain(
            size_of::<DomParsedHtmlStyleNotificationPreparation<'_, '_, '_>>()
                + size_of::<DomParsedHtmlStyleNotificationPublished<'_>>()
                + size_of::<surgeist_cssom::CssomSnapshot>(),
            0,
            0,
            cancel,
        )?;
        let producer = store.snapshot();
        let actual = producer.sheet(expected.id()).map_err(Error::Cssom)?;
        // Preserve the authentic producer refusal for unresolved identities.
        // A resolved object must still be the creation-sealed invocation sheet,
        // including the owner-cleared path after a successor has started.
        if sheet != expected {
            return Err(Error::MismatchedAssociation {
                target: expected.association().element(),
            });
        }
        meter.step(cancel)?;
        let removed = if let Some(owner) = actual.inputs().owner.as_deref() {
            if !canonical_matches {
                return Err(Error::MismatchedAssociation { target });
            }
            if owner.len() != expected_owner_label.len() {
                return Err(Error::MismatchedLabel { root });
            }
            for (actual, expected) in owner.bytes().zip(expected_owner_label.bytes()) {
                meter.step(cancel)?;
                if actual != expected {
                    return Err(Error::MismatchedLabel { root });
                }
            }
            false
        } else {
            true
        };
        // Both snapshots can disappear here: their actual canonical/producer
        // graphs remain owned by the still-exclusive DOM and real Store guard.
        drop(producer);
        drop(snapshot);
        let disposition = if removed {
            DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved {
                association: expected.association(),
            }
        } else {
            let custody = self.meter.fork();
            custody.retain(size_of::<NotificationCustody>(), 0, 0, cancel)?;
            DomParsedHtmlStyleNotificationDisposition::Token(DomStyleNotificationToken {
                origin: Some(DomStyleNotificationOrigin::ParsedOccurrence {
                    contribution: qualification.contribution(),
                    occurrence: qualification.occurrence().clone(),
                }),
                association: expected.association(),
                complete: false,
                parsed: Some(Box::new(NotificationCustody {
                    qualification: qualification.clone(),
                    meter: Mutex::new(custody),
                })),
            })
        };
        Ok(DomParsedHtmlStyleNotificationPreparation {
            graph,
            completed,
            historical,
            _dom: &mut self.dom,
            runtime: &self.meter,
            meter,
            store,
            disposition: Some(disposition),
        })
    }
}
impl<'caps> DomParsedHtmlStyleNotificationPreparation<'_, '_, 'caps> {
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.meter.step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.runtime.runtime(usage, cancel)
    }
    pub fn owner_association_removed(&self) -> bool {
        matches!(
            self.disposition,
            Some(DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved { .. })
        )
    }
    pub fn commit(
        mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleNotificationPublished<'caps>, Error> {
        self.meter.retain(0, 0, 1, cancel)?;
        self.meter.step(cancel)?;
        let removed = self.owner_association_removed();
        let notification = if self.historical {
            let CompletedOutcome::Created { notification, .. } = &mut self
                .completed
                .as_mut()
                .expect("qualified predecessor")
                .outcome
            else {
                unreachable!("qualified creation")
            };
            notification
        } else {
            let Graph::Rendered { notification, .. } = self
                .graph
                .as_mut()
                .expect("qualified original")
                .value_mut()
                .as_mut()
            else {
                unreachable!("exclusive original graph")
            };
            notification
        };
        // Only infallible scalar/move publication follows. Runtime keeps the
        // actual Store guard until its same-row queue transition is installed.
        *notification = if removed {
            Issuance::OwnerRemoved
        } else {
            Issuance::Issued
        };
        Ok(DomParsedHtmlStyleNotificationPublished {
            _store: self.store,
            disposition: self.disposition.take(),
        })
    }
}
pub(super) fn guard_error(
    error: TryLockError<impl Sized>,
    id: DomParsedHtmlAssociationId,
) -> DomError {
    match error {
        TryLockError::WouldBlock => DomError::ParsedAssociationOperationPending { pending: id },
        TryLockError::Poisoned(_) => DomError::ParsedAssociationProcessingPoisoned { pending: id },
    }
}
fn requirement(
    dom: &Dom,
    meter: &Meter,
    q: &DomParsedHtmlStyleInitialPolicyRequest,
    cancel: &AtomicBool,
) -> Result<Option<DomParsedHtmlStyleNotificationRequirement>, DomError> {
    use DomParsedHtmlStyleNotificationRequirement as R;
    use DomParsedHtmlStyleNotificationUnavailable as U;
    let id = q.contribution();
    meter.step(cancel)?;
    let index = usize::try_from(id.stamp().revision().get())
        .map_err(|_| DomError::RepresentationExhausted)?;
    if dom
        .pending_associations
        .entries
        .get(index)
        .is_none_or(|entry| entry.id != id)
    {
        meter.retain(size_of::<R>(), 0, 1, cancel)?;
        return Ok(Some(R::Unavailable {
            contribution: id,
            reason: U::OriginalEntryMissing,
        }));
    }
    // Only the issuer-controlled token can reach this helper. End authenticates
    // every original row, including this exact already-issued invocation. The
    // fixed terminal header needs no request-graph loan, even if one is leaked.
    if meter.entry().control.execution() == crate::DomParsedHtmlExecutionState::SynchronousEnd {
        return Ok(None);
    }
    Ok(Some(
        match super::frontier::remaining(dom, meter, cancel)? {
            Some(requirement) => requirement,
            None => {
                meter.retain(size_of::<R>(), 0, 1, cancel)?;
                R::Unavailable {
                    contribution: id,
                    reason: U::ConsumerFrontierUnavailable,
                }
            }
        },
    ))
}

impl Dom {
    /// Read this authentic task's synchronous gate. None requires the canonical
    /// producer-and-consumer end, never merely an exhausted inventory or queue.
    pub fn parsed_style_notification_requirement(
        &self,
        token: &DomStyleNotificationToken,
        cancel: &AtomicBool,
    ) -> Result<Option<DomParsedHtmlStyleNotificationRequirement>, DomError> {
        check_cancel(cancel)?;
        if token.owner() != self.owner {
            return Err(DomError::ForeignOwner);
        }
        if token.complete {
            return Err(DomError::StyleProtocol);
        }
        let custody = token.parsed.as_deref().ok_or(DomError::StyleProtocol)?;
        let id = custody.qualification.contribution();
        let meter = custody
            .meter
            .try_lock()
            .map_err(|error| guard_error(error, id))?;
        requirement(self, &meter, &custody.qualification, cancel)
    }
}

#[cfg(test)]
mod tests;
