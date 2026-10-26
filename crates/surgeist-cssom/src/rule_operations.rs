//! Source rule algorithms over the current retained graph. CSS owns all parsing.
use crate::parser_resources::check_resource_diagnostics;
use crate::{model::State, *};
use surgeist_css::CssRule;
use surgeist_css::*;

/// A newly owned dictionary for one authored container entry. Reads retain
/// entry order/duplicates; modifying this transport cannot modify the rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomContainerCondition {
    pub name: String,
    pub query: String,
}
fn text_limits(max_bytes: usize) -> CssSerializationLimits {
    CssSerializationLimits::new(usize::MAX, usize::MAX, max_bytes)
}
fn container_name_text(
    entry: &CssContainerQueryEntry,
    max_bytes: usize,
) -> Result<String, CssomError> {
    entry
        .name()
        .map(|name| {
            serialize_css_identifier(name.as_str(), text_limits(max_bytes))
                .map_err(CssomError::Serialization)
        })
        .unwrap_or_else(|| Ok(String::new()))
}
fn container_query_text(
    entry: &CssContainerQueryEntry,
    max_bytes: usize,
) -> Result<String, CssomError> {
    entry
        .query()
        .map(|query| {
            query
                .serialize_tokens(crate::css_adapter::token_byte_limits(max_bytes))
                .map(|text| text.as_css().to_owned())
                .map_err(CssomError::Component)
        })
        .unwrap_or_else(|| Ok(String::new()))
}
impl CssomSnapshot {
    fn container_prelude(&self, id: &CssomRuleId) -> Result<&CssContainerPrelude, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(prelude),
                ..
            } => Ok(prelude),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Byte-bounded lexical fields, with one allowance for the aggregate emitted
    /// field bytes. Admitted checked CSS payload constraints remain independent.
    pub(crate) fn container_conditions(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Vec<CssomContainerCondition>, CssomError> {
        let mut remaining = max_bytes;
        let entries = self.container_prelude(id)?.entries();
        let mut result = Vec::new();
        result
            .try_reserve(entries.len())
            .map_err(|_| CssomError::Limit {
                resource: "container read capacity",
                maximum: entries.len(),
            })?;
        for entry in entries {
            let name = container_name_text(entry, remaining)?;
            remaining -= name.len();
            let query = container_query_text(entry, remaining)?;
            remaining -= query.len();
            result.push(CssomContainerCondition { name, query });
        }
        Ok(result)
    }
    /// A multi-entry rule has no singular name; this branch emits no text and
    /// never asks the query serializer to produce an unrelated field.
    pub(crate) fn container_name(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match self.container_prelude(id)?.entries() {
            [entry] => container_name_text(entry, max_bytes),
            _ => Ok(String::new()),
        }
    }
    pub(crate) fn container_query(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match self.container_prelude(id)?.entries() {
            [entry] => container_query_text(entry, max_bytes),
            _ => Ok(String::new()),
        }
    }
    /// The selected plural algorithm composes already serialized fields through
    /// CSS-owned whitespace/comma joiners. Omitted queries add no trailing space.
    pub(crate) fn container_condition_text(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        let conditions = self.container_conditions(id, max_bytes)?;
        let mut items = Vec::new();
        items
            .try_reserve(conditions.len())
            .map_err(|_| CssomError::Limit {
                resource: "container read capacity",
                maximum: conditions.len(),
            })?;
        for condition in &conditions {
            let fields: &[&str] = match (condition.name.is_empty(), condition.query.is_empty()) {
                (false, false) => &[&condition.name, &condition.query],
                (false, true) => &[&condition.name],
                (true, _) => &[&condition.query],
            };
            items.push(
                serialize_css_whitespace_separated_list(fields, text_limits(max_bytes))
                    .map_err(CssomError::Serialization)?,
            );
        }
        let items: Vec<&str> = items.iter().map(String::as_str).collect();
        serialize_css_comma_separated_list(&items, text_limits(max_bytes))
            .map_err(CssomError::Serialization)
    }
    /// Readonly decoded name under Surgeist's inert named-support grouping policy.
    pub(crate) fn named_support_condition_name(
        &self,
        id: &CssomRuleId,
    ) -> Result<&str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::NamedSupports { name, .. } => Ok(name.as_str()),
            _ => Err(CssomError::WrongKind),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RuleDestination {
    Sheet(CssomSheetId),
    Group(CssomRuleId),
}

pub(crate) fn copy_rule_diagnostics(
    source: &[CssRecoveryDiagnostic],
) -> Result<Vec<CssRecoveryDiagnostic>, CssomError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(source.len())
        .map_err(|_| CssomError::Limit {
            resource: "object rule diagnostics capacity",
            maximum: source.len(),
        })?;
    result.extend_from_slice(source);
    Ok(result)
}

