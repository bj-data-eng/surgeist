//! A new subject reuses the original controller, never another history registry.
use super::*;
use crate::DomParsedHtmlStyleSubjectError as Error;
use surgeist_html_parser::HtmlParseAssociationOccurrenceId;
/// The new exact request and actual retired original graph custody. No sheet,
/// DOM/CSSOM publication, resource result or whole-history acknowledgement.
pub struct DomParsedHtmlStyleSubjectCompletion {
    qualification: DomParsedHtmlStyleQualification,
    _text: DocumentString,
    _nonce: Option<DocumentString>,
    _policy: Option<HtmlCspAssociation>,
    _previous: Option<CompletedInvocation>,
    _meter: Meter,
}
impl DomParsedHtmlStyleSubjectCompletion {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.qualification
    }
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        DomParsedHtmlStylePhase::InlinePolicyPending
    }
}
impl std::fmt::Debug for DomParsedHtmlStyleSubjectCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleSubjectCompletion")
            .field("qualification", &self.qualification)
            .finish()
    }
}
pub(super) fn ensure_transferred(
    graph: &Graph,
    previous: Option<&CompletedInvocation>,
) -> Result<(), DomParsedHtmlStyleError> {
    if let Graph::Rendered {
        request,
        notification: notification::Handoff::Retained(_),
    } = graph
    {
        return Err(DomParsedHtmlStyleError::HandoffRequired {
            occurrence: request.qualification.occurrence().clone(),
        });
    }
    if let Some(CompletedInvocation {
        qualification,
        outcome:
            CompletedOutcome::Created {
                notification: notification::Handoff::Retained(_),
                ..
            },
        ..
    }) = previous
    {
        return Err(DomParsedHtmlStyleError::HandoffRequired {
            occurrence: qualification.occurrence().clone(),
        });
    }
    Ok(())
}
pub(super) fn selection(
    meter: &Meter,
    row: &surgeist_html_parser::HtmlParseAssociationOccurrence<'_>,
    old: &DomParsedHtmlStyleInlinePolicyRequest,
    cancel: &AtomicBool,
) -> Result<Selection, DomParsedHtmlStyleError> {
    let context = row.context();
    let requirement = context.requirement(Role::InitialPolicy);
    let origin = match (requirement, context.csp_association(), &old.origin) {
        (Requirement::Known, Some(policy), _) => StylePolicyOrigin::known(policy),
        (Requirement::OfflineUnbound, None, StylePolicyOrigin::Offline(previous)) => {
            let slot = context
                .slot(Role::InitialPolicy)
                .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
            if slot != *previous || context.input_epoch(Role::InitialPolicy) != *previous.epoch() {
                return Err(DomParsedHtmlStyleError::PolicyContext { requirement });
            }
            StylePolicyOrigin::Offline(slot)
        }
        _ => return Err(DomParsedHtmlStyleError::PolicyContext { requirement }),
    };
    let inputs = row.style().ok_or(DomParsedHtmlStyleError::WrongRequest)?;
    if !inputs.connected() || !inputs.type_eligible() {
        return Err(DomParsedHtmlStyleError::UnsupportedProfile {
            contribution: meter.entry().id,
        });
    }
    let HtmlParseMetadataDocument::Owned(original) = row.document() else {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    };
    let document = super::super::consume::mapped(
        meter.entry(),
        frontier::original_document(row.document()),
        meter,
        cancel,
    )?;
    let mut cursor = HtmlStyleUpdateCursor::new(old.cursor.profile());
    for reply in [
        Reply::Applied,
        Reply::Connected(inputs.connected()),
        Reply::TypeEligible(inputs.type_eligible()),
    ] {
        meter.step(cancel)?;
        cursor.reply(reply)?;
    }
    Ok(Selection {
        qualification: DomParsedHtmlStyleQualification {
            contribution: meter.entry().id,
            occurrence: row.id(),
            subject: row.subject().id(),
            original_document: crate::DomParsedHtmlHistoricalDocument::Constructed(original.id()),
            document,
            context_epoch: context.epoch(),
        },
        state: inputs.state(),
        origin,
        cursor,
    })
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    pub fn advance_subject(
        &mut self,
        expected: &HtmlParseAssociationOccurrenceId,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSubjectCompletion, Error> {
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        let mut graph = lock(self.meter.entry())?;
        let mut completed = completed_lock(self.meter.entry())?;
        let held = graph
            .as_mut()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        let old = match held.value().as_ref() {
            Graph::Rendered { request, .. } => request,
            Graph::Inline(request)
                if request.cursor.action() == Action::Complete
                    && request.created_sheet.is_none() =>
            {
                request
            }
            _ => {
                return Err(DomParsedHtmlStyleError::WrongPhase {
                    phase: held.value().phase(),
                }
                .into());
            }
        };
        let scan = frontier::scan(self.dom, &self.meter, Some(expected), cancel)?;
        if scan.covered || old.qualification.occurrence() == expected {
            return Err(Error::AlreadySelected);
        }
        if let Some(reason) = scan.missing {
            self.meter.retain(
                size_of::<DomParsedHtmlStyleNotificationRequirement>(),
                0,
                1,
                cancel,
            )?;
            return Err(Error::Remaining(
                DomParsedHtmlStyleNotificationRequirement::Unavailable {
                    contribution: self.meter.entry().id,
                    reason,
                },
            ));
        }
        let row = scan.next.ok_or(Error::WrongOccurrence)?;
        if !matches!(
            row.kind(),
            Kind::StyleUpdate(surgeist_html::HtmlStyleTrigger::Insertion {
                connected: true,
                on_parser_stack: false
            })
        ) || row.subject().id() == *old.qualification.subject()
        {
            return Err(Error::Remaining(frontier::diagnostic(
                &self.meter,
                &row,
                None,
                cancel,
            )?));
        }
        if row.id() != *expected {
            return Err(Error::WrongOccurrence);
        }
        ensure_transferred(held.value(), completed.as_ref())?;
        let mut rows = self
            .meter
            .entry()
            .report
            .value()
            .value
            .pending_associations();
        let mut prior_seen = row.prior_style().is_none();
        let mut earlier_influence = false;
        loop {
            self.meter.step(cancel)?;
            match rows.advance() {
                Step::Fault(fault) => return Err(fault.into()),
                Step::Occurrence(candidate) if candidate.id() == row.id() => break,
                Step::Occurrence(candidate) if candidate.subject().id() == row.subject().id() => {
                    earlier_influence |= candidate.requires_processing();
                    if row.prior_style().as_ref() == Some(&candidate.id()) {
                        prior_seen = !candidate.requires_processing();
                    }
                }
                Step::Complete => return Err(DomParsedHtmlStyleError::WrongRequest.into()),
                _ => (),
            }
        }
        // Disconnected attribute setup is legitimate no-effect history, but an
        // earlier influencing invocation cannot be hidden behind its last edge.
        if !prior_seen || earlier_influence {
            return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                contribution: self.meter.entry().id,
            }
            .into());
        }
        let formed = self
            .meter
            .fork()
            .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
        let selection = selection(&formed, &row, old, cancel)?;
        if selection.qualification.document() != old.qualification.document()
            || selection.qualification.original_document() != old.qualification.original_document()
        {
            return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                contribution: self.meter.entry().id,
            }
            .into());
        }
        let mut element = None;
        for binding in self.meter.entry().bindings.iter() {
            self.meter.step(cancel)?;
            if binding.source == *selection.qualification.subject() {
                element = Some(binding.node);
                break;
            }
        }
        let element = element.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        let snapshot = self.dom.snapshot();
        let node = snapshot.node(element)?;
        let style = node
            .as_style()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if node.node.document() != selection.qualification.document()
            || style.state().sheet.is_some()
            || style.state().next_association != 0
            || style.state().availability
                != crate::DomStyleSheetAvailability::PendingParsedProcessing
        {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        drop(snapshot);
        let (policy, policy_usage, atomic) = match &selection.origin {
            StylePolicyOrigin::Known { .. } => {
                let (p, u, a) = form_original_policy(&formed, &selection, cancel)?;
                (Some(p), u, a)
            }
            StylePolicyOrigin::Offline(_) => (None, custody::Usage::ZERO, 0),
        };
        let (text, nonce) = form_text_nonce(&formed, selection.qualification.occurrence(), cancel)?;
        let new_usage = text_usage(&text, nonce.as_ref())?.checked_add(policy_usage)?;
        let mut old_usage = text_usage(&old.text, old.nonce.as_ref())?;
        if policy.is_some() {
            old_usage = old_usage.checked_add(super::policy_usage(&old.policy)?)?;
        }
        let next_usage = self
            .dom
            .pending_associations
            .usage
            .checked_add(new_usage)?
            .checked_sub(old_usage)?;
        Bank::admit_usage(next_usage, self.dom.limits)?;
        let meter = self.meter.fork();
        meter.retain(
            size_of::<DomParsedHtmlStyleSubjectCompletion>(),
            0,
            1,
            cancel,
        )?;
        let qualification = selection.qualification.clone();
        self.meter.check_peak(custody::Usage::ZERO)?;
        self.meter.step(cancel)?;
        let prior = completed.take();
        let old = std::mem::replace(held.value_mut().as_mut(), Graph::Transition);
        let request = match old {
            Graph::Rendered { request, .. } | Graph::Inline(request) => request,
            _ => unreachable!("qualified completed current invocation"),
        };
        let DomParsedHtmlStyleInlinePolicyRequest {
            text: old_text,
            nonce: old_nonce,
            policy: old_policy,
            ..
        } = request;
        meter.retire_persistent(
            held.debit_mut()
                .split(old_usage)
                .expect("actual old payload"),
            old_usage,
        );
        held.debit_mut()
            .absorb(formed.detach(new_usage))
            .expect("one original ledger");
        self.meter.entry().control.add_persistent(new_usage);
        self.meter.entry().control.reserve_cleanup(atomic);
        self.dom.pending_associations.usage = next_usage;
        let (policy, retired_policy) = match policy {
            Some(next) => (next, Some(old_policy)),
            None => (old_policy, None),
        };
        **held.value_mut() = Graph::Inline(DomParsedHtmlStyleInlinePolicyRequest {
            qualification: selection.qualification,
            origin: selection.origin,
            text,
            nonce,
            state: selection.state,
            policy,
            cursor: selection.cursor,
            created_sheet: None,
        });
        Ok(DomParsedHtmlStyleSubjectCompletion {
            qualification,
            _text: old_text,
            _nonce: old_nonce,
            _policy: retired_policy,
            _previous: prior,
            _meter: meter,
        })
    }
}

#[cfg(test)]
mod tests;
