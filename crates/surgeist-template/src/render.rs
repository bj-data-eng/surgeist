use crate::expr::{BinaryOp, Expr, Literal, PathSegment, UnaryOp, VariablePath};
use crate::span::{SourcePos, SourceSpan};
use crate::validate::{
    ValidatedAttrPart, ValidatedAttrValue, ValidatedAttribute, ValidatedElement, ValidatedForEach,
    ValidatedIf, ValidatedNode, ValidatedTemplate,
};

const AST: &str = "::surgeist::template::ast";
const EXPR: &str = "::surgeist::template::expr";
const NAME: &str = "::surgeist::template::name";
const SPAN: &str = "::surgeist::template::span";

/// Emits a Rust expression constructing an authored TemplateDocument through
/// the facade's real public AST constructors. Items are named through the
/// `ast`, `expr`, `name` and `span` modules under `::surgeist::template`.
/// Names, symbolic expressions and diagnostic spans are retained; the result
/// requires registry validation.
///
/// This operation neither evaluates expressions nor constructs a live DOM.
pub fn render_to_rust(template: &ValidatedTemplate) -> String {
    format!(
        "{AST}::TemplateDocument::from_nodes(::std::vec![{}])",
        render_nodes(template.nodes())
    )
}

fn render_nodes(nodes: &[ValidatedNode]) -> String {
    nodes.iter().map(render_node).collect::<Vec<_>>().join(", ")
}

fn render_node(node: &ValidatedNode) -> String {
    match node {
        ValidatedNode::NativeElement(element) => {
            render_element(element, element.name().as_str(), "native")
        }
        ValidatedNode::ComponentElement(element) => {
            render_element(element, element.name().as_str(), "component")
        }
        ValidatedNode::Text(text) => format!(
            "{AST}::Node::text({}, {})",
            render_string(text.value()),
            render_span(text.span())
        ),
        ValidatedNode::Interpolation(interpolation) => format!(
            "{AST}::Node::interpolation({}, {})",
            render_expr(interpolation.expr()),
            render_span(interpolation.span())
        ),
        ValidatedNode::If(conditional) => render_if(conditional),
        ValidatedNode::ForEach(iteration) => render_for_each(iteration),
    }
}

fn render_element<Name>(element: &ValidatedElement<Name>, name: &str, kind: &str) -> String {
    format!(
        "{AST}::Node::element({AST}::ElementName::{kind}({}).expect(\"validated element name\"), ::std::vec![{}], ::std::vec![{}], {})",
        render_string(name),
        element
            .attributes()
            .iter()
            .map(render_attribute)
            .collect::<Vec<_>>()
            .join(", "),
        render_nodes(element.children()),
        render_span(element.span())
    )
}

