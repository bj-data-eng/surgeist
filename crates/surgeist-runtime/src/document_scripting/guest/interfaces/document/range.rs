//! Host projection of a single generation-checked canonical live Range.

use super::*;
use crate::internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError};
use surgeist_bindings_js::{
    IntegerValue, NativeProjectionPrototype,
    conversion::declarations::{range_set_end_0_offset, range_set_start_0_offset},
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    range::{DomOwnedRange, DomRangeId},
    source::DomSourceEdit,
};

const ABSTRACT_RANGE: &[u16] = &ascii(b"AbstractRange");
const RANGE: &[u16] = &ascii(b"Range");

pub(super) struct Prototypes {
    range: RetainedCallbackObject,
}

#[derive(Default)]
struct Projection(OnceCell<DomOwnedRange>);

#[derive(Clone, Copy)]
enum Member {
    StartNode,
    StartOffset,
    EndNode,
    EndOffset,
    Collapsed,
    CommonAncestor,
}

#[derive(Clone, Copy)]
enum Boundary {
    Start,
    End,
}

enum Observed {
    Node(NodeId),
    Offset(usize),
    Collapsed(bool),
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<Prototypes, DocumentGuestEventError> {
    let abstract_prototype = scope.object(ObjectPrototype::Default)?;
    let abstract_constructor = scope.constructor(
        &illegal_constructor(funding.capture)?,
        FunctionMetadata {
            name: ABSTRACT_RANGE,
            length: 0,
        },
    )?;
    scope.define_property(
        &abstract_constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &abstract_prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    for (member, key, name) in [
        (
            Member::StartNode,
            &ascii(b"startContainer")[..],
            &ascii(b"get startContainer")[..],
        ),
        (
            Member::StartOffset,
            &ascii(b"startOffset"),
            &ascii(b"get startOffset"),
        ),
        (
            Member::EndNode,
            &ascii(b"endContainer"),
            &ascii(b"get endContainer"),
        ),
        (
            Member::EndOffset,
            &ascii(b"endOffset"),
            &ascii(b"get endOffset"),
        ),
        (
            Member::Collapsed,
            &ascii(b"collapsed"),
            &ascii(b"get collapsed"),
        ),
    ] {
        install_boundary_getter(
            scope,
            &abstract_prototype,
            (member, key, name),
            funding.capture,
        )?;
    }
    let global = scope.global()?;
    install_interface(
        scope,
        &global,
        ABSTRACT_RANGE,
        &abstract_constructor,
        &abstract_prototype,
    )?;

    let fallback = (funding.capture)(size_of::<Capture>())?;
    let construct = constructor_capture(funding.capture, size_of::<Capture>())?;
    let callback = ConstructorCallback::bounded(
        0,
        move |scope| fallback.range_prototype(scope),
        move |scope, arguments| construct.construct_range(scope, arguments),
    );
    let (constructor, prototype) = prepare_derived_interface(
        scope,
        &abstract_prototype,
        &abstract_constructor,
        FunctionMetadata {
            name: RANGE,
            length: 0,
        },
        &callback,
    )?;
    install_boundary_getter(
        scope,
        &prototype,
        (
            Member::CommonAncestor,
            &ascii(b"commonAncestorContainer"),
            &ascii(b"get commonAncestorContainer"),
        ),
        funding.capture,
    )?;
    for (boundary, name) in [
        (Boundary::Start, &ascii(b"setStart")[..]),
        (Boundary::End, &ascii(b"setEnd")),
    ] {
        let setter = (funding.capture)(size_of::<(Capture, Boundary)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                setter.set_projection_boundary(scope, arguments, boundary)
            }),
            FunctionMetadata { name, length: 2 },
        )?;
        define_data(
            scope,
            &prototype,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }
    let detach = (funding.capture)(size_of::<Capture>())?;
    let name = &ascii(b"detach");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| detach.detach_range(scope, arguments)),
        FunctionMetadata { name, length: 0 },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    install_constants(scope, &constructor, &prototype, interface_constants::RANGE)?;
    install_interface(scope, &global, RANGE, &constructor, &prototype)?;
    let prototype = scope.callback_object(&prototype)?;
    Ok(Prototypes {
        range: scope.retain_callback_object(prototype, funding.retain)?,
    })
}

pub(super) fn install_factory<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let factory = capture(size_of::<Capture>())?;
    let name = &ascii(b"createRange");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_range(scope, arguments)),
        FunctionMetadata { name, length: 0 },
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

fn install_boundary_getter<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    (member, key, name): (Member, &[u16], &[u16]),
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let reader = capture(size_of::<(Capture, Member)>())?;
    install_getter(
        scope,
        prototype,
        key,
        name,
        true,
        Callback::new(move |scope, arguments| reader.read_boundary(scope, arguments, member)),
    )?;
    Ok(())
}

fn range_id<'scope>(
    scope: &mut Scope<'scope>,
    value: &Value<'scope>,
) -> Result<Option<DomRangeId>, Error> {
    Ok(scope
        .native_object::<Projection>(value)?
        .and_then(|projection| projection.payload().0.get().map(DomOwnedRange::id)))
}