fn syntax() -> CssomError {
    CssomError::Source(CssomException::Syntax)
}
fn hierarchy() -> CssomError {
    CssomError::Source(CssomException::HierarchyRequest)
}

pub(crate) fn component_limits(limits: &CssomLimits) -> CssComponentValueLimits {
    CssComponentValueLimits::try_new(
        limits.max_depth.min(256) as u32,
        limits.max_entries,
        limits.max_input_bytes,
    )
    .expect("depth bounded by CSS provider maximum")
}
fn preflight_components(source: &str, limits: &CssomLimits) -> Result<(), CssomError> {
    match parse_component_values_with_limits(source, component_limits(limits)) {
        Ok(_) => Ok(()),
        Err(error) if error.is_resource_limit() => Err(CssomError::Component(error)),
        // Feature grammar owns syntax rejection and inner declaration recovery;
        // a whole-input component error is not itself its retention decision.
        Err(_) => Ok(()),
    }
}
impl State {
    fn rule_namespaces(&self, id: &CssomRuleId) -> Result<CssNamespaceContext, CssomError> {
        Ok(match self.parent_style_sheet(id)? {
            Some(sheet) => (*self.sheet(sheet)?.namespaces).clone(),
            None => CssNamespaceContext::default(),
        })
    }
    fn style_ancestor(&self, id: &CssomRuleId) -> Result<CssStyleAncestor, CssomError> {
        let mut parent = &self.rule(id)?.parent;
        while let CssomParent::Rule(id) = parent {
            let rule = self.rule(id)?;
            if matches!(rule.data(), CssomRuleData::Style { .. }) {
                return Ok(CssStyleAncestor::Present);
            }
            parent = &rule.parent;
        }
        Ok(CssStyleAncestor::Absent)
    }
    fn selector_context(&self, id: &CssomRuleId) -> Result<CssStyleSelectorContext, CssomError> {
        let mut parent = &self.rule(id)?.parent;
        while let CssomParent::Rule(id) = parent {
            let rule = self.rule(id)?;
            match rule.data() {
                CssomRuleData::Style { .. } => return Ok(CssStyleSelectorContext::Nested),
                CssomRuleData::Group {
                    prelude: CssomGroupPrelude::Scope { .. },
                    ..
                } => return Ok(CssStyleSelectorContext::Scoped(self.style_ancestor(id)?)),
                _ => parent = &rule.parent,
            }
        }
        Ok(CssStyleSelectorContext::Ordinary)
    }
    fn admission_context(&self, id: &CssomRuleId) -> Result<CssRuleAdmissionContext, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Style { .. } => Ok(CssRuleAdmissionContext::Style),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Scope { .. },
                ..
            } => Ok(CssRuleAdmissionContext::Scope(self.style_ancestor(id)?)),
            _ => Ok(match self.selector_context(id)? {
                CssStyleSelectorContext::Ordinary => CssRuleAdmissionContext::Group,
                CssStyleSelectorContext::Nested => CssRuleAdmissionContext::Style,
                CssStyleSelectorContext::Scoped(ancestor) => {
                    CssRuleAdmissionContext::ScopedGroup(ancestor)
                }
            }),
        }
    }
    fn insertion_depth(&self, parent: &CssomParent) -> Result<usize, CssomError> {
        let mut depth = 1usize;
        let mut parent = parent;
        while let CssomParent::Rule(id) = parent {
            depth = depth.checked_add(1).ok_or(CssomError::Limit {
                resource: "rule depth",
                maximum: usize::MAX,
            })?;
            parent = &self.rule(id)?.parent;
        }
        Ok(depth)
    }
}

