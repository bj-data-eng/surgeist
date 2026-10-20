//! A provisional edit owns only its selected typed group. The shared language
//! and reference children of every other group keep their original identity.
use super::super::*;
use super::{NodeRead, records::Overlay, staged::*};
use canonical::*;

pub(in crate::store) enum FieldUndo {
    Unchanged,
    AutofocusProcessed(bool),
    BaseDocumentMetadata(custody::Held<Box<super::feature_fields::BaseMetadata>>),
    Common(Common<Key>),
    Focus(FocusVersion),
    Element(Controlled<DomElement>, FocusVersion),
    ImageOwner(Option<Key>),
    ImageState(Group<crate::image::ImageState<NodeId>, crate::image::ImageState<Key>>),
    Metadata(Controlled<DomDocumentMetadata>),
    ShadowState(Controlled<crate::DomShadowRootState>),
    Text(Controlled<CharacterData>),
    Comment(Controlled<CharacterData>),
    ProcessingInstruction(Controlled<crate::DomProcessingInstruction>),
    Slottable(Option<Group<Slottable<NodeId>, Slottable<Key>>>),
    Shadow(Option<Group<ShadowHost<NodeId>, ShadowHost<Key>>>),
    Host(Option<Key>),
    Associations(Associations),
    TemplateDocument(Option<crate::template::TemplateDocument<Key>>),
    BaseDocument(BaseDocumentGroup),
    Encoding(EncodingGroup),
    Interactive(InteractiveGroup),
    Autofocus(AutofocusGroup),
    ActiveRange(crate::DomActiveRangeState),
    Children(Option<Group<ContainerChildren<NodeId>, Children>>),
    FullscreenGroup(
        Option<
            Group<
                FullscreenQueue,
                super::Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent>,
            >,
        >,
    ),
}
impl FieldUndo {
    pub fn restore(self, node: &mut DraftNode) {
        drop(self.swap(node));
    }
    pub(super) fn swap(self, node: &mut DraftNode) -> Self {
        match self {
            Self::Unchanged => Self::Unchanged,
            Self::BaseDocumentMetadata(value) => {
                Self::BaseDocumentMetadata(super::feature_fields::swap_base_metadata(node, value))
            }
            Self::AutofocusProcessed(processed) => {
                let old = match &mut node
                    .document_fields_mut()
                    .expect("Document inverse")
                    .autofocus
                {
                    AutofocusGroup::KnownEmpty { processed: value } => {
                        std::mem::replace(value, processed)
                    }
                    AutofocusGroup::Known(Group::Changed(value)) => {
                        std::mem::replace(&mut value.value_mut().processed, processed)
                    }
                    _ => unreachable!("prepared autofocus metadata"),
                };
                Self::AutofocusProcessed(old)
            }
            Self::Common(v) => Self::Common(std::mem::replace(node.common_mut(), v)),
            Self::Focus(value) => Self::Focus(std::mem::replace(
                &mut node.element_fields_mut().expect("Element focus").focus,
                value,
            )),
            Self::ImageOwner(owner) => {
                let Some(Associations::Image(Group::Changed(v))) = node.associations_mut() else {
                    unreachable!("prepared Image owner")
                };
                Self::ImageOwner(std::mem::replace(&mut v.value_mut().control_owner, owner))
            }
            Self::ImageState(image) => {
                let Some(Associations::Image(Group::Changed(v))) = node.associations_mut() else {
                    unreachable!("prepared Image")
                };
                Self::ImageState(std::mem::replace(&mut v.value_mut().image, image))
            }
            Self::Element(value, focus) => {
                let target = node.element_fields_mut().expect("Element field");
                Self::Element(
                    std::mem::replace(&mut target.element, value),
                    std::mem::replace(&mut target.focus, focus),
                )
            }
            Self::Metadata(value) => Self::Metadata(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").metadata,
                value,
            )),
            Self::ShadowState(value) => {
                let DraftNode::ShadowRoot(n) = node else {
                    unreachable!("Shadow field")
                };
                Self::ShadowState(std::mem::replace(&mut n.value_mut().state, value))
            }
            Self::Text(value) => {
                let DraftNode::Text(n) = node else {
                    unreachable!("Text field")
                };
                Self::Text(std::mem::replace(&mut n.value_mut().data, value))
            }
            Self::Comment(value) => {
                let DraftNode::Comment(n) = node else {
                    unreachable!("Comment field")
                };
                Self::Comment(std::mem::replace(&mut n.value_mut().data, value))
            }
            Self::ProcessingInstruction(value) => {
                let DraftNode::ProcessingInstruction(n) = node else {
                    unreachable!("PI field")
                };
                Self::ProcessingInstruction(std::mem::replace(&mut n.value_mut().data, value))
            }
            Self::Children(value) => Self::Children(std::mem::replace(
                node.children_mut().expect("container field"),
                value,
            )),
            Self::FullscreenGroup(value) => Self::FullscreenGroup(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").fullscreen,
                value,
            )),
            Self::Slottable(value) => Self::Slottable(std::mem::replace(
                node.slottable_mut().expect("slottable field"),
                value,
            )),
            Self::Shadow(value) => Self::Shadow(std::mem::replace(
                &mut node.element_fields_mut().expect("Element").shadow,
                value,
            )),
            Self::Host(value) => Self::Host(match node {
                DraftNode::DocumentFragment(n) => std::mem::replace(&mut n.value_mut().host, value),
                DraftNode::ShadowRoot(n) => std::mem::replace(&mut n.value_mut().host, value),
                _ => unreachable!("host field"),
            }),
            Self::Associations(value) => Self::Associations(std::mem::replace(
                node.associations_mut().expect("Element"),
                value,
            )),
            Self::TemplateDocument(value) => Self::TemplateDocument(std::mem::replace(
                &mut node
                    .document_fields_mut()
                    .expect("Document")
                    .template_document,
                value,
            )),
            Self::BaseDocument(value) => Self::BaseDocument(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").base,
                value,
            )),
            Self::Encoding(value) => Self::Encoding(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").encoding,
                value,
            )),
            Self::Interactive(value) => Self::Interactive(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").interactive,
                value,
            )),
            Self::Autofocus(value) => Self::Autofocus(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").autofocus,
                value,
            )),
            Self::ActiveRange(value) => Self::ActiveRange(std::mem::replace(
                &mut node.document_fields_mut().expect("Document").active_range,
                value,
            )),
        }
    }
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        match self {
            Self::BaseDocumentMetadata(value) => {
                value.debit().usage().bytes() + value.value().journal_bytes()
            }
            Self::Associations(v) => match v {
                Associations::Template(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Slot(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Control(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Form(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Option(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Script(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Style(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Base(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Link(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Image(Group::Changed(v)) => v.debit().usage().bytes(),
                Associations::Hyperlink(Group::Changed(v)) => v.debit().usage().bytes(),
                _ => 0,
            },
            _ => 0,
        }
    }
}

pub(super) type Finish<T> =
    fn(&Draft, Key, custody::Held<Box<T>>, &AtomicBool) -> Result<FieldUndo, DomError>;
/// Measurement, candidate observation and sealing describe one selected field.
pub(super) struct FieldAccess<T> {
    pub usage: fn(&T) -> Result<custody::Usage, DomError>,
    pub overlay: for<'b> fn(&'b T) -> Overlay<'b>,
    pub finish: Finish<T>,
}
pub(in crate::store) struct SelectedFieldsMut<'a, T> {
    draft: &'a mut Draft,
    key: Key,
    candidate: Option<custody::Held<Box<T>>>,
    retain_undo: bool,
    access: FieldAccess<T>,
}
pub(in crate::store) type ElementFieldsMut<'a> = SelectedFieldsMut<'a, DomElement>;
impl<T> std::ops::Deref for SelectedFieldsMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.candidate
            .as_ref()
            .expect("uncommitted typed fields")
            .value()
    }
}
impl<T> std::ops::DerefMut for SelectedFieldsMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.candidate
            .as_mut()
            .expect("uncommitted typed fields")
            .value_mut()
    }
}
impl<T> SelectedFieldsMut<'_, T> {
    pub fn commit_charged(
        mut self,
        charge: Option<source::checkpoint::Charge>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(debit) = charge.and_then(source::checkpoint::Charge::into_debit) {
            self.candidate
                .as_mut()
                .expect("typed candidate")
                .debit_mut()
                .absorb(debit)
                .expect("same selected field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit_formed(
        mut self,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(debit) = formation.and_then(source::formation::Formation::into_debit) {
            self.candidate
                .as_mut()
                .expect("typed candidate")
                .debit_mut()
                .absorb(debit)
                .expect("same selected field ledger");
        }
        self.commit(cancel)
    }
    pub fn commit(mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let mut candidate = self.candidate.take().expect("typed candidate commits once");
        let usage = (self.access.usage)(candidate.value())?;
        super::nodes::resize_debit(candidate.debit_mut(), usage, |usage| {
            self.draft.admit_custody_usage(usage, cancel)
        })?;
        let overlay = self.access.overlay;
        let finish = self.access.finish;
        self.draft.commit_selected(
            self.key,
            candidate,
            |v| overlay(v),
            finish,
            self.retain_undo,
            cancel,
        )
    }
}
pub(super) fn inline<T>(_: &T) -> Result<custody::Usage, DomError> {
    Ok(custody::Usage::new(size_of::<T>(), 0))
}
pub(super) fn language_usage<T>(
    value: &T,
    describe: fn(&T) -> Result<custody::Usage, DomError>,
) -> Result<custody::Usage, DomError> {
    describe(value)?.checked_sub(custody::Usage::new(
        size_of::<payload::Payload<T>>() - size_of::<T>(),
        0,
    ))
}
pub(super) fn group<C, S>(incoming: custody::Held<Box<S>>) -> Group<C, S> {
    Group::Changed(incoming)
}
pub(super) fn transform<T, U>(
    incoming: custody::Held<Box<T>>,
    draft: &Draft,
    cancel: &AtomicBool,
    make: impl FnOnce(T) -> U,
) -> Result<custody::Held<Box<U>>, DomError> {
    let (value, mut debit) = incoming.into_parts();
    let wanted = debit
        .usage()
        .checked_sub(custody::Usage::new(size_of::<T>(), 0))?
        .checked_add(custody::Usage::new(size_of::<U>(), 0))?;
    let value = make(*value);
    let extra = custody::Usage::new(
        wanted.bytes().saturating_sub(debit.usage().bytes()),
        wanted.units().saturating_sub(debit.usage().units()),
    );
    debit.grow(extra, |usage| draft.admit_custody_usage(usage, cancel))?;
    let value = Box::new(value);
    super::nodes::resize_debit(&mut debit, wanted, |usage| {
        draft.admit_custody_usage(usage, cancel)
    })?;
    Ok(custody::Held::new(value, debit))
}
pub(super) fn seal_language<T>(
    incoming: custody::Held<Box<T>>,
    usage: custody::Usage,
    draft: &Draft,
    cancel: &AtomicBool,
    children: Option<bank::Dependency>,
) -> Result<Controlled<T>, DomError> {
    let reserved = payload::Reserved::new()?;
    let (value, debit) = incoming.into_parts();
    let pending = reserved.materialize(custody::Held::new(*value, debit), usage, |usage| {
        draft.admit_custody_usage(usage, cancel)
    })?;
    pending
        .seal_with_children(draft.custody_registry(), children, |usage| {
            draft.admit_custody_usage(usage, cancel)
        })
        .map(Controlled::from)
}
impl Draft {
    pub(super) fn selected_fields<T>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        expected: custody::Usage,
        build: impl FnOnce(NodeRead<'_>) -> Result<T, DomError>,
        access: FieldAccess<T>,
    ) -> Result<SelectedFieldsMut<'_, T>, DomError> {
        self.materialize_node(key, cancel)?;
        let retain_undo = self.prepare_undo(cancel)?;
        let debit = self
            .custody_ledger()
            .retain(expected, |usage| self.admit_custody_usage(usage, cancel))?;
        let value = Box::new(build(self.node(key)?)?);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<T>(),
            });
        }
        Ok(SelectedFieldsMut {
            draft: self,
            key,
            candidate: Some(custody::Held::new(value, debit)),
            retain_undo,
            access,
        })
    }
    pub(super) fn selected_fields_formed<T>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        formation: Option<source::formation::Formation>,
        expected: custody::Usage,
        build: impl FnOnce(&Draft, NodeRead<'_>) -> Result<T, DomError>,
        access: FieldAccess<T>,
    ) -> Result<SelectedFieldsMut<'_, T>, DomError> {
        struct Incoming<F> {
            build: Option<F>,
            formation: Option<source::formation::Formation>,
        }
        struct Factory<F> {
            build: Option<F>,
            debit: custody::Debit,
        }
        let mut incoming = Incoming {
            build: Some(build),
            formation,
        };
        self.materialize_node(key, cancel)?;
        let retain_undo = self.prepare_undo(cancel)?;
        let debit = incoming
            .formation
            .take()
            .and_then(source::formation::Formation::into_debit)
            .map_or_else(
                || {
                    self.custody_ledger().retain(custody::Usage::ZERO, |next| {
                        self.admit_custody_usage(next, cancel)
                    })
                },
                Ok,
            )?;
        let mut factory = Factory {
            build: incoming.build.take(),
            debit,
        };
        let current = factory.debit.usage();
        let extra = custody::Usage::new(
            expected.bytes().saturating_sub(current.bytes()),
            expected.units().saturating_sub(current.units()),
        );
        factory
            .debit
            .grow(extra, |next| self.admit_custody_usage(next, cancel))?;
        let build = factory.build.take().expect("one selected field factory");
        let value = Box::new(build(self, self.node(key)?)?);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<T>(),
            });
        }
        Ok(SelectedFieldsMut {
            draft: self,
            key,
            candidate: Some(custody::Held::new(value, factory.debit)),
            retain_undo,
            access,
        })
    }
    pub(in crate::store) fn element_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<ElementFieldsMut<'_>, DomError> {
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(size_of::<DomElement>(), 0),
            |node| {
                node.element()
                    .map(|v| v.language().clone())
                    .ok_or(DomError::WrongNodeKind)
            },
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::Element(value),
                finish: |draft, key, incoming, cancel| {
                    let children = payload::element_dependency(
                        incoming.value(),
                        draft.custody_registry(),
                        Some(&draft.names),
                        |usage| draft.admit_custody_usage(usage, cancel),
                    )?;
                    let value = seal_language(
                        incoming,
                        source::storage::typed::<DomElement>(source::storage::Payload::default())?,
                        draft,
                        cancel,
                        children,
                    )?;
                    let target = draft
                        .records_for(key)
                        .element_fields()
                        .expect("validated Element");
                    let focus = if target.element.value.value.focus_payload().is_some()
                        && value.value.value.focus_payload().is_none()
                    {
                        FocusVersion::Cleared
                    } else {
                        target.focus
                    };
                    Ok(FieldUndo::Element(value, focus))
                },
            },
        )
    }
    pub(in crate::store) fn replace_element_formed(
        &mut self,
        key: Key,
        formed: crate::element::FormedElement,
        formation: Option<source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let crate::element::FormedElement { value, children } = formed;
        let mut fields = self.element_fields_mut(key, cancel)?;
        *fields = value;
        let result = fields.commit_formed(formation, cancel);
        drop(children);
        result
    }
    pub(in crate::store) fn document_metadata_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, DomDocumentMetadata>, DomError> {
        let value = self
            .node(key)?
            .document_metadata()
            .ok_or(DomError::WrongNodeKind)?;
        let expected = language_usage(value, source::storage::document_metadata)?;
        self.selected_fields(
            key,
            cancel,
            expected,
            |node| {
                node.document_metadata()
                    .cloned()
                    .ok_or(DomError::WrongNodeKind)
            },
            FieldAccess {
                usage: |v| language_usage(v, source::storage::document_metadata),
                overlay: |value| Overlay::Metadata(value),
                finish: |draft, _key, incoming, cancel| {
                    let usage = source::storage::document_metadata(incoming.value())?;
                    let value = seal_language(incoming, usage, draft, cancel, None)?;
                    Ok(FieldUndo::Metadata(value))
                },
            },
        )
    }
    pub(in crate::store) fn shadow_state_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, crate::DomShadowRootState>, DomError> {
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(size_of::<crate::DomShadowRootState>(), 0),
            |n| n.shadow_state().cloned().ok_or(DomError::WrongNodeKind),
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::ShadowState(value),
                finish: |draft, _key, incoming, cancel| {
                    let value = seal_language(
                        incoming,
                        source::storage::typed::<crate::DomShadowRootState>(
                            source::storage::Payload::default(),
                        )?,
                        draft,
                        cancel,
                        None,
                    )?;
                    Ok(FieldUndo::ShadowState(value))
                },
            },
        )
    }
}

