//! Private script optimization and its operation accounting.
//!
//! Enabled passes admit each typed visitor entry, each iteration (including the
//! stable check), each expression attempt and each rewrite before changing the
//! tree. Hoist searches have separate attribution and stop at their first match.
//! A replacement quotes discarded nodes before moving retained branches. These
//! retirement units are native cleanup reservations; mandatory destruction on
//! failure proceeds without requesting further authority.
//!
//! Literal bridges account for physical copies and storage requests. String,
//! BigInt and interner producers retain their own work units. Opaque numerical
//! providers use checked, operand-dependent reference prices, admitted before
//! entry, with a cancellation checkpoint before publication. Those prices are
//! neither internal instruction counts, physical-memory bounds nor deadlines;
//! opaque calls cannot be interrupted in the middle. See
//! [`crate::execution::ExecutionOptions::with_optimizer_work_limit`].
//!
//! A host refusal propagates out of private preparation and remains the first
//! operation stop even if a Rust callback catches it. A semantic abrupt
//! completion instead leaves the expression for ordinary runtime evaluation.

pub(crate) mod control;
pub(crate) mod pass;
pub(crate) mod retirement;
pub(crate) mod walker;

#[cfg(test)]
mod tests;

use self::control::{OptimizerAdmission, OptimizerWork, admit};
use self::{
    pass::{ConstantFolding, DeadCodeElimination, StrengthReduction},
    walker::Walker,
};
use crate::{JsError, JsResult};
use bitflags::bitflags;
use std::{fmt, ops::ControlFlow};
use surgeist_js_ast::{
    Expression, StatementList,
    statement::Statement,
    visitor::{VisitWith, VisitorMut},
};
use surgeist_js_interner::Interner;

bitflags! {
    /// Optimizer options.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct OptimizerOptions: u8 {
        /// Print statistics to `stdout`.
        const STATISTICS = 0b0000_0001;
        /// Apply constant folding optimization.
        const CONSTANT_FOLDING = 0b0000_0010;
        /// Apply strength reduction.
        const STRENGTH_REDUCTION = 0b0000_0100;
        /// Apply dead code elimination.
        const DEAD_CODE_ELIMINATION = 0b0000_1000;
        /// Apply all optimizations.
        const OPTIMIZE_ALL = Self::CONSTANT_FOLDING.bits()
            | Self::STRENGTH_REDUCTION.bits()
            | Self::DEAD_CODE_ELIMINATION.bits();
    }
}

/// The action to be performed after an optimization step.
#[derive(Debug)]
pub(crate) enum PassAction<T> {
    /// Keep the node, do nothing.
    Keep,
    /// The node was modified inplace.
    Modified,
    /// Replace the node.
    Replace(T),
}

/// Contains statistics about the optimizer execution.
#[derive(Debug, Default, Clone, Copy)]
pub struct OptimizerStatistics {
    /// Constant folding run count.
    pub constant_folding_run_count: usize,
    /// Constant folding pass count.
    pub constant_folding_pass_count: usize,
    /// Strength reduction run count.
    pub strength_reduction_run_count: usize,
    /// Strength reduction pass count.
    pub strength_reduction_pass_count: usize,
    /// Dead code elimination count.
    pub dead_code_elimination_count: usize,
}

impl fmt::Display for OptimizerStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Optimizer {{")?;
        writeln!(
            f,
            "    constant folding: {} run(s), {} pass(es) ({} mutating, {} checking)",
            self.constant_folding_run_count,
            self.constant_folding_pass_count,
            self.constant_folding_pass_count
                .saturating_sub(self.constant_folding_run_count),
            self.constant_folding_run_count
        )?;
        writeln!(
            f,
            "    strength reduction: {} run(s), {} pass(es)",
            self.strength_reduction_run_count, self.strength_reduction_pass_count,
        )?;
        writeln!(
            f,
            "    dead code elimination: {} elimination(s)",
            self.dead_code_elimination_count,
        )?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// This represents an AST optimizer.
pub(crate) struct Optimizer<'interner, 'control> {
    statistics: OptimizerStatistics,
    options: OptimizerOptions,
    admission: OptimizerAdmission<'control>,
    interner: &'interner mut Interner,
}

impl<'interner, 'control> Optimizer<'interner, 'control> {
    /// Create a optimizer.
    pub(crate) fn new(
        options: OptimizerOptions,
        interner: &'interner mut Interner,
        admission: OptimizerAdmission<'control>,
    ) -> Self {
        Self {
            statistics: OptimizerStatistics::default(),
            options,
            admission,
            interner,
        }
    }