impl CssomBatch {
    /// CSSStyleSheet insertion's preliminary checks precede the shared index guard.
    pub fn insert_sheet_rule(
        &mut self,
        id: &CssomSheetId,
        source: &str,
        index: usize,
    ) -> Result<(usize, CssomRuleTicket), CssomError> {
        self.apply(|this| {
            let sheet = this.staged.editable_sheet_rules(id)?;
            let constructed = sheet.inputs.constructed;
            let (candidate, diagnostics) =
                classify_rule_syntax_with_limits(source, component_limits(&this.limits))
                    .into_parts();
            check_resource_diagnostics(&diagnostics)?;
            let candidate = candidate.ok_or_else(syntax)?;
            if constructed
                && candidate
                    .at_rule_name()
                    .is_some_and(|name| name.eq_ignore_ascii_case("import"))
            {
                return Err(syntax());
            }
            this.insert_prepared_rule(
                RuleDestination::Sheet(id.clone()),
                source,
                Some(&candidate),
                index,
                false,
            )
        })
    }
    /// CSSGroupingRule insertion checks its index before classifying the input.
    pub fn insert_group_rule(
        &mut self,
        id: &CssomRuleId,
        source: &str,
        index: usize,
    ) -> Result<CssomRuleTicket, CssomError> {
        self.apply(|this| {
            if matches!(
                this.staged.rule(id)?.data(),
                CssomRuleData::NamedSupports { .. }
            ) {
                return this.insert_named_supports_child(id, source, index);
            }
            let list = match this.staged.rule(id)?.data() {
                CssomRuleData::Style { children, .. }
                | CssomRuleData::Group { children, .. }
                | CssomRuleData::Page { children, .. } => children,
                _ => return Err(CssomError::WrongKind),
            };
            if index > this.staged.lists[list].len() {
                return Err(CssomError::Source(CssomException::IndexSize));
            }
            let (candidate, diagnostics) =
                classify_rule_syntax_with_limits(source, component_limits(&this.limits))
                    .into_parts();
            check_resource_diagnostics(&diagnostics)?;
            this.insert_prepared_rule(
                RuleDestination::Group(id.clone()),
                source,
                candidate.as_ref(),
                index,
                true,
            )
            .map(|(_, ticket)| ticket)
        })
    }
    /// Called within the source wrapper's single atomic `apply` operation. A supplied
    /// candidate is the very occurrence classified by that wrapper, never reparsed.
    pub(crate) fn insert_prepared_rule(
        &mut self,
        destination: RuleDestination,
        source: &str,
        candidate: Option<&CssRuleSyntax>,
        index: usize,
        nested: bool,
    ) -> Result<(usize, CssomRuleTicket), CssomError> {
        let (list, parent, namespaces, context) = match &destination {
            RuleDestination::Sheet(id) => {
                let sheet = self.staged.sheet(id)?;
                (
                    sheet.rules.clone(),
                    CssomParent::Sheet(id.clone()),
                    (*sheet.namespaces).clone(),
                    CssRuleAdmissionContext::Stylesheet,
                )
            }
            RuleDestination::Group(id) => {
                let rule = self.staged.rule(id)?;
                let list = match rule.data() {
                    CssomRuleData::Style { children, .. }
                    | CssomRuleData::Group { children, .. }
                    | CssomRuleData::Page { children, .. } => children.clone(),
                    _ => return Err(CssomError::WrongKind),
                };
                (
                    list,
                    CssomParent::Rule(id.clone()),
                    self.staged.rule_namespaces(id)?,
                    self.staged.admission_context(id)?,
                )
            }
        };
        let members = &self.staged.lists[&list];
        if index > members.len() {
            return Err(CssomError::Source(CssomException::IndexSize));
        }
        let parser = self.staged.context.parser_context();
        let mut admitted = None;
        let mut retained_diagnostics = Vec::new();
        if let Some(candidate) = candidate {
            // Establish valid top-level-only grammar before the destination hierarchy guard.
            let top_only = candidate.at_rule_name().is_some_and(|name| {
                name.eq_ignore_ascii_case("import") || name.eq_ignore_ascii_case("namespace")
            });
            let page = matches!(&destination, RuleDestination::Group(id)
                if matches!(self.staged.rules[id].data(), CssomRuleData::Page { .. }));
            if page && !top_only {
                let (value, diagnostics) = candidate
                    .admit_page_margin_rule_with_context(parser)
                    .into_parts();
                check_resource_diagnostics(&diagnostics)?;
                retained_diagnostics = diagnostics;
                admitted = value.map(CssomAuthoredRule::Margin);
                // A valid non-margin rule still reaches the hierarchy check.
                if admitted.is_none() {
                    let (value, diagnostics) = candidate
                        .admit_with_context(
                            &namespaces,
                            CssRuleAdmissionContext::Stylesheet,
                            parser,
                        )
                        .into_parts();
                    check_resource_diagnostics(&diagnostics)?;
                    retained_diagnostics = diagnostics;
                    admitted = value.map(CssomAuthoredRule::Rule);
                }
            } else {
                let (value, diagnostics) = candidate
                    .admit_with_context(
                        &namespaces,
                        if top_only {
                            CssRuleAdmissionContext::Stylesheet
                        } else {
                            context
                        },
                        parser,
                    )
                    .into_parts();
                check_resource_diagnostics(&diagnostics)?;
                retained_diagnostics = diagnostics;
                admitted = value.map(CssomAuthoredRule::Rule);
            }
        }
        let authored = match admitted {
            Some(rule) => rule,
            None if nested => {
                let (declarations, diagnostics) = parser
                    .parse_declaration_block_contents_with_limits(
                        source,
                        component_limits(&self.limits),
                    )
                    .into_parts();
                check_resource_diagnostics(&diagnostics)?;
                if declarations.is_empty() {
                    return Err(syntax());
                }
                retained_diagnostics = diagnostics;
                CssomAuthoredRule::RawNestedDeclarations(declarations)
            }
            None => return Err(syntax()),
        };
        let incoming = phase(&authored);
        if matches!(destination, RuleDestination::Group(_)) && incoming < 2 {
            return Err(hierarchy());
        }
        if let RuleDestination::Group(id) = &destination
            && matches!(self.staged.rule(id)?.data(), CssomRuleData::Page { .. })
            && !matches!(authored, CssomAuthoredRule::Margin(_))
        {
            return Err(hierarchy());
        }
        if matches!(destination, RuleDestination::Sheet(_)) {
            // Named supports definitions preserve the current prefix phase.
            // Layer statements preserve only the initial phase.
            let mut seen_prefix = false;
            let mut last = 0;
            for (position, current) in members.iter().enumerate() {
                if position == index {
                    ordered_phase(&authored, &mut seen_prefix, &mut last)?;
                }
                ordered_phase(
                    self.staged.rules[current].authored(),
                    &mut seen_prefix,
                    &mut last,
                )?;
            }
            if index == members.len() {
                ordered_phase(&authored, &mut seen_prefix, &mut last)?;
            }
            if incoming == 1
                && members
                    .iter()
                    .any(|id| phase(self.staged.rules[id].authored()) > 1)
            {
                return Err(CssomError::Source(CssomException::InvalidState));
            }
        }
        let depth = self.staged.insertion_depth(&parent)?;
        let id = self
            .staged
            .adopt(authored, parent.clone(), depth, &self.limits)?;
        self.staged
            .rules
            .get_mut(&id)
            .expect("adopted rule")
            .admission_diagnostics = retained_diagnostics.into();
        self.staged
            .lists
            .get_mut(&list)
            .expect("owned list")
            .insert(index, id.clone());
        if let CssomParent::Sheet(sheet) = parent {
            self.refresh_namespaces(&sheet);
        }
        self.affect(
            CssomObjectId::RuleList(list),
            &[CssomChange::RuleMembership, CssomChange::Order],
        );
        self.affect(
            CssomObjectId::Rule(id.clone()),
            &[
                CssomChange::Ancestry,
                CssomChange::Namespaces,
                CssomChange::Provenance,
            ],
        );
        let ticket = self.retain_rule_ticket(id);
        Ok((index, ticket))
    }
}
fn phase(rule: &CssomAuthoredRule) -> u8 {
    match rule {
        CssomAuthoredRule::Rule(CssRule::Import(_)) => 0,
        CssomAuthoredRule::Rule(CssRule::Namespace(_)) => 1,
        _ => 2,
    }
}
fn ordered_phase(
    rule: &CssomAuthoredRule,
    seen_prefix: &mut bool,
    last: &mut u8,
) -> Result<(), CssomError> {
    // Conditional5 §8 permits these before imports/namespaces, including
    // interleaved prefix members. Preserve an existing namespace/body phase.
    if matches!(rule, CssomAuthoredRule::Rule(CssRule::SupportsCondition(_))) {
        return Ok(());
    }
    if !*seen_prefix && matches!(rule, CssomAuthoredRule::Rule(CssRule::LayerStatement(_))) {
        return Ok(());
    }
    let value = phase(rule);
    if value < *last {
        return Err(hierarchy());
    }
    *last = value;
    if value < 2 {
        *seen_prefix = true;
    }
    Ok(())
}

