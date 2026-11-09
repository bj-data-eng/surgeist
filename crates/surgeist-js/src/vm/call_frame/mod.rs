//! `CallFrame`
//!
//! This module will provides everything needed to implement the `CallFrame`

use std::rc::Rc;

use super::{ActivatedCode, ActiveRunnable, StoredActiveRunnable};
use crate::{
    JsError, JsResult, JsValue,
    builtins::iterable::{IteratorRecord, StoredIteratorRecord},
    bytecompiler::Register,
    environments::{EnvironmentStack, StoredEnvironmentStack},
    error::StoredJsError,
    realm::{Realm, RealmEdge},
    value::StoredValue,
    vm::{CodeBlock, SourcePath},
};
use surgeist_js_ast::Position;
use surgeist_js_ast::scope::{BindingLocator, Scope};
use surgeist_js_string::JsString;
use thin_vec::ThinVec;

bitflags::bitflags! {
    /// Flags associated with a [`CallFrame`].
    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct CallFrameFlags: u8 {
        /// When we return from this [`CallFrame`] to stop execution and
        /// return from [`crate::Context::run()`], and leave the remaining [`CallFrame`]s unchanged.
        const EXIT_EARLY = 0b0000_0001;

        /// Was this [`CallFrame`] created from the `__construct__()` internal object method?
        const CONSTRUCT = 0b0000_0010;

        /// Does this [`CallFrame`] need to push registers on [`Vm::push_frame()`].
        const REGISTERS_ALREADY_PUSHED = 0b0000_0100;

        /// If the `this` value has been cached.
        const THIS_VALUE_CACHED = 0b0000_1000;
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CallFrameLocation {
    pub function_name: JsString,
    pub path: SourcePath,
    pub position: Option<Position>,
}

/// A logical invocation identity independent of frame storage addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvocationId(u64);

impl InvocationId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Admission for one dispatch of one logical invocation, consumed by its frame.
#[derive(Debug)]
pub(crate) struct FirstDispatchPermit {
    invocation: InvocationId,
    pc: u32,
    code: Rc<CodeBlock>,
}

impl FirstDispatchPermit {
    pub(super) fn new(invocation: InvocationId, pc: u32, code: Rc<CodeBlock>) -> Self {
        Self {
            invocation,
            pc,
            code,
        }
    }

    pub(crate) fn validate_for(&self, code: &Rc<CodeBlock>, pc: u32) -> JsResult<()> {
        if !Rc::ptr_eq(&self.code, code) || self.pc != pc {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(())
    }

    pub(super) fn validate_dispatch(self, frame: &CallFrame) -> JsResult<()> {
        self.validate_for(&frame.code_block, frame.pc)?;
        if self.invocation != frame.invocation {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum FrameExecution {
    Bytecode(ActivatedCode),
    Host,
    HostStart,
}

/// Custody of a frame's operand/register segment is independent of its lifetime.
/// An enclosing frame remains on the VM while an inline reaction runs its capture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StackCustody {
    /// The invocation may suspend and publish its bytecode continuation.
    Invocation,
    /// A capture owns the segment; this enclosing frame may only unwind.
    Transferred,
    /// A native coroutine owns copied ambient storage, not bytecode resumption.
    #[cfg(feature = "experimental")]
    NativeSnapshot,
}

/// A `CallFrame` holds the state of a function call.
#[derive(Debug)]
pub struct CallFrame {
    pub(crate) code_block: Rc<CodeBlock>,
    execution: FrameExecution,
    invocation: InvocationId,
    first_dispatch: Option<FirstDispatchPermit>,
    stack_custody: StackCustody,
    await_origin: Option<crate::vm::opcode::AwaitRoot>,
    await_finish: Option<crate::vm::opcode::AwaitRoot>,
    pub(crate) pc: u32,
    /// The frame pointer, points to the start of this frame's data in the stack
    /// (i.e., the `this` value position).
    pub(crate) fp: u32,
    pub(crate) argument_count: u32,
    pub(crate) env_fp: u32,

    /// The register pointer, points to the start of this frame's registers on the stack.
    pub(crate) rp: u32,

    pub(crate) return_value: JsValue,

    // An exception being handled by this frame, including across suspension.
    // An empty exception can also represent generator return through finally.
    pub(crate) pending_exception: Option<Box<JsError>>,

    // Iterators and their `[[Done]]` flags that must be closed when an abrupt completion is thrown.
    pub(crate) iterators: ThinVec<IteratorRecord>,

    // The stack of bindings being updated.
    pub(crate) binding_stack: ThinVec<BindingLocator>,

    /// Global block-function names enabled by this execution's Annex B preflight.
    annex_b_eligible_names: Vec<JsString>,

    /// How many iterations a loop has done.
    pub(crate) loop_iteration_count: u64,

    /// `[[ScriptOrModule]]`
    pub(crate) active_runnable: Option<ActiveRunnable>,
    pub(crate) module_evaluation: Option<Rc<crate::module::lifecycle::EvaluationInvocation>>,

    /// \[\[Environment\]\]
    pub(crate) environments: EnvironmentStack,

    /// \[\[Realm\]\]
    pub(crate) realm: Realm,

    pub(crate) flags: CallFrameFlags,
}

/// The suspended state of a frame. Its owner traces these edges.
#[derive(Debug, Clone, surgeist_js_gc::heap::Trace)]
pub(crate) struct StoredCallFrame {
    #[heap_trace(skip)]
    code_block: Rc<CodeBlock>,
    #[heap_trace(skip)]
    execution: FrameExecution,
    #[heap_trace(skip)]
    invocation: InvocationId,
    #[heap_trace(skip)]
    stack_custody: StackCustody,
    pc: u32,
    fp: u32,
    argument_count: u32,
    env_fp: u32,
    rp: u32,
    return_value: StoredValue,
    pending_exception: Option<Box<StoredJsError>>,
    iterators: ThinVec<StoredIteratorRecord>,
    #[heap_trace(skip)]
    binding_stack: ThinVec<BindingLocator>,
    #[heap_trace(skip)]
    annex_b_eligible_names: Vec<JsString>,
    loop_iteration_count: u64,
    active_runnable: Option<StoredActiveRunnable>,
    module_evaluation: Option<Rc<crate::module::lifecycle::EvaluationInvocation>>,
    environments: StoredEnvironmentStack,
    realm: RealmEdge,
    #[heap_trace(skip)]
    flags: CallFrameFlags,
}

impl StoredCallFrame {
    pub(crate) fn to_frame(&self) -> CallFrame {
        CallFrame {
            code_block: self.code_block.clone(),
            execution: self.execution.clone(),
            module_evaluation: self.module_evaluation.clone(),
            invocation: self.invocation,
            first_dispatch: None,
            stack_custody: self.stack_custody,
            await_origin: None,
            await_finish: None,
            pc: self.pc,
            fp: self.fp,
            argument_count: self.argument_count,
            env_fp: self.env_fp,
            rp: self.rp,
            return_value: self.return_value.to_value(),
            pending_exception: self
                .pending_exception
                .as_deref()
                .map(|error| Box::new(error.to_error())),
            iterators: self
                .iterators
                .iter()
                .map(StoredIteratorRecord::to_iterator)
                .collect(),
            binding_stack: self.binding_stack.clone(),
            annex_b_eligible_names: self.annex_b_eligible_names.clone(),
            loop_iteration_count: self.loop_iteration_count,
            active_runnable: self
                .active_runnable
                .as_ref()
                .map(StoredActiveRunnable::to_runnable),
            environments: self.environments.to_stack(),
            realm: Realm::from_edge(self.realm.clone()),
            flags: self.flags,
        }
    }
}

impl CallFrame {
    pub(crate) fn invocation(&self) -> InvocationId {
        self.invocation
    }
    pub(crate) fn await_origin(&self) -> Option<&crate::vm::opcode::AwaitRoot> {
        self.await_origin.as_ref()
    }
    pub(crate) fn set_await_origin(&mut self, root: crate::vm::opcode::AwaitRoot) -> JsResult<()> {
        if root.invocation() != self.invocation {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        self.await_origin = Some(root);
        self.await_finish = None;
        Ok(())
    }
    pub(crate) fn set_await_finish(
        &mut self,
        invocation: InvocationId,
        root: crate::vm::opcode::AwaitRoot,
    ) -> JsResult<()> {
        if invocation != self.invocation || root.invocation() != invocation {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        self.await_finish = Some(root);
        Ok(())
    }
    pub(crate) fn continuation_finish(&self) -> JsResult<crate::vm::opcode::ContinuationFinish> {
        if self.owns_continuation() {
            return Ok(crate::vm::opcode::ContinuationFinish::Owning);
        }
        self.await_finish.as_ref().map_or(
            Ok(crate::vm::opcode::ContinuationFinish::Unavailable),
            crate::vm::opcode::AwaitRoot::finish,
        )
    }
    pub(crate) fn owns_continuation(&self) -> bool {
        self.stack_custody == StackCustody::Invocation
    }

    pub(crate) fn has_stack(&self) -> bool {
        self.stack_custody != StackCustody::Transferred
    }

    pub(crate) fn check_suspension(&self) -> JsResult<()> {
        if !self.owns_continuation() || self.first_dispatch.is_some() {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(())
    }
    pub(crate) fn transfer_stack(&mut self) -> JsResult<()> {
        self.check_suspension()?;
        self.stack_custody = StackCustody::Transferred;
        Ok(())
    }

    /// Reclaims a capture that was never consumed by its reaction.
    pub(crate) fn restore_stack(&mut self) -> JsResult<()> {
        if self.has_stack() {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        self.stack_custody = StackCustody::Invocation;
        self.await_finish = None;
        Ok(())
    }

    #[cfg(feature = "experimental")]
    pub(crate) fn mark_native_snapshot(&mut self) {
        assert!(self.has_stack(), "cannot copy a transferred stack");
        self.stack_custody = StackCustody::NativeSnapshot;
    }

    pub(crate) fn set_module_evaluation(
        &mut self,
        evaluation: crate::module::lifecycle::EvaluationRef,
    ) {
        evaluation.start_invocation(self.invocation);
        self.module_evaluation = Some(Rc::new(crate::module::lifecycle::EvaluationInvocation {
            evaluation,
            invocation: self.invocation,
        }));
    }

    pub(crate) fn stored(&self) -> StoredCallFrame {
        assert!(
            self.first_dispatch.is_none(),
            "cannot suspend before first dispatch"
        );
        StoredCallFrame {
            code_block: self.code_block.clone(),
            execution: self.execution.clone(),
            module_evaluation: self.module_evaluation.clone(),
            invocation: self.invocation,
            stack_custody: self.stack_custody,
            pc: self.pc,
            fp: self.fp,
            argument_count: self.argument_count,
            env_fp: self.env_fp,
            rp: self.rp,
            return_value: self.return_value.clone().into_stored(),
            pending_exception: self
                .pending_exception
                .as_deref()
                .cloned()
                .map(|error| Box::new(error.into_stored())),
            iterators: self.iterators.iter().map(IteratorRecord::stored).collect(),
            binding_stack: self.binding_stack.clone(),
            annex_b_eligible_names: self.annex_b_eligible_names.clone(),
            loop_iteration_count: self.loop_iteration_count,
            active_runnable: self.active_runnable.as_ref().map(ActiveRunnable::stored),
            environments: self.environments.stored(),
            realm: self.realm.edge(),
            flags: self.flags,
        }
    }
}

impl Clone for CallFrame {
    fn clone(&self) -> Self {
        assert!(
            self.first_dispatch.is_none(),
            "cannot clone an unused entry permit"
        );
        Self {
            code_block: self.code_block.clone(),
            execution: self.execution.clone(),
            module_evaluation: self.module_evaluation.clone(),
            invocation: self.invocation,
            first_dispatch: None,
            stack_custody: self.stack_custody,
            await_origin: self.await_origin.clone(),
            await_finish: self.await_finish.clone(),
            pc: self.pc,
            fp: self.fp,
            argument_count: self.argument_count,
            env_fp: self.env_fp,
            rp: self.rp,
            return_value: self.return_value.clone(),
            pending_exception: self.pending_exception.clone(),
            iterators: self.iterators.clone(),
            binding_stack: self.binding_stack.clone(),
            annex_b_eligible_names: self.annex_b_eligible_names.clone(),
            loop_iteration_count: self.loop_iteration_count,
            active_runnable: self.active_runnable.clone(),
            environments: self.environments.clone(),
            realm: self.realm.clone(),
            flags: self.flags,
        }
    }
}

/// ---- `CallFrame` public API ----
impl CallFrame {
    /// Retrieves the [`CodeBlock`] of this call frame.
    #[inline]
    #[must_use]
    pub const fn code_block(&self) -> &Rc<CodeBlock> {
        &self.code_block
    }

    /// Retrieves a tuple of `(`[`JsString`]`, `[`SourcePath`]`, `[`Position`]`)` to know the
    /// location of the call frame.
    #[inline]
    #[must_use]
    pub fn position(&self) -> CallFrameLocation {
        let source_info = &self.code_block.source_info;
        CallFrameLocation {
            function_name: source_info.function_name().clone(),
            path: source_info.map().path().clone(),
            position: source_info.map().find(self.pc),
        }
    }
}

/// ---- `CallFrame` creation methods ----
impl CallFrame {
    pub(crate) const FUNCTION_PROLOGUE: u32 = 2;
    pub(crate) const UNDEFINED_REGISTER_INDEX: usize = 0;
    pub(crate) const PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX: usize = 1;
    pub(crate) const PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX: usize = 2;
    pub(crate) const PROMISE_CAPABILITY_REJECT_REGISTER_INDEX: usize = 3;
    pub(crate) const ASYNC_GENERATOR_OBJECT_REGISTER_INDEX: usize = 4;

    pub(crate) fn undefined_register() -> Register {
        Register::persistent(Self::UNDEFINED_REGISTER_INDEX as u32)
    }

    pub(crate) fn promise_capability_promise_register() -> Register {
        Register::persistent(Self::PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX as u32)
    }

    pub(crate) fn promise_capability_resolve_register() -> Register {
        Register::persistent(Self::PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX as u32)
    }

    pub(crate) fn promise_capability_reject_register() -> Register {
        Register::persistent(Self::PROMISE_CAPABILITY_REJECT_REGISTER_INDEX as u32)
    }

    pub(crate) fn async_generator_object_register() -> Register {
        Register::persistent(Self::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX as u32)
    }

    /// Creates a new `CallFrame` with the provided `CodeBlock`.
    fn with_execution(
        code_block: Rc<CodeBlock>,
        execution: FrameExecution,
        invocation: InvocationId,
        first_dispatch: Option<FirstDispatchPermit>,
        active_runnable: Option<ActiveRunnable>,
        environments: EnvironmentStack,
        realm: Realm,
    ) -> Self {
        Self {
            execution,
            module_evaluation: None,
            invocation,
            first_dispatch,
            stack_custody: StackCustody::Invocation,
            await_origin: None,
            await_finish: None,
            pc: 0,
            fp: 0,
            env_fp: 0,
            argument_count: 0,
            rp: 0,
            return_value: JsValue::undefined(),
            pending_exception: None,
            iterators: ThinVec::new(),
            binding_stack: ThinVec::new(),
            annex_b_eligible_names: Vec::new(),
            code_block,
            loop_iteration_count: 0,
            active_runnable,
            environments,
            realm,
            flags: CallFrameFlags::empty(),
        }
    }

    /// Creates an executable frame only from installed code and its accepted entry.
    pub(crate) fn new(
        code: ActivatedCode,
        permit: FirstDispatchPermit,
        active_runnable: Option<ActiveRunnable>,
        environments: EnvironmentStack,
        realm: Realm,
    ) -> JsResult<Self> {
        permit.validate_for(code.code(), 0)?;
        let invocation = permit.invocation;
        Ok(Self::with_execution(
            code.code().clone(),
            FrameExecution::Bytecode(code),
            invocation,
            Some(permit),
            active_runnable,
            environments,
            realm,
        ))
    }

    pub(crate) fn host(
        code: Rc<CodeBlock>,
        active_runnable: Option<ActiveRunnable>,
        environments: EnvironmentStack,
        realm: Realm,
    ) -> Self {
        Self::with_execution(
            code,
            FrameExecution::Host,
            InvocationId(0),
            None,
            active_runnable,
            environments,
            realm,
        )
    }

    /// Pending code supplies source metadata, never executable authority.
    pub(crate) fn host_start(
        code: Rc<CodeBlock>,
        active_runnable: Option<ActiveRunnable>,
        environments: EnvironmentStack,
        realm: Realm,
    ) -> Self {
        Self::with_execution(
            code,
            FrameExecution::HostStart,
            InvocationId(0),
            None,
            active_runnable,
            environments,
            realm,
        )
    }

    /// Host and declaration-start frames have metadata but no bytecode stack slots.
    pub(crate) fn is_host(&self) -> bool {
        !matches!(self.execution, FrameExecution::Bytecode(_))
    }

    pub(crate) fn require_executable(&self) -> JsResult<()> {
        if self.is_host() {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(())
    }

    pub(super) fn require_host_start(&self) -> JsResult<()> {
        if !matches!(self.execution, FrameExecution::HostStart) {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(())
    }

    pub(super) fn take_first_dispatch(&mut self) -> Option<FirstDispatchPermit> {
        self.first_dispatch.take()
    }

    pub(crate) fn constant_scope(&self, index: usize) -> JsResult<Scope> {
        let FrameExecution::Bytecode(code) = &self.execution else {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        };
        code.constant_scope(index)
    }

    /// Sets the global block-function names enabled for this execution.
    pub(crate) fn with_annex_b_eligibility(mut self, names: Vec<JsString>) -> Self {
        self.annex_b_eligible_names = names;
        self
    }

    pub(crate) fn annex_b_global_eligible(&self, name: &JsString) -> bool {
        self.annex_b_eligible_names.contains(name)
    }

    /// Updates a `CallFrame`'s `argument_count` field with the value provided.
    pub(crate) fn with_argument_count(mut self, count: u32) -> Self {
        self.argument_count = count;
        self
    }

    /// Updates a `CallFrame`'s `env_fp` field with the value provided.
    pub(crate) fn with_env_fp(mut self, env_fp: u32) -> Self {
        self.env_fp = env_fp;
        self
    }

    /// Updates a `CallFrame`'s `flags` field with the value provided.
    pub(crate) fn with_flags(mut self, flags: CallFrameFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Returns the index of `this` in the stack.
    pub(crate) fn this_index(&self) -> usize {
        self.fp as usize
    }

    /// Returns the index of the function in the stack.
    pub(crate) fn function_index(&self) -> usize {
        self.fp as usize + 1
    }

    /// Returns the range of the arguments in the stack.
    pub(crate) fn arguments_range(&self) -> std::ops::Range<usize> {
        let start = self.fp as usize + Self::FUNCTION_PROLOGUE as usize;
        start..start + self.argument_count as usize
    }

    /// Returns the frame pointer of this `CallFrame`.
    pub(crate) fn frame_pointer(&self) -> usize {
        self.fp as usize
    }

    /// Does this have the [`CallFrameFlags::EXIT_EARLY`] flag.
    pub(crate) fn exit_early(&self) -> bool {
        self.flags.contains(CallFrameFlags::EXIT_EARLY)
    }

    /// Set the [`CallFrameFlags::EXIT_EARLY`] flag.
    pub(crate) fn set_exit_early(&mut self, early_exit: bool) {
        self.flags.set(CallFrameFlags::EXIT_EARLY, early_exit);
    }

    /// Does this have the [`CallFrameFlags::CONSTRUCT`] flag.
    pub(crate) fn construct(&self) -> bool {
        self.flags.contains(CallFrameFlags::CONSTRUCT)
    }

    /// Does this [`CallFrame`] need to push registers on [`super::Vm::push_frame()`].
    pub(crate) fn registers_already_pushed(&self) -> bool {
        self.flags
            .contains(CallFrameFlags::REGISTERS_ALREADY_PUSHED)
    }

    /// Does this [`CallFrame`] have a cached `this` value.
    ///
    /// The cached value is placed in the `this` position.
    pub(crate) fn has_this_value_cached(&self) -> bool {
        self.flags.contains(CallFrameFlags::THIS_VALUE_CACHED)
    }
}

/// Indicates how a generator function that has been called/resumed should return.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum GeneratorResumeKind {
    #[default]
    Normal = 0,
    Throw,
    Return,
}

impl From<GeneratorResumeKind> for JsValue {
    fn from(value: GeneratorResumeKind) -> Self {
        Self::new(value as u8)
    }
}

impl JsValue {
    /// Convert value to [`GeneratorResumeKind`].
    ///
    /// # Panics
    ///
    /// If not a integer type or not in the range `0..=2`.
    #[track_caller]
    pub(crate) fn to_generator_resume_kind(&self) -> GeneratorResumeKind {
        if let Some(value) = self.as_i32() {
            match value {
                0 => return GeneratorResumeKind::Normal,
                1 => return GeneratorResumeKind::Throw,
                2 => return GeneratorResumeKind::Return,
                _ => unreachable!("generator kind must be an integer between 1..=2, got {value}"),
            }
        }

        unreachable!("generator kind must be an integer type")
    }
}

#[cfg(test)]
mod tests {
    use super::CallFrame;
    use crate::{Context, js_string, vm::CodeBlock};
    use std::rc::Rc;

    #[test]
    fn annex_eligibility_is_frame_local_and_survives_clone_and_suspension() {
        let context = Context::default();
        let name = js_string!("block_function");
        let frame = CallFrame::host(
            Rc::new(CodeBlock::new(js_string!("annex"), 0, false)),
            None,
            context.vm.frame().environments.clone(),
            context.realm().clone(),
        )
        .with_annex_b_eligibility(vec![name.clone()]);
        assert!(frame.annex_b_global_eligible(&name));
        assert!(frame.clone().annex_b_global_eligible(&name));
        assert!(frame.stored().to_frame().annex_b_global_eligible(&name));
        assert!(!context.vm.frame().annex_b_global_eligible(&name));
    }
}
