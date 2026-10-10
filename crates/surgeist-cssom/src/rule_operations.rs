//! Source rule algorithms over the current retained graph. CSS owns all parsing.
use crate::{model::State, *};
use surgeist_css::*;

/// The independently revised facts used by a supplied evaluator result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomQueryFacts {
    Media {
        document: CssomInputVersion,
        window: CssomInputVersion,
    },
    Supports {
        support: CssomInputVersion,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomQueryResultRole {
    Media,
    Supports,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssomQueryMatchError {
    MissingResult,
    ForeignOwner,
    WrongRule,
    WrongRole,
    StaleRevision,
    StaleContext,
    MissingFact(CssomInputRole),
    InvalidFactRole {
        expected: CssomInputRole,
        actual: CssomInputRole,
    },
    FactBindingMismatch(CssomInputRole),
}
impl std::fmt::Display for CssomQueryMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query result correlation failed: {self:?}")
    }
}
impl std::error::Error for CssomQueryMatchError {}
/// An immutable supplied result bound to one actual owning rule capture. CSSOM
/// validates correlation and returns the value; it never evaluates the query.
#[derive(Clone, Debug)]
pub struct CssomQueryResult {
    capture: CssomSnapshot,
    rule: CssomRuleId,
    role: CssomQueryResultRole,
    facts: CssomQueryFacts,
    matched: bool,
}
impl CssomQueryResult {
    pub fn owner(&self) -> &CssomOwnerId {
        self.capture.owner()
    }
    pub fn revision(&self) -> &CssomRevision {
        self.capture.revision()
    }
    pub fn rule(&self) -> &CssomRuleId {
        &self.rule
    }
    pub const fn role(&self) -> CssomQueryResultRole {
        self.role
    }
    pub fn context(&self) -> &CssomContext {
        self.capture.context()
    }
    pub fn facts(&self) -> &CssomQueryFacts {
        &self.facts
    }
    pub const fn supplied_match(&self) -> bool {
        self.matched
    }
}
impl CssomSnapshot {
    fn query_role(&self, id: &CssomRuleId) -> Result<CssomQueryResultRole, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(_),
                ..
            } => Ok(CssomQueryResultRole::Media),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            } => Ok(CssomQueryResultRole::Supports),
            _ => Err(CssomError::WrongKind),
        }
    }
    fn query_fact(
        &self,
        version: &CssomInputVersion,
        role: CssomInputRole,
    ) -> Result<&CssomInputData, CssomError> {
        if version.role != role {
            return Err(CssomError::QueryMatch(
                CssomQueryMatchError::InvalidFactRole {
                    expected: role,
                    actual: version.role.clone(),
                },
            ));
        }
        self.state
            .context
            .inputs
            .iter()
            .find(|input| &input.version == version && input.data.role() == role)
            .map(|input| &input.data)
            .ok_or(CssomError::QueryMatch(CssomQueryMatchError::MissingFact(
                role,
            )))
    }
    /// Records an evaluator's boolean with explicit actual input endpoints. Media
    /// requires an associated sheet, Document and that Document's Window; Supports
    /// requires a support profile. No callback, host lookup or default is used.
    pub fn capture_query_result(
        &self,
        id: &CssomRuleId,
        facts: CssomQueryFacts,
        matched: bool,
    ) -> Result<CssomQueryResult, CssomError> {
        let role = self.query_role(id)?;
        match (&facts, role) {
            (CssomQueryFacts::Media { document, window }, CssomQueryResultRole::Media) => {
                let CssomInputData::Document { identity } =
                    self.query_fact(document, CssomInputRole::Document)?
                else {
                    unreachable!("checked role");
                };
                let CssomInputData::Window {
                    document: window_document,
                    ..
                } = self.query_fact(window, CssomInputRole::Window)?
                else {
                    unreachable!("checked role");
                };
                let sheet = self.parent_style_sheet(id)?.ok_or(CssomError::QueryMatch(
                    CssomQueryMatchError::MissingFact(CssomInputRole::Document),
                ))?;
                if window_document != identity
                    || self
                        .sheet(sheet)?
                        .inputs
                        .constructor_document
                        .as_ref()
                        .is_some_and(|sheet_document| sheet_document != identity)
                {
                    return Err(CssomError::QueryMatch(
                        CssomQueryMatchError::FactBindingMismatch(CssomInputRole::Document),
                    ));
                }
            }
            (CssomQueryFacts::Supports { support }, CssomQueryResultRole::Supports) => {
                self.query_fact(support, CssomInputRole::Support)?;
            }
            _ => return Err(CssomError::QueryMatch(CssomQueryMatchError::WrongRole)),
        }
        Ok(CssomQueryResult {
            capture: self.clone(),
            rule: id.clone(),
            role,
            facts,
            matched,
        })
    }
    /// Returns only a matching result from this exact immutable capture. Absence,
    /// stale input/state, a foreign owner or another rule are typed failures.
    pub fn condition_matches(
        &self,
        id: &CssomRuleId,
        result: Option<&CssomQueryResult>,
    ) -> Result<bool, CssomError> {
        let role = self.query_role(id)?;
        let result = result.ok_or(CssomError::QueryMatch(CssomQueryMatchError::MissingResult))?;
        let error = if result.owner() != self.owner() {
            Some(CssomQueryMatchError::ForeignOwner)
        } else if result.role != role {
            Some(CssomQueryMatchError::WrongRole)
        } else if &result.rule != id {
            Some(CssomQueryMatchError::WrongRule)
        } else if result.context() != self.context() {
            Some(CssomQueryMatchError::StaleContext)
        } else if result.revision() != self.revision() {
            Some(CssomQueryMatchError::StaleRevision)
        } else {
            None
        };
        if let Some(error) = error {
            return Err(CssomError::QueryMatch(error));
        }
        Ok(result.matched)
    }
}

