//! Synchronous HTML removing steps, committed with the canonical tree edit.
use super::super::*;
use crate::internal::{
    document::DomFocusedAreaDesignation,
    hosted::{DocumentAssociation, DocumentPhase, DomHostedError},
};

pub(in crate::internal::store) type Pending = custody::Held<Vec<Arc<DocumentAssociation>>>;

#[derive(Default)]
pub(in crate::internal::store) struct Prepared(Option<custody::Held<Vec<Reset>>>);

struct Reset {
    flag: Arc<AtomicBool>,
    _association: Arc<DocumentAssociation>,
}

impl Prepared {
    pub(in crate::internal::store) fn publish(self) {
        if let Some(resets) = self.0 {
            for reset in resets.value() {
                reset.flag.store(false, Ordering::Relaxed);
            }
        }
    }
}

pub(in crate::internal::store) fn prepare(
    pending: Option<Pending>,
    ledger: &custody::Ledger,
    formation: Option<&source::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<Prepared, DomError> {
    let Some(pending) = pending else {
        return Ok(Prepared::default());
    };
    let usage = |capacity| {
        usize::checked_mul(capacity, size_of::<Reset>())
            .map(|bytes| custody::Usage::new(bytes, 0))
            .ok_or(DomError::RepresentationExhausted)
    };
    let admit = |usage| match formation {
        Some(formation) => formation.admit_custody_usage(usage, cancel),
        None => check_cancel(cancel),
    };
    let len = pending.value().len();
    let mut debit = ledger.retain(usage(len)?, admit)?;
    let mut resets = Vec::new();
    resets
        .try_reserve_exact(len)
        .map_err(|_| DomError::AllocationFailed)?;
    transaction::nodes::resize_debit(&mut debit, usage(resets.capacity())?, admit)?;
    for association in pending.value() {
        source::formation::step(formation, cancel)?;
        // An issuer can expire without changing the canonical DOM stamp while
        // this Source segment is parked on an unrelated private request.
        association.validate_live().map_err(association_error)?;
        let flag = association
            .window_navigation
            .as_ref()
            .and_then(|binding| binding.focus_changed.upgrade())
            .ok_or(DomError::HostedSourceUnavailable)?;
        resets.push(Reset {
            flag,
            _association: association.clone(),
        });
    }
    Ok(Prepared(Some(custody::Held::new(resets, debit))))
}

impl Draft {
    pub(in crate::internal::store) fn focus_removing_element(
        &mut self,
        element: Key,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let subject = self.published_id(element);
        match self.node(document)?.focused_area_designation() {
            DomFocusedAreaDesignation::Element(designated) if subject == Some(designated) => {
                self.stage_focus_removal(document, cancel)
            }
            DomFocusedAreaDesignation::Unavailable => {
                let document = self.published_id(document).map(DocumentId);
                // A wholly unpublished fresh identity cannot equal the
                // original area of an already published Document.
                if matches!(element, Key::Created(_)) && subject.is_none() && document.is_some() {
                    return Ok(());
                }
                Err(DomError::FocusedAreaDesignationUnavailable { subject, document })
            }
            _ => Ok(()),
        }
    }

    fn stage_focus_removal(&mut self, document: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let document_id = self
            .published_id(document)
            .map(DocumentId)
            .ok_or(DomError::HostedSourceUnavailable)?;
        let binding = self.focus_removal_window(document, document_id, cancel)?;
        let undo = self.prepare_undo_slots(2, cancel)?;
        let empty = Vec::new();
        let pending = self
            .focus_removals
            .as_ref()
            .map_or(&empty, custody::Held::value);
        if let Some(next) = self.prepare_output_buffer(pending, 1, cancel)? {
            let before = self.focus_removals.replace(next);
            if undo {
                self.journal
                    .push_prepared(transaction::Undo::FocusRemovalsCapacity(before));
            }
        }
        self.effect_step(cancel)?;
        let pending = self
            .focus_removals
            .as_mut()
            .ok_or(DomError::RepresentationExhausted)?
            .value_mut();
        let len = pending.len();
        pending.push(binding);
        if undo {
            self.journal
                .push_prepared(transaction::Undo::FocusRemovalsLength(len));
        }
        let mut designation = self.focused_area_designation_mut(document, cancel)?;
        *designation = DomFocusedAreaDesignation::Viewport;
        designation.commit(cancel)
    }

    fn focus_removal_window(
        &self,
        document: Key,
        document_id: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<Arc<DocumentAssociation>, DomError> {
        self.source_step(cancel)?;
        let environment = self.document_metadata(document)?.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let DocumentPhase::InitialEntry {
            entry, navigable, ..
        } = association.phase
        else {
            return Err(DomError::HostedSourceUnavailable);
        };
        let graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let step = || self.source_step(cancel).map_err(DomHostedError::from);
        let current = graph
            .navigable_with(navigable, step)
            .map_err(association_error)?;
        self.source_step(cancel)?;
        hosted::history::validate_navigable_document_identity(association, current)
            .map_err(association_error)?;
        if current.active != entry
            || graph
                .entry_with(entry, step)
                .map_err(association_error)?
                .document
                != Some(document_id)
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        if association.window_navigation.is_none() {
            return Err(DomError::HostedSourceUnavailable);
        }
        Ok(association.clone())
    }

    pub(in crate::internal::store) fn finish_focus_removals(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<Pending>, DomError> {
        let Some(pending) = self.focus_removals.as_ref().map(custody::Held::value) else {
            return Ok(None);
        };
        let Some(mut output) = self.prepare_output_buffer(&Vec::new(), pending.len(), cancel)?
        else {
            return Ok(None);
        };
        for binding in pending {
            self.source_step(cancel)?;
            output.value_mut().push(binding.clone());
        }
        Ok(Some(output))
    }
}

fn association_error(error: DomHostedError) -> DomError {
    match error {
        DomHostedError::Dom(error) => error,
        _ => DomError::HostedSourceUnavailable,
    }
}
