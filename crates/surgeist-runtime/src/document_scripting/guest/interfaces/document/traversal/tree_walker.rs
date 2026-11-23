//! TreeWalker descriptors and the adapter for native suspended navigation.

use super::*;
use surgeist_dom::tree::{DomTreeWalkerDirection, DomTreeWalkerId, DomTreeWalkerProgress};

const TREE_WALKER: &[u16] = &ascii(b"TreeWalker");
const CURRENT_NODE: &[u16] = &ascii(b"currentNode");

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    funding: &OriginInterfaceFunding<'_>,
) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
    let global = scope.global()?;
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(funding.capture)?,
        FunctionMetadata {
            name: TREE_WALKER,
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
    for (member, key, name) in [
        (Member::Root, &ascii(b"root")[..], &ascii(b"get root")[..]),
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
            Callback::new(move |scope, arguments| handler.walker_member(scope, arguments, member)),
        )?;
    }
    let getter = (funding.capture)(size_of::<Capture>())?;
    let get = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            getter.walker_member(scope, arguments, Member::Current)
        }),
        FunctionMetadata {
            name: &ascii(b"get currentNode"),
            length: 0,
        },
    )?;
    let setter = (funding.capture)(size_of::<Capture>())?;
    let set = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| setter.set_walker_current(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"set currentNode"),
            length: 1,
        },
    )?;
    scope.define_new_property(
        &prototype,
        PropertyKey::String(CURRENT_NODE),
        PropertyDescriptor::Accessor {
            get: Some(&get),
            set: Some(&set),
            enumerable: true,
            configurable: true,
        },
    )?;
    for (name, direction) in [
        (&ascii(b"parentNode")[..], DomTreeWalkerDirection::Parent),
        (
            &ascii(b"firstChild")[..],
            DomTreeWalkerDirection::FirstChild,
        ),
        (&ascii(b"lastChild")[..], DomTreeWalkerDirection::LastChild),
        (
            &ascii(b"previousSibling")[..],
            DomTreeWalkerDirection::PreviousSibling,
        ),
        (
            &ascii(b"nextSibling")[..],
            DomTreeWalkerDirection::NextSibling,
        ),
        (
            &ascii(b"previousNode")[..],
            DomTreeWalkerDirection::PreviousNode,
        ),
        (&ascii(b"nextNode")[..], DomTreeWalkerDirection::NextNode),
    ] {
        let handler = (funding.capture)(size_of::<(Capture, DomTreeWalkerDirection)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.navigate_walker(scope, arguments, direction)
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
    install_interface(scope, &global, TREE_WALKER, &constructor, &prototype)?;
    let observed = scope.callback_object(&prototype)?;
    scope.retain_callback_object(observed, funding.retain)
}

pub(super) fn install_factory<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    let factory = &ascii(b"createTreeWalker");
    let method = scope.bounded_function(
        &Callback::new(move |scope, arguments| {
            handler.create_cursor(scope, arguments, CursorKind::Walker)
        }),
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
    Ok(())
}

#[derive(Clone, Copy)]
enum Member {
    Root,
    Mask,
    Filter,
    Current,
}

impl Capture {
    fn walker_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(CursorId::Walker(id)) = cursor_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"TreeWalker required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let walker = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().tree_walker(id)?)
            })?;
            let value = match member {
                Member::Root | Member::Current => {
                    let node = if matches!(member, Member::Root) {
                        walker.root()
                    } else {
                        walker.current_node()
                    };
                    self.project_observation(scope, &owner, &events, Observation::Node(Some(node)))?
                }
                Member::Mask => {
                    scope.primitive(Primitive::Number(f64::from(walker.what_to_show())))?
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

    fn set_walker_current<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(CursorId::Walker(id)) = cursor_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"TreeWalker required"))?,
            ));
        };
        let Some(input) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let Some(node) = tree::projected_node(scope, input)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            scope.check_active()?;
            events.step()?;
            self.with_cursor_dom(&owner, |dom| {
                Ok(dom.set_tree_walker_current(id, node.node(), &owner.cancellation)?)
            })?;
            Ok(Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn navigate_walker<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        direction: DomTreeWalkerDirection,
    ) -> Result<Completion<'scope>, Error> {
        let Some(CursorId::Walker(id)) = cursor_id(scope, arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"TreeWalker required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = self.navigate_walker_inner(
            scope,
            (&owner, &events),
            arguments.receiver(),
            id,
            direction,
        );
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::TreeWalkerFilterActive),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidStateError, &[])?,
            )),
            result => tree::tree_completion(scope, &events, result),
        }
    }

    fn navigate_walker_inner<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        receiver: &Value<'scope>,
        id: DomTreeWalkerId,
        direction: DomTreeWalkerDirection,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let mut progress = self.with_cursor_dom(owner, |dom| {
            dom.navigate_tree_walker(id, direction, || events.step(), &owner.cancellation)
        })?;
        let mut prepared = None;
        loop {
            match progress {
                DomTreeWalkerProgress::NoMatch => {
                    return Ok(Completion::Return(scope.primitive(Primitive::Null)?));
                }
                DomTreeWalkerProgress::Match(pending) => {
                    let node = pending.node();
                    let value = match prepared.filter(|(candidate, _)| *candidate == node) {
                        Some((_, value)) => scope.import(&value)?,
                        None => self.project_observation(
                            scope,
                            owner,
                            events,
                            Observation::Node(Some(node)),
                        )?,
                    };
                    scope.check_active()?;
                    self.with_cursor_dom(owner, |dom| pending.accept(dom))?;
                    return Ok(Completion::Return(value));
                }
                DomTreeWalkerProgress::Filter(pending) => {
                    let node = pending.node();
                    let (code, value) =
                        match self.filter_candidate(scope, (owner, events), receiver, node)? {
                            FilterResult::Decided(code, value) => (code, value),
                            FilterResult::Throw(error) => {
                                return Ok(Completion::Throw(scope.import(&error)?));
                            }
                        };
                    scope.check_active()?;
                    progress = self.with_cursor_dom(owner, |dom| pending.respond(dom, code))?;
                    prepared = Some((node, value));
                }
            }
        }
    }
}
