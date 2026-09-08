use super::{CustomPropertyDependencies, CustomPropertyName, Property, Resolved, StyleBucket};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Invalidation {
    pub layout: bool,
    pub paint: bool,
    pub text: bool,
    pub effect: bool,
    pub animation: bool,
}

impl Invalidation {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            layout: false,
            paint: false,
            text: false,
            effect: false,
            animation: false,
        }
    }

    #[must_use]
    pub fn from_properties(properties: impl IntoIterator<Item = Property>) -> Self {
        let mut invalidation = Self::empty();
        for property in properties {
            invalidation.include_property(property);
        }
        invalidation
    }

    pub fn include_property(&mut self, property: Property) {
        let impact = property.metadata().impact_flags();
        self.layout |= impact.affects_layout();
        self.paint |= impact.affects_paint();
        self.text |= impact.affects_text();
        self.effect |= impact.affects_effect();
        self.animation |= impact.affects_animation();
    }

    #[must_use]
    pub fn between(before: &Resolved, after: &Resolved) -> Self {
        let mut invalidation = Self::empty();
        for property in Property::ALL {
            if property.is_canonical() && before.get(*property) != after.get(*property) {
                invalidation.include_property(*property);
            }
        }
        invalidation
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Scope {
    pub node: bool,
    pub siblings: bool,
    pub descendants: bool,
    pub whole_tree: bool,
}

impl Scope {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            node: false,
            siblings: false,
            descendants: false,
            whole_tree: false,
        }
    }

    pub const fn include_node(&mut self) {
        self.node = true;
    }

    pub const fn include_siblings(&mut self) {
        self.siblings = true;
    }

    pub const fn include_descendants(&mut self) {
        self.descendants = true;
    }

    pub const fn include_whole_tree(&mut self) {
        self.whole_tree = true;
    }

    pub const fn include_subtree(&mut self) {
        self.node = true;
        self.descendants = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorFactChange {
    Tag,
    Key,
    Class,
    Attribute,
    RuntimeState,
    Structure,
    Scope,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConditionFactChange {
    Media,
    Container,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CascadeChange {
    LayerOrder,
    RuleScope,
    SourceOrder,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Change {
    pub rematch: bool,
    pub invalidation: Invalidation,
    pub scope: Scope,
}

impl Change {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rematch: false,
            invalidation: Invalidation::empty(),
            scope: Scope::empty(),
        }
    }

    #[must_use]
    pub fn from_properties(properties: impl IntoIterator<Item = Property>) -> Self {
        let mut change = Self::empty();
        for property in properties {
            change.scope.include_node();
            change.invalidation.include_property(property);
            if property.metadata().is_inherited() {
                change.scope.include_descendants();
            }
        }
        change
    }

    #[must_use]
    pub fn from_custom_properties(
        changed: impl IntoIterator<Item = CustomPropertyName>,
        dependencies: &CustomPropertyDependencies,
    ) -> Self {
        let mut change = Self::empty();
        for name in changed {
            change.scope.include_node();
            change.scope.include_descendants();
            for property in dependencies.properties_for_custom_property(&name) {
                change.invalidation.include_property(property);
            }
        }
        change
    }

    #[must_use]
    pub fn from_selector_fact_change(_fact: SelectorFactChange) -> Self {
        let mut change = Self::empty();
        change.rematch = true;
        change.scope.include_whole_tree();
        change
    }

    #[must_use]
    pub fn from_condition_fact_change(_fact: ConditionFactChange) -> Self {
        let mut change = Self::empty();
        change.rematch = true;
        change.scope.include_whole_tree();
        change
    }

    #[must_use]
    pub fn from_cascade_change(_change: CascadeChange) -> Self {
        let mut change = Self::empty();
        change.rematch = true;
        change.scope.include_whole_tree();
        change
    }

    #[must_use]
    pub fn from_style_bucket_change(_bucket: StyleBucket) -> Self {
        let mut change = Self::empty();
        change.rematch = true;
        change.scope.include_whole_tree();
        change
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomPropertyDependencies, CustomPropertyName};

    fn custom_name(name: &str) -> CustomPropertyName {
        CustomPropertyName::try_new(name).unwrap()
    }

    #[test]
    fn custom_property_invalidation_includes_dependent_ordinary_property_impacts() {
        let brand = custom_name("--brand");
        let mut dependencies = CustomPropertyDependencies::default();
        dependencies.insert(Property::Width, brand.clone());

        let change = Change::from_custom_properties([brand], &dependencies);

        assert!(!change.rematch);
        assert!(change.scope.node);
        assert!(change.scope.descendants);
        assert!(!change.scope.siblings);
        assert!(!change.scope.whole_tree);
        assert!(change.invalidation.layout);
        assert!(!change.invalidation.paint);
        assert!(!change.invalidation.text);
        assert!(!change.invalidation.effect);
        assert!(!change.invalidation.animation);
    }

    #[test]
    fn custom_property_invalidation_includes_descendant_scope_for_inherited_custom_properties() {
        let change = Change::from_custom_properties(
            [custom_name("--brand")],
            &CustomPropertyDependencies::default(),
        );

        assert!(!change.rematch);
        assert!(change.scope.node);
        assert!(change.scope.descendants);
        assert!(!change.scope.siblings);
        assert!(!change.scope.whole_tree);
        assert_eq!(change.invalidation, Invalidation::empty());
    }

    #[test]
    fn unrelated_custom_property_invalidation_does_not_include_ordinary_property_impacts() {
        let mut dependencies = CustomPropertyDependencies::default();
        dependencies.insert(Property::Width, custom_name("--space"));

        let change = Change::from_custom_properties([custom_name("--brand")], &dependencies);

        assert!(!change.rematch);
        assert!(change.scope.node);
        assert!(change.scope.descendants);
        assert!(!change.scope.whole_tree);
        assert_eq!(change.invalidation, Invalidation::empty());
    }

    #[test]
    fn selector_fact_changes_use_whole_tree_rematch_for_has_and_filtered_structural_safety() {
        for fact in [
            SelectorFactChange::Tag,
            SelectorFactChange::Key,
            SelectorFactChange::Class,
            SelectorFactChange::Attribute,
            SelectorFactChange::RuntimeState,
            SelectorFactChange::Structure,
            SelectorFactChange::Scope,
        ] {
            let change = Change::from_selector_fact_change(fact);

            assert!(change.rematch);
            assert!(change.scope.whole_tree);
            assert_eq!(change.invalidation, Invalidation::empty());
        }
    }

    #[test]
    fn condition_fact_changes_rematch_condition_dependent_rules() {
        let media = Change::from_condition_fact_change(ConditionFactChange::Media);
        let container = Change::from_condition_fact_change(ConditionFactChange::Container);

        assert!(media.rematch);
        assert!(media.scope.whole_tree);
        assert_eq!(media.invalidation, Invalidation::empty());
        assert!(container.rematch);
        assert!(container.scope.whole_tree);
    }

    #[test]
    fn cascade_structure_changes_rematch_without_claiming_property_output() {
        for change in [
            Change::from_cascade_change(CascadeChange::LayerOrder),
            Change::from_cascade_change(CascadeChange::RuleScope),
            Change::from_style_bucket_change(crate::StyleBucket::Before),
        ] {
            assert!(change.rematch);
            assert!(change.scope.whole_tree);
            assert_eq!(change.invalidation, Invalidation::empty());
        }
    }

    #[test]
    fn selector_fact_change_rematch_scope_documents_conservative_selector_dependencies() {
        // A descendant fact change can affect an ancestor selector such as :has(.changed).
        let ancestor_has_change = Change::from_selector_fact_change(SelectorFactChange::Class);
        assert!(ancestor_has_change.scope.whole_tree);

        // A following sibling fact change can affect earlier siblings matching :has(+ .changed)
        // or :has(~ .changed).
        let sibling_has_change = Change::from_selector_fact_change(SelectorFactChange::Attribute);
        assert!(sibling_has_change.scope.whole_tree);

        // A selector-list filter in :nth-child(... of .candidate) can reshuffle sibling matches.
        let structural_filter_change =
            Change::from_selector_fact_change(SelectorFactChange::Structure);
        assert!(structural_filter_change.scope.whole_tree);
    }
}