impl Capture {
    fn create_range<'scope>(
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
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let origin = public.other_document_origin(&owner, &events, document)?;
            public.with_range_prototype(&events, origin, |prototype| {
                self.project_range(
                    scope,
                    (&owner, &events),
                    NativeProjectionPrototype::Creation(prototype),
                    document,
                )
            })
        })();
        complete(scope, &events, result)
    }

    fn construct_range<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let request = owner.active_event_request();
            let document = self.callbacks.with_qualified_session(
                &owner,
                request.as_ref(),
                |session, attached| {
                    let issuer = self
                        .origin
                        .as_ref()
                        .map_or(attached, |origin| origin.global);
                    session.dom().raw().dispatch_global_document_with(
                        issuer,
                        &owner.cancellation,
                        &mut || events.step(),
                    )
                },
            )?;
            let prototype =
                NativeProjectionPrototype::Current(ObjectPrototype::Object(selected.prototype()));
            self.project_range(scope, (&owner, &events), prototype, document)
        })();
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn range_prototype<'scope>(&self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        let (_, events) = self.active(scope)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let origin = self.origin.as_ref().map(|origin| origin.global);
        match public.with_range_prototype(&events, origin, |prototype| {
            Ok(scope.open_callback_object(prototype)?)
        }) {
            Ok(prototype) => Ok(prototype),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn project_range<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: NativeProjectionPrototype<'_, 'scope>,
        document: DocumentId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        events.step()?;
        let (value, _, ()) = scope.try_project_retained_native_object(
            prototype,
            Projection::default(),
            |payload, holder| {
                Ok::<_, DocumentGuestEventError>((
                    events.reserve(owner, payload)?,
                    events.reserve(owner, holder)?,
                ))
            },
            |payload| {
                let request = owner.active_event_request();
                let owned = self.callbacks.with_qualified_session(
                    owner,
                    request.as_ref(),
                    |session, _| {
                        Ok(session.dom().raw_mut().create_owned_range(
                            document,
                            events.limits.source_edits,
                            &owner.cancellation,
                        )?)
                    },
                )?;
                payload.0.set(owned).map_err(|_| Error::RegistryConflict)?;
                Ok(())
            },
        )?;
        Ok(value)
    }

    fn read_boundary<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(id) = range_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Range required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let observed = self.callbacks.with_read(&owner, &events, |read, _| {
                let current = || {
                    read.step()?;
                    read.snapshot
                        .range(id)
                        .map_err(DocumentGuestEventError::from)
                };
                Ok(match member {
                    Member::StartNode => Observed::Node(current()?.start().container()),
                    Member::StartOffset => Observed::Offset(current()?.start().offset()),
                    Member::EndNode => Observed::Node(current()?.end().container()),
                    Member::EndOffset => Observed::Offset(current()?.end().offset()),
                    Member::Collapsed => Observed::Collapsed(current()?.collapsed()),
                    Member::CommonAncestor => Observed::Node(
                        read.snapshot
                            .range_common_ancestor_with(id, || read.step())?,
                    ),
                })
            })?;
            match observed {
                Observed::Node(node) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, node),
                Observed::Offset(offset) => {
                    let offset = u32::try_from(offset)
                        .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
                    Ok(scope.primitive(Primitive::Number(f64::from(offset)))?)
                }
                Observed::Collapsed(collapsed) => {
                    Ok(scope.primitive(Primitive::Boolean(collapsed))?)
                }
            }
        })();
        complete(scope, &events, result)
    }

    fn set_projection_boundary<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        boundary: Boundary,
    ) -> Result<Completion<'scope>, Error> {
        let Some(id) = range_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Range required"))?,
            ));
        };
        let Some(container) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let Some(container) = tree::projected_node(scope, container)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let offset = match boundary {
                Boundary::Start => range_set_start_0_offset(),
                Boundary::End => range_set_end_0_offset(),
            };
            let offset = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&offset, arguments.get(offset.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(IntegerValue::UnsignedLong(offset)) = *offset.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let offset = usize::try_from(offset)
                .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
            scope.check_active()?;
            let edit = match boundary {
                Boundary::Start => DomSourceEdit::set_range_start(id, container.node(), offset),
                Boundary::End => DomSourceEdit::set_range_end(id, container.node(), offset),
            };
            let drive = self.drive_guest_tree_edit(
                &owner,
                &events,
                scope.realm_authority()?.identity(),
                edit,
            )?;
            if drive.pending.map(|status| status.operation()) == drive.parent
                && matches!(
                    &drive.progress,
                    Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                        DomError::WrongNodeKind
                    )))
                )
            {
                return Ok(Completion::Throw(
                    scope.dom_exception(DomExceptionName::InvalidNodeTypeError, &[])?,
                ));
            }
            self.finish_guest_tree_edit(
                scope,
                &owner,
                &events,
                drive.progress,
                (drive.parent, drive.pending),
                |_, _| Ok(Completion::Undefined),
            )
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn detach_range<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        if range_id(scope, arguments.receiver())?.is_none() {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Range required"))?,
            ));
        }
        let (_, events) = self.active(scope)?;
        if let Err(error) = events.step() {
            return refuse(scope, &events.refusal, error);
        }
        Ok(Completion::Undefined)
    }
}

impl PublicEventBindings {
    fn with_range_prototype<T>(
        &self,
        events: &GuestEvents,
        origin: Option<DomHostedGlobalId>,
        access: impl FnOnce(&RetainedCallbackObject) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        if let Some(origin) = origin {
            return self.with_origin(events, origin, |bindings| {
                access(&bindings.documents.range.range)
            });
        }
        access(&self.document.prototypes.range.range)
    }
}
