//! Exact active Source authority shared by scoped DOM reads.

use super::*;
use crate::internal::{
    hosted::{DocumentAssociation, DomHostedError, ScriptRegistration},
    source::{
        callback::{CallbackWork, DispatchScope},
        event::DomSourceEventRequest,
    },
};
use std::{cell::Cell, sync::Weak};

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

/// One native child read's local allowance and exact active Source scope.
pub(super) struct HostedReadBudget<'dom> {
    source: Option<ReadSource<'dom>>,
    remaining: Cell<u64>,
    limit: u64,
    cancel: &'dom AtomicBool,
}

impl<'dom> HostedReadBudget<'dom> {
    pub(super) fn new(
        dom: &'dom Dom,
        limit: u64,
        cancel: &'dom AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(Self {
            source: dom.qualify_read_source(cancel)?,
            remaining: Cell::new(limit),
            limit,
            cancel,
        })
    }

    pub(super) fn validate_scope(&self) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        if let Some(source) = &self.source {
            source.validate()?;
        }
        Ok(())
    }

    pub(super) fn step<E>(
        &self,
        admission: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), HostedReadError<E>> {
        self.validate_scope()?;
        let remaining = self
            .remaining
            .get()
            .checked_sub(1)
            .ok_or(DomHostedError::WorkLimit { limit: self.limit })?;
        self.remaining.set(remaining);
        admission().map_err(HostedReadError::Caller)?;
        match &self.source {
            Some(source) => source.step(self.cancel)?,
            None => check_cancel(self.cancel)?,
        }
        Ok(())
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