pub(in crate::store) struct CommonFieldsMut<'a> {
    draft: &'a mut Draft,
    key: Key,
    value: Common<Key>,
    retain_undo: bool,
}
impl std::ops::Deref for CommonFieldsMut<'_> {
    type Target = Common<Key>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl std::ops::DerefMut for CommonFieldsMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
impl CommonFieldsMut<'_> {
    pub fn commit(self, cancel: &AtomicBool) -> Result<(), DomError> {
        let debit = self
            .draft
            .custody_ledger()
            .retain(custody::Usage::ZERO, |usage| {
                self.draft.admit_custody_usage(usage, cancel)
            })?;
        self.draft.commit_selected(
            self.key,
            custody::Held::new(self.value, debit),
            |value| Overlay::Common(value),
            |_draft, _key, value, _| {
                let (value, debit) = value.into_parts();
                drop(debit);
                Ok(FieldUndo::Common(value))
            },
            self.retain_undo,
            cancel,
        )
    }
}
impl Draft {
    pub(in crate::store) fn common_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<CommonFieldsMut<'_>, DomError> {
        self.materialize_node(key, cancel)?;
        let retain_undo = self.prepare_undo(cancel)?;
        let value = self.node(key)?.common();
        Ok(CommonFieldsMut {
            draft: self,
            key,
            value,
            retain_undo,
        })
    }
}
macro_rules! scalar_guard {
    ($method:ident,$ty:ty,$overlay:path,$read:expr,$install:expr) => {
        impl Draft {
            pub(in crate::store) fn $method(
                &mut self,
                key: Key,
                cancel: &AtomicBool,
            ) -> Result<SelectedFieldsMut<'_, $ty>, DomError> {
                self.selected_fields(
                    key,
                    cancel,
                    custody::Usage::new(size_of::<$ty>(), 0),
                    $read,
                    FieldAccess {
                        usage: inline,
                        overlay: |value| ($overlay)(value),
                        finish: $install,
                    },
                )
            }
        }
    };
}
scalar_guard!(
    host_mut,
    Option<Key>,
    Overlay::Host,
    |n| match n.kind() {
        super::KindRead::DocumentFragment | super::KindRead::ShadowRoot(_) => Ok(n.host()),
        _ => Err(DomError::WrongNodeKind),
    },
    |_draft, _key, incoming, _| {
        let (value, debit) = incoming.into_parts();
        let value = *value;
        drop(debit);
        Ok(FieldUndo::Host(value))
    }
);
scalar_guard!(
    active_range_mut,
    crate::DomActiveRangeState,
    Overlay::ActiveRange,
    |n| if n.document_metadata().is_some() {
        Ok(n.active_range())
    } else {
        Err(DomError::WrongNodeKind)
    },
    |_draft, _key, incoming, _| {
        let (value, debit) = incoming.into_parts();
        let value = *value;
        drop(debit);
        Ok(FieldUndo::ActiveRange(value))
    }
);
scalar_guard!(
    template_document_mut,
    Option<crate::template::TemplateDocument<Key>>,
    Overlay::TemplateDocument,
    |n| if n.document_metadata().is_some() {
        Ok(n.template_document())
    } else {
        Err(DomError::WrongNodeKind)
    },
    |_draft, _key, incoming, _| {
        let (value, debit) = incoming.into_parts();
        let value = *value;
        drop(debit);
        Ok(FieldUndo::TemplateDocument(value))
    }
);
scalar_guard!(
    shadow_root_mut,
    Option<Key>,
    Overlay::Shadow,
    |n| if n.element().is_some() {
        Ok(n.shadow_root())
    } else {
        Err(DomError::WrongNodeKind)
    },
    |draft, _key, incoming, cancel| {
        let value = if incoming.value().is_some() {
            Some(group(transform(incoming, draft, cancel, |v| ShadowHost {
                shadow_root: v.expect("selected nonnull shadow"),
            })?))
        } else {
            drop(incoming);
            None
        };
        Ok(FieldUndo::Shadow(value))
    }
);
scalar_guard!(
    slottable_fields_mut,
    Slottable<Key>,
    Overlay::Slottable,
    |n| if matches!(
        n.kind(),
        super::KindRead::Element(_) | super::KindRead::Text(_)
    ) {
        Ok(Slottable {
            manual_slot: n.manual_slot(),
            assigned_slot: n.assigned_slot(),
        })
    } else {
        Err(DomError::WrongNodeKind)
    },
    |_draft, _key, incoming, _| { Ok(FieldUndo::Slottable(Some(group(incoming)))) }
);
scalar_guard!(
    template_mut,
    TemplateAssociation<Key>,
    Overlay::Template,
    |n| if n.element().is_some() {
        Ok(TemplateAssociation {
            template_contents: n.template_contents(),
            template_patch: n.template_patch(),
        })
    } else {
        Err(DomError::WrongNodeKind)
    },
    |_draft, _key, incoming, _| {
        Ok(FieldUndo::Associations(Associations::Template(group(
            incoming,
        ))))
    }
);
scalar_guard!(
    option_fields_mut,
    Option<Key>,
    Overlay::Option,
    |n| if n.element().is_some_and(|e| e.option_state().is_some()) {
        Ok(n.option_select())
    } else {
        Err(DomError::WrongNodeKind)
    },
    |draft, _key, incoming, cancel| {
        let value = transform(incoming, draft, cancel, |value| OptionAssociation {
            option_select: value,
        })?;
        Ok(FieldUndo::Associations(Associations::Option(group(value))))
    }
);
scalar_guard!(
    style_parser_document_mut,
    Option<Key>,
    Overlay::StyleParser,
    |n| if n.element().is_some_and(|e| e.style_state().is_some()) {
        Ok(n.style_parser_document())
    } else {
        Err(DomError::WrongNodeKind)
    },
    |draft, _key, incoming, cancel| {
        let value = transform(incoming, draft, cancel, |value| StyleAssociation {
            parser_document: value,
        })?;
        Ok(FieldUndo::Associations(Associations::Style(group(value))))
    }
);
scalar_guard!(
    control_fields_mut,
    ControlAssociation<Key>,
    Overlay::Control,
    |n| if n.element().is_some() {
        Ok(ControlAssociation {
            control_owner: n.control_owner(),
            reflected_targets: n.reflected_targets(),
        })
    } else {
        Err(DomError::WrongNodeKind)
    },
    |draft, key, incoming, cancel| {
        if matches!(
            draft.records_for(key).associations(),
            Some(Associations::Image(_))
        ) {
            if incoming.value().reflected_targets.is_some() {
                return Err(DomError::WrongNodeKind);
            }
            if matches!(
                draft.records_for(key).associations(),
                Some(Associations::Image(Group::Changed(_)))
            ) {
                let owner = incoming.value().control_owner;
                drop(incoming);
                return Ok(FieldUndo::ImageOwner(owner));
            }
            let Some(Associations::Image(Group::Canonical(old))) =
                draft.records_for(key).associations()
            else {
                unreachable!("Image control wrapper")
            };
            let image = Group::Canonical(controlled(
                &old.value.value.image,
                draft.custody_registry(),
                &mut |usage| draft.admit_custody_usage(usage, cancel),
            )?);
            let value = transform(incoming, draft, cancel, |value| ImageFields {
                image,
                control_owner: value.control_owner,
            })?;
            return Ok(FieldUndo::Associations(Associations::Image(group(value))));
        }
        Ok(FieldUndo::Associations(Associations::Control(group(
            incoming,
        ))))
    }
);
scalar_guard!(
    script_fields_mut,
    Option<crate::script::ScriptState<Key>>,
    Overlay::Script,
    |n| Ok(n.script()),
    |draft, _key, incoming, cancel| {
        let value = if incoming.value().is_some() {
            Associations::Script(group(transform(incoming, draft, cancel, |v| {
                v.expect("selected Script")
            })?))
        } else {
            drop(incoming);
            Associations::None
        };
        Ok(FieldUndo::Associations(value))
    }
);
impl Draft {
    pub(in crate::store) fn form_names_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<SelectedFieldsMut<'_, Option<super::Names>>, DomError> {
        let names = self
            .node(key)?
            .form_past_names()
            .map(|v| {
                super::Names::from_read(v, self.custody_registry(), |usage| {
                    self.admit_custody_usage(usage, cancel)
                })
            })
            .transpose()?;
        self.selected_fields(
            key,
            cancel,
            custody::Usage::new(size_of::<Option<super::Names>>(), 0),
            |_| Ok(names),
            FieldAccess {
                usage: inline,
                overlay: |value| Overlay::Names(value),
                finish: |draft, _key, incoming, cancel| {
                    let value = if incoming.value().is_some() {
                        Associations::Form(group(transform(incoming, draft, cancel, |v| {
                            FormFields {
                                past_names: v.expect("selected Form history"),
                            }
                        })?))
                    } else {
                        drop(incoming);
                        Associations::None
                    };
                    Ok(FieldUndo::Associations(value))
                },
            },
        )
    }
}

