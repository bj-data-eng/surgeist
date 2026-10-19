//! Controlled edges follow owned typed payloads, never semantic NodeId links.
use super::*;
use crate::store::{Node, canonical};

pub(super) struct DocumentChildren {
    _metadata: bank::ObjectCustody,
    _children: Option<bank::ObjectCustody>,
    _fullscreen: Option<bank::ObjectCustody>,
    _base: Option<bank::ObjectCustody>,
    _encoding: Option<bank::ObjectCustody>,
    _interactive: Option<bank::ObjectCustody>,
    _autofocus: Option<bank::ObjectCustody>,
}
pub(super) struct ElementRootChildren {
    pub(super) _element: bank::ObjectCustody,
    _children: Option<bank::ObjectCustody>,
    _slottable: Option<bank::ObjectCustody>,
    _shadow: Option<bank::ObjectCustody>,
    pub(super) _association: Option<bank::ObjectCustody>,
}
pub(super) struct FragmentChildren {
    _children: bank::ObjectCustody,
}
pub(super) struct ShadowChildren {
    _state: bank::ObjectCustody,
    _children: Option<bank::ObjectCustody>,
}
pub(super) struct TextChildren {
    _data: bank::ObjectCustody,
    _slottable: Option<bank::ObjectCustody>,
}
pub(super) struct DataChild {
    pub(super) _data: bank::ObjectCustody,
}
pub(super) struct FormChildren {
    pub(super) _past_names: bank::ObjectCustody,
}

