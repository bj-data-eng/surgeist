//! Native ParentNode and ChildNode mutation mixins.

use super::*;
use character_data::copy_native_data;
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::{DomNodeOrString, DomSourceEdit};

#[derive(Clone, Copy)]
pub(super) enum Receiver {
    Document,
    Fragment,
    Element,
    CharacterData,
    DocumentType,
}

impl Receiver {
    pub(super) fn accepts(self, node: GuestNode) -> bool {
        matches!(
            (self, node),
            (Self::Document, GuestNode::Document(_))
                | (
                    Self::Fragment,
                    GuestNode::Fragment(_) | GuestNode::ShadowRoot(_)
                )
                | (Self::Element, GuestNode::Element(_))
                | (
                    Self::CharacterData,
                    GuestNode::Text(_)
                        | GuestNode::Comment(_)
                        | GuestNode::ProcessingInstruction(_)
                )
                | (Self::DocumentType, GuestNode::DocumentType(_))
        )
    }
}

#[derive(Clone, Copy)]
enum Method {
    Prepend,
    Append,
    ReplaceChildren,
    Before,
    After,
    ReplaceWith,
    Remove,
}

impl Method {
    fn name(self) -> &'static [u16] {
        match self {
            Self::Prepend => const { &ascii(b"prepend") },
            Self::Append => const { &ascii(b"append") },
            Self::ReplaceChildren => const { &ascii(b"replaceChildren") },
            Self::Before => const { &ascii(b"before") },
            Self::After => const { &ascii(b"after") },
            Self::ReplaceWith => const { &ascii(b"replaceWith") },
            Self::Remove => const { &ascii(b"remove") },
        }
    }

    fn edit(self, node: NodeId, operands: Vec<DomNodeOrString>) -> DomSourceEdit {
        match self {
            Self::Prepend => DomSourceEdit::prepend(node, operands),
            Self::Append => DomSourceEdit::append(node, operands),
            Self::ReplaceChildren => DomSourceEdit::replace_children(node, operands),
            Self::Before => DomSourceEdit::before(node, operands),
            Self::After => DomSourceEdit::after(node, operands),
            Self::ReplaceWith => DomSourceEdit::replace_with(node, operands),
            Self::Remove => DomSourceEdit::detach(node),
        }
    }
}

pub(super) fn unscopables<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
) -> Result<Value<'scope>, Error> {
    let names = scope.object(ObjectPrototype::Null)?;
    let symbol = scope.well_known_symbol(WellKnownSymbol::Unscopables)?;
    define_data(
        scope,
        prototype,
        PropertyKey::Value(&symbol),
        &names,
        false,
        false,
        true,
    )?;
    Ok(names)
}

pub(super) fn install_parent<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    names: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    install_methods(
        scope,
        prototype,
        receiver,
        names,
        capture,
        &[Method::Prepend, Method::Append, Method::ReplaceChildren],
    )
}

pub(super) fn install_child<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    names: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    install_methods(
        scope,
        prototype,
        receiver,
        names,
        capture,
        &[
            Method::Before,
            Method::After,
            Method::ReplaceWith,
            Method::Remove,
        ],
    )
}

fn install_methods<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    names: &Value<'scope>,
    capture: CaptureFunding<'_>,
    methods: &[Method],
) -> Result<(), DocumentGuestEventError> {
    let yes = scope.primitive(Primitive::Boolean(true))?;
    for &method in methods {
        let name = method.name();
        let handler = capture(size_of::<(Capture, Receiver, Method)>())?;
        let maximum = if matches!(method, Method::Remove) {
            0
        } else {
            u32::MAX
        };
        let function = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                handler.mutate_nodes(scope, arguments, receiver, method)
            }),
            FunctionMetadata { name, length: 0 },
            maximum,
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
        define_data(
            scope,
            names,
            PropertyKey::String(name),
            &yes,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn mutate_nodes<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let node = tree::projected_node(scope, arguments.receiver())?;
        let Some(node) = node.filter(|node| receiver.accepts(*node)) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Invalid mutation receiver"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let count = arguments.len();
            let bytes = size_of::<DomNodeOrString>()
                .checked_add(size_of::<GuestBacking>())
                .and_then(|item| count.checked_mul(item))
                .ok_or(Error::Allocation)?;
            let _buffers = events.reserve(&owner, bytes)?;
            // Operand values must release their text before its credit owners.
            let mut strings = Vec::new();
            strings
                .try_reserve_exact(count)
                .map_err(|_| Error::Allocation)?;
            let mut operands = Vec::new();
            operands
                .try_reserve_exact(count)
                .map_err(|_| Error::Allocation)?;
            for value in arguments.into_parts().1 {
                events.step()?;
                if let Some(node) = scope.native_object::<GuestNodeProjection>(&value)? {
                    let node = node.payload().id().ok_or(DomError::StaleNode)?;
                    operands.push(DomNodeOrString::Node(node.node()));
                } else {
                    let value = self
                        .callbacks
                        .conversion
                        .run(scope, |cx| cx.value(&DOM_STRING, &value))
                        .map_err(Error::from)?;
                    scope.check_active()?;
                    let (value, backing) =
                        copy_native_data(&owner, &events, value.get().as_code_units())?;
                    operands.push(DomNodeOrString::String(value));
                    strings.push(backing);
                }
            }
            scope.check_active()?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                method.edit(node.node(), operands),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
