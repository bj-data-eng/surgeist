use std::collections::BTreeMap;

use crate::{Error, ErrorCode, Result, selector::SelectorSpecificity};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayerOrder(u32);

impl LayerOrder {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StyleLayerName {
    components: Vec<String>,
}

impl StyleLayerName {
    pub fn try_new(components: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        let components = components.into_iter().map(Into::into).collect::<Vec<_>>();
        if components.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "layer name cannot be empty",
            ));
        }
        for component in &components {
            validate_layer_component(component)?;
        }
        Ok(Self { components })
    }

    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleLayerNameList {
    names: Vec<StyleLayerName>,
}

impl StyleLayerNameList {
    pub fn try_new(names: impl IntoIterator<Item = StyleLayerName>) -> Result<Self> {
        let names = names.into_iter().collect::<Vec<_>>();
        if names.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "layer name list cannot be empty",
            ));
        }
        Ok(Self { names })
    }

    #[must_use]
    pub fn names(&self) -> &[StyleLayerName] {
        &self.names
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerStatement {
    names: StyleLayerNameList,
}

impl LayerStatement {
    #[must_use]
    pub const fn new(names: StyleLayerNameList) -> Self {
        Self { names }
    }

    #[must_use]
    pub const fn names(&self) -> &StyleLayerNameList {
        &self.names
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayerBlock {
    Named(StyleLayerName),
    Anonymous,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LayerRegistry {
    named: BTreeMap<StyleLayerName, LayerOrder>,
    next_order: u32,
}

impl LayerRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, statement: &LayerStatement) -> Vec<(StyleLayerName, LayerOrder)> {
        statement
            .names()
            .names()
            .iter()
            .cloned()
            .map(|name| {
                let order = self.register_named(name.clone());
                (name, order)
            })
            .collect()
    }

    pub fn register_named(&mut self, name: StyleLayerName) -> LayerOrder {
        if let Some(order) = self.named.get(&name) {
            return *order;
        }
        let order = self.allocate_order();
        self.named.insert(name, order);
        order
    }

    pub fn register_anonymous(&mut self) -> LayerOrder {
        self.allocate_order()
    }

    #[must_use]
    pub fn order(&self, name: &StyleLayerName) -> Option<LayerOrder> {
        self.named.get(name).copied()
    }

    fn allocate_order(&mut self) -> LayerOrder {
        self.next_order += 1;
        LayerOrder::new(self.next_order)
    }
}

fn validate_layer_component(component: &str) -> Result<()> {
    if component.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "layer name component cannot be empty",
        ));
    }
    if component.contains('\0') {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "layer name component cannot contain U+0000",
        ));
    }
    if CSS_WIDE_KEYWORDS
        .iter()
        .any(|keyword| component.eq_ignore_ascii_case(keyword))
    {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "layer name component cannot be a CSS-wide keyword",
        ));
    }
    Ok(())
}

const CSS_WIDE_KEYWORDS: [&str; 5] = ["inherit", "initial", "unset", "revert", "revert-layer"];

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceOrder(u32);

impl SourceOrder {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RulePrecedence {
    layer_order: LayerOrder,
    specificity: SelectorSpecificity,
    source_order: SourceOrder,
}

impl RulePrecedence {
    #[must_use]
    pub const fn new(layer_order: LayerOrder, source_order: SourceOrder) -> Self {
        Self {
            layer_order,
            specificity: SelectorSpecificity::zero(),
            source_order,
        }
    }

    #[must_use]
    pub const fn layer_order(self) -> LayerOrder {
        self.layer_order
    }

    #[must_use]
    pub const fn source_order(self) -> SourceOrder {
        self.source_order
    }

    #[must_use]
    pub const fn specificity(self) -> SelectorSpecificity {
        self.specificity
    }

    #[must_use]
    pub const fn with_source_order(self, source_order: SourceOrder) -> Self {
        Self {
            source_order,
            ..self
        }
    }

