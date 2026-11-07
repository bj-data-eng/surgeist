//! Definitions of various **Syntax-Directed Operations** used in the [spec].
//!
//! [spec]: https://tc39.es/ecma262/#sec-syntax-directed-operations

use core::ops::ControlFlow;
use std::{collections::hash_map::Entry, convert::Infallible};

use rustc_hash::{FxHashMap, FxHashSet};
use surgeist_js_interner::{Interner, Sym};

use crate::{
    Declaration, Expression, LinearSpan, ModuleItem, Script, Statement, StatementList,
    StatementListItem,
    declaration::{
        Binding, ExportDeclaration, ImportDeclaration, LexicalDeclaration, VarDeclaration, Variable,
    },
    expression::{
        Await, Call, Identifier, NewTarget, OptionalOperationKind, SuperCall, This, Yield,
        access::{PrivatePropertyAccess, SuperPropertyAccess},
        literal::PropertyDefinition,
        operator::BinaryInPrivate,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassDeclaration, ClassElement,
        ClassElementName, ClassExpression, FormalParameterList, FunctionBody, FunctionDeclaration,
        FunctionExpression, GeneratorDeclaration, GeneratorExpression, PrivateFieldDefinition,
    },
    statement::{
        LabelledItem, With,
        iteration::{ForLoopInitializer, IterableLoopInitializer},
    },
    visitor::{NodeRef, VisitWith, Visitor},
};

#[cfg(test)]
mod tests;

/// Represents all the possible symbols searched for by the [`Contains`][contains] operation.
///
/// [contains]: https://tc39.es/ecma262/#sec-syntax-directed-operations-contains
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContainsSymbol {
    /// A node with the `super` keyword (`super(args)` or `super.prop`).
    Super,
    /// A super property access (`super.prop`).
    SuperProperty,
    /// A super constructor call (`super(args)`).
    SuperCall,
    /// A yield expression (`yield 5`).
    YieldExpression,
    /// An await expression (`await 4`).
    AwaitExpression,
    /// The new target expression (`new.target`).
    NewTarget,
    /// The body of a class definition.
    ClassBody,
    /// The super class of a class definition.
    ClassHeritage,
    /// A this expression (`this`).
    This,
    /// A method definition.
    MethodDefinition,
    /// The `BindingIdentifier` "eval" or "arguments".
    EvalOrArguments,
    /// A direct call to `eval`.
    DirectEval,
}

// Keep the refusal out of the recursive traversal's return value.
struct SearchAdmission<F, E> {
    admit: F,
    refusal: Option<E>,
}

impl<F, E> SearchAdmission<F, E>
where
    F: FnMut() -> Result<(), E>,
{
    fn new(admit: F) -> Self {
        Self {
            admit,
            refusal: None,
        }
    }

    #[inline]
    fn visit(&mut self) -> ControlFlow<()> {
        match (self.admit)() {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => {
                self.refusal = Some(error);
                ControlFlow::Break(())
            }
        }
    }

    fn finish(self, found: bool) -> Result<bool, E> {
        self.refusal.map_or(Ok(found), Err)
    }
}

/// Returns `true` if the node contains the given symbol.
///
/// This is equivalent to the [`Contains`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
#[must_use]
pub fn contains<N>(node: &N, symbol: ContainsSymbol) -> bool
where
    N: VisitWith,
{
    match try_contains(node, symbol, || Ok::<(), Infallible>(())) {
        Ok(found) => found,
        Err(never) => match never {},
    }
}

/// Searches for a symbol, admitting each inspection before it runs.
///
/// # Errors
/// Returns the first refused admission in traversal order.
pub fn try_contains<N, E>(
    node: &N,
    symbol: ContainsSymbol,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<bool, E>
where
    N: VisitWith,
{
    struct ContainsVisitor<F, E> {
        symbol: ContainsSymbol,
        admission: SearchAdmission<F, E>,
    }

    impl<F, E> ContainsVisitor<F, E> {
        fn visit_contains_eval(&mut self, contains_direct_eval: bool) -> ControlFlow<()> {
            if self.symbol == ContainsSymbol::DirectEval && contains_direct_eval {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
    }

    impl<'ast, F, E> Visitor<'ast> for ContainsVisitor<F, E>
    where
        F: FnMut() -> Result<(), E>,
    {
        type BreakTy = ();

        fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
            self.admission.visit()
        }

        fn visit_with(&mut self, node: &'ast With) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_expression(node.expression())?;
            self.visit_statement(node.statement())
        }

        fn visit_call(&mut self, node: &'ast Call) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::DirectEval {
                let mut candidate = node.function();
                loop {
                    self.admit_visit()?;
                    match candidate {
                        Expression::Parenthesized(wrapper) => candidate = wrapper.expression(),
                        Expression::Identifier(identifier) if identifier.sym() == Sym::EVAL => {
                            return ControlFlow::Break(());
                        }
                        _ => break,
                    }
                }
            }

            self.visit_expression(node.function())?;
            for arg in node.args() {
                self.visit_expression(arg)?;
            }
            ControlFlow::Continue(())
        }

        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::EvalOrArguments
                && (node.sym() == Sym::EVAL || node.sym() == Sym::ARGUMENTS)
            {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }

        fn visit_function_expression(
            &mut self,
            node: &'ast FunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_function_declaration(
            &mut self,
            node: &'ast FunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_async_function_expression(
            &mut self,
            node: &'ast AsyncFunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_async_function_declaration(
            &mut self,
            node: &'ast AsyncFunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_generator_expression(
            &mut self,
            node: &'ast GeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_generator_declaration(
            &mut self,
            node: &'ast GeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_async_generator_expression(
            &mut self,
            node: &'ast AsyncGeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_async_generator_declaration(
            &mut self,
            node: &'ast AsyncGeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_contains_eval(node.contains_direct_eval)?;
            ControlFlow::Continue(())
        }

        fn visit_class_expression(
            &mut self,
            node: &'ast ClassExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if !node.elements().is_empty() && self.symbol == ContainsSymbol::ClassBody {
                return ControlFlow::Break(());
            }

            if node.super_ref().is_some() && self.symbol == ContainsSymbol::ClassHeritage {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        fn visit_class_declaration(
            &mut self,
            node: &'ast ClassDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if !node.elements().is_empty() && self.symbol == ContainsSymbol::ClassBody {
                return ControlFlow::Break(());
            }

            if node.super_ref().is_some() && self.symbol == ContainsSymbol::ClassHeritage {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        // `ComputedPropertyContains`: https://tc39.es/ecma262/#sec-static-semantics-computedpropertycontains
        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            match node {
                ClassElement::MethodDefinition(m) => {
                    if self.symbol == ContainsSymbol::DirectEval {
                        return ControlFlow::Continue(());
                    }

                    if let ClassElementName::PropertyName(name) = m.name() {
                        self.visit_property_name(name)
                    } else {
                        ControlFlow::Continue(())
                    }
                }
                ClassElement::FieldDefinition(field)
                | ClassElement::StaticFieldDefinition(field) => {
                    self.visit_property_name(&field.name)
                }
                _ => ControlFlow::Continue(()),
            }
        }

        fn visit_property_definition(
            &mut self,
            node: &'ast PropertyDefinition,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if let PropertyDefinition::MethodDefinition(m) = node {
                if self.symbol == ContainsSymbol::DirectEval {
                    return ControlFlow::Continue(());
                }

                if self.symbol == ContainsSymbol::MethodDefinition {
                    return ControlFlow::Break(());
                }
                return self.visit_property_name(m.name());
            }

            node.visit_with(self)
        }

        fn visit_arrow_function(
            &mut self,
            node: &'ast ArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
                ContainsSymbol::DirectEval,
            ]
            .contains(&self.symbol)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }

        fn visit_async_arrow_function(
            &mut self,
            node: &'ast AsyncArrowFunction,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if ![
                ContainsSymbol::NewTarget,
                ContainsSymbol::SuperProperty,
                ContainsSymbol::SuperCall,
                ContainsSymbol::Super,
                ContainsSymbol::This,
                ContainsSymbol::DirectEval,
            ]
            .contains(&self.symbol)
            {
                return ControlFlow::Continue(());
            }

            node.visit_with(self)
        }

        fn visit_super_property_access(
            &mut self,
            node: &'ast SuperPropertyAccess,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if [ContainsSymbol::SuperProperty, ContainsSymbol::Super].contains(&self.symbol) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        fn visit_super_call(&mut self, node: &'ast SuperCall) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if [ContainsSymbol::SuperCall, ContainsSymbol::Super].contains(&self.symbol) {
                return ControlFlow::Break(());
            }
            node.visit_with(self)
        }

        fn visit_yield(&mut self, node: &'ast Yield) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::YieldExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        fn visit_await(&mut self, node: &'ast Await) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::AwaitExpression {
                return ControlFlow::Break(());
            }

            node.visit_with(self)
        }

        fn visit_this(&mut self, _node: &'ast This) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::This {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }

        fn visit_new_target(&mut self, _node: &'ast NewTarget) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if self.symbol == ContainsSymbol::NewTarget {
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        }
    }

    admit()?;
    let mut visitor = ContainsVisitor {
        symbol,
        admission: SearchAdmission::new(admit),
    };
    let found = node.visit_with(&mut visitor).is_break();
    visitor.admission.finish(found)
}

/// Returns true if the node contains an identifier reference with name `arguments`.
///
/// This is equivalent to the [`ContainsArguments`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
#[must_use]
pub fn contains_arguments<N>(node: &N) -> bool
where
    N: VisitWith,
{
    match try_contains_arguments(node, || Ok::<(), Infallible>(())) {
        Ok(found) => found,
        Err(never) => match never {},
    }
}

