//! Selected reference-bearing feature edits. A candidate owns only the one
//! feature it replaces; other language and relation children stay shared.
use super::super::*;
use super::fields::{FieldAccess, FieldUndo, SelectedFieldsMut, group, inline, transform};
use super::records::Overlay;
use super::staged::*;

fn optional_usage<T>(
    value: &Option<T>,
    describe: fn(&T) -> Result<custody::Usage, DomError>,
) -> Result<custody::Usage, DomError> {
    let extra = value
        .as_ref()
        .map(describe)
        .transpose()?
        .map_or(Ok(custody::Usage::ZERO), |usage| {
            usage.checked_sub(custody::Usage::new(size_of::<payload::Payload<T>>(), 0))
        })?;
    custody::Usage::new(size_of::<Option<T>>(), 0).checked_add(extra)
}
fn expected<T>(extra: source::storage::Payload) -> Result<custody::Usage, DomError> {
    custody::Usage::new(size_of::<Option<T>>(), 0)
        .checked_add(custody::Usage::new(extra.bytes, extra.units))
}
fn base_usage(value: &Option<crate::base::BaseState<Key>>) -> Result<custody::Usage, DomError> {
    optional_usage(value, source::storage::base)
}
fn hyperlink_usage(
    value: &Option<crate::hyperlink::HyperlinkState<Key>>,
) -> Result<custody::Usage, DomError> {
    optional_usage(value, source::storage::hyperlink)
}
fn image_usage(value: &Option<crate::image::ImageState<Key>>) -> Result<custody::Usage, DomError> {
    optional_usage(value, source::storage::image)
}

fn prepare_base(
    draft: &Draft,
    _key: Key,
    incoming: custody::Held<Box<Option<crate::base::BaseState<Key>>>>,
    cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    let next = if incoming.value().is_some() {
        Associations::Base(group(transform(incoming, draft, cancel, |v| {
            v.expect("selected Base")
        })?))
    } else {
        drop(incoming);
        Associations::None
    };
    Ok(FieldUndo::Associations(next))
}
fn prepare_hyperlink(
    draft: &Draft,
    _key: Key,
    incoming: custody::Held<Box<Option<crate::hyperlink::HyperlinkState<Key>>>>,
    cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    let next = if incoming.value().is_some() {
        Associations::Hyperlink(group(transform(incoming, draft, cancel, |v| {
            v.expect("selected Hyperlink")
        })?))
    } else {
        drop(incoming);
        Associations::None
    };
    Ok(FieldUndo::Associations(next))
}
fn prepare_image(
    draft: &Draft,
    key: Key,
    incoming: custody::Held<Box<Option<crate::image::ImageState<Key>>>>,
    cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    if incoming.value().is_none() {
        drop(incoming);
        return Ok(FieldUndo::Associations(Associations::None));
    }
    let image = group(transform(incoming, draft, cancel, |v| {
        v.expect("selected Image")
    })?);
    if matches!(
        draft.records_for(key).associations(),
        Some(Associations::Image(Group::Changed(_)))
    ) {
        return Ok(FieldUndo::ImageState(image));
    }
    let owner = draft.node(key)?.control_owner();
    let registry = draft.custody_registry().clone();
    let wrapper = boxed(
        &registry,
        &mut |usage| draft.admit_custody_usage(usage, cancel),
        || {
            Ok(ImageFields {
                image,
                control_owner: owner,
            })
        },
    )?;
    Ok(FieldUndo::Associations(Associations::Image(group(wrapper))))
}

