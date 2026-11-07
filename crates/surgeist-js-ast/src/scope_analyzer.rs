//! This module implements the scope analysis for the AST.
//!
//! The scope analysis is done in two steps:
//! 1. Collecting bindings: This step collects all the bindings in the AST and fills the scopes with them.
//! 2. Analyzing binding escapes: This step analyzes if the bindings escape their function scopes.

use crate::scope::{ScopeAccess, ScopeAnalysisError, ScopeStateError, ScopeWork};
use crate::{
    Declaration, Module, Script, StatementListItem, ToJsString,
    declaration::{Binding, ExportDeclaration, LexicalDeclaration, VariableList},
    expression::{Identifier, literal::ObjectMethodDefinition},
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassDeclaration, ClassElement,
        ClassElementName, ClassExpression, FormalParameterList, FunctionBody, FunctionDeclaration,
        FunctionExpression, GeneratorDeclaration, GeneratorExpression,
    },
    operations::{ContainsSymbol, LexicallyScopedDeclaration, VarScopedDeclaration},
    property::PropertyName,
    scope::{FunctionScopes, IdentifierReference, Scope, ScopeHandle},
    statement::{
        Block, Catch, ForInLoop, ForLoop, ForOfLoop, Switch, With,
        iteration::{ForLoopInitializer, IterableLoopInitializer},
    },
    visitor::{NodeRef, NodeRefMut, VisitorMut},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{convert::Infallible, ops::ControlFlow};
use surgeist_js_interner::{Interner, Sym};
use surgeist_js_string::JsString;

macro_rules! scope_flow {
    ($operation:expr) => {
        match $operation {
            Ok(value) => value,
            Err(error) => return ControlFlow::Break(error),
        }
    };
}

fn ordinary<T>(
    result: Result<T, ScopeAnalysisError<Infallible>>,
) -> Result<T, std::borrow::Cow<'static, str>> {
    match result {
        Ok(value) => Ok(value),
        Err(ScopeAnalysisError::Syntax(reason)) => Err(reason),
        Err(ScopeAnalysisError::Refused(never)) => match never {},
        Err(_) => panic!("uncontrolled scope operation failed"),
    }
}

pub(crate) fn collect_bindings<'a, N>(
    node: &'a mut N,
    strict: bool,
    eval: bool,
    scope: &Scope,
    interner: &Interner,
) -> Result<(), std::borrow::Cow<'static, str>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    ordinary(ScopeAccess::with_live(scope, None, |session| {
        collect_bindings_in_session(node, strict, eval, scope, interner, session)
    }))
}

pub(crate) fn analyze_binding_escapes<'a, N>(
    node: &'a mut N,
    in_eval: bool,
    scope: Scope,
    interner: &Interner,
) -> Result<(), std::borrow::Cow<'static, str>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    ordinary(ScopeAccess::with_live(&scope, None, |session| {
        analyze_binding_escapes_in_session(node, in_eval, scope.clone(), interner, session)
    }))
}

trait ScopeName {
    fn symbol(&self) -> Sym;
}
impl ScopeName for Sym {
    fn symbol(&self) -> Sym {
        *self
    }
}
impl ScopeName for Identifier {
    fn symbol(&self) -> Sym {
        self.sym()
    }
}
impl<T: ScopeName + ?Sized> ScopeName for &T {
    fn symbol(&self) -> Sym {
        (**self).symbol()
    }
}
impl<T: ScopeName + ?Sized> ScopeName for &mut T {
    fn symbol(&self) -> Sym {
        (**self).symbol()
    }
}

fn scope_name<E>(
    name: impl ScopeName,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<JsString, ScopeAnalysisError<E>> {
    session.admit(ScopeWork::NamePreparation)?;
    let symbol = name.symbol();
    if session.has_admission() {
        for _ in interner.resolve_expect(symbol).utf16() {
            session.admit(ScopeWork::NameUnit)?;
        }
    }
    Ok(symbol.to_js_string(interner))
}

fn scope_name_work<E>(
    work: crate::operations::NameWork,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    use crate::operations::NameWork;
    session.admit(match work {
        NameWork::Visit => ScopeWork::Node,
        NameWork::Record => ScopeWork::Record,
        NameWork::Comparison => ScopeWork::BindingProbe,
    })
}

macro_rules! scope_names {
    ($name:ident, $operation:ident, $item:ty) => {
        fn $name<'a, N, E>(
            node: &'a N,
            session: &mut ScopeAccess<'_, E>,
        ) -> Result<Vec<$item>, ScopeAnalysisError<E>>
        where
            &'a N: Into<NodeRef<'a>>,
        {
            crate::operations::$operation(node, Some(&mut |work| scope_name_work(work, session)))
        }
    };
}
scope_names!(scope_bound_names, try_bound_names, Sym);
scope_names!(
    scope_lexically_declared_names,
    try_lexically_declared_names,
    Sym
);
fn scope_var_declared_names<'a, N, E>(
    node: &'a N,
    session: &mut ScopeAccess<'_, E>,
) -> Result<FxHashSet<Sym>, ScopeAnalysisError<E>>
where
    &'a N: Into<NodeRef<'a>>,
{
    crate::operations::try_var_declared_names(
        node,
        Some(&mut |work| scope_name_work(work, session)),
    )
}
scope_names!(
    scope_lexically_scoped_declarations,
    try_lexically_scoped_declarations,
    LexicallyScopedDeclaration<'a>
);
scope_names!(
    scope_var_scoped_declarations,
    try_var_scoped_declarations,
    VarScopedDeclaration<'a>
);
#[cfg(feature = "annex-b")]
scope_names!(
    scope_annex_b_function_declarations_names,
    try_annex_b_function_declarations_names,
    Sym
);

fn scope_contains<N: crate::visitor::VisitWith, E>(
    node: &N,
    symbol: ContainsSymbol,
    session: &mut ScopeAccess<'_, E>,
) -> Result<bool, ScopeAnalysisError<E>> {
    crate::operations::try_contains(node, symbol, || session.admit(ScopeWork::Node))
}

fn scope_includes<T: PartialEq, E>(
    values: &[T],
    name: &T,
    session: &mut ScopeAccess<'_, E>,
) -> Result<bool, ScopeAnalysisError<E>> {
    for value in values {
        session.admit(ScopeWork::BindingProbe)?;
        if value == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn scope_push<T, E>(
    values: &mut Vec<T>,
    value: T,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    session.admit(ScopeWork::Record)?;
    if values.len() == values.capacity() {
        session.admit(ScopeWork::Capacity {
            elements: values.len(),
        })?;
        values
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
    }
    values.push(value);
    Ok(())
}

fn scope_lexical_bound_names<E>(
    declaration: &LexicallyScopedDeclaration<'_>,
    session: &mut ScopeAccess<'_, E>,
) -> Result<Vec<Sym>, ScopeAnalysisError<E>> {
    match *declaration {
        LexicallyScopedDeclaration::LexicalDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::FunctionDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::GeneratorDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::AsyncFunctionDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::AsyncGeneratorDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::ClassDeclaration(v) => scope_bound_names(v, session),
        LexicallyScopedDeclaration::AssignmentExpression(v) => scope_bound_names(v, session),
    }
}

fn scope_var_bound_names<E>(
    declaration: &VarScopedDeclaration<'_>,
    session: &mut ScopeAccess<'_, E>,
) -> Result<Vec<Sym>, ScopeAnalysisError<E>> {
    match *declaration {
        VarScopedDeclaration::VariableDeclaration(v) => scope_bound_names(v, session),
        VarScopedDeclaration::FunctionDeclaration(v) => scope_bound_names(v, session),
        VarScopedDeclaration::GeneratorDeclaration(v) => scope_bound_names(v, session),
        VarScopedDeclaration::AsyncFunctionDeclaration(v) => scope_bound_names(v, session),
        VarScopedDeclaration::AsyncGeneratorDeclaration(v) => scope_bound_names(v, session),
    }
}

/// Collect bindings and fill the scopes with them.
///
/// # Errors
/// Any break in the control flow that happened during the collection.
pub(crate) fn collect_bindings_in_session<'a, N, E>(
    node: &'a mut N,
    strict: bool,
    eval: bool,
    scope: &Scope,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let scope = session.bind_root(scope)?;
    collect_bindings_at(node, strict, eval, scope, interner, session)
}

pub(crate) fn collect_bindings_at<'a, N, E>(
    node: &'a mut N,
    strict: bool,
    eval: bool,
    scope: ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let mut visitor = BindingCollectorVisitor {
        strict,
        eval,
        in_arrow: false,
        scope,
        interner,
        session,
    };
    match visitor.visit(node) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(reason) => Err(reason),
    }
}

/// Analyze if bindings escape their function scopes.
///
/// # Errors
/// Any break in the control flow that happened during the analysis.
pub(crate) fn analyze_binding_escapes_in_session<'a, N, E>(
    node: &'a mut N,
    in_eval: bool,
    scope: Scope,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let scope = session.bind_root(&scope)?;
    analyze_binding_escapes_at(node, in_eval, scope, interner, session)
}

pub(crate) fn analyze_binding_escapes_at<'a, N, E>(
    node: &'a mut N,
    in_eval: bool,
    scope: ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let mut visitor = BindingEscapeAnalyzer {
        scope,
        direct_eval: in_eval,
        with: false,
        interner,
        session,
    };

    match visitor.visit(node.into()) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(reason) => Err(reason),
    }
}

struct BindingEscapeAnalyzer<'interner, 'session, 'admit, E> {
    session: &'session mut ScopeAccess<'admit, E>,
    scope: ScopeHandle,
    direct_eval: bool,
    with: bool,
    interner: &'interner Interner,
}

