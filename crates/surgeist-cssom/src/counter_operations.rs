//! CounterStyle partial reads and checked literal descriptor transitions.
use crate::{declaration::*, declaration_support::DeclarationBudget, *};
use surgeist_css::*;

impl CssomBlock {
    fn counter_entry_count(&self) -> Result<usize, CssomError> {
        let CssomBlockData::CounterStyle(original) = &self.data else {
            return Err(CssomError::WrongKind);
        };
        Ok(original
            .entries()
            .filter(|entry| {
                !self
                    .counter_edits
                    .iter()
                    .any(|v| v.kind() == entry.value().kind())
            })
            .count()
            + self.counter_edits.len())
    }
    /// Checked current descriptor collection, preserving original named occurrences
    /// and independently edited raw value origins. It performs no counter execution.
    pub fn counter_descriptors(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<CssCounterStyleDescriptorCollection<'_>, CssomError> {
        let CssomBlockData::CounterStyle(original) = &self.data else {
            return Err(CssomError::WrongKind);
        };
        let entries = original
            .entries()
            .filter(|entry| {
                !self
                    .counter_edits
                    .iter()
                    .any(|v| v.kind() == entry.value().kind())
            })
            .chain(
                self.counter_edits
                    .iter()
                    .map(CssCounterStyleDescriptorEntry::from_value),
            )
            .collect();
        CssCounterStyleDescriptorCollection::try_new_with_limits(entries, limits)
            .map_err(CssomError::Value)
    }
}

impl CssomSnapshot {
    /// Serializes the live decoded name as an identifier, without authored-name validation.
    pub fn counter_name(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        let CssomRuleData::CounterStyle { name, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        serialize_css_identifier_with_limits(name, limits).map_err(CssomError::Value)
    }
    /// Specified effective descriptor text; an unspecified descriptor is empty.
    /// This remains independent of whole-rule output and definition validity.
    pub fn counter_descriptor(
        &self,
        id: &CssomRuleId,
        kind: CssCounterStyleDescriptorKind,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        let CssomRuleData::CounterStyle { block, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        let mut budget = DeclarationBudget::new(
            CssomDeclarationRequestLimits {
                provider: limits,
                ..Default::default()
            },
            2,
        );
        let current = self.block(block)?.counter_descriptors(budget.provider()?)?;
        current.effective(kind).map_or_else(
            || Ok(String::new()),
            |entry| {
                entry
                    .value()
                    .to_specified_css_with_limits(budget.provider()?)
                    .map_err(CssomError::Value)
            },
        )
    }
}

impl CssomBatch {
    /// Parses a complete raw descriptor and checks the prospective definition.
    /// Grammar/undefined/algorithm-changing edits are source no-ops; unresolved
    /// definition admission and resources remain distinct typed errors.
    pub fn set_counter_descriptor(
        &mut self,
        id: &CssomRuleId,
        kind: CssCounterStyleDescriptorKind,
        source: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let CssomRuleData::CounterStyle { block, .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let block = block.clone();
            let mut budget = DeclarationBudget::new(limits, 2);
            if source.len() > this.limits.max_input_bytes {
                return Err(CssomError::Limit {
                    resource: "parse input bytes",
                    maximum: this.limits.max_input_bytes,
                });
            }
            if !complete_value_components(source, budget.input(source)?)? {
                return Ok(false);
            }
            let (value, diagnostics) =
                parse_counter_style_descriptor_value(source, kind).into_parts();
            resource_diagnostics(&diagnostics)?;
            let Some(value) = value else {
                return Ok(false);
            };
            let current = this.staged.block(&block)?;
            budget.scan(current.counter_entry_count()?)?;
            let old = current.counter_descriptors(budget.provider()?)?;
            let mut entries = old
                .entries()
                .iter()
                .copied()
                .filter(|entry| entry.value().kind() != kind)
                .collect::<Vec<_>>();
            budget.scan(entries.len() + 1)?;
            entries.push(CssCounterStyleDescriptorEntry::from_value(&value));
            let prospective = CssCounterStyleDescriptorCollection::try_new_with_limits(
                entries,
                budget.provider()?,
            )
            .map_err(CssomError::Value)?;
            match prospective.definition_status() {
                CssCounterStyleDefinitionStatus::Defined => {}
                CssCounterStyleDefinitionStatus::Undefined(_) => return Ok(false),
                status => return Err(CssomError::CounterStylePreparationUnavailable(status)),
            }
            if kind == CssCounterStyleDescriptorKind::System {
                let Some(old_algorithm) = old.algorithm() else {
                    return Err(CssomError::CounterStylePreparationUnavailable(
                        CssCounterStyleDefinitionStatus::SubstitutionDependent,
                    ));
                };
                if Some(old_algorithm) != prospective.algorithm() {
                    return Ok(false);
                }
            }
            let current = this.staged.blocks.get_mut(&block).expect("owned block");
            current.counter_edits.retain(|entry| entry.kind() != kind);
            current.counter_edits.push(value);
            if !this.counter_changes.contains(&block) {
                this.counter_changes.push(block.clone());
            }
            this.affect(
                CssomObjectId::Block(block),
                &[CssomChange::Descriptors, CssomChange::Provenance],
            );
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Descriptors, CssomChange::Provenance],
            );
            Ok(true)
        })
    }
}
