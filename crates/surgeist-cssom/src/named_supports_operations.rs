//! Surgeist's inert named-support policy; CSS owns syntax and reconstruction.
use crate::parser_resources::check_resource_diagnostics;
use crate::{model::State, rule_operations::component_limits, *};
use surgeist_css::*;

fn own_child(child: CssSupportsTestChildRef<'_>) -> CssSupportsTestChild {
    match child {
        CssSupportsTestChildRef::Declaration(v) => CssSupportsTestChild::Declaration(v.clone()),
        CssSupportsTestChildRef::QualifiedRule(v) => CssSupportsTestChild::QualifiedRule(v.clone()),
        CssSupportsTestChildRef::AtRule(v) => CssSupportsTestChild::AtRule(v.clone()),
    }
}

impl State {
    pub(crate) fn named_supports_data(
        &mut self,
        name: &CssSupportsConditionName,
        body: &CssSupportsTestBody,
        parent: &CssomRuleId,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleData, CssomError> {
        validate_body(name, body, depth, limits)?;
        let children = self.list(limits)?;
        for child in body.children() {
            let child = self.adopt(
                CssomAuthoredRule::SupportsTestChild(own_child(child)),
                CssomParent::Rule(parent.clone()),
                depth.checked_add(1).ok_or(CssomError::Limit {
                    resource: "rule depth",
                    maximum: limits.max_depth,
                })?,
                limits,
            )?;
            self.lists.get_mut(&children).expect("new list").push(child);
        }
        Ok(CssomRuleData::NamedSupports {
            name: name.clone(),
            body: body.clone(),
            children,
        })
    }
    fn named_supports_guard(&self, id: &CssomRuleId) -> Result<(), CssomError> {
        self.rule(id)?;
        if let Some(sheet) = self.parent_style_sheet(id)? {
            let sheet = self.sheet(sheet)?;
            if !sheet.inputs.origin_clean {
                return Err(CssomError::Source(CssomException::Security));
            }
            if sheet.inputs.disallow_modification {
                return Err(CssomError::Source(CssomException::NotAllowed));
            }
        }
        Ok(())
    }
    fn named_supports_depth(
        &self,
        id: &CssomRuleId,
        limits: &CssomLimits,
    ) -> Result<usize, CssomError> {
        let mut depth = 1usize;
        let mut parent = &self.rule(id)?.parent;
        while let CssomParent::Rule(id) = parent {
            depth = depth
                .checked_add(1)
                .filter(|depth| *depth <= limits.max_depth)
                .ok_or(CssomError::Limit {
                    resource: "rule depth",
                    maximum: limits.max_depth,
                })?;
            parent = &self.rule(id)?.parent;
        }
        Ok(depth)
    }
}

fn validate_body(
    name: &CssSupportsConditionName,
    body: &CssSupportsTestBody,
    depth: usize,
    limits: &CssomLimits,
) -> Result<(), CssomError> {
    retained_test_input_bytes(name.origin(), limits)?;
    let remaining = limits
        .max_depth
        .checked_sub(depth)
        .ok_or(CssomError::Limit {
            resource: "rule depth",
            maximum: limits.max_depth,
        })?;
    // One native request checks all opaque candidate/component nesting and work;
    // a list of live leaf IDs cannot establish these bounds.
    CssRuleGraphInput::named_supports(name, body)
        .serialize_cssom_with_graph_limits(
            &CssNamespaceContext::default(),
            CssStyleAncestor::Absent,
            CssRuleGraphSerializationLimits {
                values: limits.css,
                max_depth: remaining,
            },
        )
        .map(|_| ())
        .map_err(CssomError::Format)
}

impl CssomBatch {
    pub(crate) fn insert_named_supports_child(
        &mut self,
        id: &CssomRuleId,
        source: &str,
        index: usize,
    ) -> Result<CssomRuleTicket, CssomError> {
        self.staged.named_supports_guard(id)?;
        let CssomRuleData::NamedSupports { children, .. } = &self.staged.rule(id)?.data else {
            return Err(CssomError::WrongKind);
        };
        let list = children.clone();
        if index > self.staged.lists[&list].len() {
            return Err(CssomError::Source(CssomException::IndexSize));
        }
        if source.len() > self.limits.max_input_bytes {
            return Err(CssomError::Limit {
                resource: "parse input bytes",
                maximum: self.limits.max_input_bytes,
            });
        }
        let (child, diagnostics) =
            parse_supports_test_child_with_limits(source, component_limits(&self.limits))
                .into_parts();
        check_resource_diagnostics(&diagnostics)?;
        let child = child.ok_or(CssomError::Source(CssomException::Syntax))?;
        let depth = self.staged.named_supports_depth(id, &self.limits)?;
        let child = self.staged.adopt(
            CssomAuthoredRule::SupportsTestChild(child),
            CssomParent::Rule(id.clone()),
            depth.checked_add(1).ok_or(CssomError::Limit {
                resource: "rule depth",
                maximum: self.limits.max_depth,
            })?,
            &self.limits,
        )?;
        self.staged
            .rules
            .get_mut(&child)
            .expect("adopted child")
            .admission_diagnostics = diagnostics;
        self.staged
            .lists
            .get_mut(&list)
            .expect("owned list")
            .insert(index, child.clone());
        self.rebuild_named_supports(id, depth)?;
        self.affect(
            CssomObjectId::RuleList(list),
            &[CssomChange::RuleMembership, CssomChange::Order],
        );
        self.affect(
            CssomObjectId::Rule(child.clone()),
            &[CssomChange::Ancestry, CssomChange::Provenance],
        );
        Ok(self.retain_rule_ticket(child))
    }
    pub(crate) fn delete_named_supports_child(
        &mut self,
        id: &CssomRuleId,
        index: usize,
    ) -> Result<CssomRuleId, CssomError> {
        self.staged.named_supports_guard(id)?;
        let CssomRuleData::NamedSupports { children, .. } = &self.staged.rule(id)?.data else {
            return Err(CssomError::WrongKind);
        };
        let list = children.clone();
        let child = self.staged.lists[&list]
            .get(index)
            .ok_or(CssomError::Source(CssomException::IndexSize))?
            .clone();
        self.staged
            .lists
            .get_mut(&list)
            .expect("owned list")
            .remove(index);
        self.staged
            .rules
            .get_mut(&child)
            .expect("retained child")
            .parent = CssomParent::Detached;
        let depth = self.staged.named_supports_depth(id, &self.limits)?;
        self.rebuild_named_supports(id, depth)?;
        self.affect(
            CssomObjectId::RuleList(list),
            &[CssomChange::RuleMembership, CssomChange::Order],
        );
        self.affect(
            CssomObjectId::Rule(child.clone()),
            &[CssomChange::Ancestry, CssomChange::Provenance],
        );
        Ok(child)
    }
    fn rebuild_named_supports(&mut self, id: &CssomRuleId, depth: usize) -> Result<(), CssomError> {
        let CssomRuleData::NamedSupports {
            name,
            body,
            children,
        } = &self.staged.rule(id)?.data
        else {
            return Err(CssomError::WrongKind);
        };
        let mut payloads = Vec::new();
        payloads
            .try_reserve(self.staged.lists[children].len())
            .map_err(|_| CssomError::Limit {
                resource: "named test composition capacity",
                maximum: self.limits.max_entries,
            })?;
        for child in &self.staged.lists[children] {
            let CssomRuleData::SupportsTestChild(child) = &self.staged.rule(child)?.data else {
                return Err(CssomError::WrongKind);
            };
            payloads.push(child.as_ref());
        }
        let current = body
            .recompose_children_with_limits(&payloads, component_limits(&self.limits))
            .map_err(CssomError::NamedSupports)?;
        validate_body(name, &current, depth, &self.limits)?;
        let CssomRuleData::NamedSupports { body, .. } =
            &mut self.staged.rules.get_mut(id).expect("owned rule").data
        else {
            unreachable!()
        };
        *body = current;
        self.affect(
            CssomObjectId::Rule(id.clone()),
            &[CssomChange::Modification, CssomChange::Provenance],
        );
        Ok(())
    }
    /// Immutable current staged test payload; it exposes no tentative live IDs.
    pub fn named_supports_body(
        &self,
        id: &CssomRuleId,
    ) -> Result<&CssSupportsTestBody, CssomError> {
        if self.poisoned {
            return Err(CssomError::BatchAborted);
        }
        self.base.rule(id)?;
        current_body(&self.staged, id)
    }
}

fn current_body<'a>(
    state: &'a State,
    id: &CssomRuleId,
) -> Result<&'a CssSupportsTestBody, CssomError> {
    match &state.rule(id)?.data {
        CssomRuleData::NamedSupports { body, .. } => Ok(body),
        _ => Err(CssomError::WrongKind),
    }
}
impl CssomSnapshot {
    /// Current checked test contents and CSS-owned recovery metadata. The original
    /// rule and historical diagnostics remain separately retained.
    pub fn named_supports_body(
        &self,
        id: &CssomRuleId,
    ) -> Result<&CssSupportsTestBody, CssomError> {
        current_body(&self.state, id)
    }
    /// Typed opaque inert leaf, including immutable nested candidate contents.
    pub fn named_supports_test_child(
        &self,
        id: &CssomRuleId,
    ) -> Result<&CssSupportsTestChild, CssomError> {
        match &self.rule(id)?.data {
            CssomRuleData::SupportsTestChild(child) => Ok(child),
            _ => Err(CssomError::WrongKind),
        }
    }
}

