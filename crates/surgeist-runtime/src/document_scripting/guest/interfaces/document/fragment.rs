//! Canonical ordinary DocumentFragment creation and projection custody.

use super::*;

const FRAGMENT: &[u16] = &ascii(b"DocumentFragment");
const CREATE_FRAGMENT: &[u16] = &ascii(b"createDocumentFragment");

pub(super) struct FragmentBindings {
    prototype: RetainedCallbackObject,
}

impl FragmentBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let prototype = scope.object(ObjectPrototype::Object(node))?;
        let fallback = capture(size_of::<Capture>())?;
        let constructor = capture(size_of::<Capture>())?;
        let callback = ConstructorCallback::bounded(
            0,
            move |scope| {
                let (_, events) = fallback.active(scope)?;
                let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                scope.open_callback_object(&public.document.fragment.prototype)
            },
            move |scope, arguments| constructor.construct_fragment(scope, arguments),
        );
        let interface = scope.constructor(
            &callback,
            FunctionMetadata {
                name: FRAGMENT,
                length: 0,
            },
        )?;
        scope.set_prototype(&interface, ObjectPrototype::Object(node_constructor))?;
        scope.define_property(
            &interface,
            PropertyKey::String(PROTOTYPE),
            PropertyDescriptor::Data {
                value: &prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        let global = scope.global()?;
        install_interface(scope, &global, FRAGMENT, &interface, &prototype)?;
        let observed = scope.callback_object(&prototype)?;
        Ok(Self {
            prototype: scope.retain_callback_object(observed, fund)?,
        })
    }
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
                selected.prototype(),
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
            let bindings = &events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document;
            let prototype = scope.open_callback_object(&bindings.fragment.prototype)?;
            self.publish_fragment(scope, &owner, &events, document, &prototype)
        })();
        complete(scope, &events, result)
    }

    fn publish_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        document: DocumentId,
        prototype: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bindings = &events
            .public
            .as_ref()
            .ok_or(Error::ExpiredIdentity)?
            .document
            .ordinary;
        bindings.project(
            scope,
            owner,
            events,
            ordinary::NodePrototype::Selected(prototype),
            |inbox, key| {
                self.with_native(owner, |session, _| {
                    let mut batch = DomBatch::new();
                    let ticket = batch.create_fragment(document);
                    let (_, node, watch) = session
                        .dom()
                        .raw_mut()
                        .apply_with_node_retirement_watch(
                            batch,
                            &ticket,
                            inbox,
                            key,
                            events.limits.node_retirement,
                            &owner.cancellation,
                        )
                        .map_err(DocumentScriptNativeError::from)?
                        .into_parts();
                    Ok((GuestNode::Fragment(node), watch))
                })
            },
        )
    }
}
