//! Canonical specified serialization of authored named support tests.

use crate::{
    CssComponentValue, CssComponentValueRef, CssImportance, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    CssSupportsConditionRule, CssSupportsTestBody, CssSupportsTestDeclaration, CssSupportsTestItem,
    CssValueTokenRef, component_values::CssCanonicalBuilder,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

enum Event<'a> {
    Body(&'a CssSupportsTestBody, usize, usize),
    Declarations(&'a [CssSupportsTestDeclaration], usize, usize),
    Item(&'a CssSupportsTestItem, usize),
    Child(crate::CssSupportsTestChildRef<'a>, usize),
    Text(&'static str),
}

impl CssSupportsTestBody {
    /// Serializes test candidates without selecting feature truth values.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares one input, projection, and output budget across the complete body.
    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.named_supports_body(self)?;
        Ok(writer.css)
    }
}

impl CssSupportsConditionRule {
    /// Canonical specified definition text, including an explicit block closure.
    pub fn to_specified_css(&self) -> Result<String> {
        self.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    pub fn to_specified_css_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        writer.named_supports_rule(self)?;
        Ok(writer.css)
    }
}

impl SpecifiedRuleWriter {
    pub(crate) fn named_supports_rule(&mut self, rule: &CssSupportsConditionRule) -> Result<()> {
        self.named_supports_current(rule.name(), rule.body())
    }

    pub(crate) fn named_supports_current(
        &mut self,
        name: &crate::CssSupportsConditionName,
        body: &CssSupportsTestBody,
    ) -> Result<()> {
        self.check_graph_depth(self.graph_depth)
            .map_err(|e| e.at_origin(name.origin()))?;
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("@supports-condition ")?;
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append_identifier(name.as_str())?;
        self.append(" {")?;
        if !body.items().is_empty() {
            self.append(" ")?;
            self.write_named_tests(Event::Body(body, 0, self.child_depth(self.graph_depth)?))?;
        } else {
            self.context.charge_input(1)?;
            self.context.charge_projection(1)?;
        }
        self.append(" }")
    }

    pub(crate) fn named_supports_body(&mut self, body: &CssSupportsTestBody) -> Result<()> {
        self.write_named_tests(Event::Body(body, 0, self.graph_depth))
    }

    pub(crate) fn named_supports_child(
        &mut self,
        child: crate::CssSupportsTestChildRef<'_>,
    ) -> Result<()> {
        self.write_named_tests(Event::Child(child, self.graph_depth))
    }

    fn write_named_tests(&mut self, first: Event<'_>) -> Result<()> {
        let mut pending = vec![first];
        while let Some(event) = pending.pop() {
            match event {
                Event::Body(body, index, depth) => {
                    if index == 0 {
                        self.context.charge_input(1)?;
                        self.context.charge_projection(1)?;
                    }
                    if let Some(item) = body.items().get(index) {
                        push(&mut pending, Event::Body(body, index + 1, depth))?;
                        push(&mut pending, Event::Item(item, depth))?;
                        if index != 0 {
                            push(&mut pending, Event::Text(" "))?;
                        }
                    }
                }
                Event::Declarations(declarations, index, depth) => {
                    if let Some(declaration) = declarations.get(index) {
                        push(
                            &mut pending,
                            Event::Declarations(declarations, index + 1, depth),
                        )?;
                        if index != 0 {
                            self.append(" ")?;
                        }
                        self.check_graph_depth(depth)
                            .map_err(|e| e.at_origin(declaration.origin()))?;
                        self.graph_depth = depth;
                        self.named_supports_declaration(declaration)?;
                    }
                }
                Event::Item(item, depth) => match item {
                    CssSupportsTestItem::Declarations(run) => {
                        self.context.charge_input(1)?;
                        self.context.charge_projection(1)?;
                        push(
                            &mut pending,
                            Event::Declarations(run.declarations(), 0, depth),
                        )?;
                    }
                    CssSupportsTestItem::QualifiedRule(v) => push(
                        &mut pending,
                        Event::Child(crate::CssSupportsTestChildRef::QualifiedRule(v), depth),
                    )?,
                    CssSupportsTestItem::AtRule(v) => push(
                        &mut pending,
                        Event::Child(crate::CssSupportsTestChildRef::AtRule(v), depth),
                    )?,
                },
                Event::Child(child, depth) => {
                    self.check_graph_depth(depth)
                        .map_err(|e| e.at_origin(child.origin()))?;
                    self.graph_depth = depth;
                    if let crate::CssSupportsTestChildRef::Declaration(v) = child {
                        self.named_supports_declaration(v)?;
                        continue;
                    }
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    let body = match child {
                        crate::CssSupportsTestChildRef::QualifiedRule(rule) => {
                            self.append_components(rule.prelude().items())?;
                            self.append(if trim_whitespace(rule.prelude().items()).is_empty() {
                                "{"
                            } else {
                                " {"
                            })?;
                            Some(rule.body())
                        }
                        crate::CssSupportsTestChildRef::AtRule(rule) => {
                            self.append_components(std::slice::from_ref(rule.at_keyword()))?;
                            if !rule.prelude().items().is_empty() {
                                self.append(" ")?;
                                self.append_components(rule.prelude().items())?;
                            }
                            if rule.body().is_some() {
                                self.append(" {")?;
                            } else {
                                self.append(";")?;
                            }
                            rule.body()
                        }
                        crate::CssSupportsTestChildRef::Declaration(_) => {
                            unreachable!("handled above")
                        }
                    };
                    if let Some(body) = body {
                        if body.items().is_empty() {
                            self.context.charge_input(1)?;
                            self.context.charge_projection(1)?;
                            self.append(" }")?;
                        } else {
                            self.append(" ")?;
                            push(&mut pending, Event::Text(" }"))?;
                            push(&mut pending, Event::Body(body, 0, self.child_depth(depth)?))?;
                        }
                    }
                }
                Event::Text(text) => self.append(text)?,
            }
        }
        Ok(())
    }

    pub(crate) fn check_graph_depth(&self, depth: usize) -> Result<()> {
        if self.max_graph_depth.is_some_and(|limit| depth > limit) {
            return Err(CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::DepthLimit,
            ));
        }
        Ok(())
    }
    /// Specialized native rule children share their enclosing graph owner.
    /// Value-only serialization keeps its existing independent quota contract.
    pub(crate) fn check_graph_child_depth(&self) -> Result<()> {
        if self.max_graph_depth.is_some() {
            self.check_graph_depth(self.child_depth(self.graph_depth)?)?;
        }
        Ok(())
    }
    fn child_depth(&self, depth: usize) -> Result<usize> {
        depth.checked_add(1).ok_or_else(capacity_error)
    }
    fn check_named_component_depth(&self, values: &[CssComponentValue]) -> Result<()> {
        if self.max_graph_depth.is_none() {
            return Ok(());
        }
        let mut pending: Vec<_> = values.iter().map(|v| (v, self.graph_depth)).collect();
        while let Some((value, depth)) = pending.pop() {
            self.check_graph_depth(depth)
                .map_err(|e| e.at_origin(value.origin()))?;
            if let Some(children) = value.child_values() {
                let depth = self.child_depth(depth)?;
                self.check_graph_depth(depth)
                    .map_err(|e| e.at_origin(value.origin()))?;
                pending.extend(children.items().iter().map(|v| (v, depth)));
            }
        }
        Ok(())
    }

    fn named_supports_declaration(
        &mut self,
        declaration: &CssSupportsTestDeclaration,
    ) -> Result<()> {
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        let full = component_count(declaration.components())?;
        let value = component_count(declaration.value_components())?;
        self.context
            .charge_input(full.checked_sub(value).ok_or_else(capacity_error)?)?;
        self.context.charge_projection(1)?;
        self.append_identifier(declaration.property())?;
        self.append(": ")?;
        self.append_components(declaration.value_components())?;
        if declaration.importance() == CssImportance::Important {
            self.append(" !important")?;
        }
        self.append(";")
    }

    fn append_components(&mut self, components: &[CssComponentValue]) -> Result<()> {
        let count = component_count(components)?;
        self.context.charge_input(count)?;
        self.context.charge_projection(count)?;
        self.check_named_component_depth(components)?;
        let components = trim_whitespace(components);
        if components.is_empty() {
            return Ok(());
        }
        let mut builder = CssCanonicalBuilder::new(self.context.remaining_bytes());
        builder
            .push_components(components)
            .map_err(crate::component_values::specified_component_error)?;
        let serialized = builder
            .finish()
            .map_err(crate::component_values::specified_component_error)?;
        self.append(serialized.as_css())
    }
}

fn component_count(components: &[CssComponentValue]) -> Result<usize> {
    components.iter().try_fold(0usize, |count, component| {
        let descendants = match component.view() {
            CssComponentValueRef::Function(value) => value.values().component_count(),
            CssComponentValueRef::Block(value) => value.values().component_count(),
            _ => 0,
        };
        count
            .checked_add(1)
            .and_then(|value| value.checked_add(descendants))
            .ok_or_else(capacity_error)
    })
}

fn push<'a>(pending: &mut Vec<Event<'a>>, event: Event<'a>) -> Result<()> {
    pending.try_reserve(1).map_err(|_| capacity_error())?;
    pending.push(event);
    Ok(())
}

fn trim_whitespace(mut components: &[CssComponentValue]) -> &[CssComponentValue] {
    while components.first().is_some_and(is_whitespace) {
        components = &components[1..];
    }
    while components.last().is_some_and(is_whitespace) {
        components = &components[..components.len() - 1];
    }
    components
}

fn is_whitespace(component: &CssComponentValue) -> bool {
    matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}

fn capacity_error() -> CssSpecifiedValueSerializationError {
    CssSpecifiedValueSerializationError::new(
        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
    )
}

impl crate::CssSupportsTestChildRef<'_> {
    /// Standalone child root is depth zero. Nested payload/components and the
    /// complete output share the same native writer allowances.
    pub fn to_specified_css_with_graph_limits(
        self,
        limits: crate::CssRuleGraphSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits.values);
        writer.max_graph_depth = Some(limits.max_depth);
        writer.named_supports_child(self)?;
        Ok(writer.css)
    }
    pub fn to_specified_css(self) -> Result<String> {
        self.to_specified_css_with_graph_limits(crate::CssRuleGraphSerializationLimits::default())
    }
}
impl CssSupportsTestBody {
    /// A standalone body has direct children at depth zero; containers/runs are
    /// transparent. Use a named graph input to include the outer wrapper depth.
    pub fn to_specified_css_with_graph_limits(
        &self,
        limits: crate::CssRuleGraphSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits.values);
        writer.max_graph_depth = Some(limits.max_depth);
        writer.named_supports_body(self)?;
        Ok(writer.css)
    }
}
