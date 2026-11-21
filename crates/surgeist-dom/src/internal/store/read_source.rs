//! Exact active Source authority shared by scoped DOM reads.

use super::*;
use crate::internal::{
    hosted::{DocumentAssociation, DomHostedError, ScriptRegistration},
    source::{
        callback::{CallbackWork, DispatchScope},
        event::DomSourceEventRequest,
    },
};
use std::sync::Weak;

pub(super) struct ReadSource<'dom> {
    work: Arc<CallbackWork>,
    scope: Weak<DispatchScope>,
    authority: ReadAuthority<'dom>,
}

pub(super) enum ReadAuthority<'dom> {
    Event(&'dom DomSourceEventRequest),
    Script {
        registration: &'dom ScriptRegistration,
        association: Arc<DocumentAssociation>,
        attribute: bool,
    },
}

impl<'dom> ReadSource<'dom> {
    pub(super) fn new(
        work: Arc<CallbackWork>,
        authority: ReadAuthority<'dom>,
    ) -> Result<Self, DomError> {
        let scope = work.active_scope().ok_or(DomError::InvalidSourceEvent)?;
        let qualified = Self {
            work,
            scope,
            authority,
        };
        qualified.validate()?;
        Ok(qualified)
    }

    pub(super) fn validate(&self) -> Result<(), DomError> {
        if !self
            .work
            .active_scope()
            .is_some_and(|active| active.ptr_eq(&self.scope))
        {
            return Err(DomError::InvalidSourceEvent);
        }
        match &self.authority {
            ReadAuthority::Event(request) => {
                let executor = request
                    .executor
                    .upgrade()
                    .ok_or(DomError::InvalidSourceEvent)?;
                request.validate_executor(&executor)
            }
            ReadAuthority::Script {
                registration,
                association,
                attribute,
            } => {
                if !registration.is_active() {
                    return Err(if *attribute {
                        DomError::SourceAttributeExecutorUnavailable
                    } else {
                        DomError::HostedSourceUnavailable
                    });
                }
                association
                    .validate_live()
                    .map(|_| ())
                    .map_err(|_| DomError::HostedSourceUnavailable)
            }
        }
    }

    pub(super) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.validate()?;
        self.work.step(cancel)
    }
}

// Storage and graph APIs retain their own failures, and caller errors bypass
// both conversions until the public read returns.
pub(in crate::internal::store) enum HostedReadError<E> {
    Hosted(DomHostedError),
    Caller(E),
}

impl<E> From<DomHostedError> for HostedReadError<E> {
    fn from(error: DomHostedError) -> Self {
        Self::Hosted(error)
    }
}

impl<E> From<DomError> for HostedReadError<E> {
    fn from(error: DomError) -> Self {
        Self::Hosted(error.into())
    }
}

impl<E: From<DomHostedError>> HostedReadError<E> {
    pub(in crate::internal::store) fn into_caller(self) -> E {
        match self {
            Self::Hosted(error) => error.into(),
            Self::Caller(error) => error,
        }
    }
}