impl CssomSnapshot {
    /// Length in this immutable capture; use Store for live observations.
    pub(crate) fn rule_list_length(&self, list: &CssomRuleListId) -> Result<usize, CssomError> {
        Ok(self.rules(list)?.len())
    }
    pub(crate) fn rule_list_item(
        &self,
        list: &CssomRuleListId,
        index: usize,
    ) -> Result<Option<&CssomRuleId>, CssomError> {
        Ok(self.rules(list)?.get(index))
    }
    /// Deprecated CSSRule type values are determined from concrete typed kinds.
    pub(crate) fn rule_type(&self, id: &CssomRuleId) -> Result<u16, CssomError> {
        Ok(match self.rule(id)?.data() {
            CssomRuleData::Style { .. } => 1,
            CssomRuleData::Import { .. } => 3,
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(_),
                ..
            } => 4,
            CssomRuleData::FontFace { .. } => 5,
            CssomRuleData::Page { .. } => 6,
            CssomRuleData::Keyframes { .. } => 7,
            CssomRuleData::Keyframe { .. } => 8,
            CssomRuleData::Margin { .. } => 9,
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(CssRule::Namespace(_)),
            } => 10,
            CssomRuleData::CounterStyle { .. } => 11,
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            } => 12,
            CssomRuleData::FontFeatureValues { .. } => 14,
            _ => 0,
        })
    }
    pub(crate) fn style_selector_text(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        let namespaces = self.state.rule_namespaces(id)?;
        let context = self.state.selector_context(id)?;
        match self.rule(id)?.data() {
            CssomRuleData::Style { selectors, .. } => {
                selectors.serialize_cssom(&namespaces, context, limits)
            }
            _ => return Err(CssomError::WrongKind),
        }
        .map_err(CssomError::Serialization)
    }
    pub(crate) fn page_selector_text(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Page { selectors, .. } => selectors
                .serialize_cssom(limits)
                .map_err(CssomError::Serialization),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn keyframe_key_text(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Keyframe { selectors, .. } => selectors
                .serialize_key_text(limits)
                .map_err(CssomError::Serialization),
            _ => Err(CssomError::WrongKind),
        }
    }
}
impl CssomBatch {
    /// Invalid selector syntax is a source no-op; retained recovery is successful.
    pub fn set_style_selector_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<bool, CssomError> {
        self.set_style_selector_text_with_diagnostics(id, source)
            .map(|(value, _)| value)
    }
    pub(crate) fn set_style_selector_text_with_diagnostics(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(bool, Vec<CssRecoveryDiagnostic>), CssomError> {
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data(), CssomRuleData::Style { .. }) {
                return Err(CssomError::WrongKind);
            }
            let namespaces = this.staged.rule_namespaces(id)?;
            let context = this.staged.selector_context(id)?;
            let (parsed, diagnostics) = parse_style_selector_list_with_limits(
                source,
                &namespaces,
                context,
                component_limits(&this.limits),
            )
            .into_parts();
            check_resource_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok((false, diagnostics));
            };
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Style { selectors, .. } = rule.data_mut() else {
                unreachable!()
            };
            *selectors = parsed.selectors().clone();
            rule.admission_diagnostics = copy_rule_diagnostics(&diagnostics)?.into();
            rule.selector_input = Some(parsed.origin().clone());
            rule.selector_inputs = Some(inputs);
            rule.selector_namespaces = Some(namespaces.into());
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Selectors, CssomChange::Provenance],
            );
            Ok((true, diagnostics))
        })
    }
    /// Page selector admission uses its independent complete-list grammar.
    pub fn set_page_selector_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<bool, CssomError> {
        self.set_page_selector_text_with_diagnostics(id, source)
            .map(|(value, _)| value)
    }
    pub(crate) fn set_page_selector_text_with_diagnostics(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(bool, Vec<CssRecoveryDiagnostic>), CssomError> {
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data(), CssomRuleData::Page { .. }) {
                return Err(CssomError::WrongKind);
            }
            // The component owner preflights the whole raw input under product quotas.
            preflight_components(source, &this.limits)?;
            let (parsed, diagnostics) = parse_page_selector_list(source).into_parts();
            check_resource_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok((false, diagnostics));
            };
            let origin = parsed.origin().cloned();
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Page { selectors, .. } = rule.data_mut() else {
                unreachable!()
            };
            *selectors = parsed;
            rule.admission_diagnostics = copy_rule_diagnostics(&diagnostics)?.into();
            rule.selector_input = origin;
            rule.selector_inputs = Some(inputs);
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Selectors, CssomChange::Provenance],
            );
            Ok((true, diagnostics))
        })
    }
    pub fn set_keyframe_key_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(), CssomError> {
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data(), CssomRuleData::Keyframe { .. }) {
                return Err(CssomError::WrongKind);
            }
            preflight_components(source, &this.limits)?;
            let (parsed, diagnostics) = parse_keyframe_selector_list(source).into_parts();
            check_resource_diagnostics(&diagnostics)?;
            let parsed = parsed.ok_or_else(syntax)?;
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Keyframe { selectors, .. } = rule.data_mut() else {
                unreachable!()
            };
            *selectors = parsed.selectors().clone();
            rule.admission_diagnostics = diagnostics.into();
            rule.selector_input = Some(parsed.origin().clone());
            rule.selector_inputs = Some(inputs);
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Selectors, CssomChange::Provenance],
            );
            Ok(())
        })
    }
}

