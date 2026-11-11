//! Attribute merging and installation: the parse stages the attributes it builds and HTML's
//! storage appends them to elements one at a time.

use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind,
    values::{Attribute, DocumentString, Namespace, QualifiedName},
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

/// Attributes the parse has built and means to install in an element. They are installed from
/// the last to the first, so a list staged in reverse source order is installed in source order.
/// The staging list is the parse's own vector, accounted through its ticket, and is released
/// with [`Self::release`].
pub(super) struct StagedAttributes {
    values: Vec<Attribute>,
    allocation: VectorCapacity<Attribute>,
}

impl StagedAttributes {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let (values, allocation) = control.new_vector()?;
        Ok(Self { values, allocation })
    }

    /// Stages `attribute`. The caller has registered its strings and acquired its logical slot.
    /// `final_len` is the number of attributes the staging list is known to reach.
    pub(super) fn push(
        &mut self,
        control: &mut ParseControl,
        attribute: Attribute,
        final_len: usize,
    ) -> Result<(), SessionError> {
        control.reserve_vector_geometric_toward(
            &mut self.values,
            &mut self.allocation,
            1,
            final_len,
        )?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            (const { mem::size_of::<Attribute>() + 2 * mem::size_of::<FieldRef>() }) as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
        self.values.push(attribute);
        Ok(())
    }

    /// How many attributes are staged.
    #[cfg(test)]
    pub(super) fn staged(&self) -> usize {
        self.values.len()
    }

    /// Stages `attribute` as [`Self::push`] does, charging only the copy of the attribute: its
    /// strings are not bound to fields until its element exists.
    pub(super) fn push_plain(
        &mut self,
        control: &mut ParseControl,
        attribute: Attribute,
        final_len: usize,
    ) -> Result<(), SessionError> {
        control.reserve_vector_geometric_toward(
            &mut self.values,
            &mut self.allocation,
            1,
            final_len,
        )?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<Attribute>() as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
        self.values.push(attribute);
        Ok(())
    }

    /// Reverses the staged order, as the pairs swapped one at a time: each swap is charged
    /// before it is made.
    pub(super) fn reverse(&mut self, control: &mut ParseControl) -> Result<(), SessionError> {
        let len = self.values.len();
        for left in 0..len / 2 {
            let right = len
                .checked_sub(1)
                .and_then(|last| last.checked_sub(left))
                .ok_or_else(|| latch_registry_row(control))?;
            control.admit_atomic_layout(const { mem::size_of::<Attribute>() * 2 })?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                (const { mem::size_of::<Attribute>() * 2 }) as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 3)?;
            self.values.swap(left, right);
        }
        Ok(())
    }

    /// Hands the staged list to the owner of its record as a vector with its ticket.
    pub(super) fn into_parts(self) -> (Vec<Attribute>, VectorCapacity<Attribute>) {
        (self.values, self.allocation)
    }

    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        release_vector(control, self.values, self.allocation)
    }
}

/// A candidate of a merge that the element keeps: its place among the candidates and its name
/// as the plan saw it.
pub(super) struct KeptCandidate {
    pub(super) index: usize,
    /// The name, which shares the block of the token's name. It is lent back to the token while
    /// the token's strings are registered, because a string whose block has another holder
    /// cannot be released if its registration is refused.
    pub(super) name: Option<DocumentString>,
}

pub(super) struct PreparedAttributeMerge {
    pub(super) target: usize,
    pub(super) stage: StagedAttributes,
    /// The kept candidates in source order, with the names the plan decided on.
    pub(super) kept: OwnedVec<KeptCandidate>,
    /// The registry slot of the first row that staging registered.
    first_row: usize,
    /// The length of the element's attribute list before the merge.
    pub(super) start: usize,
    pub(super) count: usize,
}