impl Draft {
    pub(in crate::store) fn base_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::base::BaseState<Key>>>, DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        let mut extra = source::storage::Payload::default();
        if let Some(frozen) = self.node(key)?.base().and_then(|v| v.frozen) {
            extra.producer(frozen.value.retained_storage_usage())?;
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::base::BaseState<Key>>(extra)?,
            |node| {
                Ok(node.base().map(|v| crate::base::BaseState {
                    frozen: v.frozen.map(|f| crate::base::Frozen {
                        value: f.value.clone(),
                        document: f.document,
                        revision: f.revision.clone(),
                        response: f.response,
                    }),
                }))
            },
            FieldAccess {
                usage: base_usage,
                overlay: |value| Overlay::Base(value),
                finish: prepare_base,
            },
        )
    }
    pub(in crate::store) fn replace_base_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        build: impl FnOnce() -> Result<Option<crate::base::BaseState<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::base::BaseState<Key>>(extra)?,
            |_| build(),
            FieldAccess {
                usage: base_usage,
                overlay: |value| Overlay::Base(value),
                finish: prepare_base,
            },
        )?
        .commit(cancel)
    }
    pub(in crate::store) fn hyperlink_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::hyperlink::HyperlinkState<Key>>>, DomError>
    {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        let mut extra = source::storage::Payload::default();
        if let Some(parsed) = self.node(key)?.hyperlink().and_then(|v| v.parsed) {
            extra.hyperlink_snapshot(parsed.value)?;
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::hyperlink::HyperlinkState<Key>>(extra)?,
            |node| {
                Ok(node.hyperlink().map(|v| crate::hyperlink::HyperlinkState {
                    parsed: v.parsed.map(|p| crate::hyperlink::Parsed {
                        document: p.document,
                        value: p.value.clone(),
                    }),
                }))
            },
            FieldAccess {
                usage: hyperlink_usage,
                overlay: |value| Overlay::Hyperlink(value),
                finish: prepare_hyperlink,
            },
        )
    }
    pub(in crate::store) fn replace_hyperlink_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        build: impl FnOnce() -> Result<Option<crate::hyperlink::HyperlinkState<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::hyperlink::HyperlinkState<Key>>(extra)?,
            |_| build(),
            FieldAccess {
                usage: hyperlink_usage,
                overlay: |value| Overlay::Hyperlink(value),
                finish: prepare_hyperlink,
            },
        )?
        .commit(cancel)
    }
    pub(in crate::store) fn image_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::image::ImageState<Key>>>, DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        let mut extra = source::storage::Payload::default();
        if let Some(value) = self.node(key)?.image() {
            extra.string(value.native.current_request().current_url())?;
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::image::ImageState<Key>>(extra)?,
            |node| {
                Ok(node.image().map(|v| crate::image::ImageState {
                    native: v.native.clone(),
                    dimension_source: v.dimension_source,
                    parser_inserted: v.parser_inserted,
                    high_water: v.high_water,
                    waiting: v.waiting,
                }))
            },
            FieldAccess {
                usage: image_usage,
                overlay: |value| Overlay::Image(value),
                finish: prepare_image,
            },
        )
    }
    pub(in crate::store) fn replace_image_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        build: impl FnOnce() -> Result<Option<crate::image::ImageState<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::image::ImageState<Key>>(extra)?,
            |_| build(),
            FieldAccess {
                usage: image_usage,
                overlay: |value| Overlay::Image(value),
                finish: prepare_image,
            },
        )?
        .commit(cancel)
    }
    pub(in crate::store) fn link_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::base::LinkState<Key>>>, DomError> {
        if self.node(key)?.element().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(size_of::<Option<crate::base::LinkState<Key>>>(), 0),
            |node| {
                Ok(node.link().map(|v| crate::base::LinkState {
                    flags: v.flags,
                    parser_document: v.parser_document,
                    sheet: v.sheet.cloned(),
                }))
            },
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::Link(value),
                finish: |draft, _key, incoming, cancel| {
                    let next = if incoming.value().is_some() {
                        Associations::Link(group(transform(incoming, draft, cancel, |v| {
                            v.expect("selected Link")
                        })?))
                    } else {
                        drop(incoming);
                        Associations::None
                    };
                    Ok(FieldUndo::Associations(next))
                },
            },
        )
    }
}

