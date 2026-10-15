use crate::expr::{BinaryOp, Expr, Literal, PathSegment, UnaryOp, VariablePath};
use crate::span::{SourcePos, SourceSpan};
use crate::validate::{
    ValidatedAttrPart, ValidatedAttrValue, ValidatedAttribute, ValidatedElement, ValidatedForEach,
    ValidatedIf, ValidatedNode, ValidatedTemplate,
};

const TEMPLATE: &str = "::surgeist::template";

/// Emits a Rust expression constructing an authored TemplateDocument through
/// the facade's real public AST constructors. Names, symbolic expressions and
/// diagnostic spans are retained; the result requires registry validation.
///
/// This operation neither evaluates expressions nor constructs a live DOM.
pub fn render_to_rust(template: &ValidatedTemplate) -> String {
    format!(
        "{TEMPLATE}::TemplateDocument::from_nodes(::std::vec![{}])",
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
            "{TEMPLATE}::Node::text({}, {})",
            render_string(text.value()),
            render_span(text.span())
        ),
        ValidatedNode::Interpolation(interpolation) => format!(
            "{TEMPLATE}::Node::interpolation({}, {})",
            render_expr(interpolation.expr()),
            render_span(interpolation.span())
        ),
        ValidatedNode::If(conditional) => render_if(conditional),
        ValidatedNode::ForEach(iteration) => render_for_each(iteration),
    }
}

fn render_element<Name>(element: &ValidatedElement<Name>, name: &str, kind: &str) -> String {
    format!(
        "{TEMPLATE}::Node::element({TEMPLATE}::ElementName::{kind}({}).expect(\"validated element name\"), ::std::vec![{}], ::std::vec![{}], {})",
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
        ValidatedAttrValue::Bool => format!("{TEMPLATE}::AttrValue::Bool"),
        ValidatedAttrValue::Static(value) => {
            format!("{TEMPLATE}::AttrValue::Static({})", owned_string(value))
        }
        ValidatedAttrValue::Expression(expr) => {
            format!("{TEMPLATE}::AttrValue::Expression({})", render_expr(expr))
        }
        ValidatedAttrValue::Interpolated(parts) => format!(
            "{TEMPLATE}::AttrValue::Interpolated(::std::vec![{}])",
            parts
                .iter()
                .map(|part| match part {
                    ValidatedAttrPart::Text(value) =>
                        format!("{TEMPLATE}::AttrPart::Text({})", owned_string(value)),
                    ValidatedAttrPart::Expr(expr) =>
                        format!("{TEMPLATE}::AttrPart::Expr({})", render_expr(expr)),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    format!(
        "{TEMPLATE}::Attribute::try_new({}, {value}, {}).expect(\"validated attribute name\")",
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
                "{TEMPLATE}::IfBranch::new({}, ::std::vec![{}])",
                render_expr(branch.condition()),
                render_nodes(branch.children())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{TEMPLATE}::Node::if_block(::std::vec![{branches}], ::std::vec![{}], {}).expect(\"conditional has an initial branch\")",
        render_nodes(conditional.else_children()),
        render_span(conditional.span())
    )
}

fn render_for_each(iteration: &ValidatedForEach) -> String {
    format!(
        "{TEMPLATE}::Node::foreach({}, {TEMPLATE}::VariableName::try_new({}).expect(\"validated iteration binding\"), ::std::vec![{}], ::std::vec![{}], {})",
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
            format!("{TEMPLATE}::Expr::Literal({TEMPLATE}::Literal::{value})")
        }
        Expr::Variable(path) => render_path(path),
        Expr::Unary { op, expr } => format!(
            "{TEMPLATE}::Expr::Unary {{ op: {TEMPLATE}::UnaryOp::{}, expr: ::std::boxed::Box::new({}) }}",
            match op {
                UnaryOp::Not => "Not",
                UnaryOp::Neg => "Neg",
            },
            render_expr(expr)
        ),
        Expr::Binary { op, left, right } => format!(
            "{TEMPLATE}::Expr::Binary {{ op: {TEMPLATE}::BinaryOp::{}, left: ::std::boxed::Box::new({}), right: ::std::boxed::Box::new({}) }}",
            binary_variant(*op),
            render_expr(left),
            render_expr(right)
        ),
    }
}

fn render_path(path: &VariablePath) -> String {
    let segments = path.segments().iter().map(|segment| match segment {
        PathSegment::Field(field) => format!(
            "{TEMPLATE}::PathSegment::Field({TEMPLATE}::PathField::try_new({}).expect(\"validated path field\"))",
            render_string(field.as_str())
        ),
        PathSegment::Index(index) => format!("{TEMPLATE}::PathSegment::Index({TEMPLATE}::PathIndex::new({}u64))", index.get()),
    }).collect::<Vec<_>>().join(", ");
    format!(
        "{TEMPLATE}::Expr::Variable({TEMPLATE}::VariablePath::try_new({}, ::std::vec![{segments}]).expect(\"validated symbolic path\"))",
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
        "{TEMPLATE}::SourceSpan::try_new({}, {}).expect(\"ordered template span\")",
        render_position(span.start()),
        render_position(span.end())
    )
}

fn render_position(position: SourcePos) -> String {
    format!(
        "{TEMPLATE}::SourcePos::try_new({}usize, {}usize, {}usize).expect(\"nonzero template coordinates\")",
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
    use crate::{Expr, Literal};

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
                    "::surgeist::template::Expr::Literal(::surgeist::template::Literal::Float(::core::primitive::f64::from_bits({bits}u64)))"
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
            r#"::surgeist::template::Expr::Literal(::surgeist::template::Literal::String(::std::string::String::from("Ω\n\"quoted\"\\tail\0")))"#
        );
    }
}
