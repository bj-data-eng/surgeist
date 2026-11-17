#![forbid(unsafe_code)]

//! The generated Rust names the facade module that owns each item. These
//! expectations come from the module plan (`ast`, `expr`, `name` and `span`
//! own the emitted items), not from the renderer's output.

use surgeist_template::registry::{
    AttributeKind, AttributeRule, AttributeSpec, ComponentRegistry, ComponentSpec,
    NativeElementRegistry, NativeElementSpec,
};
use surgeist_template::{parse_template, render_to_rust, validate_template};

const PREFIX: &str = "::surgeist::template::";

fn render(source: &str) -> String {
    let spec = |name, kind| AttributeSpec::try_new(name, AttributeRule::one(kind)).unwrap();
    let native = NativeElementRegistry::try_from_specs(vec![
        NativeElementSpec::try_new("div", vec![spec("class", AttributeKind::Static)]).unwrap(),
    ])
    .unwrap();
    let components = ComponentRegistry::try_from_specs(vec![
        ComponentSpec::try_new(
            "Panel",
            vec![
                spec("disabled", AttributeKind::Bool),
                spec("title", AttributeKind::Interpolated),
                spec("value", AttributeKind::Expression),
            ],
        )
        .unwrap(),
    ])
    .unwrap();
    let authored = parse_template(source).expect("strict template");
    let validated =
        validate_template(&authored, &native, &components).expect("registries validate");
    render_to_rust(&validated)
}

const EVERY_CONSTRUCT: &str = concat!(
    r#"<Panel disabled title="Hi {$user.name}" value={$match}>"#,
    "text{$user.items[2].name + 3 * -4}",
    r#"{if !$visible || false}<div class="raw">yes</div>{elseif null}nil{else}no{/if}"#,
    r#"{if $a == "s" && 2.5 > $b}f{/if}"#,
    "{foreach $items as $item}{$item}{foreachelse}empty{/foreach}</Panel>",
);

#[test]
fn text_only_template_names_the_ast_and_span_modules() {
    assert_eq!(
        render("x"),
        concat!(
            "::surgeist::template::ast::TemplateDocument::from_nodes(::std::vec![",
            "::surgeist::template::ast::Node::text(\"x\", ",
            "::surgeist::template::span::SourceSpan::try_new(",
            "::surgeist::template::span::SourcePos::try_new(1usize, 1usize, 0usize)",
            ".expect(\"nonzero template coordinates\"), ",
            "::surgeist::template::span::SourcePos::try_new(1usize, 2usize, 1usize)",
            ".expect(\"nonzero template coordinates\"))",
            ".expect(\"ordered template span\"))])"
        )
    );
}

#[test]
fn every_emitted_facade_path_starts_with_its_owning_module() {
    let generated = render(EVERY_CONSTRUCT);
    let mut checked = 0;
    for (index, _) in generated.match_indices(PREFIX) {
        let rest = &generated[index + PREFIX.len()..];
        assert!(
            ["ast::", "expr::", "name::", "span::"]
                .iter()
                .any(|module| rest.starts_with(module)),
            "emitted path is not under an owning module: {}",
            &generated[index..generated.len().min(index + 80)]
        );
        checked += 1;
    }
    assert!(
        checked > 100,
        "expected many emitted paths, found {checked}"
    );
}

#[test]
fn each_emitted_item_is_named_through_the_module_that_owns_it() {
    let generated = render(EVERY_CONSTRUCT);
    for expected in [
        "ast::TemplateDocument::from_nodes(",
        "ast::Node::element(",
        "ast::Node::text(",
        "ast::Node::interpolation(",
        "ast::Node::if_block(",
        "ast::Node::foreach(",
        "ast::ElementName::component(",
        "ast::ElementName::native(",
        "ast::Attribute::try_new(",
        "ast::AttrValue::Bool",
        "ast::AttrValue::Static(",
        "ast::AttrValue::Expression(",
        "ast::AttrValue::Interpolated(",
        "ast::AttrPart::Text(",
        "ast::AttrPart::Expr(",
        "ast::IfBranch::new(",
        "name::VariableName::try_new(",
        "expr::Expr::Literal(",
        "expr::Expr::Variable(",
        "expr::Expr::Unary {",
        "expr::Expr::Binary {",
        "expr::Literal::String(",
        "expr::Literal::Int(",
        "expr::Literal::Float(",
        "expr::Literal::Bool(",
        "expr::Literal::Null",
        "expr::UnaryOp::Not",
        "expr::UnaryOp::Neg",
        "expr::BinaryOp::Add",
        "expr::BinaryOp::Mul",
        "expr::BinaryOp::Or",
        "expr::BinaryOp::And",
        "expr::BinaryOp::Eq",
        "expr::BinaryOp::Gt",
        "expr::PathSegment::Field(",
        "expr::PathSegment::Index(",
        "expr::PathField::try_new(",
        "expr::PathIndex::new(",
        "expr::VariablePath::try_new(",
        "span::SourceSpan::try_new(",
        "span::SourcePos::try_new(",
    ] {
        let path = format!("{PREFIX}{expected}");
        assert!(generated.contains(&path), "missing emitted path {path}");
    }
}