/// A newly owned dictionary for one authored container entry. Reads retain
/// entry order/duplicates; modifying this transport cannot modify the rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssomContainerCondition {
    pub name: String,
    pub query: String,
}
fn text_limits(max_bytes: usize) -> CssSpecifiedValueSerializationLimits {
    CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, max_bytes)
}
fn container_name_text(
    entry: &CssContainerQueryEntry,
    max_bytes: usize,
) -> Result<String, CssomError> {
    entry
        .name()
        .map(|name| {
            serialize_css_identifier_with_limits(name.as_str(), text_limits(max_bytes))
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
                .serialize_with_limit(max_bytes)
                .map(|text| text.as_css().to_owned())
                .map_err(CssomError::Component)
        })
        .unwrap_or_else(|| Ok(String::new()))
}
impl CssomSnapshot {
    fn container_prelude(&self, id: &CssomRuleId) -> Result<&CssContainerPrelude, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(prelude),
                ..
            } => Ok(prelude),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Byte-bounded lexical fields, with one allowance for the aggregate emitted
    /// field bytes. Admitted checked CSS payload constraints remain independent.
    pub fn container_conditions(
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
    pub fn container_name(&self, id: &CssomRuleId, max_bytes: usize) -> Result<String, CssomError> {
        match self.container_prelude(id)?.entries() {
            [entry] => container_name_text(entry, max_bytes),
            _ => Ok(String::new()),
        }
    }
    pub fn container_query(
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
    pub fn container_condition_text(
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
                serialize_css_whitespace_separated_list_with_limits(fields, text_limits(max_bytes))
                    .map_err(CssomError::Serialization)?,
            );
        }
        let items: Vec<&str> = items.iter().map(String::as_str).collect();
        serialize_css_comma_separated_list_with_limits(&items, text_limits(max_bytes))
            .map_err(CssomError::Serialization)
    }
    /// The defined readonly name is independent of the unresolved non-rendering
    /// test body's child projection, insertion context and whole wrapper.
    pub fn named_support_condition_name(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Ordinary(CssRule::SupportsCondition(rule)),
            }
            | CssomRuleData::Leaf {
                current: CssomAuthoredRule::Scoped(CssScopedRule::SupportsCondition(rule)),
            } => Ok(rule.name().as_str()),
            _ => Err(CssomError::WrongKind),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RuleDestination {
    Sheet(CssomSheetId),
    Group(CssomRuleId),
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
        Err(error)
            if matches!(
                error.kind(),
                CssComponentValueErrorKind::NestingLimit
                    | CssComponentValueErrorKind::ComponentLimit
                    | CssComponentValueErrorKind::ByteLimit
                    | CssComponentValueErrorKind::CapacityOverflow
            ) =>
        {
            Err(CssomError::Component(error))
        }
        // Feature grammar owns syntax rejection and inner declaration recovery;
        // a whole-input component error is not itself its retention decision.
        Err(_) => Ok(()),
    }
}
pub(crate) fn check_diagnostics(diagnostics: &[CssRecoveryDiagnostic]) -> Result<(), CssomError> {
    for diagnostic in diagnostics {
        if matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
            || matches!(diagnostic.error().kind(), ErrorKind::InvalidComponentValue(e)
                if matches!(e.kind(), CssComponentValueErrorKind::NestingLimit | CssComponentValueErrorKind::ComponentLimit | CssComponentValueErrorKind::ByteLimit | CssComponentValueErrorKind::CapacityOverflow))
        {
            return Err(CssomError::ParseResource(Box::new(diagnostic.clone())));
        }
    }
    Ok(())
}
impl State {
    fn rule_namespaces(&self, id: &CssomRuleId) -> Result<CssNamespaceContext, CssomError> {
        Ok(match self.parent_style_sheet(id)? {
            Some(sheet) => self.sheet(sheet)?.namespaces.clone(),
            None => CssNamespaceContext::default(),
        })
    }
    fn style_ancestor(&self, id: &CssomRuleId) -> Result<CssStyleAncestor, CssomError> {
        let mut parent = &self.rule(id)?.parent;
        while let CssomParent::Rule(id) = parent {
            let rule = self.rule(id)?;
            if matches!(rule.data, CssomRuleData::Style { .. }) {
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
            match &rule.data {
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
        match &self.rule(id)?.data {
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
            let sheet = this.staged.sheet(id)?;
            if !sheet.inputs.origin_clean {
                return Err(CssomError::Source(CssomException::Security));
            }
            if sheet.inputs.disallow_modification {
                return Err(CssomError::Source(CssomException::NotAllowed));
            }
            let constructed = sheet.inputs.constructed;
            let (candidate, diagnostics) =
                classify_rule_syntax_with_limits(source, component_limits(&this.limits))
                    .into_parts();
            check_diagnostics(&diagnostics)?;
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
            let list = match &this.staged.rule(id)?.data {
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
            check_diagnostics(&diagnostics)?;
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
                    sheet.namespaces.clone(),
                    CssRuleAdmissionContext::Stylesheet,
                )
            }
            RuleDestination::Group(id) => {
                let rule = self.staged.rule(id)?;
                let list = match &rule.data {
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
                if matches!(self.staged.rules[id].data, CssomRuleData::Page { .. }));
            if page && !top_only {
                let (value, diagnostics) = candidate
                    .admit_page_margin_rule_with_context(parser)
                    .into_parts();
                check_diagnostics(&diagnostics)?;
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
                    check_diagnostics(&diagnostics)?;
                    retained_diagnostics = diagnostics;
                    admitted = value.map(|value| match value {
                        CssAdmittedRule::Ordinary(rule) => CssomAuthoredRule::Ordinary(rule),
                        CssAdmittedRule::Scoped(rule) => CssomAuthoredRule::Scoped(rule),
                    });
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
                check_diagnostics(&diagnostics)?;
                retained_diagnostics = diagnostics;
                admitted = value.map(|value| match value {
                    CssAdmittedRule::Ordinary(rule) => CssomAuthoredRule::Ordinary(rule),
                    CssAdmittedRule::Scoped(rule) => CssomAuthoredRule::Scoped(rule),
                });
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
                check_diagnostics(&diagnostics)?;
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
            && matches!(self.staged.rule(id)?.data, CssomRuleData::Page { .. })
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
                    &self.staged.rules[current].authored,
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
                    .any(|id| phase(&self.staged.rules[id].authored) > 1)
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
            .admission_diagnostics = retained_diagnostics;
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
        CssomAuthoredRule::Ordinary(CssRule::Import(_)) => 0,
        CssomAuthoredRule::Ordinary(CssRule::Namespace(_)) => 1,
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
    if matches!(
        rule,
        CssomAuthoredRule::Ordinary(CssRule::SupportsCondition(_))
    ) {
        return Ok(());
    }
    if !*seen_prefix
        && matches!(
            rule,
            CssomAuthoredRule::Ordinary(CssRule::LayerStatement(_))
        )
    {
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

impl CssomStore {
    /// Resolves the stable collection ID against the current live publication.
    pub fn rule_list_length(&self, list: &CssomRuleListId) -> Result<usize, CssomError> {
        self.snapshot().rule_list_length(list)
    }
    /// CSSRuleList.item returns null outside the current membership, without
    /// changing the collection identity. A separately captured snapshot stays fixed.
    pub fn rule_list_item(
        &self,
        list: &CssomRuleListId,
        index: usize,
    ) -> Result<Option<CssomRuleId>, CssomError> {
        self.snapshot()
            .rule_list_item(list, index)
            .map(|value| value.cloned())
    }
}
impl CssomSnapshot {
    /// Length in this immutable capture; use Store for live observations.
    pub fn rule_list_length(&self, list: &CssomRuleListId) -> Result<usize, CssomError> {
        Ok(self.rules(list)?.len())
    }
    pub fn rule_list_item(
        &self,
        list: &CssomRuleListId,
        index: usize,
    ) -> Result<Option<&CssomRuleId>, CssomError> {
        Ok(self.rules(list)?.get(index))
    }
    /// Deprecated CSSRule type values are determined from concrete typed kinds.
    pub fn rule_type(&self, id: &CssomRuleId) -> Result<u16, CssomError> {
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
                current: CssomAuthoredRule::Ordinary(CssRule::Namespace(_)),
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
    pub fn style_selector_text(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        let namespaces = self.state.rule_namespaces(id)?;
        let context = self.state.selector_context(id)?;
        match &self.rule(id)?.data {
            CssomRuleData::Style {
                selectors: CssomSelectors::Ordinary(list),
                ..
            } => list.serialize_cssom_with_limits(&namespaces, context, limits),
            CssomRuleData::Style {
                selectors: CssomSelectors::Scoped(list),
                ..
            } => list.serialize_cssom_with_limits(&namespaces, context, limits),
            _ => return Err(CssomError::WrongKind),
        }
        .map_err(CssomError::Serialization)
    }
    pub fn page_selector_text(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Page { selectors, .. } => selectors
                .serialize_cssom_with_limits(limits)
                .map_err(CssomError::Serialization),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn keyframe_key_text(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Keyframe { selectors, .. } => selectors
                .serialize_key_text_with_limits(limits)
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
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data, CssomRuleData::Style { .. }) {
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
            check_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok(false);
            };
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Style { selectors, .. } = &mut rule.data else {
                unreachable!()
            };
            *selectors = match parsed.selectors() {
                CssAdmittedStyleSelectors::Ordinary(value) => {
                    CssomSelectors::Ordinary(value.clone())
                }
                CssAdmittedStyleSelectors::Scoped(value) => CssomSelectors::Scoped(value.clone()),
            };
            rule.admission_diagnostics = diagnostics;
            rule.selector_input = Some(parsed.origin().clone());
            rule.selector_inputs = Some(inputs);
            rule.selector_namespaces = Some(namespaces);
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Selectors, CssomChange::Provenance],
            );
            Ok(true)
        })
    }
    /// Page selector admission uses its independent complete-list grammar.
    pub fn set_page_selector_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<bool, CssomError> {
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data, CssomRuleData::Page { .. }) {
                return Err(CssomError::WrongKind);
            }
            // The component owner preflights the whole raw input under product quotas.
            preflight_components(source, &this.limits)?;
            let (parsed, diagnostics) = parse_page_selector_list(source).into_parts();
            check_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok(false);
            };
            let origin = parsed.origin().cloned();
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Page { selectors, .. } = &mut rule.data else {
                unreachable!()
            };
            *selectors = parsed;
            rule.admission_diagnostics = diagnostics;
            rule.selector_input = origin;
            rule.selector_inputs = Some(inputs);
            this.affect(
                CssomObjectId::Rule(id.clone()),
                &[CssomChange::Selectors, CssomChange::Provenance],
            );
            Ok(true)
        })
    }
    pub fn set_keyframe_key_text(
        &mut self,
        id: &CssomRuleId,
        source: &str,
    ) -> Result<(), CssomError> {
        self.apply(|this| {
            if !matches!(this.staged.rule(id)?.data, CssomRuleData::Keyframe { .. }) {
                return Err(CssomError::WrongKind);
            }
            preflight_components(source, &this.limits)?;
            let (parsed, diagnostics) = parse_keyframe_selector_list(source).into_parts();
            check_diagnostics(&diagnostics)?;
            let parsed = parsed.ok_or_else(syntax)?;
            let inputs = this.staged.context.clone();
            let rule = this.staged.rules.get_mut(id).expect("owned rule");
            let CssomRuleData::Keyframe { selectors, .. } = &mut rule.data else {
                unreachable!()
            };
            *selectors = parsed.selectors().clone();
            rule.admission_diagnostics = diagnostics;
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
pub enum CssomKeyframeIndex {
    Rule(CssomRuleId),
    Undefined,
}
impl CssomSnapshot {
    pub fn keyframes_name(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Keyframes { name, .. } => Ok(match name {
                CssKeyframesName::Ident(value) => value.as_str(),
                CssKeyframesName::String(value) => value.as_str(),
                _ => return Err(CssomError::WrongKind),
            }),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn keyframes_length(&self, id: &CssomRuleId) -> Result<usize, CssomError> {
        let CssomRuleData::Keyframes { children, .. } = &self.rule(id)?.data else {
            return Err(CssomError::WrongKind);
        };
        self.rule_list_length(children)
    }
    pub fn keyframes_index(
        &self,
        id: &CssomRuleId,
        index: usize,
    ) -> Result<CssomKeyframeIndex, CssomError> {
        let CssomRuleData::Keyframes { children, .. } = &self.rule(id)?.data else {
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
            let rule = this.staged.rules.get_mut(id).ok_or_else(|| {
                if id.owner != this.staged.owner {
                    CssomError::ForeignOwner
                } else {
                    CssomError::MissingObject
                }
            })?;
            let CssomRuleData::Keyframes { name, .. } = &mut rule.data else {
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
        self.apply(|this| {
            let CssomRuleData::Keyframes { children, .. } = &this.staged.rule(id)?.data else {
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
            check_diagnostics(&diagnostics)?;
            let Some(parsed) = parsed else {
                return Ok(None);
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
                .admission_diagnostics = diagnostics;
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
            Ok(Some(this.retain_rule_ticket(child)))
        })
    }
}

impl CssomSnapshot {
    /// Authored location only; resolved location is a separately supplied import fact.
    pub fn import_href(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Import { target, .. } => Ok(match target {
                CssImportTarget::String(value) => value.as_str(),
                CssImportTarget::Url(value) => value.as_str(),
                _ => return Err(CssomError::WrongKind),
            }),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn import_media(&self, id: &CssomRuleId) -> Result<&CssomMediaListId, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Import { media, .. } => Ok(media),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn import_style_sheet(
        &self,
        id: &CssomRuleId,
    ) -> Result<Option<&CssomSheetId>, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Import { child_sheet, .. } => Ok(child_sheet.as_ref()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn import_supports_text(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Option<String>, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Import { supports, .. } => supports
                .as_ref()
                .map(|supports| {
                    supports
                        .condition()
                        .serialize_with_limit(max_bytes)
                        .map(|value| value.as_css().to_owned())
                        .map_err(CssomError::Component)
                })
                .transpose(),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn namespace_uri(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Ordinary(CssRule::Namespace(rule)),
            } => Ok(rule.name().as_str()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn namespace_prefix(&self, id: &CssomRuleId) -> Result<&str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Ordinary(CssRule::Namespace(rule)),
            } => Ok(rule.prefix().map_or("", CssNamespacePrefix::as_str)),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn margin_name(&self, id: &CssomRuleId) -> Result<&'static str, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Margin { name, .. } => Ok(name.css_name()),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn group_media(&self, id: &CssomRuleId) -> Result<&CssomMediaListId, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(media),
                ..
            } => Ok(media),
            _ => Err(CssomError::WrongKind),
        }
    }
    pub fn supports_condition_text(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(condition),
                ..
            } => condition
                .serialize_with_limit(max_bytes)
                .map(|value| value.as_css().to_owned())
                .map_err(CssomError::Component),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Omitted Scope bounds remain null; reads never require a whole-rule format.
    pub fn scope_start(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        let CssomRuleData::Group {
            prelude: CssomGroupPrelude::Scope { start, .. },
            ..
        } = &self.rule(id)?.data
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
                    .serialize_cssom_with_limits(
                        &namespaces,
                        CssScopeSelectorCssomContext::Start(nesting),
                        limits,
                    )
                    .map_err(CssomError::Serialization)
            })
            .transpose()
    }
    pub fn scope_end(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<Option<String>, CssomError> {
        let CssomRuleData::Group {
            prelude: CssomGroupPrelude::Scope { end, .. },
            ..
        } = &self.rule(id)?.data
        else {
            return Err(CssomError::WrongKind);
        };
        let namespaces = self.state.rule_namespaces(id)?;
        end.as_ref()
            .map(|value| {
                value
                    .serialize_cssom_with_limits(
                        &namespaces,
                        CssScopeSelectorCssomContext::End,
                        limits,
                    )
                    .map_err(CssomError::Serialization)
            })
            .transpose()
    }
}
impl CssomSnapshot {
    /// The same collection handle is retained through insertion, deletion and detach.
    /// Keyframes uses its own operations even though it also has a rule list.
    pub fn child_rules(&self, id: &CssomRuleId) -> Result<&CssomRuleListId, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Style { children, .. }
            | CssomRuleData::Group { children, .. }
            | CssomRuleData::Page { children, .. }
            | CssomRuleData::Keyframes { children, .. } => Ok(children),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Associated declaration identity; validity domain and special flags remain
    /// in that owning block rather than being converted to ordinary properties.
    pub fn rule_style(&self, id: &CssomRuleId) -> Result<&CssomBlockId, CssomError> {
        self.rule(id)?.data.block().ok_or(CssomError::WrongKind)
    }
    pub fn condition_text(
        &self,
        id: &CssomRuleId,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(media),
                ..
            } => self
                .media(media)?
                .serialize_cssom_with_limits(limits)
                .map(|value| value.as_css().to_owned())
                .map_err(CssomError::Media),
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(_),
                ..
            } => self.supports_condition_text(id, limits.max_css_bytes()),
            _ => Err(CssomError::WrongKind),
        }
    }
}

use std::cell::OnceCell;

/// Limits one current rule text read. Preparation counts actual context/traversal
/// steps and view nodes; CSS independently owns its one cumulative graph budget.
/// These bound work and owned buffers, not allocator overhead or process RSS.
#[derive(Clone, Debug)]
pub struct CssomRuleFormatLimits {
    pub max_nodes: usize,
    pub max_preparation_steps: usize,
    pub max_depth: usize,
    pub css: CssSpecifiedValueSerializationLimits,
}
impl Default for CssomRuleFormatLimits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_preparation_steps: 1_000_000,
            max_depth: 256,
            css: CssSpecifiedValueSerializationLimits::default(),
        }
    }
}
struct RuleFormatWork<'a> {
    limits: &'a CssomRuleFormatLimits,
    used: usize,
}
impl RuleFormatWork<'_> {
    fn step(&mut self) -> Result<(), CssomError> {
        self.used = self.used.checked_add(1).ok_or(CssomError::Limit {
            resource: "rule format preparation work",
            maximum: self.limits.max_preparation_steps,
        })?;
        if self.used > self.limits.max_preparation_steps {
            return Err(CssomError::Limit {
                resource: "rule format preparation work",
                maximum: self.limits.max_preparation_steps,
            });
        }
        Ok(())
    }
    fn depth(&self, depth: usize) -> Result<(), CssomError> {
        if depth > self.limits.max_depth {
            return Err(CssomError::Limit {
                resource: "rule format preparation depth",
                maximum: self.limits.max_depth,
            });
        }
        Ok(())
    }
}
fn format_reserve<T>(items: &mut Vec<T>, additional: usize) -> Result<(), CssomError> {
    items
        .try_reserve(additional)
        .map_err(|_| CssomError::Limit {
            resource: "rule format preparation capacity",
            maximum: additional,
        })
}
impl CssomSnapshot {
    fn current_properties(
        &self,
        id: &CssomBlockId,
    ) -> Result<&CssSpecifiedDeclarationBlock, CssomError> {
        match &self.block(id)?.data {
            CssomBlockData::Properties {
                selected: CssomProjection::Available(value),
                ..
            } => Ok(value),
            CssomBlockData::Properties {
                selected: CssomProjection::Unavailable(error),
                ..
            } => Err(CssomError::Declaration(error.clone())),
            _ => Err(CssomError::WrongKind),
        }
    }
    fn current_page(
        &self,
        id: &CssomBlockId,
    ) -> Result<&CssSpecifiedPageDeclarationBlock, CssomError> {
        match &self.block(id)?.data {
            CssomBlockData::Page {
                selected: CssomProjection::Available(value),
                ..
            } => Ok(value),
            CssomBlockData::Page {
                selected: CssomProjection::Unavailable(error),
                ..
            } => Err(CssomError::Page(error.clone())),
            _ => Err(CssomError::WrongKind),
        }
    }
    fn keyframe_view(&self, id: &CssomRuleId) -> Result<CssKeyframeRuleView<'_>, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Keyframe { selectors, block } => {
                CssKeyframeRuleView::try_new(selectors, self.current_properties(block)?)
                    .map_err(CssomError::Keyframe)
            }
            _ => Err(CssomError::WrongKind),
        }
    }
    fn margin_view(&self, id: &CssomRuleId) -> Result<CssMarginRuleView<'_>, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Margin { name, block } => {
                CssMarginRuleView::try_new(*name, self.current_properties(block)?)
                    .ok_or(CssomError::WrongKind)
            }
            _ => Err(CssomError::WrongKind),
        }
    }
    fn original_failure_input(
        &self,
        id: &CssomRuleId,
    ) -> Result<CssRuleGraphInput<'_>, CssomError> {
        match self.rule(id)?.authored() {
            CssomAuthoredRule::Ordinary(rule) => Ok(CssRuleGraphInput::from_rule(rule)),
            CssomAuthoredRule::Scoped(rule) => Ok(CssRuleGraphInput::from_scoped_rule(rule)),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Formats actual current checked payloads/membership with one native CSS
    /// graph writer. Request-local preparation is independently metered by actual
    /// ancestry, traversal, edge and view-buffer work, not private CSS tariffs.
    pub fn rule_css_text(
        &self,
        id: &CssomRuleId,
        limits: &CssomRuleFormatLimits,
    ) -> Result<String, CssomError> {
        let root = self.rule(id)?;
        let mut work = RuleFormatWork { limits, used: 0 };
        work.step()?;
        work.depth(1)?;
        if limits.max_nodes == 0 {
            return Err(CssomError::Limit {
                resource: "rule format nodes",
                maximum: 0,
            });
        }
        if matches!(root.data, CssomRuleData::Keyframe { .. }) {
            return self
                .keyframe_view(id)?
                .serialize_cssom_with_limits(limits.css)
                .map_err(CssomError::Keyframe);
        }
        if matches!(root.data, CssomRuleData::Margin { .. }) {
            return self
                .margin_view(id)?
                .serialize_cssom_with_limits(limits.css)
                .map_err(CssomError::Format);
        }
        let mut parent = &root.parent;
        let mut ancestor = CssStyleAncestor::Absent;
        let mut sheet = None;
        let mut depth = 1usize;
        loop {
            match parent {
                CssomParent::Rule(id) => {
                    work.step()?;
                    depth += 1;
                    work.depth(depth)?;
                    let rule = self.rule(id)?;
                    if matches!(rule.data, CssomRuleData::Style { .. }) {
                        ancestor = CssStyleAncestor::Present;
                    }
                    parent = &rule.parent;
                }
                CssomParent::Sheet(id) => {
                    sheet = Some(self.sheet(id)?);
                    break;
                }
                CssomParent::Detached => break,
            }
        }
        let detached = CssNamespaceContext::default();
        let namespaces = sheet.map_or(&detached, |sheet| &sheet.namespaces);
        let mut nodes = Vec::<(&CssomRuleId, Vec<usize>)>::new();
        let mut pending = Vec::new();
        format_reserve(&mut pending, 1)?;
        pending.push((id, None::<usize>, 1usize));
        while let Some((id, parent, depth)) = pending.pop() {
            work.step()?;
            work.depth(depth)?;
            if nodes.len() >= limits.max_nodes {
                return Err(CssomError::Limit {
                    resource: "rule format nodes",
                    maximum: limits.max_nodes,
                });
            }
            let index = nodes.len();
            format_reserve(&mut nodes, 1)?;
            nodes.push((id, Vec::new()));
            if let Some(parent) = parent {
                work.step()?;
                format_reserve(&mut nodes[parent].1, 1)?;
                nodes[parent].1.push(index);
            }
            let rule = self.rule(id)?;
            // Unavailable/source-undefined wrappers fail before descendant
            // scheduling in the native writer. Do not prepare their bodies.
            let children = match &rule.data {
                CssomRuleData::Style { children, .. }
                | CssomRuleData::Page { children, .. }
                | CssomRuleData::Keyframes { children, .. } => Some(children),
                CssomRuleData::Group {
                    prelude: CssomGroupPrelude::Media(_) | CssomGroupPrelude::Supports(_),
                    children,
                } => Some(children),
                _ => None,
            };
            if let Some(children) = children {
                for child in self.rules(children)?.iter().rev() {
                    work.step()?;
                    if nodes.len().saturating_add(pending.len()) >= limits.max_nodes {
                        return Err(CssomError::Limit {
                            resource: "rule format nodes",
                            maximum: limits.max_nodes,
                        });
                    }
                    format_reserve(&mut pending, 1)?;
                    pending.push((child, Some(index), depth + 1));
                }
            }
        }
        let mut views = Vec::new();
        let mut children = Vec::new();
        let mut margins = Vec::new();
        let mut keyframes = Vec::new();
        for _ in &nodes {
            work.step()?;
            format_reserve(&mut views, 1)?;
            format_reserve(&mut children, 1)?;
            format_reserve(&mut margins, 1)?;
            format_reserve(&mut keyframes, 1)?;
            views.push(OnceCell::<CssRuleGraphInput<'_>>::new());
            children.push(OnceCell::<Vec<CssRuleGraphInput<'_>>>::new());
            margins.push(OnceCell::<Vec<CssMarginRuleView<'_>>>::new());
            keyframes.push(OnceCell::<Vec<CssKeyframeRuleView<'_>>>::new());
        }
        for (index, (id, child_indexes)) in nodes.iter().enumerate().rev() {
            work.step()?;
            let rule = self.rule(id)?;
            // Page/keyframe children use their actual native leaf view buffers.
            // Those leaf nodes need no standalone writer or graph input.
            if matches!(
                rule.data,
                CssomRuleData::Margin { .. } | CssomRuleData::Keyframe { .. }
            ) {
                continue;
            }
            let input = match &rule.data {
                CssomRuleData::Style { .. } | CssomRuleData::Group { .. } => {
                    let mut current = Vec::new();
                    for child in child_indexes {
                        work.step()?;
                        format_reserve(&mut current, 1)?;
                        current.push(*views[*child].get().ok_or(CssomError::WrongKind)?);
                    }
                    children[index].set(current).expect("single assignment");
                    let current = children[index].get().expect("initialized");
                    match &rule.data {
                        CssomRuleData::Style {
                            selectors, block, ..
                        } => match selectors {
                            CssomSelectors::Ordinary(selectors) => CssRuleGraphInput::style(
                                selectors,
                                self.current_properties(block)?,
                                current,
                            ),
                            CssomSelectors::Scoped(selectors) => CssRuleGraphInput::scoped_style(
                                selectors,
                                self.current_properties(block)?,
                                current,
                            ),
                        },
                        CssomRuleData::Group { prelude, .. } => match prelude {
                            CssomGroupPrelude::Media(media) => CssRuleGraphInput::group(
                                CssEditedGroupPreludeRef::Media(self.media(media)?),
                                current,
                            ),
                            CssomGroupPrelude::Supports(condition) => CssRuleGraphInput::group(
                                CssEditedGroupPreludeRef::Supports(condition),
                                current,
                            ),
                            CssomGroupPrelude::Container(condition) => CssRuleGraphInput::group(
                                CssEditedGroupPreludeRef::Container(condition),
                                current,
                            ),
                            CssomGroupPrelude::Layer(name) => CssRuleGraphInput::group(
                                CssEditedGroupPreludeRef::Layer(name.as_ref()),
                                current,
                            ),
                            _ => self.original_failure_input(id)?,
                        },
                        _ => unreachable!(),
                    }
                }
                CssomRuleData::NestedDeclarations { block } => {
                    CssRuleGraphInput::nested_declarations(self.current_properties(block)?)
                }
                CssomRuleData::Page {
                    selectors, block, ..
                } => {
                    let mut current = Vec::new();
                    for child in child_indexes {
                        work.step()?;
                        format_reserve(&mut current, 1)?;
                        current.push(self.margin_view(nodes[*child].0)?);
                    }
                    margins[index].set(current).expect("single assignment");
                    CssRuleGraphInput::page(CssPageRuleView::new(
                        selectors,
                        self.current_page(block)?,
                        margins[index].get().expect("initialized"),
                    ))
                }
                CssomRuleData::Keyframes { name, .. } => {
                    let mut current = Vec::new();
                    for child in child_indexes {
                        work.step()?;
                        format_reserve(&mut current, 1)?;
                        current.push(self.keyframe_view(nodes[*child].0)?);
                    }
                    keyframes[index].set(current).expect("single assignment");
                    CssRuleGraphInput::keyframes(name, keyframes[index].get().expect("initialized"))
                }
                CssomRuleData::Import { media, .. } => {
                    let original = match rule.authored() {
                        CssomAuthoredRule::Ordinary(CssRule::Import(import)) => import,
                        _ => return Err(CssomError::WrongKind),
                    };
                    CssRuleGraphInput::import(original, self.media(media)?)
                }
                CssomRuleData::FontFace { block } => {
                    match self.block(block)?.selected_font_face() {
                        Some(CssomProjection::Available(selected)) => {
                            CssRuleGraphInput::font_face(selected)
                        }
                        Some(CssomProjection::Unavailable(error)) => {
                            return Err(CssomError::FontFace(error.clone()));
                        }
                        None => return Err(CssomError::WrongKind),
                    }
                }
                CssomRuleData::Leaf { current } => match current {
                    CssomAuthoredRule::Ordinary(rule) => CssRuleGraphInput::from_rule(rule),
                    CssomAuthoredRule::Scoped(rule) => CssRuleGraphInput::from_scoped_rule(rule),
                    _ => return Err(CssomError::WrongKind),
                },
                // The native whole-rule wrapper is unavailable before its body.
                // This genuine original is only a failure carrier: the current
                // query may have a different live MediaList payload.
                CssomRuleData::CustomMedia { .. } => self.original_failure_input(id)?,
                _ => self.original_failure_input(id)?,
            };
            views[index].set(input).expect("single assignment");
        }
        views[0]
            .get()
            .expect("root initialized")
            .serialize_cssom_with_limits(namespaces, ancestor, limits.css)
            .map_err(CssomError::Format)
    }
}

impl CssomSnapshot {
    /// Declared local name only: absent is null, anonymous is an empty string.
    pub fn import_layer_name(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Option<String>, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Import { layer, .. } => match layer {
                None => Ok(None),
                Some(CssImportLayer::Anonymous) => Ok(Some(String::new())),
                Some(CssImportLayer::Named(name)) => name
                    .serialize_specified_with_limits(text_limits(max_bytes))
                    .map(Some)
                    .map_err(CssomError::Serialization),
                _ => Err(CssomError::WrongKind),
            },
            _ => Err(CssomError::WrongKind),
        }
    }
    /// Rule-local checked name, without ancestor layer prefixes.
    pub fn layer_block_name(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<String, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Layer(name),
                ..
            } => name
                .as_ref()
                .map(|name| {
                    name.serialize_specified_with_limits(text_limits(max_bytes))
                        .map_err(CssomError::Serialization)
                })
                .unwrap_or_else(|| Ok(String::new())),
            _ => Err(CssomError::WrongKind),
        }
    }
    /// A newly owned array of local names, retaining order and duplicates. The
    /// byte allowance is shared across emitted names; no ancestor qualification.
    pub fn layer_statement_names(
        &self,
        id: &CssomRuleId,
        max_bytes: usize,
    ) -> Result<Vec<String>, CssomError> {
        let names = match &self.rule(id)?.data {
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Ordinary(CssRule::LayerStatement(rule)),
            } => rule.names(),
            CssomRuleData::Leaf {
                current: CssomAuthoredRule::Scoped(CssScopedRule::LayerStatement(rule)),
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
                .serialize_specified_with_limits(text_limits(remaining))
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
        let list = match &self.rule(id)?.data {
            CssomRuleData::Keyframes { children, .. } => children,
            _ => return Err(CssomError::WrongKind),
        };
        preflight_components(source, limits)?;
        let (query, diagnostics) = parse_keyframe_selector_list(source).into_parts();
        check_diagnostics(&diagnostics)?;
        let Some(query) = query else {
            return Ok(None);
        };
        // One query and one monotonic native normalization/comparison budget for
        // the complete reverse-last search. No authored equality or text keys.
        let mut matcher =
            CssKeyframeSelectorMatcher::try_new_with_limits(query.selectors(), limits.css)
                .map_err(CssomError::KeyframeComparison)?;
        let children = self.lists.get(list).expect("owned Keyframes list");
        for (visited, (index, child)) in children.iter().enumerate().rev().enumerate() {
            if visited >= limits.max_entries {
                return Err(CssomError::Limit {
                    resource: "keyframe search candidates",
                    maximum: limits.max_entries,
                });
            }
            let CssomRuleData::Keyframe { selectors, .. } = &self.rule(child)?.data else {
                return Err(CssomError::WrongKind);
            };
            if matcher
                .matches(selectors)
                .map_err(CssomError::KeyframeComparison)?
            {
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
