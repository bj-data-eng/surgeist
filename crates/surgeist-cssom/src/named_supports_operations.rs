//! Surgeist's inert named-support policy; CSS owns syntax and reconstruction.
use crate::css_adapter::Adoption;
use crate::parser_resources::check_resource_diagnostics;
use crate::{model::State, rule_operations::component_limits, *};
use surgeist_css::component::{CssComponentValue, CssComponentValueRef, CssValueOrigin};
use surgeist_css::query::supports::{
    CssSupportsConditionName, CssSupportsTestBody, CssSupportsTestChild, CssSupportsTestChildRef,
};
use surgeist_css::rule::CssStyleAncestor;
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::{CssRuleGraphInput, CssRuleSerializationLimits};
use surgeist_css::{CssParser, CssParserContext};

/// The refusal for a test child of a kind this CSSOM does not know.
fn unsupported_test_child() -> CssomError {
    CssomError::InvalidInput("unsupported named-supports test child")
}

fn own_child(child: CssSupportsTestChildRef<'_>) -> Result<CssSupportsTestChild, CssomError> {
    match child {
        CssSupportsTestChildRef::Declaration(v) => Ok(CssSupportsTestChild::Declaration(v.clone())),
        CssSupportsTestChildRef::QualifiedRule(v) => {
            Ok(CssSupportsTestChild::QualifiedRule(v.clone()))
        }
        CssSupportsTestChildRef::AtRule(v) => Ok(CssSupportsTestChild::AtRule(v.clone())),
        _ => Err(unsupported_test_child()),
    }
}

impl Adoption<'_> {
    pub(crate) fn named_supports_data(
        &mut self,
        name: &CssSupportsConditionName,
        body: &CssSupportsTestBody,
        parent: &CssomRuleId,
        depth: usize,
    ) -> Result<CssomRuleData, CssomError> {
        let limits = self.limits();
        validate_body(name, body, depth, limits)?;
        let children = self.list()?;
        for child in body.children() {
            let child = self.adopt(
                CssomAuthoredRule::SupportsTestChild(own_child(child)?),
                CssomParent::Rule(parent.clone()),
                depth.checked_add(1).ok_or(CssomError::Limit {
                    resource: "rule depth",
                    maximum: limits.max_depth,
                })?,
            )?;
            self.push_child(&children, child);
        }
        Ok(CssomRuleData::NamedSupports {
            name: name.clone(),
            body: body.clone(),
            children,
        })
    }
}

impl State {
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
    CssRuleGraphInput::named_supports(name, body).serialize_cssom(
        &CssNamespaceContext::default(),
        CssStyleAncestor::Absent,
        CssRuleSerializationLimits {
            values: limits.css,
            max_depth: remaining,
        },
    )?;
    Ok(())
}