fn prepare_interactive(
    draft: &Draft,
    key: Key,
    incoming: custody::Held<Box<Option<crate::dialog::DocumentInteractiveState<Key>>>>,
    _cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    let next = match incoming.value().as_ref() {
        None => InteractiveGroup::Absent,
        Some(crate::dialog::DocumentInteractiveState::Unavailable) => InteractiveGroup::Unavailable,
        Some(crate::dialog::DocumentInteractiveState::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) => {
            if !open_dialogs.is_empty()
                || !top_layer.is_empty()
                || !pending_top_layer_removals.is_empty()
            {
                return Err(DomError::InvalidSourceOperation);
            }
            if matches!(
                draft
                    .records_for(key)
                    .document_fields()
                    .expect("Document")
                    .interactive,
                InteractiveGroup::KnownEmpty | InteractiveGroup::Known(_)
            ) {
                drop(incoming);
                return Ok(FieldUndo::Unchanged);
            }
            InteractiveGroup::KnownEmpty
        }
    };
    drop(incoming);
    Ok(FieldUndo::Interactive(next))
}
fn prepare_autofocus(
    draft: &Draft,
    key: Key,
    incoming: custody::Held<Box<Option<crate::autofocus::DocumentAutofocusState<Key>>>>,
    _cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    let next = match incoming.value().as_ref() {
        None => AutofocusGroup::Absent,
        Some(crate::autofocus::DocumentAutofocusState::Unavailable) => AutofocusGroup::Unavailable,
        Some(crate::autofocus::DocumentAutofocusState::Known {
            candidates,
            processed,
        }) => {
            if !candidates.is_empty() {
                return Err(DomError::InvalidSourceOperation);
            }
            if matches!(
                &draft
                    .records_for(key)
                    .document_fields()
                    .expect("Document")
                    .autofocus,
                AutofocusGroup::KnownEmpty { .. } | AutofocusGroup::Known(Group::Changed(_))
            ) {
                let next = *processed;
                drop(incoming);
                return Ok(FieldUndo::AutofocusProcessed(next));
            }
            if matches!(
                &draft
                    .records_for(key)
                    .document_fields()
                    .expect("Document")
                    .autofocus,
                AutofocusGroup::Known(Group::Canonical(_))
            ) {
                unreachable!("prepared autofocus metadata");
            }
            AutofocusGroup::KnownEmpty {
                processed: *processed,
            }
        }
    };
    drop(incoming);
    Ok(FieldUndo::Autofocus(next))
}

fn encoding_usage(
    value: &Option<crate::hyperlink::EncodingUrlDocument>,
) -> Result<custody::Usage, DomError> {
    optional_usage(value, source::storage::encoding_url)
}
fn prepare_encoding(
    draft: &Draft,
    _key: Key,
    incoming: custody::Held<Box<Option<crate::hyperlink::EncodingUrlDocument>>>,
    cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    let next = match incoming.value().as_ref() {
        None => {
            drop(incoming);
            EncodingGroup::Absent
        }
        Some(value) if value.facts.is_none() => {
            drop(incoming);
            EncodingGroup::KnownEmpty
        }
        Some(_) => {
            let incoming = transform(incoming, draft, cancel, |v| {
                v.expect("selected encoding facts")
            })?;
            let usage = source::storage::encoding_url(incoming.value())?;
            EncodingGroup::Known(super::fields::seal_language(
                incoming, usage, draft, cancel, None,
            )?)
        }
    };
    Ok(FieldUndo::Encoding(next))
}
impl Draft {
    pub(in crate::store) fn interactive_document_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::dialog::DocumentInteractiveState<Key>>>, DomError>
    {
        if self.node(key)?.document_metadata().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(
                size_of::<Option<crate::dialog::DocumentInteractiveState<Key>>>(),
                0,
            ),
            |node| {
                Ok(node.interactive_document().map(|v| match v {
                    super::records::InteractiveRead::Unavailable => {
                        crate::dialog::DocumentInteractiveState::Unavailable
                    }
                    super::records::InteractiveRead::Known { .. } => {
                        crate::dialog::DocumentInteractiveState::native_closed()
                    }
                }))
            },
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::Interactive(value),
                finish: prepare_interactive,
            },
        )
    }
    pub(in crate::store) fn autofocus_document_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<
        SelectedFieldsMut<'_, Option<crate::autofocus::DocumentAutofocusState<Key>>>,
        DomError,
    > {
        if self.node(key)?.document_metadata().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        if matches!(
            self.node(key)?.autofocus_document(),
            Some(super::records::AutofocusRead::Known { .. })
        ) {
            self.prepare_relation_owner(key, super::RelationField::AutofocusCandidates, cancel)?;
        }
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(
                size_of::<Option<crate::autofocus::DocumentAutofocusState<Key>>>(),
                0,
            ),
            |node| {
                Ok(node.autofocus_document().map(|v| match v {
                    super::records::AutofocusRead::Unavailable => {
                        crate::autofocus::DocumentAutofocusState::Unavailable
                    }
                    super::records::AutofocusRead::Known { processed, .. } => {
                        crate::autofocus::DocumentAutofocusState::Known {
                            candidates: Vec::new(),
                            processed,
                        }
                    }
                }))
            },
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::Autofocus(value),
                finish: prepare_autofocus,
            },
        )
    }
}

