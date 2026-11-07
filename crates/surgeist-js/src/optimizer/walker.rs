use super::{
    PassAction,
    control::{OptimizerAdmission, OptimizerWork, admit},
};
use crate::{JsError, JsResult};
use std::ops::ControlFlow;
use surgeist_js_ast::{
    Expression,
    visitor::{VisitWith, VisitorMut},
};

/// A postorder pass. Refusal propagates through every default typed visit and
/// through the pass callback; a private partial rewrite is never executable.
pub(crate) struct Walker<'a, 'control, F> {
    f: F,
    admission: &'a mut OptimizerAdmission<'control>,
    changed: bool,
}

impl<'a, 'control, F> Walker<'a, 'control, F>
where
    F: FnMut(
        &mut Expression,
        &mut OptimizerAdmission<'control>,
    ) -> JsResult<PassAction<Expression>>,
{
    pub(crate) const fn new(admission: &'a mut OptimizerAdmission<'control>, f: F) -> Self {
        Self {
            f,
            admission,
            changed: false,
        }
    }

    pub(crate) const fn changed(&self) -> bool {
        self.changed
    }

    pub(crate) fn walk_expression_postorder(&mut self, expr: &mut Expression) -> JsResult<()> {
        match self.visit_expression_mut(expr) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(error) => Err(error),
        }
    }
}

impl<'ast, 'control, F> VisitorMut<'ast> for Walker<'_, 'control, F>
where
    F: FnMut(
        &mut Expression,
        &mut OptimizerAdmission<'control>,
    ) -> JsResult<PassAction<Expression>>,
{
    type BreakTy = JsError;

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        match admit(self.admission, OptimizerWork::Visit) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    fn visit_expression_mut(&mut self, expr: &'ast mut Expression) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        expr.visit_with_mut(self)?;
        match (self.f)(expr, self.admission) {
            Err(error) => ControlFlow::Break(error),
            Ok(action) => {
                match action {
                    PassAction::Keep => {}
                    PassAction::Modified => self.changed = true,
                    PassAction::Replace(new) => {
                        *expr = new;
                        self.changed = true;
                    }
                }
                ControlFlow::Continue(())
            }
        }
    }
}
