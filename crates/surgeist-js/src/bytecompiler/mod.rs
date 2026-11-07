//! Boa's bytecode compiler.
//!
//! The bytecompiler is responsible for lowering ECMAScript AST nodes into
//! executable bytecode for the virtual machine.
//!
//! It walks the parsed AST and emits instructions using the bytecode emitter,
//! producing code blocks that are later executed by the VM.
//!
//! This module provides the necessary functionality to compile JavaScript code into bytecode.
//! Conceptually, it is the bridge between the spec's runtime semantics and Boa's opcode-based VM.
//! When the ECMAScript algorithms use the `!` prefix for an abstract operation, Boa treats the
//! corresponding assumption as an internal compiler or bytecode invariant, and documents that
//! mapping near the assertion sites that rely on it.
//!
//! Relevant specification entry points include [script evaluation][script],
//! [module evaluation][module], and the expression and statement [runtime semantics][evaluation]
//! that eventually feed into those top-level algorithms.
//!
//! [script]: https://tc39.es/ecma262/#sec-runtime-semantics-scriptevaluation
//! [module]: https://tc39.es/ecma262/#sec-moduleevaluation
//! [evaluation]: https://tc39.es/ecma262/#sec-runtime-semantics-evaluation

mod class;
pub(crate) mod control;
mod declaration;
mod declarations;
#[cfg(test)]
mod entry_tests;
mod env;
mod expression;
mod function;
mod generator;
mod index;
mod jump_control;
mod keys;
mod mapped_arguments;
mod module;
mod register;
mod scope;
mod statement;
mod utils;

use std::{
    borrow::{Borrow, BorrowMut},
    cell::Cell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    JsBigInt, JsResult, JsStr, JsString, SourceText, SpannedSourceText,
    builtins::function::ThisMode,
    js_string,
    vm::{
        CallFrame, CodeBlock, CodeBlockFlags, Constant, GeneratorResumeKind, GlobalFunctionBinding,
        Handler, InlineCache,
        opcode::{Address, BindingOpcode, BytecodeEmitter, RegisterOperand},
        source_info::{SourceInfo, SourceMap, SourceMapBuilder, SourcePath},
    },
};
use surgeist_js_ast::{
    Declaration, Expression, LinearSpan, Position, Spanned, Statement, StatementList,
    StatementListItem,
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::{
        Call, Identifier, New, Optional, OptionalOperationKind,
        access::{PropertyAccess, PropertyAccessField},
        literal::ObjectMethodDefinition,
        operator::{
            Binary,
            assign::AssignTarget,
            binary::{BinaryOp, RelationalOp},
            update::UpdateTarget,
        },
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassMethodDefinition,
        FormalParameterList, FunctionBody, FunctionDeclaration, FunctionExpression,
        GeneratorDeclaration, GeneratorExpression, PrivateName,
    },
    operations::try_returns_value,
    pattern::Pattern,
    property::MethodDefinitionKind,
    scope::{
        BindingLocator, BindingLocatorError, FunctionScopes, IdentifierReference, ScopeExtension,
        ScopeHandle,
    },
};
use surgeist_js_interner::{Interner, Sym};
use surgeist_js_macros::js_str;
use surgeist_js_string::StaticJsStrings;
use thin_vec::ThinVec;

use control::{CompilerControl, CompilerWork, payload_bytes, reserve, size_overflow};
pub(crate) use function::FunctionCompiler;
use index::CompilerIndex;
pub(crate) use jump_control::JumpControlInfo;
pub(crate) use register::*;

pub(crate) trait ToJsString {
    fn to_js_string(&self, interner: &Interner) -> JsString;

    fn try_to_js_string(
        &self,
        interner: &Interner,
        control: &mut control::CompilerControl<'_>,
    ) -> JsResult<JsString>;
}

impl ToJsString for Sym {
    fn try_to_js_string(
        &self,
        interner: &Interner,
        control: &mut control::CompilerControl<'_>,
    ) -> JsResult<JsString> {
        let units = interner.resolve_expect(*self).utf16();
        JsString::try_from_utf16(units, &mut |work| {
            control.admit(control::CompilerWork::String(work))
        })
        .map_err(control::string_error)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn to_js_string(&self, interner: &Interner) -> JsString {
        let utf16 = interner.resolve_expect(*self).utf16();
        if interner.is_latin1(*self) {
            let bytes: Vec<u8> = utf16.iter().map(|&c| c as u8).collect();
            js_string!(JsStr::latin1(&bytes))
        } else {
            js_string!(utf16)
        }
    }
}

impl ToJsString for Identifier {
    fn to_js_string(&self, interner: &Interner) -> JsString {
        self.sym().to_js_string(interner)
    }

    fn try_to_js_string(
        &self,
        interner: &Interner,
        control: &mut control::CompilerControl<'_>,
    ) -> JsResult<JsString> {
        self.sym().try_to_js_string(interner, control)
    }
}

/// Describes how a node has been defined in the source code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodeKind {
    Declaration,
    Expression,
}

/// Describes the type of a function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FunctionKind {
    Ordinary,
    Arrow,
    AsyncArrow,
    Async,
    Generator,
    AsyncGenerator,
}

impl FunctionKind {
    pub(crate) const fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow | Self::AsyncArrow)
    }

    pub(crate) const fn is_async(self) -> bool {
        matches!(self, Self::Async | Self::AsyncGenerator | Self::AsyncArrow)
    }

    pub(crate) const fn is_generator(self) -> bool {
        matches!(self, Self::Generator | Self::AsyncGenerator)
    }
}

/// Describes the complete specification of a function node.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionSpec<'a> {
    pub(crate) kind: FunctionKind,
    pub(crate) name: Option<Identifier>,
    parameters: &'a FormalParameterList,
    body: &'a FunctionBody,
    pub(crate) scopes: &'a FunctionScopes,
    pub(crate) name_scope: Option<&'a ScopeHandle>,
    linear_span: Option<LinearSpan>,
    pub(crate) contains_direct_eval: bool,
}

impl PartialEq for FunctionSpec<'_> {
    fn eq(&self, other: &Self) -> bool {
        // all fields except `linear_span`
        self.kind == other.kind
            && self.name == other.name
            && self.parameters == other.parameters
            && self.body == other.body
            && self.scopes == other.scopes
            && self.name_scope == other.name_scope
    }
}