impl CssomBatch {
    pub(crate) fn insert_named_supports_child(
        &mut self,
        id: &CssomRuleId,
        source: &str,
        index: usize,
    ) -> Result<CssomRuleTicket, CssomError> {
        self.staged.named_supports_guard(id)?;
        let CssomRuleData::NamedSupports { children, .. } = self.staged.rule(id)?.data() else {
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
        let (child, diagnostics) = CssSupportsTestChild::parse(
            &CssParser::new(CssParserContext::default(), component_limits(&self.limits)),
            source,
        )
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
            .admission_diagnostics = diagnostics.into();
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
        let CssomRuleData::NamedSupports { children, .. } = self.staged.rule(id)?.data() else {
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
        } = self.staged.rule(id)?.data()
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
            let CssomRuleData::SupportsTestChild(child) = self.staged.rule(child)?.data() else {
                return Err(CssomError::WrongKind);
            };
            payloads.push(child.as_ref());
        }
        let current = body.recompose_children(&payloads, component_limits(&self.limits))?;
        validate_body(name, &current, depth, &self.limits)?;
        let CssomRuleData::NamedSupports { body, .. } = self
            .staged
            .rules
            .get_mut(id)
            .expect("owned rule")
            .data_mut()
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
    match state.rule(id)?.data() {
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
        match self.rule(id)?.data() {
            CssomRuleData::SupportsTestChild(child) => Ok(child),
            _ => Err(CssomError::WrongKind),
        }
    }
}

/// What the traversal of one test child found, which depends on the child alone.
///
/// The traversal counts the entries of the child and the entries it has queued
/// beside them. A limit refuses when the entries it counts, or those and the
/// queued ones, exceed it, so `need` is the smallest limit that is not refused.
#[derive(Clone, Debug)]
pub(crate) struct TestStorage {
    /// The source bytes of the child's origin.
    bytes: usize,
    /// The entries counted when the traversal ended or stopped.
    entries: usize,
    /// The smallest entry limit the traversal is not refused under before it ends or stops.
    need: usize,
    stop: Option<Stop>,
}

/// Why a traversal stopped before it ended.
#[derive(Clone, Copy, Debug)]
enum Stop {
    /// The child is of a kind this CSSOM does not know.
    Child,
    /// A child it contains is of such a kind.
    Unsupported,
    /// The queue of entries could not be stored.
    Capacity,
}

impl TestStorage {
    /// The entries of the child.
    pub(crate) const fn entries(&self) -> usize {
        self.entries
    }
    /// The source bytes of the child's origin.
    pub(crate) const fn bytes(&self) -> usize {
        self.bytes
    }
    /// The entries and source bytes of the child, or the refusal `limits` give it.
    pub(crate) fn check(&self, limits: &CssomLimits) -> Result<(usize, usize), CssomError> {
        if matches!(self.stop, Some(Stop::Child)) {
            return Err(unsupported_test_child());
        }
        if self.bytes > limits.max_input_bytes {
            return Err(CssomError::Limit {
                resource: "retained test input bytes",
                maximum: limits.max_input_bytes,
            });
        }
        if self.need > limits.max_entries {
            return Err(CssomError::Limit {
                resource: "test payload entries",
                maximum: limits.max_entries,
            });
        }
        match self.stop {
            None => Ok((self.entries, self.bytes)),
            Some(Stop::Child | Stop::Unsupported) => Err(unsupported_test_child()),
            Some(Stop::Capacity) => Err(CssomError::Limit {
                resource: "test payload capacity",
                maximum: limits.max_entries,
            }),
        }
    }
}

/// Count actual retained payload entries without interpreting CSS grammar or
/// estimating the native writer's tariff. Source bytes are charged per retained
/// direct occurrence, as a logical input bound rather than allocator accounting.
pub(crate) fn measure_test_child(child: CssSupportsTestChildRef<'_>) -> TestStorage {
    enum Entry<'a> {
        Child(CssSupportsTestChildRef<'a>),
        Component(&'a CssComponentValue),
    }
    let origin = match child {
        CssSupportsTestChildRef::Declaration(v) => v.origin(),
        CssSupportsTestChildRef::QualifiedRule(v) => v.origin(),
        CssSupportsTestChildRef::AtRule(v) => v.origin(),
        _ => {
            return TestStorage {
                bytes: 0,
                entries: 0,
                need: 0,
                stop: Some(Stop::Child),
            };
        }
    };
    let mut storage = TestStorage {
        bytes: test_input_bytes(origin),
        entries: 0,
        need: 0,
        stop: None,
    };
    let mut pending = Vec::new();
    fn push<'a>(
        pending: &mut Vec<Entry<'a>>,
        entry: Entry<'a>,
        storage: &mut TestStorage,
    ) -> Result<(), Stop> {
        // The queue and the entries counted must stay below the limit.
        storage.need = storage.need.max(
            pending
                .len()
                .saturating_add(storage.entries)
                .saturating_add(1),
        );
        pending.try_reserve(1).map_err(|_| Stop::Capacity)?;
        pending.push(entry);
        Ok(())
    }
    let walked = (|| {
        push(&mut pending, Entry::Child(child), &mut storage)?;
        while let Some(entry) = pending.pop() {
            storage.entries = storage.entries.saturating_add(1);
            storage.need = storage.need.max(storage.entries);
            let (values, body) = match entry {
                Entry::Child(CssSupportsTestChildRef::Declaration(v)) => (v.components(), None),
                Entry::Child(CssSupportsTestChildRef::QualifiedRule(v)) => {
                    (v.prelude().items(), Some(v.body()))
                }
                Entry::Child(CssSupportsTestChildRef::AtRule(v)) => {
                    push(&mut pending, Entry::Component(v.at_keyword()), &mut storage)?;
                    (v.prelude().items(), v.body())
                }
                Entry::Child(_) => return Err(Stop::Unsupported),
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
                push(&mut pending, Entry::Component(value), &mut storage)?;
            }
            if let Some(body) = body {
                storage.entries = storage
                    .entries
                    .saturating_add(body.local_recovery_origins().len());
                storage.need = storage.need.max(storage.entries);
                for child in body.children() {
                    push(&mut pending, Entry::Child(child), &mut storage)?;
                }
            }
        }
        Ok(())
    })();
    storage.stop = walked.err();
    storage
}

/// The bytes of the source a test input retains.
pub(crate) fn test_input_bytes(origin: &CssValueOrigin) -> usize {
    match origin {
        CssValueOrigin::Parsed(v) => v.source().as_str().len(),
        CssValueOrigin::ImplicitClosure { opening, .. } => opening.source().as_str().len(),
        _ => 0,
    }
}

pub(crate) fn retained_test_input_bytes(
    origin: &CssValueOrigin,
    limits: &CssomLimits,
) -> Result<usize, CssomError> {
    let bytes = test_input_bytes(origin);
    if bytes > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "retained test input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    Ok(bytes)
}