impl Draft {
    pub(in crate::store) fn with_element_fields<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut DomElement) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.element_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit(cancel)?;
        Ok(result)
    }
    pub(in crate::store) fn with_element_fields_formed<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        formation: Option<source::formation::Formation>,
        edit: impl FnOnce(&mut DomElement) -> R,
    ) -> Result<R, DomError> {
        let mut fields = self.element_fields_mut(key, cancel)?;
        let result = edit(&mut fields);
        fields.commit_formed(formation, cancel)?;
        Ok(result)
    }
    pub(in crate::store) fn clear_previously_focused_element(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let undo = self.prepare_undo(cancel)?;
        if self
            .node(key)?
            .element()
            .and_then(|v| v.common_state().html().map(|_| ()))
            .is_none()
        {
            return Err(DomError::WrongNodeKind);
        }
        let debit = self
            .custody_ledger()
            .retain(custody::Usage::ZERO, |usage| {
                self.admit_custody_usage(usage, cancel)
            })?;
        self.commit_selected(
            key,
            custody::Held::new(FocusVersion::Cleared, debit),
            |value| Overlay::Focus(value),
            |_, _, value, _| {
                let (value, debit) = value.into_parts();
                drop(debit);
                Ok(FieldUndo::Focus(value))
            },
            undo,
            cancel,
        )
    }
    pub(in crate::store) fn replace_character_data_retained(
        &mut self,
        key: Key,
        incoming: custody::Held<NewNode>,
        retained: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let undo = self.prepare_undo(cancel)?;
        match incoming.value() {
            NewNode::Text(v) | NewNode::Comment(v) => {
                let text = matches!(incoming.value(), NewNode::Text(_));
                if text != matches!(self.node(key)?.kind(), super::KindRead::Text(_)) {
                    return Err(DomError::WrongNodeKind);
                }
                let mut extra = source::storage::Payload::default();
                extra.string(v)?;
                let value = super::staged::fresh_language(
                    incoming,
                    if text { true } else { retained },
                    source::storage::typed::<CharacterData>(extra)?,
                    self.custody_registry(),
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    None,
                    |kind| match kind {
                        NewNode::Text(value) | NewNode::Comment(value) => CharacterData { value },
                        _ => unreachable!("character data input"),
                    },
                )?;
                let debit = self
                    .custody_ledger()
                    .retain(custody::Usage::ZERO, |usage| {
                        self.admit_custody_usage(usage, cancel)
                    })?;
                self.commit_selected(
                    key,
                    custody::Held::new(value, debit),
                    |value| {
                        if text {
                            Overlay::Text(&value.value.value.value)
                        } else {
                            Overlay::Comment(&value.value.value.value)
                        }
                    },
                    |_draft, _key, incoming, _| {
                        let (value, debit) = incoming.into_parts();
                        drop(debit);
                        Ok(if text {
                            FieldUndo::Text(value)
                        } else {
                            FieldUndo::Comment(value)
                        })
                    },
                    undo,
                    cancel,
                )
            }
            NewNode::ProcessingInstruction(v) => {
                let usage =
                    source::storage::processing_instruction(v, || self.source_step(cancel))?;
                let value = super::staged::fresh_language(
                    incoming,
                    retained,
                    usage,
                    self.custody_registry(),
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    None,
                    |kind| {
                        let NewNode::ProcessingInstruction(v) = kind else {
                            unreachable!("PI input")
                        };
                        v
                    },
                )?;
                let debit = self
                    .custody_ledger()
                    .retain(custody::Usage::ZERO, |usage| {
                        self.admit_custody_usage(usage, cancel)
                    })?;
                self.commit_selected(
                    key,
                    custody::Held::new(value, debit),
                    |value| Overlay::ProcessingInstruction(&value.value.value),
                    |_draft, _key, incoming, _| {
                        let (value, debit) = incoming.into_parts();
                        drop(debit);
                        Ok(FieldUndo::ProcessingInstruction(value))
                    },
                    undo,
                    cancel,
                )
            }
            _ => Err(DomError::WrongNodeKind),
        }
    }
}