impl PreparedAttributeMerge {
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        let mut refused = None;
        keep_first(&mut refused, self.stage.release(control));
        keep_first(&mut refused, self.kept.release(control));
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
        let start = element.attributes().len();
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
            name,
            mut attributes,
            ..
        }) = input.take()
        else {
            // Nothing was taken: only the plan begun above needs ending.
            kept.release(control)?;
            return Err(SessionError::Consumed);
        };
        let staged = (|| {
            name.release(control)?;
            let mut stage = StagedAttributes::new(control)?;
            let first_row = self.payloads.values.len();
            let result = self.stage_merged_attributes(
                control,
                target,
                start,
                count,
                &mut attributes,
                &mut stage,
                &mut kept,
            );
            match result {
                Ok(()) => Ok((stage, first_row)),
                Err(error) => {
                    stage.release(control)?;
                    Err(error)
                }
            }
        })();
        let (stage, first_row) = match staged {
            Ok((stage, first_row)) => {
                // The token's strings are held by the plan until its names are released, and by
                // nothing else once they are registered, so they go with the rest of the token.
                if let Err(error) = release_token_attributes(attributes, control) {
                    let staging = stage.release(control);
                    let planned = kept.release(control);
                    staging?;
                    planned?;
                    return Err(error);
                }
                (stage, first_row)
            }
            Err(error) => {
                // The names the plan kept hold the token strings' blocks, which their release
                // requires to be unheld: the plan goes first.
                let planned = kept.release(control);
                let cleanup = release_token_attributes(attributes, control);
                planned?;
                cleanup?;
                return Err(error);
            }
        };
        if count == 0 {
            stage.release(control)?;
            kept.release(control)?;
            return Ok(None);
        }
        let prepared = PreparedAttributeMerge {
            target,
            stage,
            kept,
            first_row,
            start,
            count,
        };
        if let Err(error) = payload_write::<PreparedAttributeMerge>(control, 1) {
            prepared.release(control)?;
            return Err(error);
        }
        Ok(Some(prepared))
    }

    /// Registers and stages the kept candidates, last first, so that the staging list pops in
    /// source order. A candidate the plan dropped is released, its lexical origins noted.
    #[allow(clippy::too_many_arguments)]
    fn stage_merged_attributes(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        start: usize,
        count: usize,
        attributes: &mut OwnedVec<crate::parser::tokenizer::token::Attribute>,
        stage: &mut StagedAttributes,
        kept: &mut OwnedVec<KeptCandidate>,
    ) -> Result<(), SessionError> {
        let mut rank = count;
        while !attributes.as_slice().is_empty() {
            let index = attributes
                .len()
                .checked_sub(1)
                .ok_or_else(|| latch_registry_row(control))?;
            let keep = rank
                .checked_sub(1)
                .and_then(|last| kept.as_slice().get(last))
                .is_some_and(|candidate| candidate.index == index);
            // A kept candidate also visits its entry twice, to lend its name and to take it back.
            // Both visits are charged before the attribute is taken: once the name is lent, a
            // refusal could leave the plan holding the block of a string that must be released.
            control.charge(HtmlParseWorkCategory::RecordVisit, if keep { 3 } else { 1 })?;
            let Some(attribute) = attributes.try_pop(control)? else {
                return Err(latch_registry_row(control));
            };
            if !keep {
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
                continue;
            }
            rank = rank
                .checked_sub(1)
                .ok_or_else(|| latch_registry_row(control))?;
            let position = start
                .checked_add(rank)
                .ok_or_else(|| control.stop_representation(HtmlParseResource::ManagedAttributes))?;
            // The token's name is the only holder of its block while it is registered.
            kept.as_mut_slice_prepaid(control)?
                .get_mut(rank)
                .ok_or_else(|| latch_registry_row(control))?
                .name = None;
            let (local, name_ticket) = match self.register_text(control, attribute.name) {
                Ok(value) => value,
                Err(error) => {
                    attribute.value.release(control)?;
                    return Err(error);
                }
            };
            if let Some(candidate) = kept.as_mut_slice_prepaid(control)?.get_mut(rank) {
                candidate.name = Some(local.clone());
            }
            let (value, value_ticket) = self.register_text(control, attribute.value)?;
            self.holds.acquire_attribute(control)?;
            stage.push(
                control,
                Attribute::new(QualifiedName::new(Namespace::none(), None, local), value),
                count,
            )?;
            self.bind_string(name_ticket, FieldRef::AttributeName(target, position))
                .map_err(|_| latch_registry_row(control))?;
            self.bind_string(value_ticket, FieldRef::AttributeValue(target, position))
                .map_err(|_| latch_registry_row(control))?;
        }
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

    /// Appends the staged attributes in source order. The index of each decision is the one the
    /// plan made, so the storage finds the plan's entries for them in its table of names.
    fn install_merged_attributes(
        &mut self,
        control: &mut ParseControl,
        prepared: &mut PreparedAttributeMerge,
    ) -> Result<(), SessionError> {
        self.payloads.forget_attribute_strings(prepared.target);
        // The list reserves room for every kept candidate once, so that no append of the merge
        // grows it and one wide merge leaves it with no spare room.
        self.reserve_attributes(control, prepared.target, prepared.count)?;
        for rank in 0..prepared.count {
            let candidate = prepared
                .kept
                .as_slice()
                .get(rank)
                .and_then(|candidate| candidate.name.clone())
                .ok_or_else(|| latch_registry_row(control))?;
            self.append_staged_attribute(
                control,
                prepared.target,
                &mut prepared.stage,
                Some(&candidate),
            )?;
            if self.effects.form.is_some() {
                self.drain_form_effects(control)?;
            }
        }
        self.flush_attribute_origins(control, prepared.first_row)
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

    /// Appends the last staged attribute to the element at `target`, with the effects of an `id`
    /// among the form controls. A refusal leaves the attribute staged. Returns the position it
    /// took.
    pub(super) fn append_staged_attribute(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        stage: &mut StagedAttributes,
        planned: Option<&DocumentString>,
    ) -> Result<usize, SessionError> {
        let Some(attribute) = stage.values.pop() else {
            return Err(latch_registry_row(control));
        };
        let mut incoming = Some(attribute);
        let appended = self.append_attribute_with_effects(control, target, &mut incoming, planned);
        if let Some(attribute) = incoming {
            // The slot that held the attribute is still there, so this allocates nothing.
            stage.values.push(attribute);
        }
        appended
    }

    /// Appends the attribute in `incoming` to the element at `target`. An `id` that a form
    /// control may resolve a `form` attribute through leaves the effects its change needs, which
    /// the caller drains. A refusal leaves `incoming` with the caller.
    pub(super) fn append_attribute_with_effects(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        incoming: &mut Option<Attribute>,
        planned: Option<&DocumentString>,
    ) -> Result<usize, SessionError> {
        let Some(attribute) = incoming.as_ref() else {
            return Err(latch_registry_row(control));
        };
        let form_effects = self.prepare_id_attribute_effects(control, target, attribute)?;
        if form_effects.is_some() {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        let position = self.append_attribute(control, target, incoming, planned)?;
        if form_effects.is_some() {
            self.effects.form = form_effects;
        }
        Ok(position)
    }

    /// The effects of an `id` attribute that `attribute`, about to be appended to the element at
    /// `target`, makes among the listed form controls that have a `form` attribute.
    pub(super) fn prepare_id_attribute_effects(
        &self,
        control: &mut ParseControl,
        target: usize,
        attribute: &Attribute,
    ) -> Result<Option<FormEffects>, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if attribute.name().namespace().uri().is_some()
            || !exact_ascii(control, Some(attribute.name().local_name()), "id")?
        {
            return Ok(None);
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if attribute.value().is_empty() {
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
