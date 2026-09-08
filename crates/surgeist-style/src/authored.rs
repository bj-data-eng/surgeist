use crate::{
    CustomPropertyName, CustomPropertyValue, Declaration, Error, ErrorCode, Property, Result,
    StyleSourceId, Value, VariableDependentValue,
    declaration::{canonical_declarations, canonical_properties},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AuthoredProperty {
    Property(Property),
    Custom(CustomPropertyName),
    All,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssWideKeyword {
    Inherit,
    Initial,
    Unset,
    RevertLayer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthoredValue {
    Value(Value),
    CssWideKeyword(CssWideKeyword),
    CustomProperty(CustomPropertyValue),
    VariableDependent(VariableDependentValue),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredDeclaration {
    property: AuthoredProperty,
    value: AuthoredValue,
    source: Option<StyleSourceId>,
}

impl AuthoredDeclaration {
    pub fn try_new(property: AuthoredProperty, value: AuthoredValue) -> Result<Self> {
        match (&property, &value) {
            (AuthoredProperty::Property(style_property), AuthoredValue::Value(style_value)) => {
                if matches!(style_value, Value::Keyword(_)) {
                    return Err(invalid_property(
                        "legacy keyword values are not valid authored declaration values",
                    ));
                }
                style_property.validate_value(style_value)?;
            }
            (
                AuthoredProperty::Property(style_property),
                AuthoredValue::VariableDependent(variable_value),
            ) => {
                if variable_value.property() != *style_property {
                    return Err(invalid_property(
                        "variable-dependent values must target the authored property",
                    ));
                }
            }
            (AuthoredProperty::Custom(_), AuthoredValue::CustomProperty(_)) => {}
            (AuthoredProperty::Property(_), AuthoredValue::CssWideKeyword(_))
            | (AuthoredProperty::Custom(_), AuthoredValue::CssWideKeyword(_))
            | (AuthoredProperty::All, AuthoredValue::CssWideKeyword(_)) => {
                return Err(invalid_property(
                    "CSS-wide keywords must use AuthoredDeclaration::css_wide",
                ));
            }
            (AuthoredProperty::All, _) => {
                return Err(invalid_property(
                    "all accepts only explicit authored CSS-wide keywords",
                ));
            }
            (AuthoredProperty::Custom(_), _) => {
                return Err(invalid_property(
                    "custom properties accept only custom property values or explicit CSS-wide keywords",
                ));
            }
            (AuthoredProperty::Property(_), AuthoredValue::CustomProperty(_)) => {
                return Err(invalid_property(
                    "ordinary properties cannot accept custom property values",
                ));
            }
        }

        Ok(Self {
            property,
            value,
            source: None,
        })
    }

    #[must_use]
    pub fn css_wide(property: AuthoredProperty, keyword: CssWideKeyword) -> Self {
        Self {
            property,
            value: AuthoredValue::CssWideKeyword(keyword),
            source: None,
        }
    }

    #[must_use]
    pub fn with_source(mut self, source: StyleSourceId) -> Self {
        self.source = Some(source);
        self
    }

    #[must_use]
    pub fn property(&self) -> AuthoredProperty {
        self.property.clone()
    }

    #[must_use]
    pub fn value(&self) -> AuthoredValue {
        self.value.clone()
    }

    #[must_use]
    pub const fn source(&self) -> Option<StyleSourceId> {
        self.source
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthoredDeclarations {
    values: Vec<AuthoredDeclaration>,
}

impl AuthoredDeclarations {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, declaration: AuthoredDeclaration) -> &mut Self {
        self.values.push(declaration);
        self
    }

    pub fn try_push(&mut self, declaration: AuthoredDeclaration) -> Result<&mut Self> {
        Ok(self.push(declaration))
    }

    pub fn iter(&self) -> impl Iterator<Item = &AuthoredDeclaration> {
        self.values.iter()
    }

    #[must_use]
    pub fn with_source(mut self, source: StyleSourceId) -> Self {
        for declaration in &mut self.values {
            declaration.source = Some(source);
        }
        self
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn to_rule_declarations(&self) -> Result<AuthoredCanonicalDeclarations> {
        let mut canonical = AuthoredCanonicalDeclarations::new();
        for declaration in &self.values {
            match (&declaration.property, &declaration.value) {
                (AuthoredProperty::Property(property), AuthoredValue::Value(value)) => {
                    for Declaration { property, value } in
                        canonical_declarations(*property, value.clone())
                    {
                        canonical.insert_property(
                            property,
                            AuthoredCascadeValue::Value(value),
                            declaration.source,
                        );
                    }
                }
                (AuthoredProperty::Property(property), AuthoredValue::CssWideKeyword(keyword)) => {
                    for property in canonical_properties(*property) {
                        canonical.insert_property(
                            property,
                            AuthoredCascadeValue::CssWideKeyword(*keyword),
                            declaration.source,
                        );
                    }
                }
                (AuthoredProperty::Property(property), AuthoredValue::VariableDependent(value)) => {
                    canonical.insert_property(
                        *property,
                        AuthoredCascadeValue::VariableDependent(value.clone()),
                        declaration.source,
                    );
                }
                (AuthoredProperty::Property(_), AuthoredValue::CustomProperty(_)) => {
                    return Err(invalid_property(
                        "ordinary properties cannot accept custom property values",
                    ));
                }
                (AuthoredProperty::Custom(name), AuthoredValue::CustomProperty(value)) => {
                    canonical.insert_custom(
                        name.clone(),
                        CustomPropertyCascadeValue::Value(value.clone()),
                        declaration.source,
                    );
                }
                (AuthoredProperty::Custom(name), AuthoredValue::CssWideKeyword(keyword)) => {
                    canonical.insert_custom(
                        name.clone(),
                        CustomPropertyCascadeValue::CssWideKeyword(*keyword),
                        declaration.source,
                    );
                }
                (AuthoredProperty::Custom(_), AuthoredValue::Value(_))
                | (AuthoredProperty::Custom(_), AuthoredValue::VariableDependent(_)) => {
                    return Err(invalid_property(
                        "custom properties accept only custom property values or explicit CSS-wide keywords",
                    ));
                }
                (AuthoredProperty::All, AuthoredValue::CssWideKeyword(keyword)) => {
                    for property in all_css_wide_properties() {
                        canonical.insert_property(
                            property,
                            AuthoredCascadeValue::CssWideKeyword(*keyword),
                            declaration.source,
                        );
                    }
                }
                (AuthoredProperty::All, _) => {
                    return Err(invalid_property(
                        "all accepts only explicit authored CSS-wide keywords",
                    ));
                }
            }
        }
        Ok(canonical)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AuthoredCascadeValue {
    Value(Value),
    CssWideKeyword(CssWideKeyword),
    VariableDependent(VariableDependentValue),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CustomPropertyCascadeValue {
    Value(CustomPropertyValue),
    CssWideKeyword(CssWideKeyword),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AuthoredDeclarationItem {
    Property(Property, AuthoredCascadeValue, Option<StyleSourceId>),
    Custom(
        CustomPropertyName,
        CustomPropertyCascadeValue,
        Option<StyleSourceId>,
    ),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AuthoredCanonicalDeclarations {
    values: Vec<AuthoredDeclarationItem>,
}

#[allow(dead_code)]
impl AuthoredCanonicalDeclarations {
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    fn insert_property(
        &mut self,
        property: Property,
        value: AuthoredCascadeValue,
        source: Option<StyleSourceId>,
    ) {
        if let Some(AuthoredDeclarationItem::Property(_, existing, existing_source)) =
            self.values.iter_mut().find(|item| match item {
                AuthoredDeclarationItem::Property(existing_property, _, _) => {
                    *existing_property == property
                }
                AuthoredDeclarationItem::Custom(_, _, _) => false,
            })
        {
            *existing = value;
            *existing_source = source;
        } else {
            self.values
                .push(AuthoredDeclarationItem::Property(property, value, source));
        }
    }

    fn insert_custom(
        &mut self,
        name: CustomPropertyName,
        value: CustomPropertyCascadeValue,
        source: Option<StyleSourceId>,
    ) {
        if let Some(AuthoredDeclarationItem::Custom(_, existing, existing_source)) =
            self.values.iter_mut().find(|item| match item {
                AuthoredDeclarationItem::Property(_, _, _) => false,
                AuthoredDeclarationItem::Custom(existing_name, _, _) => *existing_name == name,
            })
        {
            *existing = value;
            *existing_source = source;
        } else {
            self.values
                .push(AuthoredDeclarationItem::Custom(name, value, source));
        }
    }

    #[must_use]
    pub(crate) fn get(&self, property: Property) -> Option<&AuthoredCascadeValue> {
        self.values.iter().find_map(|item| match item {
            AuthoredDeclarationItem::Property(existing_property, value, _)
                if *existing_property == property =>
            {
                Some(value)
            }
            AuthoredDeclarationItem::Property(_, _, _)
            | AuthoredDeclarationItem::Custom(_, _, _) => None,
        })
    }

    #[must_use]
    pub(crate) fn source(&self, property: Property) -> Option<StyleSourceId> {
        self.values.iter().find_map(|item| match item {
            AuthoredDeclarationItem::Property(existing_property, _, source)
                if *existing_property == property =>
            {
                *source
            }
            AuthoredDeclarationItem::Property(_, _, _)
            | AuthoredDeclarationItem::Custom(_, _, _) => None,
        })
    }

    #[must_use]
    pub(crate) fn get_custom(
        &self,
        name: &CustomPropertyName,
    ) -> Option<&CustomPropertyCascadeValue> {
        self.values.iter().find_map(|item| match item {
            AuthoredDeclarationItem::Custom(existing_name, value, _) if existing_name == name => {
                Some(value)
            }
            AuthoredDeclarationItem::Property(_, _, _)
            | AuthoredDeclarationItem::Custom(_, _, _) => None,
        })
    }

    #[must_use]
    pub(crate) fn custom_source(&self, name: &CustomPropertyName) -> Option<StyleSourceId> {
        self.values.iter().find_map(|item| match item {
            AuthoredDeclarationItem::Custom(existing_name, _, source) if existing_name == name => {
                *source
            }
            AuthoredDeclarationItem::Property(_, _, _)
            | AuthoredDeclarationItem::Custom(_, _, _) => None,
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &AuthoredDeclarationItem> {
        self.values.iter()
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub(crate) fn property_len(&self) -> usize {
        self.values
            .iter()
            .filter(|item| matches!(item, AuthoredDeclarationItem::Property(_, _, _)))
            .count()
    }

    #[must_use]
    pub(crate) fn custom_len(&self) -> usize {
        self.values
            .iter()
            .filter(|item| matches!(item, AuthoredDeclarationItem::Custom(_, _, _)))
            .count()
    }

    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[allow(dead_code)]
fn all_css_wide_properties() -> impl Iterator<Item = Property> {
    Property::ALL
        .iter()
        .copied()
        .filter(|property| property.is_canonical() && *property != Property::Direction)
}

fn invalid_property(message: impl Into<String>) -> Error {
    Error::new(ErrorCode::InvalidProperty, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthoredTokens, Color, CustomPropertyName, CustomPropertyValue, Display, ErrorCode,
        Keyword, Length, Property, StyleColor, StyleSourceId, Value, VariableDependentValue,
        VariableExpression, VariableFallback, VariableReference,
    };

    #[test]
    fn ordinary_authored_declaration_validates_against_property() {
        let declaration = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Display),
            AuthoredValue::Value(Value::Display(Display::Block)),
        )
        .unwrap();

        assert_eq!(
            declaration.property(),
            AuthoredProperty::Property(Property::Display)
        );
    }

    #[test]
    fn ordinary_authored_declaration_rejects_property_value_mismatch() {
        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Width),
            AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn all_rejects_ordinary_values() {
        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::All,
            AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn ordinary_value_path_rejects_existing_keyword_values() {
        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Color),
            AuthoredValue::Value(Value::Keyword(Keyword::Initial)),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn css_wide_keywords_must_use_explicit_constructor() {
        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Color),
            AuthoredValue::CssWideKeyword(CssWideKeyword::Initial),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn all_accepts_css_wide_keywords() {
        let declaration =
            AuthoredDeclaration::css_wide(AuthoredProperty::All, CssWideKeyword::Initial);

        assert_eq!(declaration.property(), AuthoredProperty::All);
        assert_eq!(
            declaration.value(),
            AuthoredValue::CssWideKeyword(CssWideKeyword::Initial)
        );
    }

    #[test]
    fn authored_declarations_preserve_source_ids() {
        let source = StyleSourceId::try_new(5).unwrap();
        let declaration = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Color),
            AuthoredValue::Value(Value::StyleColor(StyleColor::current_color())),
        )
        .unwrap()
        .with_source(source);

        assert_eq!(declaration.source(), Some(source));

        let mut declarations = AuthoredDeclarations::new();
        declarations.push(declaration);
        let declarations = declarations.with_source(source);

        assert!(
            declarations
                .iter()
                .all(|item| item.source() == Some(source))
        );
    }

    #[test]
    fn authored_canonical_declarations_preserve_replacement_source_ids() {
        let first = StyleSourceId::try_new(5).unwrap();
        let second = StyleSourceId::try_new(6).unwrap();
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Color),
                    AuthoredValue::Value(Value::StyleColor(StyleColor::current_color())),
                )
                .unwrap()
                .with_source(first),
            )
            .unwrap();
        declarations
            .try_push(
                AuthoredDeclaration::css_wide(
                    AuthoredProperty::Property(Property::Color),
                    CssWideKeyword::Unset,
                )
                .with_source(second),
            )
            .unwrap();
        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.source(Property::Color), Some(second));
    }

    #[test]
    fn all_expands_css_wide_keyword_to_canonical_properties_except_direction() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::All,
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(
            canonical.len(),
            Property::ALL
                .iter()
                .filter(|property| property.is_canonical() && **property != Property::Direction)
                .count()
        );
        assert_eq!(
            canonical.get(Property::Color),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(canonical.get(Property::Direction), None);
    }

    #[test]
    fn shorthand_css_wide_keyword_expands_to_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Overflow),
            CssWideKeyword::Inherit,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(
            canonical.get(Property::OverflowX),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Inherit
            ))
        );
        assert_eq!(
            canonical.get(Property::OverflowY),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Inherit
            ))
        );
    }

    #[test]
    fn transition_css_wide_keyword_expands_to_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Transition),
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Transition), None);
        assert_eq!(
            canonical.get(Property::TransitionProperty),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::TransitionDuration),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::TransitionDelay),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::TransitionTimingFunction),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
    }

    #[test]
    fn animation_css_wide_keyword_expands_to_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Animation),
            CssWideKeyword::RevertLayer,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Animation), None);
        for property in [
            Property::AnimationName,
            Property::AnimationDuration,
            Property::AnimationDelay,
            Property::AnimationTimingFunction,
            Property::AnimationIterationCount,
            Property::AnimationDirection,
            Property::AnimationFillMode,
            Property::AnimationPlayState,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::RevertLayer
                ))
            );
        }
    }

    #[test]
    fn text_decoration_css_wide_keyword_expands_to_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::TextDecoration),
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();
        assert_eq!(canonical.get(Property::TextDecoration), None);
        assert_eq!(
            canonical.get(Property::TextDecorationLine),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::TextDecorationStyle),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::TextDecorationThickness),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
    }

    #[test]
    fn list_style_css_wide_keyword_expands_to_marker_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::ListStyle),
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::ListStyle), None);
        for property in [
            Property::ListStyleType,
            Property::ListStylePosition,
            Property::ListStyleImage,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
            );
        }
    }

    #[test]
    fn new_layout_shorthands_expand_css_wide_keywords_to_canonical_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Flex),
            CssWideKeyword::RevertLayer,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::PlaceContent),
            CssWideKeyword::Unset,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Margin),
            CssWideKeyword::Initial,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();
        assert_eq!(canonical.get(Property::Flex), None);
        assert_eq!(
            canonical.get(Property::FlexGrow),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::RevertLayer
            ))
        );
        assert_eq!(
            canonical.get(Property::FlexShrink),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::RevertLayer
            ))
        );
        assert_eq!(
            canonical.get(Property::FlexBasis),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::RevertLayer
            ))
        );
        assert_eq!(canonical.get(Property::PlaceContent), None);
        assert_eq!(
            canonical.get(Property::AlignContent),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(
            canonical.get(Property::JustifyContent),
            Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
        );
        assert_eq!(canonical.get(Property::Margin), None);
        assert_eq!(
            canonical.get(Property::MarginTop),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Initial
            ))
        );
        assert_eq!(
            canonical.get(Property::MarginRight),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Initial
            ))
        );
        assert_eq!(
            canonical.get(Property::MarginBottom),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Initial
            ))
        );
        assert_eq!(
            canonical.get(Property::MarginLeft),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::Initial
            ))
        );
    }

    #[test]
    fn text_shorthands_expand_css_wide_keywords_to_canonical_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Font),
            CssWideKeyword::RevertLayer,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::TextDecoration),
            CssWideKeyword::Initial,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();
        assert_eq!(canonical.get(Property::Font), None);
        for property in [
            Property::FontStyle,
            Property::FontVariant,
            Property::FontWeight,
            Property::FontStretch,
            Property::FontSize,
            Property::LineHeight,
            Property::FontFamily,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::RevertLayer
                ))
            );
        }

        assert_eq!(canonical.get(Property::TextDecoration), None);
        for property in [
            Property::TextDecorationLine,
            Property::TextDecorationStyle,
            Property::TextDecorationThickness,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Initial
                ))
            );
        }
    }

    #[test]
    fn paint_shorthands_expand_css_wide_keywords_to_canonical_longhands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Border),
            CssWideKeyword::Unset,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Radius),
            CssWideKeyword::Inherit,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Outline),
            CssWideKeyword::Initial,
        ));
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Mask),
            CssWideKeyword::RevertLayer,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.get(Property::Border), None);
        for property in [
            Property::BorderTopWidth,
            Property::BorderRightWidth,
            Property::BorderBottomWidth,
            Property::BorderLeftWidth,
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(CssWideKeyword::Unset))
            );
        }

        assert_eq!(canonical.get(Property::Radius), None);
        for property in [
            Property::BorderTopLeftRadius,
            Property::BorderTopRightRadius,
            Property::BorderBottomRightRadius,
            Property::BorderBottomLeftRadius,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Inherit
                ))
            );
        }

        assert_eq!(canonical.get(Property::Outline), None);
        for property in [
            Property::OutlineWidth,
            Property::OutlineStyle,
            Property::OutlineColor,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::Initial
                ))
            );
        }

        assert_eq!(canonical.get(Property::Mask), None);
        for property in [
            Property::MaskImage,
            Property::MaskPosition,
            Property::MaskSize,
            Property::MaskRepeat,
        ] {
            assert_eq!(
                canonical.get(property),
                Some(&AuthoredCascadeValue::CssWideKeyword(
                    CssWideKeyword::RevertLayer
                ))
            );
        }
    }

    #[test]
    fn revert_layer_expands_without_entering_legacy_value_model() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Color),
            CssWideKeyword::RevertLayer,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(
            canonical.get(Property::Color),
            Some(&AuthoredCascadeValue::CssWideKeyword(
                CssWideKeyword::RevertLayer
            ))
        );
    }

    #[test]
    fn ordinary_values_still_expand_existing_shorthands() {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Width),
                    AuthoredValue::Value(Value::Length(Length::Px(12.0))),
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            declarations
                .to_rule_declarations()
                .unwrap()
                .get(Property::Width),
            Some(&AuthoredCascadeValue::Value(Value::Length(Length::Px(
                12.0
            ))))
        );
    }

    #[test]
    fn custom_property_accepts_custom_property_values() {
        let name = CustomPropertyName::try_new("--brand").unwrap();
        let custom_value =
            CustomPropertyValue::new(AuthoredTokens::new("var(--accent, black)"), []);
        let declaration = AuthoredDeclaration::try_new(
            AuthoredProperty::Custom(name.clone()),
            AuthoredValue::CustomProperty(custom_value.clone()),
        )
        .unwrap();

        assert_eq!(declaration.property(), AuthoredProperty::Custom(name));
        assert_eq!(
            declaration.value(),
            AuthoredValue::CustomProperty(custom_value)
        );
    }

    #[test]
    fn custom_property_accepts_css_wide_keywords_through_explicit_path() {
        let name = CustomPropertyName::try_new("--brand").unwrap();
        let declaration = AuthoredDeclaration::css_wide(
            AuthoredProperty::Custom(name.clone()),
            CssWideKeyword::Initial,
        );

        assert_eq!(declaration.property(), AuthoredProperty::Custom(name));
        assert_eq!(
            declaration.value(),
            AuthoredValue::CssWideKeyword(CssWideKeyword::Initial)
        );
    }

    #[test]
    fn all_expansion_does_not_include_custom_properties() {
        let mut declarations = AuthoredDeclarations::new();
        declarations.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::All,
            CssWideKeyword::Unset,
        ));

        let canonical = declarations.to_rule_declarations().unwrap();

        assert_eq!(canonical.custom_len(), 0);
    }

    #[test]
    fn custom_property_rejects_ordinary_values() {
        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Custom(CustomPropertyName::try_new("--brand").unwrap()),
            AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn custom_property_rejects_variable_dependent_values() {
        let variable = VariableDependentValue::try_new(
            Property::Color,
            AuthoredTokens::new("var(--brand, black)"),
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )),
        )
        .unwrap();

        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Custom(CustomPropertyName::try_new("--brand").unwrap()),
            AuthoredValue::VariableDependent(variable),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }

    #[test]
    fn property_accepts_variable_dependent_values_for_matching_property() {
        let variable = VariableDependentValue::try_new(
            Property::Color,
            AuthoredTokens::new("var(--brand, black)"),
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )),
        )
        .unwrap();

        let declaration = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Color),
            AuthoredValue::VariableDependent(variable.clone()),
        )
        .unwrap();

        assert_eq!(
            declaration.value(),
            AuthoredValue::VariableDependent(variable)
        );
    }

    #[test]
    fn property_rejects_variable_dependent_values_for_mismatched_property() {
        let variable = VariableDependentValue::try_new(
            Property::Color,
            AuthoredTokens::new("var(--brand, black)"),
            VariableExpression::Reference(VariableReference::new(
                CustomPropertyName::try_new("--brand").unwrap(),
                Some(VariableFallback::new(
                    AuthoredTokens::new("black"),
                    VariableExpression::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
                )),
            )),
        )
        .unwrap();

        let error = AuthoredDeclaration::try_new(
            AuthoredProperty::Property(Property::Width),
            AuthoredValue::VariableDependent(variable),
        )
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidProperty);
    }
}
