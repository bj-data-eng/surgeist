use std::collections::{BTreeMap, BTreeSet};

use crate::{CssWideKeyword, Error, ErrorCode, Property, Result, Value};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CustomPropertyName {
    value: String,
}

impl CustomPropertyName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let suffix = value.strip_prefix("--").ok_or_else(invalid_name)?;
        if suffix.is_empty() || !suffix.chars().all(is_custom_property_suffix_char) {
            return Err(invalid_name());
        }

        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AuthoredTokens {
    value: String,
}

impl AuthoredTokens {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomPropertyValue {
    authored: AuthoredTokens,
    references: Vec<VariableReference>,
    dependencies: Vec<CustomPropertyName>,
    typed_values: BTreeMap<Property, CustomPropertyTypedValue>,
}

impl CustomPropertyValue {
    #[must_use]
    pub fn new(
        authored: AuthoredTokens,
        references: impl IntoIterator<Item = VariableReference>,
    ) -> Self {
        let references = references.into_iter().collect::<Vec<_>>();
        let dependencies = dependencies_from_references(&references);
        Self {
            authored,
            references,
            dependencies,
            typed_values: BTreeMap::new(),
        }
    }

    pub fn try_with_typed_value(
        mut self,
        property: Property,
        expression: VariableExpression,
    ) -> Result<Self> {
        self.try_push_typed_value(CustomPropertyTypedValue::try_new(property, expression)?)?;
        Ok(self)
    }

    pub fn try_push_typed_value(
        &mut self,
        typed_value: CustomPropertyTypedValue,
    ) -> Result<&mut Self> {
        typed_value
            .expression()
            .validate_for_property(typed_value.property())?;
        self.typed_values
            .insert(typed_value.property(), typed_value);
        self.dependencies = dependencies_from_references_and_typed_values(
            &self.references,
            self.typed_values.values(),
        );
        Ok(self)
    }

    #[must_use]
    pub const fn authored(&self) -> &AuthoredTokens {
        &self.authored
    }

    #[must_use]
    pub fn references(&self) -> &[VariableReference] {
        &self.references
    }

    #[must_use]
    pub fn dependencies(&self) -> &[CustomPropertyName] {
        &self.dependencies
    }

