//! Recoverable descriptor-body traversal; family grammar and finalization stay
//! with the receiver. Font-face's fatal structural policy and counter-style's
//! combination-validation traversal deliberately remain separate.

use cssparser::{Parser, RuleBodyItemParser, RuleBodyParser};

use super::recovery::{RecoveryLoopOutcome, RecoveryProgress, RecoveryState};
use super::{consume_failed_rule_block, structural_rule_diagnostic};
use crate::error::Error;
use crate::{CssRecoveryAction, CssRecoveryDiagnostic};

pub(super) struct RecoveryContext<'a, 'i> {
    pub(super) source: &'i str,
    pub(super) recovery: &'a RecoveryState,
    pub(super) diagnostics: &'a mut Vec<CssRecoveryDiagnostic>,
}

/// Borrow the family's original source, accounting state, and diagnostic sink
/// together only after its item parser has released its mutable borrow.
pub(super) trait Receiver<'i, Item>: RuleBodyItemParser<'i, Item, Error> {
    fn recovery_context(&mut self) -> RecoveryContext<'_, 'i>;
}

pub(super) fn parse<'i, Item>(
    input: &mut Parser<'i, '_>,
    parser: &mut impl Receiver<'i, Item>,
    production: &'static str,
) -> Vec<Item> {
    let mut result = Vec::new();
    let mut previous_end = input.position().byte_index();
    let mut items = RuleBodyParser::new(input, parser);
    loop {
        let progress = RecoveryProgress::record(items.input);
        let Some(item) = items.next() else {
            break;
        };
        let context = items.parser.recovery_context();
        let failed_error = item.as_ref().err().and_then(|_| {
            consume_failed_rule_block(
                context.source,
                items.input,
                true,
                context.recovery,
                production,
            )
            .1
        });
        let outcome = progress.finish(items.input, item.is_ok());
        let end = items.input.position().byte_index();
        match item {
            Ok(member) => result.push(member),
            Err((error, unit)) => {
                let action = if unit.trim_start().starts_with('@') {
                    CssRecoveryAction::DropAtRule
                } else {
                    CssRecoveryAction::DropDescriptor
                };
                if let Some(diagnostic) = structural_rule_diagnostic(
                    context.source,
                    failed_error.unwrap_or(error),
                    unit,
                    previous_end,
                    end,
                    action,
                ) {
                    context.diagnostics.push(diagnostic);
                }
            }
        }
        previous_end = end;
        if outcome == RecoveryLoopOutcome::Terminated {
            break;
        }
    }
    result
}