impl<'ast, E> VisitorMut<'ast> for BindingEscapeAnalyzer<'_, '_, '_, E> {
    type BreakTy = ScopeAnalysisError<E>;
    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        scope_flow!(self.session.admit(ScopeWork::Node));
        ControlFlow::Continue(())
    }

    fn visit_identifier_mut(&mut self, node: &'ast mut Identifier) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let name = scope_flow!(scope_name(node, self.interner, self.session));
        scope_flow!(
            self.session
                .access_binding(&self.scope, &name, self.direct_eval || self.with)
        );
        ControlFlow::Continue(())
    }

    fn visit_block_mut(&mut self, node: &'ast mut Block) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let direct_eval_old = self.direct_eval;
        self.direct_eval = node.contains_direct_eval || self.direct_eval;
        if let Some(scope) = &mut node.scope {
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }

        self.visit_statement_list_mut(&mut node.statements)?;
        if let Some(scope) = &mut node.scope {
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_switch_mut(&mut self, node: &'ast mut Switch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_expression_mut(&mut node.val)?;
        let direct_eval_old = self.direct_eval;
        self.direct_eval = node.contains_direct_eval || self.direct_eval;
        if let Some(scope) = &mut node.scope {
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }
        for case in &mut node.cases {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_case_mut(case)?;
        }
        if let Some(scope) = &mut node.scope {
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_with_mut(&mut self, node: &'ast mut With) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let with = self.with;
        self.with = true;
        if self.direct_eval {
            scope_flow!(self.session.escape_all_bindings(&node.scope));
        }
        self.visit_expression_mut(&mut node.expression)?;
        std::mem::swap(&mut self.scope, &mut node.scope);
        self.visit_statement_mut(&mut node.statement)?;
        std::mem::swap(&mut self.scope, &mut node.scope);
        scope_flow!(self.session.reorder_binding_indices(&node.scope));
        self.with = with;
        ControlFlow::Continue(())
    }

    fn visit_catch_mut(&mut self, node: &'ast mut Catch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let direct_eval_old = self.direct_eval;
        self.direct_eval = node.contains_direct_eval || self.direct_eval;
        if self.direct_eval {
            scope_flow!(self.session.escape_all_bindings(&node.scope));
        }
        std::mem::swap(&mut self.scope, &mut node.scope);
        if let Some(binding) = &mut node.parameter {
            self.visit_binding_mut(binding)?;
        }
        self.visit_block_mut(&mut node.block)?;
        std::mem::swap(&mut self.scope, &mut node.scope);
        scope_flow!(self.session.reorder_binding_indices(&node.scope));
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_for_loop_mut(&mut self, node: &'ast mut ForLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let direct_eval_old = self.direct_eval;
        self.direct_eval = node.inner.contains_direct_eval || self.direct_eval;
        if let Some(ForLoopInitializer::Lexical(decl)) = &mut node.inner.init {
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(&decl.scope));
            }
            std::mem::swap(&mut self.scope, &mut decl.scope);
        }
        if let Some(init) = &mut node.inner.init {
            self.visit_for_loop_initializer_mut(init)?;
        }
        if let Some(condition) = &mut node.inner.condition {
            self.visit_expression_mut(condition)?;
        }
        if let Some(final_expr) = &mut node.inner.final_expr {
            self.visit_expression_mut(final_expr)?;
        }
        self.visit_statement_mut(&mut node.inner.body)?;
        if let Some(ForLoopInitializer::Lexical(decl)) = &mut node.inner.init {
            std::mem::swap(&mut self.scope, &mut decl.scope);
            scope_flow!(self.session.reorder_binding_indices(&decl.scope));
        }
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_for_in_loop_mut(&mut self, node: &'ast mut ForInLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let direct_eval_old = self.direct_eval;
        if let Some(scope) = &mut node.target_scope {
            self.direct_eval = node.target_contains_direct_eval || self.direct_eval;
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }
        self.visit_expression_mut(&mut node.target)?;
        if let Some(scope) = &mut node.target_scope {
            self.direct_eval = direct_eval_old;
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        if let Some(scope) = &mut node.scope {
            self.direct_eval = node.contains_direct_eval || self.direct_eval;
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }
        self.visit_iterable_loop_initializer_mut(&mut node.initializer)?;
        self.visit_statement_mut(&mut node.body)?;
        if let Some(scope) = &mut node.scope {
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop_mut(&mut self, node: &'ast mut ForOfLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let direct_eval_old = self.direct_eval;
        if let Some(scope) = &mut node.iterable_scope {
            self.direct_eval = node.iterable_contains_direct_eval || self.direct_eval;
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }
        self.visit_expression_mut(&mut node.iterable)?;
        if let Some(scope) = &mut node.iterable_scope {
            self.direct_eval = direct_eval_old;
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        if let Some(scope) = &mut node.scope {
            self.direct_eval = node.contains_direct_eval || self.direct_eval;
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(scope));
            }
            std::mem::swap(&mut self.scope, scope);
        }
        self.visit_iterable_loop_initializer_mut(&mut node.init)?;
        self.visit_statement_mut(&mut node.body)?;
        if let Some(scope) = &mut node.scope {
            std::mem::swap(&mut self.scope, scope);
            scope_flow!(self.session.reorder_binding_indices(scope));
        }
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }

    fn visit_function_declaration_mut(
        &mut self,
        node: &'ast mut FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_generator_declaration_mut(
        &mut self,
        node: &'ast mut GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_async_function_declaration_mut(
        &mut self,
        node: &'ast mut AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_async_generator_declaration_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_function_expression_mut(
        &mut self,
        node: &'ast mut FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_generator_expression_mut(
        &mut self,
        node: &'ast mut GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_async_function_expression_mut(
        &mut self,
        node: &'ast mut AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_async_generator_expression_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_arrow_function_mut(
        &mut self,
        node: &'ast mut ArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_async_arrow_function_mut(
        &mut self,
        node: &'ast mut AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_class_declaration_mut(
        &mut self,
        node: &'ast mut ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        scope_flow!(self.session.escape_all_bindings(&node.name_scope));
        std::mem::swap(&mut self.scope, &mut node.name_scope);
        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            self.visit_function_expression_mut(constructor)?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        std::mem::swap(&mut self.scope, &mut node.name_scope);
        scope_flow!(self.session.reorder_binding_indices(&node.name_scope));
        ControlFlow::Continue(())
    }

    fn visit_class_expression_mut(
        &mut self,
        node: &'ast mut ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let Some(name_scope) = &mut node.name_scope {
            if self.direct_eval {
                scope_flow!(self.session.escape_all_bindings(name_scope));
            }
            scope_flow!(self.session.escape_all_bindings(name_scope));
            std::mem::swap(&mut self.scope, name_scope);
        }
        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            self.visit_function_expression_mut(constructor)?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        if let Some(name_scope) = &mut node.name_scope {
            std::mem::swap(&mut self.scope, name_scope);
            scope_flow!(self.session.reorder_binding_indices(name_scope));
        }
        ControlFlow::Continue(())
    }

    fn visit_class_element_mut(
        &mut self,
        node: &'ast mut ClassElement,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ClassElement::MethodDefinition(node) => {
                match &mut node.name {
                    ClassElementName::PropertyName(name) => self.visit_property_name_mut(name)?,
                    ClassElementName::PrivateName(name) => self.visit_private_name_mut(name)?,
                }
                self.visit_function_like(
                    &mut node.parameters,
                    &mut node.body,
                    &mut node.scopes,
                    node.contains_direct_eval,
                )
            }
            ClassElement::FieldDefinition(field) | ClassElement::StaticFieldDefinition(field) => {
                self.visit_property_name_mut(&mut field.name)?;
                if let Some(e) = &mut field.initializer {
                    std::mem::swap(&mut self.scope, &mut field.scope);
                    self.visit_expression_mut(e)?;
                    std::mem::swap(&mut self.scope, &mut field.scope);
                }
                ControlFlow::Continue(())
            }
            ClassElement::PrivateFieldDefinition(field)
            | ClassElement::PrivateStaticFieldDefinition(field) => {
                if let Some(e) = &mut field.initializer {
                    std::mem::swap(&mut self.scope, &mut field.scope);
                    self.visit_expression_mut(e)?;
                    std::mem::swap(&mut self.scope, &mut field.scope);
                }
                ControlFlow::Continue(())
            }
            ClassElement::StaticBlock(node) => {
                let contains_direct_eval = scope_flow!(scope_contains(
                    node.statements(),
                    ContainsSymbol::DirectEval,
                    self.session
                ));
                self.visit_function_like(
                    &mut FormalParameterList::default(),
                    &mut node.body,
                    &mut node.scopes,
                    contains_direct_eval,
                )
            }
        }
    }

    fn visit_object_method_definition_mut(
        &mut self,
        node: &'ast mut ObjectMethodDefinition,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_property_name_mut(&mut node.name)?;
        self.visit_function_like(
            &mut node.parameters,
            &mut node.body,
            &mut node.scopes,
            node.contains_direct_eval,
        )
    }

    fn visit_export_declaration_mut(
        &mut self,
        node: &'ast mut ExportDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ExportDeclaration::ReExport {
                specifier, kind, ..
            } => {
                self.visit_module_specifier_mut(specifier)?;
                self.visit_re_export_kind_mut(kind)
            }
            ExportDeclaration::List(list) => {
                for item in &mut **list {
                    scope_flow!(self.session.admit(ScopeWork::Node));
                    self.visit_export_specifier_mut(item)?;
                }
                ControlFlow::Continue(())
            }
            ExportDeclaration::VarStatement(var) => self.visit_var_declaration_mut(var),
            ExportDeclaration::Declaration(decl) => self.visit_declaration_mut(decl),
            ExportDeclaration::DefaultFunctionDeclaration(f) => {
                self.visit_function_declaration_mut(f)
            }
            ExportDeclaration::DefaultGeneratorDeclaration(g) => {
                self.visit_generator_declaration_mut(g)
            }
            ExportDeclaration::DefaultAsyncFunctionDeclaration(af) => {
                self.visit_async_function_declaration_mut(af)
            }
            ExportDeclaration::DefaultAsyncGeneratorDeclaration(ag) => {
                self.visit_async_generator_declaration_mut(ag)
            }
            ExportDeclaration::DefaultClassDeclaration(c) => self.visit_class_declaration_mut(c),
            ExportDeclaration::DefaultAssignmentExpression(expr) => {
                let name =
                    scope_flow!(scope_name(Sym::DEFAULT_EXPORT, self.interner, self.session));
                drop(scope_flow!(self.session.create_mutable_binding(
                    &self.scope,
                    name.clone(),
                    false
                )));
                scope_flow!(self.session.access_binding(&self.scope, &name, true));
                self.visit_expression_mut(expr)
            }
        }
    }

    fn visit_module_mut(&mut self, node: &'ast mut Module) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut scope = node.scope.clone();
        scope_flow!(self.session.escape_all_bindings(&scope));
        std::mem::swap(&mut self.scope, &mut scope);
        self.visit_module_item_list_mut(&mut node.items)?;
        std::mem::swap(&mut self.scope, &mut scope);
        scope_flow!(self.session.reorder_binding_indices(&scope));
        ControlFlow::Continue(())
    }
}

impl<E> BindingEscapeAnalyzer<'_, '_, '_, E> {
    fn visit_function_like(
        &mut self,
        parameters: &mut FormalParameterList,
        body: &mut FunctionBody,
        scopes: &mut FunctionScopes,
        contains_direct_eval: bool,
    ) -> ControlFlow<ScopeAnalysisError<E>> {
        let direct_eval_old = self.direct_eval;
        self.direct_eval = contains_direct_eval || self.direct_eval;
        if self.direct_eval {
            scope_flow!(self.session.escape_function_bindings(scopes));
        }
        let mut scope = scopes.parameter_scope();
        std::mem::swap(&mut self.scope, &mut scope);
        self.visit_formal_parameter_list_mut(parameters)?;
        std::mem::swap(&mut self.scope, &mut scope);
        scope = scopes.body_scope();
        std::mem::swap(&mut self.scope, &mut scope);
        self.visit_function_body_mut(body)?;
        std::mem::swap(&mut self.scope, &mut scope);
        if scope_flow!(self.session.arguments_object_accessed(scopes))
            && scopes.mapped_arguments_object
        {
            let parameter_names = scope_flow!(scope_bound_names(parameters, self.session));
            for name in parameter_names {
                scope_flow!(self.session.admit(ScopeWork::Node));
                let name = scope_flow!(scope_name(name, self.interner, self.session));
                scope_flow!(
                    self.session
                        .access_binding(&scopes.parameter_scope(), &name, true)
                );
            }
        }
        scope_flow!(self.session.reorder_function_bindings(scopes));
        self.direct_eval = direct_eval_old;
        ControlFlow::Continue(())
    }
}