/// The indexed Keyframes getter has undefined absence, distinct from item(null).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomKeyframeIndex<T> {
    Rule(T),
    Undefined,
}
impl CssomSnapshot {
    pub(crate) fn keyframes_name(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Keyframes { name, .. } => Ok(match name {
                CssKeyframesName::Ident(value) => value.as_str(),
                CssKeyframesName::String(value) => value.as_str(),
                _ => return Err(CssomError::WrongKind),
            }),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn keyframes_length(&self, id: &CssomRuleId) -> Result<usize, CssomError> {
        let CssomRuleData::Keyframes { children, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        self.rule_list_length(children)
    }
    pub(crate) fn keyframes_index(
        &self,
        id: &CssomRuleId,
        index: usize,
    ) -> Result<CssomKeyframeIndex<CssomRuleId>, CssomError> {
        let CssomRuleData::Keyframes { children, .. } = self.rule(id)?.data() else {
            return Err(CssomError::WrongKind);
        };
        Ok(match self.rule_list_item(children, index)? {
            Some(id) => CssomKeyframeIndex::Rule(id.clone()),
            None => CssomKeyframeIndex::Undefined,
        })
    }
}
impl CssomBatch {
    /// Retains the decoded CSSOMString; authored-name token grammar is independent.
    pub fn set_keyframes_name(
        &mut self,
        id: &CssomRuleId,
        value: &str,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            let CssomRuleData::Keyframes { name, .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let old = match name {
                CssKeyframesName::Ident(value) => value.as_str(),
                CssKeyframesName::String(value) => value.as_str(),
                _ => return Err(CssomError::WrongKind),
            };
            if old == value {
                return Ok(false);
            }
            let CssomRuleData::Keyframes { name, .. } =
                this.staged.rules.get_mut(id).expect("rule").data_mut()
            else {
                unreachable!()
            };
            *name = CssKeyframesName::String(CssKeyframesString::new(value));
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Modification],
            );
            Ok(true)
        })
    }
    /// Invalid raw keyframe syntax appends nothing. Successful duplicate keys
    /// allocate distinct identities; source declaration recovery remains retained.
    pub fn append_keyframe_rule(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<Option<CssomRuleTicket>, CssomError> {
        self.append_keyframe_rule_with_diagnostics(id, source)
            .map(|(value, _)| value)
    }
    pub(crate) fn append_keyframe_rule_with_diagnostics(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(Option<CssomRuleTicket>, Vec<CssRecoveryDiagnostic>), CssomError> {
        self.apply(|this| {
            let CssomRuleData::Keyframes { children, .. } = this.staged.rule(id)?.data() else {
                return Err(CssomError::WrongKind);
            };
            let list = children.clone();
            preflight_components(source, &this.limits)?;
            let (parsed, diagnostics) = this
                .staged
                .context
                .parser_context()
                .parse_keyframe_rule(source)
                .into_parts();
            check_resource_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok((None, diagnostics));
            };
            let parent = CssomParent::Rule(id.clone());
            let depth = this.staged.insertion_depth(&parent)?;
            let child = this.staged.adopt(
                CssomAuthoredRule::Keyframe(parsed.block().clone()),
                parent,
                depth,
                &this.limits,
            )?;
            this.staged
                .rules
                .get_mut(&child)
                .expect("adopted keyframe")
                .admission_diagnostics = copy_rule_diagnostics(&diagnostics)?.into();
            this.staged
                .lists
                .get_mut(&list)
                .expect("owned list")
                .push(child.clone());
            this.affect(
                CssomObjectId::RuleList(list),
                &[CssomChange::RuleMembership, CssomChange::Order],
            );
            this.affect(
                CssomObjectId::Rule(child.clone()),
                &[CssomChange::Ancestry, CssomChange::Provenance],
            );
            Ok((Some(this.retain_rule_ticket(child)), diagnostics))
        })
    }
}

