//! Authored processing instructions over the canonical detached-node producer.

use super::*;
use surgeist_bindings_js::{
    DomString,
    conversion::{Argument, Converted, Type, declarations},
    dom_exception::DomExceptionName,
};
use surgeist_dom::DomDetachedNodeCreation;

pub(super) fn constructor(
    capture: CaptureFunding<'_>,
) -> Result<ConstructorCallback, DocumentGuestEventError> {
    let fallback = capture(size_of::<Capture>())?;
    let construct = capture(size_of::<Capture>())?;
    Ok(ConstructorCallback::bounded(
        2,
        move |scope| fallback.instruction_prototype(scope),
        move |scope, arguments| construct.construct_instruction(scope, arguments),
    ))
}

pub(super) fn install_factory<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let factory = capture(size_of::<Capture>())?;
    let name = &ascii(b"createProcessingInstruction");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_instruction(scope, arguments)),
        FunctionMetadata { name, length: 2 },
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

type InstructionArgument = Converted<ArgumentValue<DomString>>;

impl Capture {
    fn instruction_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, Error> {
        let events = self
            .callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let result = (|| {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let realm = scope.realm_authority()?.identity();
            if realm == self.callbacks.permit.realm {
                return Ok(scope.open_callback_object(
                    &public
                        .document
                        .prototypes
                        .character_data
                        .processing_instruction,
                )?);
            }
            let origins = public
                .origins
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut next = origins.as_deref();
            while let Some(origin) = next {
                events.step()?;
                if origin.target.associated_realm().identity() == realm {
                    return Ok(scope.open_callback_object(
                        &origin.documents.character_data.processing_instruction,
                    )?);
                }
                next = origin.next.as_deref();
            }
            Err(DocumentGuestEventError::CallbackRealmUnavailable)
        })();
        match result {
            Ok(value) => Ok(value),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn construct_instruction<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        if arguments.is_empty() {
            return Ok(ConstructorCompletion::Throw(
                scope.type_error(&ascii(b"Missing processing instruction target"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let args = self.instruction_arguments(
                scope,
                declarations::processing_instruction_constructor_0_target(),
                declarations::processing_instruction_constructor_0_data(),
                |index| arguments.get(index),
            )?;
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let event = owner.active_event_request();
            let document = self.callbacks.with_qualified_session(
                &owner,
                event.as_ref(),
                |session, attached| {
                    let global = self
                        .origin
                        .as_ref()
                        .map_or(attached, |origin| origin.global);
                    session.dom().raw().dispatch_global_document_with(
                        global,
                        &owner.cancellation,
                        &mut || events.step(),
                    )
                },
            )?;
            self.publish_instruction(
                scope,
                (&owner, &events),
                ordinary::NodePrototype::Selected(selected.prototype()),
                document,
                args,
            )
        })();
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(error) if invalid_character(&error) => Ok(ConstructorCompletion::Throw(
                scope.dom_exception(DomExceptionName::InvalidCharacterError, &[])?,
            )),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn create_instruction<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        if arguments.len() < 2 {
            return Ok(Completion::Throw(scope.type_error(&ascii(
                b"Missing processing instruction arguments",
            ))?));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let args = self.instruction_arguments(
                scope,
                declarations::document_create_processing_instruction_0_target(),
                declarations::document_create_processing_instruction_0_data(),
                |index| arguments.get(index),
            )?;
            scope.check_active()?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let create = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
                self.publish_instruction(
                    scope,
                    (&owner, &events),
                    ordinary::NodePrototype::Creation(
                        &prototypes.character_data.processing_instruction,
                    ),
                    document,
                    args,
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| create(scope, &origin.documents))
            } else {
                create(scope, &public.document.prototypes)
            }
        })();
        match result {
            Err(error) if invalid_character(&error) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidCharacterError, &[])?,
            )),
            result => complete(scope, &events, result),
        }
    }

    fn instruction_arguments<'args, 'scope: 'args>(
        &self,
        scope: &mut Scope<'scope>,
        target: Argument<Type<DomString>>,
        data: Argument<Type<DomString>>,
        get: impl Fn(usize) -> Option<&'args Value<'scope>>,
    ) -> Result<(InstructionArgument, InstructionArgument), Error> {
        self.callbacks
            .conversion
            .run(scope, |conversion| {
                Ok((
                    conversion.argument(&target, get(target.index()))?,
                    conversion.argument(&data, get(data.index()))?,
                ))
            })
            .map_err(Into::into)
    }

    fn publish_instruction<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: ordinary::NodePrototype<'_, 'scope>,
        document: DocumentId,
        (target, data): (InstructionArgument, InstructionArgument),
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let (ArgumentValue::Present(target), ArgumentValue::Present(data)) =
            (target.get(), data.get())
        else {
            return Err(Error::BackendUnavailable.into());
        };
        let (target, _target_backing) = copy_native_data(owner, events, target.as_code_units())?;
        let (data, _data_backing) = copy_native_data(owner, events, data.as_code_units())?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        public.document.ordinary.create_detached(
            scope,
            self,
            (owner, events),
            prototype,
            (
                document,
                DomDetachedNodeCreation::ProcessingInstruction {
                    target,
                    data,
                    fallback: creation_realm_fallback(owner)?,
                },
            ),
        )
    }
}

fn invalid_character(error: &DocumentGuestEventError) -> bool {
    matches!(
        error,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Dom(DomError::InvalidCharacter)
        ))
    )
}
