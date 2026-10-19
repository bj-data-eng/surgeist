//! Publication maps only changed typed groups. Unchanged children keep their
//! original materialization and custody until the new root has adopted them.
use super::super::*;
use super::staged::*;
use super::{Sequence, SequenceView};
use canonical::*;

pub(in crate::store) struct ProjectedNode {
    pub value: custody::Held<Node>,
    pub dependency: Option<bank::Dependency>,
}

pub(super) fn sequence_usage<C: Copy, S: Copy>(
    view: SequenceView<'_, C, S>,
) -> Result<custody::Usage, DomError> {
    let bytes = match view {
        SequenceView::Empty => 0,
        SequenceView::Canonical(values, _) | SequenceView::AliasedCanonical(values, _, _, _) => {
            values.retained_bytes()?
        }
        SequenceView::Staged(values) | SequenceView::AliasedStaged(values, _, _) => {
            if values.is_empty() && size_of::<C>() != 0 {
                0
            } else {
                values
                    .len()
                    .checked_mul(size_of::<C>())
                    .and_then(|bytes| bytes.checked_add(size_of::<Vec<C>>()))
                    .ok_or(DomError::RepresentationExhausted)?
            }
        }
    };
    Ok(custody::Usage::new(bytes, 0))
}

pub(super) fn projected_values<C: Copy, S: Copy>(
    view: SequenceView<'_, C, S>,
    map: impl Fn(S) -> C,
    debit: &mut custody::Debit,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<SharedVec<C>, DomError> {
    match view {
        SequenceView::Empty => return Ok(SharedVec::from(Vec::new())),
        SequenceView::Canonical(values, _) | SequenceView::AliasedCanonical(values, _, _, _) => {
            return Ok(values.clone());
        }
        _ => {}
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(view.len())
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if scale_metrics::ENABLED && values.capacity() * size_of::<C>() != 0 {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: values.capacity() * size_of::<C>(),
        });
    }
    let extra = values
        .capacity()
        .checked_sub(view.len())
        .and_then(|count| count.checked_mul(size_of::<C>()))
        .ok_or(DomError::RepresentationExhausted)?;
    debit.grow(custody::Usage::new(extra, 0), |next| {
        draft.admit_custody_usage(next, cancel)
    })?;
    #[cfg(test)]
    let mut copied = super::sequence::CopyObservation::relation();
    for value in view.iter() {
        draft.source_step(cancel)?;
        values.push(map(value));
        #[cfg(test)]
        copied.pushed::<C>();
    }
    let values = SharedVec::from(values);
    #[cfg(test)]
    if scale_metrics::ENABLED && values.has_backing() {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: size_of::<Vec<C>>(),
        });
    }
    Ok(values)
}

fn mapped_usage<C, S>(usage: custody::Usage) -> Result<custody::Usage, DomError> {
    usage
        .checked_sub(custody::Usage::new(size_of::<payload::Payload<S>>(), 0))?
        .checked_add(custody::Usage::new(size_of::<payload::Payload<C>>(), 0))
}

impl Draft {
    /// Admission precedes the factory; owned data precedes its debit on every
    /// refusal. Child controls stay in the caller until the fresh target seals.
    fn seal_projection<T>(
        &self,
        usage: custody::Usage,
        cancel: &AtomicBool,
        build: impl FnOnce(&mut custody::Debit) -> Result<T, DomError>,
        dependency: impl FnOnce(
            &T,
            &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        ) -> Result<Option<bank::Dependency>, DomError>,
    ) -> Result<Controlled<T>, DomError> {
        let reserved = payload::Reserved::new()?;
        let mut debit = self
            .custody_ledger()
            .retain(usage, |next| self.admit_custody_usage(next, cancel))?;
        let value = build(&mut debit)?;
        let usage = debit.usage();
        let incoming = custody::Held::new(value, debit);
        let children = dependency(incoming.value(), &mut |next| {
            self.admit_custody_usage(next, cancel)
        })?;
        Ok(reserved
            .materialize(incoming, usage, |next| {
                self.admit_custody_usage(next, cancel)
            })?
            .seal_with_children(self.custody_registry(), children, |next| {
                self.admit_custody_usage(next, cancel)
            })?
            .into())
    }

