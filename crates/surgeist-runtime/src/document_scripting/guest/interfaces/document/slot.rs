//! Slot reflections, canonical sequence observations and original manual assignment.

use super::*;
use surgeist_bindings_js::conversion::{
    ArgumentValue,
    declarations::{
        html_slot_element_assigned_elements_0_options, html_slot_element_assigned_nodes_0_options,
    },
};
use surgeist_dom::{DomNode, DomSlotQueryLimits, DomSnapshot, DomSourceEdit};

const SLOT_REQUIRED: &[u16] = &ascii(b"HTMLSlotElement required");
const NAME: &[reflection::Member] = &[(
    &ascii(b"name"),
    &ascii(b"get name"),
    &ascii(b"set name"),
    &ascii(b"name"),
    reflection::Reflection::String,
)];

#[derive(Clone, Copy)]
enum Method {
    Nodes,
    Elements,
    Assign,
}

#[derive(Clone, Copy)]
pub(super) enum SlottableReceiver {
    Element,
    Text,
}

impl SlottableReceiver {
    fn accepts(self, node: GuestNode) -> bool {
        matches!(
            (self, node),
            (Self::Element, GuestNode::Element(_)) | (Self::Text, GuestNode::Text(_))
        )
    }

    fn error_message(self) -> &'static [u16] {
        match self {
            Self::Element => const { &ascii(b"Element required") },
            Self::Text => const { &ascii(b"Text required") },
        }
    }
}

pub(super) fn is_slot(node: DomNode<'_>) -> bool {
    node.as_slot().is_some()
        && node
            .as_element()
            .is_some_and(|element| !element.has_failed_construction_interface())
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, reflection::Receiver::Slot, NAME)?;
    for (method, name) in [
        (Method::Nodes, &ascii(b"assignedNodes")[..]),
        (Method::Elements, &ascii(b"assignedElements")[..]),
        (Method::Assign, &ascii(b"assign")[..]),
    ] {
        let handler = capture(size_of::<(Capture, Method)>())?;
        let function = scope.bounded_function(
            &Callback::new(move |scope, arguments| handler.slot_method(scope, arguments, method)),
            FunctionMetadata { name, length: 0 },
            if matches!(method, Method::Assign) {
                u32::MAX
            } else {
                1
            },
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
    }
    Ok(())
}

pub(super) fn install_slottable<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: SlottableReceiver,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, SlottableReceiver)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"assignedSlot"),
        &ascii(b"get assignedSlot"),
        true,
        Callback::new(move |scope, arguments| handler.slottable_slot(scope, arguments, receiver)),
    )?;
    Ok(())
}

// The selected identities and their actual allocation credit leave the native
// read together. Neither the session nor a ledger loan crosses wrapper creation.
struct Selection {
    nodes: Vec<NodeId>,
    backing: Option<GuestBacking>,
    _source: DomSnapshot,
}

impl Selection {
    fn new(source: DomSnapshot) -> Self {
        Self {
            nodes: Vec::new(),
            backing: None,
            _source: source,
        }
    }

    fn push(&mut self, read: &NativeRead<'_>, node: NodeId) -> Result<(), DocumentGuestEventError> {
        if self.nodes.len() == self.nodes.capacity() {
            let capacity = self
                .nodes
                .capacity()
                .checked_mul(2)
                .and_then(|capacity| capacity.checked_add(1))
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            let backing = read.reserve(buffer_bytes::<NodeId>(capacity)?)?;
            let mut replacement = buffer(capacity)?;
            for &retained in &self.nodes {
                read.step()?;
                replacement.push(retained);
            }
            let previous = std::mem::replace(&mut self.nodes, replacement);
            drop(previous);
            self.backing = Some(backing);
        }
        read.step()?;
        self.nodes.push(node);
        Ok(())
    }
}

fn buffer_bytes<T>(count: usize) -> Result<usize, DocumentGuestEventError> {
    count
        .checked_mul(size_of::<T>())
        .ok_or_else(|| Error::Allocation.into())
}

fn buffer<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    if values.capacity() != count {
        return Err(Error::Allocation);
    }
    Ok(values)
}

