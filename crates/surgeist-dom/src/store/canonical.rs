//! Exact-kind canonical roots. Common topology stays inline while independently
//! edited language and relation groups retain their own immutable identity.
use super::*;
use crate::store::payload::Payload;
use surgeist_html_semantics::HtmlParseBaseFacts;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Common<K> {
    pub document: K,
    pub relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    pub parent: Option<K>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContainerChildren<K> {
    pub values: SharedVec<K>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CharacterData {
    pub value: DocumentString,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Slottable<K> {
    pub manual_slot: Option<K>,
    pub assigned_slot: Option<K>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowHost<K> {
    pub shadow_root: K,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TemplateAssociation<K> {
    pub template_contents: Option<K>,
    pub template_patch: Option<crate::template::TemplatePatch<K>>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SlotAssignment<K> {
    pub manual_nodes: SharedVec<K>,
    pub assigned_nodes: SharedVec<K>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ControlAssociation<K> {
    pub control_owner: Option<K>,
    pub reflected_targets: Option<crate::reflected::ReflectedTargets<K>>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormAssociation<K> {
    pub past_names: crate::form::FormPastNames<K>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OptionAssociation<K> {
    pub option_select: Option<K>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StyleAssociation<K> {
    pub parser_document: Option<K>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ImageAssociation<K> {
    pub image: Arc<Payload<crate::image::ImageState<K>>>,
    pub control_owner: Option<K>,
}
#[derive(Clone, Debug)]
pub(crate) enum ElementAssociations<K> {
    None,
    FormEmpty(crate::form::FormPastNames<K>),
    Template(Arc<Payload<TemplateAssociation<K>>>),
    Slot(Arc<Payload<SlotAssignment<K>>>),
    Control(Arc<Payload<ControlAssociation<K>>>),
    Form(Arc<Payload<FormAssociation<K>>>),
    Option(Arc<Payload<OptionAssociation<K>>>),
    Script(Arc<Payload<crate::script::ScriptState<K>>>),
    Style(Arc<Payload<StyleAssociation<K>>>),
    Base(Arc<Payload<crate::base::BaseState<K>>>),
    Link(Arc<Payload<crate::base::LinkState<K>>>),
    Image(Arc<Payload<ImageAssociation<K>>>),
    Hyperlink(Arc<Payload<crate::hyperlink::HyperlinkState<K>>>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InteractiveValues<K> {
    pub open_dialogs: SharedVec<K>,
    pub top_layer: SharedVec<K>,
    pub pending_top_layer_removals: SharedVec<K>,
}
#[derive(Clone, Debug)]
pub(crate) enum Interactive<K> {
    Absent,
    Unavailable,
    KnownEmpty,
    Known(Arc<Payload<InteractiveValues<K>>>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AutofocusValues<K> {
    pub candidates: SharedVec<K>,
    pub processed: bool,
}
#[derive(Clone, Debug)]
pub(crate) enum Autofocus<K> {
    Absent,
    Unavailable,
    KnownEmpty { processed: bool },
    Known(Arc<Payload<AutofocusValues<K>>>),
}
#[derive(Clone, Debug)]
pub(crate) struct BaseDocument<K> {
    pub facts: Option<HtmlParseBaseFacts>,
    pub first: Option<K>,
    pub render_blocking: SharedVec<K>,
}
impl<K: PartialEq> PartialEq for BaseDocument<K> {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first
            && self.render_blocking == other.render_blocking
            && match (&self.facts, &other.facts) {
                (None, None) => true,
                (Some(a), Some(b)) => a.same_owner(b),
                _ => false,
            }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum BaseDocumentState<K> {
    Absent,
    KnownEmpty,
    Known(Arc<Payload<BaseDocument<K>>>),
}
#[derive(Clone, Debug)]
pub(crate) enum EncodingUrl {
    Absent,
    KnownEmpty,
    Known(Arc<Payload<crate::hyperlink::EncodingUrlDocument>>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FullscreenQueue {
    pub values: SharedVec<crate::DomFullscreenEvent>,
}
#[derive(Clone, Debug)]
pub(crate) struct DocumentNode<K> {
    pub common: Common<K>,
    pub metadata: Arc<Payload<DomDocumentMetadata>>,
    pub children: Option<Arc<Payload<ContainerChildren<K>>>>,
    pub active_range: crate::DomActiveRangeState,
    pub fullscreen_events: Option<Arc<Payload<FullscreenQueue>>>,
    pub template_document: Option<crate::template::TemplateDocument<K>>,
    pub base_document: BaseDocumentState<K>,
    pub encoding_url_document: EncodingUrl,
    pub interactive_document: Interactive<K>,
    pub autofocus_document: Autofocus<K>,
}
#[derive(Clone, Debug)]
pub(crate) struct ElementNode<K> {
    pub common: Common<K>,
    pub element: Arc<Payload<DomElement>>,
    pub children: Option<Arc<Payload<ContainerChildren<K>>>>,
    pub slottable: Option<Arc<Payload<Slottable<K>>>>,
    pub shadow: Option<Arc<Payload<ShadowHost<K>>>>,
    pub associations: ElementAssociations<K>,
}
#[derive(Clone, Debug)]
pub(crate) struct FragmentNode<K> {
    pub common: Common<K>,
    pub children: Option<Arc<Payload<ContainerChildren<K>>>>,
    pub host: Option<K>,
}
#[derive(Clone, Debug)]
pub(crate) struct ShadowNode<K> {
    pub common: Common<K>,
    pub state: Arc<Payload<crate::DomShadowRootState>>,
    pub children: Option<Arc<Payload<ContainerChildren<K>>>>,
    pub host: Option<K>,
}
#[derive(Clone, Debug)]
pub(crate) struct TextNode<K> {
    pub common: Common<K>,
    pub data: Arc<Payload<CharacterData>>,
    pub slottable: Option<Arc<Payload<Slottable<K>>>>,
}
#[derive(Clone, Debug)]
pub(crate) struct CommentNode<K> {
    pub common: Common<K>,
    pub data: Arc<Payload<CharacterData>>,
}
#[derive(Clone, Debug)]
pub(crate) struct ProcessingInstructionNode<K> {
    pub common: Common<K>,
    pub data: Arc<Payload<crate::DomProcessingInstruction>>,
}
#[derive(Clone, Debug)]
pub(crate) struct DocumentTypeNode<K> {
    pub common: Common<K>,
    pub data: Arc<Payload<crate::DomDocumentType>>,
}
#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
pub(crate) enum Node {
    Document(Arc<Payload<DocumentNode<NodeId>>>),
    Element(Arc<Payload<ElementNode<NodeId>>>),
    DocumentFragment(Arc<Payload<FragmentNode<NodeId>>>),
    ShadowRoot(Arc<Payload<ShadowNode<NodeId>>>),
    Text(Arc<Payload<TextNode<NodeId>>>),
    Comment(Arc<Payload<CommentNode<NodeId>>>),
    ProcessingInstruction(Arc<Payload<ProcessingInstructionNode<NodeId>>>),
    DocumentType(Arc<Payload<DocumentTypeNode<NodeId>>>),
}
macro_rules! common {
    ($node:expr, $n:ident => $body:expr) => {
        match $node {
            Node::Document($n) => $body,
            Node::Element($n) => $body,
            Node::DocumentFragment($n) => $body,
            Node::ShadowRoot($n) => $body,
            Node::Text($n) => $body,
            Node::Comment($n) => $body,
            Node::ProcessingInstruction($n) => $body,
            Node::DocumentType($n) => $body,
        }
    };
}
impl Node {
    pub(crate) fn common(&self) -> &Common<NodeId> {
        common!(self,n => &n.value.common)
    }
    #[cfg(test)]
    pub(crate) fn identity(&self) -> custody::MaterializationId {
        common!(self,n => n.identity)
    }
    pub(crate) fn own_usage(&self) -> custody::Usage {
        common!(self,n => n.own_usage)
    }
    pub(crate) fn document(&self) -> DocumentId {
        DocumentId(self.common().document)
    }
    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.common().parent
    }
    pub(crate) fn relevant_global(&self) -> Option<crate::hosted::DomHostedGlobalId> {
        self.common().relevant_global
    }
    pub(crate) fn kind(&self) -> DomNodeKind<'_> {
        match self {
            Self::Document(n) => DomNodeKind::Document(&n.value.metadata.value),
            Self::Element(n) => DomNodeKind::Element(&n.value.element.value),
            Self::DocumentFragment(_) => DomNodeKind::DocumentFragment,
            Self::ShadowRoot(n) => DomNodeKind::ShadowRoot(&n.value.state.value),
            Self::Text(n) => DomNodeKind::Text(&n.value.data.value.value),
            Self::Comment(n) => DomNodeKind::Comment(&n.value.data.value.value),
            Self::ProcessingInstruction(n) => {
                DomNodeKind::ProcessingInstruction(&n.value.data.value)
            }
            Self::DocumentType(n) => DomNodeKind::DocumentType(&n.value.data.value),
        }
    }
    pub(crate) fn element(&self) -> Option<&DomElement> {
        match self {
            Self::Element(n) => Some(&n.value.element.value),
            _ => None,
        }
    }
    pub(crate) fn children_backing(&self) -> Option<&SharedVec<NodeId>> {
        let value = match self {
            Self::Document(n) => n.value.children.as_ref(),
            Self::Element(n) => n.value.children.as_ref(),
            Self::DocumentFragment(n) => n.value.children.as_ref(),
            Self::ShadowRoot(n) => n.value.children.as_ref(),
            _ => None,
        };
        value.map(|n| &n.value.values)
    }
    pub(crate) fn children(&self) -> &[NodeId] {
        self.children_backing().map_or(&[], SharedVec::as_slice)
    }
    pub(crate) fn active_range(&self) -> crate::DomActiveRangeState {
        match self {
            Self::Document(n) => n.value.active_range,
            _ => crate::DomActiveRangeState::Unavailable,
        }
    }
    pub(crate) fn fullscreen_events(&self) -> &[crate::DomFullscreenEvent] {
        match self {
            Self::Document(n) => n
                .value
                .fullscreen_events
                .as_ref()
                .map_or(&[], |v| v.value.values.as_slice()),
            _ => &[],
        }
    }
    pub(crate) fn shadow_root(&self) -> Option<NodeId> {
        match self {
            Self::Element(n) => n.value.shadow.as_ref().map(|v| v.value.shadow_root),
            _ => None,
        }
    }
    pub(crate) fn host(&self) -> Option<NodeId> {
        match self {
            Self::DocumentFragment(n) => n.value.host,
            Self::ShadowRoot(n) => n.value.host,
            _ => None,
        }
    }
    pub(crate) fn slottable(&self) -> Option<&Slottable<NodeId>> {
        match self {
            Self::Element(n) => n.value.slottable.as_ref().map(|v| &v.value),
            Self::Text(n) => n.value.slottable.as_ref().map(|v| &v.value),
            _ => None,
        }
    }
    pub(crate) fn manual_slot(&self) -> Option<NodeId> {
        self.slottable().and_then(|v| v.manual_slot)
    }
    pub(crate) fn assigned_slot(&self) -> Option<NodeId> {
        self.slottable().and_then(|v| v.assigned_slot)
    }
    pub(crate) fn associations(&self) -> Option<&ElementAssociations<NodeId>> {
        match self {
            Self::Element(n) => Some(&n.value.associations),
            _ => None,
        }
    }
    pub(crate) fn template_contents(&self) -> Option<NodeId> {
        match self.associations() {
            Some(ElementAssociations::Template(v)) => v.value.template_contents,
            _ => None,
        }
    }
    pub(crate) fn template_patch(&self) -> Option<crate::template::TemplatePatch<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Template(v)) => v.value.template_patch,
            _ => None,
        }
    }
    pub(crate) fn template_document(&self) -> Option<crate::template::TemplateDocument<NodeId>> {
        match self {
            Self::Document(n) => n.value.template_document,
            _ => None,
        }
    }
    pub(crate) fn manual_nodes(&self) -> &[NodeId] {
        match self.associations() {
            Some(ElementAssociations::Slot(v)) => v.value.manual_nodes.as_slice(),
            _ => &[],
        }
    }
    pub(crate) fn assigned_nodes(&self) -> &[NodeId] {
        match self.associations() {
            Some(ElementAssociations::Slot(v)) => v.value.assigned_nodes.as_slice(),
            _ => &[],
        }
    }
    pub(crate) fn control_owner(&self) -> Option<NodeId> {
        match self.associations() {
            Some(ElementAssociations::Control(v)) => v.value.control_owner,
            Some(ElementAssociations::Image(v)) => v.value.control_owner,
            _ => None,
        }
    }
    pub(crate) fn reflected_targets(&self) -> Option<crate::reflected::ReflectedTargets<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Control(v)) => v.value.reflected_targets,
            _ => None,
        }
    }
    pub(crate) fn option_select(&self) -> Option<NodeId> {
        match self.associations() {
            Some(ElementAssociations::Option(v)) => v.value.option_select,
            _ => None,
        }
    }
    pub(crate) fn form_past_names(&self) -> Option<&crate::form::FormPastNames<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Form(v)) => Some(&v.value.past_names),
            Some(ElementAssociations::FormEmpty(v)) => Some(v),
            _ => None,
        }
    }
    pub(crate) fn script(&self) -> Option<&crate::script::ScriptState<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Script(v)) => Some(&v.value),
            _ => None,
        }
    }
    pub(crate) fn style_parser_document(&self) -> Option<NodeId> {
        match self.associations() {
            Some(ElementAssociations::Style(v)) => v.value.parser_document,
            _ => None,
        }
    }
    pub(crate) fn base(&self) -> Option<&crate::base::BaseState<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Base(v)) => Some(&v.value),
            _ => None,
        }
    }
    pub(crate) fn link(&self) -> Option<&crate::base::LinkState<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Link(v)) => Some(&v.value),
            _ => None,
        }
    }
    pub(crate) fn image(&self) -> Option<&crate::image::ImageState<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Image(v)) => Some(&v.value.image.value),
            _ => None,
        }
    }
    pub(crate) fn hyperlink(&self) -> Option<&crate::hyperlink::HyperlinkState<NodeId>> {
        match self.associations() {
            Some(ElementAssociations::Hyperlink(v)) => Some(&v.value),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum InteractiveView<'a> {
    Unavailable,
    Known {
        open_dialogs: &'a [NodeId],
        top_layer: &'a [NodeId],
        pending_top_layer_removals: &'a [NodeId],
    },
}
#[derive(Clone, Copy)]
pub(crate) enum AutofocusView<'a> {
    Unavailable,
    Known {
        candidates: &'a [NodeId],
        processed: bool,
    },
}
#[derive(Clone, Copy)]
pub(crate) struct BaseDocumentView<'a> {
    pub facts: Option<&'a HtmlParseBaseFacts>,
    pub first: Option<NodeId>,
    pub render_blocking: &'a [NodeId],
}
#[derive(Clone, Copy)]
pub(crate) struct EncodingUrlView<'a> {
    pub facts: Option<&'a surgeist_html_semantics::HtmlEncodingUrlParseFacts>,
}
impl Node {
    pub(crate) fn interactive_document(&self) -> Option<InteractiveView<'_>> {
        let Self::Document(n) = self else {
            return None;
        };
        Some(match &n.value.interactive_document {
            Interactive::Absent => return None,
            Interactive::Unavailable => InteractiveView::Unavailable,
            Interactive::KnownEmpty => InteractiveView::Known {
                open_dialogs: &[],
                top_layer: &[],
                pending_top_layer_removals: &[],
            },
            Interactive::Known(v) => InteractiveView::Known {
                open_dialogs: v.value.open_dialogs.as_slice(),
                top_layer: v.value.top_layer.as_slice(),
                pending_top_layer_removals: v.value.pending_top_layer_removals.as_slice(),
            },
        })
    }
    pub(crate) fn autofocus_document(&self) -> Option<AutofocusView<'_>> {
        let Self::Document(n) = self else {
            return None;
        };
        Some(match &n.value.autofocus_document {
            Autofocus::Absent => return None,
            Autofocus::Unavailable => AutofocusView::Unavailable,
            Autofocus::KnownEmpty { processed } => AutofocusView::Known {
                candidates: &[],
                processed: *processed,
            },
            Autofocus::Known(v) => AutofocusView::Known {
                candidates: v.value.candidates.as_slice(),
                processed: v.value.processed,
            },
        })
    }
    pub(crate) fn base_document(&self) -> Option<BaseDocumentView<'_>> {
        let Self::Document(n) = self else {
            return None;
        };
        Some(match &n.value.base_document {
            BaseDocumentState::Absent => return None,
            BaseDocumentState::KnownEmpty => BaseDocumentView {
                facts: None,
                first: None,
                render_blocking: &[],
            },
            BaseDocumentState::Known(v) => BaseDocumentView {
                facts: v.value.facts.as_ref(),
                first: v.value.first,
                render_blocking: v.value.render_blocking.as_slice(),
            },
        })
    }
    pub(crate) fn encoding_url_document(&self) -> Option<EncodingUrlView<'_>> {
        let Self::Document(n) = self else {
            return None;
        };
        Some(match &n.value.encoding_url_document {
            EncodingUrl::Absent => return None,
            EncodingUrl::KnownEmpty => EncodingUrlView { facts: None },
            EncodingUrl::Known(v) => EncodingUrlView {
                facts: v.value.facts.as_ref(),
            },
        })
    }
}

