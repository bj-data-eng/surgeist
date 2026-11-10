//! Test-only managed-attribute scaffolding over the production mutation owner.
//! Production attribute edits live in the parent; nothing here is compiled
//! outside tests.
use super::*;
use surgeist_html::values::{DocumentString, Namespace, QualifiedName};

#[derive(Clone, Copy)]
pub(in crate::construction) enum ManagedFamily {
    Base,
    Link,
    Details,
    Dialog,
    Input,
    Style,
}
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
            ManagedEffect::BaseHref => self.base_change(c, node, None, true, None),
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

    /// The rows a scan of the registry finds for the attribute at `position`
    /// of `container`, indexed as name, value, namespace and prefix.
    pub(super) fn scanned_attribute_rows(
        &self,
        container: usize,
        position: usize,
    ) -> [Option<usize>; 4] {
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
    /// Checks that a verified layout names exactly the rows a registry scan finds.
    pub(super) fn assert_verified_layout(
        &self,
        container: usize,
        index: usize,
        count: usize,
        last_name: usize,
    ) {
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
}

impl ParserArenaSession {
    pub(in crate::construction) fn set_managed_attribute(
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
    pub(in crate::construction) fn remove_managed_attribute(
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
