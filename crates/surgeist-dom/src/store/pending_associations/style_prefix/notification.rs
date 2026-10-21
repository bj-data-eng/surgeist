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

type HandoffStatus = DomParsedHtmlStyleNotificationHandoffStatus;
/// One private affine settlement right. Drop deliberately performs no work.
pub(in crate::store::pending_associations) struct Right;
pub(in crate::store::pending_associations) enum Handoff {
    Retained(Right),
    Transferred,
}
impl Handoff {
    pub(super) fn status(&self) -> HandoffStatus {
        match self {
            Self::Retained(_) => HandoffStatus::Retained,
            Self::Transferred => HandoffStatus::Transferred,
        }
    }
    pub(super) fn take(&mut self) -> Right {
        match std::mem::replace(self, Self::Transferred) {
            Self::Retained(right) => right,
            Self::Transferred => unreachable!("qualified retained authority"),
        }
    }
}

/// Affine completed-rendering authority. It neither proves resource completion
/// nor grants dispatch permission. Ordinary historical output Drop never owns it.
pub struct DomParsedHtmlStyleNotificationContinuation {
    qualification: DomParsedHtmlStyleQualification,
    sheet: DomStyleSheet,
    profile: HtmlStyleExecutionProfile,
    pub(super) right: Option<Right>,
    _meter: Meter,
}
impl std::fmt::Debug for DomParsedHtmlStyleNotificationContinuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleNotificationContinuation")
            .field("qualification", &self.qualification)
            .field("sheet", &self.sheet)
            .finish_non_exhaustive()
    }
}
impl DomParsedHtmlStyleNotificationContinuation {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.qualification
    }
    pub fn sheet(&self) -> &DomStyleSheet {
        &self.sheet
    }
    pub fn profile(&self) -> HtmlStyleExecutionProfile {
        self.profile
    }
}
pub(super) fn form_continuation(
    meter: &Meter,
    request: &DomParsedHtmlStyleInlinePolicyRequest,
    cancel: &AtomicBool,
) -> Result<DomParsedHtmlStyleNotificationContinuation, DomParsedHtmlStyleError> {
    let own = meter.fork();
    own.retain(
        size_of::<DomParsedHtmlStyleNotificationContinuation>(),
        0,
        1,
        cancel,
    )?;
    let sheet = request
        .created_sheet
        .as_ref()
        .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
    Ok(DomParsedHtmlStyleNotificationContinuation {
        qualification: request.qualification.clone(),
        sheet: sheet.sheet.clone(),
        profile: request.cursor.profile(),
        right: None,
        _meter: own,
    })
}
/// Actual original receipt: historical identities and fixed frame disappear
/// before its Meter. Mutex preserves token Send/Sync without an async loan.
pub(crate) struct NotificationCustody {
    qualification: DomParsedHtmlStyleQualification,
    right: Option<Right>,
    attested: bool,
    meter: Mutex<Meter>,
}
pub struct DomParsedHtmlStyleNotificationPreparation<'hold, 'dom, 'caps> {
    input: &'hold mut Option<DomParsedHtmlStyleNotificationContinuation>,
    _dom: &'hold mut &'dom mut Dom,
    runtime: &'hold Meter,
    store: MutexGuard<'caps, CssomStore>,
    disposition: Option<DomParsedHtmlStyleNotificationDisposition>,
    meter: Meter,
}
struct NotificationRetirement {
    _continuation: DomParsedHtmlStyleNotificationContinuation,
    _meter: Meter,
}
/// The actual root stays locked through the composing owner's same-row move.
pub struct DomParsedHtmlStyleNotificationPublished<'caps> {
    _store: MutexGuard<'caps, CssomStore>,
    disposition: Option<DomParsedHtmlStyleNotificationDisposition>,
    _retirement: NotificationRetirement,
}
impl DomParsedHtmlStyleNotificationPublished<'_> {
    pub fn take_disposition(&mut self) -> Option<DomParsedHtmlStyleNotificationDisposition> {
        self.disposition.take()
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    /// Move canonical retained work once. Refusal preserves the actual receiver.
    /// Dropping the explicitly transferred continuation abandons that authority,
    /// never acknowledging history, resource success, dispatch or cleanup.
    pub fn transfer_notification(
        &mut self,
        q: &DomParsedHtmlStyleQualification,
        receiver: &mut Option<DomParsedHtmlStyleNotificationContinuation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomParsedHtmlStyleError> {
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        if receiver.is_some() {
            return Err(DomParsedHtmlStyleError::ReceiverOccupied);
        }
        self.meter.step(cancel)?;
        let mut graph = lock(self.meter.entry())?;
        let mut completed = completed_lock(self.meter.entry())?;
        let current = graph
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if current.value().qualification() == q {
            let Graph::Rendered {
                request,
                notification,
            } = current.value_mut().as_mut()
            else {
                return Err(DomParsedHtmlStyleError::WrongPhase {
                    phase: current.value().phase(),
                });
            };
            if notification.status() != HandoffStatus::Retained {
                return Err(DomParsedHtmlStyleError::NotificationTransferred);
            }
            let mut value = form_continuation(&self.meter, request, cancel)?;
            self.meter.step(cancel)?;
            value.right = Some(notification.take());
            *receiver = Some(value);
            return Ok(());
        }
        let old = completed
            .as_mut()
            .filter(|old| &old.qualification == q)
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        let CompletedOutcome::Created {
            sheet,
            notification,
        } = &mut old.outcome
        else {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        };
        if notification.status() != HandoffStatus::Retained {
            return Err(DomParsedHtmlStyleError::NotificationTransferred);
        }
        let own = self.meter.fork();
        own.retain(
            size_of::<DomParsedHtmlStyleNotificationContinuation>(),
            0,
            1,
            cancel,
        )?;
        let mut value = DomParsedHtmlStyleNotificationContinuation {
            qualification: q.clone(),
            sheet: sheet.clone(),
            profile: old.profile,
            right: None,
            _meter: own,
        };
        self.meter.step(cancel)?;
        value.right = Some(notification.take());
        *receiver = Some(value);
        Ok(())
    }
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    pub fn prepare_notification<'hold, 'caps>(
        &'hold mut self,
        input: &'hold mut Option<DomParsedHtmlStyleNotificationContinuation>,
        expected_owner_label: &str,
        cap: &'caps DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleNotificationPreparation<'hold, 'dom, 'caps>, Error> {
        let continuation = input.as_ref().ok_or(Error::MissingContinuation)?;
        if continuation.right.is_none() {
            return Err(DomError::StyleProtocol.into());
        }
        let qualification = &continuation.qualification;
        let expected = &continuation.sheet;
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        self.meter.step(cancel)?;
        if qualification.contribution() != self.meter.entry().id
            || !std::ptr::eq(continuation._meter.entry(), self.meter.entry())
        {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        if continuation.profile != HtmlStyleExecutionProfile::NoExecutableJavaScript {
            return Err(DomParsedHtmlStyleError::WrongExecutionProfile {
                profile: continuation.profile,
            }
            .into());
        }
        let root = qualification.document().node();
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
        let store = cap.lock().map_err(|e| match e {
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
        let disposition = if removed {
            DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved {
                association: expected.association(),
            }
        } else {
            let own = self.meter.fork();
            own.retain(size_of::<NotificationCustody>(), 0, 0, cancel)?;
            DomParsedHtmlStyleNotificationDisposition::Token(DomStyleNotificationToken {
                origin: Some(DomStyleNotificationOrigin::ParsedOccurrence {
                    contribution: qualification.contribution(),
                    occurrence: qualification.occurrence().clone(),
                }),
                association: expected.association(),
                complete: false,
                parsed: Some(Box::new(NotificationCustody {
                    qualification: qualification.clone(),
                    right: None,
                    attested: false,
                    meter: Mutex::new(own),
                })),
            })
        };
        Ok(DomParsedHtmlStyleNotificationPreparation {
            input,
            _dom: &mut self.dom,
            runtime: &self.meter,
            store,
            disposition: Some(disposition),
            meter,
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
        let next = if self.owner_association_removed() {
            Some(self.meter.entry().control.discharge_notification()?)
        } else {
            None
        };
        let mut continuation = self
            .input
            .take()
            .expect("exclusive authenticated continuation");
        if let Some(next) = next {
            let _settled = continuation.right.take().expect("authenticated right");
            self.meter.entry().control.publish_discharge(next);
        } else if let Some(DomParsedHtmlStyleNotificationDisposition::Token(token)) =
            self.disposition.as_mut()
        {
            token.parsed.as_mut().expect("actual parsed token").right = continuation.right.take();
        }
        Ok(DomParsedHtmlStyleNotificationPublished {
            _store: self.store,
            disposition: self.disposition.take(),
            _retirement: NotificationRetirement {
                _continuation: continuation,
                _meter: self.meter,
            },
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
    q: &DomParsedHtmlStyleQualification,
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

/// Actual trusted completed Style delivery, retaining the one settlement right.
/// Dropping it never decrements the original canonical obligation count.
pub struct DomParsedHtmlStyleDeliveryPublication {
    qualification: DomParsedHtmlStyleQualification,
    association: crate::DomStyleAssociationId,
    _right: Right,
    _meter: Meter,
}
impl std::fmt::Debug for DomParsedHtmlStyleDeliveryPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleDeliveryPublication")
            .field("qualification", &self.qualification)
            .field("association", &self.association)
            .finish_non_exhaustive()
    }
}
impl crate::DomSourceEventExecutor {
    /// Trusted attestation of this exact parsed token's completed Load dispatch.
    /// The bound executor must use its actual completed event receipt. DOM
    /// authenticates binding, lineage and affine consumption, not host callbacks.
    pub fn seal_style_notification(
        &self,
        dom: &Dom,
        token: &mut DomStyleNotificationToken,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleDeliveryPublication, DomError> {
        check_cancel(cancel)?;
        if self.binding.owner != dom.owner || token.owner() != dom.owner {
            return Err(DomError::ForeignOwner);
        }
        if !self.binding.live.load(Ordering::Relaxed)
            || !dom
                .source_event_executor
                .as_ref()
                .and_then(|binding| binding.upgrade())
                .is_some_and(|binding| Arc::ptr_eq(&binding, &self.binding))
        {
            return Err(DomError::InvalidSourceEvent);
        }
        if token.complete {
            return Err(DomError::StyleProtocol);
        }
        let custody = token.parsed.as_mut().ok_or(DomError::StyleProtocol)?;
        if custody.attested || custody.right.is_none() {
            return Err(DomError::StyleProtocol);
        }
        let guard = custody
            .meter
            .try_lock()
            .map_err(|e| guard_error(e, custody.qualification.contribution()))?;
        let meter = guard.fork();
        meter.step(cancel)?;
        let record = dom
            .pending_associations
            .controlled_entry(custody.qualification.contribution())
            .map_err(|_| DomError::StyleProtocol)?;
        if !std::ptr::eq(record.value().as_ref(), meter.entry()) {
            return Err(DomError::StyleProtocol);
        }
        meter.retain(
            size_of::<DomParsedHtmlStyleDeliveryPublication>(),
            0,
            1,
            cancel,
        )?;
        let qualification = custody.qualification.clone();
        meter.step(cancel)?;
        let right = custody.right.take().expect("qualified live right");
        custody.attested = true;
        Ok(DomParsedHtmlStyleDeliveryPublication {
            qualification,
            association: token.association,
            _right: right,
            _meter: meter,
        })
    }
}
