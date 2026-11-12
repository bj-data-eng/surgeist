//! Deliver one reached CSP pragma under the original private Source owner.

use super::*;
use crate::meta::{CspState, SelectedCsp};
use crate::store::meta::CspInsertion;
use std::cell::{Cell, RefCell};
use surgeist_html::security::csp::{
    HtmlCspAssociation, HtmlCspError, HtmlCspLayout, HtmlCspOwner, HtmlCspProtocolError,
    HtmlCspStorage, HtmlCspUnavailable, HtmlMetaCspOutcome, HtmlMetaCspPrepared,
    HtmlMetaCspProcessor, HtmlMetaCspRequest,
};

/// Failure to deliver the exact reached native Meta pragma.
#[derive(Debug)]
pub enum DomSourceMetaCspError<E> {
    /// Canonical Source qualification or admission refused the operation.
    Dom(DomError),
    /// The original HTML policy protocol rejected a qualification or result.
    Protocol(HtmlCspProtocolError),
    /// The actual DOM policy issuer is already borrowed.
    IssuerBusy,
    /// The canonical Document has no qualified prior policy container.
    PolicyUnavailable,
    /// The processor lacks a required capability for this occurrence.
    ProcessorUnavailable(HtmlCspUnavailable),
    /// The processor refused with its original typed error.
    ProcessorRefused(DomSourceMetaCspProcessorFailure<E>),
    /// Processing failed with its original typed error.
    ProcessorFailed(DomSourceMetaCspProcessorFailure<E>),
}

/// An original processor error and its admitted retained backing.
pub struct DomSourceMetaCspProcessorFailure<E> {
    reason: E,
    _retained: Option<custody::Debit>,
}

impl<E> DomSourceMetaCspProcessorFailure<E> {
    /// The original processor error, borrowed with its retained storage.
    pub const fn reason(&self) -> &E {
        &self.reason
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for DomSourceMetaCspProcessorFailure<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}

impl<E: std::fmt::Debug> std::fmt::Display for DomSourceMetaCspError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native Meta CSP delivery refused: {self:?}")
    }
}

impl<E: std::error::Error + 'static> std::error::Error for DomSourceMetaCspError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::ProcessorRefused(error) | Self::ProcessorFailed(error) => Some(error.reason()),
            _ => None,
        }
    }
}