/// Searches for an `arguments` reference, admitting each inspection first.
///
/// # Errors
/// Returns the first refused admission in traversal order.
pub fn try_contains_arguments<N, E>(
    node: &N,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<bool, E>
where
    N: VisitWith,
{
    struct ContainsArgsVisitor<F, E>(SearchAdmission<F, E>);

    impl<'ast, F, E> Visitor<'ast> for ContainsArgsVisitor<F, E>
    where
        F: FnMut() -> Result<(), E>,
    {
        type BreakTy = ();

        fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
            self.0.visit()
        }

        fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if node.sym() == Sym::ARGUMENTS {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }

        fn visit_function_expression(
            &mut self,
            _: &'ast FunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_function_declaration(
            &mut self,
            _: &'ast FunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_async_function_expression(
            &mut self,
            _: &'ast AsyncFunctionExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_async_function_declaration(
            &mut self,
            _: &'ast AsyncFunctionDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_generator_expression(
            &mut self,
            _: &'ast GeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_generator_declaration(
            &mut self,
            _: &'ast GeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_async_generator_expression(
            &mut self,
            _: &'ast AsyncGeneratorExpression,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_async_generator_declaration(
            &mut self,
            _: &'ast AsyncGeneratorDeclaration,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            ControlFlow::Continue(())
        }

        fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if let ClassElement::MethodDefinition(m) = node
                && let ClassElementName::PropertyName(name) = m.name()
            {
                return self.visit_property_name(name);
            }

            node.visit_with(self)
        }

        fn visit_property_definition(
            &mut self,
            node: &'ast PropertyDefinition,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if let PropertyDefinition::MethodDefinition(m) = node {
                self.visit_property_name(m.name())
            } else {
                node.visit_with(self)
            }
        }
    }
    admit()?;
    let mut visitor = ContainsArgsVisitor(SearchAdmission::new(admit));
    let found = node.visit_with(&mut visitor).is_break();
    visitor.0.finish(found)
}

/// Returns `true` if `method` has a super call in its parameters or body.
///
/// This is equivalent to the [`HasDirectSuper`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-hasdirectsuper
#[must_use]
pub fn has_direct_super(params: &FormalParameterList, body: &FunctionBody) -> bool {
    match try_has_direct_super(params, body, || Ok::<(), Infallible>(())) {
        Ok(found) => found,
        Err(never) => match never {},
    }
}

/// Admission policy for the visits that compute cached direct-eval metadata.
/// `None` selects the ordinary AST search.
pub type DirectEvalAdmission<'a, E> = Option<&'a mut dyn FnMut() -> Result<(), E>>;

#[inline]
pub(crate) fn try_contains_direct_eval<N: VisitWith, E>(
    node: &N,
    admission: &mut DirectEvalAdmission<'_, E>,
) -> Result<bool, E> {
    match admission {
        Some(admit) => try_contains(node, ContainsSymbol::DirectEval, &mut **admit),
        None => Ok(contains(node, ContainsSymbol::DirectEval)),
    }
}

#[inline]
pub(crate) fn try_callable_direct_eval<E>(
    parameters: &FormalParameterList,
    body: &FunctionBody,
    admission: &mut DirectEvalAdmission<'_, E>,
) -> Result<bool, E> {
    Ok(try_contains_direct_eval(parameters, admission)?
        || try_contains_direct_eval(body, admission)?)
}

#[inline]
pub(crate) fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

/// Searches parameters and body for a direct super call, admitting each inspection first.
///
/// # Errors
/// Returns the first refused admission, searching parameters before the body.
pub fn try_has_direct_super<E>(
    params: &FormalParameterList,
    body: &FunctionBody,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    Ok(try_contains(params, ContainsSymbol::SuperCall, &mut admit)?
        || try_contains(body, ContainsSymbol::SuperCall, admit)?)
}

/// Work performed by AST name collection and analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameWork {
    /// One typed visit or raw AST entry inspection.
    Visit,
    /// One attempt to record a name, including a duplicate set insertion.
    Record,
    /// One symbol comparison or logical name-table operation.
    Comparison,
}

/// Optional admission policy for AST name work.
/// `None` selects the ordinary path.
pub type NameAdmission<'a, E> = Option<&'a mut dyn FnMut(NameWork) -> Result<(), E>>;

/// A container that [`BoundNamesVisitor`] can use to push the found identifiers.
pub(crate) trait IdentList {
    fn add(&mut self, value: Sym, function: bool);
}

impl IdentList for Vec<Sym> {
    fn add(&mut self, value: Sym, _function: bool) {
        self.push(value);
    }
}

impl IdentList for Vec<(Sym, bool)> {
    fn add(&mut self, value: Sym, function: bool) {
        self.push((value, function));
    }
}

impl IdentList for FxHashSet<Sym> {
    fn add(&mut self, value: Sym, _function: bool) {
        self.insert(value);
    }
}

pub(crate) struct NameCollection<T, F, E> {
    names: T,
    admit: F,
    refusal: Option<E>,
}

impl<T, F, E> NameCollection<T, F, E>
where
    T: IdentList,
    F: FnMut(NameWork) -> Result<(), E>,
{
    pub(crate) fn new(names: T, admit: F) -> Self {
        Self {
            names,
            admit,
            refusal: None,
        }
    }

    #[inline]
    fn admit_work(&mut self, work: NameWork) -> ControlFlow<()> {
        match (self.admit)(work) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => {
                self.refusal = Some(error);
                ControlFlow::Break(())
            }
        }
    }

    #[inline]
    pub(crate) fn visit(&mut self) -> ControlFlow<()> {
        self.admit_work(NameWork::Visit)
    }

    #[inline]
    pub(crate) fn add(&mut self, value: Sym, function: bool) -> ControlFlow<()> {
        self.admit_work(NameWork::Record)?;
        self.names.add(value, function);
        ControlFlow::Continue(())
    }

    pub(crate) fn finish(self) -> Result<T, E> {
        self.refusal.map_or(Ok(self.names), Err)
    }
}

/// The [`Visitor`] used to obtain the bound names of a node.
pub(crate) struct BoundNamesVisitor<'a, T, F, E>(pub(crate) &'a mut NameCollection<T, F, E>);

impl<'ast, T, F, E> Visitor<'ast> for BoundNamesVisitor<'_, T, F, E>
where
    T: IdentList,
    F: FnMut(NameWork) -> Result<(), E>,
{
    type BreakTy = ();

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        self.0.visit()
    }

    fn visit_identifier(&mut self, node: &'ast Identifier) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.sym(), false)?;
        ControlFlow::Continue(())
    }

    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        ControlFlow::Continue(())
    }

    fn visit_function_expression(
        &mut self,
        node: &'ast FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), true)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_function_declaration(
        &mut self,
        node: &'ast FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.name().sym(), true)?;
        ControlFlow::Continue(())
    }

    fn visit_generator_expression(
        &mut self,
        node: &'ast GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_generator_declaration(
        &mut self,
        node: &'ast GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.name().sym(), false)?;
        ControlFlow::Continue(())
    }

    fn visit_async_function_expression(
        &mut self,
        node: &'ast AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_async_function_declaration(
        &mut self,
        node: &'ast AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.name().sym(), false)?;
        ControlFlow::Continue(())
    }

    fn visit_async_generator_expression(
        &mut self,
        node: &'ast AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_async_generator_declaration(
        &mut self,
        node: &'ast AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.name().sym(), false)?;
        ControlFlow::Continue(())
    }

    fn visit_class_expression(
        &mut self,
        node: &'ast ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ident) = node.name() {
            self.0.add(ident.sym(), false)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_class_declaration(
        &mut self,
        node: &'ast ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.0.add(node.name().sym(), false)?;
        ControlFlow::Continue(())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ExportDeclaration::VarStatement(var) => self.visit_var_declaration(var)?,
            ExportDeclaration::Declaration(decl) => self.visit_declaration(decl)?,
            ExportDeclaration::DefaultFunctionDeclaration(f) => {
                self.0.add(f.name().sym(), true)?;
            }
            ExportDeclaration::DefaultGeneratorDeclaration(g) => {
                self.0.add(g.name().sym(), false)?;
            }
            ExportDeclaration::DefaultAsyncFunctionDeclaration(af) => {
                self.0.add(af.name().sym(), false)?;
            }
            ExportDeclaration::DefaultAsyncGeneratorDeclaration(ag) => {
                self.0.add(ag.name().sym(), false)?;
            }
            ExportDeclaration::DefaultClassDeclaration(cl) => {
                self.0.add(cl.name().sym(), false)?;
            }
            ExportDeclaration::DefaultAssignmentExpression(_) => {
                self.0.add(Sym::DEFAULT_EXPORT, false)?;
            }
            ExportDeclaration::ReExport { .. } | ExportDeclaration::List(_) => {}
        }

        ControlFlow::Continue(())
    }
}

/// Returns a list with the bound names of an AST node, which may contain duplicates.
///
/// This is equivalent to the [`BoundNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-boundnames
#[must_use]
pub fn bound_names<'a, N>(node: &'a N) -> Vec<Sym>
where
    &'a N: Into<NodeRef<'a>>,
{
    unwrap_infallible(try_bound_names(node, None))
}

/// Collects names with the caller's admission policy.
///
/// # Errors
/// Returns the first refused visit or record attempt.
pub fn try_bound_names<'a, N, E>(
    node: &'a N,
    admission: NameAdmission<'_, E>,
) -> Result<Vec<Sym>, E>
where
    &'a N: Into<NodeRef<'a>>,
{
    match admission {
        Some(admit) => collect_bound_names(node, admit),
        None => Ok(unwrap_infallible(collect_bound_names(node, |_| Ok(())))),
    }
}

