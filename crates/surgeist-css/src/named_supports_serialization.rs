//! Canonical specified serialization of authored named support tests.

use crate::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef, CssImportance,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssSupportsConditionRule, CssSupportsTestBody,
    CssSupportsTestDeclaration, CssSupportsTestItem, CssValueTokenRef,
    component_values::CssCanonicalBuilder, specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

enum Event<'a> {
    Body(&'a CssSupportsTestBody, usize),
    Declarations(&'a [CssSupportsTestDeclaration], usize),
    Item(&'a CssSupportsTestItem),
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
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append("@supports-condition ")?;
        self.context.charge_input(1)?;
        self.context.charge_projection(1)?;
        self.append_identifier(rule.name().as_str())?;
        self.append(" {")?;
        if !rule.body().items().is_empty() {
            self.append(" ")?;
            self.named_supports_body(rule.body())?;
        } else {
            self.context.charge_input(1)?;
            self.context.charge_projection(1)?;
        }
        self.append(" }")
    }

    pub(crate) fn named_supports_body(&mut self, body: &CssSupportsTestBody) -> Result<()> {
        let mut pending = vec![Event::Body(body, 0)];
        while let Some(event) = pending.pop() {
            match event {
                Event::Body(body, index) => {
                    if index == 0 {
                        self.context.charge_input(1)?;
                        self.context.charge_projection(1)?;
                    }
                    if let Some(item) = body.items().get(index) {
                        push(&mut pending, Event::Body(body, index + 1))?;
                        push(&mut pending, Event::Item(item))?;
                        if index != 0 {
                            push(&mut pending, Event::Text(" "))?;
                        }
                    }
                }
                Event::Declarations(declarations, index) => {
                    if let Some(declaration) = declarations.get(index) {
                        push(&mut pending, Event::Declarations(declarations, index + 1))?;
                        if index != 0 {
                            self.append(" ")?;
                        }
                        self.named_supports_declaration(declaration)?;
                    }
                }
                Event::Item(item) => {
                    self.context.charge_input(1)?;
                    self.context.charge_projection(1)?;
                    match item {
                        CssSupportsTestItem::Declarations(run) => {
                            push(&mut pending, Event::Declarations(run.declarations(), 0))?;
                        }
                        CssSupportsTestItem::QualifiedRule(rule) => {
                            self.append_components(rule.prelude().items())?;
                            self.append(if trim_whitespace(rule.prelude().items()).is_empty() {
                                "{"
                            } else {
                                " {"
                            })?;
                            if !rule.body().items().is_empty() {
                                self.append(" ")?;
                                push(&mut pending, Event::Text(" }"))?;
                                push(&mut pending, Event::Body(rule.body(), 0))?;
                            } else {
                                self.context.charge_input(1)?;
                                self.context.charge_projection(1)?;
                                self.append(" }")?;
                            }
                        }
                        CssSupportsTestItem::AtRule(rule) => {
                            self.append_components(std::slice::from_ref(rule.at_keyword()))?;
                            if !rule.prelude().items().is_empty() {
                                self.append(" ")?;
                                self.append_components(rule.prelude().items())?;
                            }
                            if let Some(body) = rule.body() {
                                self.append(" {")?;
                                if !body.items().is_empty() {
                                    self.append(" ")?;
                                    push(&mut pending, Event::Text(" }"))?;
                                    push(&mut pending, Event::Body(body, 0))?;
                                } else {
                                    self.context.charge_input(1)?;
                                    self.context.charge_projection(1)?;
                                    self.append(" }")?;
                                }
                            } else {
                                self.append(";")?;
                            }
                        }
                    }
                }
                Event::Text(text) => self.append(text)?,
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
        let components = trim_whitespace(components);
        if components.is_empty() {
            return Ok(());
        }
        let mut builder = CssCanonicalBuilder::new(self.context.remaining_bytes());
        builder
            .push_components(components)
            .map_err(component_error)?;
        let serialized = builder.finish().map_err(component_error)?;
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

fn component_error(error: crate::CssComponentValueError) -> CssSpecifiedValueSerializationError {
    let kind = match error.kind() {
        CssComponentValueErrorKind::ByteLimit => CssSpecifiedValueSerializationErrorKind::ByteLimit,
        _ => CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
    };
    CssSpecifiedValueSerializationError::new(kind)
}
