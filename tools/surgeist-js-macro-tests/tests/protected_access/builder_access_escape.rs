use surgeist_js::{Context, Source, object::ObjectInitializer};

fn main() {
    let mut owner = Context::default();
    let mut access = owner.access();
    let mut builder = ObjectInitializer::new(access.reborrow());
    let mut escaped = builder.context();
    let _ = builder.build();
    escaped.eval(Source::from_bytes("42")).unwrap();
}
