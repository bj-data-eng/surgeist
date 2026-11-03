//! The frozen arena a completed session leaves behind.

use super::*;

pub(crate) struct FrozenArena {
    pub(super) storage: Option<FrozenStorage>,
    pub(super) finalizing: Option<value_sharing::Finalizing>,
    pub(super) sharing_started: bool,
}

pub(super) struct FrozenStorage {
    pub(super) view: HtmlConstructionResult,
    pub(super) control: ControlIdentity,
    pub(super) construction: ConstructionOwner,
    pub(super) lease: Option<bounded::HtmlConstructionLease>,
    pub(super) payloads: PayloadStorage,
    pub(super) effects: ConstructionEffects,
    /// Present until publication moves the evidence into the published history
    /// beside this storage, where only its `PublishedEvidence` form is read.
    pub(super) evidence: Option<ParseEvidence>,
    pub(super) fragment: Option<FragmentContextStorage>,
}

impl FrozenArena {
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
        if let Some(finalizing) = &mut self.finalizing {
            return finalizing.take_stopped_evidence(control);
        }
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
        if let Some(finalizing) = &self.finalizing {
            finalizing.require_cleanup(control)?;
        }
        if let Some(finalizing) = self.finalizing.take() {
            return finalizing.release(control);
        }
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        cleanup_owner(control, &storage.control)?;
        storage
            .payloads
            .check_aliases(ConstructionView::Frozen(&storage.view), control)?;
        // A missing lease is refused before anything is taken, so the owner
        // stays where it was.
        let Some(publication) = storage.lease.as_ref() else {
            return Err(latch_registry_row(control));
        };
        storage
            .construction
            .require_recovery(control, publication)?;
        let Some(FrozenStorage {
            view,
            control: owner,
            mut construction,
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
                construction,
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
                bounded::HtmlConstructionResult::Document(value)
            }
            HtmlConstructionResult::Fragment(value) => {
                bounded::HtmlConstructionResult::Fragment(value)
            }
        };
        let recovery = match bounded::HtmlConstructionLease::try_recover(result, held_lease) {
            Ok(recovery) => recovery,
            Err(refusal) => {
                // A refused recovery hands the result and the lease back.
                let ((result, held_lease), error) = refusal.into_parts();
                let view = match result {
                    bounded::HtmlConstructionResult::Document(value) => {
                        HtmlConstructionResult::Document(value)
                    }
                    bounded::HtmlConstructionResult::Fragment(value) => {
                        HtmlConstructionResult::Fragment(value)
                    }
                };
                self.storage = Some(FrozenStorage {
                    view,
                    control: owner,
                    construction,
                    lease: Some(held_lease),
                    payloads,
                    effects,
                    evidence,
                    fragment,
                });
                return Err(construction_owner::construction_error(control, error));
            }
        };
        // Every owner is released even after a refusal; the first one is
        // returned at the end. The recovery put the storage back before it
        // could refuse, so the owner below can still be retired.
        let mut refused = None;
        let recovered = construction.recovered(control, recovery);
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, recovered);
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
                construction,
                payloads,
            },
        );
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::FrozenOwner, &mut refused);
        keep_first(&mut refused, released);
        refused.map_or(Ok(()), Err)
    }
}