impl CssomSnapshot {
    /// Authored location only; resolved location is a separately supplied import fact.
    pub(crate) fn import_href(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Import { target, .. } => Ok(match target {
                CssImportTarget::String(value) => value.as_str(),
                CssImportTarget::Url(value) => value.as_str(),
                _ => return Err(CssomError::WrongKind),
            }),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn import_media(&self, id: &CssomRuleId) -> Result<&CssomMediaListId, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Import { media, .. } => Ok(media),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn import_style_sheet(
        &self,
        id: &CssomRuleId,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Import { child_sheet, .. } => Ok(child_sheet.as_ref()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn import_supports_text(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Option<String>, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Import { supports, .. } => supports
                .as_ref()
                .map(|supports| {
                    supports
                        .condition()
                        .serialize_tokens(crate::css_adapter::token_byte_limits(max_bytes))
                        .map(|value| value.as_css().to_owned())
                        .map_err(CssomError::Component)
                })
                .transpose(),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn namespace_uri(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(CssRule::Namespace(rule)),
            } => Ok(rule.name().as_str()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn namespace_prefix(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(CssRule::Namespace(rule)),
            } => Ok(rule.prefix().map_or("", CssNamespacePrefix::as_str)),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn margin_name(&self, id: &CssomRuleId) -> Result<&'static str, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Margin { name, .. } => Ok(name.as_str()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn group_media(&self, id: &CssomRuleId) -> Result<&CssomMediaListId, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(media),
                ..
            } => Ok(media),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub(crate) fn supports_condition_text(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(condition),
                ..
            } => Ok(condition
                .serialize_tokens(crate::css_adapter::token_byte_limits(max_bytes))?
                .as_css()
                .to_owned()),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Omitted Scope bounds remain null; reads never require a whole-rule format.
    pub(crate) fn scope_start(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        let CssomRuleData::Group {
            prelude: CssomGroupPrelude::Scope { start, .. },
            ..
        } = self.rule(id)?.data()
        else {
            return Err(CssomError::WrongKind);
        };
        let namespaces = self.state.rule_namespaces(id)?;
        let nesting = if self.state.style_ancestor(id)? == CssStyleAncestor::Present {
            CssScopeNestingContext::Style
        } else {
            match self.state.selector_context(id)? {
                CssStyleSelectorContext::Scoped(_) => CssScopeNestingContext::Scope,
                _ => CssScopeNestingContext::None,
            }
        };
        start
            .as_ref()
            .map(|value| {
                value
                    .serialize_cssom(
                        &namespaces,
                        CssScopeSelectorCssomContext::Start(nesting),
                        limits,
                    )
                    .map_err(CssomError::Serialization)
            })
            .transpose()
    }
    pub(crate) fn scope_end(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        let CssomRuleData::Group {
            prelude: CssomGroupPrelude::Scope { end, .. },
            ..
        } = self.rule(id)?.data()
        else {
            return Err(CssomError::WrongKind);
        };
        let namespaces = self.state.rule_namespaces(id)?;
        end.as_ref()
            .map(|value| {
                value
                    .serialize_cssom(&namespaces, CssScopeSelectorCssomContext::End, limits)
                    .map_err(CssomError::Serialization)
            })
            .transpose()
    }
}
impl CssomSnapshot {
    /// The same collection handle is retained through insertion, deletion and detach.
    /// Keyframes uses its own operations even though it also has a rule list.
    pub(crate) fn child_rules(&self, id: &CssomRuleId) -> Result<&CssomRuleListId, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Style { children, .. }
            | CssomRuleData::Group { children, .. }
            | CssomRuleData::Page { children, .. }
            | CssomRuleData::Keyframes { children, .. }
            | CssomRuleData::NamedSupports { children, .. } => Ok(children),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Associated declaration identity; validity domain and special flags remain
    /// in that owning block rather than being converted to ordinary properties.
    pub(crate) fn rule_style(&self, id: &CssomRuleId) -> Result<&CssomBlockId, CssomError> {
        self.rule(id)?.data.block().ok_or(CssomError::WrongKind)
    }
    pub(crate) fn condition_text(
        &self,
        id: &CssomRuleId,
        limits: CssSerializationLimits,
    ) -> Result<String, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(media),
                ..
            } => self
                .media(media)?
                .serialize_cssom(limits)
                .map(|value| value.as_css().to_owned())
                .map_err(CssomError::Media),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            } => self.supports_condition_text(id, limits.max_css_bytes()),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(_),
                ..
            } => self.container_condition_text(id, limits.max_css_bytes()),
            _ => Err(CssomError::WrongKind),
        }
    }
}