pub(crate) fn collect_bound_names<'a, N, F, E>(node: &'a N, admit: F) -> Result<Vec<Sym>, E>
where
    &'a N: Into<NodeRef<'a>>,
    F: FnMut(NameWork) -> Result<(), E>,
{
    let mut names = NameCollection::new(Vec::default(), admit);
    let _ = BoundNamesVisitor(&mut names).visit(node.into());
    names.finish()
}

/// The [`Visitor`] used to obtain the lexically declared names of a node.
struct LexicallyDeclaredNamesVisitor<'a, T, F, E>(&'a mut NameCollection<T, F, E>);

impl<'ast, T, F, E> Visitor<'ast> for LexicallyDeclaredNamesVisitor<'_, T, F, E>
where
    T: IdentList,
    F: FnMut(NameWork) -> Result<(), E>,
{
    type BreakTy = ();

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        self.0.visit()
    }

    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        top_level_lexicals(node.statements(), self.0)
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        top_level_lexicals(node.statement_list(), self.0)
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(import) => {
                // 1. Return the BoundNames of ImportDeclaration.
                BoundNamesVisitor(self.0).visit_import_declaration(import)
            }

            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(export) => {
                // 1. If ExportDeclaration is export VariableStatement, return a new empty List.
                if matches!(export.as_ref(), ExportDeclaration::VarStatement(_)) {
                    ControlFlow::Continue(())
                } else {
                    // 2. Return the BoundNames of ExportDeclaration.
                    BoundNamesVisitor(self.0).visit_export_declaration(export)
                }
            }

            // ModuleItem : StatementListItem
            ModuleItem::StatementListItem(item) => {
                // 1. Return LexicallyDeclaredNames of StatementListItem.
                self.visit_statement_list_item(item)
            }
        }
    }

    fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        ControlFlow::Continue(())
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Statement::Labelled(labelled) = node {
            return self.visit_labelled(labelled);
        }
        ControlFlow::Continue(())
    }

    fn visit_declaration(&mut self, node: &'ast Declaration) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        BoundNamesVisitor(self.0).visit_declaration(node)
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            LabelledItem::FunctionDeclaration(f) => {
                BoundNamesVisitor(self.0).visit_function_declaration(f)
            }
            LabelledItem::Statement(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_function_expression(
        &mut self,
        node: &'ast FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_function_declaration(
        &mut self,
        node: &'ast FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_function_expression(
        &mut self,
        node: &'ast AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_function_declaration(
        &mut self,
        node: &'ast AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_generator_expression(
        &mut self,
        node: &'ast GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_generator_declaration(
        &mut self,
        node: &'ast GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_generator_expression(
        &mut self,
        node: &'ast AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_generator_declaration(
        &mut self,
        node: &'ast AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_arrow_function(&mut self, node: &'ast ArrowFunction) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_arrow_function(
        &mut self,
        node: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let ClassElement::StaticBlock(block) = node {
            self.visit_function_body(&block.body)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_import_declaration(
        &mut self,
        node: &'ast ImportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        BoundNamesVisitor(self.0).visit_import_declaration(node)
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if matches!(node, ExportDeclaration::VarStatement(_)) {
            return ControlFlow::Continue(());
        }
        BoundNamesVisitor(self.0).visit_export_declaration(node)
    }
}

/// Returns a list with the lexical bindings of a node, which may contain duplicates.
///
/// This is equivalent to the [`LexicallyDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
#[must_use]
pub fn lexically_declared_names<'a, N>(node: &'a N) -> Vec<Sym>
where
    &'a N: Into<NodeRef<'a>>,
{
    unwrap_infallible(try_lexically_declared_names(node, None))
}

/// Collects names with the caller's admission policy.
///
/// # Errors
/// Returns the first refused visit or record attempt.
pub fn try_lexically_declared_names<'a, N, E>(
    node: &'a N,
    admission: NameAdmission<'_, E>,
) -> Result<Vec<Sym>, E>
where
    &'a N: Into<NodeRef<'a>>,
{
    fn collect<'a, N, F, E>(node: &'a N, admit: F) -> Result<Vec<Sym>, E>
    where
        &'a N: Into<NodeRef<'a>>,
        F: FnMut(NameWork) -> Result<(), E>,
    {
        let mut names = NameCollection::new(Vec::default(), admit);
        let _ = LexicallyDeclaredNamesVisitor(&mut names).visit(node.into());
        names.finish()
    }

    match admission {
        Some(admit) => collect(node, admit),
        None => Ok(unwrap_infallible(collect(node, |_| Ok(())))),
    }
}

/// Returns a list with the lexical bindings of a node, which may contain duplicates.
///
/// If a declared name originates from a function declaration it is flagged as `true` in the returned
/// list. (See [B.3.2.4 Changes to Block Static Semantics: Early Errors])
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
/// [changes]: https://tc39.es/ecma262/#sec-block-duplicates-allowed-static-semantics
#[must_use]
pub fn lexically_declared_names_legacy<'a, N>(node: &'a N) -> Vec<(Sym, bool)>
where
    &'a N: Into<NodeRef<'a>>,
{
    unwrap_infallible(try_lexically_declared_names_legacy(node, None))
}

/// Collects names with the caller's admission policy.
///
/// # Errors
/// Returns the first refused visit or record attempt.
pub fn try_lexically_declared_names_legacy<'a, N, E>(
    node: &'a N,
    admission: NameAdmission<'_, E>,
) -> Result<Vec<(Sym, bool)>, E>
where
    &'a N: Into<NodeRef<'a>>,
{
    fn collect<'a, N, F, E>(node: &'a N, admit: F) -> Result<Vec<(Sym, bool)>, E>
    where
        &'a N: Into<NodeRef<'a>>,
        F: FnMut(NameWork) -> Result<(), E>,
    {
        let mut names = NameCollection::new(Vec::default(), admit);
        let _ = LexicallyDeclaredNamesVisitor(&mut names).visit(node.into());
        names.finish()
    }

    match admission {
        Some(admit) => collect(node, admit),
        None => Ok(unwrap_infallible(collect(node, |_| Ok(())))),
    }
}

/// The [`Visitor`] used to obtain the var declared names of a node.
struct VarDeclaredNamesVisitor<'a, F, E>(&'a mut NameCollection<FxHashSet<Sym>, F, E>);

impl<'ast, F, E> Visitor<'ast> for VarDeclaredNamesVisitor<'_, F, E>
where
    F: FnMut(NameWork) -> Result<(), E>,
{
    type BreakTy = ();

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        self.0.visit()
    }

    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        top_level_vars(node.statements(), self.0)
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        top_level_vars(node.statement_list(), self.0)
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
                ControlFlow::Continue(())
            }

            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(export) => {
                // 1. If ExportDeclaration is export VariableStatement, return BoundNames of ExportDeclaration.
                if let ExportDeclaration::VarStatement(var) = export.as_ref() {
                    BoundNamesVisitor(self.0).visit_var_declaration(var)
                } else {
                    // 2. Return a new empty List.
                    ControlFlow::Continue(())
                }
            }

            ModuleItem::StatementListItem(item) => self.visit_statement_list_item(item),
        }
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            Statement::Empty
            | Statement::Debugger
            | Statement::Expression(_)
            | Statement::Continue(_)
            | Statement::Break(_)
            | Statement::Return(_)
            | Statement::Throw(_) => ControlFlow::Continue(()),
            Statement::Block(node) => self.visit(node),
            Statement::Var(node) => self.visit(node),
            Statement::If(node) => self.visit(node),
            Statement::DoWhileLoop(node) => self.visit(node),
            Statement::WhileLoop(node) => self.visit(node),
            Statement::ForLoop(node) => self.visit(node),
            Statement::ForInLoop(node) => self.visit(node),
            Statement::ForOfLoop(node) => self.visit(node),
            Statement::Switch(node) => self.visit(node),
            Statement::Labelled(node) => self.visit(node),
            Statement::Try(node) => self.visit(node),
            Statement::With(node) => self.visit(node),
        }
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            StatementListItem::Statement(stmt) => self.visit_statement(stmt),
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_variable(&mut self, node: &'ast Variable) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        BoundNamesVisitor(self.0).visit_variable(node)
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(node) = node.else_node() {
            self.visit(node)?;
        }
        self.visit(node.body())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit(node.body())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit(node.body())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(ForLoopInitializer::Var(node)) = node.init() {
            BoundNamesVisitor(self.0).visit_var_declaration(node)?;
        }
        self.visit(node.body())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let IterableLoopInitializer::Var(node) = node.initializer() {
            BoundNamesVisitor(self.0).visit_variable(node)?;
        }
        self.visit(node.body())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let IterableLoopInitializer::Var(node) = node.initializer() {
            BoundNamesVisitor(self.0).visit_variable(node)?;
        }
        self.visit(node.body())
    }

    fn visit_with(&mut self, node: &'ast With) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit(node.statement())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        for case in node.cases() {
            self.visit(case)?;
        }
        for case in node.cases() {
            self.0.visit()?;
            if case.is_default() {
                self.visit(case.body())?;
                break;
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            LabelledItem::FunctionDeclaration(_) => ControlFlow::Continue(()),
            LabelledItem::Statement(stmt) => self.visit(stmt),
        }
    }

    fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(node) = node.finally() {
            self.visit(node)?;
        }
        if let Some(node) = node.catch() {
            self.visit(node.block())?;
        }
        self.visit(node.block())
    }

    fn visit_function_expression(
        &mut self,
        node: &'ast FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_function_declaration(
        &mut self,
        node: &'ast FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_function_expression(
        &mut self,
        node: &'ast AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_function_declaration(
        &mut self,
        node: &'ast AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_generator_expression(
        &mut self,
        node: &'ast GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_generator_declaration(
        &mut self,
        node: &'ast GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_generator_expression(
        &mut self,
        node: &'ast AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_async_generator_declaration(
        &mut self,
        node: &'ast AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_body(node.body())
    }

    fn visit_class_element(&mut self, node: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let ClassElement::StaticBlock(block) = node {
            self.visit_function_body(&block.body)?;
        }
        node.visit_with(self)
    }

    fn visit_import_declaration(
        &mut self,
        _: &'ast ImportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        ControlFlow::Continue(())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ExportDeclaration::VarStatement(var) => {
                BoundNamesVisitor(self.0).visit_var_declaration(var)
            }
            _ => ControlFlow::Continue(()),
        }
    }
}

