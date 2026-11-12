//! Attribute merging and installation: the parse stages the attributes it builds and HTML's
//! storage appends them to elements one at a time.

use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind,
    values::{DocumentString, Namespace, QualifiedName},
};

#[derive(Clone, Copy)]
pub(super) enum AttributeMergeName {
    Html,
    Body,
}

impl AttributeMergeName {
    fn local(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Body => "body",
        }
    }
}

/// A candidate of a merge that the element keeps: its place among the candidates and its name
/// as the plan saw it.
pub(super) struct KeptCandidate {
    pub(super) index: usize,
    /// The name, which shares the block of the token's name. The merge drops it before the
    /// token's strings are released, because a string whose block has another holder cannot be
    /// released.
    pub(super) name: Option<DocumentString>,
}

/// A merge the plan decided: the token attributes in source order, which the merge appends or
/// ignores one at a time, and the candidates it keeps.
/// A constant string of an attribute's namespace or prefix, with the group of the origins of this
/// occurrence of it when the parse records them.
pub(super) struct LiteralField {
    pub(super) string: DocumentString,
    pub(super) group: Option<OriginGroupRef>,
    /// Whether the units of this occurrence are the ones its string's first registration counted:
    /// the first use of a literal in a parse creates the string and counts its units, as the
    /// occurrence that made it, so the attribute it goes to does not count them again.
    pub(super) counted: bool,
}

/// The namespace and the prefix of the name of an attribute that has them.
#[derive(Default)]
pub(super) struct QualifiedFields {
    pub(super) namespace: Option<LiteralField>,
    pub(super) prefix: Option<LiteralField>,
}

pub(super) struct PreparedAttributeMerge {
    pub(super) target: usize,
    /// The attributes of the token, last first, so that they pop in source order.
    pub(super) attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
    /// The kept candidates in source order, with the names the plan decided on.
    pub(super) kept: OwnedVec<KeptCandidate>,
    /// The length of the element's attribute list before the merge.
    pub(super) start: usize,
    pub(super) count: usize,
}

impl PreparedAttributeMerge {
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        let mut refused = None;
        // The names the plan kept hold the token strings' blocks, which their release requires
        // to be unheld: the plan goes first.
        keep_first(&mut refused, self.kept.release(control));
        keep_first(
            &mut refused,
            release_token_attributes(self.attributes, control),
        );
        refused.map_or(Ok(()), Err)
    }
}

