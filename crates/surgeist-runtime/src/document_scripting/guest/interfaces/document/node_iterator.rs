//! Ordinary native traversal projected through the pinned DOM interfaces.

use super::*;
use crate::document_scripting::iterator_retirement;
use surgeist_bindings_js::conversion::IntegerType;
use surgeist_bindings_js::conversion::declarations::document_create_node_iterator_0_what_to_show;
use surgeist_bindings_js::{
    IntegerKind, IntegerMode, IntegerValue, ValueKind, dom_exception::DomExceptionName,
};
use surgeist_dom::{
    Dom,
    tree::{
        DomNodeFilterDecision, DomNodeIteratorFilter, DomNodeIteratorId, DomNodeIteratorProgress,
    },
};

const NODE_FILTER: &[u16] = &ascii(b"NodeFilter");
const NODE_ITERATOR: &[u16] = &ascii(b"NodeIterator");
const FILTER_REFERENCE: u32 = 0;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let global = scope.global()?;
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(funding.capture)?,
        FunctionMetadata {
            name: NODE_ITERATOR,
            length: 0,
        },
    )?;
    scope.define_property(
        &constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    install_interface(scope, &global, NODE_ITERATOR, &constructor, &prototype)?;
    install_filter_interface(scope, funding.capture)?;
    for (member, key, name) in [
        (Member::Root, &ascii(b"root")[..], &ascii(b"get root")[..]),
        (
            Member::Reference,
            &ascii(b"referenceNode")[..],
            &ascii(b"get referenceNode")[..],
        ),
        (
            Member::Before,
            &ascii(b"pointerBeforeReferenceNode")[..],
            &ascii(b"get pointerBeforeReferenceNode")[..],
        ),
        (
            Member::Mask,
            &ascii(b"whatToShow")[..],
            &ascii(b"get whatToShow")[..],
        ),
        (
            Member::Filter,
            &ascii(b"filter")[..],
            &ascii(b"get filter")[..],
        ),
    ] {
        let handler = (funding.capture)(size_of::<(Capture, Member)>())?;
        install_getter(
            scope,
            &prototype,
            key,
            name,
            true,
            Callback::new(move |scope, args| handler.iterator_member(scope, args, member)),
        )?;
    }
    for (name, direction) in [
        (&ascii(b"nextNode")[..], Direction::Next),
        (&ascii(b"previousNode")[..], Direction::Previous),
    ] {
        let handler = (funding.capture)(size_of::<(Capture, Direction)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.advance_iterator(scope, arguments, direction)
            }),
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
    }
    let handler = (funding.capture)(size_of::<Capture>())?;
    let detach = &ascii(b"detach");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.detach_iterator(scope, arguments)),
        FunctionMetadata {
            name: detach,
            length: 0,
        },
    )?;
    define_data(
        scope,
        &prototype,
        PropertyKey::String(detach),
        &method,
        true,
        true,
        true,
    )?;
    let handler = (funding.capture)(size_of::<Capture>())?;
    let factory = &ascii(b"createNodeIterator");
    let method = scope.bounded_function(
        &Callback::new(move |scope, arguments| handler.create_iterator(scope, arguments)),
        FunctionMetadata {
            name: factory,
            length: 1,
        },
        3,
    )?;
    define_data(
        scope,
        document,
        PropertyKey::String(factory),
        &method,
        true,
        true,
        true,
    )?;
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

fn install_filter_interface<'scope>(
    scope: &mut Scope<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let backing = capture(size_of::<Capture>())?;
    let interface = scope.fixed_arity_function(
        &Callback::new(move |scope, _| {
            let _backing = &backing;
            Ok(Completion::Throw(
                scope.type_error(&ascii(b"Illegal invocation"))?,
            ))
        }),
        FunctionMetadata {
            name: NODE_FILTER,
            length: 0,
        },
    )?;
    for (key, value) in [
        (&ascii(b"FILTER_ACCEPT")[..], 1),
        (&ascii(b"FILTER_REJECT")[..], 2),
        (&ascii(b"FILTER_SKIP")[..], 3),
        (&ascii(b"SHOW_ALL")[..], u32::MAX),
        (&ascii(b"SHOW_ELEMENT")[..], 0x001),
        (&ascii(b"SHOW_ATTRIBUTE")[..], 0x002),
        (&ascii(b"SHOW_TEXT")[..], 0x004),
        (&ascii(b"SHOW_CDATA_SECTION")[..], 0x008),
        (&ascii(b"SHOW_ENTITY_REFERENCE")[..], 0x010),
        (&ascii(b"SHOW_ENTITY")[..], 0x020),
        (&ascii(b"SHOW_PROCESSING_INSTRUCTION")[..], 0x040),
        (&ascii(b"SHOW_COMMENT")[..], 0x080),
        (&ascii(b"SHOW_DOCUMENT")[..], 0x100),
        (&ascii(b"SHOW_DOCUMENT_TYPE")[..], 0x200),
        (&ascii(b"SHOW_DOCUMENT_FRAGMENT")[..], 0x400),
        (&ascii(b"SHOW_NOTATION")[..], 0x800),
    ] {
        let value = scope.primitive(Primitive::Number(f64::from(value)))?;
        define_data(
            scope,
            &interface,
            PropertyKey::String(key),
            &value,
            false,
            true,
            false,
        )?;
    }
    let global = scope.global()?;
    define_data(
        scope,
        &global,
        PropertyKey::String(NODE_FILTER),
        &interface,
        true,
        false,
        true,
    )?;
    Ok(())
}