pub(in crate::store) struct BaseMetadata {
    pub facts: Option<surgeist_html_semantics::HtmlParseBaseFacts>,
    pub first: Option<Key>,
    pub credit: MetadataCredit,
}
impl BaseMetadata {
    #[cfg(test)]
    pub(super) fn journal_bytes(&self) -> usize {
        match &self.credit {
            MetadataCredit::Owned { _debit: debit } => debit.usage().bytes(),
            MetadataCredit::Captured { .. } | MetadataCredit::None => 0,
        }
    }
}
pub(super) fn swap_base_metadata(
    node: &mut DraftNode,
    mut incoming: custody::Held<Box<BaseMetadata>>,
) -> custody::Held<Box<BaseMetadata>> {
    let BaseDocumentGroup::Known(Group::Changed(fields)) = &mut node
        .document_fields_mut()
        .expect("Base metadata Document")
        .base
    else {
        unreachable!("prepared Base metadata swap")
    };
    let current = fields.value_mut();
    std::mem::swap(&mut current.facts, &mut incoming.value_mut().facts);
    std::mem::swap(&mut current.first, &mut incoming.value_mut().first);
    std::mem::swap(
        &mut current.metadata_credit,
        &mut incoming.value_mut().credit,
    );
    incoming
}

fn base_document_usage(
    value: &Option<crate::base::BaseDocument<Key>>,
) -> Result<custody::Usage, DomError> {
    let mut extra = source::storage::Payload::default();
    if let Some(value) = value {
        if !value.render_blocking.is_empty() {
            return Err(DomError::InvalidSourceOperation);
        }
        if let Some(facts) = &value.facts {
            extra.producer(facts.retained_storage_usage())?;
        }
    }
    expected::<crate::base::BaseDocument<Key>>(extra)
}
fn prepare_base_document(
    draft: &Draft,
    key: Key,
    incoming: custody::Held<Box<Option<crate::base::BaseDocument<Key>>>>,
    cancel: &AtomicBool,
) -> Result<FieldUndo, DomError> {
    if incoming.value().is_none() {
        drop(incoming);
        return Ok(FieldUndo::BaseDocument(BaseDocumentGroup::Absent));
    }
    let mut incoming = transform(incoming, draft, cancel, |value| {
        let value = value.expect("selected Base Document");
        BaseMetadata {
            facts: value.facts,
            first: value.first,
            credit: MetadataCredit::None,
        }
    })?;
    let mut extra = source::storage::Payload::default();
    if let Some(facts) = &incoming.value().facts {
        extra.producer(facts.retained_storage_usage())?;
    }
    let credit = incoming
        .debit_mut()
        .split(custody::Usage::new(extra.bytes, extra.units))?;
    incoming.value_mut().credit = MetadataCredit::Owned { _debit: credit };
    if matches!(
        &draft
            .records_for(key)
            .document_fields()
            .expect("Document")
            .base,
        BaseDocumentGroup::Known(Group::Changed(_))
    ) {
        return Ok(FieldUndo::BaseDocumentMetadata(incoming));
    }
    let render = super::Sequence::empty(Key::Existing, draft.custody_ledger(), |usage| {
        draft.admit_custody_usage(usage, cancel)
    })?;
    let next = transform(incoming, draft, cancel, |metadata| BaseDocumentFields {
        facts: metadata.facts,
        first: metadata.first,
        render,
        metadata_credit: metadata.credit,
    })?;
    Ok(FieldUndo::BaseDocument(BaseDocumentGroup::Known(group(
        next,
    ))))
}

impl Draft {
    fn prepare_base_document_metadata(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.node(key)?.document_metadata().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        if self.node(key)?.base_document().is_some() {
            self.prepare_relation_owner(key, super::RelationField::RenderBlocking, cancel)?;
        }
        Ok(())
    }
    pub(in crate::store) fn base_document_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<crate::base::BaseDocument<Key>>>, DomError> {
        self.prepare_base_document_metadata(key, cancel)?;
        let mut extra = source::storage::Payload::default();
        if let Some(facts) = self
            .node(key)?
            .base_document()
            .and_then(|value| value.facts)
        {
            extra.producer(facts.retained_storage_usage())?;
        }
        self.selected_fields(
            key,
            cancel,
            expected::<crate::base::BaseDocument<Key>>(extra)?,
            |node| {
                Ok(node.base_document().map(|value| crate::base::BaseDocument {
                    facts: value.facts.cloned(),
                    first: value.first,
                    render_blocking: Vec::new(),
                }))
            },
            FieldAccess {
                usage: base_document_usage,
                overlay: |value| Overlay::BaseDocument(value),
                finish: prepare_base_document,
            },
        )
    }
    pub(in crate::store) fn replace_base_document_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        build: impl FnOnce() -> Result<Option<crate::base::BaseDocument<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.prepare_base_document_metadata(key, cancel)?;
        self.selected_fields(
            key,
            cancel,
            expected::<crate::base::BaseDocument<Key>>(extra)?,
            |_| build(),
            FieldAccess {
                usage: base_document_usage,
                overlay: |value| Overlay::BaseDocument(value),
                finish: prepare_base_document,
            },
        )?
        .commit(cancel)
    }
}

