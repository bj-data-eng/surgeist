//! Borrowed node observations. Mapping an identity never materializes a record
//! or clones retained language values.

use super::super::*;
use super::SequenceView;

pub(in crate::store) type Relations<'a> = SequenceView<'a, NodeId, Key>;
pub(in crate::store) type Events<'a> =
    SequenceView<'a, crate::DomFullscreenEvent, crate::DomFullscreenEvent>;

#[derive(Clone, Copy)]
pub(in crate::store) struct FrozenRead<'a> {
    pub value: &'a surgeist_html_semantics::HtmlBaseUrlRecord,
    pub document: Key,
    pub revision: &'a surgeist_html_semantics::HtmlBaseFactsRevision,
    pub response: crate::base::Response,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct BaseRead<'a> {
    pub frozen: Option<FrozenRead<'a>>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct BaseDocumentRead<'a> {
    pub facts: Option<&'a surgeist_html_semantics::HtmlParseBaseFacts>,
    pub first: Option<Key>,
    pub render_blocking: Relations<'a>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct LinkRead<'a> {
    pub flags: surgeist_html::HtmlLinkFlags,
    pub parser_document: Option<Key>,
    pub sheet: Option<&'a surgeist_cssom::CssomSheetId>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct ImageRead<'a> {
    pub native: &'a surgeist_html::HtmlImgFreshState,
    pub dimension_source: Key,
    pub parser_inserted: bool,
    pub high_water: u64,
    pub waiting: Option<crate::image::Waiting>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct ParsedRead<'a> {
    pub document: Key,
    pub value: &'a surgeist_html_semantics::HtmlHyperlinkUrlSnapshot,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct HyperlinkRead<'a> {
    pub parsed: Option<ParsedRead<'a>>,
}
impl BaseRead<'_> {
    pub fn units(&self) -> usize {
        self.frozen
            .as_ref()
            .map_or(0, |frozen| frozen.value.serialization().len())
    }
}
impl BaseDocumentRead<'_> {
    pub fn units(&self) -> usize {
        self.facts
            .map_or(0, |facts| facts.formation_usage().utf16_extent())
    }
}
impl HyperlinkRead<'_> {
    pub fn units(&self) -> usize {
        self.parsed
            .as_ref()
            .map_or(0, |parsed| parsed.value.retained_utf16_extent())
    }
}
#[derive(Clone, Copy)]
pub(in crate::store) struct EncodingUrlRead<'a> {
    pub facts: Option<&'a surgeist_html_semantics::HtmlEncodingUrlParseFacts>,
}
impl EncodingUrlRead<'_> {
    pub fn units(self) -> usize {
        self.facts.map_or(0, |v| v.formation_usage().utf16_extent())
    }
    pub fn records(self) -> usize {
        self.facts
            .map_or(0, |v| v.formation_usage().record_capacity())
    }
}
#[derive(Clone, Copy)]
pub(in crate::store) enum AutofocusRead<'a> {
    Known {
        candidates: Relations<'a>,
        processed: bool,
    },
    Unavailable,
}
#[derive(Clone, Copy)]
pub(in crate::store) enum InteractiveRead<'a> {
    Known {
        open_dialogs: Relations<'a>,
        top_layer: Relations<'a>,
        pending_top_layer_removals: Relations<'a>,
    },
    Unavailable,
}

#[derive(Clone, Copy)]
pub(in crate::store) enum NamesRead<'a> {
    Empty,
    Canonical(&'a crate::form::FormPastNames<NodeId>),
    Staged(&'a crate::form::FormPastNames<Key>),
    AliasedCanonical(
        &'a crate::form::FormPastNames<NodeId>,
        &'a source::AllocationIds,
    ),
    AliasedStaged(
        &'a crate::form::FormPastNames<Key>,
        &'a source::AllocationIds,
    ),
}
#[derive(Clone, Copy)]
pub(in crate::store) struct NameRead<'a> {
    pub name: &'a DocumentString,
    pub target: Key,
}
impl<'a> NamesRead<'a> {
    pub fn aliases(self, ctx: &'a source::AllocationIds) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Canonical(v) | Self::AliasedCanonical(v, _) => Self::AliasedCanonical(v, ctx),
            Self::Staged(v) | Self::AliasedStaged(v, _) => Self::AliasedStaged(v, ctx),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Canonical(values) | Self::AliasedCanonical(values, _) => values.entries().len(),
            Self::Staged(values) | Self::AliasedStaged(values, _) => values.entries().len(),
        }
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = NameRead<'_>> {
        (0..self.len()).map(|index| match self {
            Self::Empty => unreachable!("empty history has no member"),
            Self::Canonical(values) => NameRead {
                name: &values.entries()[index].name,
                target: Key::Existing(values.entries()[index].target),
            },
            Self::Staged(values) => NameRead {
                name: &values.entries()[index].name,
                target: values.entries()[index].target,
            },
            Self::AliasedCanonical(values, ctx) => NameRead {
                name: &values.entries()[index].name,
                target: source::alias_id(values.entries()[index].target, ctx),
            },
            Self::AliasedStaged(values, ctx) => NameRead {
                name: &values.entries()[index].name,
                target: source::alias_key(values.entries()[index].target, ctx),
            },
        })
    }
}

use super::staged::*;
use canonical::Common;

