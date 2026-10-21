//! Kind-sized controlled transaction owners. Canonical child aliases retain the
//! original object control; only an edited group receives owned Key operands.
use super::super::*;
use super::{RelationField, Sequence};
use canonical::*;

pub(in crate::store) struct Controlled<T> {
    pub value: Arc<payload::Payload<T>>,
    pub custody: bank::ObjectCustody,
}
impl<T> Clone for Controlled<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            custody: self.custody.clone(),
        }
    }
}
impl<T> From<payload::Sealed<T>> for Controlled<T> {
    fn from(value: payload::Sealed<T>) -> Self {
        Self {
            value: value.value,
            custody: value.custody,
        }
    }
}
pub(in crate::store) enum Group<C, S> {
    Canonical(Controlled<C>),
    Changed(custody::Held<Box<S>>),
}
pub(in crate::store) enum GroupRead<'a, C, S> {
    Canonical(&'a C),
    Changed(&'a S),
}
impl<C, S> Group<C, S> {
    pub fn read(&self) -> GroupRead<'_, C, S> {
        match self {
            Self::Canonical(v) => GroupRead::Canonical(&v.value.value),
            Self::Changed(v) => GroupRead::Changed(v.value()),
        }
    }
}
pub(in crate::store) struct Children {
    pub values: Sequence<NodeId, Key>,
}
pub(in crate::store) struct SlotFields {
    pub manual: Sequence<NodeId, Key>,
    pub assigned: Sequence<NodeId, Key>,
}
pub(in crate::store) struct InteractiveFields {
    pub open: Sequence<NodeId, Key>,
    pub top: Sequence<NodeId, Key>,
    pub pending: Sequence<NodeId, Key>,
}
pub(in crate::store) enum InteractiveGroup {
    Absent,
    Unavailable,
    KnownEmpty,
    Known(Group<InteractiveValues<NodeId>, InteractiveFields>),
}
pub(in crate::store) struct AutofocusFields {
    pub candidates: Sequence<NodeId, Key>,
    pub processed: bool,
}
pub(in crate::store) enum AutofocusGroup {
    Absent,
    Unavailable,
    KnownEmpty { processed: bool },
    Known(Group<AutofocusValues<NodeId>, AutofocusFields>),
}
pub(in crate::store) enum MetadataCredit {
    Captured { _custody: bank::ObjectCustody },
    Owned { _debit: custody::Debit },
    None,
}
pub(in crate::store) struct BaseDocumentFields {
    pub facts: Option<surgeist_html_semantics::HtmlParseBaseFacts>,
    pub policy_basis: Option<crate::base::CspBasis>,
    pub first: Option<Key>,
    pub render: Sequence<NodeId, Key>,
    pub metadata_credit: MetadataCredit,
}
pub(in crate::store) enum BaseDocumentGroup {
    Absent,
    KnownEmpty,
    Known(Group<canonical::BaseDocument<NodeId>, BaseDocumentFields>),
}
pub(in crate::store) enum EncodingGroup {
    Absent,
    KnownEmpty,
    Known(Controlled<crate::hyperlink::EncodingUrlDocument>),
}
pub(in crate::store) struct FormFields {
    pub past_names: super::Names,
}
pub(in crate::store) struct ImageFields {
    pub image: Group<crate::image::ImageState<NodeId>, crate::image::ImageState<Key>>,
    pub control_owner: Option<Key>,
}
pub(in crate::store) enum Associations {
    None,
    FormEmpty,
    Template(Group<TemplateAssociation<NodeId>, TemplateAssociation<Key>>),
    Slot(Group<SlotAssignment<NodeId>, SlotFields>),
    Control(Group<ControlAssociation<NodeId>, ControlAssociation<Key>>),
    Form(Group<FormAssociation<NodeId>, FormFields>),
    Option(Group<OptionAssociation<NodeId>, OptionAssociation<Key>>),
    Script(Group<crate::script::ScriptState<NodeId>, crate::script::ScriptState<Key>>),
    Style(Group<StyleAssociation<NodeId>, StyleAssociation<Key>>),
    Base(Group<crate::base::BaseState<NodeId>, crate::base::BaseState<Key>>),
    Link(Group<crate::base::LinkState<NodeId>, crate::base::LinkState<Key>>),
    Image(Group<ImageAssociation<NodeId>, ImageFields>),
    Hyperlink(
        Group<crate::hyperlink::HyperlinkState<NodeId>, crate::hyperlink::HyperlinkState<Key>>,
    ),
}
pub(in crate::store) struct DocumentFields {
    pub common: Common<Key>,
    pub metadata: Controlled<DomDocumentMetadata>,
    pub children: Option<Group<ContainerChildren<NodeId>, Children>>,
    pub active_range: crate::DomActiveRangeState,
    pub focused_area_designation: crate::DomFocusedAreaDesignation,
    pub fullscreen: Option<
        Group<FullscreenQueue, Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent>>,
    >,
    pub template_document: Option<crate::template::TemplateDocument<Key>>,
    pub base: BaseDocumentGroup,
    pub encoding: EncodingGroup,
    pub interactive: InteractiveGroup,
    pub autofocus: AutofocusGroup,
}
pub(in crate::store) struct ElementFields {
    pub common: Common<Key>,
    pub element: Controlled<DomElement>,
    pub children: Option<Group<ContainerChildren<NodeId>, Children>>,
    pub slottable: Option<Group<Slottable<NodeId>, Slottable<Key>>>,
    pub shadow: Option<Group<ShadowHost<NodeId>, ShadowHost<Key>>>,
    pub associations: Associations,
    pub focus: FocusVersion,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::store) enum FocusVersion {
    Canonical,
    Staged(Key),
    Cleared,
}
pub(in crate::store) struct FragmentFields {
    pub common: Common<Key>,
    pub children: Option<Group<ContainerChildren<NodeId>, Children>>,
    pub host: Option<Key>,
}
pub(in crate::store) struct ShadowFields {
    pub common: Common<Key>,
    pub state: Controlled<crate::DomShadowRootState>,
    pub children: Option<Group<ContainerChildren<NodeId>, Children>>,
    pub host: Option<Key>,
}
pub(in crate::store) struct TextFields {
    pub common: Common<Key>,
    pub data: Controlled<CharacterData>,
    pub slottable: Option<Group<Slottable<NodeId>, Slottable<Key>>>,
}
pub(in crate::store) struct CommentFields {
    pub common: Common<Key>,
    pub data: Controlled<CharacterData>,
}
pub(in crate::store) struct ProcessingInstructionFields {
    pub common: Common<Key>,
    pub data: Controlled<crate::DomProcessingInstruction>,
}
pub(in crate::store) struct DocumentTypeFields {
    pub common: Common<Key>,
    pub data: Controlled<crate::DomDocumentType>,
}
pub(in crate::store) enum DraftNode {
    Document(custody::Held<Box<DocumentFields>>),
    Element(custody::Held<Box<ElementFields>>),
    DocumentFragment(custody::Held<Box<FragmentFields>>),
    ShadowRoot(custody::Held<Box<ShadowFields>>),
    Text(custody::Held<Box<TextFields>>),
    Comment(custody::Held<Box<CommentFields>>),
    ProcessingInstruction(custody::Held<Box<ProcessingInstructionFields>>),
    DocumentType(custody::Held<Box<DocumentTypeFields>>),
}
macro_rules! common {
    ($self:expr,$n:ident => $e:expr) => {
        match $self {
            DraftNode::Document($n) => $e,
            DraftNode::Element($n) => $e,
            DraftNode::DocumentFragment($n) => $e,
            DraftNode::ShadowRoot($n) => $e,
            DraftNode::Text($n) => $e,
            DraftNode::Comment($n) => $e,
            DraftNode::ProcessingInstruction($n) => $e,
            DraftNode::DocumentType($n) => $e,
        }
    };
}
impl DraftNode {
    pub fn common(&self) -> &Common<Key> {
        common!(self,n=>&n.value().common)
    }
    pub fn common_mut(&mut self) -> &mut Common<Key> {
        common!(self,n=>&mut n.value_mut().common)
    }
    pub fn element_fields(&self) -> Option<&ElementFields> {
        match self {
            Self::Element(v) => Some(v.value()),
            _ => None,
        }
    }
    pub fn element_fields_mut(&mut self) -> Option<&mut ElementFields> {
        match self {
            Self::Element(v) => Some(v.value_mut()),
            _ => None,
        }
    }
    pub fn document_fields(&self) -> Option<&DocumentFields> {
        match self {
            Self::Document(v) => Some(v.value()),
            _ => None,
        }
    }
    pub fn document_fields_mut(&mut self) -> Option<&mut DocumentFields> {
        match self {
            Self::Document(v) => Some(v.value_mut()),
            _ => None,
        }
    }
    pub fn children(&self) -> Option<&Group<ContainerChildren<NodeId>, Children>> {
        match self {
            Self::Document(n) => n.value().children.as_ref(),
            Self::Element(n) => n.value().children.as_ref(),
            Self::DocumentFragment(n) => n.value().children.as_ref(),
            Self::ShadowRoot(n) => n.value().children.as_ref(),
            _ => None,
        }
    }
    pub fn children_mut(
        &mut self,
    ) -> Option<&mut Option<Group<ContainerChildren<NodeId>, Children>>> {
        match self {
            Self::Document(n) => Some(&mut n.value_mut().children),
            Self::Element(n) => Some(&mut n.value_mut().children),
            Self::DocumentFragment(n) => Some(&mut n.value_mut().children),
            Self::ShadowRoot(n) => Some(&mut n.value_mut().children),
            _ => None,
        }
    }
    pub fn slottable(&self) -> Option<&Group<Slottable<NodeId>, Slottable<Key>>> {
        match self {
            Self::Element(n) => n.value().slottable.as_ref(),
            Self::Text(n) => n.value().slottable.as_ref(),
            _ => None,
        }
    }
    pub fn slottable_mut(
        &mut self,
    ) -> Option<&mut Option<Group<Slottable<NodeId>, Slottable<Key>>>> {
        match self {
            Self::Element(n) => Some(&mut n.value_mut().slottable),
            Self::Text(n) => Some(&mut n.value_mut().slottable),
            _ => None,
        }
    }
    pub fn associations(&self) -> Option<&Associations> {
        self.element_fields().map(|e| &e.associations)
    }
    pub fn associations_mut(&mut self) -> Option<&mut Associations> {
        self.element_fields_mut().map(|e| &mut e.associations)
    }
    pub fn relation(&self, field: RelationField) -> Option<&Sequence<NodeId, Key>> {
        match field {
            RelationField::Children => match self.children()?.read() {
                GroupRead::Changed(v) => Some(&v.values),
                _ => None,
            },
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                match self.associations()? {
                    Associations::Slot(Group::Changed(v)) => {
                        Some(if field == RelationField::ManualNodes {
                            &v.value().manual
                        } else {
                            &v.value().assigned
                        })
                    }
                    _ => None,
                }
            }
            RelationField::RenderBlocking => match &self.document_fields()?.base {
                BaseDocumentGroup::Known(Group::Changed(v)) => Some(&v.value().render),
                _ => None,
            },
            RelationField::AutofocusCandidates => match &self.document_fields()?.autofocus {
                AutofocusGroup::Known(Group::Changed(v)) => Some(&v.value().candidates),
                _ => None,
            },
            RelationField::OpenDialogs
            | RelationField::TopLayer
            | RelationField::PendingTopLayerRemovals => {
                match &self.document_fields()?.interactive {
                    InteractiveGroup::Known(Group::Changed(v)) => Some(match field {
                        RelationField::OpenDialogs => &v.value().open,
                        RelationField::TopLayer => &v.value().top,
                        _ => &v.value().pending,
                    }),
                    _ => None,
                }
            }
        }
    }
    pub fn relation_mut(&mut self, field: RelationField) -> &mut Sequence<NodeId, Key> {
        match field {
            RelationField::Children => match self
                .children_mut()
                .expect("container kind")
                .as_mut()
                .expect("prepared child group")
            {
                Group::Changed(v) => &mut v.value_mut().values,
                _ => unreachable!("selected group prepared"),
            },
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                match self.associations_mut().expect("slot element") {
                    Associations::Slot(Group::Changed(v)) => {
                        if field == RelationField::ManualNodes {
                            &mut v.value_mut().manual
                        } else {
                            &mut v.value_mut().assigned
                        }
                    }
                    _ => unreachable!("selected slot group prepared"),
                }
            }
            RelationField::RenderBlocking => {
                match &mut self.document_fields_mut().expect("Document").base {
                    BaseDocumentGroup::Known(Group::Changed(v)) => &mut v.value_mut().render,
                    _ => unreachable!("selected base group prepared"),
                }
            }
            RelationField::AutofocusCandidates => {
                match &mut self.document_fields_mut().expect("Document").autofocus {
                    AutofocusGroup::Known(Group::Changed(v)) => &mut v.value_mut().candidates,
                    _ => unreachable!("selected autofocus group prepared"),
                }
            }
            RelationField::OpenDialogs
            | RelationField::TopLayer
            | RelationField::PendingTopLayerRemovals => {
                match &mut self.document_fields_mut().expect("Document").interactive {
                    InteractiveGroup::Known(Group::Changed(v)) => match field {
                        RelationField::OpenDialogs => &mut v.value_mut().open,
                        RelationField::TopLayer => &mut v.value_mut().top,
                        _ => &mut v.value_mut().pending,
                    },
                    _ => unreachable!("selected interaction group prepared"),
                }
            }
        }
    }
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        common!(self,n=>n.debit().usage().bytes())
    }
}