impl Draft {
    pub(in crate::store) fn establish_child_navigable(
        &mut self,
        key: Key,
        next: Option<crate::DomHostedNavigableId>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let current = self
            .node(key)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .child_navigable()
            .ok_or(DomError::WrongNodeKind)?;
        if current == next {
            return Ok(());
        }
        let registry = self.custody_registry().clone();
        let mut admit = |usage| self.admit_custody_usage(usage, cancel);
        let mut formation =
            crate::element::ElementFormation::new(&registry, None, cancel, &mut admit);
        let hosted = next.map(|value| formation.hosted(value)).transpose()?;
        let mut element = self
            .node(key)?
            .element()
            .expect("validated Element")
            .language()
            .clone();
        element.set_hosted_payload(hosted)?;
        let formed = formation.finish(element);
        self.replace_element_formed(key, formed, None, cancel)
    }
    pub(in crate::store) fn establish_element_registry(
        &mut self,
        key: Key,
        binding: crate::DomRegistryBinding,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let element = self
            .node(key)?
            .element()
            .ok_or(DomError::WrongNodeKind)?
            .language();
        if element.custom_element_registry() == binding {
            return Ok(());
        }
        let mut extra = source::storage::Payload::default();
        if let Some(value) = element.saved_is() {
            extra.string(value)?;
        }
        let formed = payload::form_group(
            self.custody_registry(),
            None,
            source::storage::Payload::default(),
            source::storage::typed::<crate::element::CustomAssociation>(extra)?,
            &mut |usage| self.admit_custody_usage(usage, cancel),
            || {
                Ok(crate::element::CustomAssociation {
                    registry: binding,
                    definition: element.custom_element_definition(),
                    saved_is: element.saved_is().cloned(),
                })
            },
        )?;
        let mut element = element.clone();
        element.set_custom_payload(Some(formed.value));
        let children = crate::element::ElementChildren {
            custom: Some(formed.custody),
            ..crate::element::ElementChildren::default()
        };
        self.replace_element_formed(
            key,
            crate::element::FormedElement {
                value: element,
                children,
            },
            None,
            cancel,
        )
    }
}

impl Draft {
    pub(in crate::store) fn replace_base_formed_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        formation: Option<source::formation::Formation>,
        build: impl FnOnce(&Draft) -> Result<Option<crate::base::BaseState<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.selected_fields_formed(
            key,
            cancel,
            formation,
            expected::<crate::base::BaseState<Key>>(extra)?,
            |draft, _| build(draft),
            FieldAccess {
                usage: base_usage,
                overlay: |value| Overlay::Base(value),
                finish: prepare_base,
            },
        )?
        .commit(cancel)
    }
    pub(in crate::store) fn replace_base_document_formed_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        formation: Option<source::formation::Formation>,
        build: impl FnOnce(&Draft) -> Result<Option<crate::base::BaseDocument<Key>>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        struct Incoming<F> {
            build: F,
            formation: Option<source::formation::Formation>,
        }
        let incoming = Incoming { build, formation };
        self.prepare_base_document_metadata(key, cancel)?;
        self.selected_fields_formed(
            key,
            cancel,
            incoming.formation,
            expected::<crate::base::BaseDocument<Key>>(extra)?,
            |draft, _| (incoming.build)(draft),
            FieldAccess {
                usage: base_document_usage,
                overlay: |value| Overlay::BaseDocument(value),
                finish: prepare_base_document,
            },
        )?
        .commit(cancel)
    }
    pub(in crate::store) fn replace_encoding_url_document_formed_from(
        &mut self,
        key: Key,
        extra: source::storage::Payload,
        formation: Option<source::formation::Formation>,
        build: impl FnOnce(&Draft) -> Result<Option<crate::hyperlink::EncodingUrlDocument>, DomError>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.selected_fields_formed(
            key,
            cancel,
            formation,
            expected::<crate::hyperlink::EncodingUrlDocument>(extra)?,
            |draft, _| build(draft),
            FieldAccess {
                usage: encoding_usage,
                overlay: |value| Overlay::Encoding(value),
                finish: prepare_encoding,
            },
        )?
        .commit(cancel)
    }
}
