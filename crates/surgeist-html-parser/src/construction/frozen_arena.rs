//! The frozen arena a completed session leaves behind.

use super::*;
use surgeist_html::{
    construction::HtmlConstructionResult,
    construction::{HtmlDocument, HtmlFragment},
};

pub(crate) struct FrozenArena {
    pub(super) storage: Option<FrozenStorage>,
}

pub(super) struct FrozenStorage {
    pub(super) view: HtmlConstructionResult,
    pub(super) control: ControlIdentity,
    pub(super) holds: StorageHolds,
    pub(super) lease: Option<surgeist_html::bounded::construction::HtmlConstructionLease>,
    pub(super) payloads: PayloadStorage,
    pub(super) effects: ConstructionEffects,
    /// Present until publication moves the evidence into the published history
    /// beside this storage, where only its `PublishedEvidence` form is read.
    pub(super) evidence: Option<ParseEvidence>,
    pub(super) fragment: Option<FragmentContextStorage>,
}

impl FrozenArena {
    /// Polls the controller before a frozen arena is published.
    pub(crate) fn require_original_transfer(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        c.checkpoint()?;
        Ok(())
    }
    /// Fixed lender of the real committed store, not a report projection.
    pub(crate) fn retained_evidence(&self) -> Result<&ParseEvidence, SessionError> {
        self.storage
            .as_ref()
            .ok_or(SessionError::Consumed)?
            .evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)
    }
    /// Partition the stopped frozen store through its original evidence issuer.
    pub(crate) fn take_stopped_evidence(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<evidence::FailureDiagnosticEvidence>, SessionError> {
        let storage = self.storage.as_mut().ok_or(SessionError::Consumed)?;
        control.cleanup_release(&storage.control, HtmlParseResource::AuxiliaryEntries, 0)?;
        ParseEvidence::partition_failed(&mut storage.evidence, control)
    }
    pub(crate) fn document(&self) -> Option<&HtmlDocument> {
        match &self.storage.as_ref()?.view {
            HtmlConstructionResult::Document(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn fragment(&self) -> Option<&HtmlFragment> {
        match &self.storage.as_ref()?.view {
            HtmlConstructionResult::Fragment(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        cleanup_owner(control, &storage.control)?;
        // A missing lease is refused before anything is taken, so the owner
        // stays where it was.
        if storage.lease.is_none() {
            return Err(latch_registry_row(control));
        }
        let Some(FrozenStorage {
            view,
            control: owner,
            holds,
            mut lease,
            payloads,
            effects,
            evidence,
            fragment,
        }) = self.storage.take()
        else {
            return Err(SessionError::Consumed);
        };
        let Some(held_lease) = lease.take() else {
            // The lease was checked above; a missing one is an impossible
            // custody state, so the owner stays where it was.
            self.storage = Some(FrozenStorage {
                view,
                control: owner,
                holds,
                lease: None,
                payloads,
                effects,
                evidence,
                fragment,
            });
            return Err(latch_registry_row(control));
        };
        let result = match view {
            HtmlConstructionResult::Document(value) => {
                surgeist_html::construction::HtmlConstructionResult::Document(value)
            }
            HtmlConstructionResult::Fragment(value) => {
                surgeist_html::construction::HtmlConstructionResult::Fragment(value)
            }
        };
        let recovery =
            match surgeist_html::bounded::construction::HtmlConstructionLease::try_recover(
                result, held_lease,
            ) {
                Ok(recovery) => recovery,
                Err(refusal) => {
                    // A refused recovery hands the result and the lease back.
                    let ((result, held_lease), error) = refusal.into_parts();
                    let view = match result {
                        surgeist_html::construction::HtmlConstructionResult::Document(value) => {
                            HtmlConstructionResult::Document(value)
                        }
                        surgeist_html::construction::HtmlConstructionResult::Fragment(value) => {
                            HtmlConstructionResult::Fragment(value)
                        }
                    };
                    self.storage = Some(FrozenStorage {
                        view,
                        control: owner,
                        holds,
                        lease: Some(held_lease),
                        payloads,
                        effects,
                        evidence,
                        fragment,
                    });
                    return Err(construction_error(control, error));
                }
            };
        // Every owner is released even after a refusal; the first one is
        // returned at the end. The recovery put the storage back before it
        // could refuse, so the owner below can still be retired.
        let mut refused = None;
        let (records, _) = recovery.into_parts();
        #[cfg(test)]
        if control.take_construction_recovery_fault() {
            keep_first(&mut refused, Err(latch_registry_row(control)));
        }
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        if let Some(fragment) = fragment {
            let released = fragment.release(control, &owner);
            #[cfg(test)]
            inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
            keep_first(&mut refused, released);
        }
        if let Some(evidence) = evidence {
            let released = evidence.release(control);
            #[cfg(test)]
            inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
            keep_first(&mut refused, released);
        }
        let released = effects.release(control);
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, released);
        let released = release_parts(
            control,
            &owner,
            ReleasedStorage {
                construction: records,
                holds,
                payloads,
            },
        );
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, released);
        refused.map_or(Ok(()), Err)
    }
}
