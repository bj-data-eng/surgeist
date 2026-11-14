//! Canonical ordinary DocumentFragment creation and projection custody.

use super::*;
use surgeist_dom::DomDetachedNodeCreation;

pub(super) const FRAGMENT: &[u16] = &ascii(b"DocumentFragment");
const CREATE_FRAGMENT: &[u16] = &ascii(b"createDocumentFragment");

pub(super) fn constructor(
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<ConstructorCallback, DocumentGuestEventError> {
    let fallback = capture(size_of::<Capture>())?;
    let constructor = constructor_capture(capture, size_of::<Capture>())?;
    Ok(ConstructorCallback::bounded(
        0,
        move |scope| {
            let (_, events) = fallback.active(scope)?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            scope.open_callback_object(&public.document.prototypes.fragment)
        },
        move |scope, arguments| constructor.construct_fragment(scope, arguments),
    ))
}

pub(super) fn install_factory<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    let factory = capture(size_of::<Capture>())?;
    let function = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_fragment(scope, arguments)),
        FunctionMetadata {
            name: CREATE_FRAGMENT,
            length: 0,
        },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(CREATE_FRAGMENT),
        &function,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn construct_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let attachment = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?;
            self.publish_fragment(
                scope,
                &owner,
                &events,
                attachment.target.document(),
                ordinary::NodePrototype::Selected(selected.prototype()),
            )
        })();
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn create_fragment<'scope>(
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
        let result = (|| {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let mut project = |prototypes: &DocumentPrototypes| {
                self.publish_fragment(
                    scope,
                    &owner,
                    &events,
                    document,
                    ordinary::NodePrototype::Creation(&prototypes.fragment),
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| project(&origin.documents))
            } else {
                project(&public.document.prototypes)
            }
        })();
        complete(scope, &events, result)
    }

    fn publish_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        document: DocumentId,
        prototype: ordinary::NodePrototype<'_, 'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bindings = &events
            .public
            .as_ref()
            .ok_or(Error::ExpiredIdentity)?
            .document
            .ordinary;
        bindings.create_detached(
            scope,
            self,
            (owner, events),
            prototype,
            (document, DomDetachedNodeCreation::Fragment),
        )
    }
}