struct BindingCollectorVisitor<'interner, 'session, 'admit, E> {
    session: &'session mut ScopeAccess<'admit, E>,
    strict: bool,
    eval: bool,
    scope: ScopeHandle,
    in_arrow: bool,
    interner: &'interner Interner,
}

impl<'ast, E> VisitorMut<'ast> for BindingCollectorVisitor<'_, '_, '_, E> {
    type BreakTy = ScopeAnalysisError<E>;
    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        scope_flow!(self.session.admit(ScopeWork::Node));
        ControlFlow::Continue(())
    }

    fn visit_this_mut(
        &mut self,
        _node: &'ast mut crate::expression::This,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        // NOTE: Arrow functions inherit 'this' from their enclosing scope, so we must escape it.
        if self.in_arrow {
            scope_flow!(
                self.session
                    .escape_this_in_enclosing_function_scope(&self.scope)
            );
        }
        ControlFlow::Continue(())
    }

    fn visit_function_declaration_mut(
        &mut self,
        node: &'ast mut FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            false,
        )
    }

    fn visit_generator_declaration_mut(
        &mut self,
        node: &'ast mut GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            false,
        )
    }

    fn visit_async_function_declaration_mut(
        &mut self,
        node: &'ast mut AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            false,
        )
    }

    fn visit_async_generator_declaration_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            false,
        )
    }

    fn visit_function_expression_mut(
        &mut self,
        node: &'ast mut FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let name = if node.has_binding_identifier {
            node.name()
        } else {
            None
        };
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            name,
            &mut node.name_scope,
            strict,
            false,
        )
    }

    fn visit_generator_expression_mut(
        &mut self,
        node: &'ast mut GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let name = if node.has_binding_identifier {
            node.name()
        } else {
            None
        };
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            name,
            &mut node.name_scope,
            strict,
            false,
        )
    }

    fn visit_async_function_expression_mut(
        &mut self,
        node: &'ast mut AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let name = if node.has_binding_identifier {
            node.name()
        } else {
            None
        };
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            name,
            &mut node.name_scope,
            strict,
            false,
        )
    }

    fn visit_async_generator_expression_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let name = if node.has_binding_identifier {
            node.name()
        } else {
            None
        };
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            name,
            &mut node.name_scope,
            strict,
            false,
        )
    }

    fn visit_arrow_function_mut(
        &mut self,
        node: &'ast mut ArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            true,
        )
    }

    fn visit_async_arrow_function_mut(
        &mut self,
        node: &'ast mut AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            true,
        )
    }

    fn visit_class_declaration_mut(
        &mut self,
        node: &'ast mut ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut name_scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
        let name = scope_flow!(scope_name(node.name(), self.interner, self.session));
        scope_flow!(
            self.session
                .create_immutable_binding(&name_scope, name, true)
        );
        std::mem::swap(&mut self.scope, &mut name_scope);
        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            self.visit_function_expression_mut(constructor)?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        std::mem::swap(&mut self.scope, &mut name_scope);
        node.name_scope = name_scope;
        ControlFlow::Continue(())
    }

    fn visit_class_expression_mut(
        &mut self,
        node: &'ast mut ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut name_scope = None;
        if let Some(name) = node.name
            && node.has_binding_identifier
        {
            let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
            let name = scope_flow!(scope_name(name, self.interner, self.session));
            scope_flow!(self.session.create_immutable_binding(&scope, name, true));
            node.name_scope = Some(scope.clone());
            std::mem::swap(&mut self.scope, &mut scope);
            name_scope = Some(scope);
        }

        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            self.visit_function_expression_mut(constructor)?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        if let Some(mut scope) = name_scope {
            std::mem::swap(&mut self.scope, &mut scope);
        }
        ControlFlow::Continue(())
    }

    fn visit_class_element_mut(
        &mut self,
        node: &'ast mut ClassElement,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ClassElement::MethodDefinition(node) => {
                match &mut node.name {
                    ClassElementName::PropertyName(name) => self.visit_property_name_mut(name)?,
                    ClassElementName::PrivateName(name) => self.visit_private_name_mut(name)?,
                }
                let strict = node.body.strict();
                self.visit_function_like(
                    &mut node.body,
                    &mut node.parameters,
                    &mut node.scopes,
                    None,
                    &mut None,
                    strict,
                    false,
                )
            }
            ClassElement::FieldDefinition(field) | ClassElement::StaticFieldDefinition(field) => {
                self.visit_property_name_mut(&mut field.name)?;
                let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), true));
                std::mem::swap(&mut self.scope, &mut scope);
                if let Some(e) = &mut field.initializer {
                    self.visit_expression_mut(e)?;
                }
                std::mem::swap(&mut self.scope, &mut scope);
                field.scope = scope;
                ControlFlow::Continue(())
            }
            ClassElement::PrivateFieldDefinition(field)
            | ClassElement::PrivateStaticFieldDefinition(field) => {
                let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), true));
                std::mem::swap(&mut self.scope, &mut scope);
                if let Some(e) = &mut field.initializer {
                    self.visit_expression_mut(e)?;
                }
                std::mem::swap(&mut self.scope, &mut scope);
                field.scope = scope;
                ControlFlow::Continue(())
            }
            ClassElement::StaticBlock(node) => {
                let strict = node.body.strict();
                self.visit_function_like(
                    &mut node.body,
                    &mut FormalParameterList::default(),
                    &mut node.scopes,
                    None,
                    &mut None,
                    strict,
                    false,
                )
            }
        }
    }

    fn visit_object_method_definition_mut(
        &mut self,
        node: &'ast mut ObjectMethodDefinition,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match &mut node.name {
            PropertyName::Literal(_) => {}
            PropertyName::Computed(name) => {
                self.visit_expression_mut(name)?;
            }
        }
        let strict = node.body.strict();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            None,
            &mut None,
            strict,
            false,
        )
    }

    fn visit_block_mut(&mut self, node: &'ast mut Block) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut scope = scope_flow!(block_declaration_instantiation(
            node,
            self.scope.clone(),
            self.interner,
            self.session
        ));
        if let Some(scope) = &mut scope {
            std::mem::swap(&mut self.scope, scope);
        }
        self.visit_statement_list_mut(&mut node.statements)?;
        if let Some(scope) = &mut scope {
            std::mem::swap(&mut self.scope, scope);
        }
        node.scope = scope;
        ControlFlow::Continue(())
    }

    fn visit_switch_mut(&mut self, node: &'ast mut Switch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_expression_mut(&mut node.val)?;
        let mut scope = scope_flow!(block_declaration_instantiation(
            node,
            self.scope.clone(),
            self.interner,
            self.session
        ));
        if let Some(scope) = &mut scope {
            std::mem::swap(&mut self.scope, scope);
        }
        for case in &mut *node.cases {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_case_mut(case)?;
        }
        if let Some(scope) = &mut scope {
            std::mem::swap(&mut self.scope, scope);
        }
        node.scope = scope;
        ControlFlow::Continue(())
    }

    fn visit_with_mut(&mut self, node: &'ast mut With) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        self.visit_expression_mut(&mut node.expression)?;
        let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
        std::mem::swap(&mut self.scope, &mut scope);
        self.visit_statement_mut(&mut node.statement)?;
        std::mem::swap(&mut self.scope, &mut scope);
        node.scope = scope;
        ControlFlow::Continue(())
    }

    fn visit_catch_mut(&mut self, node: &'ast mut Catch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
        if let Some(binding) = node.parameter() {
            match binding {
                Binding::Identifier(ident) => {
                    let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                    drop(scope_flow!(self.session.create_mutable_binding(
                        &scope,
                        ident.clone(),
                        false
                    )));
                }
                Binding::Pattern(pattern) => {
                    for ident in scope_flow!(scope_bound_names(pattern, self.session)) {
                        scope_flow!(self.session.admit(ScopeWork::Node));
                        let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                        drop(scope_flow!(
                            self.session.create_mutable_binding(&scope, ident, false)
                        ));
                    }
                }
            }
        }
        std::mem::swap(&mut self.scope, &mut scope);
        if let Some(binding) = &mut node.parameter {
            self.visit_binding_mut(binding)?;
        }
        self.visit_block_mut(&mut node.block)?;
        std::mem::swap(&mut self.scope, &mut scope);
        node.scope = scope;
        ControlFlow::Continue(())
    }

    fn visit_for_loop_mut(&mut self, node: &'ast mut ForLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let scope = match &mut node.inner.init {
            Some(ForLoopInitializer::Lexical(decl)) => {
                let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
                let names = scope_flow!(scope_bound_names(&decl.declaration, self.session));
                if decl.declaration.is_const() {
                    for name in &names {
                        scope_flow!(self.session.admit(ScopeWork::Node));
                        let name = scope_flow!(scope_name(name, self.interner, self.session));
                        scope_flow!(self.session.create_immutable_binding(&scope, name, true));
                    }
                } else {
                    for name in &names {
                        scope_flow!(self.session.admit(ScopeWork::Node));
                        let name = scope_flow!(scope_name(name, self.interner, self.session));
                        drop(scope_flow!(
                            self.session.create_mutable_binding(&scope, name, false)
                        ));
                    }
                }
                decl.scope = scope.clone();
                std::mem::swap(&mut self.scope, &mut scope);
                Some(scope)
            }
            _ => None,
        };
        if let Some(fli) = &mut node.inner.init {
            self.visit_for_loop_initializer_mut(fli)?;
        }
        if let Some(expr) = &mut node.inner.condition {
            self.visit_expression_mut(expr)?;
        }
        if let Some(expr) = &mut node.inner.final_expr {
            self.visit_expression_mut(expr)?;
        }
        self.visit_statement_mut(&mut node.inner.body)?;
        if let Some(mut scope) = scope {
            std::mem::swap(&mut self.scope, &mut scope);
        }
        ControlFlow::Continue(())
    }

    fn visit_for_in_loop_mut(&mut self, node: &'ast mut ForInLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let initializer_bound_names = match node.initializer() {
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => {
                scope_flow!(scope_bound_names(declaration, self.session))
            }
            _ => Vec::new(),
        };
        if initializer_bound_names.is_empty() {
            self.visit_expression_mut(&mut node.target)?;
        } else {
            let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
            for name in &initializer_bound_names {
                scope_flow!(self.session.admit(ScopeWork::Node));
                let name = scope_flow!(scope_name(name, self.interner, self.session));
                drop(scope_flow!(
                    self.session.create_mutable_binding(&scope, name, false)
                ));
            }
            std::mem::swap(&mut self.scope, &mut scope);
            self.visit_expression_mut(&mut node.target)?;
            std::mem::swap(&mut self.scope, &mut scope);
            node.target_scope = Some(scope);
        }
        let scope = match node.initializer() {
            IterableLoopInitializer::Let(declaration) => {
                let scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
                match declaration {
                    Binding::Identifier(ident) => {
                        let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                        drop(scope_flow!(self.session.create_mutable_binding(
                            &scope,
                            ident.clone(),
                            false
                        )));
                    }
                    Binding::Pattern(pattern) => {
                        for ident in scope_flow!(scope_bound_names(pattern, self.session)) {
                            scope_flow!(self.session.admit(ScopeWork::Node));
                            let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                            drop(scope_flow!(
                                self.session.create_mutable_binding(&scope, ident, false)
                            ));
                        }
                    }
                }
                Some(scope)
            }
            IterableLoopInitializer::Const(declaration) => {
                let scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
                match declaration {
                    Binding::Identifier(ident) => {
                        let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                        scope_flow!(self.session.create_immutable_binding(
                            &scope,
                            ident.clone(),
                            true
                        ));
                    }
                    Binding::Pattern(pattern) => {
                        for ident in scope_flow!(scope_bound_names(pattern, self.session)) {
                            scope_flow!(self.session.admit(ScopeWork::Node));
                            let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                            scope_flow!(self.session.create_immutable_binding(&scope, ident, true));
                        }
                    }
                }
                Some(scope)
            }
            _ => None,
        };
        if let Some(mut scope) = scope {
            std::mem::swap(&mut self.scope, &mut scope);
            self.visit_iterable_loop_initializer_mut(&mut node.initializer)?;
            self.visit_statement_mut(&mut node.body)?;
            std::mem::swap(&mut self.scope, &mut scope);
            node.scope = Some(scope);
        } else {
            self.visit_iterable_loop_initializer_mut(&mut node.initializer)?;
            self.visit_statement_mut(&mut node.body)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop_mut(&mut self, node: &'ast mut ForOfLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let initializer_bound_names = match node.initializer() {
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => {
                scope_flow!(scope_bound_names(declaration, self.session))
            }
            _ => Vec::new(),
        };
        if initializer_bound_names.is_empty() {
            self.visit_expression_mut(&mut node.iterable)?;
        } else {
            let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
            for name in &initializer_bound_names {
                scope_flow!(self.session.admit(ScopeWork::Node));
                let name = scope_flow!(scope_name(name, self.interner, self.session));
                drop(scope_flow!(
                    self.session.create_mutable_binding(&scope, name, false)
                ));
            }
            std::mem::swap(&mut self.scope, &mut scope);
            self.visit_expression_mut(&mut node.iterable)?;
            std::mem::swap(&mut self.scope, &mut scope);
            node.iterable_scope = Some(scope);
        }
        let scope = match node.initializer() {
            IterableLoopInitializer::Let(declaration) => {
                let scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
                match declaration {
                    Binding::Identifier(ident) => {
                        let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                        drop(scope_flow!(self.session.create_mutable_binding(
                            &scope,
                            ident.clone(),
                            false
                        )));
                    }
                    Binding::Pattern(pattern) => {
                        for ident in scope_flow!(scope_bound_names(pattern, self.session)) {
                            scope_flow!(self.session.admit(ScopeWork::Node));
                            let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                            drop(scope_flow!(
                                self.session.create_mutable_binding(&scope, ident, false)
                            ));
                        }
                    }
                }
                Some(scope)
            }
            IterableLoopInitializer::Const(declaration) => {
                let scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
                match declaration {
                    Binding::Identifier(ident) => {
                        let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                        scope_flow!(self.session.create_immutable_binding(
                            &scope,
                            ident.clone(),
                            true
                        ));
                    }
                    Binding::Pattern(pattern) => {
                        for ident in scope_flow!(scope_bound_names(pattern, self.session)) {
                            scope_flow!(self.session.admit(ScopeWork::Node));
                            let ident = scope_flow!(scope_name(ident, self.interner, self.session));
                            scope_flow!(self.session.create_immutable_binding(&scope, ident, true));
                        }
                    }
                }
                Some(scope)
            }
            _ => None,
        };
        if let Some(mut scope) = scope {
            std::mem::swap(&mut self.scope, &mut scope);
            self.visit_iterable_loop_initializer_mut(&mut node.init)?;
            self.visit_statement_mut(&mut node.body)?;
            std::mem::swap(&mut self.scope, &mut scope);
            node.scope = Some(scope);
        } else {
            self.visit_iterable_loop_initializer_mut(&mut node.init)?;
            self.visit_statement_mut(&mut node.body)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_module_mut(&mut self, node: &'ast mut Module) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let mut scope = scope_flow!(self.session.new_scope(self.scope.clone(), true));
        scope_flow!(module_instantiation(
            node,
            &scope,
            self.interner,
            self.session
        ));
        std::mem::swap(&mut self.scope, &mut scope);
        self.visit_module_item_list_mut(&mut node.items)?;
        std::mem::swap(&mut self.scope, &mut scope);
        node.scope = scope;
        ControlFlow::Continue(())
    }

    fn visit_script_mut(&mut self, node: &'ast mut Script) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if self.eval {
            self.visit_statement_list_mut(node.statements_mut())?;
        } else {
            match global_declaration_instantiation(node, &self.scope, self.interner, self.session) {
                Ok(()) => {
                    self.visit_statement_list_mut(node.statements_mut())?;
                }
                Err(e) => return ControlFlow::Break(e),
            }
        }
        ControlFlow::Continue(())
    }
}

