//! The actual readiness publication and pending-bank removal share one fence.
use super::*;
use crate::DomParsedHtmlTerminalError as Error;
#[cfg(test)]
thread_local! { pub(super) static PREPARED_ACK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
struct Output {
    contribution: DomParsedHtmlAssociationId,
    commit: Option<DomCommit>,
    _bank: Option<Bank>,
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _history: history::Retirement,
    _formation: Option<source::formation::Formation>,
    _admission: control::Admission,
    _meter: Meter,
}
/// Actual whole-contribution acknowledgement, canonical readiness commit and
/// original backing/retirement custody. Historical outputs grant no execution.
pub struct DomParsedHtmlContributionCompletion {
    output: Option<Box<Output>>,
}
impl DomParsedHtmlContributionCompletion {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.output
            .as_ref()
            .expect("published terminal output")
            .contribution
    }
    pub fn dom(&self) -> &DomCommit {
        self.output
            .as_ref()
            .and_then(|output| output.commit.as_ref())
            .expect("published readiness")
    }
}
impl std::fmt::Debug for DomParsedHtmlContributionCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlContributionCompletion")
            .field("contribution", &self.contribution())
            .field("dom", self.dom())
            .finish()
    }
}
impl Drop for DomParsedHtmlContributionCompletion {
    fn drop(&mut self) {
        if let Some(boxed) = self.output.take() {
            // Deallocate the Box before disposing represented graphs and their
            // final original Meter. A plain move-out keeps the Box alive longer.
            let output = {
                let inner = boxed;
                *inner
            };
            drop(output);
        }
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    /// Complete an already-ended, no-import contribution with no unsettled
    /// notification rights. This never executes End, resources or event work.
    /// This consumes the temporary command loan on success or refusal.
    /// Refusal preserves the same canonical contribution/controller, cumulative
    /// work and raise-only limits; retry reacquires a Hold with
    /// [`Dom::begin_parsed_html_style`].
    /// Actual participating Style availability and the exact pending bank are
    /// published together; all refusals preserve both and cumulative work.
    pub fn complete_contribution(
        self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlContributionCompletion, Error> {
        let id = self.meter.entry().id;
        self.meter.step(cancel)?;
        if self.meter.entry().control.execution()
            != crate::DomParsedHtmlExecutionState::SynchronousEnd
        {
            return Err(Error::SynchronousEndRequired { contribution: id });
        }
        if let Some(requirement) = frontier::remaining(self.dom, &self.meter, cancel)? {
            return Err(Error::Frontier(
                crate::DomParsedHtmlFrontierError::Remaining(requirement),
            ));
        }
        if self.meter.entry().control.has_imports() {
            return Err(Error::CriticalResourcesUnresolved { contribution: id });
        }
        if let Some(remaining) =
            std::num::NonZeroU64::new(self.meter.entry().control.notification_count())
        {
            return Err(Error::NotificationsUnsettled {
                contribution: id,
                remaining,
            });
        }
        let meter = self.meter.fork();
        meter.retain(
            size_of::<Output>() + size_of::<DomParsedHtmlContributionCompletion>(),
            0,
            0,
            cancel,
        )?;
        let scratch = meter.fork();
        scratch.retain(size_of::<Vec<NodeId>>(), 0, 0, cancel)?;
        let mut targets = Vec::new();
        let view = self.dom.snapshot();
        let mut rows = meter.entry().report.value().value.pending_associations();
        loop {
            meter.step(cancel)?;
            match rows.advance() {
                Step::Fault(fault) => return Err(fault.into()),
                Step::Occurrence(row) if row.style().is_some() => {
                    let document = super::super::consume::mapped(
                        meter.entry(),
                        frontier::original_document(row.document()),
                        &meter,
                        cancel,
                    )?;
                    let mut target = None;
                    for binding in meter.entry().bindings.iter() {
                        meter.step(cancel)?;
                        if binding.source == row.subject().id() {
                            target = Some(binding.node);
                            break;
                        }
                    }
                    let Some(target) = target else {
                        if row.requires_processing() {
                            return Err(DomParsedHtmlStyleError::WrongRequest.into());
                        }
                        continue;
                    };
                    let node = view.node(target)?;
                    let style = node
                        .as_style()
                        .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                    if node.node.document() != document {
                        return Err(DomParsedHtmlStyleError::WrongRequest.into());
                    }
                    if let Some(sheet) = style.state().sheet.as_ref()
                        && (sheet.association.element != target
                            || sheet.association.serial != style.state().next_association
                            || sheet.root != document.node()
                            || self.dom.style_cssom_roots.get(sheet.root).is_none())
                    {
                        return Err(DomParsedHtmlStyleError::WrongRequest.into());
                    }
                    if style.state().availability == crate::DomStyleSheetAvailability::Established {
                        continue;
                    }
                    let mut seen = false;
                    for prior in &targets {
                        scratch.step(cancel)?;
                        if *prior == target {
                            seen = true;
                            break;
                        }
                    }
                    if !seen {
                        scratch.reserve(&mut targets, 1, cancel)?;
                        targets.push(target);
                    }
                }
                Step::Complete => break,
                Step::Document(_) | Step::Occurrence(_) | Step::Skipped => (),
            }
        }
        drop(view);
        self.dom.custody_registry.reconcile_releases();
        let baseline = self.dom.custody_registry.total();
        let admission = meter.admission(baseline, cancel)?;
        let mut draft = Draft::new_parsed(self.dom, Arc::new(BatchIdentity), &admission, cancel)?;
        for target in &targets {
            admission.step(cancel)?;
            let mut style = draft.style_state_mut(Key::Existing(*target), cancel)?;
            if style.availability != crate::DomStyleSheetAvailability::PendingParsedProcessing {
                return Err(DomParsedHtmlStyleError::WrongRequest.into());
            }
            style.availability = crate::DomStyleSheetAvailability::Established;
            style.commit(cancel)?;
        }
        let prepared = draft.finish(self.dom, cancel)?;
        let mut publication = self.dom.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        let history = self.dom.history.retain_evictions(
            &publication.history_plan,
            self.dom.custody_registry.ledger(),
            |total| admission.controlled(total, cancel),
            || admission.step(cancel),
        )?;
        let acknowledgement = self.dom.pending_associations.prepare_remove(
            id,
            self.dom.limits,
            |total| meter.controlled(total, baseline, cancel),
            || meter.step(cancel),
        )?;
        #[cfg(test)]
        PREPARED_ACK.with(|count| count.set(count.get() + 1));
        drop(draft);
        drop(targets);
        drop(scratch);
        meter.retain(
            0,
            0,
            publication
                .changes
                .len()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        let mut result = DomParsedHtmlContributionCompletion {
            output: Some(Box::new(Output {
                contribution: id,
                commit: None,
                _bank: None,
                _state: self.dom.state.clone(),
                _state_custody: self.dom.state_custody.clone(),
                _history: history,
                _formation: publication._formation.take(),
                _admission: admission,
                _meter: meter,
            })),
        };
        let output = result.output.as_mut().expect("preformed terminal output");
        output._admission.step(cancel)?;
        // Both independent publications were fully prepared. No callback,
        // allocation, destructor or fallible operation separates these moves.
        output.commit = Some(self.dom.publish_prepared(publication));
        output._bank = Some(std::mem::replace(
            &mut self.dom.pending_associations,
            acknowledgement,
        ));
        Ok(result)
    }
}
