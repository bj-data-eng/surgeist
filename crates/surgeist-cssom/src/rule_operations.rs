//! Source rule algorithms over the current retained graph. CSS owns all parsing.
use crate::{model::State, *};
use surgeist_css::*;

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
            // Initial layer statements are allowed before the import/namespace prefix;
            // subsequent layer statements belong to the body phase.
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