pub(super) fn leaf<T>(
    value: &Arc<Payload<T>>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<bank::ObjectCustody, DomError> {
    registry.adopt_object(value.identity, value.own_usage, admit)
}

fn optional_leaf<T>(
    value: Option<&Arc<Payload<T>>>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::ObjectCustody>, DomError> {
    value.map(|value| leaf(value, registry, admit)).transpose()
}

fn association(
    value: &canonical::ElementAssociations<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::ObjectCustody>, DomError> {
    use canonical::ElementAssociations as Association;
    Ok(Some(match value {
        Association::None | Association::FormEmpty(_) => return Ok(None),
        Association::Form(value) => registry.adopt_object_with_children(
            value.identity,
            value.own_usage,
            |admit| form_dependency(&value.value, registry, admit),
            admit,
        )?,
        Association::Template(value) => leaf(value, registry, admit)?,
        Association::Slot(value) => leaf(value, registry, admit)?,
        Association::Control(value) => leaf(value, registry, admit)?,
        Association::Option(value) => leaf(value, registry, admit)?,
        Association::Script(value) => leaf(value, registry, admit)?,
        Association::Style(value) => leaf(value, registry, admit)?,
        Association::Base(value) => leaf(value, registry, admit)?,
        Association::Link(value) => leaf(value, registry, admit)?,
        Association::Image(value) => registry.adopt_object_with_children(
            value.identity,
            value.own_usage,
            |admit| image_association_dependency(&value.value, registry, admit),
            admit,
        )?,
        Association::Hyperlink(value) => leaf(value, registry, admit)?,
    }))
}

pub(in crate::store) fn form_dependency<K>(
    value: &canonical::FormAssociation<K>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let Some(history) = value.past_names.payload() else {
        return Ok(None);
    };
    let past_names = leaf(history, registry, admit)?;
    #[cfg(test)]
    form_dependency_refusal::before_capsule(history, &past_names, registry.ledger())?;
    bank::Dependency::new(
        FormChildren {
            _past_names: past_names,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn image_association_dependency<K>(
    value: &canonical::ImageAssociation<K>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    data_dependency(&value.image, registry, admit)
}

pub(in crate::store) fn document_dependency(
    value: &canonical::DocumentNode<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let metadata = leaf(&value.metadata, registry, admit)?;
    let children = optional_leaf(value.children.as_ref(), registry, admit)?;
    let fullscreen = optional_leaf(value.fullscreen_events.as_ref(), registry, admit)?;
    let base = match &value.base_document {
        canonical::BaseDocumentState::Known(value) => Some(leaf(value, registry, admit)?),
        canonical::BaseDocumentState::Absent | canonical::BaseDocumentState::KnownEmpty => None,
    };
    let encoding = match &value.encoding_url_document {
        canonical::EncodingUrl::Known(value) => Some(leaf(value, registry, admit)?),
        canonical::EncodingUrl::Absent | canonical::EncodingUrl::KnownEmpty => None,
    };
    let interactive = match &value.interactive_document {
        canonical::Interactive::Known(value) => Some(leaf(value, registry, admit)?),
        canonical::Interactive::Absent
        | canonical::Interactive::Unavailable
        | canonical::Interactive::KnownEmpty => None,
    };
    let autofocus = match &value.autofocus_document {
        canonical::Autofocus::Known(value) => Some(leaf(value, registry, admit)?),
        canonical::Autofocus::Absent
        | canonical::Autofocus::Unavailable
        | canonical::Autofocus::KnownEmpty { .. } => None,
    };
    bank::Dependency::new(
        DocumentChildren {
            _metadata: metadata,
            _children: children,
            _fullscreen: fullscreen,
            _base: base,
            _encoding: encoding,
            _interactive: interactive,
            _autofocus: autofocus,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn element_node_dependency(
    value: &canonical::ElementNode<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    names: Option<&super::super::name_registry::Operation>,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let element = registry.adopt_object_with_children(
        value.element.identity,
        value.element.own_usage,
        |admit| element_dependency(&value.element.value, registry, names, admit),
        &mut *admit,
    )?;
    let children = optional_leaf(value.children.as_ref(), registry, admit)?;
    let slottable = optional_leaf(value.slottable.as_ref(), registry, admit)?;
    let shadow = optional_leaf(value.shadow.as_ref(), registry, admit)?;
    let association = association(&value.associations, registry, admit)?;
    bank::Dependency::new(
        ElementRootChildren {
            _element: element,
            _children: children,
            _slottable: slottable,
            _shadow: shadow,
            _association: association,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn fragment_dependency(
    value: &canonical::FragmentNode<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let Some(children) = optional_leaf(value.children.as_ref(), registry, admit)? else {
        return Ok(None);
    };
    bank::Dependency::new(
        FragmentChildren {
            _children: children,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn shadow_dependency(
    value: &canonical::ShadowNode<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let state = leaf(&value.state, registry, admit)?;
    let children = optional_leaf(value.children.as_ref(), registry, admit)?;
    bank::Dependency::new(
        ShadowChildren {
            _state: state,
            _children: children,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn text_dependency(
    value: &canonical::TextNode<crate::NodeId>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let data = leaf(&value.data, registry, admit)?;
    let slottable = optional_leaf(value.slottable.as_ref(), registry, admit)?;
    bank::Dependency::new(
        TextChildren {
            _data: data,
            _slottable: slottable,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(super) fn node_children(
    node: &Node,
    registry: &bank::CustodyRegistry,
    names: Option<&super::super::name_registry::Operation>,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    match node {
        Node::Document(value) => document_dependency(&value.value, registry, admit),
        Node::Element(value) => element_node_dependency(&value.value, registry, names, admit),
        Node::DocumentFragment(value) => fragment_dependency(&value.value, registry, admit),
        Node::ShadowRoot(value) => shadow_dependency(&value.value, registry, admit),
        Node::Text(value) => text_dependency(&value.value, registry, admit),
        Node::Comment(value) => data_dependency(&value.value.data, registry, admit),
        Node::ProcessingInstruction(value) => data_dependency(&value.value.data, registry, admit),
        Node::DocumentType(value) => data_dependency(&value.value.data, registry, admit),
    }
}
pub(in crate::store) fn data_dependency<T>(
    value: &Arc<Payload<T>>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let data = leaf(value, registry, admit)?;
    bank::Dependency::new(DataChild { _data: data }, registry.ledger(), admit).map(Some)
}

/// The Slot directly owns one of these actual, kind-specific child capsules.
/// This closed test view rejects unknown capsules instead of silently dropping
/// an unrecognized dependency from the claimed graph.
#[cfg(test)]
pub(super) enum SlotDependency {
    Document(bank::Retained<DocumentChildren>),
    Element(bank::Retained<ElementRootChildren>),
    Fragment(bank::Retained<FragmentChildren>),
    Shadow(bank::Retained<ShadowChildren>),
    Text(bank::Retained<TextChildren>),
    Data(bank::Retained<DataChild>),
}
#[cfg(test)]
pub(super) fn inspect_slot_dependencies(
    slots: &bank::BankCustody<super::super::Slot>,
) -> Vec<SlotDependency> {
    slots
        .inspect_record_dependencies()
        .into_iter()
        .map(|value| {
            if let Some(value) = value.try_get::<DocumentChildren>() {
                return SlotDependency::Document(value);
            }
            if let Some(value) = value.try_get::<ElementRootChildren>() {
                return SlotDependency::Element(value);
            }
            if let Some(value) = value.try_get::<FragmentChildren>() {
                return SlotDependency::Fragment(value);
            }
            if let Some(value) = value.try_get::<ShadowChildren>() {
                return SlotDependency::Shadow(value);
            }
            if let Some(value) = value.try_get::<TextChildren>() {
                return SlotDependency::Text(value);
            }
            if let Some(value) = value.try_get::<DataChild>() {
                return SlotDependency::Data(value);
            }
            panic!("Slot has one recognized exact-kind child capsule");
        })
        .collect()
}

/// Claimed side only: walk the actual strong capsule graph, independently of
/// semantic Node contents. The raw oracle separately traverses stored values.
#[cfg(test)]
pub(in crate::store) fn inspect_slot_typed_custody(
    slots: &bank::BankCustody<super::super::Slot>,
) -> std::collections::BTreeMap<custody::MaterializationId, Usage> {
    fn names(
        value: &NameChildren,
        out: &mut std::collections::BTreeMap<custody::MaterializationId, Usage>,
    ) {
        if let Some(namespace) = &value._namespace {
            visit(namespace, out);
        }
        visit(&value._local, out);
    }
    fn visit(
        value: &bank::ObjectCustody,
        out: &mut std::collections::BTreeMap<custody::MaterializationId, Usage>,
    ) {
        if let Some(old) = out.insert(value.identity(), value.inspect_payload_usage()) {
            assert_eq!(old, value.inspect_payload_usage());
            return;
        }
        if let Some(d) = value.inspect_dependency::<DataChild>() {
            visit(&d._data, out);
        }
        if let Some(d) = value.inspect_dependency::<FormChildren>() {
            visit(&d._past_names, out);
        }
        if let Some(d) = value.inspect_dependency::<ElementLanguageChildren>() {
            visit(&d._name, out);
            for child in [
                &d._attributes,
                &d._native,
                &d._custom,
                &d._nonce,
                &d._focus,
                &d._hosted,
            ]
            .into_iter()
            .flatten()
            {
                visit(child, out);
            }
        }
        if let Some(d) = value.inspect_dependency::<ElementNameChildren>() {
            names(&d._names, out);
        }
        if let Some(d) = value.inspect_dependency::<AttributeBindingChildren>()
            && let Some(child) = &d._bank
        {
            visit(child, out);
        }
        if let Some(d) = value.inspect_dependency::<AttributeChildren>() {
            for child in &d._names {
                names(child, out);
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    for (identity, usage) in slots.inspect_materializations() {
        if let Some(previous) = out.insert(identity, usage) {
            assert_eq!(previous, usage);
        }
    }
    for dependency in inspect_slot_dependencies(slots) {
        match dependency {
            SlotDependency::Document(d) => {
                visit(&d._metadata, &mut out);
                for child in [
                    &d._children,
                    &d._fullscreen,
                    &d._base,
                    &d._encoding,
                    &d._interactive,
                    &d._autofocus,
                ]
                .into_iter()
                .flatten()
                {
                    visit(child, &mut out);
                }
            }
            SlotDependency::Element(d) => {
                visit(&d._element, &mut out);
                for child in [&d._children, &d._slottable, &d._shadow, &d._association]
                    .into_iter()
                    .flatten()
                {
                    visit(child, &mut out);
                }
            }
            SlotDependency::Fragment(d) => visit(&d._children, &mut out),
            SlotDependency::Shadow(d) => {
                visit(&d._state, &mut out);
                if let Some(child) = &d._children {
                    visit(child, &mut out);
                }
            }
            SlotDependency::Text(d) => {
                visit(&d._data, &mut out);
                if let Some(child) = &d._slottable {
                    visit(child, &mut out);
                }
            }
            SlotDependency::Data(d) => visit(&d._data, &mut out),
        }
    }
    out
}

/// One isolated refusal at the actual fresh Form child-capsule boundary. This
/// does not mutate the process-wide identity issuer or any admission policy.
#[cfg(test)]
pub(in crate::store) mod form_dependency_refusal {
    use super::*;
    use std::{cell::RefCell, marker::PhantomData, rc::Rc};

    struct Reached {
        identity: custody::MaterializationId,
        capacity: usize,
        destruction: super::super::destruction_observation::Observation,
    }
    struct Watch {
        reached: Option<Reached>,
    }
    thread_local! {
        static WATCH: RefCell<Option<Rc<RefCell<Watch>>>> = const { RefCell::new(None) };
    }
    pub(in crate::store) struct Refusal {
        watch: Rc<RefCell<Watch>>,
        _thread: PhantomData<Rc<()>>,
    }
    impl Refusal {
        pub(in crate::store) fn begin() -> Self {
            let watch = Rc::new(RefCell::new(Watch { reached: None }));
            WATCH.with(|slot| {
                let mut slot = slot.borrow_mut();
                assert!(slot.is_none(), "one isolated Form dependency refusal");
                *slot = Some(watch.clone());
            });
            Self {
                watch,
                _thread: PhantomData,
            }
        }
        pub(in crate::store) fn reached(&self) -> Option<(custody::MaterializationId, usize)> {
            self.watch
                .borrow()
                .reached
                .as_ref()
                .map(|value| (value.identity, value.capacity))
        }
        pub(in crate::store) fn destroyed_at(&self) -> Option<Usage> {
            self.watch
                .borrow()
                .reached
                .as_ref()
                .and_then(|value| value.destruction.observed())
        }
        pub(in crate::store) fn control_alive_at_destruction(&self) -> Option<bool> {
            self.watch
                .borrow()
                .reached
                .as_ref()
                .and_then(|value| value.destruction.control_alive_at_destruction())
        }
    }
    impl Drop for Refusal {
        fn drop(&mut self) {
            WATCH.with(|slot| *slot.borrow_mut() = None);
        }
    }
    pub(super) fn before_capsule<K>(
        history: &Arc<Payload<crate::form::History<K>>>,
        control: &bank::ObjectCustody,
        ledger: &custody::Ledger,
    ) -> Result<(), DomError> {
        WATCH.with(|slot| {
            let slot = slot.borrow();
            let Some(watch) = slot.as_ref() else {
                return Ok(());
            };
            let mut watch = watch.borrow_mut();
            if watch.reached.is_some() {
                return Ok(());
            }
            watch.reached = Some(Reached {
                identity: history.identity,
                capacity: history.value.entries.capacity(),
                destruction: super::super::destruction_observation::Observation::begin_with_control(
                    history.identity,
                    ledger,
                    control.inspect_liveness(),
                ),
            });
            Err(DomError::Cancelled)
        })
    }
}