/// Returns a set with the var bindings of a node, with no duplicates.
///
/// This is equivalent to the [`VarDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-vardeclarednames
#[must_use]
pub fn var_declared_names<'a, N>(node: &'a N) -> FxHashSet<Sym>
where
    &'a N: Into<NodeRef<'a>>,
{
    unwrap_infallible(try_var_declared_names(node, None))
}

/// Collects names with the caller's admission policy.
///
/// # Errors
/// Returns the first refused visit or record attempt.
pub fn try_var_declared_names<'a, N, E>(
    node: &'a N,
    admission: NameAdmission<'_, E>,
) -> Result<FxHashSet<Sym>, E>
where
    &'a N: Into<NodeRef<'a>>,
{
    fn collect<'a, N, F, E>(node: &'a N, admit: F) -> Result<FxHashSet<Sym>, E>
    where
        &'a N: Into<NodeRef<'a>>,
        F: FnMut(NameWork) -> Result<(), E>,
    {
        let mut names = NameCollection::new(FxHashSet::default(), admit);
        let _ = VarDeclaredNamesVisitor(&mut names).visit(node.into());
        names.finish()
    }

    match admission {
        Some(admit) => collect(node, admit),
        None => Ok(unwrap_infallible(collect(node, |_| Ok(())))),
    }
}

/// Utility function that collects the top level lexicals of a statement list into `names`.
///
/// This is equivalent to the [`TopLevelLexicallyDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallydeclarednames
fn top_level_lexicals<T, F, E>(
    stmts: &StatementList,
    names: &mut NameCollection<T, F, E>,
) -> ControlFlow<()>
where
    T: IdentList,
    F: FnMut(NameWork) -> Result<(), E>,
{
    for stmt in stmts.statements() {
        names.visit()?;
        if let StatementListItem::Declaration(decl) = stmt {
            match decl.as_ref() {
                // Note
                // At the top level of a function, or script, function declarations are treated like
                // var declarations rather than like lexical declarations.
                Declaration::FunctionDeclaration(_)
                | Declaration::GeneratorDeclaration(_)
                | Declaration::AsyncFunctionDeclaration(_)
                | Declaration::AsyncGeneratorDeclaration(_) => {}
                Declaration::ClassDeclaration(class) => {
                    BoundNamesVisitor(names).visit_class_declaration(class)?;
                }
                Declaration::Lexical(decl) => {
                    BoundNamesVisitor(names).visit_lexical_declaration(decl)?;
                }
            }
        }
    }
    ControlFlow::Continue(())
}

/// Utility function that collects the top level vars of a statement list into `names`.
///
/// This is equivalent to the [`TopLevelVarDeclaredNames`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevelvardeclarednames
fn top_level_vars<F, E>(
    stmts: &StatementList,
    names: &mut NameCollection<FxHashSet<Sym>, F, E>,
) -> ControlFlow<()>
where
    F: FnMut(NameWork) -> Result<(), E>,
{
    for stmt in stmts.statements() {
        names.visit()?;
        match stmt {
            StatementListItem::Declaration(decl) => {
                match decl.as_ref() {
                    // Note
                    // At the top level of a function, or script, function declarations are treated like
                    // var declarations rather than like lexical declarations.
                    Declaration::FunctionDeclaration(f) => {
                        BoundNamesVisitor(names).visit_function_declaration(f)?;
                    }
                    Declaration::GeneratorDeclaration(f) => {
                        BoundNamesVisitor(names).visit_generator_declaration(f)?;
                    }
                    Declaration::AsyncFunctionDeclaration(f) => {
                        BoundNamesVisitor(names).visit_async_function_declaration(f)?;
                    }
                    Declaration::AsyncGeneratorDeclaration(f) => {
                        BoundNamesVisitor(names).visit_async_generator_declaration(f)?;
                    }
                    Declaration::ClassDeclaration(_) | Declaration::Lexical(_) => {}
                }
            }
            StatementListItem::Statement(stmt) => {
                let mut stmt = Some(stmt.as_ref());
                while let Some(current) = stmt {
                    names.visit()?;
                    let Statement::Labelled(labelled) = current else {
                        break;
                    };
                    match labelled.item() {
                        LabelledItem::FunctionDeclaration(f) => {
                            BoundNamesVisitor(names).visit_function_declaration(f)?;
                            stmt = None;
                        }
                        LabelledItem::Statement(s) => stmt = Some(s),
                    }
                }
                if let Some(stmt) = stmt {
                    VarDeclaredNamesVisitor(names).visit(stmt)?;
                }
            }
        }
    }
    ControlFlow::Continue(())
}

/// Returns `true` if all private identifiers in a node are valid.
///
/// This is equivalent to the [`AllPrivateIdentifiersValid`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-allprivateidentifiersvalid
#[must_use]
#[inline]
pub fn all_private_identifiers_valid<'a, N>(node: &'a N, private_names: Vec<Sym>) -> bool
where
    &'a N: Into<NodeRef<'a>>,
{
    unwrap_infallible(try_all_private_identifiers_valid(node, private_names, None))
}

/// Validates private identifiers using the caller's admission policy.
///
/// `private_names` contains the names already visible at the root. Heritage
/// sees only the outer names. The class body sees its own declarations too.
///
/// # Errors
/// Returns the first refused visit, record, or comparison.
pub fn try_all_private_identifiers_valid<'a, N, E>(
    node: &'a N,
    private_names: Vec<Sym>,
    admission: NameAdmission<'_, E>,
) -> Result<bool, E>
where
    &'a N: Into<NodeRef<'a>>,
{
    fn validate<'a, N, F, E>(node: &'a N, names: Vec<Sym>, admit: F) -> Result<bool, E>
    where
        &'a N: Into<NodeRef<'a>>,
        F: FnMut(NameWork) -> Result<(), E>,
    {
        let mut visitor = AllPrivateIdentifiersValidVisitor(NameCollection::new(names, admit));
        let valid = visitor.visit(node.into()).is_continue();
        visitor.0.finish().map(|_| valid)
    }

    match admission {
        Some(admit) => validate(node, private_names, admit),
        None => Ok(unwrap_infallible(validate(node, private_names, |_| Ok(())))),
    }
}

struct AllPrivateIdentifiersValidVisitor<F, E>(NameCollection<Vec<Sym>, F, E>);

impl<F, E> AllPrivateIdentifiersValidVisitor<F, E>
where
    F: FnMut(NameWork) -> Result<(), E>,
{
    fn require_name(&mut self, name: Sym) -> ControlFlow<()> {
        for index in 0..self.0.names.len() {
            self.0.admit_work(NameWork::Comparison)?;
            if self.0.names[index] == name {
                return ControlFlow::Continue(());
            }
        }
        ControlFlow::Break(())
    }

    fn visit_class<'ast>(
        &mut self,
        heritage: Option<&'ast Expression>,
        constructor: Option<&'ast FunctionExpression>,
        elements: &'ast [ClassElement],
    ) -> ControlFlow<()> {
        if let Some(heritage) = heritage {
            self.visit(heritage)?;
        }

        let outer_names = self.0.names.len();
        let result = self.visit_class_body(constructor, elements);
        self.0.names.truncate(outer_names);
        result
    }

    fn visit_class_body<'ast>(
        &mut self,
        constructor: Option<&'ast FunctionExpression>,
        elements: &'ast [ClassElement],
    ) -> ControlFlow<()> {
        for element in elements {
            self.admit_visit()?;
            match element {
                ClassElement::MethodDefinition(method) => {
                    if let ClassElementName::PrivateName(name) = method.name() {
                        self.0.add(name.description(), false)?;
                    }
                }
                ClassElement::PrivateFieldDefinition(PrivateFieldDefinition { name, .. })
                | ClassElement::PrivateStaticFieldDefinition(PrivateFieldDefinition {
                    name, ..
                }) => {
                    self.0.add(name.description(), false)?;
                }
                _ => {}
            }
        }

        if let Some(constructor) = constructor {
            self.visit(constructor)?;
        }

        for element in elements {
            self.admit_visit()?;
            match element {
                ClassElement::MethodDefinition(method) => {
                    if let ClassElementName::PropertyName(name) = method.name() {
                        self.visit(name)?;
                    }
                    self.visit(method.parameters())?;
                    self.visit(method.body())?;
                }
                ClassElement::FieldDefinition(field)
                | ClassElement::StaticFieldDefinition(field) => {
                    self.visit(&field.name)?;
                    if let Some(expression) = &field.initializer {
                        self.visit(expression)?;
                    }
                }
                ClassElement::PrivateFieldDefinition(PrivateFieldDefinition {
                    initializer,
                    ..
                })
                | ClassElement::PrivateStaticFieldDefinition(PrivateFieldDefinition {
                    initializer,
                    ..
                }) => {
                    if let Some(expression) = initializer {
                        self.visit(expression)?;
                    }
                }
                ClassElement::StaticBlock(block) => self.visit(&block.body)?,
            }
        }

        ControlFlow::Continue(())
    }
}

