use surgeist_style::{
    AuthoredTokens, CustomPropertyName, Property, StyleColor, Value, VariableDependentValue,
    VariableExpression,
};

fn main() {
    let _value = VariableDependentValue {
        property: Property::Color,
        authored: AuthoredTokens::new("var(--brand)"),
        expression: VariableExpression::Value(Value::StyleColor(StyleColor::rgba(surgeist_style::Color::BLACK))),
        dependencies: vec![CustomPropertyName::try_new("--brand").unwrap()],
    };
}