pub(super) fn controlled<T>(
    value: &Arc<payload::Payload<T>>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<Controlled<T>, DomError> {
    let custody = registry.adopt_object(value.identity, value.own_usage, admit)?;
    Ok(Controlled {
        value: value.clone(),
        custody,
    })
}
pub(super) fn captured<C, S>(
    value: &Arc<payload::Payload<C>>,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<Group<C, S>, DomError> {
    controlled(value, registry, admit).map(Group::Canonical)
}
pub(super) fn boxed<T>(
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    build: impl FnOnce() -> Result<T, DomError>,
) -> Result<custody::Held<Box<T>>, DomError> {
    let debit = registry
        .ledger()
        .retain(custody::Usage::new(size_of::<T>(), 0), &mut *admit)?;
    let value = Box::new(build()?);
    #[cfg(test)]
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::RecordMaterialized {
            bytes: size_of::<T>(),
        });
    }
    Ok(custody::Held::new(value, debit))
}
fn common(value: &Common<NodeId>) -> Common<Key> {
    Common {
        document: Key::Existing(value.document),
        parent: value.parent.map(Key::Existing),
        relevant_global: value.relevant_global,
    }
}
impl DraftNode {
    pub(super) fn capture(
        node: &Node,
        registry: &bank::CustodyRegistry,
        _names: &name_registry::Operation,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        Ok(match node {
            Node::Document(n) => {
                let value = &n.value;
                let metadata = controlled(&value.metadata, registry, &mut admit)?;
                let children = value
                    .children
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let fullscreen = value
                    .fullscreen_events
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let base = match &value.base_document {
                    BaseDocumentState::Absent => BaseDocumentGroup::Absent,
                    BaseDocumentState::KnownEmpty => BaseDocumentGroup::KnownEmpty,
                    BaseDocumentState::Known(v) => {
                        BaseDocumentGroup::Known(captured(v, registry, &mut admit)?)
                    }
                };
                let encoding = match &value.encoding_url_document {
                    EncodingUrl::Absent => EncodingGroup::Absent,
                    EncodingUrl::KnownEmpty => EncodingGroup::KnownEmpty,
                    EncodingUrl::Known(v) => {
                        EncodingGroup::Known(controlled(v, registry, &mut admit)?)
                    }
                };
                let interactive = match &value.interactive_document {
                    Interactive::Absent => InteractiveGroup::Absent,
                    Interactive::Unavailable => InteractiveGroup::Unavailable,
                    Interactive::KnownEmpty => InteractiveGroup::KnownEmpty,
                    Interactive::Known(v) => {
                        InteractiveGroup::Known(captured(v, registry, &mut admit)?)
                    }
                };
                let autofocus = match &value.autofocus_document {
                    Autofocus::Absent => AutofocusGroup::Absent,
                    Autofocus::Unavailable => AutofocusGroup::Unavailable,
                    Autofocus::KnownEmpty { processed } => AutofocusGroup::KnownEmpty {
                        processed: *processed,
                    },
                    Autofocus::Known(v) => {
                        AutofocusGroup::Known(captured(v, registry, &mut admit)?)
                    }
                };
                Self::Document(boxed(registry, &mut admit, || {
                    Ok(DocumentFields {
                        common: common(&value.common),
                        metadata,
                        children,
                        active_range: value.active_range,
                        focused_area_designation: value.focused_area_designation,
                        fullscreen,
                        template_document: value.template_document.map(|v| v.map(Key::Existing)),
                        base,
                        encoding,
                        interactive,
                        autofocus,
                    })
                })?)
            }
            Node::Element(n) => {
                let value = &n.value;
                let element = controlled(&value.element, registry, &mut admit)?;
                let children = value
                    .children
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let slottable = value
                    .slottable
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let shadow = value
                    .shadow
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let associations = match &value.associations {
                    ElementAssociations::None => Associations::None,
                    ElementAssociations::FormEmpty(_) => Associations::FormEmpty,
                    ElementAssociations::Template(v) => {
                        Associations::Template(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Slot(v) => {
                        Associations::Slot(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Control(v) => {
                        Associations::Control(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Form(v) => {
                        Associations::Form(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Option(v) => {
                        Associations::Option(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Script(v) => {
                        Associations::Script(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Style(v) => {
                        Associations::Style(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Base(v) => {
                        Associations::Base(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Link(v) => {
                        Associations::Link(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Image(v) => {
                        Associations::Image(captured(v, registry, &mut admit)?)
                    }
                    ElementAssociations::Hyperlink(v) => {
                        Associations::Hyperlink(captured(v, registry, &mut admit)?)
                    }
                };
                Self::Element(boxed(registry, &mut admit, || {
                    Ok(ElementFields {
                        common: common(&value.common),
                        element,
                        children,
                        slottable,
                        shadow,
                        associations,
                        focus: FocusVersion::Canonical,
                    })
                })?)
            }
            Node::DocumentFragment(n) => {
                let children = n
                    .value
                    .children
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                Self::DocumentFragment(boxed(registry, &mut admit, || {
                    Ok(FragmentFields {
                        common: common(&n.value.common),
                        children,
                        host: n.value.host.map(Key::Existing),
                    })
                })?)
            }
            Node::ShadowRoot(n) => {
                let children = n
                    .value
                    .children
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                let state = controlled(&n.value.state, registry, &mut admit)?;
                Self::ShadowRoot(boxed(registry, &mut admit, || {
                    Ok(ShadowFields {
                        common: common(&n.value.common),
                        children,
                        state,
                        host: n.value.host.map(Key::Existing),
                    })
                })?)
            }
            Node::Text(n) => {
                let data = controlled(&n.value.data, registry, &mut admit)?;
                let slottable = n
                    .value
                    .slottable
                    .as_ref()
                    .map(|v| captured(v, registry, &mut admit))
                    .transpose()?;
                Self::Text(boxed(registry, &mut admit, || {
                    Ok(TextFields {
                        common: common(&n.value.common),
                        data,
                        slottable,
                    })
                })?)
            }
            Node::Comment(n) => {
                let data = controlled(&n.value.data, registry, &mut admit)?;
                Self::Comment(boxed(registry, &mut admit, || {
                    Ok(CommentFields {
                        common: common(&n.value.common),
                        data,
                    })
                })?)
            }
            Node::ProcessingInstruction(n) => {
                let data = controlled(&n.value.data, registry, &mut admit)?;
                Self::ProcessingInstruction(boxed(registry, &mut admit, || {
                    Ok(ProcessingInstructionFields {
                        common: common(&n.value.common),
                        data,
                    })
                })?)
            }
            Node::DocumentType(n) => {
                let data = controlled(&n.value.data, registry, &mut admit)?;
                Self::DocumentType(boxed(registry, &mut admit, || {
                    Ok(DocumentTypeFields {
                        common: common(&n.value.common),
                        data,
                    })
                })?)
            }
        })
    }
}
impl DraftNode {
    pub fn fullscreen_sequence(
        &self,
    ) -> &Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent> {
        match &self
            .document_fields()
            .expect("Document sequence")
            .fullscreen
        {
            Some(Group::Changed(value)) => value.value(),
            _ => unreachable!("prepared fullscreen sequence"),
        }
    }
    pub fn fullscreen_sequence_mut(
        &mut self,
    ) -> &mut Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent> {
        match &mut self
            .document_fields_mut()
            .expect("Document sequence")
            .fullscreen
        {
            Some(Group::Changed(value)) => value.value_mut(),
            _ => unreachable!("prepared fullscreen sequence"),
        }
    }
}

pub(super) fn fresh_language<T>(
    mut input: custody::Held<NewNode>,
    retained: bool,
    usage: custody::Usage,
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    children: Option<bank::Dependency>,
    extract: impl FnOnce(NewNode) -> T,
) -> Result<Controlled<T>, DomError> {
    let reserved = payload::Reserved::<T>::new()?;
    let original = if retained {
        usage.checked_sub(custody::Usage::new(size_of::<payload::Payload<T>>(), 0))?
    } else {
        custody::Usage::ZERO
    };
    let debit = if original == custody::Usage::ZERO {
        registry
            .ledger()
            .retain(custody::Usage::ZERO, &mut *admit)?
    } else {
        input.debit_mut().split(original)?
    };
    let (kind, excess) = input.into_parts();
    // The original fields have moved into this data-first guard before any
    // remaining admission or control construction can refuse.
    let incoming = custody::Held::new(extract(kind), debit);
    let result = reserved
        .materialize(incoming, usage, &mut *admit)?
        .seal_with_children(registry, children, &mut *admit)?;
    drop(excess);
    Ok(result.into())
}
fn fresh_group<C, S>(
    registry: &bank::CustodyRegistry,
    admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    build: impl FnOnce() -> Result<S, DomError>,
) -> Result<Group<C, S>, DomError> {
    boxed(registry, admit, build).map(Group::Changed)
}
impl DraftNode {
    pub(in crate::store) fn fresh(
        input: custody::Held<NewNode>,
        retained_occurrence: bool,
        common: Common<Key>,
        key: Key,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        use source::storage;
        let registry = draft.custody_registry();
        let names = &draft.names;
        let mut admit = |usage| draft.admit_custody_usage(usage, cancel);
        let mut step = || draft.source_step(cancel);
        Ok(match input.value() {
            NewNode::Document(metadata) => {
                let native = matches!(
                    metadata.creation_cause(),
                    crate::DomDocumentCreationCause::Native
                        | crate::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
                );
                let usage = storage::document_metadata(metadata)?;
                let metadata = fresh_language(
                    input,
                    retained_occurrence,
                    usage,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::Document(v) = v else {
                            unreachable!("Document input")
                        };
                        v
                    },
                )?;
                Self::Document(boxed(registry, &mut admit, || {
                    Ok(DocumentFields {
                        common,
                        metadata,
                        children: None,
                        active_range: if native {
                            crate::DomActiveRangeState::Known(None)
                        } else {
                            crate::DomActiveRangeState::Unavailable
                        },
                        // This is a one-time actual constructor qualification,
                        // copied thereafter even when activity/metadata changes.
                        focused_area_designation: if native {
                            crate::DomFocusedAreaDesignation::KnownNull
                        } else {
                            crate::DomFocusedAreaDesignation::Unavailable
                        },
                        fullscreen: None,
                        template_document: Some(crate::template::TemplateDocument::Ordinary {
                            inert: None,
                        }),
                        base: BaseDocumentGroup::KnownEmpty,
                        encoding: EncodingGroup::KnownEmpty,
                        interactive: if native {
                            InteractiveGroup::KnownEmpty
                        } else {
                            InteractiveGroup::Unavailable
                        },
                        autofocus: if native {
                            AutofocusGroup::KnownEmpty { processed: false }
                        } else {
                            AutofocusGroup::Unavailable
                        },
                    })
                })?)
            }
            NewNode::Element(element) => {
                let children =
                    payload::element_dependency(element, registry, Some(names), &mut admit)?;
                let usage = storage::typed::<DomElement>(storage::Payload::default())?;
                let element =
                    fresh_language(input, false, usage, registry, &mut admit, children, |v| {
                        let NewNode::Element(v) = v else {
                            unreachable!("Element input")
                        };
                        v
                    })?;
                let e = &element.value.value;
                let associations = if e.is_img() {
                    // Fresh standalone Image owns one genuine empty URL target.
                    let debit = registry.ledger().retain(
                        custody::Usage::new(
                            size_of::<crate::image::ImageState<Key>>() + size_of::<Vec<u16>>(),
                            0,
                        ),
                        &mut admit,
                    )?;
                    let image =
                        custody::Held::new(Box::new(crate::image::ImageState::fresh(key)), debit);
                    fresh_group(registry, &mut admit, || {
                        Ok(ImageFields {
                            image: Group::Changed(image),
                            control_owner: None,
                        })
                    })
                    .map(Associations::Image)?
                } else if e.is_html_script() {
                    Associations::Script(fresh_group(registry, &mut admit, || {
                        Ok(crate::script::ScriptState::fresh())
                    })?)
                } else if e.style_state().is_some() {
                    Associations::Style(fresh_group(registry, &mut admit, || {
                        Ok(StyleAssociation {
                            parser_document: None,
                        })
                    })?)
                } else if e.is_base() {
                    Associations::Base(fresh_group(registry, &mut admit, || {
                        Ok(crate::base::BaseState::fresh())
                    })?)
                } else if e.is_link() {
                    Associations::Link(fresh_group(registry, &mut admit, || {
                        Ok(crate::base::LinkState::fresh())
                    })?)
                } else if e.is_hyperlink() {
                    Associations::Hyperlink(fresh_group(registry, &mut admit, || {
                        Ok(crate::hyperlink::HyperlinkState::fresh())
                    })?)
                } else if e.form_state().is_some() {
                    Associations::FormEmpty
                } else if e.option_state().is_some() {
                    Associations::Option(fresh_group(registry, &mut admit, || {
                        Ok(OptionAssociation {
                            option_select: None,
                        })
                    })?)
                } else if e.native_control().is_some() {
                    let reflected_targets = if e.button_state().is_some() {
                        Some(crate::reflected::ReflectedTargets::button())
                    } else if e.input_state().is_some() {
                        Some(crate::reflected::ReflectedTargets::input())
                    } else {
                        None
                    };
                    Associations::Control(fresh_group(registry, &mut admit, || {
                        Ok(ControlAssociation {
                            control_owner: None,
                            reflected_targets,
                        })
                    })?)
                } else {
                    Associations::None
                };
                Self::Element(boxed(registry, &mut admit, || {
                    Ok(ElementFields {
                        common,
                        element,
                        children: None,
                        slottable: None,
                        shadow: None,
                        associations,
                        focus: FocusVersion::Canonical,
                    })
                })?)
            }
            NewNode::DocumentFragment => {
                // No variable language data belongs to this exact kind.
                drop(input);
                Self::DocumentFragment(boxed(registry, &mut admit, || {
                    Ok(FragmentFields {
                        common,
                        children: None,
                        host: None,
                    })
                })?)
            }
            NewNode::ShadowRoot(_) => {
                let state = fresh_language(
                    input,
                    false,
                    storage::typed::<crate::DomShadowRootState>(storage::Payload::default())?,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::ShadowRoot(v) = v else {
                            unreachable!("Shadow input")
                        };
                        v
                    },
                )?;
                Self::ShadowRoot(boxed(registry, &mut admit, || {
                    Ok(ShadowFields {
                        common,
                        state,
                        children: None,
                        host: None,
                    })
                })?)
            }
            NewNode::Text(value) => {
                let mut extra = storage::Payload::default();
                extra.string(value)?;
                let data = fresh_language(
                    input,
                    true,
                    storage::typed::<CharacterData>(extra)?,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::Text(value) = v else {
                            unreachable!("Text input")
                        };
                        CharacterData { value }
                    },
                )?;
                Self::Text(boxed(registry, &mut admit, || {
                    Ok(TextFields {
                        common,
                        data,
                        slottable: None,
                    })
                })?)
            }
            NewNode::Comment(value) => {
                let mut extra = storage::Payload::default();
                extra.string(value)?;
                let data = fresh_language(
                    input,
                    retained_occurrence,
                    storage::typed::<CharacterData>(extra)?,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::Comment(value) = v else {
                            unreachable!("Comment input")
                        };
                        CharacterData { value }
                    },
                )?;
                Self::Comment(boxed(registry, &mut admit, || {
                    Ok(CommentFields { common, data })
                })?)
            }
            NewNode::ProcessingInstruction(value) => {
                let usage = storage::processing_instruction(value, &mut step)?;
                let data = fresh_language(
                    input,
                    retained_occurrence,
                    usage,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::ProcessingInstruction(v) = v else {
                            unreachable!("PI input")
                        };
                        v
                    },
                )?;
                Self::ProcessingInstruction(boxed(registry, &mut admit, || {
                    Ok(ProcessingInstructionFields { common, data })
                })?)
            }
            NewNode::DocumentType(value) => {
                let usage = storage::document_type(value)?;
                let data = fresh_language(
                    input,
                    retained_occurrence,
                    usage,
                    registry,
                    &mut admit,
                    None,
                    |v| {
                        let NewNode::DocumentType(v) = v else {
                            unreachable!("Doctype input")
                        };
                        v
                    },
                )?;
                Self::DocumentType(boxed(registry, &mut admit, || {
                    Ok(DocumentTypeFields { common, data })
                })?)
            }
        })
    }
}