impl<E> BindingCollectorVisitor<'_, '_, '_, E> {
    #[allow(clippy::too_many_arguments)]
    fn visit_function_like(
        &mut self,
        body: &mut FunctionBody,
        parameters: &mut FormalParameterList,
        scopes: &mut FunctionScopes,
        name: Option<Identifier>,
        name_scope: &mut Option<ScopeHandle>,
        strict: bool,
        arrow: bool,
    ) -> ControlFlow<ScopeAnalysisError<E>> {
        let strict = self.strict || strict;
        let old_in_arrow = self.in_arrow;
        self.in_arrow = arrow;

        let function_scope = if let Some(name) = name {
            let scope = scope_flow!(self.session.new_scope(self.scope.clone(), false));
            let name = scope_flow!(scope_name(name, self.interner, self.session));
            scope_flow!(self.session.create_immutable_binding(&scope, name, strict));
            *name_scope = Some(scope.clone());
            scope_flow!(self.session.new_scope(scope, true))
        } else {
            scope_flow!(self.session.new_scope(self.scope.clone(), true))
        };

        let function_scopes = scope_flow!(function_declaration_instantiation(
            body,
            parameters,
            arrow,
            strict,
            function_scope.clone(),
            self.interner,
            self.session
        ));

        let mut params_scope = function_scopes.parameter_scope();
        let mut body_scope = function_scopes.body_scope();

        std::mem::swap(&mut self.scope, &mut params_scope);
        self.visit_formal_parameter_list_mut(parameters)?;
        std::mem::swap(&mut self.scope, &mut params_scope);

        std::mem::swap(&mut self.scope, &mut body_scope);
        self.visit_function_body_mut(body)?;
        std::mem::swap(&mut self.scope, &mut body_scope);

        *scopes = function_scopes;

        self.in_arrow = old_in_arrow;

        ControlFlow::Continue(())
    }
}

/// Optimize scope indices when scopes only contain local bindings.
pub(crate) fn optimize_scope_indices_in_session<'a, N, E>(
    node: &'a mut N,
    scope: &Scope,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let root = session.bind_root(scope)?;
    optimize_scope_indices_at(node, &root, session)
}

pub(crate) fn optimize_scope_indices_at<'a, N, E>(
    node: &'a mut N,
    root: &ScopeHandle,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let index = session.scope_index(root)?;
    let mut visitor = ScopeIndexVisitor { index, session };
    match visitor.visit(node.into()) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}

pub(crate) fn optimize_scope_indices<'a, N>(node: &'a mut N, scope: &Scope)
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    ordinary(ScopeAccess::with_live(scope, None, |session| {
        optimize_scope_indices_in_session(node, scope, session)
    }))
    .expect("scope index optimization cannot fail");
}

/// Like [`optimize_scope_indices`] for a [`FunctionExpression`] compiled by
/// the `Function` constructor, which always pushes a function scope at runtime.
pub(crate) fn optimize_scope_indices_function_constructor(
    node: &mut FunctionExpression,
    scope: &Scope,
) {
    let mut scopes = crate::scope::PreparedScopes::live(scope);
    let mut session = ScopeAccess::borrowed(&mut scopes, None);
    ordinary(optimize_scope_indices_function_constructor_in_session(
        node,
        scope,
        &mut session,
    ))
    .expect("function scope index optimization cannot fail");
}

pub(crate) fn optimize_scope_indices_function_constructor_in_session<E>(
    node: &mut FunctionExpression,
    scope: &Scope,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    let root = session.bind_root(scope)?;
    let index = session.scope_index(&root)?;
    let mut visitor = ScopeIndexVisitor { index, session };
    match visitor.visit_function_like(
        &mut node.body,
        &mut node.parameters,
        &mut node.scopes,
        &mut node.name_scope,
        false,
        true,
    ) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}

struct ScopeIndexVisitor<'session, 'admit, E> {
    session: &'session mut ScopeAccess<'admit, E>,
    index: u32,
}