#[derive(Clone, Copy)]
pub(in crate::store) enum KindRead<'a> {
    Document(&'a DomDocumentMetadata),
    Element(ElementRead<'a>),
    DocumentFragment,
    ShadowRoot(&'a crate::DomShadowRootState),
    Text(&'a DocumentString),
    Comment(&'a DocumentString),
    ProcessingInstruction(&'a crate::DomProcessingInstruction),
    DocumentType(&'a crate::DomDocumentType),
}
#[derive(Clone, Copy)]
pub(in crate::store) struct ElementRead<'a> {
    value: &'a DomElement,
    focus: Option<&'a FocusVersion>,
    aliases: Option<&'a source::AllocationIds>,
}
impl<'a> ElementRead<'a> {
    pub fn language(self) -> &'a DomElement {
        self.value
    }
    pub fn common_state(self) -> surgeist_html::HtmlCommonState<Key> {
        let map = |id| {
            self.aliases
                .map_or(Key::Existing(id), |ids| source::alias_id(id, ids))
        };
        let mut state = self.value.common_state().map(map);
        if let Some(html) = state.html_mut() {
            match self.focus {
                Some(FocusVersion::Staged(key)) => html.remember_focused_element(Some(
                    self.aliases
                        .map_or(*key, |ids| source::alias_key(*key, ids)),
                )),
                Some(FocusVersion::Cleared) => html.remember_focused_element(None),
                _ => {}
            }
        }
        state
    }
}
impl std::ops::Deref for ElementRead<'_> {
    type Target = DomElement;
    fn deref(&self) -> &DomElement {
        self.value
    }
}
#[derive(Clone, Copy)]
enum Source<'a> {
    Canonical(&'a Node),
    Staged(&'a DraftNode),
}
#[derive(Clone, Copy)]
pub(in crate::store) enum Overlay<'a> {
    Common(&'a Common<Key>),
    Focus(&'a FocusVersion),
    Element(&'a DomElement),
    Metadata(&'a DomDocumentMetadata),
    ShadowState(&'a crate::DomShadowRootState),
    ProcessingInstruction(&'a crate::DomProcessingInstruction),
    Text(&'a DocumentString),
    Comment(&'a DocumentString),
    Slottable(&'a canonical::Slottable<Key>),
    Shadow(&'a Option<Key>),
    Host(&'a Option<Key>),
    Template(&'a canonical::TemplateAssociation<Key>),
    TemplateDocument(&'a Option<crate::template::TemplateDocument<Key>>),
    Control(&'a canonical::ControlAssociation<Key>),
    Option(&'a Option<Key>),
    Names(&'a Option<super::Names>),
    Script(&'a Option<crate::script::ScriptState<Key>>),
    StyleParser(&'a Option<Key>),
    Base(&'a Option<crate::base::BaseState<Key>>),
    Link(&'a Option<crate::base::LinkState<Key>>),
    Image(&'a Option<crate::image::ImageState<Key>>),
    Hyperlink(&'a Option<crate::hyperlink::HyperlinkState<Key>>),
    BaseDocument(&'a Option<crate::base::BaseDocument<Key>>),
    Encoding(&'a Option<crate::hyperlink::EncodingUrlDocument>),
    Interactive(&'a Option<crate::dialog::DocumentInteractiveState<Key>>),
    Autofocus(&'a Option<crate::autofocus::DocumentAutofocusState<Key>>),
    ActiveRange(&'a crate::DomActiveRangeState),
}
#[derive(Clone, Copy)]
pub(in crate::store) struct NodeRead<'a> {
    source: Source<'a>,
    aliases: Option<&'a source::AllocationIds>,
    overlay: Option<Overlay<'a>>,
}
impl<'a> NodeRead<'a> {
    pub fn canonical(value: &'a Node) -> Self {
        Self {
            source: Source::Canonical(value),
            aliases: None,
            overlay: None,
        }
    }
    pub fn staged(value: &'a DraftNode) -> Self {
        Self {
            source: Source::Staged(value),
            aliases: None,
            overlay: None,
        }
    }
    pub fn changed(self, value: Overlay<'a>) -> Self {
        Self {
            overlay: Some(value),
            ..self
        }
    }
    pub fn aliases(self, ids: &'a source::AllocationIds) -> Self {
        Self {
            aliases: Some(ids),
            ..self
        }
    }
    fn alias(self, key: Key) -> Key {
        self.aliases.map_or(key, |ids| source::alias_key(key, ids))
    }
    fn id(self, id: NodeId) -> Key {
        self.alias(Key::Existing(id))
    }
    pub fn common(self) -> Common<Key> {
        let value = if let Some(Overlay::Common(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => Common {
                    document: Key::Existing(n.common().document),
                    relevant_global: n.common().relevant_global,
                    parent: n.common().parent.map(Key::Existing),
                },
                Source::Staged(n) => *n.common(),
            }
        };
        Common {
            document: self.alias(value.document),
            parent: value.parent.map(|k| self.alias(k)),
            ..value
        }
    }
    pub fn document(self) -> Key {
        self.common().document
    }
    pub fn parent(self) -> Option<Key> {
        self.common().parent
    }
    pub fn relevant_global(self) -> Option<crate::hosted::DomHostedGlobalId> {
        self.common().relevant_global
    }
    pub fn kind(self) -> KindRead<'a> {
        if let Some(value) = self.overlay {
            match value {
                Overlay::Element(v) => {
                    return KindRead::Element(ElementRead {
                        value: v,
                        focus: self.stage_focus(),
                        aliases: self.aliases,
                    });
                }
                Overlay::Metadata(v) => return KindRead::Document(v),
                Overlay::ProcessingInstruction(v) => return KindRead::ProcessingInstruction(v),
                Overlay::ShadowState(v) => return KindRead::ShadowRoot(v),
                Overlay::Text(v) => return KindRead::Text(v),
                Overlay::Comment(v) => return KindRead::Comment(v),
                _ => {}
            }
        }
        match self.source {
            Source::Canonical(n) => match n.kind() {
                DomNodeKind::Document(v) => KindRead::Document(v),
                DomNodeKind::Element(v) => KindRead::Element(ElementRead {
                    value: v,
                    focus: self.stage_focus(),
                    aliases: self.aliases,
                }),
                DomNodeKind::DocumentFragment => KindRead::DocumentFragment,
                DomNodeKind::ShadowRoot(v) => KindRead::ShadowRoot(v),
                DomNodeKind::Text(v) => KindRead::Text(v),
                DomNodeKind::Comment(v) => KindRead::Comment(v),
                DomNodeKind::ProcessingInstruction(v) => KindRead::ProcessingInstruction(v),
                DomNodeKind::DocumentType(v) => KindRead::DocumentType(v),
            },
            Source::Staged(n) => match n {
                DraftNode::Document(v) => KindRead::Document(&v.value().metadata.value.value),
                DraftNode::Element(v) => KindRead::Element(ElementRead {
                    value: &v.value().element.value.value,
                    focus: self.stage_focus(),
                    aliases: self.aliases,
                }),
                DraftNode::DocumentFragment(_) => KindRead::DocumentFragment,
                DraftNode::ShadowRoot(v) => KindRead::ShadowRoot(&v.value().state.value.value),
                DraftNode::Text(v) => KindRead::Text(&v.value().data.value.value.value),
                DraftNode::Comment(v) => KindRead::Comment(&v.value().data.value.value.value),
                DraftNode::ProcessingInstruction(v) => {
                    KindRead::ProcessingInstruction(&v.value().data.value.value)
                }
                DraftNode::DocumentType(v) => KindRead::DocumentType(&v.value().data.value.value),
            },
        }
    }
    fn stage_focus(self) -> Option<&'a FocusVersion> {
        if let Some(Overlay::Focus(value)) = self.overlay {
            return Some(value);
        }
        match self.source {
            Source::Staged(DraftNode::Element(v)) => Some(&v.value().focus),
            _ => None,
        }
    }

    pub fn element(self) -> Option<ElementRead<'a>> {
        match self.kind() {
            KindRead::Element(v) => Some(v),
            _ => None,
        }
    }
    pub fn document_metadata(self) -> Option<&'a DomDocumentMetadata> {
        match self.kind() {
            KindRead::Document(v) => Some(v),
            _ => None,
        }
    }
    pub fn shadow_state(self) -> Option<&'a crate::DomShadowRootState> {
        match self.kind() {
            KindRead::ShadowRoot(v) => Some(v),
            _ => None,
        }
    }
    fn relation_aliases(self, v: Relations<'a>) -> Relations<'a> {
        self.aliases
            .map_or(v, |ids| v.aliases(ids, source::alias_key))
    }

    pub fn children(self) -> Relations<'a> {
        let value = match self.source {
            Source::Canonical(n) => n
                .children_backing()
                .map_or(Relations::Empty, |v| Relations::Canonical(v, Key::Existing)),
            Source::Staged(n) => match n.children().map(Group::read) {
                Some(GroupRead::Canonical(v)) => Relations::Canonical(&v.values, Key::Existing),
                Some(GroupRead::Changed(v)) => v.values.view(),
                None => Relations::Empty,
            },
        };
        self.relation_aliases(value)
    }
    fn slot_relation(self, manual: bool) -> Relations<'a> {
        let value = match self.source {
            Source::Canonical(Node::Element(n)) => match &n.value.associations {
                canonical::ElementAssociations::Slot(v) => Relations::Canonical(
                    if manual {
                        &v.value.manual_nodes
                    } else {
                        &v.value.assigned_nodes
                    },
                    Key::Existing,
                ),
                _ => Relations::Empty,
            },
            Source::Staged(n) => match n.associations() {
                Some(Associations::Slot(v)) => match v.read() {
                    GroupRead::Canonical(v) => Relations::Canonical(
                        if manual {
                            &v.manual_nodes
                        } else {
                            &v.assigned_nodes
                        },
                        Key::Existing,
                    ),
                    GroupRead::Changed(v) => {
                        if manual {
                            v.manual.view()
                        } else {
                            v.assigned.view()
                        }
                    }
                },
                _ => Relations::Empty,
            },
            _ => Relations::Empty,
        };
        self.relation_aliases(value)
    }
    pub fn manual_nodes(self) -> Relations<'a> {
        self.slot_relation(true)
    }
    pub fn assigned_nodes(self) -> Relations<'a> {
        self.slot_relation(false)
    }
    fn slottable(self) -> Option<canonical::Slottable<Key>> {
        let v = if let Some(Overlay::Slottable(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => {
                    let v = n.slottable()?;
                    canonical::Slottable {
                        manual_slot: v.manual_slot.map(Key::Existing),
                        assigned_slot: v.assigned_slot.map(Key::Existing),
                    }
                }
                Source::Staged(n) => match n.slottable()?.read() {
                    GroupRead::Canonical(v) => canonical::Slottable {
                        manual_slot: v.manual_slot.map(Key::Existing),
                        assigned_slot: v.assigned_slot.map(Key::Existing),
                    },
                    GroupRead::Changed(v) => *v,
                },
            }
        };
        Some(canonical::Slottable {
            manual_slot: v.manual_slot.map(|k| self.alias(k)),
            assigned_slot: v.assigned_slot.map(|k| self.alias(k)),
        })
    }
    pub fn manual_slot(self) -> Option<Key> {
        self.slottable().and_then(|v| v.manual_slot)
    }
    pub fn assigned_slot(self) -> Option<Key> {
        self.slottable().and_then(|v| v.assigned_slot)
    }
    pub fn shadow_root(self) -> Option<Key> {
        if let Some(Overlay::Shadow(v)) = self.overlay {
            return v.map(|k| self.alias(k));
        }
        match self.source {
            Source::Canonical(n) => n.shadow_root().map(|id| self.id(id)),
            Source::Staged(DraftNode::Element(n)) => match n.value().shadow.as_ref()?.read() {
                GroupRead::Canonical(v) => Some(self.id(v.shadow_root)),
                GroupRead::Changed(v) => Some(self.alias(v.shadow_root)),
            },
            _ => None,
        }
    }
    pub fn host(self) -> Option<Key> {
        if let Some(Overlay::Host(v)) = self.overlay {
            return v.map(|k| self.alias(k));
        }
        match self.source {
            Source::Canonical(n) => n.host().map(|id| self.id(id)),
            Source::Staged(DraftNode::DocumentFragment(v)) => v.value().host.map(|k| self.alias(k)),
            Source::Staged(DraftNode::ShadowRoot(v)) => v.value().host.map(|k| self.alias(k)),
            _ => None,
        }
    }
    fn template(self) -> Option<canonical::TemplateAssociation<Key>> {
        let v = if let Some(Overlay::Template(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => canonical::TemplateAssociation {
                    template_contents: n.template_contents().map(Key::Existing),
                    template_patch: n.template_patch().map(|v| v.map(Key::Existing)),
                },
                Source::Staged(n) => match n.associations()? {
                    Associations::Template(v) => match v.read() {
                        GroupRead::Canonical(v) => canonical::TemplateAssociation {
                            template_contents: v.template_contents.map(Key::Existing),
                            template_patch: v.template_patch.map(|v| v.map(Key::Existing)),
                        },
                        GroupRead::Changed(v) => *v,
                    },
                    _ => return None,
                },
            }
        };
        Some(canonical::TemplateAssociation {
            template_contents: v.template_contents.map(|k| self.alias(k)),
            template_patch: v.template_patch.map(|v| v.map(|k| self.alias(k))),
        })
    }
    pub fn template_contents(self) -> Option<Key> {
        self.template().and_then(|v| v.template_contents)
    }
    pub fn template_patch(self) -> Option<crate::template::TemplatePatch<Key>> {
        self.template().and_then(|v| v.template_patch)
    }
    pub fn template_document(self) -> Option<crate::template::TemplateDocument<Key>> {
        let v = if let Some(Overlay::TemplateDocument(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => n.template_document().map(|v| v.map(Key::Existing)),
                Source::Staged(DraftNode::Document(n)) => n.value().template_document,
                _ => None,
            }
        };
        v.map(|v| v.map(|k| self.alias(k)))
    }
    pub fn control_owner(self) -> Option<Key> {
        let v = if let Some(Overlay::Control(v)) = self.overlay {
            v.control_owner
        } else {
            match self.source {
                Source::Canonical(n) => n.control_owner().map(Key::Existing),
                Source::Staged(n) => match n.associations()? {
                    Associations::Control(v) => match v.read() {
                        GroupRead::Canonical(v) => v.control_owner.map(Key::Existing),
                        GroupRead::Changed(v) => v.control_owner,
                    },
                    Associations::Image(v) => match v.read() {
                        GroupRead::Canonical(v) => v.control_owner.map(Key::Existing),
                        GroupRead::Changed(v) => v.control_owner,
                    },
                    _ => None,
                },
            }
        };
        v.map(|k| self.alias(k))
    }
    pub fn reflected_targets(self) -> Option<crate::reflected::ReflectedTargets<Key>> {
        let v = if let Some(Overlay::Control(v)) = self.overlay {
            v.reflected_targets
        } else {
            match self.source {
                Source::Canonical(n) => n.reflected_targets().map(|v| v.map(Key::Existing)),
                Source::Staged(n) => match n.associations()? {
                    Associations::Control(v) => match v.read() {
                        GroupRead::Canonical(v) => {
                            v.reflected_targets.map(|v| v.map(Key::Existing))
                        }
                        GroupRead::Changed(v) => v.reflected_targets,
                    },
                    _ => None,
                },
            }
        };
        v.map(|v| v.map(|k| self.alias(k)))
    }
    pub fn option_select(self) -> Option<Key> {
        let v = if let Some(Overlay::Option(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => n.option_select().map(Key::Existing),
                Source::Staged(n) => match n.associations()? {
                    Associations::Option(v) => match v.read() {
                        GroupRead::Canonical(v) => v.option_select.map(Key::Existing),
                        GroupRead::Changed(v) => v.option_select,
                    },
                    _ => None,
                },
            }
        };
        v.map(|k| self.alias(k))
    }
    pub fn script(self) -> Option<crate::script::ScriptState<Key>> {
        let v = if let Some(Overlay::Script(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => n.script().copied().map(|v| v.map(Key::Existing)),
                Source::Staged(n) => match n.associations()? {
                    Associations::Script(v) => match v.read() {
                        GroupRead::Canonical(v) => Some((*v).map(Key::Existing)),
                        GroupRead::Changed(v) => Some(*v),
                    },
                    _ => None,
                },
            }
        };
        v.map(|v| v.map(|k| self.alias(k)))
    }
    pub fn style_parser_document(self) -> Option<Key> {
        let v = if let Some(Overlay::StyleParser(v)) = self.overlay {
            *v
        } else {
            match self.source {
                Source::Canonical(n) => n.style_parser_document().map(Key::Existing),
                Source::Staged(n) => match n.associations()? {
                    Associations::Style(v) => match v.read() {
                        GroupRead::Canonical(v) => v.parser_document.map(Key::Existing),
                        GroupRead::Changed(v) => v.parser_document,
                    },
                    _ => None,
                },
            }
        };
        v.map(|k| self.alias(k))
    }
    pub fn form_past_names(self) -> Option<NamesRead<'a>> {
        let v = if let Some(Overlay::Names(v)) = self.overlay {
            v.as_ref().map(super::Names::view)
        } else {
            match self.source {
                Source::Canonical(n) => n.form_past_names().map(NamesRead::Canonical),
                Source::Staged(n) => match n.associations()? {
                    Associations::Form(v) => match v.read() {
                        GroupRead::Canonical(v) => Some(NamesRead::Canonical(&v.past_names)),
                        GroupRead::Changed(v) => Some(v.past_names.view()),
                    },
                    Associations::FormEmpty => Some(NamesRead::Empty),
                    _ => None,
                },
            }
        };
        v.map(|v| self.aliases.map_or(v, |ids| v.aliases(ids)))
    }
    pub fn base(self) -> Option<BaseRead<'a>> {
        if let Some(Overlay::Base(v)) = self.overlay {
            let v = v.as_ref()?;
            return Some(BaseRead {
                frozen: v.frozen.as_ref().map(|f| FrozenRead {
                    value: &f.value,
                    document: self.alias(f.document),
                    revision: &f.revision,
                    response: f.response,
                }),
            });
        }
        match self.source {
            Source::Canonical(n) => {
                let v = n.base()?;
                Some(BaseRead {
                    frozen: v.frozen.as_ref().map(|f| FrozenRead {
                        value: &f.value,
                        document: self.id(f.document),
                        revision: &f.revision,
                        response: f.response,
                    }),
                })
            }
            Source::Staged(n) => match n.associations()? {
                Associations::Base(group) => match group.read() {
                    GroupRead::Canonical(v) => Some(BaseRead {
                        frozen: v.frozen.as_ref().map(|f| FrozenRead {
                            value: &f.value,
                            document: self.id(f.document),
                            revision: &f.revision,
                            response: f.response,
                        }),
                    }),
                    GroupRead::Changed(v) => Some(BaseRead {
                        frozen: v.frozen.as_ref().map(|f| FrozenRead {
                            value: &f.value,
                            document: self.alias(f.document),
                            revision: &f.revision,
                            response: f.response,
                        }),
                    }),
                },
                _ => None,
            },
        }
    }
    pub fn link(self) -> Option<LinkRead<'a>> {
        if let Some(Overlay::Link(v)) = self.overlay {
            let v = v.as_ref()?;
            return Some(LinkRead {
                flags: v.flags,
                parser_document: v.parser_document.map(|k| self.alias(k)),
                sheet: v.sheet.as_ref(),
            });
        }
        match self.source {
            Source::Canonical(n) => {
                let v = n.link()?;
                Some(LinkRead {
                    flags: v.flags,
                    parser_document: v.parser_document.map(|id| self.id(id)),
                    sheet: v.sheet.as_ref(),
                })
            }
            Source::Staged(n) => match n.associations()? {
                Associations::Link(group) => match group.read() {
                    GroupRead::Canonical(v) => Some(LinkRead {
                        flags: v.flags,
                        parser_document: v.parser_document.map(|id| self.id(id)),
                        sheet: v.sheet.as_ref(),
                    }),
                    GroupRead::Changed(v) => Some(LinkRead {
                        flags: v.flags,
                        parser_document: v.parser_document.map(|k| self.alias(k)),
                        sheet: v.sheet.as_ref(),
                    }),
                },
                _ => None,
            },
        }
    }
    pub fn image(self) -> Option<ImageRead<'a>> {
        if let Some(Overlay::Image(v)) = self.overlay {
            let v = v.as_ref()?;
            return Some(ImageRead {
                native: &v.native,
                dimension_source: self.alias(v.dimension_source),
                parser_inserted: v.parser_inserted,
                high_water: v.high_water,
                waiting: v.waiting,
            });
        }
        match self.source {
            Source::Canonical(n) => {
                let v = n.image()?;
                Some(ImageRead {
                    native: &v.native,
                    dimension_source: self.id(v.dimension_source),
                    parser_inserted: v.parser_inserted,
                    high_water: v.high_water,
                    waiting: v.waiting,
                })
            }
            Source::Staged(n) => match n.associations()? {
                Associations::Image(group) => match group.read() {
                    GroupRead::Canonical(v) => {
                        let v = &v.image.value;
                        Some(ImageRead {
                            native: &v.native,
                            dimension_source: self.id(v.dimension_source),
                            parser_inserted: v.parser_inserted,
                            high_water: v.high_water,
                            waiting: v.waiting,
                        })
                    }
                    GroupRead::Changed(v) => match v.image.read() {
                        GroupRead::Canonical(v) => Some(ImageRead {
                            native: &v.native,
                            dimension_source: self.id(v.dimension_source),
                            parser_inserted: v.parser_inserted,
                            high_water: v.high_water,
                            waiting: v.waiting,
                        }),
                        GroupRead::Changed(v) => Some(ImageRead {
                            native: &v.native,
                            dimension_source: self.alias(v.dimension_source),
                            parser_inserted: v.parser_inserted,
                            high_water: v.high_water,
                            waiting: v.waiting,
                        }),
                    },
                },
                _ => None,
            },
        }
    }
    pub fn hyperlink(self) -> Option<HyperlinkRead<'a>> {
        if let Some(Overlay::Hyperlink(v)) = self.overlay {
            let v = v.as_ref()?;
            return Some(HyperlinkRead {
                parsed: v.parsed.as_ref().map(|p| ParsedRead {
                    document: self.alias(p.document),
                    value: &p.value,
                }),
            });
        }
        match self.source {
            Source::Canonical(n) => {
                let v = n.hyperlink()?;
                Some(HyperlinkRead {
                    parsed: v.parsed.as_ref().map(|p| ParsedRead {
                        document: self.id(p.document),
                        value: &p.value,
                    }),
                })
            }
            Source::Staged(n) => match n.associations()? {
                Associations::Hyperlink(group) => match group.read() {
                    GroupRead::Canonical(v) => Some(HyperlinkRead {
                        parsed: v.parsed.as_ref().map(|p| ParsedRead {
                            document: self.id(p.document),
                            value: &p.value,
                        }),
                    }),
                    GroupRead::Changed(v) => Some(HyperlinkRead {
                        parsed: v.parsed.as_ref().map(|p| ParsedRead {
                            document: self.alias(p.document),
                            value: &p.value,
                        }),
                    }),
                },
                _ => None,
            },
        }
    }
    pub fn active_range(self) -> crate::DomActiveRangeState {
        if let Some(Overlay::ActiveRange(v)) = self.overlay {
            return *v;
        }
        match self.source {
            Source::Canonical(n) => n.active_range(),
            Source::Staged(DraftNode::Document(n)) => n.value().active_range,
            _ => crate::DomActiveRangeState::Unavailable,
        }
    }
    pub fn fullscreen_events(self) -> Events<'a> {
        match self.source {
            Source::Canonical(Node::Document(n)) => n
                .value
                .fullscreen_events
                .as_ref()
                .map_or(Events::Empty, |v| {
                    Events::Canonical(&v.value.values, std::convert::identity)
                }),
            Source::Staged(DraftNode::Document(n)) => {
                match n.value().fullscreen.as_ref().map(Group::read) {
                    Some(GroupRead::Canonical(v)) => {
                        Events::Canonical(&v.values, std::convert::identity)
                    }
                    Some(GroupRead::Changed(v)) => v.view(),
                    None => Events::Empty,
                }
            }
            _ => Events::Empty,
        }
    }
    pub fn encoding_url_document(self) -> Option<EncodingUrlRead<'a>> {
        if let Some(Overlay::Encoding(v)) = self.overlay {
            return v.as_ref().map(|v| EncodingUrlRead {
                facts: v.facts.as_ref(),
            });
        }
        Some(match self.source {
            Source::Canonical(Node::Document(n)) => match &n.value.encoding_url_document {
                canonical::EncodingUrl::Absent => return None,
                canonical::EncodingUrl::KnownEmpty => EncodingUrlRead { facts: None },
                canonical::EncodingUrl::Known(v) => EncodingUrlRead {
                    facts: v.value.facts.as_ref(),
                },
            },
            Source::Staged(DraftNode::Document(n)) => match &n.value().encoding {
                EncodingGroup::Absent => return None,
                EncodingGroup::KnownEmpty => EncodingUrlRead { facts: None },
                EncodingGroup::Known(v) => EncodingUrlRead {
                    facts: v.value.value.facts.as_ref(),
                },
            },
            _ => return None,
        })
    }
    pub fn base_document(self) -> Option<BaseDocumentRead<'a>> {
        if let Some(Overlay::BaseDocument(v)) = self.overlay {
            return v.as_ref().map(|v| BaseDocumentRead {
                facts: v.facts.as_ref(),
                first: v.first.map(|k| self.alias(k)),
                render_blocking: self.base_render(),
            });
        }
        match self.source {
            Source::Canonical(Node::Document(n)) => match &n.value.base_document {
                canonical::BaseDocumentState::Absent => None,
                canonical::BaseDocumentState::KnownEmpty => Some(BaseDocumentRead {
                    facts: None,
                    first: None,
                    render_blocking: Relations::Empty,
                }),
                canonical::BaseDocumentState::Known(v) => Some(BaseDocumentRead {
                    facts: v.value.facts.as_ref(),
                    first: v.value.first.map(|id| self.id(id)),
                    render_blocking: self.relation_aliases(Relations::Canonical(
                        &v.value.render_blocking,
                        Key::Existing,
                    )),
                }),
            },
            Source::Staged(DraftNode::Document(n)) => match &n.value().base {
                BaseDocumentGroup::Absent => None,
                BaseDocumentGroup::KnownEmpty => Some(BaseDocumentRead {
                    facts: None,
                    first: None,
                    render_blocking: Relations::Empty,
                }),
                BaseDocumentGroup::Known(v) => Some(match v.read() {
                    GroupRead::Canonical(v) => BaseDocumentRead {
                        facts: v.facts.as_ref(),
                        first: v.first.map(|id| self.id(id)),
                        render_blocking: self.relation_aliases(Relations::Canonical(
                            &v.render_blocking,
                            Key::Existing,
                        )),
                    },
                    GroupRead::Changed(v) => BaseDocumentRead {
                        facts: v.facts.as_ref(),
                        first: v.first.map(|k| self.alias(k)),
                        render_blocking: self.relation_aliases(v.render.view()),
                    },
                }),
            },
            _ => None,
        }
    }
    fn base_render(self) -> Relations<'a> {
        NodeRead {
            overlay: None,
            ..self
        }
        .base_document()
        .map_or(Relations::Empty, |v| v.render_blocking)
    }
    pub fn autofocus_document(self) -> Option<AutofocusRead<'a>> {
        if let Some(Overlay::Autofocus(v)) = self.overlay {
            return v.as_ref().map(|v| match v {
                crate::autofocus::DocumentAutofocusState::Unavailable => AutofocusRead::Unavailable,
                crate::autofocus::DocumentAutofocusState::Known { processed, .. } => {
                    AutofocusRead::Known {
                        processed: *processed,
                        candidates: NodeRead {
                            overlay: None,
                            ..self
                        }
                        .autofocus_candidates(),
                    }
                }
            });
        }
        let (processed, candidates) = match self.source {
            Source::Canonical(Node::Document(n)) => match &n.value.autofocus_document {
                canonical::Autofocus::Absent => return None,
                canonical::Autofocus::Unavailable => return Some(AutofocusRead::Unavailable),
                canonical::Autofocus::KnownEmpty { processed } => (*processed, Relations::Empty),
                canonical::Autofocus::Known(v) => (
                    v.value.processed,
                    Relations::Canonical(&v.value.candidates, Key::Existing),
                ),
            },
            Source::Staged(DraftNode::Document(n)) => match &n.value().autofocus {
                AutofocusGroup::Absent => return None,
                AutofocusGroup::Unavailable => return Some(AutofocusRead::Unavailable),
                AutofocusGroup::KnownEmpty { processed } => (*processed, Relations::Empty),
                AutofocusGroup::Known(v) => match v.read() {
                    GroupRead::Canonical(v) => (
                        v.processed,
                        Relations::Canonical(&v.candidates, Key::Existing),
                    ),
                    GroupRead::Changed(v) => (v.processed, v.candidates.view()),
                },
            },
            _ => return None,
        };
        Some(AutofocusRead::Known {
            processed,
            candidates: self.relation_aliases(candidates),
        })
    }
    pub fn autofocus_candidates(self) -> Relations<'a> {
        match self.autofocus_document() {
            Some(AutofocusRead::Known { candidates, .. }) => candidates,
            _ => Relations::Empty,
        }
    }
    pub fn interactive_document(self) -> Option<InteractiveRead<'a>> {
        if let Some(Overlay::Interactive(v)) = self.overlay {
            return v.as_ref().map(|v| match v {
                crate::dialog::DocumentInteractiveState::Unavailable => {
                    InteractiveRead::Unavailable
                }
                crate::dialog::DocumentInteractiveState::Known { .. } => {
                    let original = NodeRead {
                        overlay: None,
                        ..self
                    };
                    InteractiveRead::Known {
                        open_dialogs: original.open_dialogs(),
                        top_layer: original.top_layer(),
                        pending_top_layer_removals: original.pending_top_layer_removals(),
                    }
                }
            });
        }
        let (open, top, pending) = match self.source {
            Source::Canonical(Node::Document(n)) => match &n.value.interactive_document {
                canonical::Interactive::Absent => return None,
                canonical::Interactive::Unavailable => return Some(InteractiveRead::Unavailable),
                canonical::Interactive::KnownEmpty => {
                    (Relations::Empty, Relations::Empty, Relations::Empty)
                }
                canonical::Interactive::Known(v) => (
                    Relations::Canonical(&v.value.open_dialogs, Key::Existing),
                    Relations::Canonical(&v.value.top_layer, Key::Existing),
                    Relations::Canonical(&v.value.pending_top_layer_removals, Key::Existing),
                ),
            },
            Source::Staged(DraftNode::Document(n)) => match &n.value().interactive {
                InteractiveGroup::Absent => return None,
                InteractiveGroup::Unavailable => return Some(InteractiveRead::Unavailable),
                InteractiveGroup::KnownEmpty => {
                    (Relations::Empty, Relations::Empty, Relations::Empty)
                }
                InteractiveGroup::Known(v) => match v.read() {
                    GroupRead::Canonical(v) => (
                        Relations::Canonical(&v.open_dialogs, Key::Existing),
                        Relations::Canonical(&v.top_layer, Key::Existing),
                        Relations::Canonical(&v.pending_top_layer_removals, Key::Existing),
                    ),
                    GroupRead::Changed(v) => (v.open.view(), v.top.view(), v.pending.view()),
                },
            },
            _ => return None,
        };
        Some(InteractiveRead::Known {
            open_dialogs: self.relation_aliases(open),
            top_layer: self.relation_aliases(top),
            pending_top_layer_removals: self.relation_aliases(pending),
        })
    }
    pub fn open_dialogs(self) -> Relations<'a> {
        match self.interactive_document() {
            Some(InteractiveRead::Known { open_dialogs, .. }) => open_dialogs,
            _ => Relations::Empty,
        }
    }
    pub fn top_layer(self) -> Relations<'a> {
        match self.interactive_document() {
            Some(InteractiveRead::Known { top_layer, .. }) => top_layer,
            _ => Relations::Empty,
        }
    }
    pub fn pending_top_layer_removals(self) -> Relations<'a> {
        match self.interactive_document() {
            Some(InteractiveRead::Known {
                pending_top_layer_removals,
                ..
            }) => pending_top_layer_removals,
            _ => Relations::Empty,
        }
    }
    pub fn logical_units(self) -> Result<usize, DomError> {
        let mut units = match self.kind() {
            KindRead::Document(v) => v.units(),
            KindRead::Element(v) => v.language().units(),
            KindRead::Text(v) | KindRead::Comment(v) => Some(v.len()),
            KindRead::ProcessingInstruction(v) => v.units(),
            KindRead::DocumentType(v) => v.units(),
            _ => Some(0),
        }
        .ok_or(DomError::RepresentationExhausted)?;
        for extra in [
            self.base().map_or(0, |v| v.units()),
            self.hyperlink().map_or(0, |v| v.units()),
            self.base_document().map_or(0, |v| v.units()),
            self.encoding_url_document()
                .map_or(0, EncodingUrlRead::units),
            self.form_past_names().map_or(0, |v| match v {
                NamesRead::Empty => 0,
                NamesRead::Canonical(v) | NamesRead::AliasedCanonical(v, _) => v.logical_units(),
                NamesRead::Staged(v) | NamesRead::AliasedStaged(v, _) => v.logical_units(),
            }),
        ] {
            units = units
                .checked_add(extra)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(units)
    }
}

/// A selected immutable language input retained across destination Draft edits.
/// This owns the original materialization control, never a second occurrence.
pub(in crate::store) enum FrozenNodeInput {
    Document(Controlled<DomDocumentMetadata>),
    Element {
        value: Controlled<DomElement>,
        focus: FocusVersion,
    },
    DocumentFragment,
    ShadowRoot(Controlled<crate::DomShadowRootState>),
    Text(Controlled<canonical::CharacterData>),
    Comment(Controlled<canonical::CharacterData>),
    ProcessingInstruction(Controlled<crate::DomProcessingInstruction>),
    DocumentType(Controlled<crate::DomDocumentType>),
}
impl FrozenNodeInput {
    pub fn kind(&self) -> KindRead<'_> {
        match self {
            Self::Document(v) => KindRead::Document(&v.value.value),
            Self::Element { value, focus } => KindRead::Element(ElementRead {
                value: &value.value.value,
                focus: Some(focus),
                aliases: None,
            }),
            Self::DocumentFragment => KindRead::DocumentFragment,
            Self::ShadowRoot(v) => KindRead::ShadowRoot(&v.value.value),
            Self::Text(v) => KindRead::Text(&v.value.value.value),
            Self::Comment(v) => KindRead::Comment(&v.value.value.value),
            Self::ProcessingInstruction(v) => KindRead::ProcessingInstruction(&v.value.value),
            Self::DocumentType(v) => KindRead::DocumentType(&v.value.value),
        }
    }
}
impl NodeRead<'_> {
    fn freeze(
        self,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<FrozenNodeInput, DomError> {
        use FrozenNodeInput as Frozen;
        let focus = self.element().map(|e| {
            e.common_state()
                .html()
                .and_then(|v| v.previously_focused_element())
                .copied()
        });
        Ok(match self.source {
            Source::Canonical(node) => match node {
                Node::Document(n) => {
                    Frozen::Document(controlled(&n.value.metadata, registry, &mut admit)?)
                }
                Node::Element(n) => Frozen::Element {
                    value: controlled(&n.value.element, registry, &mut admit)?,
                    focus: focus
                        .flatten()
                        .map_or(FocusVersion::Cleared, FocusVersion::Staged),
                },
                Node::DocumentFragment(_) => Frozen::DocumentFragment,
                Node::ShadowRoot(n) => {
                    Frozen::ShadowRoot(controlled(&n.value.state, registry, &mut admit)?)
                }
                Node::Text(n) => Frozen::Text(controlled(&n.value.data, registry, &mut admit)?),
                Node::Comment(n) => {
                    Frozen::Comment(controlled(&n.value.data, registry, &mut admit)?)
                }
                Node::ProcessingInstruction(n) => {
                    Frozen::ProcessingInstruction(controlled(&n.value.data, registry, &mut admit)?)
                }
                Node::DocumentType(n) => {
                    Frozen::DocumentType(controlled(&n.value.data, registry, &mut admit)?)
                }
            },
            Source::Staged(node) => match node {
                DraftNode::Document(n) => Frozen::Document(n.value().metadata.clone()),
                DraftNode::Element(n) => Frozen::Element {
                    value: n.value().element.clone(),
                    focus: focus
                        .flatten()
                        .map_or(FocusVersion::Cleared, FocusVersion::Staged),
                },
                DraftNode::DocumentFragment(_) => Frozen::DocumentFragment,
                DraftNode::ShadowRoot(n) => Frozen::ShadowRoot(n.value().state.clone()),
                DraftNode::Text(n) => Frozen::Text(n.value().data.clone()),
                DraftNode::Comment(n) => Frozen::Comment(n.value().data.clone()),
                DraftNode::ProcessingInstruction(n) => {
                    Frozen::ProcessingInstruction(n.value().data.clone())
                }
                DraftNode::DocumentType(n) => Frozen::DocumentType(n.value().data.clone()),
            },
        })
    }
}
impl Draft {
    pub(in crate::store) fn freeze_node_input(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<FrozenNodeInput, DomError> {
        self.node(key)?.freeze(self.custody_registry(), |usage| {
            self.admit_custody_usage(usage, cancel)
        })
    }
    pub(in crate::store) fn freeze_published_node_input(
        &self,
        snapshot: &crate::DomSnapshot,
        id: NodeId,
        cancel: &AtomicBool,
    ) -> Result<FrozenNodeInput, DomError> {
        let node = snapshot.state.node(snapshot.owner, id)?;
        NodeRead::canonical(node).freeze(self.custody_registry(), |usage| {
            self.admit_custody_usage(usage, cancel)
        })
    }
}
