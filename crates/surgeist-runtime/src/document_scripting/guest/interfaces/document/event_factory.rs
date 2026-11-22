//! The legacy Document factory over the original uninitialized native Event.

use super::*;
use surgeist_bindings_js::{
    conversion::declarations::document_create_event_0_interface, dom_exception::DomExceptionName,
};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let factory = capture(size_of::<Capture>())?;
    let name = &ascii(b"createEvent");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_legacy_event(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn create_legacy_event<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let projection = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Document(document)) = projection.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let interface = document_create_event_0_interface();
            let interface = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&interface, arguments.get(interface.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(interface) = interface.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            events.step()?;
            let request = owner.attached_event_request();
            let (native, attachment_global) = self.callbacks.with_qualified_session(
                &owner,
                request.as_ref(),
                |session, attachment_global| {
                    let event = session
                        .hosted_execution()
                        .create_legacy_event(
                            &mut session.dom(),
                            document,
                            interface.as_code_units(),
                            &owner.cancellation,
                        )
                        .map_err(DocumentGuestEventError::Construction)?;
                    Ok((event, attachment_global))
                },
            )?;
            let global = native.creation().ok_or(Error::RegistryConflict)?.global();
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            if global == attachment_global {
                public
                    .event
                    .project_legacy(scope, (&owner, &events), native)
            } else {
                public.with_origin(&events, global, |origin| {
                    origin
                        .event
                        .project_legacy(scope, (&owner, &events), native)
                })
            }
        })();
        match result {
            Err(DocumentGuestEventError::Construction(
                DomEventConstructionError::UnsupportedInterface,
            )) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
            )),
            result => complete(scope, &events, result),
        }
    }
}
