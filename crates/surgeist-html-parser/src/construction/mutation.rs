//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::string_storage::AttributeString;
use super::*;
use surgeist_html::{construction::ConstructionNodeKind, values::Namespace};

#[cfg(test)]
#[path = "mutation_managed.rs"]
mod managed;
#[cfg(test)]
pub(in crate::construction) use managed::ManagedFamily;

/// The strings of an attribute.
const ALL_STRINGS: [AttributeString; 4] = [
    AttributeString::Name,
    AttributeString::Value,
    AttributeString::Namespace,
    AttributeString::Prefix,
];

impl SessionStorage {
    /// Actual one-attribute append shared by lexical and generated writeback. The strings are
    /// copied into the storage and released.
    pub(super) fn append_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        name: FieldString,
        value: FieldString,
    ) -> Result<(), SessionError> {
        if let Err(stop) = c.charge(HtmlParseWorkCategory::RecordVisit, 1) {
            name.release(c)?;
            value.release(c)?;
            return Err(stop.into());
        }
        let position = self.append_fields(c, node, name, value, false, true)?;
        if self.effects.form.is_some() {
            self.drain_form_effects(c)?;
        }
        self.attributes_installed(c, node, position, 1)
    }
    pub(super) fn replace_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
        name: FieldString,
        value: FieldString,
    ) -> Result<(), SessionError> {
        let replace = (|| {
            // All fixed registry progress is admitted before the edit; old source history
            // retires before the edit replaces the attribute.
            payload_write::<[FieldRef; 4]>(c, 4)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
            self.retire_attribute_strings(c, node, position, &ALL_STRINGS)?;
            let id = self.intern_name(c, &Namespace::none(), name.units())?;
            self.replace_named_attribute(c, node, position, &id, value.units())
        })();
        let released_name = name.value.release(c);
        let released_value = value.value.release(c);
        replace?;
        released_name?;
        released_value?;
        self.bind_field(c, FieldRef::AttributeName(node, position), name.group, 0)?;
        self.bind_field(c, FieldRef::AttributeValue(node, position), value.group, 0)?;
        self.dialog_attribute_at(c, node, position, true)?;
        self.association_attribute_hook(c, node, position)?;
        Ok(())
    }
    /// Replaces the value of the attribute at `position` of `node`, which keeps its name and the
    /// group of the name's origins. The group of the old value retires as in
    /// [`Self::replace_registered_attribute`], and the group of `value` becomes the value's.
    pub(super) fn replace_registered_attribute_value(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
        value: FieldString,
    ) -> Result<(), SessionError> {
        let replace = (|| {
            payload_write::<[FieldRef; 2]>(c, 2)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let name = element_attribute(&self.construction, c, node, position)?
                .name()
                .clone();
            self.retire_attribute_strings(c, node, position, &[AttributeString::Value])?;
            let id = self.intern_qualified_name(
                c,
                name.namespace(),
                name.prefix(),
                name.local_name().code_units(),
            )?;
            self.replace_named_attribute(c, node, position, &id, value.units())
        })();
        let released = value.value.release(c);
        replace?;
        released?;
        self.bind_field(c, FieldRef::AttributeValue(node, position), value.group, 0)?;
        self.dialog_attribute_at(c, node, position, true)?;
        self.association_attribute_hook(c, node, position)?;
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let mut selected = None;
        for (index, attr) in element.attributes().each(c)?.enumerate() {
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
        // Historical destinations stay at the original slot. Retire their original groups before
        // the attribute goes, preserving the cause.
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        self.retire_attribute_strings(c, node, index, &ALL_STRINGS)?;
        self.remove_attribute(c, node, index)?;
        if let Some(evidence) = self.evidence.as_mut() {
            self.payloads
                .detail_origins
                .close_gap(c, evidence, node, index)?;
        }
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
            self.base_change(c, node, None, true, None)?;
        }
        Ok(())
    }
}
