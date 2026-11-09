//! Sealed reaction custody facts and the actual continuation lifecycle they bind.

use crate::{
    JsError, JsResult,
    job::{JobFault, JobFaultCause, JobFaultOwner},
    module::lifecycle::{ModuleAction, ModuleActionHold, ModuleActionRoot},
    vm::opcode::{AwaitPhase, AwaitRef, AwaitRoot, PreparedAwaitRef},
};
use surgeist_js_gc::heap::{ObjectId, Trace};

/// An owning registration observation; only Promise's real commit paths create it.
#[must_use]
pub(crate) struct ReactionRegistration {
    state: RegistrationState,
}
enum RegistrationState {
    Uncommitted(JsError),
    Committed(CommittedReactions),
}
/// Actual custody at a completed Promise registration boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReactionCustody {
    SourcePending,
    ExecutorAccepted,
    PolicyDiscarded,
}
/// A committed observation retains its first completion cause independently.
pub(crate) struct CommittedReactions {
    custody: ReactionCustody,
    completion: JsResult<()>,
    await_identity: Option<ObjectId>,
}
pub(crate) enum RegistrationOutcome {
    Uncommitted(JsError),
    Committed(CommittedReactions),
}
impl ReactionRegistration {
    pub(super) fn uncommitted(error: JsError) -> Self {
        Self {
            state: RegistrationState::Uncommitted(error),
        }
    }
    pub(super) fn committed(
        custody: ReactionCustody,
        completion: JsResult<()>,
        hold: &PublicationHold,
    ) -> Self {
        let mut committed = CommittedReactions {
            custody,
            completion,
            await_identity: hold.await_root().map(|root| root.reference().identity()),
        };
        if let Some(root) = hold.await_root() {
            let qualified = root.qualify(&committed);
            merge(&mut committed.completion, qualified);
        }
        Self {
            state: RegistrationState::Committed(committed),
        }
    }
    pub(super) fn is_committed(&self) -> bool {
        matches!(self.state, RegistrationState::Committed(_))
    }
    pub(super) fn complete(mut self, result: JsResult<()>) -> Self {
        match &mut self.state {
            RegistrationState::Committed(committed) => merge(&mut committed.completion, result),
            RegistrationState::Uncommitted(_) => {
                let mut cleanup = crate::job::UnwindCleanup::default();
                cleanup.run(|| drop(result));
                return cleanup.finish_with(self);
            }
        }
        self
    }
    pub(crate) fn into_outcome(self) -> RegistrationOutcome {
        match self.state {
            RegistrationState::Uncommitted(error) => RegistrationOutcome::Uncommitted(error),
            RegistrationState::Committed(committed) => RegistrationOutcome::Committed(committed),
        }
    }
    pub(crate) fn into_result(self) -> JsResult<()> {
        match self.into_outcome() {
            RegistrationOutcome::Uncommitted(error) => Err(error),
            RegistrationOutcome::Committed(committed) => committed.into_result(),
        }
    }
}
impl CommittedReactions {
    pub(crate) fn custody(&self) -> ReactionCustody {
        match self.custody {
            ReactionCustody::SourcePending => ReactionCustody::SourcePending,
            ReactionCustody::ExecutorAccepted => ReactionCustody::ExecutorAccepted,
            ReactionCustody::PolicyDiscarded => ReactionCustody::PolicyDiscarded,
        }
    }
    pub(crate) fn await_identity(&self) -> Option<ObjectId> {
        self.await_identity
    }
    pub(crate) fn into_result(self) -> JsResult<()> {
        self.completion
    }
}
fn merge(first: &mut JsResult<()>, later: JsResult<()>) {
    if first.is_ok() {
        *first = later;
    } else {
        let retained = std::mem::replace(first, Ok(()));
        let mut cleanup = crate::job::UnwindCleanup::default();
        cleanup.run(|| drop(later));
        *first = cleanup.finish_with(retained);
    }
}

