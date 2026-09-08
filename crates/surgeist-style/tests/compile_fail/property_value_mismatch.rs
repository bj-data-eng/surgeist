use surgeist_style::{Color, Declarations, Property, Value};

fn main() {
    let mut declarations = Declarations::new();
    let _ = declarations.insert(Property::Width, Value::Color(Color::BLACK));
}
