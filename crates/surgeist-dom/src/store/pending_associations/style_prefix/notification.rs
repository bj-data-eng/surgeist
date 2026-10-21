//! Original notification issuance and metered absence of a dispatch frontier.
use super::*;
use crate::parsed_style::DomParsedHtmlStyleNotificationError as Error;
use crate::{
    DomStyleCssomRoot, DomStyleNotificationOrigin, DomStyleNotificationToken, DomStyleSheet,
};
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
            (old.notification, old.profile, &old.sheet)
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
            &mut self
                .completed
                .as_mut()
                .expect("qualified predecessor")
                .notification
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
fn guard_error(error: TryLockError<impl Sized>, id: DomParsedHtmlAssociationId) -> DomError {
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
) -> Result<DomParsedHtmlStyleNotificationRequirement, DomError> {
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
        return Ok(R::Unavailable {
            contribution: id,
            reason: U::OriginalEntryMissing,
        });
    }
    let report = &meter.entry().report.value().value;
    let ordinary = meter.entry().context_basis.is_none()
        && report.document().is_some()
        && report.effective_scripting_mode() == surgeist_html::HtmlParserScriptingMode::Disabled;
    let graph = meter
        .entry()
        .style_prefix
        .try_lock()
        .map_err(|error| guard_error(error, id))?;
    let completed = meter
        .entry()
        .completed_style
        .try_lock()
        .map_err(|e| guard_error(e, id))?;
    let executed = matches!(graph.as_ref().map(|stored|stored.value().as_ref()),Some(Graph::Rendered{request,notification:Issuance::Issued}) if request.qualification()==q)
        || completed
            .as_ref()
            .is_some_and(|old| &old.qualification == q && old.notification == Issuance::Issued);
    let phase = graph
        .as_ref()
        .map_or(DomParsedHtmlStylePhase::Unstarted, |g| g.value().phase());
    // The first remaining candidate may coexist with the final unavailable
    // result until full inventory validation ends. Admit that exact header
    // overlap and one returned output before cloning any occurrence identity.
    meter.retain(size_of::<Option<R>>() + size_of::<R>(), 0, 1, cancel)?;
    let mut first = None;
    let mut missing = None;
    let mut seen = false;
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Document(row) => {
                if let surgeist_html_parser::HtmlParseAssociationCaptureDisposition::Unavailable(
                    kind,
                ) = row.disposition()
                {
                    missing.get_or_insert(U::IncompleteCapture(kind));
                }
            }
            Step::Occurrence(row) if row.requires_processing() => {
                let current_done = matches!(graph.as_ref().map(|g|g.value().as_ref()), Some(Graph::Rendered{request,..}) if request.qualification().occurrence()==&row.id());
                let prior_done = completed
                    .as_ref()
                    .is_some_and(|old| old.qualification.occurrence() == &row.id());
                if current_done || prior_done {
                    if &row.id() == q.occurrence() && executed {
                        seen = true;
                    }
                    continue;
                }
                if first.is_none() {
                    let source = match row.document() {
                        HtmlParseMetadataDocument::Owned(node) => {
                            crate::DomParsedHtmlHistoricalDocument::Constructed(node.id())
                        }
                        HtmlParseMetadataDocument::Captured(node) => {
                            crate::DomParsedHtmlHistoricalDocument::Captured(node.id())
                        }
                    };
                    let mut document = None;
                    for binding in &meter.entry().documents {
                        meter.step(cancel)?;
                        if binding.source == source {
                            document = Some(binding.document);
                            break;
                        }
                    }
                    if let Some(document) = document {
                        first = Some(
                            if graph.as_ref().is_some_and(|g| {
                                g.value().qualification().occurrence() == &row.id()
                            }) {
                                R::Invocation {
                                    contribution: id,
                                    occurrence: row.id(),
                                    document,
                                    phase,
                                }
                            } else {
                                R::Occurrence {
                                    contribution: id,
                                    occurrence: row.id(),
                                    document,
                                    kind: row.kind(),
                                }
                            },
                        );
                    } else {
                        missing.get_or_insert(U::UnsupportedInventory);
                    }
                }
            }
            Step::Complete => break,
            _ => (),
        }
    }
    let result = if !ordinary {
        R::Unavailable {
            contribution: id,
            reason: U::UnsupportedInventory,
        }
    } else if let Some(reason) = missing {
        R::Unavailable {
            contribution: id,
            reason,
        }
    } else if !executed {
        R::Invocation {
            contribution: id,
            occurrence: q.occurrence().clone(),
            document: q.document(),
            phase,
        }
    } else if !seen {
        R::Unavailable {
            contribution: id,
            reason: U::InvocationNotExecuted,
        }
    } else {
        first.unwrap_or(R::Unavailable {
            contribution: id,
            reason: U::ConsumerFrontierUnavailable,
        })
    };
    Ok(result)
}
impl Dom {
    /// Read the actual remaining synchronous requirement. This slice never
    /// returns a positive frontier, even if the captured inventory is exhausted.
    pub fn parsed_style_notification_requirement(
        &self,
        token: &DomStyleNotificationToken,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleNotificationRequirement, DomError> {
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
