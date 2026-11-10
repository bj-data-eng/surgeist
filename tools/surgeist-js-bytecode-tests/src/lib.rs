#[test]
fn compile_bytecode() {
    use insta::glob;
    use surgeist_js::{Context, Script, Source};

    glob!("../scripts/", "**/*.js", |path| {
        let context = &mut Context::builder().build().unwrap();
        let source = Source::from_filepath(path).expect("Could not load source");
        let script = Script::parse(source, None, context.access()).unwrap();
        let output = script.codeblock(context.access()).unwrap().to_string();
        insta::assert_snapshot!(output);
    });
}
