//! Common declaration orchestration over the CSS-owned Page and Fonts descriptor domains.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum DescriptorName {
    Page(CssPageDescriptorKind),
    FontFace(CssFontFaceDescriptorKind),
}
pub(super) fn descriptor_name(block: &CssomBlock, name: &str) -> Option<DescriptorName> {
    match block.data {
        CssomBlockData::Page { .. } => {
            CssPageDescriptorKind::from_name(name).map(DescriptorName::Page)
        }
        CssomBlockData::FontFace(_) => {
            CssFontFaceDescriptorKind::from_css_name(name).map(DescriptorName::FontFace)
        }
        _ => None,
    }
}
pub(super) fn font_selected(
    block: &CssomBlock,
) -> Result<&CssSpecifiedFontFaceDeclarationBlock, CssomError> {
    match &block.selected_font_face {
        Some(CssomProjection::Available(selected)) => Ok(selected),
        Some(CssomProjection::Unavailable(error)) => Err(CssomError::FontFace(error.clone())),
        None => Err(CssomError::WrongKind),
    }
}
fn page_selected(block: &CssomBlock) -> Result<&CssSpecifiedPageDeclarationBlock, CssomError> {
    match &block.data {
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected),
        CssomBlockData::Page {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Page(error.clone())),
        _ => Err(CssomError::WrongKind),
    }
}
pub(super) fn descriptor_priority(
    block: &CssomBlock,
    name: DescriptorName,
) -> Result<&'static str, CssomError> {
    match name {
        DescriptorName::Page(kind) => Ok(if page_selected(block)?.entries().iter().any(|entry| matches!(entry, CssSpecifiedPageDeclarationEntry::Descriptor(value) if value.value().kind() == kind && value.importance() == CssImportance::Important)) { "important" } else { "" }),
        DescriptorName::FontFace(_) => { font_selected(block)?; Ok("") }
    }
}
pub(super) fn descriptor_value(
    block: &CssomBlock,
    name: DescriptorName,
    budget: &mut DeclarationBudget,
) -> Result<String, CssomError> {
    match name {
        DescriptorName::Page(kind) => {
            let entry = page_selected(block)?
                .entries()
                .iter()
                .find_map(|entry| match entry {
                    CssSpecifiedPageDeclarationEntry::Descriptor(value)
                        if value.value().kind() == kind =>
                    {
                        Some(value)
                    }
                    _ => None,
                });
            match entry {
                Some(value) => value
                    .value()
                    .serialize_specified_with_limits(budget.provider()?)
                    .map_err(CssomError::Value),
                None => Ok(String::new()),
            }
        }
        DescriptorName::FontFace(kind) => {
            match font_selected(block)?
                .entries()
                .iter()
                .find(|entry| entry.value().kind() == kind)
            {
                Some(value) => value
                    .value()
                    .serialize_specified_with_limits(budget.provider()?)
                    .map_err(CssomError::Value),
                None => Ok(String::new()),
            }
        }
    }
}
pub(super) fn value_requirement(
    context: &CssomContext,
    value: &CssomDeclarationValue,
) -> Option<CssomUsabilityRequirement> {
    match value {
        CssomDeclarationValue::Property(value) => requirement(context, value),
        CssomDeclarationValue::PageDescriptor(value) => support(context)?
            .1
            .page_descriptors()
            .iter()
            .find(|(kind, _)| *kind == value.value().kind())
            .map(|(_, requirement)| *requirement),
        CssomDeclarationValue::FontFaceDescriptor(value) => support(context)?
            .1
            .font_face_descriptors()
            .iter()
            .find(|(kind, _)| *kind == value.value().kind())
            .map(|(_, requirement)| *requirement),
    }
}
impl CssomBatch {
    pub(super) fn prepare_descriptor(
        &mut self,
        id: &CssomBlockId,
        kind: DescriptorName,
        name: &str,
        value: &str,
        priority: &str,
        mut budget: DeclarationBudget,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        let supported = support(&self.staged.context).is_some_and(|(_, profile)| match kind {
            DescriptorName::Page(kind) => profile
                .page_descriptors()
                .iter()
                .any(|(candidate, _)| *candidate == kind),
            DescriptorName::FontFace(kind) => profile
                .font_face_descriptors()
                .iter()
                .any(|(candidate, _)| *candidate == kind),
        });
        if !supported {
            return Ok(self.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
        }
        if value.is_empty() {
            return Ok(self.preparation(
                id.clone(),
                PreparedAction::Remove(name.to_owned()),
                Vec::new(),
                budget,
            ));
        }
        budget.text(priority)?;
        let importance = if priority.is_empty() {
            CssImportance::Normal
        } else if priority.eq_ignore_ascii_case("important") {
            CssImportance::Important
        } else {
            return Ok(self.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
        };
        // Fonts owns descriptor priority: important descriptors are not admitted.
        if matches!(kind, DescriptorName::FontFace(_)) && importance == CssImportance::Important {
            return Ok(self.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
        }
        check_input_storage(value, self.limits.max_input_bytes)?;
        let component_limits = budget.input(value)?;
        if !complete_value_components(value, component_limits)? {
            return Ok(self.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
        }
        let (value, diagnostics) = match kind {
            DescriptorName::Page(kind) => {
                let (value, diagnostics) = parse_page_descriptor_value(value, kind).into_parts();
                (
                    value.map(|value| {
                        CssomDeclarationValue::PageDescriptor(CssPageDescriptor::new(
                            value, importance,
                        ))
                    }),
                    diagnostics,
                )
            }
            DescriptorName::FontFace(kind) => {
                let (value, diagnostics) =
                    parse_font_face_descriptor_value(value, kind).into_parts();
                (
                    value.map(|value| {
                        CssomDeclarationValue::FontFaceDescriptor(CssFontFaceDescriptor::new(value))
                    }),
                    diagnostics,
                )
            }
        };
        resource_diagnostics(&diagnostics)?;
        Ok(self.preparation(
            id.clone(),
            value.map_or(PreparedAction::Noop, PreparedAction::Set),
            diagnostics,
            budget,
        ))
    }
    pub(super) fn replace_font_contents(
        &mut self,
        id: &CssomBlockId,
        values: Vec<CssomDeclarationValue>,
        budget: &mut DeclarationBudget,
    ) -> Result<(), CssomError> {
        let entries = values
            .into_iter()
            .map(|value| match value {
                CssomDeclarationValue::FontFaceDescriptor(value) => Ok(value),
                _ => Err(CssomError::WrongKind),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let winners = font_winners(entries.clone());
        let selected = CssSpecifiedFontFaceDeclarationBlock::try_from_entries_with_limits(
            &winners,
            budget.provider()?,
        )
        .map_err(CssomError::FontFace)?;
        let block = self.staged.blocks.get_mut(id).expect("owned block");
        block.data = CssomBlockData::FontFace(CssFontFaceDescriptors::new(entries));
        block.selected_font_face = Some(CssomProjection::Available(selected));
        Ok(())
    }
    fn replace_font_selected(
        &mut self,
        id: &CssomBlockId,
        entries: &[CssFontFaceDescriptor],
        budget: &mut DeclarationBudget,
    ) -> Result<(), CssomError> {
        let selected = CssSpecifiedFontFaceDeclarationBlock::try_from_entries_with_limits(
            entries,
            budget.provider()?,
        )
        .map_err(CssomError::FontFace)?;
        self.staged
            .blocks
            .get_mut(id)
            .expect("owned block")
            .selected_font_face = Some(CssomProjection::Available(selected));
        Ok(())
    }
    pub(super) fn set_descriptor(
        &mut self,
        id: &CssomBlockId,
        value: CssomDeclarationValue,
        budget: &mut DeclarationBudget,
    ) -> Result<bool, CssomError> {
        let before = serialize_block(self.staged.block(id)?, budget)?;
        match value {
            CssomDeclarationValue::PageDescriptor(value) => {
                let mut entries = selected_entries(self.staged.block(id)?, budget)?;
                if let Some(index) = entries.iter().position(|entry| matches!(entry, SelectedEntry::Descriptor(old) if old.value().kind() == value.value().kind())) { entries[index] = SelectedEntry::Descriptor(value); }
                else { entries.push(SelectedEntry::Descriptor(value)); }
                self.replace_selected(id, entries, budget)?;
            }
            CssomDeclarationValue::FontFaceDescriptor(value) => {
                budget.scan(entry_count(self.staged.block(id)?)?)?;
                let mut entries = font_selected(self.staged.block(id)?)?.entries().to_vec();
                if let Some(index) = entries
                    .iter()
                    .position(|entry| entry.value().kind() == value.value().kind())
                {
                    entries[index] = value;
                } else {
                    entries.push(value);
                }
                self.replace_font_selected(id, &entries, budget)?;
            }
            _ => return Err(CssomError::WrongKind),
        }
        let updated = before != serialize_block(self.staged.block(id)?, budget)?;
        if updated {
            self.request_owner_update(id, budget)?;
        }
        self.declaration_changed(id);
        Ok(updated)
    }
    pub(super) fn remove_descriptor(
        &mut self,
        id: &CssomBlockId,
        kind: DescriptorName,
        budget: &mut DeclarationBudget,
    ) -> Result<bool, CssomError> {
        match kind {
            DescriptorName::Page(kind) => {
                let mut entries = selected_entries(self.staged.block(id)?, budget)?;
                let previous = entries.len();
                entries.retain(|entry| !matches!(entry, SelectedEntry::Descriptor(value) if value.value().kind() == kind));
                if previous == entries.len() {
                    return Ok(false);
                }
                self.replace_selected(id, entries, budget)?;
            }
            DescriptorName::FontFace(kind) => {
                budget.scan(entry_count(self.staged.block(id)?)?)?;
                let mut entries = font_selected(self.staged.block(id)?)?.entries().to_vec();
                let previous = entries.len();
                entries.retain(|entry| entry.value().kind() != kind);
                if previous == entries.len() {
                    return Ok(false);
                }
                self.replace_font_selected(id, &entries, budget)?;
            }
        }
        Ok(true)
    }
    pub(super) fn add_admission_inputs(
        &mut self,
        id: &CssomBlockId,
        inputs: Vec<CssomInputVersion>,
    ) {
        let facts = &mut self
            .staged
            .blocks
            .get_mut(id)
            .expect("owned block")
            .admission_inputs;
        for input in inputs {
            if !facts.contains(&input) {
                facts.push(input);
            }
        }
    }
}