fn render_attribute(attribute: &ValidatedAttribute) -> String {
    let value = match attribute.value() {
        ValidatedAttrValue::Bool => format!("{AST}::AttrValue::Bool"),
        ValidatedAttrValue::Static(value) => {
            format!("{AST}::AttrValue::Static({})", owned_string(value))
        }
        ValidatedAttrValue::Expression(expr) => {
            format!("{AST}::AttrValue::Expression({})", render_expr(expr))
        }
        ValidatedAttrValue::Interpolated(parts) => format!(
            "{AST}::AttrValue::Interpolated(::std::vec![{}])",
            parts
                .iter()
                .map(|part| match part {
                    ValidatedAttrPart::Text(value) =>
                        format!("{AST}::AttrPart::Text({})", owned_string(value)),
                    ValidatedAttrPart::Expr(expr) =>
                        format!("{AST}::AttrPart::Expr({})", render_expr(expr)),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    format!(
        "{AST}::Attribute::try_new({}, {value}, {}).expect(\"validated attribute name\")",
        render_string(attribute.name().as_str()),
        render_span(attribute.span())
    )
}

fn render_if(conditional: &ValidatedIf) -> String {
    let branches = conditional
        .branches()
        .iter()
        .map(|branch| {
            format!(
                "{AST}::IfBranch::new({}, ::std::vec![{}])",
                render_expr(branch.condition()),
                render_nodes(branch.children())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{AST}::Node::if_block(::std::vec![{branches}], ::std::vec![{}], {}).expect(\"conditional has an initial branch\")",
        render_nodes(conditional.else_children()),
        render_span(conditional.span())
    )
}

fn render_for_each(iteration: &ValidatedForEach) -> String {
    format!(
        "{AST}::Node::foreach({}, {NAME}::VariableName::try_new({}).expect(\"validated iteration binding\"), ::std::vec![{}], ::std::vec![{}], {})",
        render_expr(iteration.collection()),
        render_string(iteration.item_name().as_str()),
        render_nodes(iteration.children()),
        render_nodes(iteration.else_children()),
        render_span(iteration.span())
    )
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(literal) => {
            let value = match literal {
                Literal::String(value) => format!("String({})", owned_string(value)),
                Literal::Int(value) => format!("Int({value}i64)"),
                Literal::Float(value) => format!(
                    "Float(::core::primitive::f64::from_bits({}u64))",
                    value.to_bits()
                ),
                Literal::Bool(value) => format!("Bool({value})"),
                Literal::Null => "Null".to_owned(),
            };
            format!("{EXPR}::Expr::Literal({EXPR}::Literal::{value})")
        }
        Expr::Variable(path) => render_path(path),
        Expr::Unary { op, expr } => format!(
            "{EXPR}::Expr::Unary {{ op: {EXPR}::UnaryOp::{}, expr: ::std::boxed::Box::new({}) }}",
            match op {
                UnaryOp::Not => "Not",
                UnaryOp::Neg => "Neg",
            },
            render_expr(expr)
        ),
        Expr::Binary { op, left, right } => format!(
            "{EXPR}::Expr::Binary {{ op: {EXPR}::BinaryOp::{}, left: ::std::boxed::Box::new({}), right: ::std::boxed::Box::new({}) }}",
            binary_variant(*op),
            render_expr(left),
            render_expr(right)
        ),
    }
}

fn render_path(path: &VariablePath) -> String {
    let segments = path.segments().iter().map(|segment| match segment {
        PathSegment::Field(field) => format!(
            "{EXPR}::PathSegment::Field({EXPR}::PathField::try_new({}).expect(\"validated path field\"))",
            render_string(field.as_str())
        ),
        PathSegment::Index(index) => format!("{EXPR}::PathSegment::Index({EXPR}::PathIndex::new({}u64))", index.get()),
    }).collect::<Vec<_>>().join(", ");
    format!(
        "{EXPR}::Expr::Variable({EXPR}::VariablePath::try_new({}, ::std::vec![{segments}]).expect(\"validated symbolic path\"))",
        render_string(path.root())
    )
}

fn binary_variant(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Or => "Or",
        BinaryOp::And => "And",
        BinaryOp::Eq => "Eq",
        BinaryOp::Ne => "Ne",
        BinaryOp::Gt => "Gt",
        BinaryOp::Ge => "Ge",
        BinaryOp::Lt => "Lt",
        BinaryOp::Le => "Le",
        BinaryOp::Add => "Add",
        BinaryOp::Sub => "Sub",
        BinaryOp::Mul => "Mul",
        BinaryOp::Div => "Div",
        BinaryOp::Rem => "Rem",
    }
}

fn render_span(span: SourceSpan) -> String {
    format!(
        "{SPAN}::SourceSpan::try_new({}, {}).expect(\"ordered template span\")",
        render_position(span.start()),
        render_position(span.end())
    )
}

fn render_position(position: SourcePos) -> String {
    format!(
        "{SPAN}::SourcePos::try_new({}usize, {}usize, {}usize).expect(\"nonzero template coordinates\")",
        position.line(),
        position.column(),
        position.byte()
    )
}

fn owned_string(value: &str) -> String {
    format!("::std::string::String::from({})", render_string(value))
}

fn render_string(value: &str) -> String {
    // Rust's string Debug representation is a quoted, escaped Rust literal.
    format!("{value:?}")
}

#[cfg(test)]
mod tests {
    use super::render_expr;
    use crate::expr::{Expr, Literal};

    #[test]
    fn float_source_uses_exact_bits_for_negative_zero_and_nonfinite_payloads() {
        for bits in [
            0x8000_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_0042,
        ] {
            assert_eq!(
                render_expr(&Expr::Literal(Literal::Float(f64::from_bits(bits)))),
                format!(
                    "::surgeist::template::expr::Expr::Literal(::surgeist::template::expr::Literal::Float(::core::primitive::f64::from_bits({bits}u64)))"
                )
            );
        }
    }

    #[test]
    fn string_literal_source_retains_controls_quotes_backslashes_and_unicode() {
        assert_eq!(
            render_expr(&Expr::Literal(Literal::String(
                "Ω\n\"quoted\"\\tail\0".to_owned()
            ))),
            r#"::surgeist::template::expr::Expr::Literal(::surgeist::template::expr::Literal::String(::std::string::String::from("Ω\n\"quoted\"\\tail\0")))"#
        );
    }
}