    #[must_use]
    pub fn typed_value(&self, property: Property) -> Option<&CustomPropertyTypedValue> {
        self.typed_values.get(&property)
    }
}

fn dependencies_from_references(references: &[VariableReference]) -> Vec<CustomPropertyName> {
    dependencies_from_references_and_typed_values(references, std::iter::empty())
}

fn dependencies_from_references_and_typed_values<'a>(
    references: &[VariableReference],
    typed_values: impl IntoIterator<Item = &'a CustomPropertyTypedValue>,
) -> Vec<CustomPropertyName> {
    let mut dependencies = BTreeSet::new();
    for reference in references {
        dependencies.insert(reference.name().clone());
        if let Some(fallback) = reference.fallback() {
            fallback
                .expression()
                .collect_dependencies(&mut dependencies);
        }
    }
    for typed_value in typed_values {
        typed_value
            .expression()
            .collect_dependencies(&mut dependencies);
    }
    dependencies.into_iter().collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomPropertyTypedValue {
    property: Property,
    expression: VariableExpression,
}

impl CustomPropertyTypedValue {
    pub fn try_new(property: Property, expression: VariableExpression) -> Result<Self> {
        expression.validate_for_property(property)?;
        Ok(Self {
            property,
            expression,
        })
    }

    #[must_use]
    pub const fn property(&self) -> Property {
        self.property
    }

    #[must_use]
    pub const fn expression(&self) -> &VariableExpression {
        &self.expression
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableExpression {
    Value(Value),
    CssWideKeyword(CssWideKeyword),
    Reference(VariableReference),
}

impl VariableExpression {
    #[must_use]
    pub fn dependencies(&self) -> Vec<CustomPropertyName> {
        let mut dependencies = BTreeSet::new();
        self.collect_dependencies(&mut dependencies);
        dependencies.into_iter().collect()
    }

    #[must_use]
    pub fn contains_reference(&self) -> bool {
        match self {
            Self::Value(_) | Self::CssWideKeyword(_) => false,
            Self::Reference(_) => true,
        }
    }

    pub fn validate_for_property(&self, property: Property) -> Result<()> {
        match self {
            Self::Value(value) => property.validate_value(value),
            Self::CssWideKeyword(_) => Ok(()),
            Self::Reference(reference) => {
                if let Some(fallback) = reference.fallback() {
                    fallback.expression().validate_for_property(property)?;
                }
                Ok(())
            }
        }
    }

    fn collect_dependencies(&self, dependencies: &mut BTreeSet<CustomPropertyName>) {
        match self {
            Self::Value(_) | Self::CssWideKeyword(_) => {}
            Self::Reference(reference) => {
                dependencies.insert(reference.name().clone());
                if let Some(fallback) = reference.fallback() {
                    fallback.expression().collect_dependencies(dependencies);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableReference {
    name: CustomPropertyName,
    fallback: Option<VariableFallback>,
}

impl VariableReference {
    #[must_use]
    pub const fn new(name: CustomPropertyName, fallback: Option<VariableFallback>) -> Self {
        Self { name, fallback }
    }

    #[must_use]
    pub const fn name(&self) -> &CustomPropertyName {
        &self.name
    }

    #[must_use]
    pub const fn fallback(&self) -> Option<&VariableFallback> {
        self.fallback.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableFallback {
    authored: AuthoredTokens,
    expression: Box<VariableExpression>,
}

impl VariableFallback {
    #[must_use]
    pub fn new(authored: AuthoredTokens, expression: VariableExpression) -> Self {
        Self {
            authored,
            expression: Box::new(expression),
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &AuthoredTokens {
        &self.authored
    }

    #[must_use]
    pub const fn expression(&self) -> &VariableExpression {
        &self.expression
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDependentValue {
    property: Property,
    authored: AuthoredTokens,
    expression: VariableExpression,
    dependencies: Vec<CustomPropertyName>,
}

impl VariableDependentValue {
    pub fn try_new(
        property: Property,
        authored: AuthoredTokens,
        expression: VariableExpression,
    ) -> Result<Self> {
        if !expression.contains_reference() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "variable-dependent values must contain at least one variable reference",
            ));
        }
        expression.validate_for_property(property)?;
        let dependencies = expression.dependencies();

        Ok(Self {
            property,
            authored,
            expression,
            dependencies,
        })
    }

    #[must_use]
    pub const fn property(&self) -> Property {
        self.property
    }

    #[must_use]
    pub const fn authored(&self) -> &AuthoredTokens {
        &self.authored
    }

    #[must_use]
    pub const fn expression(&self) -> &VariableExpression {
        &self.expression
    }

    #[must_use]
    pub fn dependencies(&self) -> &[CustomPropertyName] {
        &self.dependencies
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomPropertyDependencies {
    by_property: BTreeMap<Property, BTreeSet<CustomPropertyName>>,
}

impl CustomPropertyDependencies {
    pub fn for_property(&self, property: Property) -> impl Iterator<Item = &CustomPropertyName> {
        self.by_property.get(&property).into_iter().flatten()
    }

    pub fn properties_for_custom_property<'a>(
        &'a self,
        name: &'a CustomPropertyName,
    ) -> impl Iterator<Item = Property> + 'a {
        self.by_property
            .iter()
            .filter_map(move |(property, names)| names.contains(name).then_some(*property))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_property.is_empty()
    }

    pub(crate) fn insert(&mut self, property: Property, name: CustomPropertyName) {
        self.by_property.entry(property).or_default().insert(name);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomPropertyResolution {
    value: Option<CustomPropertyValue>,
    invalid: bool,
}

impl CustomPropertyResolution {
    #[must_use]
    pub(crate) fn valid(value: CustomPropertyValue) -> Self {
        Self {
            value: Some(value),
            invalid: false,
        }
    }

    #[must_use]
    pub(crate) const fn invalid() -> Self {
        Self {
            value: None,
            invalid: true,
        }
    }

    #[must_use]
    pub fn value(&self) -> Option<&CustomPropertyValue> {
        self.value.as_ref()
    }

    #[must_use]
    pub const fn is_invalid(&self) -> bool {
        self.invalid
    }
}

fn is_custom_property_suffix_char(value: char) -> bool {
    value.is_alphanumeric() || value == '-' || value == '_'
}

fn invalid_name() -> Error {
    Error::new(
        ErrorCode::InvalidString,
        "custom property names must start with -- and have a non-empty alphanumeric suffix using only alphanumeric characters, - or _",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, ErrorCode, Length, Property, StyleColor, Value};

    #[test]
    fn custom_property_name_preserves_case_and_accepts_css_custom_shape() {
        let name = CustomPropertyName::try_new("--BrandColor").unwrap();
        assert_eq!(name.as_str(), "--BrandColor");
        assert_eq!(
            CustomPropertyName::try_new("--brand_color-1")
                .unwrap()
                .as_str(),
            "--brand_color-1",
        );
    }

    #[test]
    fn custom_property_name_accepts_non_ascii_alphanumeric_suffix_chars() {
        let name = CustomPropertyName::try_new("--bränd").unwrap();

        assert_eq!(name.as_str(), "--bränd");
    }

    #[test]
    fn custom_property_name_rejects_non_custom_names() {
        for invalid in ["color", "-gap", "--", "-- bad", "--;", "--gap;", "--gap\n"] {
            let error = CustomPropertyName::try_new(invalid).unwrap_err();
            assert_eq!(error.code(), ErrorCode::InvalidString);
        }
    }

    #[test]
    fn custom_property_authored_tokens_preserve_empty_and_non_empty_css() {
        assert_eq!(AuthoredTokens::new("").as_css(), "");
        assert_eq!(
            AuthoredTokens::new("calc(var(--space) * 2)").as_css(),
            "calc(var(--space) * 2)"
        );
    }

    #[test]
    fn variable_reference_exposes_custom_property_name() {
        let name = CustomPropertyName::try_new("--space").unwrap();
        let reference = VariableReference::new(name.clone(), None);

        assert_eq!(reference.name(), &name);
        assert!(reference.fallback().is_none());
    }

    #[test]
    fn variable_fallback_preserves_authored_css() {
        let fallback = VariableFallback::new(
            AuthoredTokens::new("8px"),
            VariableExpression::Value(Value::Length(Length::px(8.0))),
        );

        assert_eq!(fallback.authored().as_css(), "8px");
        assert_eq!(
            fallback.expression(),
            &VariableExpression::Value(Value::Length(Length::px(8.0)))
        );
    }

    #[test]
    fn variable_nested_fallback_references_are_included_in_dependencies() {
        let expression = VariableExpression::Reference(VariableReference::new(
            CustomPropertyName::try_new("--space").unwrap(),
            Some(VariableFallback::new(
                AuthoredTokens::new("var(--fallback, 4px)"),
                VariableExpression::Reference(VariableReference::new(
                    CustomPropertyName::try_new("--fallback").unwrap(),
                    Some(VariableFallback::new(
                        AuthoredTokens::new("4px"),
                        VariableExpression::Value(Value::Length(Length::px(4.0))),
                    )),
                )),
            )),
        ));

        let dependencies = expression.dependencies();

        assert_eq!(dependencies.len(), 2);
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.as_str() == "--space")
        );
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.as_str() == "--fallback")
        );
    }

    #[test]
    fn variable_dependent_value_requires_at_least_one_reference() {
        let error = VariableDependentValue::try_new(
            Property::Width,
            AuthoredTokens::new("8px"),
            VariableExpression::Value(Value::Length(Length::px(8.0))),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn variable_dependent_value_validates_literal_branches_against_target_property() {
        let error = VariableDependentValue::try_new(
            Property::Width,
            AuthoredTokens::new("var(--space, black)"),
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--space").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);

        let value = VariableDependentValue::try_new(
            Property::Width,
            AuthoredTokens::new("var(--space, 8px)"),
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--space").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("8px"),
                    VariableExpression::Value(Value::Length(Length::px(8.0))),
                )),
            )),
        )
        .unwrap();

        assert_eq!(value.property(), Property::Width);
        assert_eq!(value.authored().as_css(), "var(--space, 8px)");
        assert_eq!(value.dependencies().len(), 1);
        assert_eq!(value.dependencies()[0].as_str(), "--space");
    }

    #[test]
    fn custom_property_value_preserves_authored_tokens_and_references() {
        let value = CustomPropertyValue::new(
            AuthoredTokens::new("var(--brand, black)"),
            [VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )],
        );

        assert_eq!(value.authored().as_css(), "var(--brand, black)");
        assert_eq!(value.references().len(), 1);
        assert_eq!(value.references()[0].name().as_str(), "--brand");
        assert_eq!(
            value.references()[0]
                .fallback()
                .unwrap()
                .authored()
                .as_css(),
            "black"
        );
        assert_eq!(value.dependencies()[0].as_str(), "--brand");
        assert!(value.typed_value(Property::Color).is_none());
    }

    #[test]
    fn custom_property_value_preserves_authored_nested_fallback_references() {
        let value = CustomPropertyValue::new(
            AuthoredTokens::new("var(--space, var(--fallback, 4px))"),
            [VariableReference::new(
                CustomPropertyName::try_new("--space").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("var(--fallback, 4px)"),
                    VariableExpression::Reference(VariableReference::new(
                        CustomPropertyName::try_new("--fallback").unwrap(),
                        Some(VariableFallback::new(
                            AuthoredTokens::new("4px"),
                            VariableExpression::Value(Value::Length(Length::px(4.0))),
                        )),
                    )),
                )),
            )],
        );

        assert_eq!(
            value.references()[0].fallback().unwrap().expression(),
            &VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--fallback").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("4px"),
                    VariableExpression::Value(Value::Length(Length::px(4.0))),
                )),
            ))
        );
        assert_eq!(value.dependencies().len(), 2);
        assert_eq!(value.dependencies()[0].as_str(), "--fallback");
        assert_eq!(value.dependencies()[1].as_str(), "--space");
    }

    #[test]
    fn custom_property_value_accepts_empty_authored_tokens() {
        let value = CustomPropertyValue::new(AuthoredTokens::new(""), []);

        assert_eq!(value.authored().as_css(), "");
        assert!(value.references().is_empty());
    }

    #[test]
    fn custom_property_typed_value_validates_literals_against_property() {
        let error = CustomPropertyTypedValue::try_new(
            Property::Width,
            VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);

        let typed = CustomPropertyTypedValue::try_new(
            Property::Width,
            VariableExpression::Value(Value::Length(Length::px(8.0))),
        )
        .unwrap();

        assert_eq!(typed.property(), Property::Width);
        assert_eq!(
            typed.expression(),
            &VariableExpression::Value(Value::Length(Length::px(8.0)))
        );
    }

    #[test]
    fn custom_property_typed_value_allows_references_and_builder_stores_by_property() {
        let typed = CustomPropertyTypedValue::try_new(
            Property::Color,
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )),
        )
        .unwrap();

        let mut value = CustomPropertyValue::new(
            AuthoredTokens::new("var(--brand, black)"),
            [VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                None,
            )],
        );
        value.try_push_typed_value(typed).unwrap();

        assert_eq!(
            value.typed_value(Property::Color).unwrap().property(),
            Property::Color
        );
        assert!(value.typed_value(Property::Width).is_none());
    }

    #[test]
    fn custom_property_value_authored_dependencies_include_typed_expression_references() {
        let mut value = CustomPropertyValue::new(AuthoredTokens::new("8px"), []);
        value
            .try_push_typed_value(
                CustomPropertyTypedValue::try_new(
                    Property::Width,
                    VariableExpression::Reference(VariableReference::new(
                        CustomPropertyName::try_new("--typed-only").unwrap(),
                        Some(VariableFallback::new(
                            AuthoredTokens::new("4px"),
                            VariableExpression::Value(Value::Length(Length::px(4.0))),
                        )),
                    )),
                )
                .unwrap(),
            )
            .unwrap();

        assert!(value.references().is_empty());
        assert_eq!(value.dependencies().len(), 1);
        assert_eq!(value.dependencies()[0].as_str(), "--typed-only");
    }

    #[test]
    fn custom_property_value_authored_dependencies_dedupe_authored_and_typed_references() {
        let mut value = CustomPropertyValue::new(
            AuthoredTokens::new("var(--shared, 8px)"),
            [VariableReference::new(
                CustomPropertyName::try_new("--shared").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("8px"),
                    VariableExpression::Value(Value::Length(Length::px(8.0))),
                )),
            )],
        );
        value
            .try_push_typed_value(
                CustomPropertyTypedValue::try_new(
                    Property::Width,
                    VariableExpression::Reference(VariableReference::new(
                        CustomPropertyName::try_new("--shared").unwrap(),
                        Some(VariableFallback::new(
                            AuthoredTokens::new("4px"),
                            VariableExpression::Value(Value::Length(Length::px(4.0))),
                        )),
                    )),
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(value.references().len(), 1);
        assert_eq!(value.dependencies().len(), 1);
        assert_eq!(value.dependencies()[0].as_str(), "--shared");
    }

    #[test]
    fn custom_property_dependencies_record_names_by_ordinary_property() {
        let brand = CustomPropertyName::try_new("--brand").unwrap();
        let space = CustomPropertyName::try_new("--space").unwrap();
        let mut dependencies = CustomPropertyDependencies::default();

        assert!(dependencies.is_empty());

        dependencies.insert(Property::Color, brand.clone());
        dependencies.insert(Property::Width, space.clone());
        dependencies.insert(Property::Color, space.clone());

        assert!(!dependencies.is_empty());
        assert_eq!(
            dependencies
                .for_property(Property::Color)
                .map(CustomPropertyName::as_str)
                .collect::<Vec<_>>(),
            ["--brand", "--space"]
        );
        assert_eq!(
            dependencies
                .for_property(Property::Width)
                .map(CustomPropertyName::as_str)
                .collect::<Vec<_>>(),
            ["--space"]
        );
        assert_eq!(
            dependencies
                .properties_for_custom_property(&space)
                .collect::<Vec<_>>(),
            [Property::Width, Property::Color]
        );
        assert!(
            dependencies
                .properties_for_custom_property(&CustomPropertyName::try_new("--unused").unwrap())
                .next()
                .is_none()
        );
    }
}