    fn project_leaf<C, S>(
        &self,
        group: &Group<C, S>,
        cancel: &AtomicBool,
        usage: impl FnOnce(&S) -> Result<custody::Usage, DomError>,
        map: impl FnOnce(&S) -> C,
    ) -> Result<Controlled<C>, DomError> {
        match group {
            Group::Canonical(value) => Ok(value.clone()),
            Group::Changed(value) => self.seal_projection(
                usage(value.value())?,
                cancel,
                |_| Ok(map(value.value())),
                |_, _| Ok(None),
            ),
        }
    }

    fn project_children(
        &self,
        group: Option<&Group<ContainerChildren<NodeId>, Children>>,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<Option<Controlled<ContainerChildren<NodeId>>>, DomError> {
        let Some(group) = group else { return Ok(None) };
        match group {
            Group::Canonical(value) => Ok(Some(value.clone())),
            Group::Changed(value) => {
                let view = value.value().values.view();
                let backing = sequence_usage(view)?;
                if backing == custody::Usage::ZERO {
                    return Ok(None);
                }
                let usage = custody::Usage::new(
                    size_of::<payload::Payload<ContainerChildren<NodeId>>>(),
                    0,
                )
                .checked_add(backing)?;
                self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(ContainerChildren {
                            values: projected_values(view, map, debit, self, cancel)?,
                        })
                    },
                    |_, _| Ok(None),
                )
                .map(Some)
            }
        }
    }

    fn project_slottable(
        &self,
        group: Option<&Group<Slottable<NodeId>, Slottable<Key>>>,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<Option<Controlled<Slottable<NodeId>>>, DomError> {
        let Some(group) = group else { return Ok(None) };
        if let Group::Changed(value) = group
            && value.value().manual_slot.is_none()
            && value.value().assigned_slot.is_none()
        {
            return Ok(None);
        }
        self.project_leaf(
            group,
            cancel,
            |_| source::storage::typed::<Slottable<NodeId>>(source::storage::Payload::default()),
            |value| Slottable {
                manual_slot: value.manual_slot.map(map),
                assigned_slot: value.assigned_slot.map(map),
            },
        )
        .map(Some)
    }

    fn project_association(
        &self,
        association: &Associations,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<(ElementAssociations<NodeId>, Option<bank::ObjectCustody>), DomError> {
        macro_rules! leaf {
            ($variant:ident, $group:expr, $usage:expr, $convert:expr) => {{
                let value = self.project_leaf($group, cancel, $usage, $convert)?;
                (
                    ElementAssociations::$variant(value.value),
                    Some(value.custody),
                )
            }};
        }
        Ok(match association {
            Associations::None => (ElementAssociations::None, None),
            Associations::FormEmpty => (
                ElementAssociations::FormEmpty(crate::form::FormPastNames::fresh()),
                None,
            ),
            Associations::Template(group) => leaf!(
                Template,
                group,
                |_| source::storage::typed::<TemplateAssociation<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| TemplateAssociation {
                    template_contents: v.template_contents.map(map),
                    template_patch: v.template_patch.map(|v| v.map(map))
                }
            ),
            Associations::Control(group) => leaf!(
                Control,
                group,
                |_| source::storage::typed::<ControlAssociation<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| ControlAssociation {
                    control_owner: v.control_owner.map(map),
                    reflected_targets: v.reflected_targets.map(|v| v.map(map))
                }
            ),
            Associations::Option(group) => leaf!(
                Option,
                group,
                |_| source::storage::typed::<OptionAssociation<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| OptionAssociation {
                    option_select: v.option_select.map(map)
                }
            ),
            Associations::Style(group) => leaf!(
                Style,
                group,
                |_| source::storage::typed::<StyleAssociation<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| StyleAssociation {
                    parser_document: v.parser_document.map(map)
                }
            ),
            Associations::Script(group) => leaf!(
                Script,
                group,
                |_| source::storage::typed::<crate::script::ScriptState<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| (*v).map(map)
            ),
            Associations::Base(group) => leaf!(
                Base,
                group,
                |v| mapped_usage::<crate::base::BaseState<NodeId>, crate::base::BaseState<Key>>(
                    source::storage::base(v)?
                ),
                |v| v.map(map)
            ),
            Associations::Link(group) => leaf!(
                Link,
                group,
                |_| source::storage::typed::<crate::base::LinkState<NodeId>>(
                    source::storage::Payload::default()
                ),
                |v| v.map(map)
            ),
            Associations::Hyperlink(group) => leaf!(
                Hyperlink,
                group,
                |v| mapped_usage::<
                    crate::hyperlink::HyperlinkState<NodeId>,
                    crate::hyperlink::HyperlinkState<Key>,
                >(source::storage::hyperlink(v)?),
                |v| v.map(map)
            ),
            Associations::Slot(Group::Canonical(value)) => (
                ElementAssociations::Slot(value.value.clone()),
                Some(value.custody.clone()),
            ),
            Associations::Slot(Group::Changed(value)) => {
                let value = value.value();
                let usage =
                    custody::Usage::new(size_of::<payload::Payload<SlotAssignment<NodeId>>>(), 0)
                        .checked_add(sequence_usage(value.manual.view())?)?
                        .checked_add(sequence_usage(value.assigned.view())?)?;
                let projected = self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(SlotAssignment {
                            manual_nodes: projected_values(
                                value.manual.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                            assigned_nodes: projected_values(
                                value.assigned.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                        })
                    },
                    |_, _| Ok(None),
                )?;
                (
                    ElementAssociations::Slot(projected.value),
                    Some(projected.custody),
                )
            }
            Associations::Form(Group::Canonical(value)) => (
                ElementAssociations::Form(value.value.clone()),
                Some(value.custody.clone()),
            ),
            Associations::Form(Group::Changed(value)) => {
                let names = value.value().past_names.project(self, map, cancel)?;
                if names.value.payload().is_none() {
                    return Ok((ElementAssociations::FormEmpty(names.value), None));
                }
                let usage = source::storage::typed::<FormAssociation<NodeId>>(
                    source::storage::Payload::default(),
                )?;
                let projected = self.seal_projection(
                    usage,
                    cancel,
                    |_| {
                        Ok(FormAssociation {
                            past_names: names.value.clone(),
                        })
                    },
                    |value, admit| payload::form_dependency(value, self.custody_registry(), admit),
                )?;
                (
                    ElementAssociations::Form(projected.value),
                    Some(projected.custody),
                )
            }
            Associations::Image(Group::Canonical(value)) => (
                ElementAssociations::Image(value.value.clone()),
                Some(value.custody.clone()),
            ),
            Associations::Image(Group::Changed(value)) => {
                let value = value.value();
                let image = self.project_leaf(
                    &value.image,
                    cancel,
                    |v| {
                        mapped_usage::<
                            crate::image::ImageState<NodeId>,
                            crate::image::ImageState<Key>,
                        >(source::storage::image(v)?)
                    },
                    |v| v.map(map),
                )?;
                let usage = source::storage::typed::<ImageAssociation<NodeId>>(
                    source::storage::Payload::default(),
                )?;
                let projected = self.seal_projection(
                    usage,
                    cancel,
                    |_| {
                        Ok(ImageAssociation {
                            image: image.value.clone(),
                            control_owner: value.control_owner.map(map),
                        })
                    },
                    |value, admit| {
                        payload::image_association_dependency(value, self.custody_registry(), admit)
                    },
                )?;
                (
                    ElementAssociations::Image(projected.value),
                    Some(projected.custody),
                )
            }
        })
    }

    fn project_interactive(
        &self,
        value: &InteractiveGroup,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<(Interactive<NodeId>, Option<bank::ObjectCustody>), DomError> {
        Ok(match value {
            InteractiveGroup::Absent => (Interactive::Absent, None),
            InteractiveGroup::Unavailable => (Interactive::Unavailable, None),
            InteractiveGroup::KnownEmpty => (Interactive::KnownEmpty, None),
            InteractiveGroup::Known(Group::Canonical(value)) => (
                Interactive::Known(value.value.clone()),
                Some(value.custody.clone()),
            ),
            InteractiveGroup::Known(Group::Changed(value)) => {
                let value = value.value();
                if [value.open.view(), value.top.view(), value.pending.view()]
                    .into_iter()
                    .try_fold(custody::Usage::ZERO, |usage, view| {
                        usage.checked_add(sequence_usage(view)?)
                    })?
                    == custody::Usage::ZERO
                {
                    return Ok((Interactive::KnownEmpty, None));
                }
                let usage = custody::Usage::new(
                    size_of::<payload::Payload<InteractiveValues<NodeId>>>(),
                    0,
                )
                .checked_add(sequence_usage(value.open.view())?)?
                .checked_add(sequence_usage(value.top.view())?)?
                .checked_add(sequence_usage(value.pending.view())?)?;
                let value = self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(InteractiveValues {
                            open_dialogs: projected_values(
                                value.open.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                            top_layer: projected_values(
                                value.top.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                            pending_top_layer_removals: projected_values(
                                value.pending.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                        })
                    },
                    |_, _| Ok(None),
                )?;
                (Interactive::Known(value.value), Some(value.custody))
            }
        })
    }

    fn project_autofocus(
        &self,
        value: &AutofocusGroup,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<(Autofocus<NodeId>, Option<bank::ObjectCustody>), DomError> {
        Ok(match value {
            AutofocusGroup::Absent => (Autofocus::Absent, None),
            AutofocusGroup::Unavailable => (Autofocus::Unavailable, None),
            AutofocusGroup::KnownEmpty { processed } => (
                Autofocus::KnownEmpty {
                    processed: *processed,
                },
                None,
            ),
            AutofocusGroup::Known(Group::Canonical(value)) => (
                Autofocus::Known(value.value.clone()),
                Some(value.custody.clone()),
            ),
            AutofocusGroup::Known(Group::Changed(value)) => {
                let value = value.value();
                if sequence_usage(value.candidates.view())? == custody::Usage::ZERO {
                    return Ok((
                        Autofocus::KnownEmpty {
                            processed: value.processed,
                        },
                        None,
                    ));
                }
                let usage =
                    custody::Usage::new(size_of::<payload::Payload<AutofocusValues<NodeId>>>(), 0)
                        .checked_add(sequence_usage(value.candidates.view())?)?;
                let value = self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(AutofocusValues {
                            candidates: projected_values(
                                value.candidates.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                            processed: value.processed,
                        })
                    },
                    |_, _| Ok(None),
                )?;
                (Autofocus::Known(value.value), Some(value.custody))
            }
        })
    }

    fn project_base_document(
        &self,
        value: &BaseDocumentGroup,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<(BaseDocumentState<NodeId>, Option<bank::ObjectCustody>), DomError> {
        Ok(match value {
            BaseDocumentGroup::Absent => (BaseDocumentState::Absent, None),
            BaseDocumentGroup::KnownEmpty => (BaseDocumentState::KnownEmpty, None),
            BaseDocumentGroup::Known(Group::Canonical(value)) => (
                BaseDocumentState::Known(value.value.clone()),
                Some(value.custody.clone()),
            ),
            BaseDocumentGroup::Known(Group::Changed(value)) => {
                let value = value.value();
                if value.facts.is_none()
                    && value.first.is_none()
                    && sequence_usage(value.render.view())? == custody::Usage::ZERO
                {
                    return Ok((BaseDocumentState::KnownEmpty, None));
                }
                let mut usage = custody::Usage::new(
                    size_of::<payload::Payload<canonical::BaseDocument<NodeId>>>(),
                    0,
                )
                .checked_add(sequence_usage(value.render.view())?)?;
                if let Some(facts) = &value.facts {
                    let facts = facts.retained_storage_usage();
                    usage = usage.checked_add(custody::Usage::new(
                        facts.retained_bytes(),
                        facts.utf16_extent(),
                    ))?;
                }
                let value = self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(canonical::BaseDocument {
                            facts: value.facts.clone(),
                            first: value.first.map(map),
                            render_blocking: projected_values(
                                value.render.view(),
                                map,
                                debit,
                                self,
                                cancel,
                            )?,
                        })
                    },
                    |_, _| Ok(None),
                )?;
                (BaseDocumentState::Known(value.value), Some(value.custody))
            }
        })
    }

    fn project_fullscreen(
        &self,
        value: Option<
            &Group<FullscreenQueue, Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent>>,
        >,
        cancel: &AtomicBool,
    ) -> Result<Option<Controlled<FullscreenQueue>>, DomError> {
        let Some(value) = value else { return Ok(None) };
        match value {
            Group::Canonical(value) => Ok(Some(value.clone())),
            Group::Changed(value) => {
                let view = value.value().view();
                let backing = sequence_usage(view)?;
                if backing == custody::Usage::ZERO {
                    return Ok(None);
                }
                let usage = custody::Usage::new(size_of::<payload::Payload<FullscreenQueue>>(), 0)
                    .checked_add(backing)?;
                self.seal_projection(
                    usage,
                    cancel,
                    |debit| {
                        Ok(FullscreenQueue {
                            values: projected_values(view, |v| v, debit, self, cancel)?,
                        })
                    },
                    |_, _| Ok(None),
                )
                .map(Some)
            }
        }
    }

    pub(in crate::store) fn project_node(
        &self,
        key: Key,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<ProjectedNode, DomError> {
        let source = self.node(key)?;
        if source
            .base()
            .and_then(|v| v.frozen)
            .is_some_and(|v| v.response == crate::base::Response::Pending)
        {
            return Err(DomError::BaseFacts(
                crate::base::DomBaseFactsError::Missing(
                    surgeist_html_semantics::HtmlBaseMissing::ResponseDomain,
                ),
            ));
        }
        let map = |key| map(self.live_key(key));
        let common = |value: &Common<Key>| Common {
            document: map(value.document),
            relevant_global: value.relevant_global,
            parent: value.parent.map(map),
        };
        macro_rules! root {
            ($variant:ident, $ty:ty, $value:expr, $dependency:expr) => {{
                // A root belongs to exactly one Slot materialization. Keep its
                // admitted body affine until that Slot seals; independently
                // shared language and feature children keep their own custody.
                let reserved = payload::Reserved::<$ty>::new()?;
                let usage = source::storage::typed::<$ty>(source::storage::Payload::default())?;
                let debit = self
                    .custody_ledger()
                    .retain(usage, |next| self.admit_custody_usage(next, cancel))?;
                let incoming = custody::Held::new($value, debit);
                let dependency = ($dependency)(incoming.value(), &mut |next| {
                    self.admit_custody_usage(next, cancel)
                })?;
                let formed = reserved
                    .materialize(incoming, usage, |next| {
                        self.admit_custody_usage(next, cancel)
                    })?
                    .into_unregistered();
                let (value, debit) = formed.into_parts();
                ProjectedNode {
                    value: custody::Held::new(Node::$variant(value), debit),
                    dependency,
                }
            }};
        }
        Ok(match self.records_for(key) {
            DraftNode::Document(value) => {
                let value = value.value();
                let children = self.project_children(value.children.as_ref(), &map, cancel)?;
                let fullscreen = self.project_fullscreen(value.fullscreen.as_ref(), cancel)?;
                let base = self.project_base_document(&value.base, &map, cancel)?;
                let interactive = self.project_interactive(&value.interactive, &map, cancel)?;
                let autofocus = self.project_autofocus(&value.autofocus, &map, cancel)?;
                let encoding = match &value.encoding {
                    EncodingGroup::Absent => EncodingUrl::Absent,
                    EncodingGroup::KnownEmpty => EncodingUrl::KnownEmpty,
                    EncodingGroup::Known(value) => EncodingUrl::Known(value.value.clone()),
                };
                root!(
                    Document,
                    DocumentNode<NodeId>,
                    DocumentNode {
                        common: common(&value.common),
                        metadata: value.metadata.value.clone(),
                        children: children.as_ref().map(|v| v.value.clone()),
                        active_range: value.active_range,
                        fullscreen_events: fullscreen.as_ref().map(|v| v.value.clone()),
                        template_document: value.template_document.map(|v| v.map(map)),
                        base_document: base.0.clone(),
                        encoding_url_document: encoding,
                        interactive_document: interactive.0.clone(),
                        autofocus_document: autofocus.0.clone(),
                    },
                    |value, admit| payload::document_dependency(
                        value,
                        self.custody_registry(),
                        admit
                    )
                )
            }
            DraftNode::Element(value) => {
                let value = value.value();
                let children = self.project_children(value.children.as_ref(), &map, cancel)?;
                let slottable = self.project_slottable(value.slottable.as_ref(), &map, cancel)?;
                let shadow = value
                    .shadow
                    .as_ref()
                    .map(|v| {
                        self.project_leaf(
                            v,
                            cancel,
                            |_| {
                                source::storage::typed::<ShadowHost<NodeId>>(
                                    source::storage::Payload::default(),
                                )
                            },
                            |v| ShadowHost {
                                shadow_root: map(v.shadow_root),
                            },
                        )
                    })
                    .transpose()?;
                let associations = self.project_association(&value.associations, &map, cancel)?;
                let element = match value.focus {
                    FocusVersion::Canonical => value.element.clone(),
                    FocusVersion::Staged(key) => {
                        let focus = self.seal_projection(
                            source::storage::typed::<crate::element::FocusReference<NodeId>>(
                                source::storage::Payload::default(),
                            )?,
                            cancel,
                            |_| Ok(crate::element::FocusReference { value: map(key) }),
                            |_, _| Ok(None),
                        )?;
                        self.seal_projection(
                            source::storage::typed::<DomElement>(
                                source::storage::Payload::default(),
                            )?,
                            cancel,
                            |_| {
                                let mut element = value.element.value.value.clone();
                                element.set_focus_payload(Some(focus.value.clone()));
                                Ok(element)
                            },
                            |value, admit| {
                                payload::element_dependency(
                                    value,
                                    self.custody_registry(),
                                    Some(&self.names),
                                    admit,
                                )
                            },
                        )?
                    }
                    FocusVersion::Cleared => self.seal_projection(
                        source::storage::typed::<DomElement>(source::storage::Payload::default())?,
                        cancel,
                        |_| {
                            let mut element = value.element.value.value.clone();
                            element.set_focus_payload(None);
                            Ok(element)
                        },
                        |value, admit| {
                            payload::element_dependency(
                                value,
                                self.custody_registry(),
                                Some(&self.names),
                                admit,
                            )
                        },
                    )?,
                };
                root!(
                    Element,
                    ElementNode<NodeId>,
                    ElementNode {
                        common: common(&value.common),
                        element: element.value.clone(),
                        children: children.as_ref().map(|v| v.value.clone()),
                        slottable: slottable.as_ref().map(|v| v.value.clone()),
                        shadow: shadow.as_ref().map(|v| v.value.clone()),
                        associations: associations.0.clone(),
                    },
                    |value, admit| payload::element_node_dependency(
                        value,
                        self.custody_registry(),
                        Some(&self.names),
                        admit
                    )
                )
            }
            DraftNode::DocumentFragment(value) => {
                let value = value.value();
                let children = self.project_children(value.children.as_ref(), &map, cancel)?;
                root!(
                    DocumentFragment,
                    FragmentNode<NodeId>,
                    FragmentNode {
                        common: common(&value.common),
                        children: children.as_ref().map(|v| v.value.clone()),
                        host: value.host.map(map)
                    },
                    |value, admit| payload::fragment_dependency(
                        value,
                        self.custody_registry(),
                        admit
                    )
                )
            }
            DraftNode::ShadowRoot(value) => {
                let value = value.value();
                let children = self.project_children(value.children.as_ref(), &map, cancel)?;
                root!(
                    ShadowRoot,
                    ShadowNode<NodeId>,
                    ShadowNode {
                        common: common(&value.common),
                        state: value.state.value.clone(),
                        children: children.as_ref().map(|v| v.value.clone()),
                        host: value.host.map(map)
                    },
                    |value, admit| payload::shadow_dependency(
                        value,
                        self.custody_registry(),
                        admit
                    )
                )
            }
            DraftNode::Text(value) => {
                let value = value.value();
                let slottable = self.project_slottable(value.slottable.as_ref(), &map, cancel)?;
                root!(
                    Text,
                    TextNode<NodeId>,
                    TextNode {
                        common: common(&value.common),
                        data: value.data.value.clone(),
                        slottable: slottable.as_ref().map(|v| v.value.clone())
                    },
                    |value, admit| payload::text_dependency(value, self.custody_registry(), admit)
                )
            }
            DraftNode::Comment(value) => {
                let value = value.value();
                root!(
                    Comment,
                    CommentNode<NodeId>,
                    CommentNode {
                        common: common(&value.common),
                        data: value.data.value.clone()
                    },
                    |value: &CommentNode<NodeId>, admit| payload::data_dependency(
                        &value.data,
                        self.custody_registry(),
                        admit
                    )
                )
            }
            DraftNode::ProcessingInstruction(value) => {
                let value = value.value();
                root!(
                    ProcessingInstruction,
                    ProcessingInstructionNode<NodeId>,
                    ProcessingInstructionNode {
                        common: common(&value.common),
                        data: value.data.value.clone()
                    },
                    |value: &ProcessingInstructionNode<NodeId>, admit| payload::data_dependency(
                        &value.data,
                        self.custody_registry(),
                        admit
                    )
                )
            }
            DraftNode::DocumentType(value) => {
                let value = value.value();
                root!(
                    DocumentType,
                    DocumentTypeNode<NodeId>,
                    DocumentTypeNode {
                        common: common(&value.common),
                        data: value.data.value.clone()
                    },
                    |value: &DocumentTypeNode<NodeId>, admit| payload::data_dependency(
                        &value.data,
                        self.custody_registry(),
                        admit
                    )
                )
            }
        })
    }
}
