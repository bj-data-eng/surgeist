//! Actual mutable-root withdrawal at the original destruction cleanup boundary.
use super::*;
use crate::style::DomStyleCssomRoot;
use std::sync::MutexGuard;
use surgeist_cssom::{CssomSnapshot, CssomStore};

/// Immutable evidence of actual canonical custody withdrawal, not completion of
/// the hosted destruction or authority over a current mutable producer.
#[derive(Debug)]
pub struct DomHostedStyleRootWithdrawal {
    root: NodeId,
    document: DocumentId,
    cleanup: DomHostedOperationId,
    stamp: crate::DomStamp,
    snapshot: CssomSnapshot,
    _retained: Retained,
}
impl DomHostedStyleRootWithdrawal {
    /// Exact root generation whose canonical mutable registration was removed.
    pub fn root(&self) -> NodeId {
        self.root
    }
    /// Actual Document of the original retained destruction request.
    pub fn document(&self) -> DocumentId {
        self.document
    }
    /// The same cleanup operation which must still be resumed normally.
    pub fn cleanup_operation(&self) -> DomHostedOperationId {
        self.cleanup
    }
    /// Canonical tree stamp inspected; custody withdrawal changes no tree state.
    pub fn stamp(&self) -> crate::DomStamp {
        self.stamp
    }
    /// Historical producer capture under the withdrawal lock. Independently
    /// escaped mutable capabilities may subsequently change that same producer.
    pub fn withdrawal_cssom_snapshot(&self) -> &CssomSnapshot {
        &self.snapshot
    }
}

/// A fully admitted bank candidate and actual producer lock. Drop publishes
/// nothing. This does not discharge consumer-owned task/resource obligations;
/// a composing owner must establish those before calling `publish`.
pub struct DomHostedStyleRootWithdrawalPreparation<'d, 'r> {
    dom: &'d mut Dom,
    _source: &'d DomHostedSource,
    guard: MutexGuard<'r, CssomStore>,
    candidate: style_custody::Bank,
    outcome: DomHostedStyleRootWithdrawal,
    _formation: source::formation::Formation,
    _frame: Retained,
}
impl std::fmt::Debug for DomHostedStyleRootWithdrawalPreparation<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHostedStyleRootWithdrawalPreparation")
            .field("root", &self.outcome.root)
            .field("cleanup", &self.outcome.cleanup)
            .finish_non_exhaustive()
    }
}

/// Canonical custody has been withdrawn. The actual producer lock, exclusive
/// DOM and issuer loans and displaced bank stay alive for the consumer's
/// infallible suffix.
/// `finish` unlocks and disposes displaced semantic/debit holders afterwards.
pub struct DomHostedStyleRootWithdrawalPublished<'d, 'r> {
    _dom: &'d mut Dom,
    _source: &'d DomHostedSource,
    guard: MutexGuard<'r, CssomStore>,
    retired: style_custody::Bank,
    outcome: DomHostedStyleRootWithdrawal,
    _formation: source::formation::Formation,
    _frame: Retained,
}
impl std::fmt::Debug for DomHostedStyleRootWithdrawalPublished<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHostedStyleRootWithdrawalPublished")
            .field("root", &self.outcome.root)
            .field("cleanup", &self.outcome.cleanup)
            .finish_non_exhaustive()
    }
}
impl<'d, 'r> DomHostedStyleRootWithdrawalPreparation<'d, 'r> {
    /// Actual empty producer capture protected by the retained lock.
    pub fn withdrawal_snapshot(&self) -> &CssomSnapshot {
        &self.outcome.snapshot
    }
    /// Publish only canonical custody withdrawal. A composing owner performs
    /// its preformed, infallible resource move before releasing this result.
    /// This neither advances nor acknowledges the original cleanup request.
    pub fn publish(self) -> DomHostedStyleRootWithdrawalPublished<'d, 'r> {
        let Self {
            dom,
            _source,
            guard,
            candidate,
            outcome,
            _formation,
            _frame,
        } = self;
        let retired = std::mem::replace(&mut dom.style_cssom_roots, candidate);
        DomHostedStyleRootWithdrawalPublished {
            _dom: dom,
            _source,
            guard,
            retired,
            outcome,
            _formation,
            _frame,
        }
    }
}
impl DomHostedStyleRootWithdrawalPublished<'_, '_> {
    /// End the consumer fence, unlock the actual producer, then dispose the
    /// displaced bank before its original preparation accounting is released.
    pub fn finish(self) -> DomHostedStyleRootWithdrawal {
        let Self {
            _dom: _,
            _source: _,
            guard,
            retired,
            outcome,
            _formation,
            _frame,
        } = self;
        drop(guard);
        drop(retired);
        drop(_formation);
        drop(_frame);
        outcome
    }
}

