//! Clears analysis annotations while retaining the authored tree.

use std::ops::ControlFlow;

use crate::{
    Module,
    expression::literal::ObjectMethodDefinition,
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassDeclaration, ClassElement,
        ClassExpression, FunctionDeclaration, FunctionExpression, GeneratorDeclaration,
        GeneratorExpression,
    },
    scope::{ScopeAccess, ScopeAnalysisError, ScopeWork},
    statement::{Block, Catch, ForInLoop, ForOfLoop, Switch, With, iteration::ForLoopInitializer},
    visitor::{NodeRefMut, VisitWith, VisitorMut},
};

/// Resets annotations before a fresh analysis of the retained syntax.
/// An interrupted traversal leaves syntax intact and may be restarted at its root.
pub(crate) fn reset_scopes<'a, N, E>(
    node: &'a mut N,
    access: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>>
where
    &'a mut N: Into<NodeRefMut<'a>>,
{
    let mut visitor = ResetScopes { access };
    match visitor.visit(node.into()) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}

struct ResetScopes<'access, 'store, E> {
    access: &'access mut ScopeAccess<'store, E>,
}

macro_rules! reset_node {
    ($method:ident, $node:ty, $($field:ident),+ $(,)?) => {
        fn $method(&mut self, node: &'ast mut $node) -> ControlFlow<Self::BreakTy> {
            self.admit_visit()?;
            $(node.$field = Default::default();)+
            node.visit_with_mut(self)
        }
    };
}

impl<'ast, E> VisitorMut<'ast> for ResetScopes<'_, '_, E> {
    type BreakTy = ScopeAnalysisError<E>;

    fn admit_visit(&mut self) -> ControlFlow<Self::BreakTy> {
        match self.access.admit(ScopeWork::Node) {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    reset_node!(visit_module_mut, Module, scope);
    reset_node!(visit_block_mut, Block, scope);
    reset_node!(visit_switch_mut, Switch, scope);
    reset_node!(visit_with_mut, With, scope);
    reset_node!(visit_catch_mut, Catch, scope);
    reset_node!(visit_for_in_loop_mut, ForInLoop, target_scope, scope);
    reset_node!(visit_for_of_loop_mut, ForOfLoop, iterable_scope, scope);
    reset_node!(visit_function_declaration_mut, FunctionDeclaration, scopes);
    reset_node!(
        visit_generator_declaration_mut,
        GeneratorDeclaration,
        scopes
    );
    reset_node!(
        visit_async_function_declaration_mut,
        AsyncFunctionDeclaration,
        scopes
    );
    reset_node!(
        visit_async_generator_declaration_mut,
        AsyncGeneratorDeclaration,
        scopes
    );
    reset_node!(
        visit_function_expression_mut,
        FunctionExpression,
        name_scope,
        scopes
    );
    reset_node!(
        visit_generator_expression_mut,
        GeneratorExpression,
        name_scope,
        scopes
    );
    reset_node!(
        visit_async_function_expression_mut,
        AsyncFunctionExpression,
        name_scope,
        scopes
    );
    reset_node!(
        visit_async_generator_expression_mut,
        AsyncGeneratorExpression,
        name_scope,
        scopes
    );
    reset_node!(visit_arrow_function_mut, ArrowFunction, scopes);
    reset_node!(visit_async_arrow_function_mut, AsyncArrowFunction, scopes);
    reset_node!(
        visit_object_method_definition_mut,
        ObjectMethodDefinition,
        scopes
    );
    reset_node!(visit_class_declaration_mut, ClassDeclaration, name_scope);
    reset_node!(visit_class_expression_mut, ClassExpression, name_scope);

    fn visit_for_loop_initializer_mut(
        &mut self,
        node: &'ast mut ForLoopInitializer,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        if let ForLoopInitializer::Lexical(initializer) = node {
            initializer.scope = Default::default();
        }
        node.visit_with_mut(self)
    }

    fn visit_class_element_mut(
        &mut self,
        node: &'ast mut ClassElement,
    ) -> ControlFlow<Self::BreakTy> {
        self.admit_visit()?;
        match node {
            ClassElement::MethodDefinition(method) => method.scopes = Default::default(),
            ClassElement::FieldDefinition(field) | ClassElement::StaticFieldDefinition(field) => {
                field.scope = Default::default();
            }
            ClassElement::PrivateFieldDefinition(field)
            | ClassElement::PrivateStaticFieldDefinition(field) => {
                field.scope = Default::default();
            }
            ClassElement::StaticBlock(block) => block.scopes = Default::default(),
        }
        node.visit_with_mut(self)
    }
}
