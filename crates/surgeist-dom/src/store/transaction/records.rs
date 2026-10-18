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
            Self::Canonical(v) | Self::AliasedCanonical(v, _) => Self::AliasedCanonical(v, ctx),
            Self::Staged(v) | Self::AliasedStaged(v, _) => Self::AliasedStaged(v, ctx),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Canonical(values) | Self::AliasedCanonical(values, _) => values.entries().len(),
            Self::Staged(values) | Self::AliasedStaged(values, _) => values.entries().len(),
        }
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = NameRead<'_>> {
        (0..self.len()).map(|index| match self {
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

#[derive(Clone, Copy)]
pub(in crate::store) struct NodeRead<'a> {
    pub fullscreen_events: Events<'a>,
    pub active_range: crate::DomActiveRangeState,
    pub relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    pub document: Key,
    pub kind: &'a DomNodeKind,
    pub parent: Option<Key>,
    pub children: Relations<'a>,
    pub shadow_root: Option<Key>,
    pub host: Option<Key>,
    pub template_contents: Option<Key>,
    pub template_patch: Option<crate::template::TemplatePatch<Key>>,
    pub template_document: Option<crate::template::TemplateDocument<Key>>,
    pub manual_nodes: Relations<'a>,
    pub assigned_nodes: Relations<'a>,
    pub manual_slot: Option<Key>,
    pub assigned_slot: Option<Key>,
    pub reflected_targets: Option<crate::reflected::ReflectedTargets<Key>>,
    pub control_owner: Option<Key>,
    pub option_select: Option<Key>,
    pub form_past_names: Option<NamesRead<'a>>,
    pub script: Option<crate::script::ScriptState<Key>>,
    pub style_parser_document: Option<Key>,
    pub base: Option<BaseRead<'a>>,
    pub base_document: Option<BaseDocumentRead<'a>>,
    pub link: Option<LinkRead<'a>>,
    pub image: Option<ImageRead<'a>>,
    pub hyperlink: Option<HyperlinkRead<'a>>,
    pub encoding_url_document: Option<&'a crate::hyperlink::EncodingUrlDocument>,
    pub interactive_document: Option<InteractiveRead<'a>>,
    pub autofocus_document: Option<AutofocusRead<'a>>,
}

impl<'a> NodeRead<'a> {
    pub fn staged(node: &'a DraftNode) -> Self {
        Self::staged_fields(node, node.fields.value())
    }
    pub fn staged_fields(node: &'a DraftNode, fields: &'a super::NodeFields) -> Self {
        Self {
            fullscreen_events: node.fullscreen_events.view(),
            active_range: fields.active_range,
            relevant_global: fields.relevant_global,
            document: fields.document,
            kind: &fields.kind,
            parent: fields.parent,
            children: node.children.view(),
            shadow_root: fields.shadow_root,
            host: fields.host,
            template_contents: fields.template_contents,
            template_patch: fields.template_patch,
            template_document: fields.template_document,
            manual_nodes: node.manual_nodes.view(),
            assigned_nodes: node.assigned_nodes.view(),
            manual_slot: fields.manual_slot,
            assigned_slot: fields.assigned_slot,
            reflected_targets: fields.reflected_targets,
            control_owner: fields.control_owner,
            option_select: fields.option_select,
            form_past_names: fields.form_past_names.as_ref().map(super::Names::view),
            script: fields.script,
            style_parser_document: fields.style_parser_document,
            base: fields.base.as_ref().map(|state| BaseRead {
                frozen: state.frozen.as_ref().map(|frozen| FrozenRead {
                    value: &frozen.value,
                    document: frozen.document,
                    revision: &frozen.revision,
                    response: frozen.response,
                }),
            }),
            base_document: fields.base_document.as_ref().map(|state| BaseDocumentRead {
                facts: state.facts.as_ref(),
                first: state.first,
                render_blocking: node.render_blocking.view(),
            }),
            link: fields.link.as_ref().map(|state| LinkRead {
                flags: state.flags,
                parser_document: state.parser_document,
                sheet: state.sheet.as_ref(),
            }),
            image: fields.image.as_ref().map(|state| ImageRead {
                native: &state.native,
                dimension_source: state.dimension_source,
                parser_inserted: state.parser_inserted,
                high_water: state.high_water,
                waiting: state.waiting,
            }),
            hyperlink: fields.hyperlink.as_ref().map(|state| HyperlinkRead {
                parsed: state.parsed.as_ref().map(|parsed| ParsedRead {
                    document: parsed.document,
                    value: &parsed.value,
                }),
            }),
            encoding_url_document: fields.encoding_url_document.as_ref(),
            interactive_document: fields
                .interactive_document
                .as_ref()
                .map(|state| match state {
                    crate::dialog::DocumentInteractiveState::Known { .. } => {
                        InteractiveRead::Known {
                            open_dialogs: node.open_dialogs.view(),
                            top_layer: node.top_layer.view(),
                            pending_top_layer_removals: node.pending_top_layer_removals.view(),
                        }
                    }
                    crate::dialog::DocumentInteractiveState::Unavailable => {
                        InteractiveRead::Unavailable
                    }
                }),
            autofocus_document: fields.autofocus_document.as_ref().map(|state| match state {
                crate::autofocus::DocumentAutofocusState::Known { processed, .. } => {
                    AutofocusRead::Known {
                        candidates: node.autofocus_candidates.view(),
                        processed: *processed,
                    }
                }
                crate::autofocus::DocumentAutofocusState::Unavailable => AutofocusRead::Unavailable,
            }),
        }
    }

    pub fn canonical(node: &'a Node) -> Self {
        let id = Key::Existing;
        Self {
            fullscreen_events: Events::Canonical(&node.fullscreen_events, std::convert::identity),
            active_range: node.active_range,
            relevant_global: node.relevant_global,
            document: id(node.document.0),
            kind: &node.kind,
            parent: node.parent.map(id),
            children: Relations::Canonical(&node.children, id),
            shadow_root: node.shadow_root.map(id),
            host: node.host.map(id),
            template_contents: node.template_contents.map(id),
            template_patch: node.template_patch.map(|value| value.map(id)),
            template_document: node.template_document.map(|value| value.map(id)),
            manual_nodes: Relations::Canonical(&node.manual_nodes, id),
            assigned_nodes: Relations::Canonical(&node.assigned_nodes, id),
            manual_slot: node.manual_slot.map(id),
            assigned_slot: node.assigned_slot.map(id),
            reflected_targets: node.reflected_targets.map(|value| value.map(id)),
            control_owner: node.control_owner.map(id),
            option_select: node.option_select.map(id),
            form_past_names: node.form_past_names.as_ref().map(NamesRead::Canonical),
            script: node.script.map(|value| value.map(id)),
            style_parser_document: node.style_parser_document.map(id),
            base: node.base.as_ref().map(|state| BaseRead {
                frozen: state.frozen.as_ref().map(|frozen| FrozenRead {
                    value: &frozen.value,
                    document: id(frozen.document),
                    revision: &frozen.revision,
                    response: frozen.response,
                }),
            }),
            base_document: node.base_document.as_ref().map(|state| BaseDocumentRead {
                facts: state.facts.as_ref(),
                first: state.first.map(id),
                render_blocking: Relations::Canonical(&state.render_blocking, id),
            }),
            link: node.link.as_ref().map(|state| LinkRead {
                flags: state.flags,
                parser_document: state.parser_document.map(id),
                sheet: state.sheet.as_ref(),
            }),
            image: node.image.as_ref().map(|state| ImageRead {
                native: &state.native,
                dimension_source: id(state.dimension_source),
                parser_inserted: state.parser_inserted,
                high_water: state.high_water,
                waiting: state.waiting,
            }),
            hyperlink: node.hyperlink.as_ref().map(|state| HyperlinkRead {
                parsed: state.parsed.as_ref().map(|parsed| ParsedRead {
                    document: id(parsed.document),
                    value: &parsed.value,
                }),
            }),
            encoding_url_document: node.encoding_url_document.as_ref(),
            interactive_document: node.interactive_document.as_ref().map(|state| match state {
                canonical::Interactive::Known {
                    open_dialogs,
                    top_layer,
                    pending_top_layer_removals,
                } => InteractiveRead::Known {
                    open_dialogs: Relations::Canonical(open_dialogs, id),
                    top_layer: Relations::Canonical(top_layer, id),
                    pending_top_layer_removals: Relations::Canonical(
                        pending_top_layer_removals,
                        id,
                    ),
                },
                canonical::Interactive::Unavailable => InteractiveRead::Unavailable,
            }),
            autofocus_document: node.autofocus_document.as_ref().map(|state| match state {
                canonical::Autofocus::Known {
                    candidates,
                    processed,
                } => AutofocusRead::Known {
                    candidates: Relations::Canonical(candidates, id),
                    processed: *processed,
                },
                canonical::Autofocus::Unavailable => AutofocusRead::Unavailable,
            }),
        }
    }
}

impl<'a> NodeRead<'a> {
    pub fn aliases(mut self, ctx: &'a source::AllocationIds) -> Self {
        let map = |key| source::alias_key(key, ctx);
        self.document = map(self.document);
        self.parent = self.parent.map(map);
        self.shadow_root = self.shadow_root.map(map);
        self.host = self.host.map(map);
        self.template_contents = self.template_contents.map(map);
        self.template_patch = self.template_patch.map(|v| v.map(map));
        self.template_document = self.template_document.map(|v| v.map(map));
        self.manual_slot = self.manual_slot.map(map);
        self.assigned_slot = self.assigned_slot.map(map);
        self.reflected_targets = self.reflected_targets.map(|v| v.map(map));
        self.control_owner = self.control_owner.map(map);
        self.option_select = self.option_select.map(map);
        self.script = self.script.map(|v| v.map(map));
        self.style_parser_document = self.style_parser_document.map(map);
        for relation in [
            &mut self.children,
            &mut self.manual_nodes,
            &mut self.assigned_nodes,
        ] {
            *relation = relation.aliases(ctx, source::alias_key);
        }
        if let Some(base) = &mut self.base
            && let Some(frozen) = &mut base.frozen
        {
            frozen.document = map(frozen.document);
        }
        if let Some(base) = &mut self.base_document {
            base.first = base.first.map(map);
            base.render_blocking = base.render_blocking.aliases(ctx, source::alias_key);
        }
        if let Some(link) = &mut self.link {
            link.parser_document = link.parser_document.map(map);
        }
        if let Some(image) = &mut self.image {
            image.dimension_source = map(image.dimension_source);
        }
        if let Some(hyperlink) = &mut self.hyperlink
            && let Some(parsed) = &mut hyperlink.parsed
        {
            parsed.document = map(parsed.document);
        }
        self.form_past_names = self.form_past_names.map(|v| v.aliases(ctx));
        if let Some(AutofocusRead::Known { candidates, .. }) = &mut self.autofocus_document {
            *candidates = candidates.aliases(ctx, source::alias_key);
        }
        if let Some(InteractiveRead::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = &mut self.interactive_document
        {
            for list in [open_dialogs, top_layer, pending_top_layer_removals] {
                *list = list.aliases(ctx, source::alias_key);
            }
        }
        self
    }
}