impl<'ast, E> VisitorMut<'ast> for ScopeIndexVisitor<'_, '_, E> {
    type BreakTy = ScopeAnalysisError<E>;
    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        scope_flow!(self.session.admit(ScopeWork::Node));
        ControlFlow::Continue(())
    }

    fn visit_function_declaration_mut(
        &mut self,
        node: &'ast mut FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            false,
            contains_direct_eval,
        )
    }

    fn visit_generator_declaration_mut(
        &mut self,
        node: &'ast mut GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            false,
            contains_direct_eval,
        )
    }

    fn visit_async_function_declaration_mut(
        &mut self,
        node: &'ast mut AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            false,
            contains_direct_eval,
        )
    }

    fn visit_async_generator_declaration_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            false,
            contains_direct_eval,
        )
    }

    fn visit_function_expression_mut(
        &mut self,
        node: &'ast mut FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut node.name_scope,
            false,
            contains_direct_eval,
        )
    }

    fn visit_generator_expression_mut(
        &mut self,
        node: &'ast mut GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut node.name_scope,
            false,
            contains_direct_eval,
        )
    }

    fn visit_async_function_expression_mut(
        &mut self,
        node: &'ast mut AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut node.name_scope,
            false,
            contains_direct_eval,
        )
    }

    fn visit_async_generator_expression_mut(
        &mut self,
        node: &'ast mut AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut node.name_scope,
            false,
            contains_direct_eval,
        )
    }

    fn visit_arrow_function_mut(
        &mut self,
        node: &'ast mut ArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            true,
            contains_direct_eval,
        )
    }

    fn visit_async_arrow_function_mut(
        &mut self,
        node: &'ast mut AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            true,
            contains_direct_eval,
        )
    }

    fn visit_class_declaration_mut(
        &mut self,
        node: &'ast mut ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        if !scope_flow!(self.session.all_bindings_local(&node.name_scope)) {
            self.index = scope_flow!(self.index.checked_add(1).ok_or(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted
            )));
        }
        scope_flow!(self.session.set_index(&node.name_scope, self.index));
        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            let node = constructor;
            self.visit_function_like(
                &mut node.body,
                &mut node.parameters,
                &mut node.scopes,
                &mut node.name_scope,
                false,
                true,
            )?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_class_expression_mut(
        &mut self,
        node: &'ast mut ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        if let Some(scope) = &node.name_scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        if let Some(super_ref) = &mut node.super_ref {
            self.visit_expression_mut(super_ref)?;
        }
        if let Some(constructor) = &mut node.constructor {
            self.visit_function_like(
                &mut constructor.body,
                &mut constructor.parameters,
                &mut constructor.scopes,
                &mut constructor.name_scope,
                false,
                true,
            )?;
        }
        for element in &mut *node.elements {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_class_element_mut(element)?;
        }
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_class_element_mut(
        &mut self,
        node: &'ast mut ClassElement,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ClassElement::MethodDefinition(node) => {
                match &mut node.name {
                    ClassElementName::PropertyName(name) => self.visit_property_name_mut(name)?,
                    ClassElementName::PrivateName(name) => self.visit_private_name_mut(name)?,
                }
                let contains_direct_eval = node.contains_direct_eval();
                self.visit_function_like(
                    &mut node.body,
                    &mut node.parameters,
                    &mut node.scopes,
                    &mut None,
                    false,
                    contains_direct_eval,
                )
            }
            ClassElement::FieldDefinition(field) | ClassElement::StaticFieldDefinition(field) => {
                self.visit_property_name_mut(&mut field.name)?;
                let index = self.index;
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
                scope_flow!(self.session.set_index(&field.scope, self.index));
                if let Some(e) = &mut field.initializer {
                    self.visit_expression_mut(e)?;
                }
                self.index = index;
                ControlFlow::Continue(())
            }
            ClassElement::PrivateFieldDefinition(field)
            | ClassElement::PrivateStaticFieldDefinition(field) => {
                let index = self.index;
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
                scope_flow!(self.session.set_index(&field.scope, self.index));
                if let Some(e) = &mut field.initializer {
                    self.visit_expression_mut(e)?;
                }
                self.index = index;
                ControlFlow::Continue(())
            }
            ClassElement::StaticBlock(node) => self.visit_function_like(
                &mut node.body,
                &mut FormalParameterList::default(),
                &mut node.scopes,
                &mut None,
                false,
                true,
            ),
        }
    }

    fn visit_object_method_definition_mut(
        &mut self,
        node: &'ast mut ObjectMethodDefinition,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match &mut node.name {
            PropertyName::Literal(_) => {}
            PropertyName::Computed(name) => {
                self.visit_expression_mut(name)?;
            }
        }
        let contains_direct_eval = node.contains_direct_eval();
        self.visit_function_like(
            &mut node.body,
            &mut node.parameters,
            &mut node.scopes,
            &mut None,
            false,
            contains_direct_eval,
        )
    }

    fn visit_block_mut(&mut self, node: &'ast mut Block) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        if let Some(scope) = &node.scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        self.visit_statement_list_mut(&mut node.statements)?;
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_switch_mut(&mut self, node: &'ast mut Switch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        self.visit_expression_mut(&mut node.val)?;
        if let Some(scope) = &node.scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        for case in &mut *node.cases {
            scope_flow!(self.session.admit(ScopeWork::Node));
            self.visit_case_mut(case)?;
        }
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_with_mut(&mut self, node: &'ast mut With) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        self.visit_expression_mut(&mut node.expression)?;
        self.index = scope_flow!(self.index.checked_add(1).ok_or(ScopeAnalysisError::State(
            ScopeStateError::IdentityExhausted
        )));
        scope_flow!(self.session.set_index(&node.scope, self.index));
        self.visit_statement_mut(&mut node.statement)?;
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_catch_mut(&mut self, node: &'ast mut Catch) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        if !scope_flow!(self.session.all_bindings_local(&node.scope)) {
            self.index = scope_flow!(self.index.checked_add(1).ok_or(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted
            )));
        }
        scope_flow!(self.session.set_index(&node.scope, self.index));
        if let Some(binding) = &mut node.parameter {
            self.visit_binding_mut(binding)?;
        }
        self.visit_block_mut(&mut node.block)?;
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_for_loop_mut(&mut self, node: &'ast mut ForLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        let index = self.index;
        if let Some(ForLoopInitializer::Lexical(decl)) = &mut node.inner.init {
            if !scope_flow!(self.session.all_bindings_local(&decl.scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(&decl.scope, self.index));
        }
        if let Some(fli) = &mut node.inner.init {
            self.visit_for_loop_initializer_mut(fli)?;
        }
        if let Some(expr) = &mut node.inner.condition {
            self.visit_expression_mut(expr)?;
        }
        if let Some(expr) = &mut node.inner.final_expr {
            self.visit_expression_mut(expr)?;
        }
        self.visit_statement_mut(&mut node.inner.body)?;
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_for_in_loop_mut(&mut self, node: &'ast mut ForInLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        {
            let index = self.index;
            if let Some(scope) = &node.target_scope {
                if !scope_flow!(self.session.all_bindings_local(scope)) {
                    self.index = scope_flow!(self.index.checked_add(1).ok_or(
                        ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                    ));
                }
                scope_flow!(self.session.set_index(scope, self.index));
            }
            self.visit_expression_mut(&mut node.target)?;
            self.index = index;
        }
        let index = self.index;
        if let Some(scope) = &node.scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        self.visit_iterable_loop_initializer_mut(&mut node.initializer)?;
        self.visit_statement_mut(&mut node.body)?;
        self.index = index;
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop_mut(&mut self, node: &'ast mut ForOfLoop) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        {
            let index = self.index;
            if let Some(scope) = &node.iterable_scope {
                if !scope_flow!(self.session.all_bindings_local(scope)) {
                    self.index = scope_flow!(self.index.checked_add(1).ok_or(
                        ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                    ));
                }
                scope_flow!(self.session.set_index(scope, self.index));
            }
            self.visit_expression_mut(&mut node.iterable)?;
            self.index = index;
        }
        let index = self.index;
        if let Some(scope) = &node.scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        self.visit_iterable_loop_initializer_mut(&mut node.init)?;
        self.visit_statement_mut(&mut node.body)?;
        self.index = index;
        ControlFlow::Continue(())
    }
}

impl<E> ScopeIndexVisitor<'_, '_, E> {
    fn visit_function_like(
        &mut self,
        body: &mut FunctionBody,
        parameters: &mut FormalParameterList,
        scopes: &mut FunctionScopes,
        name_scope: &mut Option<ScopeHandle>,
        arrow: bool,
        force_function_scope: bool,
    ) -> ControlFlow<ScopeAnalysisError<E>> {
        let index = self.index;
        if let Some(scope) = name_scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }

        if force_function_scope
            || !scope_flow!(self.session.all_bindings_local(scopes.function_scope()))
        {
            scopes.requires_function_scope = true;
            self.index = scope_flow!(self.index.checked_add(1).ok_or(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted
            )));
        } else if !arrow {
            assert!(scopes.function_scope().is_function());
            scopes.requires_function_scope =
                scope_flow!(self.session.escaped_this(scopes.function_scope()))
                    || scope_flow!(scope_contains(
                        parameters,
                        ContainsSymbol::Super,
                        self.session
                    ))
                    || scope_flow!(scope_contains(body, ContainsSymbol::Super, self.session))
                    || scope_flow!(scope_contains(
                        parameters,
                        ContainsSymbol::NewTarget,
                        self.session
                    ))
                    || scope_flow!(scope_contains(
                        body,
                        ContainsSymbol::NewTarget,
                        self.session
                    ));
            self.index = scope_flow!(
                self.index
                    .checked_add(u32::from(scopes.requires_function_scope))
                    .ok_or(ScopeAnalysisError::State(
                        ScopeStateError::IdentityExhausted
                    ))
            );
        }

        scope_flow!(self.session.set_index(&scopes.function_scope, self.index));
        if let Some(scope) = &scopes.parameters_eval_scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        self.visit_formal_parameter_list_mut(parameters)?;
        if let Some(scope) = &scopes.parameters_scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        if let Some(scope) = &scopes.lexical_scope {
            if !scope_flow!(self.session.all_bindings_local(scope)) {
                self.index = scope_flow!(self.index.checked_add(1).ok_or(
                    ScopeAnalysisError::State(ScopeStateError::IdentityExhausted)
                ));
            }
            scope_flow!(self.session.set_index(scope, self.index));
        }
        self.visit_function_body_mut(body)?;
        self.index = index;
        ControlFlow::Continue(())
    }
}