struct Projection {
    id: Cell<Option<DomNodeIteratorId>>,
    owner: Weak<NativeOwner>,
    retirement: Option<Box<iterator_retirement::Record>>,
}

impl Drop for Projection {
    fn drop(&mut self) {
        if let (Some(owner), Some(id), Some(record)) =
            (self.owner.upgrade(), self.id.get(), self.retirement.take())
        {
            owner.iterator_retirements.enqueue(id, record);
        }
    }
}

#[derive(Clone, Copy)]
enum Member {
    Root,
    Reference,
    Before,
    Mask,
    Filter,
}

#[derive(Clone, Copy)]
enum Direction {
    Next,
    Previous,
}

enum FilterResult {
    Decided(DomNodeFilterDecision, Root),
    Throw(Root),
}

fn iterator_id<'scope>(
    scope: &mut Scope<'scope>,
    receiver: &Value<'scope>,
) -> Result<Option<DomNodeIteratorId>, Error> {
    Ok(scope
        .native_object::<Projection>(receiver)?
        .and_then(|object| object.payload().id.get()))
}

impl Capture {
    fn create_iterator<'scope>(
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
        let Some(input) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let Some(root) = tree::projected_node(scope, input)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(
                        &document_create_node_iterator_0_what_to_show(),
                        arguments.get(1),
                    )
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(IntegerValue::UnsignedLong(mask)) = *converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            events.step()?;
            let filter = if let Some(input) = arguments.get(2) {
                match scope.kind(input)? {
                    ValueKind::Object => Some(input),
                    ValueKind::Null | ValueKind::Undefined => None,
                    _ => {
                        return Ok(Completion::Throw(
                            scope.type_error(&ascii(b"NodeFilter object required"))?,
                        ));
                    }
                }
            } else {
                None
            };
            scope.check_active()?;
            self.callbacks.with_read(&owner, &events, |read, _| {
                read.node(document.node())?;
                read.node(root.node())?;
                Ok(())
            })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let prototype = match &self.origin {
                Some(origin) => public.with_origin(&events, origin.global, |bindings| {
                    Ok(scope.open_callback_object(&bindings.documents.iterator)?)
                })?,
                None => scope.open_callback_object(&public.document.prototypes.iterator)?,
            };
            let backing = events.reserve(&owner, size_of::<iterator_retirement::Record>())?;
            let payload = Projection {
                id: Cell::new(None),
                owner: Rc::downgrade(&owner),
                retirement: Some(iterator_retirement::Record::new(backing)),
            };
            let prototype = ObjectPrototype::Object(&prototype);
            let admission = |bytes| events.reserve(&owner, bytes);
            let (value, filter) = match filter {
                Some(filter) => (
                    scope.project_native_object_with_reference(
                        prototype,
                        payload,
                        FILTER_REFERENCE,
                        filter,
                        admission,
                    )?,
                    DomNodeIteratorFilter::Callback,
                ),
                None => (
                    scope.project_native_object(prototype, payload, admission)?,
                    DomNodeIteratorFilter::None,
                ),
            };
            let projection = scope
                .native_object::<Projection>(&value)?
                .ok_or(Error::RegistryConflict)?;
            let id = self.with_iterator_dom(&owner, |dom| {
                if dom.pending_source_edit().is_none() {
                    owner.iterator_retirements.reap(dom)?;
                }
                Ok(dom.create_node_iterator(root.node(), mask, filter, &owner.cancellation)?)
            })?;
            projection.payload().id.set(Some(id));
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn iterator_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(id) = iterator_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NodeIterator required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let iterator = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().node_iterator(id)?)
            })?;
            let value = match member {
                Member::Root | Member::Reference => {
                    let node = if matches!(member, Member::Root) {
                        iterator.root()
                    } else {
                        iterator.reference_node()
                    };
                    self.project_observation(scope, &owner, &events, Observation::Node(Some(node)))?
                }
                Member::Before => {
                    scope.primitive(Primitive::Boolean(iterator.pointer_before_reference_node()))?
                }
                Member::Mask => {
                    scope.primitive(Primitive::Number(f64::from(iterator.what_to_show())))?
                }
                Member::Filter => match scope
                    .native_object_reference::<Projection>(arguments.receiver(), FILTER_REFERENCE)?
                {
                    Some(filter) => filter,
                    None => scope.primitive(Primitive::Null)?,
                },
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn detach_iterator<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        if iterator_id(scope, arguments.receiver())?.is_none() {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NodeIterator required"))?,
            ));
        }
        Ok(Completion::Undefined)
    }

    fn with_iterator_dom<T>(
        &self,
        owner: &NativeOwner,
        action: impl FnOnce(&mut Dom) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        let event = owner.active_event_request();
        self.callbacks
            .with_qualified_session(owner, event.as_ref(), |session, _| {
                action(session.dom().raw_mut())
            })
    }

    fn advance_iterator<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        direction: Direction,
    ) -> Result<Completion<'scope>, Error> {
        let Some(id) = iterator_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NodeIterator required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = self.advance_iterator_inner(
            scope,
            (&owner, &events),
            arguments.receiver(),
            id,
            direction,
        );
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::NodeIteratorFilterActive),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidStateError, &[])?,
            )),
            result => tree::tree_completion(scope, &events, result),
        }
    }

    fn advance_iterator_inner<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        receiver: &Value<'scope>,
        id: DomNodeIteratorId,
        direction: Direction,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let mut progress = self.with_iterator_dom(owner, |dom| {
            let admit = || events.step();
            match direction {
                Direction::Next => dom.node_iterator_next_accounted(id, admit, &owner.cancellation),
                Direction::Previous => {
                    dom.node_iterator_previous_accounted(id, admit, &owner.cancellation)
                }
            }
        })?;
        loop {
            match progress {
                DomNodeIteratorProgress::Complete(None) => {
                    return Ok(Completion::Return(scope.primitive(Primitive::Null)?));
                }
                DomNodeIteratorProgress::Complete(Some(_)) => {
                    return Err(Error::RegistryConflict.into());
                }
                DomNodeIteratorProgress::Match(pending) => {
                    let value = self.project_observation(
                        scope,
                        owner,
                        events,
                        Observation::Node(Some(pending.node())),
                    )?;
                    scope.check_active()?;
                    self.with_iterator_dom(owner, |dom| pending.accept(dom, &owner.cancellation))?;
                    return Ok(Completion::Return(value));
                }
                DomNodeIteratorProgress::Filter(pending) => {
                    let node = pending.node();
                    let (decision, value) =
                        match self.filter_candidate(scope, (owner, events), receiver, node)? {
                            FilterResult::Decided(decision, value) => (decision, value),
                            FilterResult::Throw(error) => {
                                return Ok(Completion::Throw(scope.import(&error)?));
                            }
                        };
                    let accepted = if decision == DomNodeFilterDecision::Accept {
                        Some(scope.import(&value)?)
                    } else {
                        None
                    };
                    scope.check_active()?;
                    progress = self.with_iterator_dom(owner, |dom| {
                        pending.respond(dom, decision, &owner.cancellation)
                    })?;
                    if let Some(value) = accepted {
                        if matches!(progress, DomNodeIteratorProgress::Complete(Some(accepted)) if accepted == node)
                        {
                            return Ok(Completion::Return(value));
                        }
                        return Err(Error::RegistryConflict.into());
                    }
                }
            }
        }
    }

    fn filter_candidate<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        iterator: &Value<'scope>,
        node: NodeId,
    ) -> Result<FilterResult, DocumentGuestEventError> {
        events.step()?;
        let filter = scope
            .native_object_reference::<Projection>(iterator, FILTER_REFERENCE)?
            .ok_or(Error::RegistryConflict)?;
        let callback = scope.callback_object(&filter)?;
        scope.in_callback_object_realm(&callback, |scope, filter| {
            let callable = scope.is_callable(filter)?;
            let method = if callable {
                None
            } else {
                events.step()?;
                let method = scope.get(filter, PropertyKey::String(&ascii(b"acceptNode")))?;
                if !scope.is_callable(&method)? {
                    let error =
                        scope.type_error(&ascii(b"NodeFilter.acceptNode must be callable"))?;
                    return Ok(FilterResult::Throw(scope.retain(&error)));
                }
                Some(method)
            };
            let value =
                self.project_observation(scope, owner, events, Observation::Node(Some(node)))?;
            let args = std::slice::from_ref(&value);
            let output = if let Some(method) = method {
                scope.call(&method, filter, args)?
            } else {
                let undefined = scope.primitive(Primitive::Undefined)?;
                scope.call(filter, &undefined, args)?
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.value(
                        &IntegerType::new(IntegerKind::UnsignedShort, IntegerMode::Wrap),
                        &output,
                    )
                })
                .map_err(Error::from)?;
            let IntegerValue::UnsignedShort(code) = *converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            let decision = match code {
                1 => DomNodeFilterDecision::Accept,
                2 => DomNodeFilterDecision::Reject,
                _ => DomNodeFilterDecision::Skip,
            };
            Ok(FilterResult::Decided(decision, scope.retain(&value)))
        })
    }
}
