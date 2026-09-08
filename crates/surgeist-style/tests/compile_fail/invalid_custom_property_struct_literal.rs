use surgeist_style::{AuthoredTokens, CustomPropertyValue};

fn main() {
    let _value = CustomPropertyValue {
        authored: AuthoredTokens::new("8px"),
        references: Vec::new(),
        dependencies: Vec::new(),
        typed_values: std::collections::BTreeMap::new(),
    };
}