pub(crate) fn script_declarations<E>(
    script: &Script,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<crate::ScriptDeclarations, ScopeAnalysisError<E>> {
    let mut declarations = crate::ScriptDeclarations::default();
    let lexical_names = scope_lexically_declared_names(script, session)?;
    for name in &lexical_names {
        let name = scope_name(name, interner, session)?;
        scope_push(&mut declarations.lexical, name, session)?;
    }
    let mut seen_var_names = Vec::new();
    for declaration in scope_var_scoped_declarations(script, session)? {
        session.admit(ScopeWork::Node)?;
        for name in scope_var_bound_names(&declaration, session)? {
            session.admit(ScopeWork::Node)?;
            if !scope_includes(&seen_var_names, &name, session)? {
                scope_push(&mut seen_var_names, name, session)?;
                let name = scope_name(name, interner, session)?;
                scope_push(&mut declarations.var, name, session)?;
            }
        }
    }
    #[cfg(feature = "annex-b")]
    if !script.strict() {
        for name in scope_annex_b_function_declarations_names(script, session)? {
            session.admit(ScopeWork::Node)?;
            if !scope_includes(&lexical_names, &name, session)? {
                let name = scope_name(name, interner, session)?;
                scope_push(&mut declarations.annex_b, name, session)?;
            }
        }
    }
    Ok(declarations)
}

/// `GlobalDeclarationInstantiation ( script, env )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
///
/// # Errors
///
/// - If a duplicate lexical declaration is found.
fn global_declaration_instantiation<E>(
    script: &Script,
    env: &ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    if !session.projects_declarations() {
        // 1. Let lexNames be the LexicallyDeclaredNames of script.
        let lex_names = scope_lexically_declared_names(script, session)?;

        // 2. Let varNames be the VarDeclaredNames of script.
        let var_names = scope_var_declared_names(script, session)?;

        // 3. For each element name of lexNames, do
        for name in lex_names {
            session.admit(ScopeWork::Node)?;
            let name = scope_name(name, interner, session)?;

            // Note: Our implementation differs from the spec here.
            // a. If env.HasVarDeclaration(name) is true, throw a SyntaxError exception.
            // b. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if session.has_binding(env, &name)? {
                return Err(ScopeAnalysisError::Syntax(
                    "duplicate lexical declaration".into(),
                ));
            }
        }

        // 4. For each element name of varNames, do
        for name in var_names {
            session.admit(ScopeWork::Node)?;
            let name = scope_name(name, interner, session)?;

            // a. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if session.has_lex_binding(env, &name)? {
                return Err(ScopeAnalysisError::Syntax(
                    "duplicate lexical declaration".into(),
                ));
            }
        }
    }

    // 13. Let lexDeclarations be the LexicallyScopedDeclarations of script.
    // 14. Let privateEnv be null.
    // 15. For each element d of lexDeclarations, do
    for statement in &**script.statements() {
        session.admit(ScopeWork::Node)?;
        // a. NOTE: Lexically declared names are only instantiated here but not initialized.
        // b. For each element dn of the BoundNames of d, do
        //     i. If IsConstantDeclaration of d is true, then
        //         1. Perform ? env.CreateImmutableBinding(dn, true).
        //     ii. Else,
        //         1. Perform ? env.CreateMutableBinding(dn, false).
        if let StatementListItem::Declaration(declaration) = statement {
            match declaration.as_ref() {
                Declaration::ClassDeclaration(class) => {
                    for name in scope_bound_names(class.as_ref(), session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        if session.projects_declarations() {
                            session.project_binding(env, name, true)?;
                        } else {
                            drop(session.create_mutable_binding(env, name, false)?);
                        }
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        if session.projects_declarations() {
                            session.project_binding(env, name, true)?;
                        } else {
                            drop(session.create_mutable_binding(env, name, false)?);
                        }
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        if session.projects_declarations() {
                            session.project_binding(env, name, false)?;
                        } else {
                            session.create_immutable_binding(env, name, true)?;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// `BlockDeclarationInstantiation ( code, env )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-blockdeclarationinstantiation
fn block_declaration_instantiation<'a, N, E>(
    block: &'a N,
    scope: ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<Option<ScopeHandle>, ScopeAnalysisError<E>>
where
    &'a N: Into<NodeRef<'a>>,
{
    let scope = session.new_scope(scope, false)?;

    // 1. Let declarations be the LexicallyScopedDeclarations of code.
    let declarations = scope_lexically_scoped_declarations(block, session)?;

    // 2. Let privateEnv be the running execution context's PrivateEnvironment.
    // Note: Private environments are currently handled differently.

    // 3. For each element d of declarations, do
    for d in &declarations {
        session.admit(ScopeWork::Node)?;
        // i. If IsConstantDeclaration of d is true, then
        if let LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Const(d)) = d {
            // a. For each element dn of the BoundNames of d, do
            for dn in scope_bound_names::<VariableList, E>(d, session)? {
                session.admit(ScopeWork::Node)?;
                // 1. Perform ! env.CreateImmutableBinding(dn, true).
                let dn = scope_name(dn, interner, session)?;
                session.create_immutable_binding(&scope, dn, true)?;
            }
        }
        // ii. Else,
        else {
            // a. For each element dn of the BoundNames of d, do
            for dn in scope_lexical_bound_names(d, session)? {
                session.admit(ScopeWork::Node)?;
                let dn = scope_name(dn, interner, session)?;

                #[cfg(not(feature = "annex-b"))]
                // 1. Perform ! env.CreateMutableBinding(dn, false). NOTE: This step is replaced in section B.3.2.6.
                drop(session.create_mutable_binding(&scope, dn, false)?);

                #[cfg(feature = "annex-b")]
                // 1. If ! env.HasBinding(dn) is false, then
                if !session.has_binding(&scope, &dn)? {
                    // a. Perform  ! env.CreateMutableBinding(dn, false).
                    drop(session.create_mutable_binding(&scope, dn, false)?);
                }
            }
        }
    }

    if session.num_bindings(&scope)? > 0 {
        Ok(Some(scope))
    } else {
        Ok(None)
    }
}

/// `FunctionDeclarationInstantiation ( func, argumentsList )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
fn function_declaration_instantiation<E>(
    body: &FunctionBody,
    formals: &FormalParameterList,
    arrow: bool,
    strict: bool,
    function_scope: ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<FunctionScopes, ScopeAnalysisError<E>> {
    let mut scopes = FunctionScopes {
        function_scope,
        parameters_eval_scope: None,
        parameters_scope: None,
        lexical_scope: None,
        mapped_arguments_object: false,
        requires_function_scope: false,
    };

    // 1. Let calleeContext be the running execution context.
    // 2. Let code be func.[[ECMAScriptCode]].
    // 3. Let strict be func.[[Strict]].
    // 4. Let formals be func.[[FormalParameters]].

    // 5. Let parameterNames be the BoundNames of formals.
    let mut parameter_names = scope_bound_names(formals, session)?;

    // 6. If parameterNames has any duplicate entries, let hasDuplicates be true. Otherwise, let hasDuplicates be false.
    // 7. Let simpleParameterList be IsSimpleParameterList of formals.

    // 8. Let hasParameterExpressions be ContainsExpression of formals.
    let has_parameter_expressions = formals.has_expressions();

    // 9. Let varNames be the VarDeclaredNames of code.
    let var_names = scope_var_declared_names(body, session)?;

    // 10. Let varDeclarations be the VarScopedDeclarations of code.
    let var_declarations = scope_var_scoped_declarations(body, session)?;

    // 11. Let lexicalNames be the LexicallyDeclaredNames of code.
    let lexical_names = scope_lexically_declared_names(body, session)?;

    // 12. Let functionNames be a new empty List.
    let mut function_names = Vec::new();

    // 13. Let functionsToInitialize be a new empty List.
    // let mut functions_to_initialize = Vec::new();

    // 14. For each element d of varDeclarations, in reverse List order, do
    for declaration in var_declarations.iter().rev() {
        session.admit(ScopeWork::Node)?;
        // a. If d is neither a VariableDeclaration nor a ForBinding nor a BindingIdentifier, then
        // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
        // a.ii. Let fn be the sole element of the BoundNames of d.
        let name = match *declaration {
            VarScopedDeclaration::FunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::GeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncFunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncGeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::VariableDeclaration(_) => continue,
        };

        // a.iii. If functionNames does not contain fn, then
        if !scope_includes(&function_names, &name.sym(), session)? {
            // 1. Insert fn as the first element of functionNames.
            scope_push(&mut function_names, name.sym(), session)?;
        }
    }

    for index in 0..function_names.len() / 2 {
        session.admit(ScopeWork::Node)?;
        let opposite = function_names.len() - index - 1;
        function_names.swap(index, opposite);
    }

    // 15. Let argumentsObjectNeeded be true.
    let mut arguments_object_needed = true;

    let arguments = Sym::ARGUMENTS;

    // 16. If func.[[ThisMode]] is lexical, then
    // 17. Else if parameterNames contains "arguments", then
    if arrow || scope_includes(&parameter_names, &arguments, session)? {
        // 16.a. NOTE: Arrow functions never have an arguments object.
        // 16.b. Set argumentsObjectNeeded to false.
        // 17.a. Set argumentsObjectNeeded to false.
        arguments_object_needed = false;
    }
    // 18. Else if hasParameterExpressions is false, then
    else if !has_parameter_expressions {
        //a. If functionNames contains "arguments" or lexicalNames contains "arguments", then
        if scope_includes(&function_names, &arguments, session)?
            || scope_includes(&lexical_names, &arguments, session)?
        {
            // i. Set argumentsObjectNeeded to false.
            arguments_object_needed = false;
        }
    }

    // 19. If strict is true or hasParameterExpressions is false, then
    let env = if strict || !has_parameter_expressions {
        // a. NOTE: Only a single Environment Record is needed for the parameters,
        //    since calls to eval in strict mode code cannot create new bindings which are visible outside of the eval.
        // b. Let env be the LexicalEnvironment of calleeContext.
        scopes.function_scope.clone()
    }
    // 20. Else,
    else {
        // a. NOTE: A separate Environment Record is needed to ensure that bindings created by
        //    direct eval calls in the formal parameter list are outside the environment where parameters are declared.
        // b. Let calleeEnv be the LexicalEnvironment of calleeContext.
        // c. Let env be NewDeclarativeEnvironment(calleeEnv).
        // d. Assert: The VariableEnvironment of calleeContext is calleeEnv.
        // e. Set the LexicalEnvironment of calleeContext to env.
        let scope = session.new_scope(scopes.function_scope.clone(), false)?;
        scopes.parameters_eval_scope = Some(scope.clone());
        scope
    };

    // 22. If argumentsObjectNeeded is true, then
    //
    // NOTE(HalidOdat): Has been moved up, so "arguments" gets registered as
    //     the first binding in the environment with index 0.
    if arguments_object_needed {
        let arguments = scope_name(arguments, interner, session)?;

        // c. If strict is true, then
        if strict {
            // i. Perform ! env.CreateImmutableBinding("arguments", false).
            // ii. NOTE: In strict mode code early errors prevent attempting to assign
            //           to this binding, so its mutability is not observable.
            session.create_immutable_binding(&env, arguments.clone(), false)?;
        }
        // d. Else,
        else {
            // i. Perform ! env.CreateMutableBinding("arguments", false).
            drop(session.create_mutable_binding(&env, arguments.clone(), false)?);
        }
    }

    // 21. For each String paramName of parameterNames, do
    for param_name in &parameter_names {
        session.admit(ScopeWork::Node)?;
        let param_name = scope_name(param_name, interner, session)?;

        // a. Let alreadyDeclared be ! env.HasBinding(paramName).
        let already_declared = session.has_binding(&env, &param_name)?;

        // b. NOTE: Early errors ensure that duplicate parameter names can only occur in non-strict
        //    functions that do not have parameter default values or rest parameters.

        // c. If alreadyDeclared is false, then
        if !already_declared {
            // i. Perform ! env.CreateMutableBinding(paramName, false).
            drop(session.create_mutable_binding(&env, param_name.clone(), false)?);

            // Note: In this case the function contains a mapped arguments object.
            // Because we do not track (yet) if the mapped arguments object escapes the function,
            // we have to assume that the binding might escape trough the arguments object.
            if arguments_object_needed && !strict && formals.is_simple() {
                scopes.mapped_arguments_object = true;
            }

            // Note: These steps are not necessary in our implementation.
            // ii. If hasDuplicates is true, then
            //     1. Perform ! env.InitializeBinding(paramName, undefined).
        }
    }

    // 22. If argumentsObjectNeeded is true, then
    if arguments_object_needed {
        // MOVED: a-e.
        //
        // NOTE(HalidOdat): Has been moved up, see comment above.

        // f. Let parameterBindings be the list-concatenation of parameterNames and « "arguments" ».
        scope_push(&mut parameter_names, arguments, session)?;
    }

    // 23. Else,
    //     a. Let parameterBindings be parameterNames.
    let mut parameter_bindings = Vec::new();
    for name in &parameter_names {
        session.admit(ScopeWork::Record)?;
        scope_push(&mut parameter_bindings, *name, session)?;
    }

    // 27. If hasParameterExpressions is false, then
    // 28. Else,
    #[allow(unused_variables, unused_mut)]
    let (mut instantiated_var_names, mut var_env) = if has_parameter_expressions {
        // a. NOTE: A separate Environment Record is needed to ensure that closures created by
        //          expressions in the formal parameter list do not have
        //          visibility of declarations in the function body.
        // b. Let varEnv be NewDeclarativeEnvironment(env).
        let var_env = session.new_scope(env.clone(), false)?;
        scopes.parameters_scope = Some(var_env.clone());

        // c. Set the VariableEnvironment of calleeContext to varEnv.

        // d. Let instantiatedVarNames be a new empty List.
        let mut instantiated_var_names = Vec::new();

        // e. For each element n of varNames, do
        for n in var_names {
            session.admit(ScopeWork::Node)?;
            // i. If instantiatedVarNames does not contain n, then
            if !scope_includes(&instantiated_var_names, &n, session)? {
                // 1. Append n to instantiatedVarNames.
                scope_push(&mut instantiated_var_names, n, session)?;

                let n_string = scope_name(n, interner, session)?;

                // 2. Perform ! varEnv.CreateMutableBinding(n, false).
                drop(session.create_mutable_binding(&var_env, n_string, false)?);
            }
        }

        (instantiated_var_names, var_env)
    } else {
        // a. NOTE: Only a single Environment Record is needed for the parameters and top-level vars.
        // b. Let instantiatedVarNames be a copy of the List parameterBindings.
        let mut instantiated_var_names = parameter_bindings;

        // c. For each element n of varNames, do
        for n in var_names {
            session.admit(ScopeWork::Node)?;
            // i. If instantiatedVarNames does not contain n, then
            if !scope_includes(&instantiated_var_names, &n, session)? {
                // 1. Append n to instantiatedVarNames.
                scope_push(&mut instantiated_var_names, n, session)?;

                let n = scope_name(n, interner, session)?;

                // 2. Perform ! env.CreateMutableBinding(n, false).
                // 3. Perform ! env.InitializeBinding(n, undefined).
                drop(session.create_mutable_binding(&env, n, true)?);
            }
        }

        // d. Let varEnv be env.
        (instantiated_var_names, env)
    };

    // 29. NOTE: Annex B.3.2.1 adds additional steps at this point.
    // 29. If strict is false, then
    #[cfg(feature = "annex-b")]
    if !strict {
        // a. For each FunctionDeclaration f that is directly contained in the StatementList
        //    of a Block, CaseClause, or DefaultClause, do
        for f in scope_annex_b_function_declarations_names(body, session)? {
            session.admit(ScopeWork::Node)?;
            // i. Let F be StringValue of the BindingIdentifier of f.
            // ii. If replacing the FunctionDeclaration f with a VariableStatement that has F
            //     as a BindingIdentifier would not produce any Early Errors
            //     for func and parameterNames does not contain F, then
            if !scope_includes(&lexical_names, &f, session)?
                && !scope_includes(&parameter_names, &f, session)?
            {
                // 1. NOTE: A var binding for F is only instantiated here if it is neither a
                //    VarDeclaredName, the name of a formal parameter, or another FunctionDeclaration.

                // 2. If initializedBindings does not contain F and F is not "arguments", then
                if !scope_includes(&instantiated_var_names, &f, session)? && f != arguments {
                    let f_string = scope_name(f, interner, session)?;

                    // a. Perform ! varEnv.CreateMutableBinding(F, false).
                    // b. Perform ! varEnv.InitializeBinding(F, undefined).
                    drop(session.create_mutable_binding(&var_env, f_string, false)?);

                    // c. Append F to instantiatedVarNames.
                    scope_push(&mut instantiated_var_names, f, session)?;
                }
            }
        }
    }

    // 30. If strict is false, then
    // 31. Else,
    let lex_env = if strict {
        // a. Let lexEnv be varEnv.
        var_env
    } else {
        // a. Let lexEnv be NewDeclarativeEnvironment(varEnv).
        // b. NOTE: Non-strict functions use a separate Environment Record for top-level lexical
        //    declarations so that a direct eval can determine whether any var scoped declarations
        //    introduced by the eval code conflict with pre-existing top-level lexically scoped declarations.
        //    This is not needed for strict functions because a strict direct eval always
        //    places all declarations into a new Environment Record.
        let lex_env = session.new_scope(var_env, false)?;
        scopes.lexical_scope = Some(lex_env.clone());
        lex_env
    };

    // 32. Set the LexicalEnvironment of calleeContext to lexEnv.
    // 33. Let lexDeclarations be the LexicallyScopedDeclarations of code.
    // 34. For each element d of lexDeclarations, do
    //     a. NOTE: A lexically declared name cannot be the same as a function/generator declaration,
    //        formal parameter, or a var name. Lexically declared names are only instantiated here but not initialized.
    //     b. For each element dn of the BoundNames of d, do
    //         i. If IsConstantDeclaration of d is true, then
    //             1. Perform ! lexEnv.CreateImmutableBinding(dn, true).
    //         ii. Else,
    //             1. Perform ! lexEnv.CreateMutableBinding(dn, false).
    for statement in body.statements() {
        session.admit(ScopeWork::Node)?;
        if let StatementListItem::Declaration(declaration) = statement {
            match declaration.as_ref() {
                Declaration::ClassDeclaration(class) => {
                    for name in scope_bound_names(class.as_ref(), session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        drop(session.create_mutable_binding(&lex_env, name, false)?);
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        drop(session.create_mutable_binding(&lex_env, name, false)?);
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        session.admit(ScopeWork::Node)?;
                        let name = scope_name(name, interner, session)?;
                        session.create_immutable_binding(&lex_env, name, true)?;
                    }
                }
                _ => {}
            }
        }
    }

    // 35. Let privateEnv be the PrivateEnvironment of calleeContext.
    // 36. For each Parse Node f of functionsToInitialize, do

    if let Some(lexical_scope) = &scopes.lexical_scope
        && session.num_bindings(lexical_scope)? == 0
    {
        scopes.lexical_scope = None;
    }

    // 37. Return unused.
    Ok(scopes)
}

/// Abstract operation [`InitializeEnvironment ( )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-source-text-module-record-initialize-environment
fn module_instantiation<E>(
    module: &Module,
    env: &ScopeHandle,
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    for entry in module.items().import_entries() {
        session.admit(ScopeWork::Node)?;
        let local_name = scope_name(entry.local_name(), interner, session)?;
        session.create_immutable_binding(env, local_name, true)?;
    }
    let var_declarations = scope_var_scoped_declarations(module, session)?;
    let mut declared_var_names = Vec::new();
    for var in var_declarations {
        session.admit(ScopeWork::Node)?;
        for name in scope_var_bound_names(&var, session)? {
            session.admit(ScopeWork::Node)?;
            let name = scope_name(name, interner, session)?;
            if !scope_includes(&declared_var_names, &name, session)? {
                drop(session.create_mutable_binding(env, name.clone(), false)?);
                scope_push(&mut declared_var_names, name, session)?;
            }
        }
    }

    let lex_declarations = scope_lexically_scoped_declarations(module, session)?;
    for declaration in lex_declarations {
        session.admit(ScopeWork::Node)?;
        match declaration {
            LexicallyScopedDeclaration::FunctionDeclaration(f) => {
                let name = scope_name(scope_bound_names(f, session)?[0], interner, session)?;
                drop(session.create_mutable_binding(env, name, false)?);
            }
            LexicallyScopedDeclaration::GeneratorDeclaration(g) => {
                let name = scope_name(scope_bound_names(g, session)?[0], interner, session)?;
                drop(session.create_mutable_binding(env, name, false)?);
            }
            LexicallyScopedDeclaration::AsyncFunctionDeclaration(af) => {
                let name = scope_name(scope_bound_names(af, session)?[0], interner, session)?;
                drop(session.create_mutable_binding(env, name, false)?);
            }
            LexicallyScopedDeclaration::AsyncGeneratorDeclaration(ag) => {
                let name = scope_name(scope_bound_names(ag, session)?[0], interner, session)?;
                drop(session.create_mutable_binding(env, name, false)?);
            }
            LexicallyScopedDeclaration::ClassDeclaration(class) => {
                for name in scope_bound_names(class, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    drop(session.create_mutable_binding(env, name, false)?);
                }
            }
            LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Const(c)) => {
                for name in scope_bound_names(c, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    session.create_immutable_binding(env, name, true)?;
                }
            }
            LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Let(l)) => {
                for name in scope_bound_names(l, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    drop(session.create_mutable_binding(env, name, false)?);
                }
            }
            LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Using(u)) => {
                for name in scope_bound_names(u, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    drop(session.create_mutable_binding(env, name, false)?);
                }
            }
            LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::AwaitUsing(au)) => {
                for name in scope_bound_names(au, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    drop(session.create_mutable_binding(env, name, false)?);
                }
            }
            LexicallyScopedDeclaration::AssignmentExpression(expr) => {
                for name in scope_bound_names(expr, session)? {
                    session.admit(ScopeWork::Node)?;
                    let name = scope_name(name, interner, session)?;
                    drop(session.create_mutable_binding(env, name, false)?);
                }
            }
        }
    }
    Ok(())
}

/// This struct isused to store bindings created during the declaration of an eval ast node.
#[derive(Debug, Default)]
pub struct EvalDeclarationBindings {
    /// New annexb function names created during the declaration of an eval ast node.
    pub new_annex_b_function_names: Vec<IdentifierReference>,

    /// New function names created during the declaration of an eval ast node.
    pub new_function_names: FxHashMap<Identifier, IdentifierReference>,

    /// Variable names declared during the declaration of an eval ast node.
    pub declared_var_names: Vec<IdentifierReference>,
}

/// `EvalDeclarationInstantiation ( body, varEnv, lexEnv, privateEnv, strict )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-evaldeclarationinstantiation
///
/// # Errors
///
/// * Returns a syntax error if a duplicate lexical declaration is found.
/// * Returns a syntax error if a variable declaration in an eval function already exists as a lexical variable.
#[allow(clippy::missing_panics_doc)]
pub(crate) fn eval_declaration_instantiation_scope(
    body: &Script,
    strict: bool,
    var_env: &Scope,
    lex_env: &Scope,
    #[allow(unused_variables)] annex_b_function_names: &[Sym],
    interner: &Interner,
) -> Result<EvalDeclarationBindings, String> {
    ScopeAccess::<Infallible>::with_live(lex_env, None, |session| {
        let variable = session.associate(&var_env.handle())?;
        let lexical = session.root_handle()?;
        eval_declaration_instantiation_in_session(
            body,
            strict,
            &variable,
            &lexical,
            annex_b_function_names,
            interner,
            session,
        )
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn eval_declaration_instantiation_in_session<E>(
    body: &Script,
    strict: bool,
    var_env: &ScopeHandle,
    lex_env: &ScopeHandle,
    #[allow(unused_variables)] annex_b_function_names: &[Sym],
    interner: &Interner,
    session: &mut ScopeAccess<'_, E>,
) -> Result<EvalDeclarationBindings, ScopeAnalysisError<E>> {
    let mut result = EvalDeclarationBindings::default();

    // 2. Let varDeclarations be the VarScopedDeclarations of body.
    let var_declarations = scope_var_scoped_declarations(body, session)?;

    // 3. If strict is false, then
    if !strict {
        // 1. Let varNames be the VarDeclaredNames of body.
        let var_names = scope_var_declared_names(body, session)?;

        // a. If varEnv is a Global Environment Record, then
        if session.is_global(var_env)? {
            // i. For each element name of varNames, do
            for name in &var_names {
                let name = scope_name(name, interner, session)?;

                // 1. If varEnv.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
                // 2. NOTE: eval will not create a global var declaration that would be shadowed by a global lexical declaration.
                if session.has_lex_binding(var_env, &name)? {
                    return Err(ScopeAnalysisError::Syntax(
                        format!(
                            "duplicate lexical declaration {}",
                            name.to_std_string_escaped()
                        )
                        .into(),
                    ));
                }
            }
        }

        // b. Let thisEnv be lexEnv.
        let mut this_env = lex_env.clone();

        // c. Assert: The following loop will terminate.
        // d. Repeat, while thisEnv is not varEnv,
        while session.scope_index(&this_env)? != session.scope_index(var_env)? {
            // i. If thisEnv is not an Object Environment Record, then
            // 1. NOTE: The environment of with statements cannot contain any lexical
            //    declaration so it doesn't need to be checked for var/let hoisting conflicts.
            // 2. For each element name of varNames, do
            for name in &var_names {
                let name = scope_name(name, interner, session)?;

                // a. If ! thisEnv.HasBinding(name) is true, then
                if session.has_binding(&this_env, &name)? {
                    // i. Throw a SyntaxError exception.
                    // ii. NOTE: Annex B.3.4 defines alternate semantics for the above step.
                    return Err(ScopeAnalysisError::Syntax(format!(
                        "variable declaration {} in eval function already exists as a lexical variable",
                        name.to_std_string_escaped()
                    ).into()));
                }
                // b. NOTE: A direct eval will not hoist var declaration over a like-named lexical declaration.
            }

            // ii. Set thisEnv to thisEnv.[[OuterEnv]].
            if let Some(outer) = session.outer(&this_env)? {
                this_env = outer;
            } else {
                break;
            }
        }
    }

    // NOTE: These steps depend on the current environment state are done before bytecode compilation,
    //       in `eval_declaration_instantiation_context`.
    //
    // SKIP: 4. Let privateIdentifiers be a new empty List.
    // SKIP: 5. Let pointer be privateEnv.
    // SKIP: 6. Repeat, while pointer is not null,
    //           a. For each Private Name binding of pointer.[[Names]], do
    //               i. If privateIdentifiers does not contain binding.[[Description]],
    //                  append binding.[[Description]] to privateIdentifiers.
    //           b. Set pointer to pointer.[[OuterPrivateEnvironment]].
    // SKIP: 7. If AllPrivateIdentifiersValid of body with argument privateIdentifiers is false, throw a SyntaxError exception.

    // 8. Let functionsToInitialize be a new empty List.
    let mut functions_to_initialize = Vec::new();

    // 9. Let declaredFunctionNames be a new empty List.
    let mut declared_function_names = Vec::new();

    // 10. For each element d of varDeclarations, in reverse List order, do
    for declaration in var_declarations.iter().rev() {
        // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
        // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
        // a.iii. Let fn be the sole element of the BoundNames of d.
        let name = match *declaration {
            VarScopedDeclaration::FunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::GeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncFunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncGeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::VariableDeclaration(_) => continue,
        };
        // a.iv. If declaredFunctionNames does not contain fn, then
        if !scope_includes(&declared_function_names, &name.sym(), session)? {
            // 1. If varEnv is a Global Environment Record, then
            // 2. Append fn to declaredFunctionNames.
            scope_push(&mut declared_function_names, name.sym(), session)?;

            // 3. Insert d as the first element of functionsToInitialize.
            scope_push(&mut functions_to_initialize, *declaration, session)?;
        }
    }

    for _ in 0..functions_to_initialize.len() / 2 {
        session.admit(ScopeWork::Record)?;
    }
    functions_to_initialize.reverse();

    // 11. NOTE: Annex B.3.2.3 adds additional steps at this point.
    // 11. If strict is false, then
    #[cfg(feature = "annex-b")]
    if !strict {
        // NOTE: This diviates from the specification, we split the first part of defining the annex-b names
        //       in `eval_declaration_instantiation_context`, because it depends on the context.
        if !session.is_global(var_env)? {
            for name in annex_b_function_names {
                let f = scope_name(name, interner, session)?;
                // i. Let bindingExists be ! varEnv.HasBinding(F).
                // ii. If bindingExists is false, then
                if !session.has_binding(var_env, &f)? {
                    // i. Perform ! varEnv.CreateMutableBinding(F, true).
                    // ii. Perform ! varEnv.InitializeBinding(F, undefined).
                    let binding = session.create_mutable_binding(var_env, f, true)?;
                    let reference =
                        IdentifierReference::new(binding, !session.is_function(var_env)?, true);
                    scope_push(&mut result.new_annex_b_function_names, reference, session)?;
                }
            }
        }
    }

    // 12. Let declaredVarNames be a new empty List.
    let mut declared_var_names = Vec::new();

    // 13. For each element d of varDeclarations, do
    for declaration in var_declarations {
        // a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
            continue;
        };

        // a.i. For each String vn of the BoundNames of d, do
        for name in scope_bound_names(declaration, session)? {
            // 1. If declaredFunctionNames does not contain vn, then
            if !scope_includes(&declared_function_names, &name, session)? {
                // a. If varEnv is a Global Environment Record, then
                // b. If declaredVarNames does not contain vn, then
                if !scope_includes(&declared_var_names, &name, session)? {
                    // i. Append vn to declaredVarNames.
                    scope_push(&mut declared_var_names, name, session)?;
                }
            }
        }
    }

    // 14. NOTE: No abnormal terminations occur after this algorithm step unless varEnv is a
    //           Global Environment Record and the global object is a Proxy exotic object.

    // 15. Let lexDeclarations be the LexicallyScopedDeclarations of body.
    // 16. For each element d of lexDeclarations, do
    for statement in &**body.statements() {
        session.admit(ScopeWork::Node)?;
        // a. NOTE: Lexically declared names are only instantiated here but not initialized.
        // b. For each element dn of the BoundNames of d, do
        //     i. If IsConstantDeclaration of d is true, then
        //         1. Perform ? lexEnv.CreateImmutableBinding(dn, true).
        //     ii. Else,
        //         1. Perform ? lexEnv.CreateMutableBinding(dn, false).
        if let StatementListItem::Declaration(declaration) = statement {
            match declaration.as_ref() {
                Declaration::ClassDeclaration(class) => {
                    for name in scope_bound_names(class.as_ref(), session)? {
                        let name = scope_name(name, interner, session)?;
                        drop(session.create_mutable_binding(lex_env, name, false)?);
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        let name = scope_name(name, interner, session)?;
                        drop(session.create_mutable_binding(lex_env, name, false)?);
                    }
                }
                Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                    for name in scope_bound_names(declaration, session)? {
                        let name = scope_name(name, interner, session)?;
                        session.create_immutable_binding(lex_env, name, true)?;
                    }
                }
                _ => {}
            }
        }
    }

    // 17. For each Parse Node f of functionsToInitialize, do
    for function in functions_to_initialize {
        // a. Let fn be the sole element of the BoundNames of f.
        let name = match function {
            VarScopedDeclaration::FunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::GeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncFunctionDeclaration(f) => f.name(),
            VarScopedDeclaration::AsyncGeneratorDeclaration(f) => f.name(),
            VarScopedDeclaration::VariableDeclaration(_) => {
                continue;
            }
        };

        // c. If varEnv is a Global Environment Record, then
        // d. Else,
        if !session.is_global(var_env)? {
            // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
            let n = scope_name(name, interner, session)?;

            // i. Let bindingExists be ! varEnv.HasBinding(fn).
            let binding_exists = session.has_binding(var_env, &n)?;

            // ii. If bindingExists is false, then
            // iii. Else,
            if binding_exists {
                // 1. Perform ! varEnv.SetMutableBinding(fn, fo, false).
                let binding = session
                    .set_mutable_binding(var_env, n)?
                    .expect("validated mutable eval function binding");
                session.admit(ScopeWork::Record)?;
                session.admit(ScopeWork::Capacity {
                    elements: result.new_function_names.len(),
                })?;
                result
                    .new_function_names
                    .try_reserve(1)
                    .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
                result.new_function_names.insert(
                    name,
                    IdentifierReference::new(
                        binding.locator(),
                        !session.is_function(var_env)?,
                        true,
                    ),
                );
            } else {
                // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                // 2. Perform ! varEnv.CreateMutableBinding(fn, true).
                // 3. Perform ! varEnv.InitializeBinding(fn, fo).
                let binding = session.create_mutable_binding(var_env, n, !strict)?;
                session.admit(ScopeWork::Record)?;
                session.admit(ScopeWork::Capacity {
                    elements: result.new_function_names.len(),
                })?;
                result
                    .new_function_names
                    .try_reserve(1)
                    .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
                result.new_function_names.insert(
                    name,
                    IdentifierReference::new(binding, !session.is_function(var_env)?, true),
                );
            }
        }
    }

    // 18. For each String vn of declaredVarNames, do
    for name in declared_var_names {
        // a. If varEnv is a Global Environment Record, then
        // b. Else,
        if !session.is_global(var_env)? {
            let name = scope_name(name, interner, session)?;

            // i. Let bindingExists be ! varEnv.HasBinding(vn).
            let binding_exists = session.has_binding(var_env, &name)?;

            // ii. If bindingExists is false, then
            if !binding_exists {
                // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                // 2. Perform ! varEnv.CreateMutableBinding(vn, true).
                // 3. Perform ! varEnv.InitializeBinding(vn, undefined).
                drop(session.create_mutable_binding(var_env, name.clone(), true)?);
            }

            let binding = session
                .get_binding_reference(var_env, &name)?
                .expect("created eval variable binding");
            let reference =
                IdentifierReference::new(binding.locator(), !session.is_function(var_env)?, true);
            scope_push(&mut result.declared_var_names, reference, session)?;
        }
    }

    // 19. Return unused.
    Ok(result)
}
