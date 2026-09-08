use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    AuthoredDeclarations, Change, Condition, Declarations, KeyframesRule, Property, Result,
    RulePrecedence, Selector, SourceOrder, StyleBucket, Tree, Value,
    selector::{PrimaryKey, SelectorSpecificity},
};
use crate::{
    CustomPropertyName, Error, ErrorCode, LayerOrder, LayerRegistry, LayerStatement, RuleScope,
    StyleClass, StyleKey, StyleLayerName, StyleLayerNameList, StyleSourceId, StyleTag,
    authored::{
        AuthoredCanonicalDeclarations, AuthoredCascadeValue, AuthoredDeclarationItem,
        CustomPropertyCascadeValue,
    },
};

static NEXT_VERSION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Version(u64);

impl Version {
    fn next() -> Self {
        Self(NEXT_VERSION.fetch_add(1, Ordering::Relaxed))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleTarget {
    selector: Selector,
    bucket: StyleBucket,
}

impl RuleTarget {
    #[must_use]
    pub fn new(selector: Selector, bucket: StyleBucket) -> Self {
        Self { selector, bucket }
    }

    #[must_use]
    pub fn element(selector: Selector) -> Self {
        Self::new(selector, StyleBucket::Element)
    }

    #[must_use]
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    #[must_use]
    pub const fn bucket(&self) -> StyleBucket {
        self.bucket
    }

    #[must_use]
    pub fn specificity(&self) -> SelectorSpecificity {
        self.selector.specificity()
    }

    pub(crate) fn primary_key(&self) -> PrimaryKey {
        self.selector.primary_key()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    target: RuleTarget,
    declarations: RuleDeclarations,
    conditions: Vec<Condition>,
    scope: Option<RuleScope>,
    precedence: RulePrecedence,
    source_order_policy: RuleSourceOrderPolicy,
}

impl Rule {
    #[must_use]
    pub fn new(selector: Selector, declarations: Declarations) -> Self {
        Self::with_order(selector, declarations, 0)
    }

    #[must_use]
    pub fn targeted(target: RuleTarget, declarations: Declarations) -> Self {
        Self::with_target_order(target, declarations, 0)
    }

    #[must_use]
    pub(crate) fn with_order(selector: Selector, declarations: Declarations, order: u32) -> Self {
        Self::with_target_order(RuleTarget::element(selector), declarations, order)
    }

    #[must_use]
    pub(crate) fn with_target_order(
        target: RuleTarget,
        declarations: Declarations,
        order: u32,
    ) -> Self {
        Self::with_target_precedence(
            target,
            declarations,
            RulePrecedence::default().with_source_order(SourceOrder::new(order)),
        )
    }

    #[must_use]
    pub(crate) fn with_layer_order(
        layer_order: LayerOrder,
        selector: Selector,
        declarations: Declarations,
        source_order: SourceOrder,
    ) -> Self {
        Self::with_target_precedence(
            RuleTarget::element(selector),
            declarations,
            RulePrecedence::new(layer_order, source_order),
        )
    }

    #[must_use]
    fn with_target_precedence(
        target: RuleTarget,
        declarations: Declarations,
        precedence: RulePrecedence,
    ) -> Self {
        let specificity = target.specificity();
        Self {
            target,
            declarations: RuleDeclarations::Legacy(declarations),
            conditions: Vec::new(),
            scope: None,
            precedence: precedence.with_specificity(specificity),
            source_order_policy: RuleSourceOrderPolicy::RebaseOnExtend,
        }
    }

    #[must_use]
    pub(crate) fn with_authored(
        selector: Selector,
        declarations: AuthoredCanonicalDeclarations,
        precedence: RulePrecedence,
    ) -> Self {
        Self::with_authored_target(RuleTarget::element(selector), declarations, precedence)
    }

    #[must_use]
    pub(crate) fn with_authored_target(
        target: RuleTarget,
        declarations: AuthoredCanonicalDeclarations,
        precedence: RulePrecedence,
    ) -> Self {
        Self {
            target,
            declarations: RuleDeclarations::Authored(declarations),
            conditions: Vec::new(),
            scope: None,
            precedence,
            source_order_policy: RuleSourceOrderPolicy::PreserveExplicit,
        }
    }

    #[must_use]
    pub fn when(mut self, conditions: impl IntoIterator<Item = Condition>) -> Self {
        self.conditions = conditions.into_iter().collect();
        self
    }

    #[must_use]
    pub fn scoped(mut self, scope: RuleScope) -> Self {
        self.scope = Some(scope);
        self
    }

    #[must_use]
    pub fn selector(&self) -> &Selector {
        self.target.selector()
    }

    #[must_use]
    pub fn target(&self) -> &RuleTarget {
        &self.target
    }

    #[must_use]
    pub const fn style_bucket(&self) -> StyleBucket {
        self.target.bucket()
    }

    #[must_use]
    pub fn legacy_declarations(&self) -> Option<&Declarations> {
        if self.declaration_origin() != RuleDeclarationOrigin::Legacy {
            return None;
        }
        match &self.declarations {
            RuleDeclarations::Legacy(declarations) => Some(declarations),
            RuleDeclarations::Authored(_) => None,
        }
    }

    #[must_use]
    pub(crate) fn declaration_items(&self) -> Vec<RuleDeclarationItem<'_>> {
        match &self.declarations {
            RuleDeclarations::Legacy(declarations) => declarations
                .iter()
                .map(|declaration| {
                    RuleDeclarationItem::new(
                        declaration.property(),
                        RuleDeclarationOrigin::Legacy,
                        RuleDeclarationValue::Value(declaration.value()),
                        None,
                    )
                })
                .collect(),
            RuleDeclarations::Authored(declarations) => declarations
                .iter()
                .filter_map(|item| {
                    let AuthoredDeclarationItem::Property(property, value, source) = item else {
                        return None;
                    };
                    Some(RuleDeclarationItem::new(
                        *property,
                        RuleDeclarationOrigin::Authored,
                        RuleDeclarationValue::Authored(value),
                        *source,
                    ))
                })
                .collect(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn custom_declaration_items(&self) -> Vec<RuleCustomDeclarationItem<'_>> {
        match &self.declarations {
            RuleDeclarations::Legacy(_) => Vec::new(),
            RuleDeclarations::Authored(declarations) => declarations
                .iter()
                .filter_map(|item| {
                    let AuthoredDeclarationItem::Custom(name, value, source) = item else {
                        return None;
                    };
                    Some(RuleCustomDeclarationItem::new(
                        name,
                        RuleDeclarationOrigin::Authored,
                        value,
                        *source,
                    ))
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }

    #[must_use]
    pub const fn scope(&self) -> Option<&RuleScope> {
        self.scope.as_ref()
    }

    #[must_use]
    pub const fn precedence(&self) -> RulePrecedence {
        self.precedence
    }

    #[must_use]
    pub const fn order(&self) -> u32 {
        self.precedence.source_order().get()
    }

    #[must_use]
    const fn declaration_origin(&self) -> RuleDeclarationOrigin {
        self.declarations.origin()
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RuleDeclarations {
    Legacy(Declarations),
    Authored(AuthoredCanonicalDeclarations),
}

impl RuleDeclarations {
    #[must_use]
    const fn origin(&self) -> RuleDeclarationOrigin {
        match self {
            Self::Legacy(_) => RuleDeclarationOrigin::Legacy,
            Self::Authored(_) => RuleDeclarationOrigin::Authored,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleDeclarationOrigin {
    Legacy,
    Authored,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RuleDeclarationItem<'a> {
    property: Property,
    origin: RuleDeclarationOrigin,
    value: RuleDeclarationValue<'a>,
    source: Option<StyleSourceId>,
}

impl<'a> RuleDeclarationItem<'a> {
    #[must_use]
    const fn new(
        property: Property,
        origin: RuleDeclarationOrigin,
        value: RuleDeclarationValue<'a>,
        source: Option<StyleSourceId>,
    ) -> Self {
        Self {
            property,
            origin,
            value,
            source,
        }
    }

    #[must_use]
    pub(crate) const fn property(self) -> Property {
        self.property
    }

    #[must_use]
    pub(crate) const fn origin(self) -> RuleDeclarationOrigin {
        self.origin
    }

    #[must_use]
    pub(crate) const fn value(self) -> RuleDeclarationValue<'a> {
        self.value
    }

    #[must_use]
    pub(crate) const fn source(self) -> Option<StyleSourceId> {
        self.source
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RuleDeclarationValue<'a> {
    Value(&'a Value),
    Authored(&'a AuthoredCascadeValue),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct RuleCustomDeclarationItem<'a> {
    name: &'a CustomPropertyName,
    origin: RuleDeclarationOrigin,
    value: &'a CustomPropertyCascadeValue,
    source: Option<StyleSourceId>,
}

#[allow(dead_code)]
impl<'a> RuleCustomDeclarationItem<'a> {
    #[must_use]
    const fn new(
        name: &'a CustomPropertyName,
        origin: RuleDeclarationOrigin,
        value: &'a CustomPropertyCascadeValue,
        source: Option<StyleSourceId>,
    ) -> Self {
        Self {
            name,
            origin,
            value,
            source,
        }
    }

    #[must_use]
    pub(crate) const fn name(self) -> &'a CustomPropertyName {
        self.name
    }

    #[must_use]
    pub(crate) const fn origin(self) -> RuleDeclarationOrigin {
        self.origin
    }

    #[must_use]
    pub(crate) const fn value(self) -> &'a CustomPropertyCascadeValue {
        self.value
    }

    #[must_use]
    pub(crate) const fn source(self) -> Option<StyleSourceId> {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuleSourceOrderPolicy {
    RebaseOnExtend,
    PreserveExplicit,
}

#[derive(Clone, Debug)]
pub struct Sheet {
    rules: Vec<Rule>,
    keyframes: Vec<KeyframesRule>,
    layers: LayerRegistry,
    index: RuleIndex,
    version: Version,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            keyframes: Vec::new(),
            layers: LayerRegistry::new(),
            index: RuleIndex::default(),
            version: Version::next(),
        }
    }
}

impl PartialEq for Sheet {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules
            && self.keyframes == other.keyframes
            && self.layers == other.layers
    }
}

impl Sheet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn rule(mut self, selector: Selector, declarations: Declarations) -> Self {
        self.push_rule(selector, declarations);
        self
    }

    #[must_use]
    pub fn targeted_rule(mut self, target: RuleTarget, declarations: Declarations) -> Self {
        self.push_targeted_rule(target, declarations);
        self
    }

    #[must_use]
    pub fn keyframes_rule(mut self, rule: KeyframesRule) -> Self {
        self.push_keyframes_rule(rule);
        self
    }

    #[must_use]
    pub fn conditional_rule(
        mut self,
        selector: Selector,
        declarations: Declarations,
        conditions: impl IntoIterator<Item = Condition>,
    ) -> Self {
        self.push_conditional_rule(selector, declarations, conditions);
        self
    }

    pub fn push_rule(&mut self, selector: Selector, declarations: Declarations) -> &mut Self {
        let order = self.rules.len() as u32;
        self.push(Rule::with_order(selector, declarations, order));
        self
    }

    pub fn push_targeted_rule(
        &mut self,
        target: RuleTarget,
        declarations: Declarations,
    ) -> &mut Self {
        let order = self.rules.len() as u32;
        self.push(Rule::with_target_order(target, declarations, order));
        self
    }

    pub fn push_conditional_rule(
        &mut self,
        selector: Selector,
        declarations: Declarations,
        conditions: impl IntoIterator<Item = Condition>,
    ) -> &mut Self {
        let order = self.rules.len() as u32;
        self.push(Rule::with_order(selector, declarations, order).when(conditions));
        self
    }

    pub fn push_scoped_rule(
        &mut self,
        scope: RuleScope,
        selector: Selector,
        declarations: Declarations,
    ) -> Result<&mut Self> {
        let order = self.rules.len() as u32;
        self.push(Rule::with_order(selector, declarations, order).scoped(scope));
        Ok(self)
    }

    pub fn push_authored_rule(
        &mut self,
        selector: Selector,
        declarations: AuthoredDeclarations,
        precedence: RulePrecedence,
    ) -> Result<&mut Self> {
        let declarations = declarations.to_rule_declarations()?;
        self.push(Rule::with_authored(selector, declarations, precedence));
        Ok(self)
    }

    pub fn push_authored_scoped_rule(
        &mut self,
        scope: RuleScope,
        selector: Selector,
        declarations: AuthoredDeclarations,
        precedence: RulePrecedence,
    ) -> Result<&mut Self> {
        let declarations = declarations.to_rule_declarations()?;
        self.push(Rule::with_authored(selector, declarations, precedence).scoped(scope));
        Ok(self)
    }

    pub fn push_authored_targeted_rule(
        &mut self,
        target: RuleTarget,
        declarations: AuthoredDeclarations,
        precedence: RulePrecedence,
    ) -> Result<&mut Self> {
        let declarations = declarations.to_rule_declarations()?;
        self.push(Rule::with_authored_target(target, declarations, precedence));
        Ok(self)
    }

    pub fn declare_layers(&mut self, names: StyleLayerNameList) -> &mut Self {
        self.layers.declare(&LayerStatement::new(names));
        self.version = Version::next();
        self
    }

    #[must_use]
    pub fn layer_order(&self, name: &StyleLayerName) -> Option<LayerOrder> {
        self.layers.order(name)
    }

    pub fn register_anonymous_layer(&mut self) -> LayerOrder {
        let order = self.layers.register_anonymous();
        self.version = Version::next();
        order
    }

    pub fn push_layer_order_rule(
        &mut self,
        layer_order: LayerOrder,
        selector: Selector,
        declarations: Declarations,
    ) -> Result<&mut Self> {
        self.ensure_layered_order(layer_order)?;
        let source_order = SourceOrder::new(self.rules.len() as u32);
        self.push(Rule::with_layer_order(
            layer_order,
            selector,
            declarations,
            source_order,
        ));
        Ok(self)
    }

    pub fn push_layer_rule(
        &mut self,
        layer: StyleLayerName,
        selector: Selector,
        declarations: Declarations,
    ) -> Result<&mut Self> {
        let layer_order = self.layers.register_named(layer);
        self.push_layer_order_rule(layer_order, selector, declarations)
    }

    pub fn push_authored_layer_rule(
        &mut self,
        layer: StyleLayerName,
        selector: Selector,
        declarations: AuthoredDeclarations,
        source_order: SourceOrder,
    ) -> Result<&mut Self> {
        let layer_order = self.layers.register_named(layer);
        self.push_authored_layer_order_rule(layer_order, selector, declarations, source_order)
    }

    pub fn push_authored_layer_order_rule(
        &mut self,
        layer_order: LayerOrder,
        selector: Selector,
        declarations: AuthoredDeclarations,
        source_order: SourceOrder,
    ) -> Result<&mut Self> {
        self.ensure_layered_order(layer_order)?;
        let specificity = selector.specificity();
        let declarations = declarations.to_rule_declarations()?;
        self.push(Rule::with_authored(
            selector,
            declarations,
            RulePrecedence::new(layer_order, source_order).with_specificity(specificity),
        ));
        Ok(self)
    }

    pub fn extend(&mut self, rules: impl IntoIterator<Item = Rule>) -> &mut Self {
        for mut rule in rules {
            if rule.source_order_policy == RuleSourceOrderPolicy::RebaseOnExtend {
                rule.precedence = rule
                    .precedence
                    .with_source_order(SourceOrder::new(self.rules.len() as u32));
            }
            self.push(rule);
        }
        self
    }

    pub fn push_keyframes_rule(&mut self, rule: KeyframesRule) -> &mut Self {
        self.keyframes.push(rule);
        self.version = Version::next();
        self
    }

    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    #[must_use]
    pub fn keyframes_rule_count(&self) -> usize {
        self.keyframes.len()
    }

    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    #[must_use]
    pub fn keyframes_rules(&self) -> &[KeyframesRule] {
        &self.keyframes
    }

    pub fn rules_for_selector<'a>(
        &'a self,
        selector: &'a Selector,
    ) -> impl Iterator<Item = &'a Rule> + 'a {
        self.rules
            .iter()
            .filter(move |rule| rule.selector() == selector)
    }

    pub fn rules_for_class<'a>(
        &'a self,
        class: &'a StyleClass,
    ) -> impl Iterator<Item = &'a Rule> + 'a {
        self.index
            .by_class
            .get(class)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|index| &self.rules[*index]))
    }

    pub fn rules_for_tag<'a>(&'a self, tag: &'a StyleTag) -> impl Iterator<Item = &'a Rule> + 'a {
        self.index
            .by_tag
            .get(tag)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|index| &self.rules[*index]))
    }

    pub fn rules_for_key<'a>(&'a self, key: &'a StyleKey) -> impl Iterator<Item = &'a Rule> + 'a {
        self.index
            .by_key
            .get(key)
            .into_iter()
            .flat_map(|indices| indices.iter().map(|index| &self.rules[*index]))
    }

    pub fn conditional_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules
            .iter()
            .filter(|rule| !rule.conditions().is_empty())
    }

    pub fn unconditional_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules
            .iter()
            .filter(|rule| rule.conditions().is_empty())
    }

    #[must_use]
    pub fn media_condition_change(&self) -> Change {
        self.condition_change(Condition::is_media)
    }

    #[must_use]
    pub fn container_condition_change(&self) -> Change {
        self.condition_change(Condition::is_container)
    }

    pub fn candidate_rule_count<T: Tree>(&self, tree: &T, id: T::Id) -> Result<usize> {
        Ok(self.candidate_indices(tree, id)?.len())
    }

    pub(crate) fn candidate_rules<T: Tree>(&self, tree: &T, id: T::Id) -> Result<Vec<&Rule>> {
        Ok(self
            .candidate_indices(tree, id)?
            .into_iter()
            .map(|index| &self.rules[index])
            .collect())
    }

    fn push(&mut self, rule: Rule) {
        let index = self.rules.len();
        self.index.insert(index, rule.target().primary_key());
        self.rules.push(rule);
        self.version = Version::next();
    }

    fn condition_change(&self, predicate: impl Fn(&Condition) -> bool) -> Change {
        let mut change = Change::empty();
        for rule in &self.rules {
            if !rule.conditions().iter().any(&predicate) {
                continue;
            }
            change.rematch = true;
            change.scope.include_subtree();
            match &rule.declarations {
                RuleDeclarations::Legacy(declarations) => {
                    for declaration in declarations.iter() {
                        change.invalidation.include_property(declaration.property);
                    }
                }
                RuleDeclarations::Authored(declarations) => {
                    for item in declarations.iter() {
                        if let AuthoredDeclarationItem::Property(property, _, _) = item {
                            change.invalidation.include_property(*property);
                        }
                    }
                }
            }
        }
        change
    }

    fn candidate_indices<T: Tree>(&self, tree: &T, id: T::Id) -> Result<Vec<usize>> {
        let node = tree.node(id)?;
        let mut candidates = BTreeSet::new();
        candidates.extend(self.index.universal.iter().copied());
        if let Some(key) = node.key.as_ref()
            && let Some(indices) = self.index.by_key.get(key)
        {
            candidates.extend(indices.iter().copied());
        }
        if let Some(tag) = node.tag.as_ref()
            && let Some(indices) = self.index.by_tag.get(tag)
        {
            candidates.extend(indices.iter().copied());
        }
        for class in &node.classes {
            if let Some(indices) = self.index.by_class.get(class) {
                candidates.extend(indices.iter().copied());
            }
        }
        Ok(candidates.into_iter().collect())
    }

    fn ensure_layered_order(&self, layer_order: LayerOrder) -> Result<()> {
        if layer_order == LayerOrder::default() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "layered rules require a non-default layer order",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuleIndex {
    universal: Vec<usize>,
    by_key: BTreeMap<StyleKey, Vec<usize>>,
    by_class: BTreeMap<StyleClass, Vec<usize>>,
    by_tag: BTreeMap<StyleTag, Vec<usize>>,
}

impl RuleIndex {
    fn insert(&mut self, index: usize, key: PrimaryKey) {
        match key {
            PrimaryKey::Universal => self.universal.push(index),
            PrimaryKey::Key(key) => self.by_key.entry(key).or_default().push(index),
            PrimaryKey::Class(class) => self.by_class.entry(class).or_default().push(index),
            PrimaryKey::Tag(tag) => self.by_tag.entry(tag).or_default().push(index),
        }
    }
}

#[cfg(test)]
mod precedence_tests {
    use super::*;
    use crate::{
        AuthoredDeclaration, AuthoredDeclarations, AuthoredProperty, AuthoredValue, Color,
        ContainerCondition, ContainerFeatureQuery, Context, CssWideKeyword, Error, ErrorCode,
        KeyframeBlock, KeyframeOffset, KeyframeSelectorList, KeyframesIdent, KeyframesName,
        KeyframesRule, LayerOrder, MediaCondition, MediaFeatureQuery, MediaQuery, MediaQueryList,
        Node, Property, QueryComparison, QueryLength, QueryLengthUnit, RangeFeature, Resolver,
        RulePrecedence, Selector, SelectorSpecificity, SourceOrder, StyleColor, StyleLayerName,
        StyleLayerNameList, StyleRole, StyleState, Traversal, Value,
    };

    fn authored_color(color: Color) -> AuthoredDeclarations {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Color),
                    AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(color))),
                )
                .unwrap(),
            )
            .unwrap();
        declarations
    }

    #[test]
    fn existing_rule_api_uses_default_layer_and_insertion_source_order() {
        let mut sheet = Sheet::new();
        sheet.push_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );

        let rule = &sheet.rules()[0];
        assert_eq!(
            rule.precedence(),
            RulePrecedence::new(LayerOrder::default(), SourceOrder::new(0))
                .with_specificity(SelectorSpecificity::new(0, 0, 1))
        );
        assert_eq!(rule.order(), 0);
    }

    #[test]
    fn sheet_rule_builder_preserves_existing_source_order_behavior() {
        let sheet = Sheet::new()
            .rule(
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::BLACK)
                    .unwrap(),
            )
            .rule(
                Selector::class("primary").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::TRANSPARENT)
                    .unwrap(),
            );

        assert_eq!(sheet.rules()[0].order(), 0);
        assert_eq!(sheet.rules()[1].order(), 1);
        assert_eq!(
            sheet.rules()[1].precedence(),
            RulePrecedence::new(LayerOrder::default(), SourceOrder::new(1))
                .with_specificity(SelectorSpecificity::new(0, 1, 0))
        );
    }

    #[test]
    fn rule_new_derives_specificity_from_selector() {
        let rule = Rule::new(
            Selector::key("submit").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );

        assert_eq!(
            rule.precedence(),
            RulePrecedence::default().with_specificity(SelectorSpecificity::new(1, 0, 0))
        );
    }

    #[test]
    fn conditional_legacy_rules_derive_specificity_and_preserve_source_order() {
        let mut sheet = Sheet::new();
        sheet.push_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );
        sheet.push_conditional_rule(
            Selector::class("primary").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::TRANSPARENT)
                .unwrap(),
            [Condition::media(
                MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
                    MediaFeatureQuery::Width(RangeFeature::new(
                        Some(QueryComparison::GreaterThanOrEqual),
                        QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap(),
                    )),
                ))])
                .unwrap(),
            )],
        );

        assert_eq!(
            sheet.rules()[1].precedence(),
            RulePrecedence::new(LayerOrder::default(), SourceOrder::new(1))
                .with_specificity(SelectorSpecificity::new(0, 1, 0))
        );
    }

    #[test]
    fn media_condition_change_includes_conditional_rule_properties() {
        let sheet = Sheet::new()
            .rule(
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::BLACK)
                    .unwrap(),
            )
            .conditional_rule(
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_set(Property::Width, Value::Length(crate::Length::Px(48.0)))
                    .unwrap(),
                [Condition::media(
                    MediaQueryList::try_new([MediaQuery::Condition(MediaCondition::Feature(
                        MediaFeatureQuery::Width(RangeFeature::new(
                            Some(QueryComparison::GreaterThanOrEqual),
                            QueryLength::try_new(320.0, QueryLengthUnit::Px).unwrap(),
                        )),
                    ))])
                    .unwrap(),
                )],
            );

        let change = sheet.media_condition_change();

        assert!(change.rematch);
        assert!(change.invalidation.layout);
        assert!(!change.invalidation.paint);
    }

    #[test]
    fn container_condition_change_includes_conditional_rule_properties() {
        let sheet = Sheet::new().conditional_rule(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::TRANSPARENT)
                .unwrap(),
            [Condition::container(ContainerCondition::Feature(
                ContainerFeatureQuery::Width(RangeFeature::new(
                    Some(QueryComparison::GreaterThanOrEqual),
                    QueryLength::try_new(300.0, QueryLengthUnit::Px).unwrap(),
                )),
            ))],
        );

        let change = sheet.container_condition_change();

        assert!(change.rematch);
        assert!(change.invalidation.paint);
        assert!(!change.invalidation.layout);
        assert!(!sheet.media_condition_change().invalidation.paint);
    }

    #[test]
    fn condition_change_helpers_remain_sheet_local() {
        let sheet = Sheet::new();

        assert_eq!(sheet.media_condition_change(), Change::empty());
        assert_eq!(sheet.container_condition_change(), Change::empty());
    }

    #[test]
    fn authored_rule_preserves_supplied_precedence() {
        let precedence = RulePrecedence::new(LayerOrder::new(7), SourceOrder::new(3));
        let mut sheet = Sheet::new();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                authored_color(Color::BLACK),
                precedence,
            )
            .unwrap();

        assert_eq!(sheet.rules()[0].precedence(), precedence);
    }

    #[test]
    fn rule_new_defaults_to_element_bucket() {
        let rule = Rule::new(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );

        assert_eq!(rule.style_bucket(), StyleBucket::Element);
        assert_eq!(
            rule.target(),
            &RuleTarget::element(Selector::tag("button").unwrap())
        );
    }

    #[test]
    fn targeted_rule_preserves_bucket() {
        let selector = Selector::class("badge").unwrap();
        let target = RuleTarget::new(selector.clone(), StyleBucket::BeforeMarker);
        let rule = Rule::targeted(
            target.clone(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );
        let sheet = Sheet::new().targeted_rule(
            target.clone(),
            Declarations::new()
                .try_concrete_text_color(Color::TRANSPARENT)
                .unwrap(),
        );

        assert_eq!(rule.selector(), &selector);
        assert_eq!(rule.target(), &target);
        assert_eq!(rule.style_bucket(), StyleBucket::BeforeMarker);
        assert_eq!(sheet.rules()[0].target(), &target);
        assert_eq!(sheet.rules()[0].style_bucket(), StyleBucket::BeforeMarker);
    }

    #[test]
    fn targeted_rule_uses_origin_selector_primary_key() {
        let mut sheet = Sheet::new();
        sheet.push_targeted_rule(
            RuleTarget::new(Selector::class("badge").unwrap(), StyleBucket::After),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );

        let class = StyleClass::new("badge").unwrap();
        let indexed_rules: Vec<_> = sheet.rules_for_class(&class).collect();
        assert_eq!(indexed_rules.len(), 1);
        assert_eq!(indexed_rules[0].style_bucket(), StyleBucket::After);
    }

    #[test]
    fn keyframes_store_on_sheet_without_entering_rule_index() {
        let mut first_declarations = AuthoredDeclarations::new();
        first_declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Opacity),
                    AuthoredValue::Value(Value::Number(0.0)),
                )
                .unwrap(),
            )
            .unwrap();
        let first_keyframes = KeyframesRule::try_new(
            KeyframesName::Ident(KeyframesIdent::try_new("fade-in").unwrap()),
            [KeyframeBlock::try_new(
                KeyframeSelectorList::try_new([KeyframeOffset::from()]).unwrap(),
                first_declarations,
            )
            .unwrap()],
        )
        .unwrap();

        let mut second_declarations = AuthoredDeclarations::new();
        second_declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Opacity),
                    AuthoredValue::Value(Value::Number(1.0)),
                )
                .unwrap(),
            )
            .unwrap();
        let second_keyframes = KeyframesRule::try_new(
            KeyframesName::Ident(KeyframesIdent::try_new("fade-out").unwrap()),
            [KeyframeBlock::try_new(
                KeyframeSelectorList::try_new([KeyframeOffset::to()]).unwrap(),
                second_declarations,
            )
            .unwrap()],
        )
        .unwrap();

        let mut sheet = Sheet::new().keyframes_rule(first_keyframes.clone());
        sheet.push_keyframes_rule(second_keyframes.clone());
        sheet.push_rule(Selector::class("badge").unwrap(), Declarations::new());

        assert_eq!(sheet.rule_count(), 1);
        assert_eq!(sheet.keyframes_rule_count(), 2);
        assert_eq!(sheet.keyframes_rules()[0], first_keyframes);
        assert_eq!(sheet.keyframes_rules()[1], second_keyframes);

        let class = StyleClass::new("badge").unwrap();
        assert_eq!(sheet.rules_for_class(&class).count(), 1);
        let missing_class = StyleClass::new("fade-in").unwrap();
        assert_eq!(sheet.rules_for_class(&missing_class).count(), 0);
    }

    #[test]
    fn push_authored_targeted_rule_preserves_explicit_precedence() {
        let precedence = RulePrecedence::new(LayerOrder::new(7), SourceOrder::new(3));
        let mut sheet = Sheet::new();
        sheet
            .push_authored_targeted_rule(
                RuleTarget::new(Selector::tag("button").unwrap(), StyleBucket::Before),
                authored_color(Color::BLACK),
                precedence,
            )
            .unwrap();

        assert_eq!(sheet.rules()[0].precedence(), precedence);
        assert_eq!(sheet.rules()[0].style_bucket(), StyleBucket::Before);
    }

    #[test]
    fn authored_rules_do_not_expose_legacy_declarations() {
        let mut authored = AuthoredDeclarations::new();
        authored.push(AuthoredDeclaration::css_wide(
            AuthoredProperty::Property(Property::Color),
            CssWideKeyword::RevertLayer,
        ));

        let mut sheet = Sheet::new();
        sheet
            .push_authored_rule(
                Selector::tag("button").unwrap(),
                authored,
                RulePrecedence::new(LayerOrder::new(2), SourceOrder::new(0)),
            )
            .unwrap();

        assert_eq!(sheet.rules()[0].legacy_declarations(), None);
    }

    #[test]
    fn extend_rebases_legacy_rules_and_preserves_authored_precedence() {
        let authored_precedence = RulePrecedence::new(LayerOrder::new(9), SourceOrder::new(20));
        let legacy_rule = Rule::new(
            Selector::tag("button").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );
        let mut authored_sheet = Sheet::new();
        authored_sheet
            .push_authored_rule(
                Selector::class("primary").unwrap(),
                authored_color(Color::TRANSPARENT),
                authored_precedence,
            )
            .unwrap();
        let authored_rule = authored_sheet.rules()[0].clone();

        let mut sheet = Sheet::new();
        sheet.push_rule(
            Selector::key("root").unwrap(),
            Declarations::new()
                .try_concrete_text_color(Color::BLACK)
                .unwrap(),
        );
        sheet.extend([legacy_rule, authored_rule]);

        assert_eq!(sheet.rules()[1].order(), 1);
        assert_eq!(
            sheet.rules()[1].precedence().layer_order(),
            LayerOrder::default()
        );
        assert_eq!(sheet.rules()[2].precedence(), authored_precedence);
    }

    #[test]
    fn layer_statements_register_named_layers_in_order() {
        let mut sheet = Sheet::new();
        let reset = StyleLayerName::try_new(["reset"]).unwrap();
        let theme = StyleLayerName::try_new(["theme"]).unwrap();
        sheet.declare_layers(StyleLayerNameList::try_new([reset.clone(), theme.clone()]).unwrap());

        assert_eq!(sheet.layer_order(&reset), Some(LayerOrder::new(1)));
        assert_eq!(sheet.layer_order(&theme), Some(LayerOrder::new(2)));
    }

    #[test]
    fn named_layer_rules_use_registered_layer_order() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let base = StyleLayerName::try_new(["base"]).unwrap();
        let theme = StyleLayerName::try_new(["theme"]).unwrap();
        let mut sheet = Sheet::new();
        sheet.declare_layers(StyleLayerNameList::try_new([base.clone(), theme.clone()]).unwrap());
        sheet
            .push_layer_rule(
                base,
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                    .unwrap(),
            )
            .unwrap();
        sheet
            .push_layer_rule(
                theme,
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.0, 0.0, 1.0, 1.0))
                    .unwrap(),
            )
            .unwrap();

        let mut resolver = Resolver::new(sheet);
        let resolved = resolver.resolve(Context::new(&tree, 0)).unwrap();

        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.0, 0.0, 1.0, 1.0))
        );
    }

    #[test]
    fn anonymous_layer_blocks_get_fresh_order() {
        let tree = TestTree::new(vec![TestNode::new(0, "button")]);
        let mut sheet = Sheet::new();
        let first = sheet.register_anonymous_layer();
        let second = sheet.register_anonymous_layer();
        sheet
            .push_layer_order_rule(
                first,
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(1.0, 0.0, 0.0, 1.0))
                    .unwrap(),
            )
            .unwrap();
        sheet
            .push_layer_order_rule(
                second,
                Selector::tag("button").unwrap(),
                Declarations::new()
                    .try_concrete_text_color(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
                    .unwrap(),
            )
            .unwrap();

        assert!(second > first);
        let mut resolver = Resolver::new(sheet);
        let resolved = resolver.resolve(Context::new(&tree, 0)).unwrap();
        assert_eq!(
            resolved.text_color(),
            &StyleColor::rgba(Color::raw_rgba(0.0, 1.0, 0.0, 1.0))
        );
    }

    #[derive(Clone, Debug)]
    struct TestNode {
        id: usize,
        tag: StyleTag,
        classes: Vec<StyleClass>,
        children: Vec<usize>,
    }

    impl TestNode {
        fn new(id: usize, tag: &str) -> Self {
            Self {
                id,
                tag: StyleTag::new(tag).unwrap(),
                classes: Vec::new(),
                children: Vec::new(),
            }
        }
    }

    struct TestTree {
        nodes: Vec<TestNode>,
    }

    impl TestTree {
        fn new(nodes: Vec<TestNode>) -> Self {
            Self { nodes }
        }
    }

    impl Tree for TestTree {
        type Id = usize;

        fn version_hint(&self) -> Option<u64> {
            Some(1)
        }

        fn node(&self, id: Self::Id) -> Result<Node<Self::Id>> {
            let node = self
                .nodes
                .get(id)
                .ok_or_else(|| Error::new(ErrorCode::MissingNode, "missing test node"))?;
            Ok(Node {
                id: node.id,
                tag: Some(node.tag.clone()),
                key: None,
                classes: node.classes.clone(),
                attributes: Vec::new(),
                role: StyleRole::default(),
                state: StyleState::default(),
                text: false,
            })
        }

        fn parent(&self, id: Self::Id, _traversal: Traversal) -> Result<Option<Self::Id>> {
            Ok(self
                .nodes
                .iter()
                .find(|node| node.children.contains(&id))
                .map(|node| node.id))
        }

        fn children(
            &self,
            id: Self::Id,
            _traversal: Traversal,
        ) -> Result<impl Iterator<Item = Self::Id> + '_> {
            Ok(self.nodes[id].children.iter().copied())
        }

        fn previous_sibling(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>> {
            let Some(parent) = self.parent(id, traversal)? else {
                return Ok(None);
            };
            let siblings = &self.nodes[parent].children;
            Ok(siblings
                .iter()
                .position(|sibling| *sibling == id)
                .and_then(|index| index.checked_sub(1))
                .map(|index| siblings[index]))
        }
    }
}
