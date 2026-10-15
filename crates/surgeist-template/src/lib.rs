//! Template and DSL-facing contracts for Surgeist.
//!
//! Strict parsing, native authored AST construction, registry validation and
//! symbolic Rust generation. Template owns its direct lower-level adapters;
//! runtime evaluation and host policy belong to their consuming owners.

#![forbid(unsafe_code)]

mod ast;
mod error;
mod expr;
mod lexer;
mod name;
mod parser;
mod render;
mod span;
mod validate;

pub use ast::{
    AstConstructionError, AttrPart, AttrValue, Attribute, ElementName, ElementNode, ForEachNode,
    IfBranch, IfNode, InterpolationNode, Node, TemplateDocument, TextNode,
};
pub use error::{ParseError, ParseErrorKind, ValidationError, ValidationErrorKind};
pub use expr::{BinaryOp, Expr, Literal, PathField, PathIndex, PathSegment, UnaryOp, VariablePath};
pub use name::{AttributeName, ComponentName, NameError, NativeElementName, VariableName};
pub use parser::parse_template;
pub use render::render_to_rust;
pub use span::{SourcePos, SourceSpan, SpanError};
pub use validate::{
    AttributeKind, AttributeRule, AttributeSpec, ComponentRegistry, ComponentSpec,
    NativeElementRegistry, NativeElementSpec, RegistryError, ValidatedAttrPart, ValidatedAttrValue,
    ValidatedAttribute, ValidatedElement, ValidatedForEach, ValidatedIf, ValidatedIfBranch,
    ValidatedInterpolation, ValidatedNode, ValidatedTemplate, ValidatedText, validate_template,
};

/// Crate identity string used by smoke tests and API artifacts.
pub const CRATE_NAME: &str = "surgeist-template";

#[cfg(test)]
mod tests {
    use super::CRATE_NAME;

    #[test]
    fn exposes_crate_identity() {
        assert_eq!(CRATE_NAME, "surgeist-template");
    }
}