    /// Run the constant folding optimization on an expression.
    fn run_constant_folding_pass(&mut self, expr: &mut Expression) -> JsResult<bool> {
        self.statistics.constant_folding_run_count += 1;

        let mut has_changes = false;
        for _ in 0..Self::MAX_PASS_ITERATIONS {
            admit(&mut self.admission, OptimizerWork::Iteration)?;
            self.statistics.constant_folding_pass_count += 1;
            let mut walker = Walker::new(&mut self.admission, |expr, admission| {
                ConstantFolding::fold_expression(expr, self.interner, admission)
            });
            // NOTE: postoder traversal is optimal for constant folding,
            // since it evaluates the tree bottom-up.
            walker.walk_expression_postorder(expr)?;
            if !walker.changed() {
                break;
            }
            has_changes = true;
        }
        Ok(has_changes)
    }

    /// Maximum number of iterations for optimization passes.
    /// This prevents infinite loops if a pass has a bug that keeps producing changes.
    const MAX_PASS_ITERATIONS: usize = 10;

    /// Run the strength reduction optimization on an expression.
    fn run_strength_reduction_pass(&mut self, expr: &mut Expression) -> JsResult<bool> {
        self.statistics.strength_reduction_run_count += 1;

        let mut has_changes = false;
        for _ in 0..Self::MAX_PASS_ITERATIONS {
            admit(&mut self.admission, OptimizerWork::Iteration)?;
            self.statistics.strength_reduction_pass_count += 1;
            let mut walker = Walker::new(&mut self.admission, |expr, admission| {
                StrengthReduction::reduce_expression(expr, admission)
            });
            walker.walk_expression_postorder(expr)?;
            if !walker.changed() {
                break;
            }
            has_changes = true;
        }
        Ok(has_changes)
    }

    fn run_all(&mut self, expr: &mut Expression) -> JsResult<()> {
        if self.options.contains(OptimizerOptions::CONSTANT_FOLDING) {
            self.run_constant_folding_pass(expr)?;
        }
        if self.options.contains(OptimizerOptions::STRENGTH_REDUCTION) {
            self.run_strength_reduction_pass(expr)?;
        }
        Ok(())
    }

    /// Apply optimizations inplace.
    pub(crate) fn apply(
        &mut self,
        statement_list: &mut StatementList,
    ) -> JsResult<OptimizerStatistics> {
        if let ControlFlow::Break(error) = self.visit_statement_list_mut(statement_list) {
            return Err(error);
        }

        #[allow(clippy::print_stdout)]
        if self.options.contains(OptimizerOptions::STATISTICS) {
            println!("{}", self.statistics);
        }
        Ok(self.statistics)
    }
}

impl<'ast> VisitorMut<'ast> for Optimizer<'_, '_> {
    type BreakTy = JsError;

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        match admit(&mut self.admission, OptimizerWork::Visit) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    fn visit_expression_mut(&mut self, node: &'ast mut Expression) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match self.run_all(node) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    fn visit_statement_mut(&mut self, node: &'ast mut Statement) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        // Keep the existing pass coverage: expression traversal belongs to the
        // enabled expression passes, including statement bodies reached by them.
        node.visit_with_mut(self)?;
        if self
            .options
            .contains(OptimizerOptions::DEAD_CODE_ELIMINATION)
        {
            match DeadCodeElimination::try_eliminate(node, &mut self.admission) {
                Ok(true) => self.statistics.dead_code_elimination_count += 1,
                Ok(false) => {}
                Err(error) => return ControlFlow::Break(error),
            }
        }
        ControlFlow::Continue(())
    }
}
