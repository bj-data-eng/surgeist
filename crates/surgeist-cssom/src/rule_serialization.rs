//! Bounded preparation and native serialization of current live rule payloads.
use crate::{model::State, *};
use std::cell::OnceCell;
use surgeist_css::CssRule;
use surgeist_css::*;

/// Limits one current rule text read. Preparation counts actual context/traversal
/// steps and view nodes; CSS independently owns its one cumulative graph budget.
/// These bound work and owned buffers, not allocator overhead or process RSS.
#[derive(Clone, Debug)]
pub struct CssomRuleFormatLimits {
    pub max_nodes: usize,
    pub max_preparation_steps: usize,
    pub max_depth: usize,
    pub css: CssSerializationLimits,
}
impl Default for CssomRuleFormatLimits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_preparation_steps: 1_000_000,
            max_depth: 256,
            css: CssSerializationLimits::default(),
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
impl State {
    fn current_properties(
        &self,
        id: &CssomBlockId,
    ) -> Result<&CssSpecifiedDeclarationBlock, CssomError> {
        match self.block(id)?.data() {
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
        match self.block(id)?.data() {
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
        match self.rule(id)?.data() {
            CssomRuleData::Keyframe { selectors, block } => {
                CssKeyframeRuleView::try_new(selectors, self.current_properties(block)?)
                    .map_err(CssomError::Keyframe)
            }
            _ => Err(CssomError::WrongKind),
        }
    }
    fn margin_view(&self, id: &CssomRuleId) -> Result<CssMarginRuleView<'_>, CssomError> {
        match self.rule(id)?.data() {
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
    pub(crate) fn format_rule_text(
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
        if matches!(root.data(), CssomRuleData::Keyframe { .. }) {
            return self
                .keyframe_view(id)?
                .serialize_cssom(limits.css)
                .map_err(CssomError::Keyframe);
        }
        if matches!(root.data(), CssomRuleData::Margin { .. }) {
            return self
                .margin_view(id)?
                .serialize_cssom(limits.css)
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
                    if matches!(rule.data(), CssomRuleData::Style { .. }) {
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
        pending.push((id, None::<usize>, depth));
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
            let children = match rule.data() {
                CssomRuleData::Style { children, .. }
                | CssomRuleData::Page { children, .. }
                | CssomRuleData::NamedSupports { children, .. }
                | CssomRuleData::Keyframes { children, .. } => Some(children),
                CssomRuleData::Group {
                    prelude: CssomGroupPrelude::Media(_) | CssomGroupPrelude::Supports(_),
                    children,
                } => Some(children),
                _ => None,
            };
            if let Some(children) = children {
                for child in self
                    .lists
                    .get(children)
                    .ok_or(CssomError::MissingObject)?
                    .iter()
                    .rev()
                {
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
                rule.data(),
                CssomRuleData::Margin { .. } | CssomRuleData::Keyframe { .. }
            ) {
                continue;
            }
            let input = match rule.data() {
                CssomRuleData::Style { .. } | CssomRuleData::Group { .. } => {
                    let mut current = Vec::new();
                    for child in child_indexes {
                        work.step()?;
                        format_reserve(&mut current, 1)?;
                        current.push(*views[*child].get().ok_or(CssomError::WrongKind)?);
                    }
                    children[index].set(current).expect("single assignment");
                    let current = children[index].get().expect("initialized");
                    match rule.data() {
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
                CssomRuleData::NamedSupports { name, body, .. } => {
                    CssRuleGraphInput::named_supports(name, body)
                }
                CssomRuleData::SupportsTestChild(child) => {
                    CssRuleGraphInput::supports_test_child(child.as_ref())
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
            .serialize_cssom(
                namespaces,
                ancestor,
                CssRuleSerializationLimits {
                    values: limits.css,
                    max_depth: limits.max_depth.saturating_sub(depth),
                },
            )
            .map_err(CssomError::Format)
    }
}

impl CssomSnapshot {
    /// Formats committed current payloads under one bounded native CSS operation.
    pub(crate) fn rule_css_text(
        &self,
        id: &CssomRuleId,
        limits: &CssomRuleFormatLimits,
    ) -> Result<String, CssomError> {
        self.state.format_rule_text(id, limits)
    }
}
impl CssomBatch {
    /// Formats staged current payloads for an existing committed ID, without
    /// publishing state or exposing tentative identities. A poisoned batch
    /// returns BatchAborted. The same bounded writer serves committed captures.
    pub fn rule_css_text(
        &self,
        id: &CssomRuleId,
        limits: &CssomRuleFormatLimits,
    ) -> Result<String, CssomError> {
        if self.poisoned {
            return Err(CssomError::BatchAborted);
        }
        self.base.rule(id)?;
        self.staged.format_rule_text(id, limits)
    }
}
