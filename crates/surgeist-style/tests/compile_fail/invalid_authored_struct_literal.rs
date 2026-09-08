use surgeist_style::{
    AuthoredDeclaration, AuthoredDeclarations, AuthoredProperty, AuthoredValue, Color, Property,
    StyleColor, Value,
};

fn main() {
    let _declaration = AuthoredDeclaration {
        property: AuthoredProperty::Property(Property::Color),
        value: AuthoredValue::Value(Value::StyleColor(StyleColor::rgba(Color::BLACK))),
    };
    let _declarations = AuthoredDeclarations { values: Vec::new() };
}
