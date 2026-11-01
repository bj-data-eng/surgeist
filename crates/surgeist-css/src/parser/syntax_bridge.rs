//! The diagnostics for the failures of building the source's token tree.
//!
//! The units of a rule list are read from the tree by the parser crate's
//! drivers (see [`rule_list`](super::rule_list)); this module reports a tree
//! that could not be built.
use crate::diagnostic::error::from_parse_error;
use crate::parser::recovery::recovery_action_for_error;

pub(super) fn arena_error(
    source: &str,
    error: crate::component::CssComponentValueError,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    component_error_diagnostic(
        source,
        error,
        crate::source::CssSourceSpan::new(
            crate::source::CssSourcePosition::from_byte_offset_in(source, 0),
            crate::source::CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("complete source span"),
        crate::diagnostic::CssRecoveryAction::DropQualifiedRule,
    )
}

pub(super) fn component_error_diagnostic(
    source: &str,
    error: crate::component::CssComponentValueError,
    span: crate::source::CssSourceSpan,
    ordinary_action: crate::diagnostic::CssRecoveryAction,
) -> crate::diagnostic::CssRecoveryDiagnostic {
    let position = match error.origin() {
        crate::component::CssValueOrigin::Parsed(origin) => origin.span().start(),
        _ => crate::source::CssSourcePosition::from_byte_offset_in(source, 0),
    };
    let parsed = crate::diagnostic::error::invalid_component_value(
        cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        },
        error,
    );
    let action = recovery_action_for_error(&parsed, ordinary_action);
    crate::diagnostic::CssRecoveryDiagnostic::new(from_parse_error(source, parsed), span, action)
        .expect("arena failure belongs to this source")
}
