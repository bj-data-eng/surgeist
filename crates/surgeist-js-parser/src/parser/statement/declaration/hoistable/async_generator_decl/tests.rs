use crate::parser::tests::check_script_parser;
use surgeist_js_ast::{
    Declaration, LinearPosition, LinearSpan, Span, StatementList,
    expression::Identifier,
    function::{AsyncGeneratorDeclaration, FormalParameterList, FunctionBody},
};
use surgeist_js_interner::Interner;
use surgeist_js_macros::utf16;

#[test]
fn async_generator_function_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "async function* gen() {}",
        vec![
            Declaration::AsyncGeneratorDeclaration(AsyncGeneratorDeclaration::new(
                Identifier::new(
                    interner.get_or_intern_static("gen", utf16!("gen")),
                    Span::new((1, 17), (1, 20)),
                ),
                FormalParameterList::default(),
                FunctionBody::new(StatementList::default(), Span::new((1, 23), (1, 25))),
                LinearSpan::new(LinearPosition::default(), LinearPosition::default()),
            ))
            .into(),
        ],
        interner,
    );
}