impl<'a> From<&'a FunctionDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a FunctionDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Ordinary,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a GeneratorDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a GeneratorDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Generator,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncFunctionDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a AsyncFunctionDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Async,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncGeneratorDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a AsyncGeneratorDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncGenerator,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a FunctionExpression> for FunctionSpec<'a> {
    fn from(function: &'a FunctionExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Ordinary,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: function.linear_span(),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ArrowFunction> for FunctionSpec<'a> {
    fn from(function: &'a ArrowFunction) -> Self {
        FunctionSpec {
            kind: FunctionKind::Arrow,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncArrowFunction> for FunctionSpec<'a> {
    fn from(function: &'a AsyncArrowFunction) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncArrow,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncFunctionExpression> for FunctionSpec<'a> {
    fn from(function: &'a AsyncFunctionExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Async,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a GeneratorExpression> for FunctionSpec<'a> {
    fn from(function: &'a GeneratorExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Generator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncGeneratorExpression> for FunctionSpec<'a> {
    fn from(function: &'a AsyncGeneratorExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncGenerator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ClassMethodDefinition> for FunctionSpec<'a> {
    fn from(method: &'a ClassMethodDefinition) -> Self {
        let kind = match method.kind() {
            MethodDefinitionKind::Generator => FunctionKind::Generator,
            MethodDefinitionKind::AsyncGenerator => FunctionKind::AsyncGenerator,
            MethodDefinitionKind::Async => FunctionKind::Async,
            _ => FunctionKind::Ordinary,
        };

        FunctionSpec {
            kind,
            name: None,
            parameters: method.parameters(),
            body: method.body(),
            scopes: method.scopes(),
            name_scope: None,
            linear_span: Some(method.linear_span()),
            contains_direct_eval: method.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ObjectMethodDefinition> for FunctionSpec<'a> {
    fn from(method: &'a ObjectMethodDefinition) -> Self {
        let kind = match method.kind() {
            MethodDefinitionKind::Generator => FunctionKind::Generator,
            MethodDefinitionKind::AsyncGenerator => FunctionKind::AsyncGenerator,
            MethodDefinitionKind::Async => FunctionKind::Async,
            _ => FunctionKind::Ordinary,
        };

        FunctionSpec {
            kind,
            name: method.name().literal(),
            parameters: method.parameters(),
            body: method.body(),
            scopes: method.scopes(),
            name_scope: None,
            linear_span: Some(method.linear_span()),
            contains_direct_eval: method.contains_direct_eval(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MethodKind {
    Get,
    Set,
    Ordinary,
}

/// Represents a callable expression, like `f()` or `new Cl()`
#[derive(Debug, Clone, Copy)]
enum Callable<'a> {
    Call(&'a Call),
    New(&'a New),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Literal {
    String(JsString),
    BigInt(JsBigInt),
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Label {
    index: Address,
}

/// A loop-invariant operand that was hoisted out of a loop condition.
///
/// When a loop condition like `i < 10` has a literal operand, we can compile
/// the constant (`10`) into a register once before the loop, rather than
/// re-emitting a `PushInt8`/`PushInt32` on every iteration.
pub(crate) struct HoistedOperand {
    /// The register holding the pre-compiled constant value.
    pub(crate) register: Register,
    /// `true` if this operand is the RHS of the comparison (e.g. `10` in `i < 10`).
    pub(crate) is_rhs: bool,
}

#[derive(Debug, Clone, Copy)]
#[allow(variant_size_differences)]
enum Access<'a> {
    Variable { name: Identifier },
    Property { access: &'a PropertyAccess },
    This,
}

impl Access<'_> {
    const fn from_assign_target(target: &AssignTarget) -> Result<Access<'_>, &Pattern> {
        match target {
            AssignTarget::Identifier(ident) => Ok(Access::Variable { name: *ident }),
            AssignTarget::Access(access) => Ok(Access::Property { access }),
            AssignTarget::Pattern(pat) => Err(pat),
        }
    }

    const fn from_update_target(target: &UpdateTarget) -> Access<'_> {
        match target {
            UpdateTarget::Identifier(name) => Access::Variable { name: *name },
            UpdateTarget::PropertyAccess(access) => Access::Property { access },
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum BindingAccessOpcode {
    PutLexicalValue,
    DefInitVar,
    SetName,
    SetNameByLocator,
    GetName,
    GetNameAndLocator,
    GetNameOrUndefined,
    DeleteName,
    GetLocator,
    DefVar,
    DefEvalVar,
}

/// Completes source metadata explicitly on success and abandons it on drop.
pub(crate) struct SourcePositionGuard<'a, 'b, 'store> {
    compiler: &'a mut ByteCompiler<'b, 'store>,
    finished: bool,
}
impl<'a, 'b, 'store> SourcePositionGuard<'a, 'b, 'store> {
    fn new(compiler: &'a mut ByteCompiler<'b, 'store>, position: Position) -> JsResult<Self> {
        compiler.push_source_position(position)?;
        Ok(Self {
            compiler,
            finished: false,
        })
    }

    /// Completes this scope's source ranges after successful compilation.
    pub(crate) fn finish(mut self) -> JsResult<()> {
        self.compiler.pop_source_position()?;
        self.finished = true;
        Ok(())
    }
}
impl Drop for SourcePositionGuard<'_, '_, '_> {
    fn drop(&mut self) {
        if !self.finished {
            self.compiler.source_map_builder.abort_source_position();
        }
    }
}
impl<'a, 'store> Deref for SourcePositionGuard<'_, 'a, 'store> {
    type Target = ByteCompiler<'a, 'store>;
    fn deref(&self) -> &Self::Target {
        self.compiler
    }
}
impl DerefMut for SourcePositionGuard<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.compiler
    }
}
impl<'a, 'store> Borrow<ByteCompiler<'a, 'store>> for SourcePositionGuard<'_, 'a, 'store> {
    fn borrow(&self) -> &ByteCompiler<'a, 'store> {
        self.compiler
    }
}
impl<'a, 'store> BorrowMut<ByteCompiler<'a, 'store>> for SourcePositionGuard<'_, 'a, 'store> {
    fn borrow_mut(&mut self) -> &mut ByteCompiler<'a, 'store> {
        self.compiler
    }
}

/// The [`ByteCompiler`] is used to compile ECMAScript AST from [`surgeist_js_ast`] to bytecode.
#[allow(clippy::struct_excessive_bools)]
pub struct ByteCompiler<'ctx, 'store> {
    /// Name of this function.
    pub(crate) function_name: JsString,

    /// The number of arguments expected.
    pub(crate) length: u32,

    pub(crate) register_allocator: RegisterAllocator,

    /// `[[ThisMode]]`
    pub(crate) this_mode: ThisMode,

    /// ScopeHandle of the function parameters.
    pub(crate) parameter_scope: ScopeHandle,

    /// Bytecode
    pub(crate) bytecode: BytecodeEmitter,

    pub(crate) source_map_builder: SourceMapBuilder,
    pub(crate) source_path: SourcePath,

    pub(crate) constants: ThinVec<Constant>,

    /// Locators for all bindings in the codeblock.
    pub(crate) bindings: Vec<BindingLocator>,

    local_binding_registers: CompilerIndex<IdentifierReference, u32>,

    /// The current variable scope.
    pub(crate) variable_scope: ScopeHandle,

    /// The current lexical scope.
    pub(crate) lexical_scope: ScopeHandle,

    pub(crate) current_open_environments_count: u32,
    code_block_flags: CodeBlockFlags,
    handlers: ThinVec<Handler>,
    pub(crate) ic: Vec<InlineCache>,
    literals_map: CompilerIndex<Literal, u32>,
    names_map: CompilerIndex<Sym, u32>,
    bindings_map: CompilerIndex<BindingLocator, u32>,

    /// Cache of non-local `const` binding values in persistent registers.
    /// Avoids repeated `GetName` environment lookups for immutable bindings.
    const_binding_cache: CompilerIndex<BindingLocator, u32>,

    jump_info: Vec<JumpControlInfo>,

    /// Used to handle exception throws that escape the async function types.
    ///
    /// Async functions and async generator functions, need to be closed and resolved.
    pub(crate) async_handler: Option<u32>,
    json_parse: bool,

    /// Whether the function is in a `with` statement.
    pub(crate) in_with: bool,

    /// Used to determine if a we emitted a `CreateUnmappedArgumentsObject` opcode
    pub(crate) emitted_mapped_arguments_object_opcode: bool,

    pub(crate) interner: &'ctx mut Interner,
    pub(crate) scopes: &'ctx mut ScopeExtension<'store>,
    pub(crate) control: CompilerControl<'ctx>,
    spanned_source_text: SpannedSourceText,

    pub(crate) global_fns: Vec<GlobalFunctionBinding>,
    pub(crate) global_vars: Vec<u32>,

    #[cfg(feature = "annex-b")]
    pub(crate) annex_b_function_names: Vec<Sym>,
    #[cfg(feature = "annex-b")]
    script_global_annex: bool,
    annex_b_candidates: Vec<JsString>,
}

impl std::fmt::Debug for ByteCompiler<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ByteCompiler")
            .field("function_name", &self.function_name)
            .field("flags", &self.code_block_flags)
            .field("control", &self.control)
            .finish_non_exhaustive()
    }
}

pub(crate) enum BindingKind {
    Stack(u32),
    Local(Option<u32>),
    Global(u32),
}

/// Where the call result should go.
#[derive(Copy, Clone)]
enum CallResultDest<'a> {
    /// Pop result into the given register.
    Register(&'a Register),
    /// Discard the result (emit Pop).
    Discard,
    /// Leave the result on the stack.
    Stack,
}

impl<'ctx, 'store> ByteCompiler<'ctx, 'store> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: Address = Address::new(u32::MAX);
    const DUMMY_LABEL: Label = Label {
        index: Address::new(u32::MAX),
    };

    /// Creates a new [`ByteCompiler`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    pub(crate) fn new(
        name: JsString,
        strict: bool,
        json_parse: bool,
        variable_scope: ScopeHandle,
        lexical_scope: ScopeHandle,
        is_async: bool,
        is_generator: bool,
        interner: &'ctx mut Interner,
        in_with: bool,
        spanned_source_text: SpannedSourceText,
        source_path: SourcePath,
        scopes: &'ctx mut ScopeExtension<'store>,
        mut control: CompilerControl<'ctx>,
    ) -> JsResult<ByteCompiler<'ctx, 'store>> {
        control.admit(CompilerWork::MetadataWrite)?;
        let mut code_block_flags = CodeBlockFlags::empty();
        code_block_flags.set(CodeBlockFlags::STRICT, strict);
        code_block_flags.set(CodeBlockFlags::IS_ASYNC, is_async);
        code_block_flags.set(CodeBlockFlags::IS_GENERATOR, is_generator);
        code_block_flags |= CodeBlockFlags::HAS_PROTOTYPE_PROPERTY;

        let mut register_allocator = RegisterAllocator::default();
        let undefined_register = register_allocator.alloc_persistent(&mut control)?;
        debug_assert_eq!(
            undefined_register.index(),
            CallFrame::undefined_register().index()
        );
        if is_async {
            let promise_register = register_allocator.alloc_persistent(&mut control)?;
            let resolve_register = register_allocator.alloc_persistent(&mut control)?;
            let reject_register = register_allocator.alloc_persistent(&mut control)?;

            debug_assert_eq!(
                promise_register.index(),
                CallFrame::promise_capability_promise_register().index()
            );
            debug_assert_eq!(
                resolve_register.index(),
                CallFrame::promise_capability_resolve_register().index()
            );
            debug_assert_eq!(
                reject_register.index(),
                CallFrame::promise_capability_reject_register().index()
            );

            if is_generator {
                let async_function_object_register =
                    register_allocator.alloc_persistent(&mut control)?;
                debug_assert_eq!(
                    async_function_object_register.index(),
                    CallFrame::async_generator_object_register().index()
                );
            }
        }

        Ok(Self {
            function_name: name,
            length: 0,
            bytecode: BytecodeEmitter::new(),
            source_map_builder: SourceMapBuilder::default(),
            constants: ThinVec::default(),
            bindings: Vec::default(),
            local_binding_registers: CompilerIndex::default(),
            this_mode: ThisMode::Global,
            parameter_scope: variable_scope.clone(),
            current_open_environments_count: 0,

            register_allocator,
            code_block_flags,
            handlers: ThinVec::default(),
            ic: Vec::default(),

            literals_map: CompilerIndex::default(),
            names_map: CompilerIndex::default(),
            bindings_map: CompilerIndex::default(),
            const_binding_cache: CompilerIndex::default(),
            jump_info: Vec::new(),
            async_handler: None,
            json_parse,
            variable_scope,
            lexical_scope,
            interner,
            scopes,
            control,
            spanned_source_text,
            source_path,

            #[cfg(feature = "annex-b")]
            annex_b_function_names: Vec::new(),
            #[cfg(feature = "annex-b")]
            script_global_annex: false,
            annex_b_candidates: Vec::new(),
            in_with,
            emitted_mapped_arguments_object_opcode: false,

            global_fns: Vec::new(),
            global_vars: Vec::new(),
        })
    }

    pub(crate) fn source_text(&self) -> SourceText {
        self.spanned_source_text.source_text()
    }

    pub(crate) const fn strict(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::STRICT)
    }

    pub(crate) const fn is_async(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::IS_ASYNC)
    }

    pub(crate) const fn is_generator(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::IS_GENERATOR)
    }

    pub(crate) const fn is_async_generator(&self) -> bool {
        self.is_async() && self.is_generator()
    }

    fn flatten_expression<'ast>(
        &mut self,
        mut expr: &'ast Expression,
    ) -> JsResult<&'ast Expression> {
        loop {
            self.control.admit(CompilerWork::Search)?;
            match expr {
                Expression::Parenthesized(parenthesized) => expr = parenthesized.expression(),
                _ => return Ok(expr),
            }
        }
    }

    /// Admit wrapper traversal before the AST owner's constant-size leaf query.
    fn is_anonymous_function_definition(&mut self, expression: &Expression) -> JsResult<bool> {
        let expression = self.flatten_expression(expression)?;
        self.control.admit(CompilerWork::Search)?;
        Ok(expression.is_anonymous_function_definition())
    }

    fn access_expression<'ast>(
        &mut self,
        expr: &'ast Expression,
    ) -> JsResult<Option<Access<'ast>>> {
        Ok(match self.flatten_expression(expr)? {
            Expression::Identifier(name) => Some(Access::Variable { name: *name }),
            Expression::PropertyAccess(access) => Some(Access::Property { access }),
            Expression::This(_) => Some(Access::This),
            _ => None,
        })
    }

    fn arguments_contain_spread(&mut self, args: &[Expression]) -> JsResult<bool> {
        for arg in args {
            self.control.admit(CompilerWork::Search)?;
            if matches!(arg, Expression::Spread(_)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn push_inline_cache(&mut self, name: JsString) -> JsResult<u32> {
        let index = u32::try_from(self.ic.len()).map_err(|_| size_overflow())?;
        reserve(&mut self.ic, 1, &mut self.control)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.ic.push(InlineCache::new(name));
        Ok(index)
    }

    fn get_or_insert_literal(&mut self, literal: Literal) -> JsResult<u32> {
        if let Some(index) = self.literals_map.get(&literal, &mut self.control)? {
            return Ok(*index);
        }
        // Retain another descriptor, not another copy of the string/magnitude.
        self.control.admit(CompilerWork::MetadataWrite)?;
        let value = match literal.clone() {
            Literal::String(value) => Constant::String(value),
            Literal::BigInt(value) => Constant::BigInt(value),
        };
        let index = u32::try_from(self.constants.len()).map_err(|_| size_overflow())?;
        control::thin_push(&mut self.constants, value, &mut self.control)?;
        self.literals_map
            .insert(literal, index, &mut self.control)?;
        Ok(index)
    }

    fn get_or_insert_name(&mut self, name: Sym) -> JsResult<u32> {
        if let Some(index) = self.names_map.get(&name, &mut self.control)? {
            return Ok(*index);
        }
        let index = u32::try_from(self.constants.len()).map_err(|_| size_overflow())?;
        let string = name.try_to_js_string(&*self.interner, &mut self.control)?;
        control::thin_push(
            &mut self.constants,
            Constant::String(string),
            &mut self.control,
        )?;
        self.names_map.insert(name, index, &mut self.control)?;
        Ok(index)
    }

    fn get_or_insert_string(&mut self, value: JsString) -> JsResult<u32> {
        self.get_or_insert_literal(Literal::String(value))
    }

    #[inline]
    fn get_or_insert_private_name(&mut self, name: PrivateName) -> JsResult<u32> {
        self.get_or_insert_name(name.description())
    }

    // TODO: Make this return `Option<BindingKind>` instead of making BindingKind::Local
    //       inner field optional.
    #[inline]
    pub(crate) fn get_binding(&mut self, binding: &IdentifierReference) -> JsResult<BindingKind> {
        if binding.local() {
            return Ok(BindingKind::Local(
                self.local_binding_registers
                    .get(binding, &mut self.control)?
                    .copied(),
            ));
        }
        self.control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<BindingLocator>(),
        })?;
        let index = self.indexed_locator(binding.locator())?;
        Ok(if binding.is_global_object() {
            BindingKind::Global(index)
        } else {
            BindingKind::Stack(index)
        })
    }

    #[inline]
    pub(crate) fn insert_binding(&mut self, binding: IdentifierReference) -> JsResult<BindingKind> {
        if binding.local() {
            if let Some(index) = self
                .local_binding_registers
                .get(&binding, &mut self.control)?
            {
                return Ok(BindingKind::Local(Some(*index)));
            }
            let index = self
                .register_allocator
                .alloc_persistent(&mut self.control)?
                .index();
            self.local_binding_registers
                .insert(binding, index, &mut self.control)?;
            return Ok(BindingKind::Local(Some(index)));
        }
        self.control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<BindingLocator>(),
        })?;
        let index = self.indexed_locator(binding.locator())?;
        Ok(if binding.is_global_object() {
            BindingKind::Global(index)
        } else {
            BindingKind::Stack(index)
        })
    }

    fn indexed_locator(&mut self, locator: BindingLocator) -> JsResult<u32> {
        if let Some(index) = self.bindings_map.get(&locator, &mut self.control)? {
            return Ok(*index);
        }
        let index = u32::try_from(self.bindings.len()).map_err(|_| size_overflow())?;
        reserve(&mut self.bindings, 1, &mut self.control)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<BindingLocator>(),
        })?;
        self.bindings.push(locator.clone());
        self.bindings_map
            .insert(locator, index, &mut self.control)?;
        Ok(index)
    }

    #[inline]
    pub(crate) fn push_function_to_constants(&mut self, function: Rc<CodeBlock>) -> JsResult<u32> {
        let index = u32::try_from(self.constants.len()).map_err(|_| size_overflow())?;
        control::thin_push(
            &mut self.constants,
            Constant::Function(function),
            &mut self.control,
        )?;
        Ok(index)
    }

    fn emit_binding(
        &mut self,
        opcode: BindingOpcode,
        name: JsString,
        value: &Register,
    ) -> JsResult<()> {
        match opcode {
            BindingOpcode::Var => {
                let binding = self.variable_identifier_reference(name)?;
                if !binding.locator().is_global() {
                    let index = self.insert_binding(binding)?;
                    self.emit_binding_access(BindingAccessOpcode::DefVar, &index, value)?;
                }
            }
            BindingOpcode::InitVar => match self.lexical_set_mutable_binding(name.clone())? {
                Ok(binding) => {
                    let index = self.insert_binding(binding)?;
                    self.emit_binding_access(BindingAccessOpcode::DefInitVar, &index, value)?;
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name)?;
                    self.bytecode
                        .emit_throw_mutate_immutable(index.into(), &mut self.control)?;
                }
                Err(BindingLocatorError::Silent) => {}
            },
            BindingOpcode::InitLexical => {
                let binding = self.lexical_identifier_reference(name)?;
                let index = self.insert_binding(binding)?;
                self.emit_binding_access(BindingAccessOpcode::PutLexicalValue, &index, value)?;
            }
            BindingOpcode::SetName => match self.lexical_set_mutable_binding(name.clone())? {
                Ok(binding) => {
                    let index = self.insert_binding(binding)?;
                    self.emit_binding_access(BindingAccessOpcode::SetName, &index, value)?;
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name)?;
                    self.bytecode
                        .emit_throw_mutate_immutable(index.into(), &mut self.control)?;
                }
                Err(BindingLocatorError::Silent) => {}
            },
        }
        Ok(())
    }

    fn next_opcode_location(&mut self) -> JsResult<Address> {
        self.bytecode.next_opcode_location(&mut self.control)
    }

    pub(crate) fn position_guard(
        &mut self,
        spanned: impl Spanned,
    ) -> JsResult<SourcePositionGuard<'_, 'ctx, 'store>> {
        SourcePositionGuard::new(self, spanned.span().start())
    }

    pub(crate) fn push_source_position<T>(&mut self, position: T) -> JsResult<()>
    where
        T: Into<Option<Position>>,
    {
        let start_pc = self.next_opcode_location()?;
        self.source_map_builder.push_source_position(
            start_pc.as_u32(),
            position.into(),
            &mut self.control,
        )
    }

    pub(crate) fn pop_source_position(&mut self) -> JsResult<()> {
        let start_pc = self.next_opcode_location()?;
        self.source_map_builder
            .pop_source_position(start_pc.as_u32(), &mut self.control)
    }

    pub(crate) fn emit_get_function(&mut self, dst: &Register, index: u32) -> JsResult<()> {
        self.bytecode
            .emit_get_function(dst.variable(), index.into(), &mut self.control)?;
        Ok(())
    }

    fn pop_into_register(&mut self, dst: &Register) -> JsResult<()> {
        self.bytecode
            .emit_pop_into_register(dst.variable(), &mut self.control)?;
        Ok(())
    }

    pub(crate) fn push_from_register(&mut self, src: &Register) -> JsResult<()> {
        self.bytecode
            .emit_push_from_register(src.variable(), &mut self.control)?;
        Ok(())
    }

    pub(crate) fn emit_binding_access(
        &mut self,
        opcode: BindingAccessOpcode,
        binding: &BindingKind,
        value: &Register,
    ) -> JsResult<()> {
        match binding {
            BindingKind::Global(index) => match opcode {
                BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode
                        .emit_set_name_by_locator(value.variable(), &mut self.control)?;
                }
                BindingAccessOpcode::GetName => {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    let name = self.bindings[*index as usize].name().clone();
                    let ic_index = self.push_inline_cache(name)?;
                    self.bytecode.emit_get_name_global(
                        value.variable(),
                        (*index).into(),
                        ic_index.into(),
                        &mut self.control,
                    )?;
                }
                BindingAccessOpcode::GetLocator => self
                    .bytecode
                    .emit_get_locator((*index).into(), &mut self.control)?,
                BindingAccessOpcode::DefVar => self
                    .bytecode
                    .emit_def_var((*index).into(), &mut self.control)?,
                BindingAccessOpcode::DefEvalVar => {
                    self.bytecode
                        .emit_def_eval_var((*index).into(), &mut self.control)?;
                }
                BindingAccessOpcode::PutLexicalValue => self.bytecode.emit_put_lexical_value(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::DefInitVar => self.bytecode.emit_def_init_var(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::SetName => self.bytecode.emit_set_name(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::GetNameAndLocator => self.bytecode.emit_get_name_and_locator(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::GetNameOrUndefined => {
                    self.bytecode.emit_get_name_or_undefined(
                        value.variable(),
                        (*index).into(),
                        &mut self.control,
                    )?
                }
                BindingAccessOpcode::DeleteName => self.bytecode.emit_delete_name(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
            },
            BindingKind::Stack(index) => match opcode {
                BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode
                        .emit_set_name_by_locator(value.variable(), &mut self.control)?;
                }
                BindingAccessOpcode::GetLocator => self
                    .bytecode
                    .emit_get_locator((*index).into(), &mut self.control)?,
                BindingAccessOpcode::DefVar => self
                    .bytecode
                    .emit_def_var((*index).into(), &mut self.control)?,
                BindingAccessOpcode::DefEvalVar => {
                    self.bytecode
                        .emit_def_eval_var((*index).into(), &mut self.control)?;
                }
                BindingAccessOpcode::PutLexicalValue => self.bytecode.emit_put_lexical_value(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::DefInitVar => self.bytecode.emit_def_init_var(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::SetName => self.bytecode.emit_set_name(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::GetName => self.bytecode.emit_get_name(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::GetNameAndLocator => self.bytecode.emit_get_name_and_locator(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
                BindingAccessOpcode::GetNameOrUndefined => {
                    self.bytecode.emit_get_name_or_undefined(
                        value.variable(),
                        (*index).into(),
                        &mut self.control,
                    )?
                }
                BindingAccessOpcode::DeleteName => self.bytecode.emit_delete_name(
                    value.variable(),
                    (*index).into(),
                    &mut self.control,
                )?,
            },
            BindingKind::Local(None) => {
                let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
                    "access of uninitialized binding"
                )))?;
                self.bytecode
                    .emit_throw_new_reference_error(error_msg.into(), &mut self.control)?;
            }
            BindingKind::Local(Some(index)) => match opcode {
                BindingAccessOpcode::GetName
                | BindingAccessOpcode::GetNameOrUndefined
                | BindingAccessOpcode::GetNameAndLocator => {
                    self.bytecode.emit_move(
                        value.variable(),
                        (*index).into(),
                        &mut self.control,
                    )?;
                }
                BindingAccessOpcode::GetLocator
                | BindingAccessOpcode::DefVar
                | BindingAccessOpcode::DefEvalVar => {}
                BindingAccessOpcode::SetName
                | BindingAccessOpcode::DefInitVar
                | BindingAccessOpcode::PutLexicalValue
                | BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode.emit_move(
                        (*index).into(),
                        value.variable(),
                        &mut self.control,
                    )?;
                }
                BindingAccessOpcode::DeleteName => self
                    .bytecode
                    .emit_store_false(value.variable(), &mut self.control)?,
            },
        }
        Ok(())
    }

    fn emit_get_property_by_name(
        &mut self,
        dst: &Register,
        receiver: Option<&Register>,
        value: &Register,
        ident: Sym,
    ) -> JsResult<()> {
        let name_index = self.get_or_insert_name(ident)?;
        self.control.admit(CompilerWork::MetadataVisit)?;
        let Constant::String(name) = &self.constants[name_index as usize] else {
            unreachable!("there should be a string at index")
        };
        self.control.admit(CompilerWork::MetadataWrite)?;
        let name = name.clone();
        let is_length = receiver.is_none()
            && name
                .as_str()
                .try_eq(StaticJsStrings::LENGTH.as_str(), &mut |work| {
                    self.control.admit(CompilerWork::String(work))
                })
                .map_err(control::string_error)?;
        let ic_index = self.push_inline_cache(name)?;

        if let Some(receiver) = receiver {
            self.bytecode.emit_get_property_by_name_with_this(
                dst.variable(),
                receiver.variable(),
                value.variable(),
                ic_index.into(),
                &mut self.control,
            )?;
        } else if is_length {
            self.bytecode.emit_get_length_property(
                dst.variable(),
                value.variable(),
                ic_index.into(),
                &mut self.control,
            )?;
        } else {
            self.bytecode.emit_get_property_by_name(
                dst.variable(),
                value.variable(),
                ic_index.into(),
                &mut self.control,
            )?;
        }
        Ok(())
    }

    fn emit_set_property_by_name(
        &mut self,
        value: &Register,
        receiver: Option<&Register>,
        object: &Register,
        ident: Sym,
    ) -> JsResult<()> {
        let name_index = self.get_or_insert_name(ident)?;
        self.control.admit(CompilerWork::MetadataVisit)?;
        let Constant::String(name) = &self.constants[name_index as usize] else {
            unreachable!("there should be a string at index")
        };
        self.control.admit(CompilerWork::MetadataWrite)?;
        let name = name.clone();
        let ic_index = self.push_inline_cache(name)?;

        if let Some(receiver) = receiver {
            self.bytecode.emit_set_property_by_name_with_this(
                value.variable(),
                receiver.variable(),
                object.variable(),
                ic_index.into(),
                &mut self.control,
            )?;
        } else {
            self.bytecode.emit_set_property_by_name(
                value.variable(),
                object.variable(),
                ic_index.into(),
                &mut self.control,
            )?;
        }
        Ok(())
    }

    fn emit_type_error(&mut self, message: &str) -> JsResult<()> {
        let message = JsString::try_from_utf8(message, &mut |work| {
            self.control.admit(CompilerWork::String(work))
        })
        .map_err(control::string_error)?;
        let error_msg = self.get_or_insert_literal(Literal::String(message))?;
        self.bytecode
            .emit_throw_new_type_error(error_msg.into(), &mut self.control)?;
        Ok(())
    }

    fn emit_store_integer(&mut self, value: i32, dst: &Register) -> JsResult<()> {
        self.emit_store_integer_with_index(value, dst.variable())?;
        Ok(())
    }

    fn emit_store_integer_with_index(&mut self, value: i32, dst: RegisterOperand) -> JsResult<()> {
        match value {
            0 => self.bytecode.emit_store_zero(dst, &mut self.control)?,
            1 => self.bytecode.emit_store_one(dst, &mut self.control)?,
            x if i32::from(x as i8) == x => {
                self.bytecode
                    .emit_store_int8(dst, x as i8, &mut self.control)?
            }
            x if i32::from(x as i16) == x => {
                self.bytecode
                    .emit_store_int16(dst, x as i16, &mut self.control)?;
            }
            x => self.bytecode.emit_store_int32(dst, x, &mut self.control)?,
        }
        Ok(())
    }

    fn emit_store_literal(&mut self, literal: Literal, dst: &Register) -> JsResult<()> {
        let index = self.get_or_insert_literal(literal)?;
        self.bytecode
            .emit_store_literal(dst.variable(), index.into(), &mut self.control)?;
        Ok(())
    }

    fn emit_store_rational(&mut self, value: f64, dst: &Register) -> JsResult<()> {
        if value.is_nan() {
            return self
                .bytecode
                .emit_store_nan(dst.variable(), &mut self.control);
        }

        if value.is_infinite() {
            if value.is_sign_positive() {
                return self
                    .bytecode
                    .emit_store_positive_infinity(dst.variable(), &mut self.control);
            }
            return self
                .bytecode
                .emit_store_negative_infinity(dst.variable(), &mut self.control);
        }

        // Check if the f64 value can fit in an i32.
        if f64::from(value as i32).to_bits() == value.to_bits() {
            self.emit_store_integer(value as i32, dst)?;
        } else {
            let f32_value = value as f32;

            #[allow(clippy::float_cmp)]
            if f64::from(f32_value) == value {
                self.bytecode
                    .emit_store_float(dst.variable(), f32_value, &mut self.control)?;
            } else {
                self.bytecode
                    .emit_store_double(dst.variable(), value, &mut self.control)?;
            }
        }
        Ok(())
    }

    fn jump(&mut self) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode
            .emit_jump(Self::DUMMY_ADDRESS, &mut self.control)?;
        Ok(Label { index })
    }

    pub(crate) fn jump_if_true(&mut self, value: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_true(
            Self::DUMMY_ADDRESS,
            value.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    pub(crate) fn if_else(
        &mut self,
        bool: &Register,
        true_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
        false_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
    ) -> JsResult<()> {
        let jump_false = self.jump_if_false(bool)?;

        // if true, jump to end to avoid running the code for the `else`
        true_case(self)?;
        let jump_to_end = self.jump()?;

        // if false, we should be already at the end so no need to do anything.
        self.patch_jump(jump_false)?;
        false_case(self)?;
        self.patch_jump(jump_to_end)?;
        Ok(())
    }

    /// Generates the `if-else` pattern.
    ///
    /// This will also deallocate the `bool` register.
    pub(crate) fn if_else_with_dealloc(
        &mut self,
        bool: Register,
        true_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
        false_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
    ) -> JsResult<()> {
        let jump_false = self.jump_if_false(&bool)?;
        self.register_allocator.dealloc(bool, &mut self.control)?;

        // if true, jump to end to avoid running the code for the `else`
        true_case(self)?;
        let jump_to_end = self.jump()?;

        // if false, we should be already at the end so no need to do anything.
        self.patch_jump(jump_false)?;
        false_case(self)?;
        self.patch_jump(jump_to_end)?;
        Ok(())
    }

    /// Generates the `if`-`else` pattern directly from a condition expression,
    /// fusing a relational condition into the branch when possible.
    ///
    /// For a relational condition (`<`, `<=`, `>`, `>=`) this emits a single
    /// fused comparison+branch opcode (e.g. `if (a < b)` becomes one
    /// `JumpIfNotLessThan`) instead of computing a boolean into a register with
    /// `LessThan` and then testing it with `JumpIfFalse` — saving a dispatch and
    /// a register. Non-relational conditions fall back to compiling the
    /// condition into a temporary register followed by `JumpIfFalse`.
    pub(crate) fn compile_if_else(
        &mut self,
        condition: &Expression,
        true_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
        false_case: impl FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<()>,
    ) -> JsResult<()> {
        let jump_false = self.compile_condition_and_branch(condition, None)?;

        // if true, jump to end to avoid running the code for the `else`
        true_case(self)?;
        let jump_to_end = self.jump()?;

        // if false, we should be already at the end so no need to do anything.
        self.patch_jump(jump_false)?;
        false_case(self)?;
        self.patch_jump(jump_to_end)?;
        Ok(())
    }

    pub(crate) fn jump_if_false(&mut self, value: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_false(
            Self::DUMMY_ADDRESS,
            value.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    /// Compile a condition expression and emit a conditional jump.
    ///
    /// When the condition is a relational comparison (`<`, `<=`, `>`, `>=`),
    /// emits a single fused comparison+branch opcode instead of separate
    /// `LessThan` + `JumpIfFalse` instructions.
    ///
    /// If `hoisted` is provided, the pre-compiled register is used for one
    /// operand of the comparison, avoiding a constant reload on every iteration.
    pub(crate) fn compile_condition_and_branch(
        &mut self,
        condition: &Expression,
        hoisted: Option<&HoistedOperand>,
    ) -> JsResult<Label> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        // `flatten()` strips outer parentheses so that conditions like
        // `(a < b)` (common in ternaries and hand-parenthesized code) still
        // reach the fused comparison+branch path.
        if let Expression::Binary(binary) = self.flatten_expression(condition)?
            && let BinaryOp::Relational(op) = binary.op()
            && let Some(label) = self.try_fused_comparison_branch(op, binary, hoisted)?
        {
            return Ok(label);
        }
        // Fallback: compile expr + jump_if_false
        let value = self.register_allocator.alloc(&mut self.control)?;
        self.compile_expr(condition, &value)?;
        let label = self.jump_if_false(&value)?;
        self.register_allocator.dealloc(value, &mut self.control)?;
        Ok(label)
    }

    /// Try to hoist a constant operand from a loop condition into a register
    /// that is loaded once before the loop, rather than on every iteration.
    ///
    /// Returns `Some(HoistedOperand)` when one side of a relational comparison
    /// is a literal value (e.g. `i < 10` hoists `10`).
    pub(crate) fn try_hoist_loop_condition(
        &mut self,
        condition: Option<&Expression>,
    ) -> JsResult<Option<HoistedOperand>> {
        self.control.admit(CompilerWork::Search)?;
        let Some(condition) = condition else {
            return Ok(None);
        };
        let Expression::Binary(binary) = condition else {
            return Ok(None);
        };
        let BinaryOp::Relational(op) = binary.op() else {
            return Ok(None);
        };
        match op {
            RelationalOp::LessThan
            | RelationalOp::LessThanOrEqual
            | RelationalOp::GreaterThan
            | RelationalOp::GreaterThanOrEqual => {}
            _ => return Ok(None),
        }
        // Prefer hoisting RHS (most common pattern: `i < 10`)
        if self.is_loop_invariant(binary.rhs())? {
            let reg = self.register_allocator.alloc(&mut self.control)?;
            self.compile_expr(binary.rhs(), &reg)?;
            return Ok(Some(HoistedOperand {
                register: reg,
                is_rhs: true,
            }));
        }
        // Try LHS (less common: `0 < i`)
        if self.is_loop_invariant(binary.lhs())? {
            let reg = self.register_allocator.alloc(&mut self.control)?;
            self.compile_expr(binary.lhs(), &reg)?;
            return Ok(Some(HoistedOperand {
                register: reg,
                is_rhs: false,
            }));
        }
        Ok(None)
    }

    /// Returns `true` if the expression is loop-invariant and would benefit
    /// from being hoisted out of a loop (compiled once before the loop rather
    /// than on every iteration).
    ///
    /// An expression qualifies if it is:
    /// - A literal value (always immutable, always emits code)
    /// - A `const` identifier that is NOT already optimized by
    ///   [`compile_expr_operand`](Self::compile_expr_operand) (i.e., not local
    ///   and not in `const_binding_cache`), since those are already handled
    ///   with zero instructions inside the loop.
    fn is_loop_invariant(&mut self, expr: &Expression) -> JsResult<bool> {
        self.control.admit(CompilerWork::Search)?;
        Ok(match expr {
            Expression::Literal(_) => true,
            Expression::Identifier(name) => {
                let name = self.resolve_identifier_expect(*name)?;
                let binding = self.lexical_identifier_reference(name.clone())?;
                // Local bindings already use persistent registers directly
                // in compile_expr_operand — hoisting would add a redundant Move.
                if binding.local() {
                    return Ok(false);
                }
                // Cached const bindings are already handled by compile_expr_operand
                // without emitting code — hoisting would add a redundant Move.
                if !self.in_with
                    && self
                        .const_binding_cache
                        .contains_key(&binding.locator(), &mut self.control)?
                {
                    return Ok(false);
                }
                // Only hoist if the binding is immutable (const).
                // Mutable bindings (let, var) can change between iterations.
                matches!(self.lexical_is_binding_mutable(&name)?, Some(false))
            }
            _ => false,
        })
    }

    fn try_fused_comparison_branch(
        &mut self,
        op: RelationalOp,
        binary: &Binary,
        hoisted: Option<&HoistedOperand>,
    ) -> JsResult<Option<Label>> {
        self.control.admit(CompilerWork::Search)?;
        use crate::vm::opcode::BytecodeEmitter;

        let emit_fn: fn(
            &mut BytecodeEmitter,
            Address,
            RegisterOperand,
            RegisterOperand,
            &mut CompilerControl<'_>,
        ) -> JsResult<()> = match op {
            RelationalOp::LessThan => BytecodeEmitter::emit_jump_if_not_less_than,
            RelationalOp::LessThanOrEqual => BytecodeEmitter::emit_jump_if_not_less_than_or_equal,
            RelationalOp::GreaterThan => BytecodeEmitter::emit_jump_if_not_greater_than,
            RelationalOp::GreaterThanOrEqual => {
                BytecodeEmitter::emit_jump_if_not_greater_than_or_equal
            }
            _ => return Ok(None),
        };
        let mut label_index = Address::new(0);
        match hoisted {
            Some(h) if h.is_rhs => {
                let rhs = h.register.variable();
                self.compile_expr_operand(binary.lhs(), |compiler, lhs| {
                    label_index = compiler.next_opcode_location()?;
                    emit_fn(
                        &mut compiler.bytecode,
                        Self::DUMMY_ADDRESS,
                        lhs,
                        rhs,
                        &mut compiler.control,
                    )?;
                    Ok(())
                })?;
            }
            Some(h) => {
                let lhs = h.register.variable();
                self.compile_expr_operand(binary.rhs(), |compiler, rhs| {
                    label_index = compiler.next_opcode_location()?;
                    emit_fn(
                        &mut compiler.bytecode,
                        Self::DUMMY_ADDRESS,
                        lhs,
                        rhs,
                        &mut compiler.control,
                    )?;
                    Ok(())
                })?;
            }
            None => {
                self.compile_expr_operand(binary.lhs(), |compiler, lhs| {
                    compiler.compile_expr_operand(binary.rhs(), |compiler, rhs| {
                        label_index = compiler.next_opcode_location()?;
                        emit_fn(
                            &mut compiler.bytecode,
                            Self::DUMMY_ADDRESS,
                            lhs,
                            rhs,
                            &mut compiler.control,
                        )?;
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
        }
        Ok(Some(Label { index: label_index }))
    }

    pub(crate) fn jump_if_null_or_undefined(&mut self, value: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_null_or_undefined(
            Self::DUMMY_ADDRESS,
            value.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    pub(crate) fn jump_if_not_undefined(&mut self, value: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_not_undefined(
            Self::DUMMY_ADDRESS,
            value.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    pub(crate) fn jump_if_neq(&mut self, lhs: &Register, rhs: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_not_equal(
            Self::DUMMY_ADDRESS,
            lhs.variable(),
            rhs.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    pub(crate) fn case(&mut self, value: &Register, condition: &Register) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_case(
            Self::DUMMY_ADDRESS,
            value.variable(),
            condition.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    pub(crate) fn template_lookup(&mut self, dst: &Register, site: u64) -> JsResult<Label> {
        let index = self.next_opcode_location()?;
        self.bytecode.emit_template_lookup(
            Self::DUMMY_ADDRESS,
            site,
            dst.variable(),
            &mut self.control,
        )?;
        Ok(Label { index })
    }

    fn emit_resume_kind(
        &mut self,
        resume_kind: GeneratorResumeKind,
        dst: &Register,
    ) -> JsResult<()> {
        self.emit_store_integer(resume_kind as i32, dst)?;
        Ok(())
    }

    fn jump_if_not_resume_kind(
        &mut self,
        resume_kind: GeneratorResumeKind,
        value: &Register,
    ) -> JsResult<Label> {
        let r1 = self.register_allocator.alloc(&mut self.control)?;
        self.emit_store_integer((resume_kind as u8).into(), &r1)?;

        let index = self.next_opcode_location()?;
        self.bytecode.emit_jump_if_not_equal(
            Self::DUMMY_ADDRESS,
            r1.variable(),
            value.variable(),
            &mut self.control,
        )?;
        self.register_allocator.dealloc(r1, &mut self.control)?;
        Ok(Label { index })
    }

    #[track_caller]
    pub(crate) fn patch_jump_with_target(&mut self, label: Label, target: Address) -> JsResult<()> {
        self.bytecode
            .patch_jump(label.index, target, &mut self.control)?;
        Ok(())
    }

    fn patch_jump(&mut self, label: Label) -> JsResult<()> {
        let target = self.next_opcode_location()?;
        self.patch_jump_with_target(label, target)?;
        Ok(())
    }

    fn resolve_identifier_expect(&mut self, identifier: Identifier) -> JsResult<JsString> {
        identifier.try_to_js_string(&*self.interner, &mut self.control)
    }

    fn super_(&mut self, this: &Register, super_: &Register) -> JsResult<()> {
        // Reuse super to avoid allocating a register just to get the function object.
        self.bytecode
            .emit_this(this.variable(), &mut self.control)?;
        self.bytecode
            .emit_get_function_object(super_.variable(), &mut self.control)?;
        self.bytecode
            .emit_get_home_object(super_.variable(), &mut self.control)?;

        let r1 = self.register_allocator.alloc(&mut self.control)?;
        self.bytecode
            .emit_store_null(r1.variable(), &mut self.control)?;
        // jump to evaluation if `super` is already a valid home object.
        let skip_move = self.jump_if_neq(&r1, super_)?;
        self.bytecode
            .emit_move(r1.variable(), this.variable(), &mut self.control)?;
        self.bytecode
            .emit_is_object(r1.variable(), &mut self.control)?;

        // If `this` is also not an object, then `super` should be left as `null`.
        // Jump to the end in this case because `super` is already `null`.
        let skip_eval = self.jump_if_false(&r1)?;
        self.register_allocator.dealloc(r1, &mut self.control)?;

        // `this` is an object, so replace `super` with it and get its
        // prototype.
        self.bytecode
            .emit_move(super_.variable(), this.variable(), &mut self.control)?;
        self.patch_jump(skip_move)?;

        self.bytecode
            .emit_get_prototype(super_.variable(), &mut self.control)?;

        self.patch_jump(skip_eval)?;
        Ok(())
    }

    fn access_get(&mut self, access: Access<'_>, dst: &Register) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match access {
            Access::Variable { name } => {
                let name = self.resolve_identifier_expect(name)?;
                let binding = self.lexical_identifier_reference(name)?;
                let index = self.get_binding(&binding)?;
                if !self.in_with
                    && let Some(&cached_reg) = self
                        .const_binding_cache
                        .get(&binding.locator(), &mut self.control)?
                {
                    self.bytecode.emit_move(
                        dst.variable(),
                        cached_reg.into(),
                        &mut self.control,
                    )?;
                    return Ok(());
                }
                self.emit_binding_access(BindingAccessOpcode::GetName, &index, dst)?;
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let mut source_position = self.position_guard(access.field())?;
                    let compiler = &mut *source_position;

                    let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.compile_expr(access.target(), &object)?;

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(dst, None, &object, ident.sym())?;
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(expr, &key)?;
                            compiler.bytecode.emit_get_property_by_value(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                                &mut compiler.control,
                            )?;
                            compiler
                                .register_allocator
                                .dealloc(key, &mut compiler.control)?;
                        }
                    }
                    compiler
                        .register_allocator
                        .dealloc(object, &mut compiler.control)?;
                    source_position.finish()?;
                }
                PropertyAccess::Private(access) => {
                    let mut source_position = self.position_guard(access.field())?;
                    let compiler = &mut *source_position;

                    let index = compiler.get_or_insert_private_name(access.field())?;
                    let object = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.compile_expr(access.target(), &object)?;
                    compiler.bytecode.emit_get_private_field(
                        dst.variable(),
                        object.variable(),
                        index.into(),
                        &mut compiler.control,
                    )?;
                    compiler
                        .register_allocator
                        .dealloc(object, &mut compiler.control)?;
                    source_position.finish()?;
                }
                PropertyAccess::Super(access) => {
                    let mut source_position = self.position_guard(access.field())?;
                    let compiler = &mut *source_position;

                    let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                    let receiver = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.super_(&receiver, &value)?;

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(
                                dst,
                                Some(&receiver),
                                &value,
                                ident.sym(),
                            )?;
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc(&mut compiler.control)?;
                            compiler.compile_expr(expr, &key)?;
                            compiler.bytecode.emit_get_property_by_value(
                                dst.variable(),
                                key.variable(),
                                receiver.variable(),
                                value.variable(),
                                &mut compiler.control,
                            )?;
                            compiler
                                .register_allocator
                                .dealloc(key, &mut compiler.control)?;
                        }
                    }
                    compiler
                        .register_allocator
                        .dealloc(receiver, &mut compiler.control)?;
                    compiler
                        .register_allocator
                        .dealloc(value, &mut compiler.control)?;
                    source_position.finish()?;
                }
            },
            Access::This => {
                self.bytecode.emit_this(dst.variable(), &mut self.control)?;
            }
        }
        Ok(())
    }

    fn access_set<'a, F>(&mut self, access: Access<'_>, expr_fn: F) -> JsResult<()>
    where
        F: FnOnce(&mut ByteCompiler<'_, '_>) -> JsResult<&'a Register>,
    {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match access {
            Access::Variable { name } => {
                let name = self.resolve_identifier_expect(name)?;
                let binding = self.lexical_identifier_reference(name.clone())?;
                let is_lexical = binding.is_lexical();
                let index = self.get_binding(&binding)?;

                let value = self.register_allocator.alloc(&mut self.control)?;
                if !is_lexical {
                    self.emit_binding_access(BindingAccessOpcode::GetLocator, &index, &value)?;
                }
                self.register_allocator.dealloc(value, &mut self.control)?;

                let value = expr_fn(self)?;

                if is_lexical {
                    match self.lexical_set_mutable_binding(name.clone())? {
                        Ok(binding) => {
                            let index = self.insert_binding(binding)?;
                            self.emit_binding_access(BindingAccessOpcode::SetName, &index, value)?;
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name)?;
                            self.bytecode
                                .emit_throw_mutate_immutable(index.into(), &mut self.control)?;
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    self.emit_binding_access(BindingAccessOpcode::SetNameByLocator, &index, value)?;
                }
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(access.target(), &object)?;
                        let value = expr_fn(self)?;
                        self.emit_set_property_by_name(value, None, &object, name.sym())?;
                        self.register_allocator.dealloc(object, &mut self.control)?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(access.target(), &object)?;

                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;

                        let value = expr_fn(self)?;

                        self.bytecode.emit_set_property_by_value(
                            value.variable(),
                            key.variable(),
                            object.variable(),
                            object.variable(),
                            &mut self.control,
                        )?;

                        self.register_allocator.dealloc(object, &mut self.control)?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                },
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field())?;

                    let object = self.register_allocator.alloc(&mut self.control)?;
                    self.compile_expr(access.target(), &object)?;

                    let value = expr_fn(self)?;

                    self.bytecode.emit_set_private_field(
                        value.variable(),
                        object.variable(),
                        index.into(),
                        &mut self.control,
                    )?;

                    self.register_allocator.dealloc(object, &mut self.control)?;
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc(&mut self.control)?;
                        let receiver = self.register_allocator.alloc(&mut self.control)?;
                        self.super_(&receiver, &object)?;

                        let value = expr_fn(self)?;

                        self.emit_set_property_by_name(
                            value,
                            Some(&receiver),
                            &object,
                            name.sym(),
                        )?;

                        self.register_allocator
                            .dealloc(receiver, &mut self.control)?;
                        self.register_allocator.dealloc(object, &mut self.control)?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc(&mut self.control)?;
                        let receiver = self.register_allocator.alloc(&mut self.control)?;
                        self.super_(&receiver, &object)?;

                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;

                        let value = expr_fn(self)?;

                        self.bytecode.emit_set_property_by_value(
                            value.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
                            &mut self.control,
                        )?;

                        self.register_allocator.dealloc(key, &mut self.control)?;
                        self.register_allocator
                            .dealloc(receiver, &mut self.control)?;
                        self.register_allocator.dealloc(object, &mut self.control)?;
                    }
                },
            },
            Access::This => {
                // In non-strict code, assignment to `this` is a no-op per spec; we still evaluate
                // the RHS for side effects (e.g. `this = foo()` must call foo()).
                let _ = expr_fn(self)?;
            }
        }
        Ok(())
    }

    fn access_delete(&mut self, access: Access<'_>, dst: &Register) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match access {
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name(name.sym())?;
                        self.compile_expr(access.target(), dst)?;
                        self.bytecode.emit_delete_property_by_name(
                            dst.variable(),
                            index.into(),
                            &mut self.control,
                        )?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), dst)?;
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;
                        self.bytecode.emit_delete_property_by_value(
                            dst.variable(),
                            key.variable(),
                            &mut self.control,
                        )?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                },
                PropertyAccess::Super(_) => {
                    self.bytecode.emit_delete_super_throw(&mut self.control)?
                }
                PropertyAccess::Private(_) => {
                    unreachable!("deleting private properties should always throw early errors.")
                }
            },
            Access::Variable { name } => {
                let name = name.try_to_js_string(&*self.interner, &mut self.control)?;
                let binding = self.lexical_identifier_reference(name)?;
                let index = self.get_binding(&binding)?;
                self.emit_binding_access(BindingAccessOpcode::DeleteName, &index, dst)?;
            }
            Access::This => self
                .bytecode
                .emit_store_true(dst.variable(), &mut self.control)?,
        }
        Ok(())
    }

    /// Compile a [`StatementList`].
    pub fn compile_statement_list(
        &mut self,
        list: &StatementList,
        use_expr: bool,
        block: bool,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        if use_expr || self.jump_control_info_has_use_expr()? {
            let mut use_expr_index = 0;
            for (i, statement) in list.statements().iter().enumerate() {
                self.control.admit(CompilerWork::Search)?;
                match statement {
                    StatementListItem::Statement(statement) => match statement.as_ref() {
                        Statement::Break(_) | Statement::Continue(_) => break,
                        Statement::Empty | Statement::Var(_) => {}
                        Statement::Block(block)
                            if !self.control.with_name_admission(|admission| {
                                try_returns_value(block, admission)
                            })? => {}
                        _ => use_expr_index = i,
                    },
                    StatementListItem::Declaration(_) => {}
                }
            }

            for (i, item) in list.statements().iter().enumerate() {
                self.compile_stmt_list_item(item, i == use_expr_index, block)?;
            }
        } else {
            for item in list.statements() {
                self.compile_stmt_list_item(item, false, block)?;
            }
        }
        Ok(())
    }

    /// Compile an [`Expression`].
    #[inline]
    pub(crate) fn compile_expr(&mut self, expr: &Expression, dst: &'_ Register) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        self.compile_expr_impl(expr, dst)?;
        Ok(())
    }

    /// Compile an expression purely for its side effects, discarding the result.
    ///
    /// This avoids allocating a register and emitting a store for the result
    /// when it's not needed (e.g., expression statements, for-loop updates).
    pub(crate) fn compile_expr_for_side_effects(&mut self, expr: &Expression) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match expr {
            Expression::Call(call) => {
                self.call(Callable::Call(call), CallResultDest::Discard)?;
            }
            Expression::New(new) => {
                self.call(Callable::New(new), CallResultDest::Discard)?;
            }
            Expression::Update(update) => {
                let tmp = self.register_allocator.alloc(&mut self.control)?;
                self.compile_update(update, &tmp, true)?;
                self.register_allocator.dealloc(tmp, &mut self.control)?;
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr_for_side_effects(parenthesized.expression())?;
            }
            _ => {
                let tmp = self.register_allocator.alloc(&mut self.control)?;
                self.compile_expr(expr, &tmp)?;
                self.register_allocator.dealloc(tmp, &mut self.control)?;
            }
        }
        Ok(())
    }

    /// Compile an expression and leave the result on the stack.
    ///
    /// For call/new expressions, this avoids the `PopIntoRegister` + `PushFromRegister`
    /// round-trip by leaving the call result directly on the stack. For identifier
    /// expressions resolving to a local or const-cached register, this pushes from
    /// that register directly, avoiding a `Move` into a temporary.
    pub(crate) fn compile_expr_to_stack(&mut self, expr: &Expression) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match expr {
            Expression::Call(call) => {
                self.call(Callable::Call(call), CallResultDest::Stack)?;
            }
            Expression::New(new) => {
                self.call(Callable::New(new), CallResultDest::Stack)?;
            }
            Expression::Parenthesized(parenthesized) => {
                self.compile_expr_to_stack(parenthesized.expression())?;
            }
            _ => {
                self.compile_expr_operand(expr, |this, op| {
                    this.bytecode
                        .emit_push_from_register(op, &mut this.control)?;
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    /// Compile an expression and return the operand where the result lives.
    ///
    /// For local variable references, this returns the local's persistent register
    /// directly without emitting a `Move` instruction. For all other expressions,
    /// it allocates a temporary register and compiles into it.
    ///
    /// The `inner_fn` passed in will be called before the register get deallocated.
    pub(crate) fn compile_expr_operand(
        &mut self,
        expr: &Expression,
        inner_fn: impl FnOnce(&mut Self, RegisterOperand) -> JsResult<()>,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        if let Expression::Identifier(name) = expr {
            let name = self.resolve_identifier_expect(*name)?;
            let binding = self.lexical_identifier_reference(name)?;
            let index = self.get_binding(&binding)?;
            if let BindingKind::Local(Some(local_reg)) = &index {
                inner_fn(self, RegisterOperand::from(*local_reg))?;
                return Ok(());
            }
            if !self.in_with
                && let Some(&cached_reg) = self
                    .const_binding_cache
                    .get(&binding.locator(), &mut self.control)?
            {
                inner_fn(self, cached_reg.into())?;
                return Ok(());
            }
        }
        let reg = self.register_allocator.alloc(&mut self.control)?;
        self.compile_expr(expr, &reg)?;
        let op = reg.variable();
        inner_fn(self, op)?;
        self.register_allocator.dealloc(reg, &mut self.control)?;
        Ok(())
    }

    /// Compile a property access expression, prepending `this` to the property value in the stack.
    ///
    /// This compiles the access in a way that the state of the stack after executing the property
    /// access becomes `...rest, this, value`. where `...rest` is the rest of the stack, `this` is the
    /// `this` value of the access, and `value` is the final result of the access.
    ///
    /// This is mostly useful for optional chains with calls (`a.b?.()`) and for regular chains
    /// with calls (`a.b()`), since both of them must have `a` be the value of `this` for the function
    /// call `b()`, but a regular compilation of the access would lose the `this` value after accessing
    /// `b`.
    fn compile_access_preserve_this(
        &mut self,
        access: &PropertyAccess,
        this: &Register,
        dst: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match access {
            PropertyAccess::Simple(access) => {
                self.compile_expr(access.target(), this)?;

                match access.field() {
                    PropertyAccessField::Const(ident) => {
                        self.emit_get_property_by_name(dst, None, this, ident.sym())?;
                    }
                    PropertyAccessField::Expr(field) => {
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(field, &key)?;
                        self.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            this.variable(),
                            this.variable(),
                            &mut self.control,
                        )?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                }
            }
            PropertyAccess::Private(access) => {
                self.compile_expr(access.target(), this)?;

                let index = self.get_or_insert_private_name(access.field())?;
                self.bytecode.emit_get_private_field(
                    dst.variable(),
                    this.variable(),
                    index.into(),
                    &mut self.control,
                )?;
            }
            PropertyAccess::Super(access) => {
                let object = self.register_allocator.alloc(&mut self.control)?;
                self.super_(this, &object)?;

                match access.field() {
                    PropertyAccessField::Const(ident) => {
                        self.emit_get_property_by_name(dst, Some(this), &object, ident.sym())?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;
                        self.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            this.variable(),
                            object.variable(),
                            &mut self.control,
                        )?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                }
                self.register_allocator.dealloc(object, &mut self.control)?;
            }
        }
        Ok(())
    }

    /// Compile an optional chain expression, prepending `this` to the property value in the stack.
    ///
    /// This compiles the access in a way that the state of the stack after executing the optional
    /// chain becomes `...rest, this, value`. where `...rest` is the rest of the stack, `this` is the
    /// `this` value of the chain, and `value` is the result of the chain.
    ///
    /// This is mostly useful for inner optional chains with external calls (`(a?.b)()`), because the
    /// external call is not in the optional chain, and compiling an optional chain in the usual way
    /// would only return the result of the chain without preserving the `this` value. In other words,
    /// `this` would be set to `undefined` for that call, which is incorrect since `a` should be the
    /// `this` value of the call.
    fn compile_optional_preserve_this(
        &mut self,
        optional: &Optional,
        this: &Register,
        value: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        let mut jumps = Vec::new();
        reserve(&mut jumps, optional.chain().len(), &mut self.control)?;

        match self.flatten_expression(optional.target())? {
            Expression::PropertyAccess(access) => {
                self.compile_access_preserve_this(access, this, value)?;
            }
            Expression::Optional(opt) => self.compile_optional_preserve_this(opt, this, value)?,
            expr => {
                self.bytecode
                    .emit_store_undefined(this.variable(), &mut self.control)?;
                self.compile_expr(expr, value)?;
            }
        }

        let jump = self.jump_if_null_or_undefined(value)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        jumps.push(jump);

        let (first, rest) = optional
            .chain()
            .split_first()
            .expect("chain must have at least one element");
        self.control.admit(CompilerWork::Search)?;
        assert!(first.shorted());

        self.compile_optional_item_kind(first.kind(), this, value)?;

        for item in rest {
            self.control.admit(CompilerWork::Visit)?;
            if item.shorted() {
                let jump = self.jump_if_null_or_undefined(value)?;
                self.control.admit(CompilerWork::MetadataWrite)?;
                jumps.push(jump);
            }
            self.compile_optional_item_kind(item.kind(), this, value)?;
        }

        let skip_undef = self.jump()?;

        for label in jumps {
            self.control.admit(CompilerWork::MetadataVisit)?;
            self.patch_jump(label)?;
            self.bytecode
                .emit_store_undefined(value.variable(), &mut self.control)?;
        }

        self.patch_jump(skip_undef)?;
        Ok(())
    }

    /// Compile a `delete` expression on an optional chain.
    ///
    /// Follows the same short-circuit logic as `compile_optional_preserve_this`, but
    /// emits delete opcodes for the final link in the chain instead of get opcodes.
    /// When the chain short-circuits (base is null/undefined), returns `true` per spec.
    pub(crate) fn compile_optional_delete(
        &mut self,
        optional: &Optional,
        dst: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        let value = self.register_allocator.alloc(&mut self.control)?;
        let this = self.register_allocator.alloc(&mut self.control)?;

        let mut jumps = Vec::new();
        reserve(&mut jumps, optional.chain().len(), &mut self.control)?;

        // Compile the target expression (base of the chain).
        match self.flatten_expression(optional.target())? {
            Expression::PropertyAccess(access) => {
                self.compile_access_preserve_this(access, &this, &value)?;
            }
            Expression::Optional(opt) => {
                self.compile_optional_preserve_this(opt, &this, &value)?;
            }
            expr => {
                self.bytecode
                    .emit_store_undefined(this.variable(), &mut self.control)?;
                self.compile_expr(expr, &value)?;
            }
        }

        let jump = self.jump_if_null_or_undefined(&value)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        jumps.push(jump);

        let chain = optional.chain();
        let chain_len = chain.len();

        let (first, rest) = chain
            .split_first()
            .expect("chain must have at least one element");
        self.control.admit(CompilerWork::Search)?;
        assert!(first.shorted());

        if chain_len == 1 {
            // The only item is also the last — emit delete.
            self.compile_optional_delete_final(first.kind(), &this, &value, dst)?;
        } else {
            // Not the last — emit a regular get.
            self.compile_optional_item_kind(first.kind(), &this, &value)?;

            for (i, item) in rest.iter().enumerate() {
                self.control.admit(CompilerWork::Visit)?;
                if item.shorted() {
                    let jump = self.jump_if_null_or_undefined(&value)?;
                    self.control.admit(CompilerWork::MetadataWrite)?;
                    jumps.push(jump);
                }
                if i == rest.len() - 1 {
                    // Last item — emit delete.
                    self.compile_optional_delete_final(item.kind(), &this, &value, dst)?;
                } else {
                    self.compile_optional_item_kind(item.kind(), &this, &value)?;
                }
            }
        }

        let skip_true = self.jump()?;

        // Short-circuit path: delete on null/undefined base returns true per spec.
        for label in jumps {
            self.control.admit(CompilerWork::MetadataVisit)?;
            self.patch_jump(label)?;
        }
        self.bytecode
            .emit_store_true(dst.variable(), &mut self.control)?;

        self.patch_jump(skip_true)?;

        self.register_allocator.dealloc(this, &mut self.control)?;
        self.register_allocator.dealloc(value, &mut self.control)?;
        Ok(())
    }

    /// Emit delete opcodes for the final operation in an optional chain.
    fn compile_optional_delete_final(
        &mut self,
        kind: &OptionalOperationKind,
        this: &Register,
        value: &Register,
        dst: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match kind {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.bytecode
                    .emit_move(this.variable(), value.variable(), &mut self.control)?;
                match field {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name(name.sym())?;
                        self.bytecode.emit_move(
                            dst.variable(),
                            value.variable(),
                            &mut self.control,
                        )?;
                        self.bytecode.emit_delete_property_by_name(
                            dst.variable(),
                            index.into(),
                            &mut self.control,
                        )?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.bytecode.emit_move(
                            dst.variable(),
                            value.variable(),
                            &mut self.control,
                        )?;
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;
                        self.bytecode.emit_delete_property_by_value(
                            dst.variable(),
                            key.variable(),
                            &mut self.control,
                        )?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { .. } => {
                unreachable!("deleting private properties should always throw early errors.")
            }
            OptionalOperationKind::Call { .. } => {
                // `delete obj?.method()` — evaluate the call, then return true.
                self.compile_optional_item_kind(kind, this, value)?;
                self.bytecode
                    .emit_store_true(dst.variable(), &mut self.control)?;
            }
        }
        Ok(())
    }

    /// Compile a single operation in an optional chain.
    ///
    /// On successful compilation, the state of the stack on execution will become `...rest, this, value`,
    /// where `this` is the target of the property access (`undefined` on calls), and `value` is the
    /// result of executing the action.
    /// For example, in the expression `a?.b.c()`, after compiling and executing:
    ///
    /// - `a?.b`, the state of the stack will become `...rest, a, b`.
    /// - `b.c`, the state of the stack will become `...rest, b, c`.
    /// - `c()`, the state of the stack will become `...rest, undefined, c()`.
    ///
    /// # Requirements
    /// - This should only be called after verifying that the previous value of the chain
    ///   is not null or undefined (if the operator `?.` was used).
    /// - This assumes that the state of the stack before compiling is `...rest, this, value`,
    ///   since the operation compiled by this function could be a call.
    fn compile_optional_item_kind(
        &mut self,
        kind: &OptionalOperationKind,
        this: &Register,
        value: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match kind {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.bytecode
                    .emit_move(this.variable(), value.variable(), &mut self.control)?;
                match field {
                    PropertyAccessField::Const(name) => {
                        self.emit_get_property_by_name(value, None, value, name.sym())?;
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc(&mut self.control)?;
                        self.compile_expr(expr, &key)?;
                        self.bytecode.emit_get_property_by_value(
                            value.variable(),
                            key.variable(),
                            value.variable(),
                            value.variable(),
                            &mut self.control,
                        )?;
                        self.register_allocator.dealloc(key, &mut self.control)?;
                    }
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.bytecode
                    .emit_move(this.variable(), value.variable(), &mut self.control)?;
                let index = self.get_or_insert_private_name(*field)?;
                self.bytecode.emit_get_private_field(
                    value.variable(),
                    value.variable(),
                    index.into(),
                    &mut self.control,
                )?;
            }
            OptionalOperationKind::Call { args } => {
                self.push_from_register(this)?;
                self.push_from_register(value)?;

                let args = &**args;
                let contains_spread = self.arguments_contain_spread(args)?;

                if contains_spread {
                    let array = self.register_allocator.alloc(&mut self.control)?;
                    let array_op = array.variable();

                    self.bytecode
                        .emit_store_new_array(array_op, &mut self.control)?;

                    for arg in args {
                        self.control.admit(CompilerWork::Visit)?;
                        if let Expression::Spread(spread) = arg {
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            self.compile_expr(spread.target(), &value)?;
                            self.bytecode
                                .emit_get_iterator(value.variable(), &mut self.control)?;
                            self.bytecode
                                .emit_push_iterator_to_array(array_op, &mut self.control)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        } else {
                            self.compile_expr_operand(arg, |this, op| {
                                this.bytecode.emit_push_value_to_array(
                                    op,
                                    array_op,
                                    &mut this.control,
                                )?;
                                Ok(())
                            })?;
                        }
                    }

                    self.push_from_register(&array)?;
                    self.register_allocator.dealloc(array, &mut self.control)?;

                    self.bytecode.emit_call_spread(&mut self.control)?;
                } else {
                    for arg in args {
                        self.control.admit(CompilerWork::Visit)?;
                        self.compile_expr_to_stack(arg)?;
                    }
                    self.bytecode.emit_call(
                        u32::try_from(args.len())
                            .map_err(|_| size_overflow())?
                            .into(),
                        &mut self.control,
                    )?;
                }

                self.pop_into_register(value)?;
                self.bytecode
                    .emit_store_undefined(this.variable(), &mut self.control)?;
            }
        }
        Ok(())
    }

    /// Compile a [`VarDeclaration`].
    fn compile_var_decl(&mut self, decl: &VarDeclaration) -> JsResult<()> {
        for variable in decl.0.as_ref() {
            self.control.admit(CompilerWork::Visit)?;
            match variable.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident.try_to_js_string(&*self.interner, &mut self.control)?;
                    if let Some(expr) = variable.init() {
                        let binding = self.lexical_identifier_reference(ident.clone())?;
                        let index = self.insert_binding(binding)?;
                        let value = self.register_allocator.alloc(&mut self.control)?;
                        self.emit_binding_access(BindingAccessOpcode::GetLocator, &index, &value)?;
                        self.compile_expr(expr, &value)?;
                        self.emit_binding_access(
                            BindingAccessOpcode::SetNameByLocator,
                            &index,
                            &value,
                        )?;
                        self.register_allocator.dealloc(value, &mut self.control)?;
                    } else {
                        let value = self.register_allocator.alloc(&mut self.control)?;
                        self.emit_binding(BindingOpcode::Var, ident, &value)?;
                        self.register_allocator.dealloc(value, &mut self.control)?;
                    }
                }
                Binding::Pattern(pattern) => {
                    let value = self.register_allocator.alloc(&mut self.control)?;
                    if let Some(init) = variable.init() {
                        self.compile_expr(init, &value)?;
                    } else {
                        self.bytecode
                            .emit_store_undefined(value.variable(), &mut self.control)?;
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar, &value)?;
                    self.register_allocator.dealloc(value, &mut self.control)?;
                }
            }
        }
        Ok(())
    }

    /// Compile a [`LexicalDeclaration`].
    fn compile_lexical_decl(&mut self, decl: &LexicalDeclaration) -> JsResult<()> {
        match decl {
            LexicalDeclaration::Let(decls) => {
                for variable in decls.as_ref() {
                    self.control.admit(CompilerWork::Visit)?;
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;
                            let binding = self.lexical_identifier_reference(ident)?;
                            if binding.local() {
                                // Pre-allocate the persistent register but do NOT
                                // insert into local_binding_registers yet, so that
                                // TDZ checks still fire during initializer compilation.
                                let reg = self
                                    .register_allocator
                                    .alloc_persistent(&mut self.control)?;
                                if let Some(init) = variable.init() {
                                    self.compile_expr(init, &reg)?;
                                } else {
                                    self.bytecode
                                        .emit_store_undefined(reg.variable(), &mut self.control)?;
                                }
                                self.local_binding_registers.insert(
                                    binding,
                                    reg.index(),
                                    &mut self.control,
                                )?;
                            } else {
                                let value = self.register_allocator.alloc(&mut self.control)?;
                                if let Some(init) = variable.init() {
                                    self.compile_expr(init, &value)?;
                                } else {
                                    self.bytecode.emit_store_undefined(
                                        value.variable(),
                                        &mut self.control,
                                    )?;
                                }
                                let binding_kind = self.insert_binding(binding)?;
                                self.emit_binding_access(
                                    BindingAccessOpcode::PutLexicalValue,
                                    &binding_kind,
                                    &value,
                                )?;
                                self.register_allocator.dealloc(value, &mut self.control)?;
                            }
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }
                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            )?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                    }
                }
            }
            LexicalDeclaration::Const(decls) => {
                for variable in decls.as_ref() {
                    self.control.admit(CompilerWork::Visit)?;
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;
                            let init = variable
                                .init()
                                .expect("const declaration must have initializer");
                            let binding = self.lexical_identifier_reference(ident.clone())?;
                            if binding.local() {
                                let reg = self
                                    .register_allocator
                                    .alloc_persistent(&mut self.control)?;
                                self.compile_expr(init, &reg)?;
                                self.local_binding_registers.insert(
                                    binding,
                                    reg.index(),
                                    &mut self.control,
                                )?;
                            } else {
                                let value = self.register_allocator.alloc(&mut self.control)?;
                                self.compile_expr(init, &value)?;
                                let binding_kind = self.insert_binding(binding.clone())?;
                                self.emit_binding_access(
                                    BindingAccessOpcode::PutLexicalValue,
                                    &binding_kind,
                                    &value,
                                )?;
                                // Cache non-local const bindings in a persistent register
                                // so subsequent reads avoid GetName environment lookups.
                                let cache_reg = self
                                    .register_allocator
                                    .alloc_persistent(&mut self.control)?;
                                self.bytecode.emit_move(
                                    cache_reg.variable(),
                                    value.variable(),
                                    &mut self.control,
                                )?;
                                self.const_binding_cache.insert(
                                    binding.locator(),
                                    cache_reg.index(),
                                    &mut self.control,
                                )?;
                                self.register_allocator.dealloc(value, &mut self.control)?;
                            }
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc(&mut self.control)?;
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }
                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            )?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                    }
                }
            }
            LexicalDeclaration::Using(decls) => {
                // For each using declaration, we need to:
                // 1. Evaluate the initializer
                // 2. Add the resource to the disposal stack
                // 3. Bind the variable
                for variable in decls.as_ref() {
                    self.control.admit(CompilerWork::Visit)?;
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;
                            let value = self.register_allocator.alloc(&mut self.control)?;

                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }

                            // TODO(@abhinavs1920): Add resource to disposal stack
                            // For now, we just bind the variable like a let declaration
                            // Full implementation will add: AddDisposableResource opcode

                            self.emit_binding(BindingOpcode::InitLexical, ident, &value)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc(&mut self.control)?;

                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }

                            // TODO: Same as above

                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            )?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                    }
                }
            }
            LexicalDeclaration::AwaitUsing(decls) => {
                for variable in decls.as_ref() {
                    self.control.admit(CompilerWork::Visit)?;
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident =
                                ident.try_to_js_string(&*self.interner, &mut self.control)?;
                            let value = self.register_allocator.alloc(&mut self.control)?;

                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }

                            // TODO: Add resource to async disposal stack
                            // For now, we just bind the variable like a let declaration
                            // Full implementation will add: AddAsyncDisposableResource opcode

                            self.emit_binding(BindingOpcode::InitLexical, ident, &value)?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc(&mut self.control)?;

                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value)?;
                            } else {
                                self.bytecode
                                    .emit_store_undefined(value.variable(), &mut self.control)?;
                            }

                            // TODO: SAME
                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            )?;
                            self.register_allocator.dealloc(value, &mut self.control)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Compile a [`StatementListItem`].
    fn compile_stmt_list_item(
        &mut self,
        item: &StatementListItem,
        use_expr: bool,
        block: bool,
    ) -> JsResult<()> {
        self.control.admit(CompilerWork::Visit)?;
        match item {
            StatementListItem::Statement(stmt) => {
                self.compile_stmt(stmt, use_expr, false)?;
            }
            StatementListItem::Declaration(decl) => self.compile_decl(decl, block)?,
        }
        Ok(())
    }

    /// Compile a [`Declaration`].
    #[allow(unused_variables)]
    pub fn compile_decl(&mut self, decl: &Declaration, block: bool) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        match decl {
            #[cfg(feature = "annex-b")]
            Declaration::FunctionDeclaration(function) if block => {
                let identifier = function.name();
                let name = identifier.try_to_js_string(&*self.interner, &mut self.control)?;
                let mut enabled = false;
                if self.script_global_annex {
                    for candidate in &self.annex_b_candidates {
                        self.control.admit(CompilerWork::Search)?;
                        if candidate
                            .as_str()
                            .try_eq(name.as_str(), &mut |work| {
                                self.control.admit(CompilerWork::String(work))
                            })
                            .map_err(control::string_error)?
                        {
                            enabled = true;
                            break;
                        }
                    }
                } else {
                    for candidate in &self.annex_b_function_names {
                        self.control.admit(CompilerWork::Search)?;
                        self.control.admit(CompilerWork::Name(
                            surgeist_js_ast::operations::NameWork::Comparison,
                        ))?;
                        if *candidate == identifier.sym() {
                            enabled = true;
                            break;
                        }
                    }
                }
                if enabled {
                    let binding = self.lexical_identifier_reference(name.clone())?;
                    let index = self.get_binding(&binding)?;

                    let value = self.register_allocator.alloc(&mut self.control)?;
                    self.emit_binding_access(BindingAccessOpcode::GetName, &index, &value)?;
                    if self.script_global_annex {
                        let name_index = self.get_or_insert_string(name)?;
                        self.bytecode.emit_set_annex_b_global(
                            value.variable(),
                            name_index.into(),
                            &mut self.control,
                        )?;
                    } else {
                        match self.variable_set_mutable_binding_var(name.clone())? {
                            Ok(binding) => {
                                let index = self.get_binding(&binding)?;
                                self.emit_binding_access(
                                    BindingAccessOpcode::SetName,
                                    &index,
                                    &value,
                                )?;
                            }
                            Err(BindingLocatorError::MutateImmutable) => {
                                let index = self.get_or_insert_string(name)?;
                                self.bytecode
                                    .emit_throw_mutate_immutable(index.into(), &mut self.control)?;
                            }
                            Err(BindingLocatorError::Silent) => {}
                        }
                    }
                    self.register_allocator.dealloc(value, &mut self.control)?;
                }
            }
            Declaration::ClassDeclaration(class) => {
                self.compile_class(class.as_ref().into(), None)?
            }
            Declaration::Lexical(lexical) => self.compile_lexical_decl(lexical)?,
            _ => {}
        }
        Ok(())
    }

    /// Compiles a function AST Node into bytecode, and returns its index into
    /// the `functions` array.
    pub(crate) fn function(&mut self, function: FunctionSpec<'_>) -> JsResult<u32> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );

        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            name_scope,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            Some(
                name.sym()
                    .try_to_js_string(&*self.interner, &mut self.control)?,
            )
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .in_with(self.in_with)
            .name_scope(name_scope.cloned())
            .source_path(self.source_path.clone())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
                &mut *self.scopes,
                self.control.reborrow(),
            )?;

        self.push_function_to_constants(code)
    }

    /// Compiles a function AST Node into bytecode, setting its corresponding binding or
    /// pushing it to the stack if necessary.
    pub(crate) fn function_with_binding(
        &mut self,
        function: FunctionSpec<'_>,
        node_kind: NodeKind,
        dst: &Register,
    ) -> JsResult<()> {
        let name = function.name;
        let index = self.function(function)?;
        self.emit_get_function(dst, index)?;
        match node_kind {
            NodeKind::Declaration => {
                let name = name
                    .expect("function declaration must have a name")
                    .try_to_js_string(&*self.interner, &mut self.control)?;
                self.emit_binding(BindingOpcode::InitVar, name, dst)?;
            }
            NodeKind::Expression => {}
        }
        Ok(())
    }

    /// Compile an object method AST Node into bytecode.
    pub(crate) fn object_method(
        &mut self,
        function: FunctionSpec<'_>,
        kind: MethodKind,
    ) -> JsResult<Register> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            name_scope,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            let name = name
                .sym()
                .try_to_js_string(&*self.interner, &mut self.control)?;
            match kind {
                MethodKind::Ordinary => Some(name),
                MethodKind::Get => Some(
                    JsString::try_concat(js_str!("get "), name.as_str(), &mut |work| {
                        self.control.admit(CompilerWork::String(work))
                    })
                    .map_err(control::string_error)?,
                ),
                MethodKind::Set => Some(
                    JsString::try_concat(js_str!("set "), name.as_str(), &mut |work| {
                        self.control.admit(CompilerWork::String(work))
                    })
                    .map_err(control::string_error)?,
                ),
            }
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .method(true)
            .in_with(self.in_with)
            .name_scope(name_scope.cloned())
            .source_path(self.source_path.clone())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
                &mut *self.scopes,
                self.control.reborrow(),
            )?;

        let index = self.push_function_to_constants(code)?;
        let dst = self.register_allocator.alloc(&mut self.control)?;
        self.emit_get_function(&dst, index)?;
        Ok(dst)
    }

    /// Compile a class method AST Node into bytecode.
    fn method(&mut self, function: FunctionSpec<'_>) -> JsResult<Register> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            Some(
                name.sym()
                    .try_to_js_string(&*self.interner, &mut self.control)?,
            )
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(true)
            .arrow(arrow)
            .method(true)
            .in_with(self.in_with)
            .name_scope(function.name_scope.cloned())
            .source_path(self.source_path.clone())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
                &mut *self.scopes,
                self.control.reborrow(),
            )?;

        let index = self.push_function_to_constants(code)?;
        let dst = self.register_allocator.alloc(&mut self.control)?;
        self.emit_get_function(&dst, index)?;
        Ok(dst)
    }

    fn call(&mut self, callable: Callable<'_>, dst: CallResultDest<'_>) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        #[derive(PartialEq)]
        enum CallKind {
            CallEval,
            Call,
            New,
        }

        let (call, mut kind) = match callable {
            Callable::Call(call) => (call, CallKind::Call),
            Callable::New(new) => (new.call(), CallKind::New),
        };

        match self.flatten_expression(call.function())? {
            Expression::PropertyAccess(access) if kind == CallKind::Call => {
                let this = self.register_allocator.alloc(&mut self.control)?;
                let dst = self.register_allocator.alloc(&mut self.control)?;
                self.compile_access_preserve_this(access, &this, &dst)?;
                self.push_from_register(&this)?;
                self.push_from_register(&dst)?;
                self.register_allocator.dealloc(this, &mut self.control)?;
                self.register_allocator.dealloc(dst, &mut self.control)?;
            }
            Expression::Optional(opt) if kind == CallKind::Call => {
                let this = self.register_allocator.alloc(&mut self.control)?;
                let dst = self.register_allocator.alloc(&mut self.control)?;
                self.compile_optional_preserve_this(opt, &this, &dst)?;
                self.push_from_register(&this)?;
                self.push_from_register(&dst)?;
                self.register_allocator.dealloc(this, &mut self.control)?;
                self.register_allocator.dealloc(dst, &mut self.control)?;
            }
            expr if kind == CallKind::Call => {
                if let Expression::Identifier(ident) = expr {
                    if ident.sym() == Sym::EVAL {
                        kind = CallKind::CallEval;
                    }

                    if self.in_with {
                        let name = self.resolve_identifier_expect(*ident)?;
                        let binding = self.lexical_identifier_reference(name)?;
                        let index = self.get_binding(&binding)?;
                        let index = match index {
                            BindingKind::Global(index) | BindingKind::Stack(index) => index,
                            BindingKind::Local(_) => {
                                unreachable!("with binding cannot be local")
                            }
                        };
                        let value = self.register_allocator.alloc(&mut self.control)?;
                        self.bytecode.emit_this_for_object_environment_name(
                            value.variable(),
                            index.into(),
                            &mut self.control,
                        )?;
                        self.push_from_register(&value)?;
                        self.register_allocator.dealloc(value, &mut self.control)?;
                    } else {
                        self.push_from_register(&CallFrame::undefined_register())?;
                    }
                } else {
                    self.push_from_register(&CallFrame::undefined_register())?;
                }

                self.compile_expr_to_stack(expr)?;
            }
            expr => {
                let value = self.register_allocator.alloc(&mut self.control)?;
                self.compile_expr(expr, &value)?;
                self.push_from_register(&CallFrame::undefined_register())?;
                self.push_from_register(&value)?;
                self.register_allocator.dealloc(value, &mut self.control)?;
            }
        }

        let mut source_position = self.position_guard(call)?;
        let compiler = &mut *source_position;

        let contains_spread = compiler.arguments_contain_spread(call.args())?;

        if contains_spread {
            let array = compiler.register_allocator.alloc(&mut compiler.control)?;
            let array_op = array.variable();

            compiler
                .bytecode
                .emit_store_new_array(array_op, &mut compiler.control)?;

            for arg in call.args() {
                compiler.control.admit(CompilerWork::Visit)?;
                if let Expression::Spread(spread) = arg {
                    let value = compiler.register_allocator.alloc(&mut compiler.control)?;
                    compiler.compile_expr(spread.target(), &value)?;
                    compiler
                        .bytecode
                        .emit_get_iterator(value.variable(), &mut compiler.control)?;
                    compiler
                        .bytecode
                        .emit_push_iterator_to_array(array_op, &mut compiler.control)?;
                    compiler
                        .register_allocator
                        .dealloc(value, &mut compiler.control)?;
                } else {
                    compiler.compile_expr_operand(arg, |this, op| {
                        this.bytecode
                            .emit_push_value_to_array(op, array_op, &mut this.control)?;
                        Ok(())
                    })?;
                }
            }

            compiler.push_from_register(&array)?;
            compiler
                .register_allocator
                .dealloc(array, &mut compiler.control)?;
        } else {
            for arg in call.args() {
                compiler.control.admit(CompilerWork::Visit)?;
                compiler.compile_expr_to_stack(arg)?;
            }
        }

        match kind {
            CallKind::CallEval => {
                let lexical_scope = compiler.lexical_scope.clone();
                let scope_index = compiler.push_scope_constant(&lexical_scope)?;
                if contains_spread {
                    compiler
                        .bytecode
                        .emit_call_eval_spread(scope_index.into(), &mut compiler.control)?;
                } else {
                    compiler.bytecode.emit_call_eval(
                        u32::try_from(call.args().len())
                            .map_err(|_| size_overflow())?
                            .into(),
                        scope_index.into(),
                        &mut compiler.control,
                    )?;
                }
            }
            CallKind::Call if contains_spread => {
                compiler.bytecode.emit_call_spread(&mut compiler.control)?
            }
            CallKind::Call => {
                compiler.bytecode.emit_call(
                    u32::try_from(call.args().len())
                        .map_err(|_| size_overflow())?
                        .into(),
                    &mut compiler.control,
                )?;
            }
            CallKind::New if contains_spread => {
                compiler.bytecode.emit_new_spread(&mut compiler.control)?
            }
            CallKind::New => compiler.bytecode.emit_new(
                u32::try_from(call.args().len())
                    .map_err(|_| size_overflow())?
                    .into(),
                &mut compiler.control,
            )?,
        }
        match dst {
            CallResultDest::Register(dst) => compiler.pop_into_register(dst)?,
            CallResultDest::Discard => compiler.bytecode.emit_pop(&mut compiler.control)?,
            CallResultDest::Stack => {} // leave result on stack
        }
        source_position.finish()?;
        Ok(())
    }

    /// Finish compiling code with the [`ByteCompiler`] and return the generated [`CodeBlock`].
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    pub fn finish(mut self, parameters: Option<&FormalParameterList>) -> JsResult<CodeBlock> {
        let scope_association = self.with_scopes(|access| Ok(access.root_handle()?))?;
        if let Some(async_handler) = self.async_handler {
            self.patch_handler(async_handler)?;
        }
        self.r#return(false)?;
        let final_bytecode_len = self.next_opcode_location()?;
        self.control.admit(CompilerWork::MetadataVisit)?;
        let parameter_length = parameters
            .map(|formals| u32::try_from(formals.as_ref().len()).map_err(|_| size_overflow()))
            .transpose()?
            .unwrap_or(0);
        let mapped_arguments_binding_indices = if self.emitted_mapped_arguments_object_opcode {
            mapped_arguments::binding_indices(
                parameters.expect("mapped arguments require original function parameters"),
                self.interner,
                &self.parameter_scope,
                self.scopes,
                &mut self.control,
            )?
        } else {
            ThinVec::default()
        };
        let register_count = self.register_allocator.finish(&mut self.control)?;
        let source_map_entries = self
            .source_map_builder
            .build(final_bytecode_len.as_u32(), &mut self.control)?;
        let bytecode = self.bytecode.into_bytecode(&mut self.control)?;
        let bindings = boxed_metadata(self.bindings, &mut self.control)?;
        let ic = boxed_metadata(self.ic, &mut self.control)?;
        let global_fns = boxed_metadata(self.global_fns, &mut self.control)?;
        let global_vars = boxed_metadata(self.global_vars, &mut self.control)?;
        let annex_b_candidates = boxed_metadata(self.annex_b_candidates, &mut self.control)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        // All refusal-capable preparation precedes the shared debug identity.
        Ok(CodeBlock {
            scope_association,
            length: self.length,
            register_count,
            this_mode: self.this_mode,
            parameter_length,
            mapped_arguments_binding_indices,
            bytecode,
            constants: self.constants,
            bindings,
            handlers: self.handlers,
            flags: Cell::new(self.code_block_flags),
            ic,
            source_info: SourceInfo::new(
                SourceMap::new(source_map_entries, self.source_path),
                self.function_name,
                self.spanned_source_text,
            ),
            global_fns,
            global_vars,
            annex_b_candidates,
            debug_id: CodeBlock::get_next_codeblock_id(),
            #[cfg(feature = "trace")]
            traced: Cell::new(false),
        })
    }

    fn compile_declaration_pattern(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
        object: &Register,
    ) -> JsResult<()> {
        let _depth = self.control.enter()?;
        self.control.admit(CompilerWork::Visit)?;
        self.compile_declaration_pattern_impl(pattern, def, object)?;
        Ok(())
    }
}

fn boxed_metadata<T>(values: Vec<T>, control: &mut CompilerControl<'_>) -> JsResult<Box<[T]>> {
    let bytes = payload_bytes::<T>(values.len())?;
    if values.len() != values.capacity() {
        control.admit(CompilerWork::Capacity { bytes })?;
        control.admit(CompilerWork::Relocate { bytes })?;
    }
    control.admit(CompilerWork::MetadataWrite)?;
    Ok(values.into_boxed_slice())
}
