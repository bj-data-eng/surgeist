use surgeist_style::{Color, Declaration, Property, Value};

fn main() {
    let _declaration = Declaration {
        property: Property::Color,
        value: Value::Color(Color::BLACK),
    };
}