impl DomHostedDestructionLease {
    /// Prepare withdrawal of this exact idle, registered root at the original
    /// reached cleanup request. Only the same cleanup's hosted guard is allowed;
    /// unrelated Source/hosted work and all canonical cleanup guards remain.
    /// An asynchronous cleanup need not occupy the synchronous hosted guard:
    /// qualification uses its actual retained phase and sealed publication.
    ///
    /// The composing owner must separately prove its retained resource/task/
    /// publication bank no longer needs the mutable capability. DOM validates
    /// its own actual associations, producer emptiness and canonical obligations.
    /// The preparation also borrows this actual issuer until the fence ends,
    /// preventing issuer revision changes between qualification and publication.
    /// Refusal or dropping the preparation preserves both custody and the same
    /// cleanup/publication, including cumulative original work for retry.
    pub fn prepare_style_root_withdrawal<'d, 'r>(
        &'d self,
        dom: &'d mut Dom,
        source: &'d DomHostedSource,
        cleanup: &DomHostedDestructionPublication,
        root: &'r DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomHostedStyleRootWithdrawalPreparation<'d, 'r>, DomHostedError> {
        dom.validate_hosted_source(source)?;
        dom.require_sync_source_idle()?;
        if dom
            .state
            .incomplete_hosted
            .is_some_and(|id| id != self.operation)
            || dom
                .hosted_operations
                .synchronous()
                .is_some_and(|id| id != self.operation)
        {
            return Err(DomHostedError::IncompleteOperation);
        }
        let operation = dom.hosted_operation(self.operation)?;
        let CreationPhase::DestroyCleanup(request) = &operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        source.validate_style_root_withdrawal(request, cleanup, cancel)?;
        let work = &request.request.work;
        work.step(cancel)?;
        let document = request.document();
        let node = dom.state.node(dom.owner, root.root())?;
        if !matches!(
            node.kind(),
            DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
        ) || node.document() != document
        {
            return Err(DomError::StyleReceiptMismatch.into());
        }
        if !dom
            .style_cssom_roots
            .get(root.root())
            .is_some_and(|owner| owner.same_owner(root))
        {
            return Err(DomError::StyleReceiptMismatch.into());
        }
        dom.require_hosted_invocation_children(request.invocation, work, cancel)?;
        let guard = root
            .lock()
            .map_err(crate::DomStyleCssomRootError::into_dom_error)?;
        let frame_bytes = size_of::<DomHostedStyleRootWithdrawalPreparation<'_, '_>>()
            .checked_sub(size_of::<DomHostedStyleRootWithdrawal>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        crate::hosted::admit(
            frame_bytes + size_of::<DomHostedStyleRootWithdrawal>(),
            source.limits().scratch_bytes(),
            DomHostedResource::ScratchBytes,
        )?;
        work.step(cancel)?;
        let frame = source.retain(frame_bytes, cancel)?;
        let output = source.retain(size_of::<DomHostedStyleRootWithdrawal>(), cancel)?;
        work.step(cancel)?;
        let snapshot = guard.snapshot();
        if !snapshot.sheets().is_empty() {
            return Err(DomHostedError::PendingDocumentObligations);
        }
        let draft = dom.hosted_draft(operation, source, cancel)?;
        let formation = source::formation::Formation::new(&draft, cancel)?
            .ok_or(DomError::HostedSourceUnavailable)?;
        dom.state.inbound.visit_target(
            Key::Existing(document.node()),
            |owner| {
                draft.source_step(cancel)?;
                let Key::Existing(id) = owner else {
                    return Err(DomError::UnavailableCreation);
                };
                let node = dom.state.node(dom.owner, id)?;
                if node.document() == document
                    && let DomNodeKind::Element(element) = node.kind()
                    && let Some(style) = element.style_state()
                    && (style.availability
                        == crate::DomStyleSheetAvailability::PendingParsedProcessing
                        || style
                            .sheet
                            .as_ref()
                            .is_some_and(|sheet| sheet.root == root.root()))
                {
                    return Err(DomError::StyleProcessingUnavailable);
                }
                Ok(())
            },
            || draft.source_step(cancel),
        )?;
        let candidate = dom.style_cssom_roots.prepare_withdrawal(
            root,
            document,
            |usage| formation.admit_custody_usage(usage, cancel),
            || draft.source_step(cancel),
        )?;
        dom.require_hosted_cleanup_obligations(operation, document, &candidate, cancel)?;
        // Fixed result copying and the represented bank/frame disposal are
        // admitted before publication. Deep CSSOM storage remains producer-owned.
        work.step(cancel)?;
        work.step(cancel)?;
        let outcome = DomHostedStyleRootWithdrawal {
            root: root.root(),
            document,
            cleanup: self.operation,
            stamp: crate::DomStamp {
                owner: dom.owner,
                revision: dom.revision,
            },
            snapshot,
            _retained: output,
        };
        Ok(DomHostedStyleRootWithdrawalPreparation {
            dom,
            _source: source,
            guard,
            candidate,
            outcome,
            _formation: formation,
            _frame: frame,
        })
    }
}

#[cfg(test)]
mod tests;