impl<'ast, F, E> Visitor<'ast> for AllPrivateIdentifiersValidVisitor<F, E>
where
    F: FnMut(NameWork) -> Result<(), E>,
{
    type BreakTy = ();

    fn admit_visit(&mut self) -> ControlFlow<()> {
        self.0.visit()
    }

    fn visit_class_expression(&mut self, node: &'ast ClassExpression) -> ControlFlow<()> {
        self.admit_visit()?;
        self.visit_class(node.super_ref(), node.constructor(), node.elements())
    }

    fn visit_class_declaration(&mut self, node: &'ast ClassDeclaration) -> ControlFlow<()> {
        self.admit_visit()?;
        self.visit_class(node.super_ref(), node.constructor(), node.elements())
    }

    fn visit_private_property_access(
        &mut self,
        node: &'ast PrivatePropertyAccess,
    ) -> ControlFlow<()> {
        self.admit_visit()?;
        self.require_name(node.field().description())?;
        self.visit(node.target())
    }

    fn visit_binary_in_private(&mut self, node: &'ast BinaryInPrivate) -> ControlFlow<()> {
        self.admit_visit()?;
        self.require_name(node.lhs().description())?;
        self.visit(node.rhs())
    }

    fn visit_optional_operation_kind(
        &mut self,
        node: &'ast OptionalOperationKind,
    ) -> ControlFlow<()> {
        self.admit_visit()?;
        match node {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.visit_property_access_field(field)
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.require_name(field.description())
            }
            OptionalOperationKind::Call { args } => {
                for arg in args {
                    self.visit_expression(arg)?;
                }
                ControlFlow::Continue(())
            }
        }
    }
}

/// Errors that can occur when checking labels.
#[derive(Debug, Clone, Copy)]
pub enum CheckLabelsError {
    /// A label was used multiple times.
    DuplicateLabel(Sym),

    /// A `break` statement was used with a label that was not defined.
    UndefinedBreakTarget(Sym),

    /// A `continue` statement was used with a label that was not defined.
    UndefinedContinueTarget(Sym),

    /// A `break` statement was used in a non-looping context.
    IllegalBreakStatement,

    /// A `continue` statement was used in a non-looping context.
    IllegalContinueStatement,
}

impl CheckLabelsError {
    /// Returns an error message based on the error.
    #[must_use]
    pub fn message(&self, interner: &Interner) -> String {
        match self {
            Self::DuplicateLabel(label) => {
                format!("duplicate label: {}", interner.resolve_expect(*label))
            }
            Self::UndefinedBreakTarget(label) => {
                format!(
                    "undefined break target: {}",
                    interner.resolve_expect(*label)
                )
            }
            Self::UndefinedContinueTarget(label) => format!(
                "undefined continue target: {}",
                interner.resolve_expect(*label)
            ),
            Self::IllegalBreakStatement => "illegal break statement".into(),
            Self::IllegalContinueStatement => "illegal continue statement".into(),
        }
    }
}

/// Work admitted during label resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelCheckWork {
    /// One root or typed AST visit.
    Visit,
    /// One logical label table lookup. Registration includes its scoped cleanup.
    Lookup,
}

/// The reason label resolution stopped.
#[derive(Debug)]
pub enum CheckLabelsFailure<E> {
    /// The first invalid label or control-flow target.
    Syntax(CheckLabelsError),
    /// The caller refused further work.
    Refused(E),
}

/// This function checks multiple syntax errors conditions for labels, `break` and `continue`.
///
/// The following syntax errors are checked:
/// - [`ContainsDuplicateLabels`][ContainsDuplicateLabels]
/// - [`ContainsUndefinedBreakTarget`][ContainsUndefinedBreakTarget]
/// - [`ContainsUndefinedContinueTarget`][ContainsUndefinedContinueTarget]
/// - Early errors for [`BreakStatement`][BreakStatement]
/// - Early errors for [`ContinueStatement`][ContinueStatement]
///
/// [ContainsDuplicateLabels]: https://tc39.es/ecma262/#sec-static-semantics-containsduplicatelabels
/// [ContainsUndefinedBreakTarget]: https://tc39.es/ecma262/#sec-static-semantics-containsundefinedbreaktarget
/// [ContainsUndefinedContinueTarget]: https://tc39.es/ecma262/#sec-static-semantics-containsundefinedcontinuetarget
/// [BreakStatement]: https://tc39.es/ecma262/#sec-break-statement-static-semantics-early-errors
/// [ContinueStatement]: https://tc39.es/ecma262/#sec-continue-statement-static-semantics-early-errors
///
/// # Errors
///
/// Returns the first syntax error or admission refusal in traversal order.
pub fn check_labels<N, E>(
    node: &N,
    mut admit: impl FnMut(LabelCheckWork) -> Result<(), E>,
) -> Result<(), CheckLabelsFailure<E>>
where
    N: VisitWith,
{
    struct CheckLabelsResolver<F, E> {
        admit: F,
        failure: Option<CheckLabelsFailure<E>>,
        // Adjacent labels share a chain. A loop promotes that chain for its body.
        labels: FxHashMap<Sym, usize>,
        continuable: Vec<bool>,
        pending_chain: Option<usize>,
        iteration: bool,
        switch: bool,
    }

    impl<F, E> CheckLabelsResolver<F, E>
    where
        F: FnMut(LabelCheckWork) -> Result<(), E>,
    {
        fn admit_work(&mut self, work: LabelCheckWork) -> ControlFlow<()> {
            match (self.admit)(work) {
                Ok(()) => ControlFlow::Continue(()),
                Err(error) => {
                    self.failure = Some(CheckLabelsFailure::Refused(error));
                    ControlFlow::Break(())
                }
            }
        }

        fn syntax(&mut self, error: CheckLabelsError) -> ControlFlow<()> {
            self.failure = Some(CheckLabelsFailure::Syntax(error));
            ControlFlow::Break(())
        }

        fn visit_loop_body(&mut self, body: &Statement) -> ControlFlow<()> {
            let chain = self.pending_chain.take();
            if let Some(chain) = chain {
                self.continuable[chain] = true;
            }
            let iteration = std::mem::replace(&mut self.iteration, true);
            let result = self.visit_statement(body);
            self.iteration = iteration;
            if let Some(chain) = chain {
                self.continuable[chain] = false;
            }
            self.pending_chain = chain;
            result
        }
    }

    impl<'ast, F, E> Visitor<'ast> for CheckLabelsResolver<F, E>
    where
        F: FnMut(LabelCheckWork) -> Result<(), E>,
    {
        type BreakTy = ();

        fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
            self.admit_work(LabelCheckWork::Visit)
        }

        fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            match node {
                Statement::Block(node) => self.visit_block(node),
                Statement::Var(_)
                | Statement::Empty
                | Statement::Debugger
                | Statement::Expression(_)
                | Statement::Return(_)
                | Statement::Throw(_) => ControlFlow::Continue(()),
                Statement::If(node) => self.visit_if(node),
                Statement::DoWhileLoop(node) => self.visit_do_while_loop(node),
                Statement::WhileLoop(node) => self.visit_while_loop(node),
                Statement::ForLoop(node) => self.visit_for_loop(node),
                Statement::ForInLoop(node) => self.visit_for_in_loop(node),
                Statement::ForOfLoop(node) => self.visit_for_of_loop(node),
                Statement::Switch(node) => self.visit_switch(node),
                Statement::Labelled(node) => self.visit_labelled(node),
                Statement::Try(node) => self.visit_try(node),
                Statement::Continue(node) => self.visit_continue(node),
                Statement::Break(node) => self.visit_break(node),
                Statement::With(with) => self.visit_with(with),
            }
        }

        fn visit_block(
            &mut self,
            node: &'ast crate::statement::Block,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            self.visit_statement_list(node.statement_list())?;
            self.pending_chain = pending_chain;
            ControlFlow::Continue(())
        }

        fn visit_break(
            &mut self,
            node: &'ast crate::statement::Break,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if let Some(label) = node.label() {
                self.admit_work(LabelCheckWork::Lookup)?;
                if !self.labels.contains_key(&label) {
                    return self.syntax(CheckLabelsError::UndefinedBreakTarget(label));
                }
            } else if !self.iteration && !self.switch {
                return self.syntax(CheckLabelsError::IllegalBreakStatement);
            }
            ControlFlow::Continue(())
        }

        fn visit_continue(
            &mut self,
            node: &'ast crate::statement::Continue,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            if !self.iteration {
                return self.syntax(CheckLabelsError::IllegalContinueStatement);
            }

            if let Some(label) = node.label() {
                self.admit_work(LabelCheckWork::Lookup)?;
                if !self
                    .labels
                    .get(&label)
                    .is_some_and(|&chain| self.continuable[chain])
                {
                    return self.syntax(CheckLabelsError::UndefinedContinueTarget(label));
                }
            }
            ControlFlow::Continue(())
        }

        fn visit_do_while_loop(
            &mut self,
            node: &'ast crate::statement::DoWhileLoop,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_loop_body(node.body())
        }

        fn visit_while_loop(
            &mut self,
            node: &'ast crate::statement::WhileLoop,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_loop_body(node.body())
        }

        fn visit_for_loop(
            &mut self,
            node: &'ast crate::statement::ForLoop,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_loop_body(node.body())
        }

        fn visit_for_in_loop(
            &mut self,
            node: &'ast crate::statement::ForInLoop,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_loop_body(node.body())
        }

        fn visit_for_of_loop(
            &mut self,
            node: &'ast crate::statement::ForOfLoop,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.visit_loop_body(node.body())
        }

        fn visit_statement_list_item(
            &mut self,
            node: &'ast StatementListItem,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            if let StatementListItem::Statement(stmt) = node {
                self.visit_statement(stmt)?;
            }
            self.pending_chain = pending_chain;
            ControlFlow::Continue(())
        }

        fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            self.visit_statement(node.body())?;
            if let Some(stmt) = node.else_node() {
                self.visit_statement(stmt)?;
            }
            self.pending_chain = pending_chain;
            ControlFlow::Continue(())
        }

        fn visit_switch(
            &mut self,
            node: &'ast crate::statement::Switch,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            let switch = self.switch;
            self.switch = true;
            for case in node.cases() {
                self.admit_visit()?;
                self.visit_statement_list(case.body())?;
            }
            self.pending_chain = pending_chain;
            self.switch = switch;
            ControlFlow::Continue(())
        }

        fn visit_labelled(
            &mut self,
            node: &'ast crate::statement::Labelled,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            self.admit_work(LabelCheckWork::Lookup)?;
            let label = node.label();
            let previous_chain = self.pending_chain;
            let chain = previous_chain.unwrap_or(self.continuable.len());
            let Entry::Vacant(entry) = self.labels.entry(label) else {
                return self.syntax(CheckLabelsError::DuplicateLabel(label));
            };
            entry.insert(chain);
            if previous_chain.is_none() {
                self.continuable.push(false);
            }
            self.pending_chain = Some(chain);
            let result = self.visit_labelled_item(node.item());
            self.labels.remove(&label);
            self.pending_chain = previous_chain;
            if previous_chain.is_none() {
                self.continuable.pop();
            }
            result
        }

        fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            match node {
                LabelledItem::Statement(stmt) => self.visit_statement(stmt),
                LabelledItem::FunctionDeclaration(_) => ControlFlow::Continue(()),
            }
        }

        fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            self.visit_block(node.block())?;
            if let Some(catch) = node.catch() {
                self.visit_block(catch.block())?;
            }
            if let Some(finally) = node.finally() {
                self.visit_block(finally.block())?;
            }
            self.pending_chain = pending_chain;
            ControlFlow::Continue(())
        }

        fn visit_module_item_list(
            &mut self,
            node: &'ast crate::ModuleItemList,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            let pending_chain = self.pending_chain.take();
            for item in node.items() {
                self.visit_module_item(item)?;
            }
            self.pending_chain = pending_chain;
            ControlFlow::Continue(())
        }

        fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            match node {
                ModuleItem::ImportDeclaration(_) | ModuleItem::ExportDeclaration(_) => {
                    ControlFlow::Continue(())
                }
                ModuleItem::StatementListItem(node) => self.visit_statement_list_item(node),
            }
        }
    }

    admit(LabelCheckWork::Visit).map_err(CheckLabelsFailure::Refused)?;
    let mut visitor = CheckLabelsResolver {
        admit,
        failure: None,
        labels: FxHashMap::default(),
        continuable: Vec::new(),
        pending_chain: None,
        iteration: false,
        switch: false,
    };
    let _ = node.visit_with(&mut visitor);
    visitor.failure.map_or(Ok(()), Err)
}

