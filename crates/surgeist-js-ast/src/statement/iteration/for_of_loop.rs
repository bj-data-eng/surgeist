use crate::operations::{DirectEvalAdmission, try_contains_direct_eval, unwrap_infallible};
use crate::scope::ScopeHandle;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    expression::Expression,
    statement::{Statement, iteration::IterableLoopInitializer},
};
use core::ops::ControlFlow;
use surgeist_js_interner::{Interner, ToIndentedString, ToInternedString};

/// A `for...of` loop statement, as defined by the [spec].
///
/// [`for..of`][forof] statements loop over a sequence of values obtained from an iterable object (Array,
/// String, Map, generators).
///
/// This type combines `for..of` and [`for await...of`][forawait] statements in a single structure,
/// since `for await...of` is essentially the same statement but with async iterable objects
/// as the source of iteration.
///
/// [forof]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
/// [forawait]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for-await...of
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForOfLoop {
    pub(crate) init: IterableLoopInitializer,
    pub(crate) iterable: Expression,
    pub(crate) body: Box<Statement>,
    r#await: bool,
    pub(crate) iterable_contains_direct_eval: bool,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) iterable_scope: Option<ScopeHandle>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Option<ScopeHandle>,
}

impl ForOfLoop {
    /// Creates a new "for of" loop AST node.
    #[inline]
    #[must_use]
    pub fn new(
        init: IterableLoopInitializer,
        iterable: Expression,
        body: Statement,
        r#await: bool,
    ) -> Self {
        unwrap_infallible(Self::try_new(init, iterable, body, r#await, None))
    }

    /// Constructs the node after admitting the visits needed for direct-eval metadata.
    ///
    /// # Errors
    /// Returns the first refused visit.
    #[inline]
    pub fn try_new<E>(
        init: IterableLoopInitializer,
        iterable: Expression,
        body: Statement,
        r#await: bool,
        mut admission: DirectEvalAdmission<'_, E>,
    ) -> Result<Self, E> {
        let iterable_contains_direct_eval = try_contains_direct_eval(&iterable, &mut admission)?;
        let contains_direct_eval = try_contains_direct_eval(&init, &mut admission)?
            || try_contains_direct_eval(&body, &mut admission)?;
        Ok(Self {
            init,
            iterable,
            body: body.into(),
            iterable_contains_direct_eval,
            contains_direct_eval,
            r#await,
            iterable_scope: None,
            scope: None,
        })
    }

    /// Gets the initializer of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn initializer(&self) -> &IterableLoopInitializer {
        &self.init
    }

    /// Gets the iterable expression of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn iterable(&self) -> &Expression {
        &self.iterable
    }

    /// Gets the body to execute in the for...of loop.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &Statement {
        &self.body
    }

    /// Returns true if this "for...of" loop is an "for await...of" loop.
    #[inline]
    #[must_use]
    pub const fn r#await(&self) -> bool {
        self.r#await
    }

    /// Return the iterable scope of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn iterable_scope(&self) -> Option<&ScopeHandle> {
        self.iterable_scope.as_ref()
    }

    /// Return the scope of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeHandle> {
        self.scope.as_ref()
    }
}

impl ToIndentedString for ForOfLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "for ({} of {}) {}",
            self.init.to_interned_string(interner),
            self.iterable.to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        )
    }
}

impl From<ForOfLoop> for Statement {
    #[inline]
    fn from(for_of: ForOfLoop) -> Self {
        Self::ForOfLoop(for_of)
    }
}

impl VisitWith for ForOfLoop {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_iterable_loop_initializer(&self.init)?;
        visitor.visit_expression(&self.iterable)?;
        visitor.visit_statement(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_iterable_loop_initializer_mut(&mut self.init)?;
        visitor.visit_expression_mut(&mut self.iterable)?;
        visitor.visit_statement_mut(&mut self.body)
    }
}