impl<E> From<DomError> for DomSourceMetaCspError<E> {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl<E> From<HtmlCspError<DomError>> for DomSourceMetaCspError<E> {
    fn from(error: HtmlCspError<DomError>) -> Self {
        match error {
            HtmlCspError::Protocol(error) => Self::Protocol(error),
            HtmlCspError::Interrupted(error) => Self::Dom(error),
        }
    }
}

struct Admitted<T> {
    value: T,
    retained: formation::Formation,
}

pub(in crate::store::source::operation) struct CspDelivery {
    prior: Option<Admitted<HtmlCspAssociation>>,
    request: Option<Admitted<HtmlMetaCspRequest>>,
    prepared: Option<Admitted<HtmlMetaCspPrepared>>,
    selected: Option<Admitted<SelectedCsp>>,
    _header: formation::Formation,
}

impl DomSourceLease<'_> {
    /// Process the exact private Meta occurrence through its original
    /// Document policy issuer. The subsequent Source advance publishes the
    /// insertion, its policy and the remaining native effects.
    pub fn process_meta_csp<P: HtmlMetaCspProcessor>(
        &mut self,
        request: DomSourceMetaCspRequest,
        processor: &mut P,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceMetaCspError<P::Error>> {
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .filter(|operation| !self.complete && operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let frame = operation
            .draft
            .pending_meta_csp()
            .filter(|frame| {
                frame
                    .ordinal
                    .is_some_and(|ordinal| operation.meta_csp_request(*frame, ordinal) == request)
            })
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.cancelled && !operation.published && !operation.event_exposed {
            return Err(DomError::Cancelled.into());
        }
        operation.refresh_admission(cancel)?;
        let input_bytes = operation.input_bytes()?;
        let budget = formation::Formation::new(&operation.draft, cancel)?
            .ok_or(DomError::InvalidSourceOperation)?;
        let mut issuer = self
            .dom
            .csp_owner
            .value()
            .try_borrow_mut()
            .map_err(|_| DomSourceMetaCspError::IssuerBusy)?;
        if operation.meta_csp.is_none() {
            let header = budget.fork();
            header.step(cancel)?;
            header.retain_bytes(size_of::<CspDelivery>(), cancel)?;
            operation.meta_csp = Some(formation::allocated_box(CspDelivery {
                prior: None,
                request: None,
                prepared: None,
                selected: None,
                _header: header,
            }));
        }
        let delivery = operation
            .meta_csp
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        delivery.prepare(
            &operation.draft,
            frame,
            &mut issuer,
            processor,
            &budget,
            cancel,
        )?;
        drop(issuer);
        let selected = delivery
            .selected
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let checkpoint = operation.draft.source_checkpoint(input_bytes, 0, cancel)?;
        let installed = (|| {
            let previous = operation
                .draft
                .document_metadata(frame.document)?
                .policy_container()
                .csp_units();
            let units = operation
                .draft
                .units
                .checked_sub(previous)
                .and_then(|units| units.checked_add(selected.value.units()))
                .ok_or(DomError::RepresentationExhausted)?;
            admit(units, self.dom.limits.units, DomResource::StringCodeUnits)?;
            let retained = budget.fork();
            retained.retain_payload(
                storage::Payload {
                    bytes: selected
                        .value
                        .bytes()
                        .ok_or(DomError::RepresentationExhausted)?,
                    units: selected.value.units(),
                },
                cancel,
            )?;
            let mut metadata = operation
                .draft
                .document_metadata_mut(frame.document, cancel)?;
            metadata.policy_container_mut().csp = CspState::Associated(selected.value.clone());
            metadata.commit_formed(Some(retained), cancel)?;
            let Some(select::Effect::MetaCsp(effect)) = operation
                .draft
                .effects
                .last_mut()
                .map(|entry| &mut entry.effect)
            else {
                return Err(DomError::InvalidSourceOperation);
            };
            effect.applied = true;
            operation.draft.units = units;
            Ok(())
        })();
        if let Err(error) = installed {
            checkpoint.restore(&mut operation.draft);
            return Err(error.into());
        }
        drop(checkpoint);
        operation.meta_csp = None;
        Ok(())
    }
}

impl CspDelivery {
    fn prepare<P: HtmlMetaCspProcessor>(
        &mut self,
        draft: &Draft,
        frame: CspInsertion,
        issuer: &mut HtmlCspOwner,
        processor: &mut P,
        budget: &formation::Formation,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceMetaCspError<P::Error>> {
        if let Some(selected) = &self.selected {
            let control = Admission::new(budget, cancel);
            let qualification = selected
                .value
                .association()
                .latest()
                .ok_or(DomError::InvalidSourceOperation)?;
            qualification
                .processor()
                .validate_selected_with(processor.profile(), &mut |layout| control.admit(layout))?;
            return Ok(());
        }
        if self.prior.is_none() {
            let control = Admission::new(budget, cancel);
            let state = draft
                .document_metadata(frame.document)?
                .policy_container()
                .csp();
            let value = match state {
                crate::DomDocumentCspState::ConstructorInitialEmpty => {
                    issuer.initial_document_with(&mut |layout| control.admit(layout))?
                }
                crate::DomDocumentCspState::Associated(prior) => {
                    control.retain_alias(
                        prior
                            .retained_layout()
                            .map_err(DomSourceMetaCspError::Protocol)?
                            .storage(),
                    )?;
                    prior.clone()
                }
                crate::DomDocumentCspState::Unavailable => {
                    return Err(DomSourceMetaCspError::PolicyUnavailable);
                }
            };
            self.prior = Some(control.finish(value));
        }
        let prior = self
            .prior
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        if let Some(request) = &self.request {
            let control = Admission::new(budget, cancel);
            request
                .value
                .qualification()
                .processor()
                .validate_selected_with(processor.profile(), &mut |layout| control.admit(layout))?;
        } else {
            let control = Admission::new(budget, cancel);
            let content = draft.meta_csp_content(frame, cancel)?;
            let request = issuer
                .request_meta_insertion_with(
                    &prior.value,
                    processor.profile(),
                    content.clone(),
                    &mut |layout| control.admit(layout),
                )
                .map_err(|rejected| rejected.error)?;
            self.request = Some(control.finish(request));
        }
        if self.prepared.is_none() {
            let request = self
                .request
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            let control = Admission::new(budget, cancel);
            let outcome = processor.prepare(
                &request.value,
                &mut |layout| control.admit(layout),
                &mut |storage| control.release(storage),
            )?;
            if control.invalid_release.get() {
                return Err(DomError::HtmlChargeProtocol.into());
            }
            match outcome {
                HtmlMetaCspOutcome::Completed(prepared) => {
                    self.prepared = Some(control.finish(prepared));
                }
                HtmlMetaCspOutcome::Unavailable(reason) => {
                    return Err(DomSourceMetaCspError::ProcessorUnavailable(reason));
                }
                HtmlMetaCspOutcome::Cancelled => return Err(DomError::Cancelled.into()),
                HtmlMetaCspOutcome::Refused(reason) => {
                    let admitted = control.finish(reason);
                    return Err(DomSourceMetaCspError::ProcessorRefused(
                        DomSourceMetaCspProcessorFailure {
                            reason: admitted.value,
                            _retained: admitted.retained.into_debit(),
                        },
                    ));
                }
                HtmlMetaCspOutcome::Failed(reason) => {
                    let admitted = control.finish(reason);
                    return Err(DomSourceMetaCspError::ProcessorFailed(
                        DomSourceMetaCspProcessorFailure {
                            reason: admitted.value,
                            _retained: admitted.retained.into_debit(),
                        },
                    ));
                }
            }
        }
        let control = Admission::new(budget, cancel);
        // This wrapper is additional to the HTML publication receipt.
        control.retained.step(cancel)?;
        let wrapper = custody::Usage::new(SelectedCsp::wrapper_bytes(), 0);
        control.retain(wrapper)?;
        let request = self
            .request
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let prepared = self
            .prepared
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let publication = issuer.prepare_publication_with(
            &mut prior.value,
            request.value,
            prepared.value,
            &mut |layout| control.admit(layout),
        );
        match publication {
            Ok(publication) => drop(publication.publish()),
            Err(rejected) => {
                let (original_request, original_prepared) = rejected.retained;
                self.request = Some(Admitted {
                    value: original_request,
                    retained: request.retained,
                });
                self.prepared = Some(Admitted {
                    value: original_prepared,
                    retained: prepared.retained,
                });
                return Err(rejected.error.into());
            }
        }
        let prior = self.prior.take().ok_or(DomError::InvalidSourceOperation)?;
        let layout = prior
            .value
            .retained_layout()
            .map_err(DomSourceMetaCspError::Protocol)?;
        control.trim_to(usage(layout.storage()).checked_add(wrapper)?)?;
        self.selected = Some(control.finish(SelectedCsp::new(prior.value, layout)));
        Ok(())
    }
}

struct Admission<'a> {
    retained: formation::Formation,
    scratch: RefCell<Option<formation::Formation>>,
    usage: Cell<custody::Usage>,
    invalid_release: Cell<bool>,
    cancel: &'a AtomicBool,
}

impl<'a> Admission<'a> {
    fn new(budget: &formation::Formation, cancel: &'a AtomicBool) -> Self {
        Self {
            retained: budget.fork(),
            scratch: RefCell::new(None),
            usage: Cell::new(custody::Usage::default()),
            invalid_release: Cell::new(false),
            cancel,
        }
    }

    fn admit(&self, layout: HtmlCspLayout) -> Result<(), DomError> {
        if self.invalid_release.get() {
            return Err(DomError::HtmlChargeProtocol);
        }
        // Successive callbacks bracket distinct native action frames.
        self.scratch.borrow_mut().take();
        layout
            .charge_work(&mut crate::store::html_credits::Credits(|credits| {
                for _ in 0..credits {
                    self.retained.step(self.cancel)?;
                }
                Ok::<(), DomError>(())
            }))
            .map_err(crate::store::html_credits::charged)?;
        let transferred = usage(layout.transferred().storage());
        let retained = usage(layout.retained().storage());
        let additional = retained.checked_sub(transferred)?;
        let scratch = usage(layout.peak())
            .checked_sub(retained)?
            .checked_add(custody::Usage::new(layout.atomic_bytes(), 0))?;
        let loan = self.retained.fork();
        loan.retain_payload(payload(scratch), self.cancel)?;
        self.retain(additional)?;
        *self.scratch.borrow_mut() = Some(loan);
        Ok(())
    }

    fn retain_alias(&self, storage: HtmlCspStorage) -> Result<(), DomError> {
        for _ in 0..4 {
            self.retained.step(self.cancel)?;
        }
        self.retain(usage(storage))
    }

    fn retain(&self, incoming: custody::Usage) -> Result<(), DomError> {
        let total = self.usage.get().checked_add(incoming)?;
        self.retained
            .retain_payload(payload(incoming), self.cancel)?;
        self.usage.set(total);
        Ok(())
    }

    fn release(&self, storage: HtmlCspStorage) {
        let outgoing = usage(storage);
        match self.usage.get().checked_sub(outgoing) {
            Ok(remaining) => {
                self.retained.release_payload(payload(outgoing));
                self.usage.set(remaining);
            }
            Err(_) => self.invalid_release.set(true),
        }
    }

    fn trim_to(&self, retained: custody::Usage) -> Result<(), DomError> {
        let excess = self.usage.get().checked_sub(retained)?;
        self.retained.release_payload(payload(excess));
        self.usage.set(retained);
        Ok(())
    }

    fn finish<T>(self, value: T) -> Admitted<T> {
        Admitted {
            value,
            retained: self.retained,
        }
    }
}

fn usage(storage: HtmlCspStorage) -> custody::Usage {
    custody::Usage::new(storage.bytes(), storage.utf16_units())
}

fn payload(usage: custody::Usage) -> storage::Payload {
    storage::Payload {
        bytes: usage.bytes(),
        units: usage.units(),
    }
}