/// Looks for a `CoverInitializedName`, admitting each walk step before inspection.
///
/// Stops on the first match or admission error in traversal order.
///
/// # Errors
/// Returns the error from the first refused admission.
pub fn contains_invalid_object_literal<N, E>(
    node: &N,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<bool, E>
where
    N: VisitWith,
{
    struct ContainsInvalidObjectLiteral<F, E> {
        admit: F,
        refusal: Option<E>,
    }

    impl<'ast, F, E> Visitor<'ast> for ContainsInvalidObjectLiteral<F, E>
    where
        F: FnMut() -> Result<(), E>,
    {
        type BreakTy = ();

        fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
            match (self.admit)() {
                Ok(()) => ControlFlow::Continue(()),
                Err(error) => {
                    self.refusal = Some(error);
                    ControlFlow::Break(())
                }
            }
        }

        fn visit_object_literal(
            &mut self,
            node: &'ast crate::expression::literal::ObjectLiteral,
        ) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            for pd in node.properties() {
                self.admit_visit()?;
                if let PropertyDefinition::CoverInitializedName(..) = pd {
                    return ControlFlow::Break(());
                }
                pd.visit_with(self)?;
            }
            ControlFlow::Continue(())
        }
    }

    admit()?;
    // Keep the refusal out of the recursive traversal's return value.
    let mut visitor = ContainsInvalidObjectLiteral {
        admit,
        refusal: None,
    };
    let found = node.visit_with(&mut visitor).is_break();
    visitor.refusal.map_or(Ok(found), Err)
}

/// The type of a lexically scoped declaration.
#[derive(Copy, Clone, Debug)]
pub enum LexicallyScopedDeclaration<'a> {
    /// See [`LexicalDeclaration`]
    LexicalDeclaration(&'a LexicalDeclaration),

    /// See [`FunctionDeclaration`]
    FunctionDeclaration(&'a FunctionDeclaration),

    /// See [`GeneratorDeclaration`]
    GeneratorDeclaration(&'a GeneratorDeclaration),

    /// See [`AsyncFunctionDeclaration`]
    AsyncFunctionDeclaration(&'a AsyncFunctionDeclaration),

    /// See [`AsyncGeneratorDeclaration`]
    AsyncGeneratorDeclaration(&'a AsyncGeneratorDeclaration),

    /// See [`ClassDeclaration`]
    ClassDeclaration(&'a ClassDeclaration),

    /// A default assignment expression as an export declaration.
    ///
    /// Only valid inside module exports.
    AssignmentExpression(&'a Expression),
}

impl LexicallyScopedDeclaration<'_> {
    /// Return the bound names of the declaration.
    #[must_use]
    pub fn bound_names(&self) -> Vec<Sym> {
        match *self {
            Self::LexicalDeclaration(v) => bound_names(v),
            Self::FunctionDeclaration(f) => bound_names(f),
            Self::GeneratorDeclaration(g) => bound_names(g),
            Self::AsyncFunctionDeclaration(f) => bound_names(f),
            Self::AsyncGeneratorDeclaration(g) => bound_names(g),
            Self::ClassDeclaration(cl) => bound_names(cl),
            Self::AssignmentExpression(expr) => bound_names(expr),
        }
    }
}

impl<'ast> From<&'ast Declaration> for LexicallyScopedDeclaration<'ast> {
    fn from(value: &'ast Declaration) -> LexicallyScopedDeclaration<'ast> {
        match value {
            Declaration::FunctionDeclaration(f) => Self::FunctionDeclaration(f),
            Declaration::GeneratorDeclaration(g) => Self::GeneratorDeclaration(g),
            Declaration::AsyncFunctionDeclaration(af) => Self::AsyncFunctionDeclaration(af),
            Declaration::AsyncGeneratorDeclaration(ag) => Self::AsyncGeneratorDeclaration(ag),
            Declaration::ClassDeclaration(c) => Self::ClassDeclaration(c),
            Declaration::Lexical(lex) => Self::LexicalDeclaration(lex),
        }
    }
}

/// Returns a list of lexically scoped declarations of the given node.
///
/// This is equivalent to the [`LexicallyScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallyscopeddeclarations
#[must_use]
pub fn lexically_scoped_declarations<'a, N>(node: &'a N) -> Vec<LexicallyScopedDeclaration<'a>>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    let _ = LexicallyScopedDeclarationsVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used to obtain the lexically scoped declarations of a node.
#[derive(Debug)]
struct LexicallyScopedDeclarationsVisitor<'a, 'ast>(&'a mut Vec<LexicallyScopedDeclaration<'ast>>);

impl<'ast> Visitor<'ast> for LexicallyScopedDeclarationsVisitor<'_, 'ast> {
    type BreakTy = Infallible;

    // ScriptBody : StatementList
    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelLexicallyScopedDeclarations of StatementList.
        TopLevelLexicallyScopedDeclarationsVisitor(self.0).visit_statement_list(node.statements())
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelVarScopedDeclarations of StatementList.
        TopLevelLexicallyScopedDeclarationsVisitor(self.0)
            .visit_statement_list(node.statement_list())
    }

    fn visit_export_declaration(
        &mut self,
        node: &'ast ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        let decl = match node {
            // ExportDeclaration :
            // export ExportFromClause FromClause ;
            // export NamedExports ;
            // export VariableStatement
            ExportDeclaration::ReExport { .. }
            | ExportDeclaration::List(_)
            | ExportDeclaration::VarStatement(_) => {
                //     1. Return a new empty List.
                return ControlFlow::Continue(());
            }

            // ExportDeclaration : export Declaration
            ExportDeclaration::Declaration(decl) => {
                // 1. Return a List whose sole element is DeclarationPart of Declaration.
                decl.into()
            }

            // ExportDeclaration : export default HoistableDeclaration
            // 1. Return a List whose sole element is DeclarationPart of HoistableDeclaration.
            ExportDeclaration::DefaultFunctionDeclaration(f) => {
                LexicallyScopedDeclaration::FunctionDeclaration(f)
            }
            ExportDeclaration::DefaultGeneratorDeclaration(g) => {
                LexicallyScopedDeclaration::GeneratorDeclaration(g)
            }
            ExportDeclaration::DefaultAsyncFunctionDeclaration(af) => {
                LexicallyScopedDeclaration::AsyncFunctionDeclaration(af)
            }
            ExportDeclaration::DefaultAsyncGeneratorDeclaration(ag) => {
                LexicallyScopedDeclaration::AsyncGeneratorDeclaration(ag)
            }

            // ExportDeclaration : export default ClassDeclaration
            ExportDeclaration::DefaultClassDeclaration(c) => {
                // 1. Return a List whose sole element is ClassDeclaration.
                LexicallyScopedDeclaration::ClassDeclaration(c)
            }

            // ExportDeclaration : export default AssignmentExpression ;
            ExportDeclaration::DefaultAssignmentExpression(expr) => {
                // 1. Return a List whose sole element is this ExportDeclaration.
                LexicallyScopedDeclaration::AssignmentExpression(expr)
            }
        };

        self.0.push(decl);

        ControlFlow::Continue(())
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            // StatementListItem : Statement
            StatementListItem::Statement(statement) => {
                // 1. If Statement is Statement : LabelledStatement , return LexicallyScopedDeclarations of LabelledStatement.
                if let Statement::Labelled(labelled) = statement.as_ref() {
                    self.visit_labelled(labelled)
                } else {
                    // 2. Return a new empty List.
                    ControlFlow::Continue(())
                }
            }

            // StatementListItem : Declaration
            StatementListItem::Declaration(declaration) => {
                // 1. Return a List whose sole element is DeclarationPart of Declaration.
                self.0.push(declaration.as_ref().into());
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // LabelledItem : FunctionDeclaration
            LabelledItem::FunctionDeclaration(f) => {
                // 1. Return « FunctionDeclaration ».
                self.0
                    .push(LexicallyScopedDeclaration::FunctionDeclaration(f));
            }

            // LabelledItem : Statement
            LabelledItem::Statement(_) => {
                // 1. Return a new empty List.
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            ModuleItem::StatementListItem(item) => self.visit_statement_list_item(item),
            ModuleItem::ExportDeclaration(export) => self.visit_export_declaration(export),

            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
                ControlFlow::Continue(())
            }
        }
    }
}
/// The [`Visitor`] used to obtain the top level lexically scoped declarations of a node.
///
/// This is equivalent to the [`TopLevelLexicallyScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallyscopeddeclarations
#[derive(Debug)]
struct TopLevelLexicallyScopedDeclarationsVisitor<'a, 'ast>(
    &'a mut Vec<LexicallyScopedDeclaration<'ast>>,
);

impl<'ast> Visitor<'ast> for TopLevelLexicallyScopedDeclarationsVisitor<'_, 'ast> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            // StatementListItem : Declaration
            StatementListItem::Declaration(d) => match d.as_ref() {
                // 1. If Declaration is Declaration : HoistableDeclaration , then
                Declaration::FunctionDeclaration(_)
                | Declaration::GeneratorDeclaration(_)
                | Declaration::AsyncFunctionDeclaration(_)
                | Declaration::AsyncGeneratorDeclaration(_) => {
                    // a. Return a new empty List.
                }

                // 2. Return « Declaration ».
                Declaration::ClassDeclaration(cl) => {
                    self.0
                        .push(LexicallyScopedDeclaration::ClassDeclaration(cl));
                }
                Declaration::Lexical(lex) => {
                    self.0
                        .push(LexicallyScopedDeclaration::LexicalDeclaration(lex));
                }
            },

            // StatementListItem : Statement
            StatementListItem::Statement(_) => {
                // 1. Return a new empty List.
            }
        }

        ControlFlow::Continue(())
    }
}

