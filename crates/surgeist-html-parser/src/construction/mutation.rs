//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::clone::same_location;
use super::*;
#[cfg(test)]
use surgeist_html::values::{DocumentString, Namespace, QualifiedName};
use surgeist_html::{
    construction::HtmlNodeKind,
    values::{Attribute, OrderedAttributes},
};

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum ManagedFamily {
    Base,
    Link,
    Details,
    Dialog,
    Input,
    Style,
}
#[cfg(test)]
impl ManagedFamily {
    fn local(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Link => "link",
            Self::Details => "details",
            Self::Dialog => "dialog",
            Self::Input => "input",
            Self::Style => "style",
        }
    }
    fn boolean(self) -> &'static str {
        match self {
            Self::Base => "target",
            Self::Link => "disabled",
            Self::Details | Self::Dialog => "open",
            Self::Input => "checked",
            Self::Style => "title",
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum ManagedEffect {
    Dialog,
    Style,
    BaseHref,
    BaseTarget,
    LinkFlags(Option<surgeist_html::elements::link::HtmlLinkFlags>),
    Details(surgeist_html::elements::details::HtmlDetailsAttributeEffect),
    InputName,
    InputType,
    InputSource,
    InputColor,
    InputValue,
    InputChecked(Option<surgeist_html::elements::input::HtmlInputCheckedAttributeChange>),
}
impl SessionStorage {
    #[cfg(test)]
    fn managed_effect(
        &self,
        c: &mut ParseControl,
        family: ManagedFamily,
        node: usize,
        local: &str,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
    ) -> Result<ManagedEffect, SessionError> {
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<ManagedEffect>(c, 1)?;
        Ok(match family {
            ManagedFamily::Dialog => ManagedEffect::Dialog,
            ManagedFamily::Style => ManagedEffect::Style,
            ManagedFamily::Base if local == "href" => ManagedEffect::BaseHref,
            ManagedFamily::Base => ManagedEffect::BaseTarget,
            ManagedFamily::Link => {
                let flags = if local == "disabled" && old.is_some() && new.is_none() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let view = self
                        .construction
                        .node_at(node)
                        .ok_or(SessionError::ForeignNode)?;
                    let HtmlNodeKind::Element(element) = view.kind() else {
                        return Err(SessionError::WrongToken);
                    };
                    let mut flags = element.link_flags().ok_or(SessionError::WrongToken)?;
                    flags.disabled_attribute_changed(&Namespace::none(), old, new);
                    Some(flags)
                } else {
                    None
                };
                ManagedEffect::LinkFlags(flags)
            }
            ManagedFamily::Details => {
                ManagedEffect::Details(surgeist_html::elements::details::details_attribute_effect(
                    &Namespace::none(),
                    if local == "name" {
                        surgeist_html::elements::details::HtmlDetailsAttribute::Name
                    } else {
                        surgeist_html::elements::details::HtmlDetailsAttribute::Open
                    },
                    old,
                    new,
                ))
            }
            ManagedFamily::Input if local == "type" => ManagedEffect::InputType,
            ManagedFamily::Input if local == "src" => ManagedEffect::InputSource,
            ManagedFamily::Input if local == "name" => ManagedEffect::InputName,
            ManagedFamily::Input if matches!(local, "alpha" | "colorspace") => {
                ManagedEffect::InputColor
            }
            ManagedFamily::Input if local == "value" => ManagedEffect::InputValue,
            ManagedFamily::Input => {
                ManagedEffect::InputChecked(match (old.is_some(), new.is_some()) {
                    (false, true) => {
                        Some(surgeist_html::elements::input::HtmlInputCheckedAttributeChange::Added)
                    }
                    (true, false) => Some(
                        surgeist_html::elements::input::HtmlInputCheckedAttributeChange::Removed,
                    ),
                    _ => None,
                })
            }
        })
    }
    #[cfg(test)]
    fn preflight_managed_effect(
        &self,
        c: &mut ParseControl,
        node: usize,
        effect: ManagedEffect,
    ) -> Result<(), SessionError> {
        if matches!(
            effect,
            ManagedEffect::Details(
                surgeist_html::elements::details::HtmlDetailsAttributeEffect::Notify { .. }
            )
        ) {
            self.require_details_notification(c, node)?;
        }
        Ok(())
    }
    #[cfg(test)]
    fn managed_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        effect: ManagedEffect,
    ) -> Result<(), SessionError> {
        match effect {
            // The generic actual backing owner already ran the exact Dialog
            // hook, including equal writes and removed attributes.
            ManagedEffect::Dialog | ManagedEffect::Style => Ok(()),
            ManagedEffect::BaseHref => self.base_change(c, node, None, true),
            ManagedEffect::BaseTarget => Ok(()),
            ManagedEffect::LinkFlags(flags) => {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if let Some(flags) = flags {
                    payload_write::<surgeist_html::elements::link::HtmlLinkFlags>(c, 1)?;
                    let view = self
                        .construction
                        .node_at(node)
                        .ok_or(SessionError::ForeignNode)?;
                    c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    let element = view.handle();
                    self.construction.apply(
                        c,
                        surgeist_html::bounded::construction::HtmlConstructionCommand::SetLinkFlags { element, flags },
                        &[],
                    )?;
                }
                Ok(())
            }
            ManagedEffect::Details(effect) => self.details_change(c, node, effect),
            ManagedEffect::InputName => self.normalize_radio(c, node, None),
            ManagedEffect::InputType => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                let (association, _) = find_form_association_state(&self.payloads, c, node)?
                    .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                let container = self.element_attribute_container(c, node)?;
                let HtmlNodeKind::Element(element) = &self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind()
                else {
                    unreachable!()
                };
                let mut position = None;
                for (i, a) in element.attributes().iter().enumerate() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                    if a.name().namespace().uri().is_none()
                        && exact_ascii(c, Some(a.name().local_name()), "type")?
                    {
                        position = Some(i);
                        break;
                    }
                }
                match position {
                    Some(position) => self.initial_input_attribute(
                        c,
                        node,
                        slot,
                        association,
                        (container, position),
                        InputAttributeInitialization::Installed,
                    ),
                    None => self.change_input_type(
                        c,
                        node,
                        slot,
                        container,
                        state,
                        surgeist_html::attributes::HtmlInputType::Text,
                    ),
                }
            }
            ManagedEffect::InputColor | ManagedEffect::InputValue => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Color
                    && (!matches!(effect, ManagedEffect::InputValue) || !state.flags.dirty_value())
                {
                    self.update_input_color(c, node, slot)?;
                }
                Ok(())
            }
            ManagedEffect::InputSource => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Image {
                    self.image_source_changed(c, node, slot)?;
                }
                Ok(())
            }
            ManagedEffect::InputChecked(None) => Ok(()),
            ManagedEffect::InputChecked(Some(change)) => {
                let (slot, mut state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                let Some(flags) = state.flags.checked_attribute_changed(change) else {
                    return Ok(());
                };
                state.flags = flags;
                self.write_input_state(c, slot, state)?;
                self.normalize_radio(c, node, None)
            }
        }
    }

    // Actual one-attribute stage shared by lexical and generated writeback.
    // Preserve the original admission, transfer and native commit order.
    pub(super) fn append_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [name_slot, value_slot] = fields;
        let (values, allocation) = c.new_vector()?;
        let admission = match self.prepare_payload_slot(c) {
            Ok(admission) => admission,
            Err(error) => {
                release_vector(c, values, allocation)?;
                drop(incoming);
                return Err(error);
            }
        };
        let stage = self
            .payloads
            .push_row(
                admission,
                RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: 1,
                    reversed_pairs: 0,
                })),
            )
            .map_err(|_| latch_registry_row(c))?;
        let prepared = (|| {
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                allocation,
                ..
            }))) = self.payloads.values.get_mut(stage)
            else {
                return Err(latch_registry_row(c));
            };
            c.reserve_vector(values, allocation, 1)?;
            payload_write::<Attribute>(c, 1)?;
            payload_write::<[StringLocation; 2]>(c, 2)?;
            c.acquire(HtmlParseResource::ManagedAttributes, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = prepared {
            drop(incoming);
            return Err(error);
        }
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. }))) =
            self.payloads.values.get_mut(stage)
        else {
            return Err(latch_registry_row(c));
        };
        values.push(incoming);
        self.bind_string(name_slot, StringLocation::AttributeName(stage, 0))
            .map_err(|_| latch_registry_row(c))?;
        self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0))
            .map_err(|_| latch_registry_row(c))?;
        self.edit_attributes(c, node, container, stage, 1, false)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let start = element_record(&self.construction, c, node)?
            .attributes()
            .len();
        self.begin_attribute_transfer(c, stage, container, start)?;
        self.finish_attribute_merge(
            c,
            PreparedAttributeMerge {
                target: node,
                container,
                stage,
                start,
                count: 1,
                earlier: [Some(name_slot), Some(value_slot), None, None],
            },
        )?;
        Ok(())
    }
    /// The registry rows of the name and value of the attribute at `position`
    /// of `node`, found from the layout creation registered instead of by a
    /// scan of the registry, or `None` when the layout cannot be relied on.
    /// The attribute must have no namespace and no prefix, which are strings of
    /// their own, and each candidate row is used only when its location names
    /// this attribute, the location a scan matches, so a hint that an edit made
    /// stale falls back to the scan and finds the same rows.
    fn hinted_attribute_rows(
        &self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        position: usize,
    ) -> Result<Option<[usize; 2]>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let attributes = element_record(&self.construction, c, node)?
            .attributes()
            .as_slice();
        let Some(attribute) = attributes.get(position) else {
            return Ok(None);
        };
        if attribute.name().namespace().uri().is_some() || attribute.name().prefix().is_some() {
            return Ok(None);
        }
        let Some(value) = self
            .payloads
            .attribute_value_candidate(node, attributes.len(), position)
        else {
            return Ok(None);
        };
        let Some(name) = value.checked_sub(1) else {
            return Ok(None);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if self.row_located(name, StringLocation::AttributeName(container, position))
            && self.row_located(value, StringLocation::AttributeValue(container, position))
        {
            Ok(Some([name, value]))
        } else {
            Ok(None)
        }
    }
    /// The rows a scan of the registry finds for the attribute at `position`
    /// of `container`, indexed as name, value, namespace and prefix.
    #[cfg(test)]
    fn scanned_attribute_rows(&self, container: usize, position: usize) -> [Option<usize>; 4] {
        let mut rows = [None; 4];
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            if let RetainedPayload::String { location, .. } = payload {
                let which = match *location {
                    StringLocation::AttributeName(o, p) if o == container && p == position => 0,
                    StringLocation::AttributeValue(o, p) if o == container && p == position => 1,
                    StringLocation::AttributeNamespace(o, p) if o == container && p == position => {
                        2
                    }
                    StringLocation::AttributePrefix(o, p) if o == container && p == position => 3,
                    _ => continue,
                };
                rows[which] = Some(slot);
            }
        }
        rows
    }
    pub(super) fn replace_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        location: [usize; 2],
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [container, position] = location;
        let [name_slot, value_slot] = fields;
        payload_write::<Option<Attribute>>(c, 1)?;
        let mut incoming = Some(incoming);
        let replace = (|| {
            // All fixed registry progress is admitted before the generic ready
            // boundary; old source history retires while that owner is exclusive.
            payload_write::<[StringLocation; 6]>(c, 6)?;
            let mut old_slots = [None; 4];
            if let Some([name, value]) = self.hinted_attribute_rows(c, node, container, position)? {
                for (which, slot) in [(0, name), (1, value)] {
                    payload_write::<Option<usize>>(c, 1)?;
                    if let Some(old) = old_slots.get_mut(which) {
                        *old = Some(slot);
                    }
                }
                #[cfg(test)]
                assert_eq!(
                    old_slots,
                    self.scanned_attribute_rows(container, position),
                    "a verified hint names the rows a scan finds"
                );
            } else {
                for (slot, payload) in self.payloads.values.iter().enumerate() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let RetainedPayload::String { location, .. } = payload {
                        let which = match *location {
                            StringLocation::AttributeName(o, p)
                                if o == container && p == position =>
                            {
                                Some(0)
                            }
                            StringLocation::AttributeValue(o, p)
                                if o == container && p == position =>
                            {
                                Some(1)
                            }
                            StringLocation::AttributeNamespace(o, p)
                                if o == container && p == position =>
                            {
                                Some(2)
                            }
                            StringLocation::AttributePrefix(o, p)
                                if o == container && p == position =>
                            {
                                Some(3)
                            }
                            _ => None,
                        };
                        if let Some(which) = which {
                            payload_write::<Option<usize>>(c, 1)?;
                            let Some(old) = old_slots.get_mut(which) else {
                                return Err(latch_registry_row(c));
                            };
                            *old = Some(slot);
                        }
                    }
                }
            }
            payload_write::<Option<AttributeStorage>>(c, 2)?;
            payload_write::<OrderedAttributes>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
            let old = self.replace_attribute_storage(
                c,
                node,
                container,
                position,
                &mut incoming,
                &old_slots,
            )?;
            for slot in old_slots.into_iter().flatten() {
                self.bind_string(slot, StringLocation::Detached)
                    .map_err(|_| latch_registry_row(c))?;
            }
            self.bind_string(
                name_slot,
                StringLocation::AttributeName(container, position),
            )
            .map_err(|_| latch_registry_row(c))?;
            self.bind_string(
                value_slot,
                StringLocation::AttributeValue(container, position),
            )
            .map_err(|_| latch_registry_row(c))?;

            drop(old);
            Ok::<_, SessionError>(())
        })();
        drop(incoming);
        replace?;
        self.dialog_attribute_at(c, node, position, true)?;
        self.association_attribute_hook(c, node, position)?;
        Ok(())
    }
    /// Where the name rows of the attributes from `index` on sit, as the slot of
    /// the last attribute's name, when creation's layout still holds for all of
    /// them: each name and value row is verified by its location, and none of
    /// the attributes has a namespace or prefix row. `None` sends the removal to
    /// the registry scan, which finds the same rows.
    fn removal_layout(
        &self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        index: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let attributes = element_record(&self.construction, c, node)?
            .attributes()
            .as_slice();
        let count = attributes.len();
        let Some(last) = count.checked_sub(1) else {
            return Ok(None);
        };
        let Some(last_name) = self
            .payloads
            .attribute_value_candidate(node, count, last)
            .and_then(|value| value.checked_sub(1))
        else {
            return Ok(None);
        };
        for position in index..count {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let Some(attribute) = attributes.get(position) else {
                return Ok(None);
            };
            if attribute.name().namespace().uri().is_some() || attribute.name().prefix().is_some() {
                return Ok(None);
            }
            let Some([name, value]) = attribute_row_slots(last_name, count, position) else {
                return Ok(None);
            };
            if !(self.row_located(name, StringLocation::AttributeName(container, position))
                && self.row_located(value, StringLocation::AttributeValue(container, position)))
            {
                return Ok(None);
            }
        }
        #[cfg(test)]
        {
            let mut expected: Vec<usize> = Vec::new();
            for (slot, payload) in self.payloads.values.iter().enumerate() {
                if let RetainedPayload::String { location, .. } = payload
                    && let StringLocation::AttributeName(owner, position)
                    | StringLocation::AttributeValue(owner, position)
                    | StringLocation::AttributeNamespace(owner, position)
                    | StringLocation::AttributePrefix(owner, position) = *location
                    && owner == container
                    && position >= index
                {
                    expected.push(slot);
                }
            }
            let mut hinted: Vec<usize> = (index..count)
                .flat_map(|position| attribute_row_slots(last_name, count, position).unwrap())
                .collect();
            hinted.sort_unstable();
            assert_eq!(
                hinted, expected,
                "a verified layout names the rows a scan finds"
            );
        }
        Ok(Some(last_name))
    }
    fn row_located(&self, slot: usize, expected: StringLocation) -> bool {
        matches!(
            self.payloads.values.get(slot),
            Some(RetainedPayload::String { location, .. }) if same_location(*location, expected)
        )
    }
    /// Rebinds the string row at `slot`, if it belongs to the attributes of
    /// `container`, for the removal of the attribute at `index`, and records
    /// that the registry up to it has been walked.
    fn rebind_removed_attribute_row(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        container: usize,
        index: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<StringLocation>(c, 1)?;
        payload_write::<usize>(c, 1)?;
        if let Some(RetainedPayload::String { location, .. }) = self.payloads.values.get_mut(slot) {
            // The constructor rebuilds the same location kind at its new
            // position, so no second match can disagree with the first.
            let (owner, position, rebuild): (usize, usize, fn(usize, usize) -> StringLocation) =
                match *location {
                    StringLocation::AttributeName(o, p) => (o, p, StringLocation::AttributeName),
                    StringLocation::AttributeValue(o, p) => (o, p, StringLocation::AttributeValue),
                    StringLocation::AttributeNamespace(o, p) => {
                        (o, p, StringLocation::AttributeNamespace)
                    }
                    StringLocation::AttributePrefix(o, p) => {
                        (o, p, StringLocation::AttributePrefix)
                    }
                    _ => (usize::MAX, 0, StringLocation::AttributeName),
                };
            if owner == container {
                *location = if position == index {
                    StringLocation::Detached
                } else {
                    rebuild(
                        owner,
                        position.saturating_sub(usize::from(position > index)),
                    )
                };
            }
        }
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Removing { rebound, .. }))) =
            self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        *rebound = slot.saturating_add(1);
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        self.payloads
            .check_new_aliases(ConstructionView::Active(self.construction.storage(c)?), c)?;
        let container = self.element_attribute_container(c, node)?;
        self.payloads.check_row_aliases(
            ConstructionView::Active(self.construction.storage(c)?),
            c,
            container,
        )?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let mut selected = None;
        for (index, attr) in element.attributes().as_slice().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                let style_effect = surgeist_html::vocabulary::KnownHtmlAttribute::from_name(
                    attr.name(),
                )
                .map(|attribute| {
                    surgeist_html::elements::style::style_attribute_effect(
                        attr.name().namespace(),
                        attribute,
                    )
                });
                selected = Some((index, style_effect));
                break;
            }
        }
        let Some((index, style_effect)) = selected else {
            return Ok(());
        };
        let layout = self.removal_layout(c, node, container, index)?;
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
        };
        let len = values.len();
        *storage = Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            index,
            shifted: 0,
            popped: false,
            rebound: 0,
        });
        // Historical destinations stay at the original slot. Retire their
        // original groups before any location is rebound, preserving the cause.
        if let Some(last_name) = layout {
            for slot in
                attribute_row_slots(last_name, len, index).ok_or_else(|| latch_registry_row(c))?
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                self.retire_field_group(c, slot)?;
            }
        } else {
            for slot in 0..self.payloads.values.len() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if matches!(self.payloads.row(c, slot)?, RetainedPayload::String { location: StringLocation::AttributeName(owner, position) | StringLocation::AttributeValue(owner, position) | StringLocation::AttributeNamespace(owner, position) | StringLocation::AttributePrefix(owner, position), .. } if *owner == container && *position == index)
                {
                    self.retire_field_group(c, slot)?;
                }
            }
        }
        let Some(last) = len.checked_sub(1) else {
            return Err(latch_registry_row(c));
        };
        for position in index..last {
            self.edit_removing_attribute(c, node, container, Some(position))?;
        }
        self.edit_removing_attribute(c, node, container, None)?;
        if let Some(last_name) = layout {
            // The rows that can change are those of the attributes from `index`
            // on, and creation registered the last attribute first, so their
            // registry order is by descending position.
            for position in (index..len).rev() {
                for slot in attribute_row_slots(last_name, len, position)
                    .ok_or_else(|| latch_registry_row(c))?
                {
                    self.rebind_removed_attribute_row(c, slot, container, index)?;
                }
            }
        } else {
            for slot in 0..self.payloads.values.len() {
                self.rebind_removed_attribute_row(c, slot, container, index)?;
            }
        }
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        payload_write::<usize>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            ..
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
        };
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        if local == "open" {
            self.dialog_open_changed(c, node, true, false)?;
        }
        if style_effect
            == Some(surgeist_html::elements::style::HtmlStyleAttributeEffect::OwnerAttributes)
            && self.html_local(c, node, "style")?
        {
            self.record_association(
                c,
                node,
                pending_association::HtmlParseAssociationOccurrenceKind::StyleOwnerAttributes,
                None,
                None,
                false,
            )?;
        }
        if base::is_base(
            self.construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?,
        ) && local == "href"
        {
            self.base_change(c, node, None, true)?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl ParserArenaSession {
    pub(super) fn set_managed_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        input: &mut Option<crate::parser::tokenizer::token::Attribute>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        family: ManagedFamily,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        if !storage.html_local(c, node, family.local())? {
            return Err(SessionError::WrongToken);
        }
        storage.payloads.check_new_aliases(
            ConstructionView::Active(storage.construction.test_storage()),
            c,
        )?;
        let attr = input.as_ref().ok_or(SessionError::Consumed)?;
        c.require_source_range(&attr.source)?;
        check_formatting_text(c, &attr.name, &attr.source)?;
        check_formatting_text(c, &attr.value, &attr.source)?;
        let local = if matches!(family, ManagedFamily::Base)
            && exact_ascii(c, Some(attr.name.value.text()), "href")?
        {
            "href"
        } else if matches!(family, ManagedFamily::Link)
            && exact_ascii(c, Some(attr.name.value.text()), "rel")?
        {
            "rel"
        } else if exact_ascii(c, Some(attr.name.value.text()), "name")? {
            "name"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "type")?
        {
            "type"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "src")?
        {
            "src"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "alpha")?
        {
            "alpha"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "colorspace")?
        {
            "colorspace"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "value")?
        {
            "value"
        } else if exact_ascii(c, Some(attr.name.value.text()), family.boolean())? {
            family.boolean()
        } else {
            return Err(SessionError::WrongToken);
        };
        if matches!(family, ManagedFamily::Input) {
            let (_, state) = find_input_state(&storage.payloads, c, node)?;
            if local == "type" {
                let next = input_type_state(c, attr.value.value.text())?;
                let file_image = matches!(
                    state.flags.input_type(),
                    surgeist_html::attributes::HtmlInputType::File
                        | surgeist_html::attributes::HtmlInputType::Image
                ) || matches!(
                    next,
                    surgeist_html::attributes::HtmlInputType::File
                        | surgeist_html::attributes::HtmlInputType::Image
                );
                let radio = next == surgeist_html::attributes::HtmlInputType::Radio
                    && matches!(
                        state.flags.input_type(),
                        surgeist_html::attributes::HtmlInputType::Radio
                            | surgeist_html::attributes::HtmlInputType::Checkbox
                    );
                let color = state.flags.input_type()
                    == surgeist_html::attributes::HtmlInputType::Color
                    || next == surgeist_html::attributes::HtmlInputType::Color;
                if !(radio || file_image || color) {
                    return Err(SessionError::WrongToken);
                }
            } else if matches!(local, "value" | "alpha" | "colorspace")
                && state.flags.input_type() != surgeist_html::attributes::HtmlInputType::Color
            {
                return Err(SessionError::WrongToken);
            }
        }
        let container = storage.element_attribute_container(c, node)?;
        storage.payloads.check_row_aliases(
            ConstructionView::Active(storage.construction.test_storage()),
            c,
            container,
        )?;
        let mut position = None;
        let HtmlNodeKind::Element(element) = &storage
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            unreachable!()
        };
        for (index, old) in element.attributes().as_slice().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if old.name().namespace().uri().is_none()
                && exact_ascii(c, Some(old.name().local_name()), local)?
            {
                position = Some(index);
                break;
            }
        }
        let effect = storage.managed_effect(
            c,
            family,
            node,
            local,
            position.map(|p| element.attributes().as_slice()[p].value()),
            Some(attr.value.value.text()),
        )?;
        storage.preflight_managed_effect(c, node, effect)?;
        let previous = storage.begin_attribute_change(c, node, Some(&attr.source))?;
        payload_write::<Option<crate::parser::tokenizer::token::Attribute>>(c, 1)?;
        let attr = input.take().expect("qualified original owner");
        let result = (|| {
            let (name, name_slot) = match storage.register_text(c, attr.name) {
                Ok(value) => value,
                Err(error) => {
                    attr.value.release(c)?;
                    return Err(error);
                }
            };
            let (value, value_slot) = storage.register_text(c, attr.value)?;
            payload_write::<Attribute>(c, 1)?;
            let incoming = Attribute::new(QualifiedName::new(Namespace::none(), None, name), value);
            if let Some(position) = position {
                storage.replace_registered_attribute(
                    c,
                    node,
                    [container, position],
                    incoming,
                    [name_slot, value_slot],
                )?;
            } else {
                storage.append_registered_attribute(
                    c,
                    node,
                    container,
                    incoming,
                    [name_slot, value_slot],
                )?;
            }
            storage.managed_change(c, node, effect)?;
            storage.drain_construction_effects(c)?;
            Ok(())
        })();
        if c.stop().is_none() {
            storage.end_node_copy(c, previous.as_ref())?;
        }
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(error) => Err(super::latch_effect_fault(c, error)),
        }
    }
    pub(super) fn remove_managed_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        local: &str,
        family: ManagedFamily,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        if !storage.html_local(c, node, family.local())? {
            return Err(SessionError::WrongToken);
        }
        if matches!(family, ManagedFamily::Input) {
            find_input_state(&storage.payloads, c, node)?;
        }
        let Some(old) = storage.form_attribute(c, node, local)? else {
            return Ok(());
        };
        let effect = storage.managed_effect(c, family, node, local, Some(old), None)?;
        storage.preflight_managed_effect(c, node, effect)?;
        let previous = storage.begin_attribute_change(c, node, None)?;
        let result = (|| {
            storage.remove_managed_field(c, node, local)?;
            // Removing name still invokes the name-change algorithm; its
            // actual now-absent name provides the source early return.
            storage.managed_change(c, node, effect)?;
            storage.drain_construction_effects(c)?;
            Ok(())
        })();
        if c.stop().is_none() {
            storage.end_node_copy(c, previous.as_ref())?;
        }
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(error) => Err(super::latch_effect_fault(c, error)),
        }
    }
}

/// The registry slots of the name and value of the attribute at `position` of
/// an element of `count` attributes, as creation registered them: the last
/// attribute's name at `last_name`, its value after it, then the previous
/// attribute's name and value, and so on.
fn attribute_row_slots(last_name: usize, count: usize, position: usize) -> Option<[usize; 2]> {
    let name = last_name.checked_add(
        count
            .checked_sub(1)?
            .checked_sub(position)?
            .checked_mul(2)?,
    )?;
    Some([name, name.checked_add(1)?])
}