fn same_payload<T: PartialEq>(a: &Arc<Payload<T>>, b: &Arc<Payload<T>>) -> bool {
    Arc::ptr_eq(a, b) || a.value == b.value
}
fn same_optional<T: PartialEq>(a: &Option<Arc<Payload<T>>>, b: &Option<Arc<Payload<T>>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => same_payload(a, b),
        _ => false,
    }
}
impl<K: PartialEq> PartialEq for ElementAssociations<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::FormEmpty(a), Self::FormEmpty(b)) => a == b,
            (Self::FormEmpty(a), Self::Form(b)) => a == &b.value.past_names,
            (Self::Form(a), Self::FormEmpty(b)) => &a.value.past_names == b,
            (Self::None, value) | (value, Self::None) => match value {
                Self::Template(v) => {
                    v.value.template_contents.is_none() && v.value.template_patch.is_none()
                }
                Self::Slot(v) => {
                    v.value.manual_nodes.is_empty() && v.value.assigned_nodes.is_empty()
                }
                Self::Control(v) => {
                    v.value.control_owner.is_none() && v.value.reflected_targets.is_none()
                }
                Self::Option(v) => v.value.option_select.is_none(),
                Self::Style(v) => v.value.parser_document.is_none(),
                _ => false,
            },
            (Self::Template(a), Self::Template(b)) => same_payload(a, b),
            (Self::Slot(a), Self::Slot(b)) => same_payload(a, b),
            (Self::Control(a), Self::Control(b)) => same_payload(a, b),
            (Self::Form(a), Self::Form(b)) => same_payload(a, b),
            (Self::Option(a), Self::Option(b)) => same_payload(a, b),
            (Self::Script(a), Self::Script(b)) => same_payload(a, b),
            (Self::Style(a), Self::Style(b)) => same_payload(a, b),
            (Self::Base(a), Self::Base(b)) => same_payload(a, b),
            (Self::Link(a), Self::Link(b)) => same_payload(a, b),
            (Self::Image(a), Self::Image(b)) => same_payload(a, b),
            (Self::Hyperlink(a), Self::Hyperlink(b)) => same_payload(a, b),
            _ => false,
        }
    }
}
impl<K: PartialEq> PartialEq for Interactive<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) => true,
            (Self::Unavailable, Self::Unavailable) => true,
            (Self::KnownEmpty, Self::KnownEmpty) => true,
            (Self::Known(a), Self::Known(b)) => same_payload(a, b),
            (Self::KnownEmpty, Self::Known(v)) | (Self::Known(v), Self::KnownEmpty) => {
                v.value.open_dialogs.is_empty()
                    && v.value.top_layer.is_empty()
                    && v.value.pending_top_layer_removals.is_empty()
            }
            _ => false,
        }
    }
}
impl<K: PartialEq> PartialEq for BaseDocumentState<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) => true,
            (Self::KnownEmpty, Self::KnownEmpty) => true,
            (Self::Known(a), Self::Known(b)) => same_payload(a, b),
            (Self::KnownEmpty, Self::Known(v)) | (Self::Known(v), Self::KnownEmpty) => {
                v.value.facts.is_none()
                    && v.value.first.is_none()
                    && v.value.render_blocking.is_empty()
            }
            _ => false,
        }
    }
}
impl PartialEq for EncodingUrl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) => true,
            (Self::KnownEmpty, Self::KnownEmpty) => true,
            (Self::Known(a), Self::Known(b)) => same_payload(a, b),
            (Self::KnownEmpty, Self::Known(v)) | (Self::Known(v), Self::KnownEmpty) => {
                v.value.facts.is_none()
            }
            _ => false,
        }
    }
}
impl<K: PartialEq> PartialEq for Autofocus<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Absent, Self::Absent) | (Self::Unavailable, Self::Unavailable) => true,
            (Self::KnownEmpty { processed: a }, Self::KnownEmpty { processed: b }) => a == b,
            (Self::Known(a), Self::Known(b)) => same_payload(a, b),
            (Self::KnownEmpty { processed }, Self::Known(v))
            | (Self::Known(v), Self::KnownEmpty { processed }) => {
                *processed == v.value.processed && v.value.candidates.is_empty()
            }
            _ => false,
        }
    }
}
fn same_children<K: PartialEq>(
    a: &Option<Arc<Payload<ContainerChildren<K>>>>,
    b: &Option<Arc<Payload<ContainerChildren<K>>>>,
) -> bool {
    match (a, b) {
        (None, Some(v)) | (Some(v), None) => v.value.values.is_empty(),
        _ => same_optional(a, b),
    }
}
fn same_slottable<K: PartialEq>(
    a: &Option<Arc<Payload<Slottable<K>>>>,
    b: &Option<Arc<Payload<Slottable<K>>>>,
) -> bool {
    match (a, b) {
        (None, Some(v)) | (Some(v), None) => {
            v.value.manual_slot.is_none() && v.value.assigned_slot.is_none()
        }
        _ => same_optional(a, b),
    }
}
fn same_fullscreen(
    a: &Option<Arc<Payload<FullscreenQueue>>>,
    b: &Option<Arc<Payload<FullscreenQueue>>>,
) -> bool {
    match (a, b) {
        (None, Some(v)) | (Some(v), None) => v.value.values.is_empty(),
        _ => same_optional(a, b),
    }
}
impl<K: PartialEq> PartialEq for DocumentNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
            && same_payload(&self.metadata, &other.metadata)
            && same_children(&self.children, &other.children)
            && self.active_range == other.active_range
            && same_fullscreen(&self.fullscreen_events, &other.fullscreen_events)
            && self.template_document == other.template_document
            && self.base_document == other.base_document
            && self.encoding_url_document == other.encoding_url_document
            && self.interactive_document == other.interactive_document
            && self.autofocus_document == other.autofocus_document
    }
}
impl<K: PartialEq> PartialEq for ElementNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
            && same_payload(&self.element, &other.element)
            && same_children(&self.children, &other.children)
            && same_slottable(&self.slottable, &other.slottable)
            && same_optional(&self.shadow, &other.shadow)
            && self.associations == other.associations
    }
}
impl<K: PartialEq> PartialEq for FragmentNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
            && same_children(&self.children, &other.children)
            && self.host == other.host
    }
}
impl<K: PartialEq> PartialEq for ShadowNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
            && same_payload(&self.state, &other.state)
            && same_children(&self.children, &other.children)
            && self.host == other.host
    }
}
impl<K: PartialEq> PartialEq for TextNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
            && same_payload(&self.data, &other.data)
            && same_slottable(&self.slottable, &other.slottable)
    }
}
impl<K: PartialEq> PartialEq for CommentNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common && same_payload(&self.data, &other.data)
    }
}
impl<K: PartialEq> PartialEq for ProcessingInstructionNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common && same_payload(&self.data, &other.data)
    }
}
impl<K: PartialEq> PartialEq for DocumentTypeNode<K> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common && same_payload(&self.data, &other.data)
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Document(a), Self::Document(b)) => same_payload(a, b),
            (Self::Element(a), Self::Element(b)) => same_payload(a, b),
            (Self::DocumentFragment(a), Self::DocumentFragment(b)) => same_payload(a, b),
            (Self::ShadowRoot(a), Self::ShadowRoot(b)) => same_payload(a, b),
            (Self::Text(a), Self::Text(b)) => same_payload(a, b),
            (Self::Comment(a), Self::Comment(b)) => same_payload(a, b),
            (Self::ProcessingInstruction(a), Self::ProcessingInstruction(b)) => same_payload(a, b),
            (Self::DocumentType(a), Self::DocumentType(b)) => same_payload(a, b),
            _ => false,
        }
    }
}
