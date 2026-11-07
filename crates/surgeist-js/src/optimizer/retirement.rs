//! Prepaid cleanup for subtrees discarded by an optimizer rewrite.
//!
//! Reservations price typed nodes and optional sequence slots, rather than
//! claiming to count Rust destructor calls. A quote completes before ownership
//! changes, so refusal preserves the original AST and cleanup never calls back
//! into a stopped host.

use super::control::{OptimizerAdmission, OptimizerWork, admit};
use crate::{JsError, JsResult};
use core::ops::ControlFlow;
use surgeist_js_ast::{Expression, Statement, visitor::Visitor};

struct RetirementVisitor<'borrow, 'host> {
    admission: &'borrow mut OptimizerAdmission<'host>,
}

impl<'ast> Visitor<'ast> for RetirementVisitor<'_, '_> {
    type BreakTy = JsError;

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        match container(self.admission) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }
}

/// Quotes one removed wrapper without traversing any retained descendants.
pub(super) fn container(admission: &mut OptimizerAdmission<'_>) -> JsResult<()> {
    admit(admission, OptimizerWork::RetirementVisit)?;
    admit(admission, OptimizerWork::RetirementReservation)
}

pub(super) fn expression(
    expression: &Expression,
    admission: &mut OptimizerAdmission<'_>,
) -> JsResult<()> {
    if admission.is_none() {
        return Ok(());
    }
    match (RetirementVisitor { admission }).visit_expression(expression) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}

pub(super) fn statement(
    statement: &Statement,
    admission: &mut OptimizerAdmission<'_>,
) -> JsResult<()> {
    if admission.is_none() {
        return Ok(());
    }
    match (RetirementVisitor { admission }).visit_statement(statement) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}