    #[must_use]
    pub const fn with_specificity(self, specificity: SelectorSpecificity) -> Self {
        Self {
            specificity,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_layer_outranks_later_source_order() {
        let lower_layer_late_source =
            RulePrecedence::new(LayerOrder::new(1), SourceOrder::new(100));
        let higher_layer_early_source =
            RulePrecedence::new(LayerOrder::new(2), SourceOrder::new(0));

        assert!(higher_layer_early_source > lower_layer_late_source);
    }

    #[test]
    fn source_order_breaks_ties_inside_same_layer() {
        let early = RulePrecedence::new(LayerOrder::new(7), SourceOrder::new(1));
        let late = RulePrecedence::new(LayerOrder::new(7), SourceOrder::new(2));

        assert!(late > early);
    }

    #[test]
    fn default_precedence_is_zero_layer_zero_source() {
        let precedence = RulePrecedence::default();

        assert_eq!(precedence.layer_order(), LayerOrder::new(0));
        assert_eq!(precedence.specificity(), SelectorSpecificity::zero());
        assert_eq!(precedence.source_order(), SourceOrder::new(0));
    }

    #[test]
    fn selector_specificity_orders_between_layer_and_source_order() {
        let low_specificity_late = RulePrecedence::new(LayerOrder::new(1), SourceOrder::new(9))
            .with_specificity(SelectorSpecificity::new(0, 0, 1));
        let high_specificity_early = RulePrecedence::new(LayerOrder::new(1), SourceOrder::new(1))
            .with_specificity(SelectorSpecificity::new(0, 1, 0));
        let higher_layer = RulePrecedence::new(LayerOrder::new(2), SourceOrder::new(0))
            .with_specificity(SelectorSpecificity::zero());

        assert!(high_specificity_early > low_specificity_late);
        assert!(higher_layer > high_specificity_early);
    }

    #[test]
    fn style_layer_names_validate_components_without_normalizing() {
        assert!(StyleLayerName::try_new([""]).is_err());
        assert!(StyleLayerName::try_new(["theme\0"]).is_err());
        assert!(StyleLayerName::try_new(["revert-layer"]).is_err());
        assert!(StyleLayerName::try_new(["Inherit"]).is_err());
        assert!(StyleLayerName::try_new(["INITIAL"]).is_err());
        assert!(StyleLayerName::try_new(["Revert-Layer"]).is_err());
        assert!(StyleLayerName::try_new([" inherit "]).is_ok());
        let layer = StyleLayerName::try_new(["Theme", "buttons"]).unwrap();
        assert_eq!(
            layer.components(),
            &["Theme".to_string(), "buttons".to_string()]
        );
    }

    #[test]
    fn style_layer_name_lists_require_at_least_one_name() {
        assert!(StyleLayerNameList::try_new([]).is_err());
        assert!(StyleLayerNameList::try_new([StyleLayerName::try_new(["base"]).unwrap()]).is_ok());
    }

    #[test]
    fn layer_registry_preserves_named_order_and_allocates_anonymous_fresh() {
        let base = StyleLayerName::try_new(["base"]).unwrap();
        let theme = StyleLayerName::try_new(["theme"]).unwrap();
        let statement = LayerStatement::new(
            StyleLayerNameList::try_new([base.clone(), theme.clone()]).unwrap(),
        );
        let mut registry = LayerRegistry::new();

        assert_eq!(
            registry.declare(&statement),
            vec![
                (base.clone(), LayerOrder::new(1)),
                (theme.clone(), LayerOrder::new(2))
            ]
        );
        assert_eq!(registry.register_named(base.clone()), LayerOrder::new(1));
        assert_eq!(registry.register_anonymous(), LayerOrder::new(3));
        assert_eq!(registry.register_anonymous(), LayerOrder::new(4));
        assert_eq!(registry.order(&theme), Some(LayerOrder::new(2)));
    }
}
