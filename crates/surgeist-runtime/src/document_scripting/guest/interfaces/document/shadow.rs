//! Canonical ShadowRoot interface and its native host relationship.

use super::*;
use surgeist_bindings_js::{
    conversion::{ArgumentValue, declarations::element_attach_shadow_0_init},
    declarations::{ShadowRootMode, SlotAssignmentMode},
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    DomShadowRootInit, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
};

const OPEN: &[u16] = &ascii(b"open");
const CLOSED: &[u16] = &ascii(b"closed");
const MANUAL: &[u16] = &ascii(b"manual");
const NAMED: &[u16] = &ascii(b"named");

#[derive(Clone, Copy)]
enum Member {
    Mode,
    DelegatesFocus,
    Serializable,
    SlotAssignment,
    Clonable,
    Host,
}

enum Observation {
    Host(NodeId),
    Boolean(bool),
    Keyword(&'static [u16]),
}

pub(super) fn install_attachment<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"attachShadow");
    let function = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.attach_shadow(scope, arguments)),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        element,
        PropertyKey::String(name),
        &function,
        true,
        true,
        true,
    )
    .map_err(Into::into)
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    fragment_constructor: &Value<'scope>,
    fragment: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let (_, prototype) = install_derived_interface(
        scope,
        fragment,
        fragment_constructor,
        FunctionMetadata {
            name: &ascii(b"ShadowRoot"),
            length: 0,
        },
        &illegal_constructor(),
    )?;
    for (member, key, name) in [
        (Member::Mode, &ascii(b"mode")[..], &ascii(b"get mode")[..]),
        (
            Member::DelegatesFocus,
            &ascii(b"delegatesFocus")[..],
            &ascii(b"get delegatesFocus")[..],
        ),
        (
            Member::Serializable,
            &ascii(b"serializable")[..],
            &ascii(b"get serializable")[..],
        ),
        (
            Member::SlotAssignment,
            &ascii(b"slotAssignment")[..],
            &ascii(b"get slotAssignment")[..],
        ),
        (
            Member::Clonable,
            &ascii(b"clonable")[..],
            &ascii(b"get clonable")[..],
        ),
        (Member::Host, &ascii(b"host")[..], &ascii(b"get host")[..]),
    ] {
        let handler = capture(size_of::<(Capture, Member)>())?;
        install_getter(
            scope,
            &prototype,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| handler.shadow_member(scope, arguments, member)),
        )?;
    }
    Ok(prototype)
}

impl Capture {
    fn attach_shadow<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(host)) = receiver.and_then(|node| node.payload().id()) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let descriptor = element_attach_shadow_0_init();
            let converted = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    cx.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(init) = *converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            let mode = match init.mode {
                ShadowRootMode::Open => HtmlShadowRootMode::Open,
                ShadowRootMode::Closed => HtmlShadowRootMode::Closed,
            };
            let assignment = match init.slot_assignment {
                SlotAssignmentMode::Named => HtmlSlotAssignment::Named,
                SlotAssignmentMode::Manual => HtmlSlotAssignment::Manual,
            };
            let mut native = DomShadowRootInit::from(HtmlShadowRootProperties::new(
                mode,
                assignment,
                init.delegates_focus,
                init.clonable,
                init.serializable,
            ));
            if init.custom_element_registry_is_null {
                native = native.with_null_registry();
            }
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let (document, existing) = self.with_native(&owner, |session, _| {
                let snapshot = session.snapshot();
                let node = snapshot.node_with(host, || events.step())?;
                let document = node
                    .owner_document()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                Ok((document, node.hosted_shadow_root()))
            })?;
            if let Some(root) = existing
                && let Some(projected) = public
                    .document
                    .ordinary
                    .open_with(scope, root, || events.step())?
            {
                self.with_native(&owner, |session, _| {
                    let (_, attached) = session.dom().raw_mut().attach_shadow_synchronously(
                        host,
                        native,
                        events.limits.source_edits,
                        &owner.cancellation,
                    )?;
                    if root != attached {
                        return Err(DocumentScriptNativeError::ProjectionCustody.into());
                    }
                    Ok(())
                })?;
                return Ok(projected);
            }
            let origin = if let Some(root) = existing {
                self.with_native(&owner, |session, attached| {
                    let snapshot = session.snapshot();
                    let root = snapshot.node_with(root, || events.step())?;
                    events.step()?;
                    let global = root
                        .relevant_global()
                        .map_err(DomHostedRuntimeError::Dom)
                        .map_err(DocumentScriptNativeError::from)?
                        .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
                    Ok((global != attached).then_some(global))
                })?
            } else {
                public.other_document_origin(&owner, &events, document)?
            };
            let mut publish = |prototypes: &DocumentPrototypes| {
                public.document.ordinary.create(
                    scope,
                    &owner,
                    &events,
                    ordinary::NodePrototype::Creation(&prototypes.shadow),
                    |inbox, key| {
                        self.with_native(&owner, |session, _| {
                            let (_, root, watch) = session
                                .dom()
                                .raw_mut()
                                .attach_shadow_with_node_retirement_watch(
                                    host,
                                    native,
                                    inbox,
                                    key,
                                    (events.limits.source_edits, events.limits.node_retirement),
                                    &owner.cancellation,
                                )?
                                .into_parts();
                            Ok((GuestNode::ShadowRoot(root), watch))
                        })
                    },
                )
            };
            if let Some(origin) = origin {
                public.with_origin(&events, origin, |origin| publish(&origin.documents))
            } else {
                publish(&public.document.prototypes)
            }
        })();
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::NotSupported),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
            )),
            _ => complete(scope, &events, result),
        }
    }

    fn shadow_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::ShadowRoot(root)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ShadowRoot required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let observation = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(root)?;
                let DomNodeKind::ShadowRoot(state) = node.kind() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                read.step()?;
                Ok(match member {
                    Member::Mode => Observation::Keyword(match state.mode() {
                        HtmlShadowRootMode::Open => OPEN,
                        HtmlShadowRootMode::Closed => CLOSED,
                    }),
                    Member::DelegatesFocus => Observation::Boolean(state.delegates_focus()),
                    Member::Serializable => Observation::Boolean(state.serializable()),
                    Member::SlotAssignment => Observation::Keyword(match state.slot_assignment() {
                        HtmlSlotAssignment::Manual => MANUAL,
                        HtmlSlotAssignment::Named => NAMED,
                    }),
                    Member::Clonable => Observation::Boolean(state.clonable()),
                    Member::Host => Observation::Host(
                        node.shadow_host()
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
                    ),
                })
            })?;
            match observation {
                Observation::Host(host) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, host),
                Observation::Boolean(value) => Ok(scope.primitive(Primitive::Boolean(value))?),
                Observation::Keyword(value) => Ok(scope.string(value)?),
            }
        })();
        complete(scope, &events, result)
    }
}