impl CssomSnapshot {
    /// Declared local name only: absent is null, anonymous is an empty string.
    pub(crate) fn import_layer_name(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Option<String>, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Import { layer, .. } => match layer {
                None => Ok(None),
                Some(CssImportLayer::Anonymous) => Ok(Some(String::new())),
                Some(CssImportLayer::Named(name)) => name
                    .serialize_specified(text_limits(max_bytes))
                    .map(Some)
                    .map_err(CssomError::Serialization),
                _ => Err(CssomError::WrongKind),
            },
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Rule-local checked name, without ancestor layer prefixes.
    pub(crate) fn layer_block_name(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match self.rule(id)?.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Layer(name),
                ..
            } => name
                .as_ref()
                .map(|name| {
                    name.serialize_specified(text_limits(max_bytes))
                        .map_err(CssomError::Serialization)
                })
                .unwrap_or_else(|| Ok(String::new())),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// A newly owned array of local names, retaining order and duplicates. The
    /// byte allowance is shared across emitted names; no ancestor qualification.
    pub(crate) fn layer_statement_names(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Vec<String>, CssomError> {
        let names = match self.rule(id)?.data() {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(CssRule::LayerStatement(rule)),
            } => rule.names(),
            _ => return Err(CssomError::WrongKind),
        };
        let mut result = Vec::new();
        result
            .try_reserve(names.names().len())
            .map_err(|_| CssomError::Limit {
                resource: "layer name read capacity",
                maximum: names.names().len(),
            })?;
        let mut remaining = max_bytes;
        for name in names.names() {
            let text = name
                .serialize_specified(text_limits(remaining))
                .map_err(CssomError::Serialization)?;
            remaining -= text.len();
            result.push(text);
        }
        Ok(result)
    }
}

impl State {
    fn last_keyframe_index(
        &self,
        id: &CssomRuleId,
        source: &str,
        limits: &CssomLimits,
    ) -> Result<Option<usize>, CssomError> {
        let list = match self.rule(id)?.data() {
            CssomRuleData::Keyframes { children, .. } => children,
            _ => return Err(CssomError::WrongKind),
        };
        preflight_components(source, limits)?;
        let (query, diagnostics) = parse_keyframe_selector_list(source).into_parts();
        check_resource_diagnostics(&diagnostics)?;
        let Some(query) = query else {
            return Ok(None);
        };
        // One query and one monotonic native normalization/comparison budget for
        // the complete reverse-last search. No authored equality or text keys.
        let mut matcher = CssKeyframeSelectorMatcher::try_new(query.selectors(), limits.css)?;
        let children = self.lists.get(list).expect("owned Keyframes list");
        for (visited, (index, child)) in children.iter().enumerate().rev().enumerate() {
            if visited >= limits.max_entries {
                return Err(CssomError::Limit {
                    resource: "keyframe search candidates",
                    maximum: limits.max_entries,
                });
            }
            let CssomRuleData::Keyframe { selectors, .. } = self.rule(child)?.data() else {
                return Err(CssomError::WrongKind);
            };
            if matcher.matches(selectors)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}
impl CssomSnapshot {
    /// Matches the complete ordered normalized selector sequence and returns the
    /// last live member, or null on unmatched/invalid source. Captures stay stable.
    pub fn find_keyframe_rule(
        &self,
        id: &CssomRuleId,
        source: &str,
        limits: &CssomLimits,
    ) -> Result<Option<&CssomRuleId>, CssomError> {
        let index = self.state.last_keyframe_index(id, source, limits)?;
        let list = self.rule(id)?.data.children().expect("checked Keyframes");
        let members = self.rules(list)?;
        Ok(index.map(|index| &members[index]))
    }
}
impl CssomBatch {
    /// Deletes only the last normalized full-sequence match and detaches that
    /// root. Invalid/unmatched input is a no-op; typed resource failure is atomic.
    pub fn delete_keyframe_rule(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<Option<CssomRuleId>, CssomError> {
        self.apply(|this| {
            let Some(index) = this.staged.last_keyframe_index(id, source, &this.limits)? else {
                return Ok(None);
            };
            let list = this
                .staged
                .rule(id)?
                .data
                .children()
                .expect("checked Keyframes")
                .clone();
            this.delete(&list, index).map(Some)
        })
    }
}
