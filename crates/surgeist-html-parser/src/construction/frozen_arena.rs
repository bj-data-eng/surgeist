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
        // A result another owner holds cannot be released: the refusal is raised before anything
        // is taken, so the owner stays where it was and the release can be retried once the other
        // owner is gone.
        if storage.view.is_shared() {
            return Err(construction_error(
                control,
                surgeist_html::construction::HtmlConstructionCause::SharedAllocationAliased.into(),
            ));
        }
        let Some(FrozenStorage {
            view,
            control: owner,
            holds,
            payloads,
            effects,
            evidence,
            fragment,
        }) = self.storage.take()
        else {
            return Err(SessionError::Consumed);
        };
        // The usage the published records hold is read here, and the result is dropped with the
        // other owners of the records before the figures the parse holds for them are released.
        let usage = view.usage();
        // Every owner is released even after a refusal; the first one is
        // returned at the end.
        let mut refused = None;
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
        drop(view);
        let released = release_parts(
            control,
            &owner,
            ReleasedStorage {
                usage,
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
