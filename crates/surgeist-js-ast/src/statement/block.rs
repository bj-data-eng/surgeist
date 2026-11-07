//! Block AST node.

use crate::{
    Statement, StatementList,
    operations::{DirectEvalAdmission, try_contains_direct_eval, unwrap_infallible},
    scope::ScopeHandle,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use core::ops::ControlFlow;
use surgeist_js_interner::{Interner, ToIndentedString};

/// A `block` statement (or compound statement in other languages) is used to group zero or
/// more statements.
///
/// The block statement is often called compound statement in other languages.
/// It allows you to use multiple statements where ECMAScript expects only one statement.
/// Combining statements into blocks is a common practice in ECMAScript. The opposite behavior
/// is possible using an empty statement, where you provide no statement, although one is
/// required.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Block {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub(crate) statements: StatementList,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Option<ScopeHandle>,
}

impl Block {
    /// Constructs a block after admitting the direct-eval metadata visits.
    ///
    /// # Errors
    /// Returns the first refused visit.
    #[inline]
    pub fn try_from_statement_list<E>(
        statements: StatementList,
        mut admission: DirectEvalAdmission<'_, E>,
    ) -> Result<Self, E> {
        let contains_direct_eval = try_contains_direct_eval(&statements, &mut admission)?;
        Ok(Self {
            statements,
            scope: None,
            contains_direct_eval,
        })
    }

    /// Gets the list of statements and declarations in this block.
    #[inline]
    #[must_use]
    pub const fn statement_list(&self) -> &StatementList {
        &self.statements
    }

    /// Gets the scope of the block.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeHandle> {
        self.scope.as_ref()
    }
}

impl<T> From<T> for Block
where
    T: Into<StatementList>,
{
    fn from(list: T) -> Self {
        unwrap_infallible(Self::try_from_statement_list(list.into(), None))
    }
}

impl ToIndentedString for Block {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "{{\n{}{}}}",
            self.statements
                .to_indented_string(interner, indentation + 1),
            "    ".repeat(indentation)
        )
    }
}

impl From<Block> for Statement {
    #[inline]
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}

impl VisitWith for Block {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_statement_list(&self.statements)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_statement_list_mut(&mut self.statements)
    }
}