/// Count actual retained payload entries without interpreting CSS grammar or
/// estimating the native writer's tariff. Source bytes are charged per retained
/// direct occurrence, as a logical input bound rather than allocator accounting.
pub(crate) fn test_child_storage(
    child: CssSupportsTestChildRef<'_>,
    limits: &CssomLimits,
) -> Result<(usize, usize), CssomError> {
    enum Entry<'a> {
        Child(CssSupportsTestChildRef<'a>),
        Component(&'a CssComponentValue),
    }
    let mut pending = Vec::new();
    let mut entries = 0usize;
    fn push<'a>(
        pending: &mut Vec<Entry<'a>>,
        entry: Entry<'a>,
        entries: usize,
        limits: &CssomLimits,
    ) -> Result<(), CssomError> {
        if pending.len().saturating_add(entries) >= limits.max_entries {
            return Err(CssomError::Limit {
                resource: "test payload entries",
                maximum: limits.max_entries,
            });
        }
        pending.try_reserve(1).map_err(|_| CssomError::Limit {
            resource: "test payload capacity",
            maximum: limits.max_entries,
        })?;
        pending.push(entry);
        Ok(())
    }
    let origin = match child {
        CssSupportsTestChildRef::Declaration(v) => v.origin(),
        CssSupportsTestChildRef::QualifiedRule(v) => v.origin(),
        CssSupportsTestChildRef::AtRule(v) => v.origin(),
    };
    let bytes = retained_test_input_bytes(origin, limits)?;
    push(&mut pending, Entry::Child(child), entries, limits)?;
    while let Some(entry) = pending.pop() {
        entries = entries
            .checked_add(1)
            .filter(|n| *n <= limits.max_entries)
            .ok_or(CssomError::Limit {
                resource: "test payload entries",
                maximum: limits.max_entries,
            })?;
        let (values, body) = match entry {
            Entry::Child(CssSupportsTestChildRef::Declaration(v)) => (v.components(), None),
            Entry::Child(CssSupportsTestChildRef::QualifiedRule(v)) => {
                (v.prelude().items(), Some(v.body()))
            }
            Entry::Child(CssSupportsTestChildRef::AtRule(v)) => {
                push(
                    &mut pending,
                    Entry::Component(v.at_keyword()),
                    entries,
                    limits,
                )?;
                (v.prelude().items(), v.body())
            }
            Entry::Component(v) => (
                match v.view() {
                    CssComponentValueRef::Function(v) => v.values().items(),
                    CssComponentValueRef::Block(v) => v.values().items(),
                    _ => &[],
                },
                None,
            ),
        };
        for value in values {
            push(&mut pending, Entry::Component(value), entries, limits)?;
        }
        if let Some(body) = body {
            entries = entries
                .checked_add(body.local_recovery_origins().len())
                .filter(|n| *n <= limits.max_entries)
                .ok_or(CssomError::Limit {
                    resource: "test payload entries",
                    maximum: limits.max_entries,
                })?;
            for child in body.children() {
                push(&mut pending, Entry::Child(child), entries, limits)?;
            }
        }
    }
    Ok((entries, bytes))
}

pub(crate) fn retained_test_input_bytes(
    origin: &CssValueOrigin,
    limits: &CssomLimits,
) -> Result<usize, CssomError> {
    let bytes = match origin {
        CssValueOrigin::Parsed(v) => v.source().as_str().len(),
        CssValueOrigin::ImplicitClosure { opening, .. } => opening.source().as_str().len(),
        _ => 0,
    };
    if bytes > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "retained test input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    Ok(bytes)
}