#[derive(Clone, Trace)]
pub(crate) struct ReactionLifecycle {
    kind: LifecycleKind,
}
#[derive(Clone, Trace)]
enum LifecycleKind {
    Untracked,
    OrdinaryAwait(AwaitRef),
    Module(ModuleAction),
}
impl ReactionLifecycle {
    pub(crate) fn untracked() -> Self {
        Self {
            kind: LifecycleKind::Untracked,
        }
    }
    pub(crate) fn for_await(
        prepared: &PreparedAwaitRef,
        action: Option<ModuleAction>,
    ) -> JsResult<Self> {
        let state = prepared.reference();
        let kind = match action {
            Some(action) => {
                if !action.await_ref().is_some_and(|actual| actual.same(state)) {
                    return Err(
                        JobFault::new(JobFaultOwner::Task, JobFaultCause::OwnerMismatch).into(),
                    );
                }
                LifecycleKind::Module(action)
            }
            None => LifecycleKind::OrdinaryAwait(state.clone()),
        };
        Ok(Self { kind })
    }
    pub(crate) fn for_completion(action: ModuleAction) -> JsResult<Self> {
        if action.await_ref().is_some() {
            return Err(JobFault::new(
                JobFaultOwner::ModuleEvaluation,
                JobFaultCause::StateInconsistency,
            )
            .into());
        }
        Ok(Self {
            kind: LifecycleKind::Module(action),
        })
    }
    fn await_ref(&self) -> Option<&AwaitRef> {
        match &self.kind {
            LifecycleKind::Untracked => None,
            LifecycleKind::OrdinaryAwait(state) => Some(state),
            LifecycleKind::Module(action) => action.await_ref(),
        }
    }
    pub(super) fn hold(&self) -> JsResult<PublicationHold> {
        match &self.kind {
            LifecycleKind::Untracked => Ok(PublicationHold::Untracked),
            LifecycleKind::OrdinaryAwait(state) => state.root().map(PublicationHold::Ordinary),
            LifecycleKind::Module(action) => action.hold().map(PublicationHold::Module),
        }
    }
    pub(super) fn unavailable(&self) -> Result<(), JobFault> {
        match self.await_ref() {
            Some(state) => state.preparation_failed(),
            None => Ok(()),
        }
    }
    pub(super) fn preparation_failed(&self, error: &JsError) -> Result<(), JobFault> {
        if let Some(state) = self.await_ref() {
            state.preparation_failed()?;
        }
        if let LifecycleKind::Module(action) = &self.kind {
            action.preparation_failed(error)?;
        }
        Ok(())
    }
}

/// Root leases only. Dropping this value never changes continuation availability.
#[derive(Clone)]
pub(super) enum PublicationHold {
    Untracked,
    Ordinary(AwaitRoot),
    Module(ModuleActionHold),
}
impl PublicationHold {
    fn await_root(&self) -> Option<&AwaitRoot> {
        match self {
            Self::Untracked => None,
            Self::Ordinary(root) => Some(root),
            Self::Module(hold) => hold.await_root(),
        }
    }
    pub(super) fn job_owner(&self) -> ReactionJobLifecycle {
        let kind = match self {
            Self::Untracked => JobLifecycleKind::Untracked,
            Self::Ordinary(root) => JobLifecycleKind::Ordinary(root.clone()),
            Self::Module(hold) => JobLifecycleKind::Module(hold.job_owner()),
        };
        ReactionJobLifecycle { kind }
    }
    pub(super) fn refused(&self, error: &JsError) -> JsResult<()> {
        let mut result = Ok(());
        if let Some(root) = self.await_root() {
            match root.phase() {
                AwaitPhase::Prepared => merge(&mut result, root.uncommitted()),
                AwaitPhase::SourcePending | AwaitPhase::ExecutorAccepted => {
                    merge(&mut result, root.unavailable());
                }
                _ => {}
            }
        }
        if let Self::Module(hold) = self {
            merge(
                &mut result,
                hold.preparation_failed(error).map_err(Into::into),
            );
        }
        result
    }
    pub(super) fn unavailable(&self) -> Result<(), JobFault> {
        match self.await_root() {
            Some(root) => root.reference().preparation_failed(),
            None => Ok(()),
        }
    }
    pub(super) fn preparation_failed(&self, error: &JsError) -> Result<(), JobFault> {
        if let Some(root) = self.await_root() {
            root.reference().preparation_failed()?;
        }
        if let Self::Module(hold) = self {
            hold.preparation_failed(error)?;
        }
        Ok(())
    }
}

/// The unique job owner. Only InvocationCustody applies its disposal event.
pub(crate) struct ReactionJobLifecycle {
    kind: JobLifecycleKind,
}
enum JobLifecycleKind {
    Untracked,
    Ordinary(AwaitRoot),
    Module(ModuleActionRoot),
}
impl ReactionJobLifecycle {
    pub(crate) fn untracked() -> Self {
        Self {
            kind: JobLifecycleKind::Untracked,
        }
    }
    pub(crate) fn is_module(&self) -> bool {
        matches!(self.kind, JobLifecycleKind::Module(_))
    }
    pub(crate) fn check(&self) -> JsResult<()> {
        match &self.kind {
            JobLifecycleKind::Module(action) => action.check(),
            _ => Ok(()),
        }
    }
    pub(crate) fn finish(&self, error: &JsError) -> Result<(), JobFault> {
        match &self.kind {
            JobLifecycleKind::Module(action) => action.finish(error),
            _ => Ok(()),
        }
    }
    pub(crate) fn dispose(&self, entered: bool) -> Result<(), JobFault> {
        match &self.kind {
            JobLifecycleKind::Untracked => Ok(()),
            JobLifecycleKind::Ordinary(root) => root.dispose(entered),
            JobLifecycleKind::Module(action) => match action.await_root() {
                Some(root) => root.dispose(entered),
                None => Ok(()),
            },
        }
    }
}