/// The type of a var scoped declaration.
#[derive(Clone, Debug)]
pub enum VarScopedDeclaration {
    /// See [`VarDeclaration`]
    VariableDeclaration(Variable),

    /// See [`FunctionDeclaration`]
    FunctionDeclaration(FunctionDeclaration),

    /// See [`GeneratorDeclaration`]
    GeneratorDeclaration(GeneratorDeclaration),

    /// See [`AsyncFunctionDeclaration`]
    AsyncFunctionDeclaration(AsyncFunctionDeclaration),

    /// See [`AsyncGeneratorDeclaration`]
    AsyncGeneratorDeclaration(AsyncGeneratorDeclaration),
}

impl VarScopedDeclaration {
    /// Return the bound names of the declaration.
    #[must_use]
    pub fn bound_names(&self) -> Vec<Sym> {
        match self {
            Self::VariableDeclaration(v) => bound_names(v),
            Self::FunctionDeclaration(f) => bound_names(f),
            Self::GeneratorDeclaration(g) => bound_names(g),
            Self::AsyncFunctionDeclaration(f) => bound_names(f),
            Self::AsyncGeneratorDeclaration(g) => bound_names(g),
        }
    }

    /// Return [`LinearSpan`] of this declaration (if there is).
    #[must_use]
    pub fn linear_span(&self) -> Option<LinearSpan> {
        match self {
            VarScopedDeclaration::FunctionDeclaration(f) => Some(f.linear_span()),
            VarScopedDeclaration::GeneratorDeclaration(f) => Some(f.linear_span()),
            VarScopedDeclaration::AsyncFunctionDeclaration(f) => Some(f.linear_span()),
            VarScopedDeclaration::AsyncGeneratorDeclaration(f) => Some(f.linear_span()),
            VarScopedDeclaration::VariableDeclaration(_) => None,
        }
    }
}

/// Returns a list of var scoped declarations of the given node.
///
/// This is equivalent to the [`VarScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-varscopeddeclarations
#[must_use]
pub fn var_scoped_declarations<'a, N>(node: &'a N) -> Vec<VarScopedDeclaration>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    let _ = VarScopedDeclarationsVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used to obtain the var scoped declarations of a node.
#[derive(Debug)]
struct VarScopedDeclarationsVisitor<'a>(&'a mut Vec<VarScopedDeclaration>);

impl<'ast> Visitor<'ast> for VarScopedDeclarationsVisitor<'_> {
    type BreakTy = Infallible;

    // ScriptBody : StatementList
    fn visit_script(&mut self, node: &'ast Script) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelVarScopedDeclarations of StatementList.
        TopLevelVarScopedDeclarationsVisitor(self.0).visit_statement_list(node.statements())
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        // 1. Return TopLevelVarScopedDeclarations of StatementList.
        TopLevelVarScopedDeclarationsVisitor(self.0).visit_statement_list(node.statement_list())
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Block(s) => self.visit(s),
            Statement::Var(s) => self.visit(s),
            Statement::If(s) => self.visit(s),
            Statement::DoWhileLoop(s) => self.visit(s),
            Statement::WhileLoop(s) => self.visit(s),
            Statement::ForLoop(s) => self.visit(s),
            Statement::ForInLoop(s) => self.visit(s),
            Statement::ForOfLoop(s) => self.visit(s),
            Statement::Switch(s) => self.visit(s),
            Statement::Labelled(s) => self.visit(s),
            Statement::Try(s) => self.visit(s),
            Statement::With(s) => self.visit(s),
            Statement::Empty
            | Statement::Debugger
            | Statement::Expression(_)
            | Statement::Continue(_)
            | Statement::Break(_)
            | Statement::Return(_)
            | Statement::Throw(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
            StatementListItem::Statement(s) => self.visit(s.as_ref()),
        }
    }

    fn visit_var_declaration(&mut self, node: &'ast VarDeclaration) -> ControlFlow<Self::BreakTy> {
        for var in node.0.as_ref() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        ControlFlow::Continue(())
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;
        if let Some(else_node) = node.else_node() {
            self.visit(else_node)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(ForLoopInitializer::Var(v)) = node.init() {
            self.visit(v)?;
        }
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(var) = node.initializer() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        if let IterableLoopInitializer::Var(var) = node.initializer() {
            self.0
                .push(VarScopedDeclaration::VariableDeclaration(var.clone()));
        }
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_with(&mut self, node: &'ast With) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement())?;
        ControlFlow::Continue(())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        for case in node.cases() {
            self.visit(case)?;
        }
        if let Some(default) = node.default() {
            self.visit(default)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_case(&mut self, node: &'ast crate::statement::Case) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;
        ControlFlow::Continue(())
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Statement(s) => self.visit(s),
            LabelledItem::FunctionDeclaration(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_catch(&mut self, node: &'ast crate::statement::Catch) -> ControlFlow<Self::BreakTy> {
        self.visit(node.block())?;
        ControlFlow::Continue(())
    }

    fn visit_module_item(&mut self, node: &'ast ModuleItem) -> ControlFlow<Self::BreakTy> {
        match node {
            // ModuleItem : ExportDeclaration
            ModuleItem::ExportDeclaration(decl) => {
                if let ExportDeclaration::VarStatement(var) = decl.as_ref() {
                    //     1. If ExportDeclaration is export VariableStatement, return VarScopedDeclarations of VariableStatement.
                    self.visit_var_declaration(var)?;
                }
                // 2. Return a new empty List.
            }
            ModuleItem::StatementListItem(item) => {
                self.visit_statement_list_item(item)?;
            }
            // ModuleItem : ImportDeclaration
            ModuleItem::ImportDeclaration(_) => {
                // 1. Return a new empty List.
            }
        }
        ControlFlow::Continue(())
    }
}

/// The [`Visitor`] used to obtain the top level var scoped declarations of a node.
///
/// This is equivalent to the [`TopLevelVarScopedDeclarations`][spec] syntax operation in the spec.
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevelvarscopeddeclarations
#[derive(Debug)]
struct TopLevelVarScopedDeclarationsVisitor<'a>(&'a mut Vec<VarScopedDeclaration>);

impl<'ast> Visitor<'ast> for TopLevelVarScopedDeclarationsVisitor<'_> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Declaration(d) => {
                match d.as_ref() {
                    Declaration::FunctionDeclaration(f) => {
                        self.0
                            .push(VarScopedDeclaration::FunctionDeclaration(f.clone()));
                    }
                    Declaration::GeneratorDeclaration(f) => {
                        self.0
                            .push(VarScopedDeclaration::GeneratorDeclaration(f.clone()));
                    }
                    Declaration::AsyncFunctionDeclaration(f) => {
                        self.0
                            .push(VarScopedDeclaration::AsyncFunctionDeclaration(f.clone()));
                    }
                    Declaration::AsyncGeneratorDeclaration(f) => {
                        self.0
                            .push(VarScopedDeclaration::AsyncGeneratorDeclaration(f.clone()));
                    }
                    _ => {}
                }
                ControlFlow::Continue(())
            }
            StatementListItem::Statement(statement) => {
                if let Statement::Labelled(labelled) = statement.as_ref() {
                    self.visit(labelled)
                } else {
                    VarScopedDeclarationsVisitor(self.0).visit(statement.as_ref())
                }
            }
        }
    }

    fn visit_labelled_item(&mut self, node: &'ast LabelledItem) -> ControlFlow<Self::BreakTy> {
        match node {
            LabelledItem::Statement(Statement::Labelled(s)) => self.visit(s),
            LabelledItem::Statement(s) => {
                VarScopedDeclarationsVisitor(self.0).visit(s)?;
                ControlFlow::Continue(())
            }
            LabelledItem::FunctionDeclaration(f) => {
                self.0
                    .push(VarScopedDeclaration::FunctionDeclaration(f.clone()));
                ControlFlow::Continue(())
            }
        }
    }
}

