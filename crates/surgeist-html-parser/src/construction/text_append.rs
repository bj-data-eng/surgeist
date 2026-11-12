//! Character data joins the text before it: one direct append to the text of its record.
use super::*;
use surgeist_html::construction::ConstructionNodeKind;

impl SessionStorage {
    /// Appends the character token `text` to the text record at `index`, which `parent` holds
    /// as the sibling before the insertion location. The text grows in place through the
    /// construction owner's one admission; the token's own string is released afterwards, as
    /// it is when the append is refused.
    ///
    /// When the parse records the origin graph, the group of the text and the group of the
    /// token become the parts of a group that replaces the text's own in the table of origins,
    /// and the replaced group is retired. A refusal of the append leaves the text, the usage
    /// and the units the parse holds as they were. Three residues remain: the text keeps the
    /// capacity that a growth reserved and recorded before the refusal, and the bytes counter
    /// and the ledger hold the capacity last recorded (the capacity chosen, when the second
    /// record of the growth was refused and the text holds the capacity it reached); a text that
    /// had no group has the group of no source origins that the merge made for it; the evidence
    /// keeps the groups it made, which no field names, and the retirement of the replaced group.
    /// The leaf of the token's group in the merged group names units the text does not hold:
    /// reading it returns the wrong token. A parse that refuses an append stops, and a stopped
    /// parse publishes none of them.
    pub(super) fn merge_text(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        index: usize,
        text: Text,
        at_end: bool,
    ) -> Result<TextInsertion, SessionError> {
        let Text { value, origins } = text;
        let mut origins = Some(origins);
        let result = self.merge_units(
            control,
            parent,
            index,
            value.text().code_units(),
            &mut origins,
            at_end,
        );
        // The token's origins and string are released with it, whatever the append did.
        let origins = origins.map_or(Ok(()), |origins| origins.release(control));
        let value = value.release(control);
        let inserted = result?;
        origins?;
        value?;
        Ok(inserted)
    }

    fn merge_units(
        &mut self,
        control: &mut ParseControl,
        parent: usize,
        index: usize,
        incoming: &[u16],
        origins: &mut Option<OwnedVec<HtmlParseTextOrigin>>,
        at_end: bool,
    ) -> Result<TextInsertion, SessionError> {
        let field = FieldRef::Text(index);
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let ConstructionNodeKind::Text(old) = self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let offset = old
            .len()
            .map_err(|error| construction_error(control, error))?;
        let added = incoming.len();
        // Two live strings' lengths: their sum is bounded by the address space.
        let Some(total) = offset.checked_add(added) else {
            return Err(latch_registry_row(control));
        };
        let node = self.node(control, index)?;
        // Whether the text is a child of an element that reacts to its children changing.
        let storage = &self.construction;
        let children_changed = self.textarea_slot(control, parent)?.is_some()
            || Self::html_local_in(control, storage, parent, "script")?
            || Self::html_local_in(control, storage, parent, "style")?;
        if children_changed {
            payload_write::<Option<FormEffects>>(control, 1)?;
        }
        let merged =
            self.merge_groups_of_text(control, field, origins.take(), offset, added, total)?;
        let appended = self.append_text(control, index, incoming)?;
        if appended != offset {
            return Err(latch_registry_row(control));
        }
        if let Some(merged) = merged {
            // The replacement was charged before the append, and the field holds a group.
            self.payloads
                .detail_origins
                .replace_prepaid(control, field, merged)?;
        }
        if children_changed {
            self.appended_units = if at_end {
                Some(NonZeroUsize::new(added).ok_or_else(|| latch_registry_row(control))?)
            } else {
                None
            };
            self.effects.form = Some(FormEffects {
                root: parent,
                old_parent: None,
                new_parent: Some(parent),
                stage: if at_end {
                    FormEffectStage::AppendChildren
                } else {
                    FormEffectStage::InsertionChildren
                },
                insert: false,
                cause: InsertionCause::NonParser,
                next: None,
                pass: FormEffectPass::Local,
                href_base: base::HrefBaseAhead::Unscanned,
            });
            self.drain_form_effects(control)?;
        }
        Ok(TextInsertion {
            _node: node,
            _offset: offset,
        })
    }

    /// The group of the merged text, when the parse records the origin graph: the group of the
    /// text's origins followed by the group of the token's. The token's leaf names the units it
    /// will occupy, and the group of the text is retired as the text's group. The group of a
    /// text that has none is a group of no source origins, which the table then holds.
    fn merge_groups_of_text(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
        offset: usize,
        added: usize,
        total: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let Some(incoming) = self.origin_group(control, origins, added)? else {
            return Ok(None);
        };
        if self.payloads.detail_origins.get(control, field)?.is_none() {
            // An absent group is explicitly source independent. It is never attributed to the
            // triggering token merely because a later input has lexical origins.
            let group = self
                .graph()
                .ok_or(SessionError::WrongToken)?
                .source_independent_group(control, offset)?;
            self.bind_field(control, field, Some(group), 0)?;
        }
        self.payloads
            .detail_origins
            .charge_replace(control, field)?;
        let Some(old) = self.payloads.detail_origins.get(control, field)? else {
            return Err(latch_registry_row(control));
        };
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        let merged = evidence.merge_groups(control, old, &incoming, offset, total)?;
        evidence.bind_leaf_field(control, &incoming, field, offset)?;
        evidence.retire_group(control, old, field)?;
        Ok(Some(merged))
    }
}
