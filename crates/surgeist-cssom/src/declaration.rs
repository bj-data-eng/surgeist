//! Source declaration algorithms over current checked selections.
use crate::{
    declaration_support::{DeclarationBudget, PendingPreparation},
    *,
};
use std::sync::Arc;
use surgeist_css::*;
#[path = "declaration_descriptors.rs"]
mod descriptors;
use crate::css_adapter::font_winners;
use descriptors::{
    descriptor_name, descriptor_priority, descriptor_value, font_selected, value_requirement,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomDeclarationEditResult {
    Noop,
    Set { updated: bool },
    Replaced,
}
/// An owning preparation. Dropping it abandons that source operation without publishing.
pub struct CssomPreparedDeclaration {
    batch: Arc<()>,
    preparation: Arc<()>,
    block: CssomBlockId,
    context: CssomContext,
    action: PreparedAction,
    reflect_owner: bool,
    input: Option<CssomDeclarationInput>,
    candidates: Vec<CssomDeclarationCandidate>,
    decisions_required: Vec<usize>,
    diagnostics: Vec<CssRecoveryDiagnostic>,
    budget: DeclarationBudget,
    _pending: PendingPreparation,
}
#[derive(Clone)]
enum PreparedAction {
    Noop,
    Set(CssomDeclarationValue),
    Replace(Vec<CssomDeclarationValue>),
    Remove(String),
}
impl CssomPreparedDeclaration {
    pub fn candidates(&self) -> &[CssomDeclarationCandidate] {
        &self.candidates
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    pub fn profile_version(&self) -> Option<&CssomInputVersion> {
        self.context
            .inputs
            .iter()
            .find(|input| matches!(input.data, CssomInputData::Support(_)))
            .map(|input| &input.version)
    }
}
/// One source read view; flags and payloads belong to the owning immutable capture.
pub struct CssomDeclarationView<'a> {
    block: &'a CssomBlock,
    limits: CssomDeclarationRequestLimits,
}
impl CssomSnapshot {
    pub fn declarations(
        &self,
        id: &CssomBlockId,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomDeclarationView<'_>, CssomError> {
        Ok(CssomDeclarationView {
            block: self.block(id)?,
            limits,
        })
    }
}
impl CssomDeclarationView<'_> {
    pub fn parent_rule(&self) -> Option<&CssomRuleId> {
        self.block.parent()
    }
    pub fn flags(&self) -> CssomBlockFlags {
        self.block.flags()
    }
    pub fn length(&self) -> Result<usize, CssomError> {
        let mut budget = DeclarationBudget::new(self.limits, 1);
        let n = entry_count(self.block)?;
        budget.scan(n)?;
        Ok(n)
    }
    pub fn item(&self, index: usize) -> Result<String, CssomError> {
        let mut budget = DeclarationBudget::new(self.limits, 1);
        budget.scan(entry_count(self.block)?)?;
        let name = entry_name(self.block, index)?;
        budget.scalar_output(name)?;
        Ok(name.to_owned())
    }
    pub fn css_text(&self) -> Result<String, CssomError> {
        if self.block.flags.computed {
            return Ok(String::new());
        }
        serialize_block(self.block, &mut DeclarationBudget::new(self.limits, 1))
    }
    pub fn get_property_value(&self, name: &str) -> Result<String, CssomError> {
        query_value(
            self.block,
            name,
            &mut DeclarationBudget::new(self.limits, 3),
        )
    }
    pub fn get_property_priority(&self, name: &str) -> Result<String, CssomError> {
        let mut budget = DeclarationBudget::new(self.limits, 1);
        budget.text(name)?;
        if let Some(kind) = descriptor_name(self.block, name) {
            budget.scan(entry_count(self.block)?)?;
            let priority = descriptor_priority(self.block, kind)?;
            budget.scalar_output(priority)?;
            return Ok(priority.to_owned());
        }
        if matches!(self.block.data, CssomBlockData::FontFace(_)) {
            return Ok(String::new());
        }
        let Some(name) = normalize_in(name, self.block) else {
            return Ok(String::new());
        };
        budget.scan(entry_count(self.block)?)?;
        let entries = properties(self.block)?;
        let targets = targets(&name, &entries, &mut budget)?;
        let priority = if !targets.is_empty()
            && targets.iter().all(|target| {
                entries.iter().any(|entry| {
                    entry.property_name() == target.as_ref()
                        && entry.importance() == CssImportance::Important
                })
            }) {
            "important"
        } else {
            ""
        };
        budget.scalar_output(priority)?;
        Ok(priority.to_owned())
    }
    pub fn css_float(&self) -> Result<String, CssomError> {
        self.get_property_value("float")
    }
}
#[derive(Clone)]
enum Name {
    SvgGlyph,
    Known(CssPropertyGrammar),
    Custom(CssCustomPropertyName),
}
impl Name {
    fn as_ref(&self) -> CssPropertyNameRef<'_> {
        match self {
            Self::SvgGlyph => CssSvgGlyphOrientationVerticalDeclaration::metadata().property(),
            Self::Known(grammar) => CssPropertyNameRef::Known(grammar.target_property()),
            Self::Custom(name) => CssPropertyNameRef::Custom(name),
        }
    }
}
fn normalize(name: &str) -> Option<Name> {
    if name.starts_with("--") {
        CssCustomPropertyName::try_from_decoded(name).map(Name::Custom)
    } else {
        CssPropertyGrammar::from_name(&name.to_ascii_lowercase()).map(Name::Known)
    }
}
fn targets(
    name: &Name,
    _entries: &[CssSpecifiedDeclarationEntry],
    budget: &mut DeclarationBudget,
) -> Result<Vec<Name>, CssomError> {
    let Name::Known(grammar) = name else {
        budget.scan(1)?;
        return Ok(vec![name.clone()]);
    };
    let meta = grammar.metadata().map_err(CssomError::Metadata)?;
    let members = match meta.kind() {
        CssPropertyKindRef::Longhand(_) => {
            budget.scan(1)?;
            return Ok(vec![name.clone()]);
        }
        CssPropertyKindRef::Shorthand(meta) => {
            budget.scan(meta.members().len())?;
            meta.members().to_vec()
        }
        CssPropertyKindRef::FourSideShorthand(meta) => {
            budget.scan(meta.members(CssBoxSideKind::Physical).len())?;
            meta.members(CssBoxSideKind::Physical).to_vec()
        }
        CssPropertyKindRef::UniversalReset(meta) => {
            budget.scan(CssKnownProperty::all().len())?;
            let mut members = Vec::new();
            for property in CssKnownProperty::all() {
                if !meta.excludes(CssPropertyNameRef::Known(*property))
                    && matches!(
                        property.metadata().map_err(CssomError::Metadata)?.kind(),
                        CssPropertyKindRef::Longhand(_)
                    )
                {
                    members.push(Name::Known(property.grammar()));
                }
            }
            return Ok(members);
        }
        _ => {
            return Err(CssomError::InvalidInput(
                "unavailable declaration metadata branch",
            ));
        }
    };
    Ok(members
        .iter()
        .map(|member| Name::Known(member.known_property().grammar()))
        .collect())
}
fn properties(block: &CssomBlock) -> Result<Vec<CssSpecifiedDeclarationEntry>, CssomError> {
    match &block.data {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected.entries().to_vec()),
        CssomBlockData::Properties {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Declaration(error.clone())),
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected
            .entries()
            .iter()
            .filter_map(|entry| {
                if let CssSpecifiedPageDeclarationEntry::Property(entry) = entry {
                    Some(entry.clone())
                } else {
                    None
                }
            })
            .collect()),
        CssomBlockData::Page {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Page(error.clone())),
        _ => Err(CssomError::WrongKind),
    }
}
fn selected_properties(
    block: &CssomBlock,
    entries: &[CssSpecifiedDeclarationEntry],
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<CssSpecifiedDeclarationBlock, CssomError> {
    match &block.data {
        CssomBlockData::Properties { domain, .. } => match domain {
            CssomPropertyDomain::Ordinary => {
                CssSpecifiedDeclarationBlock::try_from_entries_with_limits(entries, limits)
            }
            CssomPropertyDomain::Keyframe => {
                CssSpecifiedDeclarationBlock::try_from_keyframe_entries_with_limits(entries, limits)
            }
            CssomPropertyDomain::Margin => {
                CssSpecifiedDeclarationBlock::try_from_margin_entries_with_limits(entries, limits)
            }
        },
        CssomBlockData::Page { .. } => {
            CssSpecifiedDeclarationBlock::try_from_page_entries_with_limits(entries, limits)
        }
        _ => return Err(CssomError::WrongKind),
    }
    .map_err(CssomError::Declaration)
}
fn entry_name(block: &CssomBlock, index: usize) -> Result<&str, CssomError> {
    match &block.data {
        CssomBlockData::FontFace(_) => Ok(font_selected(block)?
            .entries()
            .get(index)
            .map_or("", |entry| entry.value().kind().css_name())),
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected
            .entries()
            .get(index)
            .map_or("", |entry| match entry {
                CssSpecifiedPageDeclarationEntry::Property(entry) => {
                    property_name(entry.property_name())
                }
                CssSpecifiedPageDeclarationEntry::Descriptor(entry) => {
                    entry.value().kind().css_name()
                }
                _ => unreachable!("CSS Page entry"),
            })),
        CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected
            .entries()
            .get(index)
            .map_or("", |entry| property_name(entry.property_name()))),
        CssomBlockData::Properties {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Declaration(error.clone())),
        CssomBlockData::Page {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Page(error.clone())),
        _ => Err(CssomError::WrongKind),
    }
}
pub(crate) fn serialize_block(
    block: &CssomBlock,
    budget: &mut DeclarationBudget,
) -> Result<String, CssomError> {
    match &block.data {
        CssomBlockData::FontFace(_) => font_selected(block)?
            .serialize_cssom_with_limits(budget.provider()?)
            .map_err(CssomError::FontFace),
        CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } => selected
            .serialize_cssom_with_limits(budget.provider()?)
            .map_err(CssomError::Declaration),
        CssomBlockData::Properties {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Declaration(error.clone())),
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => selected
            .serialize_cssom_with_limits(budget.provider()?)
            .map_err(CssomError::Format),
        CssomBlockData::Page {
            selected: CssomProjection::Unavailable(error),
            ..
        } => Err(CssomError::Page(error.clone())),
        _ => Err(CssomError::WrongKind),
    }
}
fn query_value(
    block: &CssomBlock,
    name: &str,
    budget: &mut DeclarationBudget,
) -> Result<String, CssomError> {
    budget.text(name)?;
    if let Some(descriptor) = descriptor_name(block, name) {
        budget.scan(entry_count(block)?)?;
        return descriptor_value(block, descriptor, budget);
    }
    if matches!(block.data, CssomBlockData::FontFace(_)) {
        return Ok(String::new());
    }
    let Some(name) = normalize_in(name, block) else {
        return Ok(String::new());
    };
    budget.scan(entry_count(block)?)?;
    let mut entries = properties(block)?;
    if let Name::Known(grammar) = &name
        && matches!(
            grammar.metadata().map_err(CssomError::Metadata)?.kind(),
            CssPropertyKindRef::FourSideShorthand(_)
        )
    {
        let members = targets(&name, &entries, budget)?;
        entries.retain(|entry| {
            members
                .iter()
                .any(|member| entry.property_name() == member.as_ref())
        });
    }
    let projection = selected_properties(block, &entries, budget.provider()?)?;
    projection
        .property_value_with_limits(name.as_ref(), budget.provider()?)
        .map(|value| value.unwrap_or_default())
        .map_err(CssomError::Declaration)
}
// Complete setter values reject tokenizer syntax as a source null parse.
// Resource and other checked-provider failures retain their typed cause.
pub(crate) fn complete_value_components(
    value: &str,
    limits: CssComponentValueLimits,
) -> Result<bool, CssomError> {
    match parse_component_values_with_limits(value, limits) {
        Ok(_) => Ok(true),
        Err(error)
            if matches!(
                error.kind(),
                CssComponentValueErrorKind::BadString
                    | CssComponentValueErrorKind::BadUrl
                    | CssComponentValueErrorKind::UnmatchedClosingDelimiter
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(CssomError::Component(error)),
    }
}
pub(crate) fn resource_diagnostics(
    diagnostics: &[CssRecoveryDiagnostic],
) -> Result<(), CssomError> {
    for diagnostic in diagnostics {
        if matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
            || matches!(diagnostic.error().kind(),ErrorKind::InvalidComponentValue(e) if matches!(e.kind(),CssComponentValueErrorKind::NestingLimit|CssComponentValueErrorKind::ComponentLimit|CssComponentValueErrorKind::ByteLimit|CssComponentValueErrorKind::CapacityOverflow))
        {
            return Err(CssomError::ParseResource(Box::new(diagnostic.clone())));
        }
    }
    Ok(())
}
fn source_precondition(block: &CssomBlock) -> Result<(), CssomError> {
    if block.flags.computed {
        Err(CssomError::ComputedStyleUpdatePrecondition)
    } else {
        Ok(())
    }
}
fn readonly(block: &CssomBlock) -> Result<(), CssomError> {
    if block.flags.readonly {
        Err(CssomError::Source(CssomException::NoModificationAllowed))
    } else {
        Ok(())
    }
}
fn support(context: &CssomContext) -> Option<(&CssomInputVersion, &CssomDeclarationSupport)> {
    context.inputs.iter().find_map(|input| {
        if let CssomInputData::Support(profile) = &input.data {
            Some((&input.version, profile))
        } else {
            None
        }
    })
}
fn requirement(
    context: &CssomContext,
    declaration: &CssDeclaration,
) -> Option<CssomUsabilityRequirement> {
    if declaration.custom().is_some() {
        Some(CssomUsabilityRequirement::AllCheckedValues)
    } else if declaration.svg_glyph_orientation_vertical().is_some() {
        support(context)?.1.svg_glyph_orientation_vertical()
    } else {
        declaration
            .known()
            .and_then(|known| support(context)?.1.requirement(known.grammar()))
    }
}
impl CssomBatch {
    fn preparation(
        &mut self,
        block: CssomBlockId,
        action: PreparedAction,
        diagnostics: Vec<CssRecoveryDiagnostic>,
        budget: DeclarationBudget,
    ) -> CssomPreparedDeclaration {
        let preparation = Arc::new(());
        let declarations = match &action {
            PreparedAction::Set(value) => vec![value.clone()],
            PreparedAction::Replace(items) => items.clone(),
            PreparedAction::Noop | PreparedAction::Remove(_) => Vec::new(),
        };
        let mut candidates = Vec::new();
        let mut decisions_required = Vec::new();
        for (ordinal, value) in declarations.into_iter().enumerate() {
            if value_requirement(&self.staged.context, &value)
                == Some(CssomUsabilityRequirement::WholeDeclarationDecision)
            {
                decisions_required.push(ordinal);
                candidates.push(CssomDeclarationCandidate {
                    preparation: preparation.clone(),
                    ordinal,
                    value,
                });
            }
        }
        CssomPreparedDeclaration {
            batch: self.token.clone(),
            preparation,
            block,
            context: self.staged.context.clone(),
            action,
            reflect_owner: true,
            input: None,
            candidates,
            decisions_required,
            diagnostics,
            budget,
            _pending: PendingPreparation::new(self.pending_preparations.clone()),
        }
    }
    pub fn prepare_css_text(
        &mut self,
        id: &CssomBlockId,
        text: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.prepare_contents(id, text, limits, true)
    }
    fn prepare_contents(
        &mut self,
        id: &CssomBlockId,
        text: &str,
        limits: CssomDeclarationRequestLimits,
        reflect_owner: bool,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.apply(|this| {
            if reflect_owner {
                readonly(this.staged.block(id)?)?;
            }
            let mut budget = DeclarationBudget::new(limits, 12);
            check_input_storage(text, this.limits.max_input_bytes)?;
            let component_limits = budget.input(text)?;
            let (items, diagnostics) =
                if matches!(this.staged.block(id)?.data, CssomBlockData::Page { .. }) {
                    let report = this
                        .staged
                        .context
                        .parser_context()
                        .parse_page_declaration_block_contents_with_limits(text, component_limits);
                    let (body, diagnostics) = report.into_parts();
                    resource_diagnostics(&diagnostics)?;
                    let body = body.ok_or_else(|| {
                        diagnostics
                            .first()
                            .map(|diagnostic| {
                                CssomError::ParseResource(Box::new(diagnostic.clone()))
                            })
                            .unwrap_or(CssomError::InvalidInput("Page contents resource failure"))
                    })?;
                    (
                        body.occurrences()
                            .iter()
                            .filter_map(|item| match item {
                                CssPageDeclaration::Property(value) => {
                                    Some(CssomDeclarationValue::Property(value.clone()))
                                }
                                CssPageDeclaration::Descriptor(value) => {
                                    Some(CssomDeclarationValue::PageDescriptor(value.clone()))
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                        diagnostics,
                    )
                } else if matches!(this.staged.block(id)?.data, CssomBlockData::FontFace(_)) {
                    let (body, diagnostics) =
                        parse_font_face_declaration_block_contents_with_limits(
                            text,
                            component_limits,
                        )
                        .into_parts();
                    resource_diagnostics(&diagnostics)?;
                    let body = body.ok_or(CssomError::InvalidInput(
                        "FontFace contents resource failure",
                    ))?;
                    (
                        body.occurrences()
                            .cloned()
                            .map(CssomDeclarationValue::FontFaceDescriptor)
                            .collect(),
                        diagnostics,
                    )
                } else {
                    let report = this
                        .staged
                        .context
                        .parser_context()
                        .parse_declaration_block_contents_with_limits(text, component_limits);
                    let (body, diagnostics) = report.into_parts();
                    resource_diagnostics(&diagnostics)?;
                    (
                        body.iter()
                            .cloned()
                            .map(CssomDeclarationValue::Property)
                            .collect(),
                        diagnostics,
                    )
                };
            budget.scan(items.len())?;
            budget.phases(
                items
                    .len()
                    .checked_mul(4)
                    .and_then(|n| n.checked_add(6))
                    .ok_or(CssomError::Limit {
                        resource: "declaration phases",
                        maximum: limits.max_entries,
                    })?,
            );
            let input = CssomDeclarationInput {
                source: text.to_owned(),
                parser_context: this.staged.context.parser_context(),
                diagnostics: diagnostics.clone(),
            };
            let mut preparation = this.preparation(
                id.clone(),
                PreparedAction::Replace(items),
                diagnostics,
                budget,
            );
            preparation.reflect_owner = reflect_owner;
            preparation.input = Some(input);
            Ok(preparation)
        })
    }
    pub fn prepare_set_property(
        &mut self,
        id: &CssomBlockId,
        name: &str,
        value: &str,
        priority: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.apply(|this| {
            readonly(this.staged.block(id)?)?;
            let mut budget = DeclarationBudget::new(limits, 12);
            budget.text(name)?;
            if let Some(kind) = descriptor_name(this.staged.block(id)?, name) {
                return this.prepare_descriptor(id, kind, name, value, priority, budget);
            }
            if matches!(this.staged.block(id)?.data, CssomBlockData::FontFace(_)) {
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            }
            let parser_context = this.staged.context.parser_context();
            let svg = CssSvgGlyphOrientationVerticalDeclaration::metadata();
            let normalized = if name.eq_ignore_ascii_case(svg.name())
                && parser_context == parser_context.with_svg_glyph_orientation_vertical()
            {
                Some(Name::SvgGlyph)
            } else if name.starts_with("--") {
                normalize_in(name, this.staged.block(id)?)
            } else {
                normalize(name)
            };
            let Some(normalized) = normalized else {
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            };
            if let Name::Known(grammar) = normalized.clone()
                && support(&this.staged.context)
                    .and_then(|(_, support)| support.requirement(grammar))
                    .is_none()
            {
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            }
            if matches!(normalized, Name::SvgGlyph)
                && support(&this.staged.context)
                    .and_then(|(_, profile)| profile.svg_glyph_orientation_vertical())
                    .is_none()
            {
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            }
            if value.is_empty() {
                return Ok(this.preparation(
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
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            };
            check_input_storage(value, this.limits.max_input_bytes)?;
            let component_limits = budget.input(value)?;
            if !complete_value_components(value, component_limits)? {
                return Ok(this.preparation(id.clone(), PreparedAction::Noop, Vec::new(), budget));
            }
            let context = this.staged.context.parser_context();
            let report = match normalized {
                Name::SvgGlyph => context.parse_property_value_text(
                    value,
                    CssSvgGlyphOrientationVerticalDeclaration::metadata().property(),
                    importance,
                ),
                Name::Known(grammar) => {
                    context.parse_property_value_text_for_grammar(value, grammar, importance)
                }
                Name::Custom(name) => context.parse_property_value_text(
                    value,
                    CssPropertyNameRef::Custom(&name),
                    importance,
                ),
            };
            let (mut declaration, diagnostics) = report.into_parts();
            resource_diagnostics(&diagnostics)?;
            if let Some(value) = declaration.as_ref()
                && matches!(
                    this.staged.block(id)?.data,
                    CssomBlockData::Page { .. }
                        | CssomBlockData::Properties {
                            domain: CssomPropertyDomain::Margin,
                            ..
                        }
                )
            {
                declaration = match CssMarginDeclarationBlock::try_new_with_limits(
                    vec![value.clone()],
                    budget.provider()?,
                ) {
                    Ok(block) => block.properties().iter().next().cloned(),
                    Err(error) if page_rejection(&error) => None,
                    Err(error) => return Err(CssomError::PageBlock(error)),
                };
            }
            let action = declaration.map_or(PreparedAction::Noop, |value| {
                PreparedAction::Set(CssomDeclarationValue::Property(value))
            });
            Ok(this.preparation(id.clone(), action, diagnostics, budget))
        })
    }
    /// Supplied inline attribute notification; qualifying external changes do not echo a host write.
    /// Unrelated local names/namespaces and computed blocks take the source no-op branch.
    pub fn prepare_style_attribute_change(
        &mut self,
        id: &CssomBlockId,
        change: &CssomStyleAttributeChange,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<Option<CssomPreparedDeclaration>, CssomError> {
        self.apply(|this| {
            let block = this.staged.block(id)?;
            if change.local_name != "style"
                || change.namespace.is_some()
                || block.flags.computed
                || block.flags.updating
            {
                return Ok(None);
            }
            if block.owner.as_ref() != Some(&change.owner) {
                return Err(CssomError::StaleOwnerNotification);
            }
            this.prepare_contents(id, change.value.as_deref().unwrap_or(""), limits, false)
                .map(Some)
        })
    }
    pub fn prepare_css_float(
        &mut self,
        id: &CssomBlockId,
        value: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomPreparedDeclaration, CssomError> {
        self.prepare_set_property(id, "float", value, "", limits)
    }
    pub fn apply_declaration(
        &mut self,
        prepared: CssomPreparedDeclaration,
        decisions: &[CssomDeclarationDecision],
    ) -> Result<CssomDeclarationEditResult, CssomError> {
        let current = self
            .staged
            .context
            .inputs
            .iter()
            .filter(|input| input.version.role == CssomInputRole::Support)
            .map(|input| input.version.clone())
            .collect::<Vec<_>>();
        self.apply_declaration_with_inputs(prepared, decisions, &current)
    }
    /// Current independently revised composed-owner decision inputs, supplied without callbacks.
    pub fn apply_declaration_with_inputs(
        &mut self,
        mut prepared: CssomPreparedDeclaration,
        decisions: &[CssomDeclarationDecision],
        current: &[CssomInputVersion],
    ) -> Result<CssomDeclarationEditResult, CssomError> {
        self.apply(|this| {
            prepared.budget.scan(current.len())?;
            for (index, input) in current.iter().enumerate() {
                if input.role != CssomInputRole::Support
                    || current[..index]
                        .iter()
                        .any(|previous| previous.identity == input.identity)
                {
                    return Err(CssomError::InvalidInput(
                        "ambiguous current usability decision inputs",
                    ));
                }
            }
            if current.iter().any(|current| {
                prepared.context.inputs.iter().any(|captured| {
                    captured.version.role == CssomInputRole::Support
                        && captured.version.identity == current.identity
                        && captured.version != *current
                })
            }) {
                return Err(CssomError::StaleDeclarationDecision);
            }
            if !Arc::ptr_eq(&this.token, &prepared.batch) {
                return Err(CssomError::ForeignTicket);
            }
            if this.staged.context != prepared.context {
                return Err(CssomError::StaleDeclarationDecision);
            }
            if prepared.reflect_owner {
                readonly(this.staged.block(&prepared.block)?)?;
            }
            if decisions.len() != prepared.decisions_required.len() {
                return Err(CssomError::UnresolvedDeclarationPreparation);
            }
            for (index, decision) in decisions.iter().enumerate() {
                if !current.contains(&decision.version)
                    || !Arc::ptr_eq(&decision.preparation, &prepared.preparation)
                    || !prepared.decisions_required.contains(&decision.ordinal)
                    || decisions[..index]
                        .iter()
                        .any(|previous| previous.ordinal == decision.ordinal)
                {
                    return Err(CssomError::StaleDeclarationDecision);
                }
            }
            let usable = |ordinal: usize, value: &CssomDeclarationValue| match value_requirement(
                &prepared.context,
                value,
            ) {
                Some(CssomUsabilityRequirement::AllCheckedValues) => true,
                Some(CssomUsabilityRequirement::WholeDeclarationDecision) => {
                    decisions.iter().any(|decision| {
                        decision.ordinal == ordinal
                            && decision.usability == CssomDeclarationUsability::Usable
                    })
                }
                None => false,
            };
            let mut admission_inputs = Vec::new();
            if let Some((version, _)) = support(&prepared.context) {
                admission_inputs.push(version.clone());
            }
            admission_inputs.extend(decisions.iter().map(|decision| decision.version.clone()));
            let id = &prepared.block;
            match prepared.action.clone() {
                PreparedAction::Noop => Ok(CssomDeclarationEditResult::Noop),
                PreparedAction::Remove(name) => {
                    this.remove_internal(&prepared.block, &name, &mut prepared.budget)?;
                    Ok(CssomDeclarationEditResult::Noop)
                }
                PreparedAction::Set(value) => {
                    if !usable(0, &value) {
                        return Ok(CssomDeclarationEditResult::Noop);
                    }
                    if !matches!(value, CssomDeclarationValue::Property(_)) {
                        let updated = this.set_descriptor(id, value, &mut prepared.budget)?;
                        this.add_admission_inputs(id, admission_inputs);
                        return Ok(CssomDeclarationEditResult::Set { updated });
                    }
                    let CssomDeclarationValue::Property(declaration) = value else {
                        unreachable!()
                    };
                    let new =
                        match this.project_declarations(id, &[declaration], &mut prepared.budget) {
                            Ok(entries) => entries,
                            Err(CssomError::PageBlock(error)) if page_rejection(&error) => {
                                return Ok(CssomDeclarationEditResult::Noop);
                            }
                            Err(CssomError::Declaration(error))
                                if matches!(
                                    error.kind(),
                                    CssDeclarationBlockErrorKind::InvalidKeyframeSource { .. }
                                        | CssDeclarationBlockErrorKind::InvalidPageSource
                                ) =>
                            {
                                return Ok(CssomDeclarationEditResult::Noop);
                            }
                            Err(error) => return Err(error),
                        };
                    let before = serialize_block(this.staged.block(id)?, &mut prepared.budget)?;
                    let mut entries =
                        selected_entries(this.staged.block(id)?, &mut prepared.budget)?;
                    prepared.budget.scan(new.len())?;
                    for target in new {
                        let old = entries.iter().position(|entry| {
                            entry.property().is_some_and(|entry| {
                                entry.property_name() == target.property_name()
                            })
                        });
                        if let Some(index) = old {
                            entries.remove(index);
                        }
                        let mapping = longhand(target.property_name())?;
                        let mut conflict = None;
                        for (index, entry) in entries.iter().enumerate() {
                            if let Some(other) = entry
                                .property()
                                .map(|entry| longhand(entry.property_name()))
                                .transpose()?
                                .flatten()
                                && mapping.is_some_and(|mapping| {
                                    mapping.has_mapping_order_conflict(other)
                                })
                            {
                                conflict = Some(index);
                            }
                        }
                        let insert = match old {
                            None => entries.len(),
                            Some(old) => conflict.map_or(old.min(entries.len()), |conflict| {
                                old.max(conflict + 1).min(entries.len())
                            }),
                        };
                        entries.insert(insert, SelectedEntry::Property(target));
                    }
                    this.replace_selected(id, entries, &mut prepared.budget)?;
                    let after = serialize_block(this.staged.block(id)?, &mut prepared.budget)?;
                    let updated = before != after;
                    if updated {
                        this.request_owner_update(id, &prepared.budget)?;
                    }
                    let facts = &mut this
                        .staged
                        .blocks
                        .get_mut(id)
                        .expect("owned block")
                        .admission_inputs;
                    for input in admission_inputs {
                        if !facts.contains(&input) {
                            facts.push(input);
                        }
                    }
                    this.declaration_changed(id);
                    Ok(CssomDeclarationEditResult::Set { updated })
                }
                PreparedAction::Replace(items) => {
                    let previous_empty = entry_count(this.staged.block(id)?)? == 0;
                    let previous_authored_empty = match &this.staged.block(id)?.data {
                        CssomBlockData::Properties {
                            authored:
                                CssomPropertyOccurrences::Ordinary(v)
                                | CssomPropertyOccurrences::RawContents(v),
                            ..
                        } => v.is_empty(),
                        _ => false,
                    };
                    let previous_admissions = this.staged.block(id)?.admission_inputs.clone();
                    let previous_input = this.staged.block(id)?.replacement_input.clone();
                    let retained = items
                        .into_iter()
                        .enumerate()
                        .filter_map(|(ordinal, value)| usable(ordinal, &value).then_some(value))
                        .collect::<Vec<_>>();
                    prepared.budget.scan(retained.len())?;
                    let page = matches!(this.staged.block(id)?.data, CssomBlockData::Page { .. });
                    if page {
                        let authored = CssPageDeclarationBlock::try_new_with_limits(
                            retained
                                .into_iter()
                                .map(|value| match value {
                                    CssomDeclarationValue::Property(value) => {
                                        Ok(CssPageDeclaration::Property(value))
                                    }
                                    CssomDeclarationValue::PageDescriptor(value) => {
                                        Ok(CssPageDeclaration::Descriptor(value))
                                    }
                                    _ => Err(CssomError::WrongKind),
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                            prepared.budget.provider()?,
                        )
                        .map_err(CssomError::PageBlock)?;
                        let selected = authored
                            .try_specified_with_limits(prepared.budget.provider()?)
                            .map_err(CssomError::Page)?;
                        this.staged.blocks.get_mut(id).expect("owned block").data =
                            CssomBlockData::Page {
                                authored,
                                selected: CssomProjection::Available(selected),
                            };
                    } else if matches!(this.staged.block(id)?.data, CssomBlockData::FontFace(_)) {
                        this.replace_font_contents(id, retained, &mut prepared.budget)?;
                    } else {
                        let declarations = retained
                            .into_iter()
                            .filter_map(|item| {
                                if let CssomDeclarationValue::Property(declaration) = item {
                                    Some(declaration)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let mut entries = Vec::new();
                        let mut admitted = Vec::new();
                        for declaration in declarations {
                            match this.project_declarations(
                                id,
                                std::slice::from_ref(&declaration),
                                &mut prepared.budget,
                            ) {
                                Ok(new) => {
                                    for entry in new {
                                        if let Some(index) = entries.iter().position(
                                            |old: &CssSpecifiedDeclarationEntry| {
                                                old.property_name() == entry.property_name()
                                            },
                                        ) {
                                            if entries[index].importance()
                                                == CssImportance::Important
                                                && entry.importance() != CssImportance::Important
                                            {
                                                continue;
                                            }
                                            entries.remove(index);
                                        }
                                        entries.push(entry);
                                    }
                                    admitted.push(declaration);
                                }
                                Err(CssomError::PageBlock(error)) if page_rejection(&error) => {}
                                Err(CssomError::Declaration(error))
                                    if matches!(
                                        error.kind(),
                                        CssDeclarationBlockErrorKind::InvalidKeyframeSource { .. }
                                            | CssDeclarationBlockErrorKind::InvalidPageSource
                                    ) => {}
                                Err(error) => return Err(error),
                            }
                        }
                        this.replace_properties(id, entries, &mut prepared.budget)?;
                        if let CssomBlockData::Properties { authored, .. } =
                            &mut this.staged.blocks.get_mut(id).expect("owned block").data
                        {
                            *authored = CssomPropertyOccurrences::RawContents(
                                CssDeclarationList::try_new_with_limits(
                                    admitted,
                                    prepared.budget.provider()?,
                                )
                                .map_err(CssomError::Value)?,
                            );
                        }
                    }
                    if prepared.reflect_owner {
                        this.request_owner_update(id, &prepared.budget)?;
                    }
                    this.staged
                        .blocks
                        .get_mut(id)
                        .expect("owned block")
                        .admission_inputs = admission_inputs;
                    this.staged
                        .blocks
                        .get_mut(id)
                        .expect("owned block")
                        .replacement_input = prepared.input.take();
                    if previous_input != this.staged.block(id)?.replacement_input
                        || !previous_empty
                        || !previous_authored_empty
                        || entry_count(this.staged.block(id)?)? != 0
                        || previous_admissions != this.staged.block(id)?.admission_inputs
                    {
                        this.declaration_changed(id);
                    }
                    Ok(CssomDeclarationEditResult::Replaced)
                }
            }
        })
    }
    pub fn remove_property(
        &mut self,
        id: &CssomBlockId,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<String, CssomError> {
        self.apply(|this| this.remove_internal(id, name, &mut DeclarationBudget::new(limits, 8)))
    }
    fn remove_internal(
        &mut self,
        id: &CssomBlockId,
        name: &str,
        budget: &mut DeclarationBudget,
    ) -> Result<String, CssomError> {
        readonly(self.staged.block(id)?)?;
        let old = query_value(self.staged.block(id)?, name, budget)?;
        if let Some(kind) = descriptor_name(self.staged.block(id)?, name) {
            let changed = self.remove_descriptor(id, kind, budget)?;
            if changed {
                self.request_owner_update(id, budget)?;
                self.declaration_changed(id);
            }
            return Ok(old);
        }
        if matches!(self.staged.block(id)?.data, CssomBlockData::FontFace(_)) {
            return Ok(old);
        }
        let Some(name) = normalize_in(name, self.staged.block(id)?) else {
            return Ok(old);
        };
        let props = properties(self.staged.block(id)?)?;
        let targets = targets(&name, &props, budget)?;
        let mut entries = selected_entries(self.staged.block(id)?, budget)?;
        let previous = entries.len();
        entries.retain(|entry| {
            entry.property().is_none_or(|entry| {
                !targets
                    .iter()
                    .any(|target| target.as_ref() == entry.property_name())
            })
        });
        if entries.len() != previous {
            self.replace_selected(id, entries, budget)?;
            self.request_owner_update(id, budget)?;
            self.declaration_changed(id);
        }
        Ok(old)
    }
    fn declaration_changed(&mut self, id: &CssomBlockId) {
        if !self.declaration_changes.contains(id) {
            self.declaration_changes.push(id.clone());
        }
        self.affect(
            CssomObjectId::Block(id.clone()),
            &[CssomChange::Declarations, CssomChange::Provenance],
        );
    }
    fn request_owner_update(
        &mut self,
        id: &CssomBlockId,
        budget: &DeclarationBudget,
    ) -> Result<(), CssomError> {
        let block = self.staged.block(id)?;
        source_precondition(block)?;
        if block.owner.is_some() {
            self.owner_updates.retain(|(old, _)| old != id);
            self.owner_updates.push((id.clone(), budget.clone()));
        }
        Ok(())
    }
    fn replace_properties(
        &mut self,
        id: &CssomBlockId,
        entries: Vec<CssSpecifiedDeclarationEntry>,
        budget: &mut DeclarationBudget,
    ) -> Result<(), CssomError> {
        self.replace_selected(
            id,
            entries.into_iter().map(SelectedEntry::Property).collect(),
            budget,
        )
    }
    fn replace_selected(
        &mut self,
        id: &CssomBlockId,
        entries: Vec<SelectedEntry>,
        budget: &mut DeclarationBudget,
    ) -> Result<(), CssomError> {
        if matches!(self.staged.block(id)?.data, CssomBlockData::Page { .. }) {
            let mixed = entries
                .into_iter()
                .map(|entry| match entry {
                    SelectedEntry::Property(entry) => {
                        CssSpecifiedPageDeclarationEntry::Property(entry)
                    }
                    SelectedEntry::Descriptor(entry) => {
                        CssSpecifiedPageDeclarationEntry::Descriptor(entry)
                    }
                })
                .collect::<Vec<_>>();
            let selected = CssSpecifiedPageDeclarationBlock::try_from_entries_with_limits(
                &mixed,
                budget.provider()?,
            )
            .map_err(CssomError::Page)?;
            let CssomBlockData::Page {
                selected: current, ..
            } = &mut self.staged.blocks.get_mut(id).expect("owned block").data
            else {
                unreachable!()
            };
            *current = CssomProjection::Available(selected);
        } else {
            let properties = entries
                .into_iter()
                .map(|entry| match entry {
                    SelectedEntry::Property(entry) => Ok(entry),
                    SelectedEntry::Descriptor(_) => Err(CssomError::WrongKind),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let selected =
                selected_properties(self.staged.block(id)?, &properties, budget.provider()?)?;
            let CssomBlockData::Properties {
                selected: current, ..
            } = &mut self.staged.blocks.get_mut(id).expect("owned block").data
            else {
                return Err(CssomError::WrongKind);
            };
            *current = CssomProjection::Available(selected);
        }
        Ok(())
    }
    fn project_declarations(
        &self,
        id: &CssomBlockId,
        declarations: &[CssDeclaration],
        budget: &mut DeclarationBudget,
    ) -> Result<Vec<CssSpecifiedDeclarationEntry>, CssomError> {
        budget.scan(declarations.len())?;
        let block = self.staged.block(id)?;
        for declaration in declarations {
            budget.expansion(declaration)?;
        }
        let page = if matches!(
            block.data,
            CssomBlockData::Page { .. }
                | CssomBlockData::Properties {
                    domain: CssomPropertyDomain::Margin,
                    ..
                }
        ) {
            Some(
                CssMarginDeclarationBlock::try_new_with_limits(
                    declarations.to_vec(),
                    budget.provider()?,
                )
                .map_err(CssomError::PageBlock)?,
            )
        } else {
            None
        };
        let declarations = page
            .as_ref()
            .map_or(declarations, |page| page.properties().as_slice());
        let expansions = declarations
            .iter()
            .map(expand_declaration)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CssomError::Expansion)?;
        let selected = match block.data {
            CssomBlockData::Properties {
                domain: CssomPropertyDomain::Keyframe,
                ..
            } => CssSpecifiedDeclarationBlock::try_from_keyframe_expansions_with_limits(
                &expansions,
                budget.provider()?,
            ),
            _ => CssSpecifiedDeclarationBlock::try_from_expansions_with_limits(
                &expansions,
                budget.provider()?,
            ),
        }
        .map_err(CssomError::Declaration)?;
        selected_properties(block, selected.entries(), budget.provider()?)
            .map(|selected| selected.entries().to_vec())
    }
}
fn longhand(name: CssPropertyNameRef<'_>) -> Result<Option<CssLonghandProperty>, CssomError> {
    let CssPropertyNameRef::Known(property) = name else {
        return Ok(None);
    };
    match property.metadata().map_err(CssomError::Metadata)?.kind() {
        CssPropertyKindRef::Longhand(meta) => Ok(Some(meta.property())),
        _ => Ok(None),
    }
}

fn property_name(name: CssPropertyNameRef<'_>) -> &str {
    match name {
        CssPropertyNameRef::Known(property) => property.canonical_name(),
        CssPropertyNameRef::Custom(name) => name.as_str(),
        CssPropertyNameRef::SvgGlyphOrientationVertical => {
            CssSvgGlyphOrientationVerticalDeclaration::metadata().name()
        }
        _ => unreachable!("CSS property name"),
    }
}

fn entry_count(block: &CssomBlock) -> Result<usize, CssomError> {
    match &block.data {
        CssomBlockData::FontFace(_) => Ok(font_selected(block)?.entries().len()),
        CssomBlockData::Properties {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected.entries().len()),
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected.entries().len()),
        _ => properties(block).map(|items| items.len()),
    }
}

#[derive(Clone)]
enum SelectedEntry {
    Property(CssSpecifiedDeclarationEntry),
    Descriptor(CssPageDescriptor),
}
impl SelectedEntry {
    fn property(&self) -> Option<&CssSpecifiedDeclarationEntry> {
        match self {
            Self::Property(entry) => Some(entry),
            Self::Descriptor(_) => None,
        }
    }
}
fn selected_entries(
    block: &CssomBlock,
    budget: &mut DeclarationBudget,
) -> Result<Vec<SelectedEntry>, CssomError> {
    budget.scan(entry_count(block)?)?;
    match &block.data {
        CssomBlockData::Page {
            selected: CssomProjection::Available(selected),
            ..
        } => Ok(selected
            .entries()
            .iter()
            .map(|entry| match entry {
                CssSpecifiedPageDeclarationEntry::Property(entry) => {
                    SelectedEntry::Property(entry.clone())
                }
                CssSpecifiedPageDeclarationEntry::Descriptor(entry) => {
                    SelectedEntry::Descriptor(entry.clone())
                }
                _ => unreachable!("CSS Page entry"),
            })
            .collect()),
        _ => properties(block)
            .map(|entries| entries.into_iter().map(SelectedEntry::Property).collect()),
    }
}

fn page_rejection(error: &CssPageBlockError) -> bool {
    matches!(
        error.kind(),
        CssPageBlockErrorKind::PropertyNotApplicable | CssPageBlockErrorKind::InvalidPropertyValue
    )
}

fn normalize_in(name: &str, block: &CssomBlock) -> Option<Name> {
    let svg = CssSvgGlyphOrientationVerticalDeclaration::metadata();
    if name.eq_ignore_ascii_case(svg.name()) {
        let has_svg = match &block.data {
            CssomBlockData::Properties {
                selected: CssomProjection::Available(selected),
                ..
            } => selected
                .entries()
                .iter()
                .any(|entry| entry.property_name() == svg.property()),
            _ => false,
        };
        if has_svg {
            return Some(Name::SvgGlyph);
        }
    }
    if name.starts_with("--") {
        let find = |entry: &CssSpecifiedDeclarationEntry| match entry.property_name() {
            CssPropertyNameRef::Custom(checked) if checked.as_str() == name => {
                Some(Name::Custom(checked.clone()))
            }
            _ => None,
        };
        let existing = match &block.data {
            CssomBlockData::Properties {
                selected: CssomProjection::Available(selected),
                ..
            } => selected.entries().iter().find_map(find),
            CssomBlockData::Page {
                selected: CssomProjection::Available(selected),
                ..
            } => selected.entries().iter().find_map(|entry| match entry {
                CssSpecifiedPageDeclarationEntry::Property(entry) => find(entry),
                _ => None,
            }),
            _ => None,
        };
        existing.or_else(|| normalize(name))
    } else {
        normalize(name)
    }
}

fn check_input_storage(source: &str, maximum: usize) -> Result<(), CssomError> {
    if source.len() > maximum {
        Err(CssomError::Limit {
            resource: "retained parse input bytes",
            maximum,
        })
    } else {
        Ok(())
    }
}