/// Returns a list function declaration names that are directly contained in a statement lists
/// `Block`, `CaseClause` or `DefaultClause`.
/// If the function declaration would cause an early error it is not included in the list.
///
/// This behavior is used in the following annexB sections:
/// * [B.3.2.1 Changes to FunctionDeclarationInstantiation][spec0]
/// * [B.3.2.2 Changes to GlobalDeclarationInstantiation][spec1]
/// * [B.3.2.3 Changes to EvalDeclarationInstantiation][spec2]
///
/// [spec0]: https://tc39.es/ecma262/#sec-web-compat-functiondeclarationinstantiation
/// [spec1]: https://tc39.es/ecma262/#sec-web-compat-globaldeclarationinstantiation
/// [spec2]: https://tc39.es/ecma262/#sec-web-compat-evaldeclarationinstantiation
#[must_use]
pub fn annex_b_function_declarations_names<'a, N>(node: &'a N) -> Vec<Sym>
where
    &'a N: Into<NodeRef<'a>>,
{
    let mut declarations = Vec::new();
    let _ = AnnexBFunctionDeclarationNamesVisitor(&mut declarations).visit(node.into());
    declarations
}

/// The [`Visitor`] used for [`annex_b_function_declarations_names`].
#[derive(Debug)]
struct AnnexBFunctionDeclarationNamesVisitor<'a>(&'a mut Vec<Sym>);

impl<'ast> Visitor<'ast> for AnnexBFunctionDeclarationNamesVisitor<'_> {
    type BreakTy = Infallible;

    fn visit_statement_list_item(
        &mut self,
        node: &'ast StatementListItem,
    ) -> ControlFlow<Self::BreakTy> {
        match node {
            StatementListItem::Statement(node) => self.visit(node.as_ref()),
            StatementListItem::Declaration(_) => ControlFlow::Continue(()),
        }
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Block(node) => self.visit(node),
            Statement::If(node) => self.visit(node),
            Statement::DoWhileLoop(node) => self.visit(node),
            Statement::WhileLoop(node) => self.visit(node),
            Statement::ForLoop(node) => self.visit(node),
            Statement::ForInLoop(node) => self.visit(node),
            Statement::ForOfLoop(node) => self.visit(node),
            Statement::Switch(node) => self.visit(node),
            Statement::Labelled(node) => self.visit(node),
            Statement::Try(node) => self.visit(node),
            Statement::With(node) => self.visit(node),
            _ => ControlFlow::Continue(()),
        }
    }

    fn visit_block(&mut self, node: &'ast crate::statement::Block) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement_list())?;
        for statement in node.statement_list().statements() {
            if let StatementListItem::Declaration(declaration) = statement
                && let Declaration::FunctionDeclaration(function) = &**declaration
            {
                let name = function.name();
                self.0.push(name.sym());
            }
        }

        let lexically_declared_names = lexically_declared_names_legacy(node.statement_list());

        self.0
            .retain(|name| !lexically_declared_names.contains(&(*name, false)));

        ControlFlow::Continue(())
    }

    fn visit_switch(&mut self, node: &'ast crate::statement::Switch) -> ControlFlow<Self::BreakTy> {
        for case in node.cases() {
            self.visit(case)?;
            for statement in case.body().statements() {
                if let StatementListItem::Declaration(declaration) = statement
                    && let Declaration::FunctionDeclaration(function) = &**declaration
                {
                    let name = function.name();
                    self.0.push(name.sym());
                }
            }
        }
        if let Some(default) = node.default() {
            self.visit(default)?;
            for statement in default.statements() {
                if let StatementListItem::Declaration(declaration) = statement
                    && let Declaration::FunctionDeclaration(function) = declaration.as_ref()
                {
                    let name = function.name();
                    self.0.push(name.sym());
                }
            }
        }

        let lexically_declared_names = lexically_declared_names_legacy(node);

        self.0
            .retain(|name| !lexically_declared_names.contains(&(*name, false)));

        ControlFlow::Continue(())
    }

    fn visit_try(&mut self, node: &'ast crate::statement::Try) -> ControlFlow<Self::BreakTy> {
        self.visit(node.block())?;
        if let Some(catch) = node.catch() {
            self.visit(catch.block())?;

            if let Some(Binding::Pattern(pattern)) = catch.parameter() {
                let bound_names = bound_names(pattern);

                self.0.retain(|name| !bound_names.contains(name));
            }
        }
        if let Some(finally) = node.finally() {
            self.visit(finally.block())?;
        }
        ControlFlow::Continue(())
    }

    fn visit_if(&mut self, node: &'ast crate::statement::If) -> ControlFlow<Self::BreakTy> {
        if let Some(node) = node.else_node() {
            self.visit(node)?;
        }
        self.visit(node.body())
    }

    fn visit_do_while_loop(
        &mut self,
        node: &'ast crate::statement::DoWhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_while_loop(
        &mut self,
        node: &'ast crate::statement::WhileLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())
    }

    fn visit_for_loop(
        &mut self,
        node: &'ast crate::statement::ForLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;

        if let Some(ForLoopInitializer::Lexical(node)) = node.init() {
            let bound_names = bound_names(&node.declaration);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_for_in_loop(
        &mut self,
        node: &'ast crate::statement::ForInLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;

        if let IterableLoopInitializer::Let(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }
        if let IterableLoopInitializer::Const(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_for_of_loop(
        &mut self,
        node: &'ast crate::statement::ForOfLoop,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit(node.body())?;

        if let IterableLoopInitializer::Let(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }
        if let IterableLoopInitializer::Const(node) = node.initializer() {
            let bound_names = bound_names(node);
            self.0.retain(|name| !bound_names.contains(name));
        }

        ControlFlow::Continue(())
    }

    fn visit_labelled(
        &mut self,
        node: &'ast crate::statement::Labelled,
    ) -> ControlFlow<Self::BreakTy> {
        if let LabelledItem::Statement(node) = node.item() {
            self.visit(node)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with(&mut self, node: &'ast With) -> ControlFlow<Self::BreakTy> {
        self.visit(node.statement())
    }
}

/// Returns `true` if the given statement returns a value.
#[must_use]
pub fn returns_value<'a, N>(node: &'a N) -> bool
where
    &'a N: Into<NodeRef<'a>>,
{
    ReturnsValueVisitor.visit(node.into()).is_break()
}

/// The [`Visitor`] used for [`returns_value`].
#[derive(Debug)]
struct ReturnsValueVisitor;

impl<'ast> Visitor<'ast> for ReturnsValueVisitor {
    type BreakTy = ();

    fn visit_block(&mut self, node: &'ast crate::statement::Block) -> ControlFlow<Self::BreakTy> {
        for statement in node.statement_list().statements() {
            match statement {
                StatementListItem::Declaration(_) => {}
                StatementListItem::Statement(node) => self.visit(node.as_ref())?,
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_statement(&mut self, node: &'ast Statement) -> ControlFlow<Self::BreakTy> {
        match node {
            Statement::Empty | Statement::Var(_) => {}
            Statement::Block(node) => self.visit(node)?,
            Statement::Labelled(node) => self.visit(node)?,
            _ => return ControlFlow::Break(()),
        }
        ControlFlow::Continue(())
    }

    fn visit_case(&mut self, node: &'ast crate::statement::Case) -> ControlFlow<Self::BreakTy> {
        for statement in node.body().statements() {
            match statement {
                StatementListItem::Declaration(_) => {}
                StatementListItem::Statement(node) => self.visit(node.as_ref())?,
            }
        }
        ControlFlow::Continue(())
    }

    fn visit_labelled(
        &mut self,
        node: &'ast crate::statement::Labelled,
    ) -> ControlFlow<Self::BreakTy> {
        match node.item() {
            LabelledItem::Statement(node) => self.visit(node)?,
            LabelledItem::FunctionDeclaration(_) => {}
        }
        ControlFlow::Continue(())
    }
}