impl Capture {
    fn slot_method<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(slot)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(SLOT_REQUIRED)?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            // Native Slot authentication precedes dictionary getters and all
            // variadic operand processing, including on borrowed methods.
            let branded = self
                .callbacks
                .with_read(&owner, &events, |read, _| Ok(is_slot(read.node(slot)?)))?;
            if !branded {
                return Ok(Completion::Throw(scope.type_error(SLOT_REQUIRED)?));
            }
            if matches!(method, Method::Assign) {
                return self.assign_slot(scope, &owner, &events, slot, &arguments);
            }
            let descriptor = match method {
                Method::Nodes => html_slot_element_assigned_nodes_0_options(),
                Method::Elements => html_slot_element_assigned_elements_0_options(),
                Method::Assign => return Err(Error::BackendUnavailable.into()),
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    cx.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(options) = *converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            let selection = self.slot_selection(
                &owner,
                &events,
                slot,
                options.flatten,
                matches!(method, Method::Elements),
            )?;
            self.slot_array(scope, &owner, &events, selection)
                .map(Completion::Return)
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn slot_selection(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        slot: NodeId,
        flatten: bool,
        elements: bool,
    ) -> Result<Selection, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let slot = read
                .node(slot)?
                .as_slot()
                .ok_or(DomError::SlotInapplicable)?;
            read.step()?;
            let mut selected = Selection::new(read.snapshot.clone());
            let visit = |node: DomNode<'_>| {
                read.step()?;
                if !elements || node.as_element().is_some() {
                    selected.push(read, node.id())?;
                }
                Ok::<_, DocumentGuestEventError>(())
            };
            if flatten {
                // Every physical query allocation still passes read.reserve;
                // these finite extents only bound the native query's shape.
                let extent = events.limits.storage_bytes / size_of::<NodeId>();
                let limits = DomSlotQueryLimits::new(extent, extent)?;
                slot.visit_flattened_slottables_with(
                    limits,
                    || read.step(),
                    |bytes| read.reserve(bytes),
                    visit,
                )?;
            } else {
                slot.visit_assigned_nodes_with(|| read.step(), visit)?;
            }
            Ok(selected)
        })
    }

    fn slot_array<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        selection: Selection,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let _output_backing = self.callbacks.with_read(owner, events, |read, _| {
            read.reserve(buffer_bytes::<Value<'scope>>(selection.nodes.len())?)
        })?;
        let mut values = buffer(selection.nodes.len())?;
        for &node in &selection.nodes {
            self.callbacks
                .with_read(owner, events, |read, _| read.step())?;
            values.push(public.project_node(scope, owner, events, node)?);
        }
        Ok(scope.array(&values)?)
    }

    fn assign_slot<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        slot: NodeId,
        arguments: &Arguments<'scope>,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let count = arguments.len();
        let input_backing = self.callbacks.with_read(owner, events, |read, _| {
            read.reserve(buffer_bytes::<NodeId>(count)?)
        })?;
        let mut nodes = buffer(count)?;
        for index in 0..count {
            let value = arguments.get(index).ok_or(Error::BackendUnavailable)?;
            let node = match tree::projected_node(scope, value)? {
                Some(GuestNode::Element(node) | GuestNode::Text(node)) => node,
                _ => {
                    return Ok(Completion::Throw(
                        scope.type_error(&ascii(b"Element or Text required"))?,
                    ));
                }
            };
            self.callbacks.with_read(owner, events, |read, _| {
                read.node(node)?;
                read.step()
            })?;
            nodes.push(node);
        }
        scope.check_active()?;
        let input = self.callbacks.with_read(owner, events, |read, _| {
            DomSourceEdit::assign_slot_with(
                slot,
                &nodes,
                || read.step(),
                |bytes| Ok(read.reserve(bytes)?.into_source_input()),
            )
        })?;
        drop(nodes);
        drop(input_backing);
        self.apply_tree_edit(scope, owner, events, input, Completion::Undefined)
    }

    fn slottable_slot<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: SlottableReceiver,
    ) -> Result<Completion<'scope>, Error> {
        let node = tree::projected_node(scope, arguments.receiver())?;
        let Some(node) = node.filter(|node| receiver.accepts(*node)) else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let (_source, slot) = self.callbacks.with_read(&owner, &events, |read, _| {
                let slot = read.node(node.node())?.assigned_slot_with(|| read.step())?;
                read.step()?;
                Ok((read.snapshot.clone(), slot))
            })?;
            match slot {
                Some(slot) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, slot),
                None => Ok(scope.primitive(Primitive::Null)?),
            }
            .map(Completion::Return)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
