use surgeist_style::{Color, Metadata, Property, Value};

fn main() {
    let _metadata = Metadata::new(Value::Color(Color::BLACK));

    let mut metadata = Property::Color.metadata();
    metadata.default = Value::Number(3.0);
    metadata.inherited = false;
    metadata.impact.layout = false;
}
