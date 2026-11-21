use super::*;
use surgeist_bindings_js::FunctionMetadata;

pub(super) const fn capture_bytes() -> usize {
    size_of::<(Weak<NativeOwner>, DomHostedRuntimeRetention)>()
}

pub(super) fn install(
    scope: &mut Scope<'_>,
    owner: Weak<NativeOwner>,
    backing: DomHostedRuntimeRetention,
) -> Result<Root, Error> {
    let capture = (owner, backing);
    let callback = Callback::new(move |scope, _| {
        let (owner, _backing) = &capture;
        let owner = owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        let attachment = owner.attachment.get().ok_or(Error::ExpiredIdentity)?;
        if scope.realm_authority()?.identity() != attachment.realm.identity() {
            return refuse(
                scope,
                &attachment.failures,
                DocumentScriptNativeError::ForeignPermit,
            );
        }
        let result = (|| {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            if owner.observe(attachment, &mut session)?.status() != HtmlScriptingStatus::Enabled {
                return Err(DocumentScriptNativeError::Hosted(
                    DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled),
                ));
            }
            session.run_media_microtask(&owner.cancellation)?;
            Ok(())
        })();
        match result {
            Ok(()) => Ok(Completion::Undefined),
            Err(error) => refuse(scope, &attachment.failures, error),
        }
    });
    let value = scope.fixed_arity_function(
        &callback,
        FunctionMetadata {
            name: &[],
            length: 0,
        },
    )?;
    Ok(scope.retain(&value))
}

impl NativeOwner {
    pub(super) fn schedule_media_removals(&self, scope: &mut Scope<'_>) -> Result<(), Error> {
        let attachment = self.attachment.get().ok_or(Error::ExpiredIdentity)?;
        let mut session = match self.session.try_borrow_mut() {
            Ok(session) => session,
            Err(_) => return refuse(scope, &attachment.failures, DocumentScriptNativeError::Busy),
        };
        let Some(mut next) = media_admission(scope, attachment, session.next_media_microtask())?
        else {
            return Ok(());
        };
        let job = self.media_removal_job.borrow();
        let job = scope.import(job.as_ref().ok_or(Error::ExpiredIdentity)?)?;
        loop {
            // This uses the already retained function and runs no guest code.
            scope.enqueue_microtask(&job)?;
            media_admission(scope, attachment, session.scheduled_media_microtask(next))?;
            let Some(following) =
                media_admission(scope, attachment, session.next_media_microtask())?
            else {
                return Ok(());
            };
            next = following;
        }
    }

    pub(super) fn discard_media_removals(&self) {
        if let Ok(mut session) = self.session.try_borrow_mut() {
            // The Session holds the matching original scheduler and DOM.
            let _ = session.discard_media_microtasks();
            self.media_removal_job.replace(None);
        }
    }
}

fn media_admission<T>(
    scope: &mut Scope<'_>,
    attachment: &Attachment,
    result: Result<T, DomError>,
) -> Result<T, Error> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => refuse(
            scope,
            &attachment.failures,
            DocumentScriptNativeError::Dom(error),
        ),
    }
}

impl DocumentScriptHost {
    pub(super) fn schedule_native_media_removals(&mut self) -> Result<(), DocumentScriptError> {
        let needs_scheduling = self
            .native
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .next_media_microtask()
            .map_err(DocumentScriptNativeError::from)?
            .is_some();
        if !needs_scheduling {
            return Ok(());
        }
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        engine
            .process
            .execution()
            .in_realm(&engine.realm, |scope| {
                self.native.schedule_media_removals(scope)
            })?;
        Ok(())
    }
}