impl SessionStorage {
    /// Same parser-owned HTML root, real slots and original lexical origins.
    /// Imported/native attribute backing requires a later admitted importer.
    pub(super) fn add_attributes_if_missing(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<(), SessionError> {
        if let Some(prepared) = self.prepare_attribute_merge(control, target, input)? {
            self.finish_attribute_merge(control, prepared)?;
        }
        Ok(())
    }
    /// Real borrowed-input admission and owned staging shared by the complete
    /// operation and its bounded commit. Rejection before take preserves Tag.
    pub(super) fn prepare_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        self.prepare_named_attribute_merge(control, target, input, AttributeMergeName::Html)
    }
    pub(super) fn prepare_named_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        target: &ParserNode,
        input: &mut Option<Tag>,
        name: AttributeMergeName,
    ) -> Result<Option<PreparedAttributeMerge>, SessionError> {
        let target = self.index(control, target)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(target)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        if !exact_ascii(
            control,
            element.name().namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? || !exact_ascii(control, Some(element.name().local_name()), name.local())?
        {
            return Err(SessionError::WrongToken);
        }
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        Self::check_text(control, &mut tag.name)?;
        if !exact_ascii(control, Some(tag.name.value.text()), name.local())? {
            return Err(SessionError::WrongToken);
        }
        check_text_source(control, &tag.name)?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(a) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            Self::check_text(control, &mut a.name)?;
            Self::check_text(control, &mut a.value)?;
            lowercase_token_name(control, a.name.value.text())?;
            check_text_source(control, &a.name)?;
            check_text_source(control, &a.value)?;
            control.require_source_range(&a.source)?;
        }
        let start = element.attributes().count(control)?;
        let entries = start.saturating_add(tag.attributes.len());
        if entries >= NAME_INDEX_MIN_ENTRIES {
            // The table also serves the effects of the attributes the merge installs, and stays
            // for the next merge into this element.
            self.holds
                .provide_name_index(control, entries.saturating_add(NAME_INDEX_SPARE_ENTRIES))?;
        }
        let mut kept = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let planned =
            self.plan_attribute_merge(control, target, tag.attributes.as_slice(), &mut kept);
        let count = match planned {
            Ok(count) => count,
            Err(error) => {
                kept.release(control)?;
                return Err(error);
            }
        };
        // Final borrowed-input admission leaves the complete Tag caller-owned.
        if let Err(error) = payload_write::<Option<Tag>>(control, 1) {
            kept.release(control)?;
            return Err(error);
        }
        let Some(Tag {
            name, attributes, ..
        }) = input.take()
        else {
            // Nothing was taken: only the plan begun above needs ending.
            kept.release(control)?;
            return Err(SessionError::Consumed);
        };
        let prepared = PreparedAttributeMerge {
            target,
            attributes,
            kept,
            start,
            count,
        };
        let mut prepared = prepared;
        let named = (|| {
            for candidate in prepared.kept.as_mut_slice_prepaid(control)? {
                candidate.name = None;
            }
            name.release(control)?;
            self.reverse_attributes(control, &mut prepared.attributes)
        })();
        if let Err(error) = named {
            prepared.release(control)?;
            return Err(error);
        }
        if count == 0 {
            self.discard_merge_attributes(control, prepared)?;
            return Ok(None);
        }
        if let Err(error) = payload_write::<PreparedAttributeMerge>(control, 1) {
            prepared.release(control)?;
            return Err(error);
        }
        Ok(Some(prepared))
    }

    /// Reverses the attributes of a token, as the pairs swapped one at a time, each swap charged
    /// before it is made, so that they pop in source order.
    pub(super) fn reverse_attributes(
        &mut self,
        control: &mut ParseControl,
        attributes: &mut OwnedVec<crate::parser::tokenizer::token::Attribute>,
    ) -> Result<(), SessionError> {
        let len = attributes.len();
        for left in 0..len / 2 {
            let right = len
                .checked_sub(1)
                .and_then(|last| last.checked_sub(left))
                .ok_or_else(|| latch_registry_row(control))?;
            if !attributes.swap_slots(left, right, control)? {
                return Err(latch_registry_row(control));
            }
        }
        Ok(())
    }

    /// Ignores every attribute of a merge that keeps none, noting each as already present, and
    /// releases the merge.
    fn discard_merge_attributes(
        &mut self,
        control: &mut ParseControl,
        mut prepared: PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        let ignored = (|| {
            while let Some(attribute) = prepared.attributes.try_pop(control)? {
                self.ignore_merge_attribute(control, attribute)?;
            }
            Ok::<_, SessionError>(())
        })();
        let released = prepared.release(control);
        ignored?;
        released
    }

    /// An attribute the merge does not keep: its lexical origin is noted and both its strings are
    /// released.
    fn ignore_merge_attribute(
        &mut self,
        control: &mut ParseControl,
        attribute: crate::parser::tokenizer::token::Attribute,
    ) -> Result<(), SessionError> {
        let retained = (|| {
            if let Some(evidence) = self.evidence.as_mut() {
                evidence.lexical_attribute_ignored(control, attribute.lexical.as_ref())?;
            }
            Ok::<_, SessionError>(())
        })();
        // Both strings are released whatever the first release reports.
        let name = attribute.name.release(control);
        let value = attribute.value.release(control);
        retained?;
        name?;
        value?;
        Ok(())
    }

    pub(super) fn finish_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        mut prepared: PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        let installed = control
            .require_identity(&self.control)
            .map_err(SessionError::from)
            .and_then(|()| self.install_merged_attributes(control, &mut prepared));
        let target = prepared.target;
        let (start, count) = (prepared.start, prepared.count);
        let released = prepared.release(control);
        installed?;
        released?;
        self.attributes_installed(control, target, start, count)
    }

    /// Appends the kept attributes in source order, ignoring the others. The index of each
    /// decision is the one the plan made, so the storage finds the plan's entries for them in its
    /// table of names.
    fn install_merged_attributes(
        &mut self,
        control: &mut ParseControl,
        prepared: &mut PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        // The run reserves room for every kept candidate once, so that no append of the merge
        // grows it and one wide merge leaves it with no spare room.
        self.reserve_attributes(control, prepared.target, prepared.count)?;
        let mut rank = 0usize;
        let mut source = 0usize;
        loop {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let keep = prepared
                .kept
                .as_slice()
                .get(rank)
                .is_some_and(|candidate| candidate.index == source);
            let Some(attribute) = prepared.attributes.try_pop(control)? else {
                break;
            };
            source = source
                .checked_add(1)
                .ok_or_else(|| latch_registry_row(control))?;
            if !keep {
                self.ignore_merge_attribute(control, attribute)?;
                continue;
            }
            rank = rank
                .checked_add(1)
                .ok_or_else(|| latch_registry_row(control))?;
            self.append_token_attribute(control, prepared.target, attribute, true, true)?;
            if self.effects.form.is_some() {
                self.drain_form_effects(control)?;
            }
        }
        Ok(())
    }

    /// The effects of attributes that were appended to an element: the hooks of the families
    /// that read them, and the base element's change.
    pub(super) fn attributes_installed(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        start: usize,
        count: usize,
    ) -> Result<(), SessionError> {
        let end = start
            .checked_add(count)
            .ok_or_else(|| latch_registry_row(control))?;
        for position in start..end {
            self.initial_nonce_attribute(control, target, position)?;
            self.initial_meta_attribute(control, target, position)?;
            self.association_attribute_hook(control, target, position)?;
            self.initial_dialog_attribute(control, target, position)?;
        }
        if base::is_base(
            self.construction
                .node_at(target)
                .ok_or_else(|| latch_registry_row(control))?,
        ) {
            // Only an actual appended no-namespace href changes the pinned
            // first-Base href trigger. Other fields share this merge owner.
            let mut href_changed = false;
            for position in start..end {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let attribute = element_attribute(&self.construction, control, target, position)?;
                if attribute.name().namespace().uri().is_none()
                    && exact_ascii(control, Some(attribute.name().local_name()), "href")?
                {
                    href_changed = true;
                    break;
                }
            }
            self.base_change(control, target, None, href_changed, None)?;
        }
        Ok(())
    }

    /// Appends the attribute of a token to the element at `node`; see [`Self::append_fields`].
    pub(super) fn append_token_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        attribute: crate::parser::tokenizer::token::Attribute,
        planned: bool,
        effects: bool,
    ) -> Result<usize, SessionError> {
        let crate::parser::tokenizer::token::Attribute {
            name,
            value,
            lexical: _,
            source: _,
        } = attribute;
        let name = match self.text_field(control, name) {
            Ok(name) => name,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        let value = match self.text_field(control, value) {
            Ok(value) => value,
            Err(error) => {
                name.value.release(control)?;
                return Err(error);
            }
        };
        self.append_fields(control, node, name, value, planned, effects)
    }

    /// Appends an attribute to the element at `node`: the name is interned, the value is copied
    /// into the storage, both strings are released, and the groups of their origins become the
    /// groups of the attribute's fields. `planned` says that the attribute is the next one a
    /// merge plan kept. With `effects`, an `id` among form controls leaves the effects its change
    /// needs, which the caller drains. A refusal releases the strings. Returns the position the
    /// attribute took.
    pub(super) fn append_fields(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        name: FieldString,
        value: FieldString,
        planned: bool,
        effects: bool,
    ) -> Result<usize, SessionError> {
        self.append_qualified_fields(
            control,
            node,
            QualifiedFields::default(),
            name,
            value,
            planned,
            effects,
        )
    }

    /// As [`Self::append_fields`] for an attribute whose name has a namespace or a prefix: their
    /// strings are shared with the interned name, and the groups of their origins become the
    /// groups of the attribute's namespace and prefix fields.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn append_qualified_fields(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        qualified: QualifiedFields,
        name: FieldString,
        value: FieldString,
        planned: bool,
        effects: bool,
    ) -> Result<usize, SessionError> {
        let QualifiedFields { namespace, prefix } = qualified;
        let mut counted = 0usize;
        let (namespace, namespace_group) = match namespace {
            Some(LiteralField {
                string,
                group,
                counted: first,
            }) => {
                if first {
                    counted = counted
                        .checked_add(string.len())
                        .ok_or_else(|| latch_registry_row(control))?;
                }
                (Namespace::from_uri(string), group)
            }
            None => (Namespace::none(), None),
        };
        let (prefix, prefix_group) = match prefix {
            Some(LiteralField {
                string,
                group,
                counted: first,
            }) => {
                if first {
                    counted = counted
                        .checked_add(string.len())
                        .ok_or_else(|| latch_registry_row(control))?;
                }
                (Some(string), group)
            }
            None => (None, None),
        };
        let appended = (|| {
            let id =
                self.intern_qualified_name(control, &namespace, prefix.as_ref(), name.units())?;
            let local = self
                .construction
                .name(&id)
                .map_err(|_| latch_registry_row(control))?
                .clone();
            let form_effects = if effects {
                self.prepare_id_attribute_effects(control, node, &local, value.units().is_empty())?
            } else {
                None
            };
            if form_effects.is_some() {
                payload_write::<Option<FormEffects>>(control, 1)?;
            }
            let candidate = planned.then(|| local.local_name().clone());
            let position = self.append_named_attribute(
                control,
                node,
                &id,
                value.units(),
                candidate.as_ref(),
                counted,
            )?;
            if form_effects.is_some() {
                self.effects.form = form_effects;
            }
            Ok::<usize, SessionError>(position)
        })();
        let released_name = name.value.release(control);
        let released_value = value.value.release(control);
        let position = appended?;
        released_name?;
        released_value?;
        self.bind_field(
            control,
            FieldRef::AttributeName(node, position),
            name.group,
            0,
        )?;
        self.bind_field(
            control,
            FieldRef::AttributeValue(node, position),
            value.group,
            0,
        )?;
        self.bind_field(
            control,
            FieldRef::AttributeNamespace(node, position),
            namespace_group,
            0,
        )?;
        self.bind_field(
            control,
            FieldRef::AttributePrefix(node, position),
            prefix_group,
            0,
        )?;
        Ok(position)
    }

    /// The effects of an `id` attribute of the name `name`, about to be appended to the element at
    /// `target` with a value that is empty or not, among the listed form controls that have a
    /// `form` attribute.
    pub(super) fn prepare_id_attribute_effects(
        &self,
        control: &mut ParseControl,
        target: usize,
        name: &QualifiedName,
        empty_value: bool,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if name.namespace().uri().is_some() || !exact_ascii(control, Some(name.local_name()), "id")?
        {
            return Ok(None);
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if empty_value {
            return Ok(None);
        }
        #[cfg(test)]
        self.assert_explicit_forms_exact();
        for &index in self.payloads.explicit_form_list() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            }) = self.payloads.values.get(index)
                && state.kind.listed()
                && self.form_attribute(control, *node, "form")?.is_some()
            {
                return Ok(Some(FormEffects {
                    root: target,
                    old_parent: None,
                    new_parent: None,
                    stage: FormEffectStage::IdChange,
                    insert: false,
                    cause: InsertionCause::NonParser,
                    next: Some(target),
                    pass: FormEffectPass::Local,
                    href_base: base::HrefBaseAhead::Unscanned,
                }));
            }
        }
        Ok(None)
    }
}
