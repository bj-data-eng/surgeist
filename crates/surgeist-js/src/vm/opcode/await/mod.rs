use super::RegisterOperand;
use crate::module::lifecycle::{EvaluationInvocation, ModuleAction};
use crate::{
    Context, JsArgs, JsExpect, JsResult, JsValue,
    builtins::{
        Promise,
        async_generator::AsyncGenerator,
        generator::{GeneratorContext, StoredGeneratorContext},
        promise::PromiseCapability,
    },
    error::PanicError,
    js_string,
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    vm::{CompletionRecord, GeneratorResumeKind, opcode::Operation},
};
use std::{
    cell::RefCell,
    ops::ControlFlow,
    rc::{Rc, Weak},
};
use surgeist_js_gc::heap::{RecordRef, Trace};

#[derive(surgeist_js_gc::heap::Trace)]
struct AwaitState(Rc<AwaitData>);
#[derive(Trace)]
struct AwaitData {
    continuation: RefCell<Option<StoredGeneratorContext>>,
    evaluation: Option<Rc<EvaluationInvocation>>,
}
#[derive(Clone, Trace)]
pub(crate) struct AwaitRef {
    edge: RecordRef<AwaitState>,
    #[heap_trace(skip)]
    data: Weak<AwaitData>,
}
impl std::fmt::Debug for AwaitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Await").field(&self.edge.identity()).finish()
    }
}
impl AwaitRef {
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.edge == other.edge
    }
    pub(crate) fn retire(&self) {
        if let Some(data) = self.data.upgrade() {
            let owned = data.continuation.borrow_mut().take();
            drop(owned);
        }
    }
}

impl AwaitState {
    fn resume(
        &self,
        value: JsValue,
        kind: GeneratorResumeKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(evaluation) = &self.0.evaluation
            && let Some(cause) = evaluation.evaluation.cause()
        {
            let owned = self.0.continuation.borrow_mut().take();
            drop(owned);
            return Err(crate::error::EngineError::ModuleInterrupted(cause.external()).into());
        }
        if let Some(evaluation) = &self.0.evaluation {
            evaluation
                .evaluation
                .validate_invocation(evaluation.invocation)?;
        }
        let mut continuation = self
            .0
            .continuation
            .borrow_mut()
            .take()
            .js_expect("await reaction should only run once")?
            .to_context();
        let generator = continuation
            .async_generator_object()?
            .map(|object| {
                object
                    .downcast::<AsyncGenerator>()
                    .ok()
                    .js_expect("must be async generator")
            })
            .transpose()?;
        let result = continuation.resume(Some(value), kind, context);
        if let Some(generator) = generator {
            AsyncGenerator::finish_resume(&generator, &continuation, result)?;
        } else {
            result.consume()?;
        }
        Ok(JsValue::undefined())
    }
}

/// `Await` implements the Opcode Operation for `Opcode::Await`
///
/// Operation:
///  - Stops the current Async function and schedules it to resume later.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Await;

impl Await {
    #[inline(always)]
    pub(super) fn operation(
        value: RegisterOperand,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let value = context.vm.get_register(value.into());

        // 2. Let promise be ? PromiseResolve(%Promise%, value).
        let promise = match Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value.clone(),
            context,
        ) {
            Ok(promise) => match promise.downcast::<Promise>().ok() {
                Some(v) => v,
                None => {
                    return context.handle_error(
                        PanicError::new("%Promise% constructor must return a `Promise` object")
                            .into(),
                    );
                }
            },
            Err(err) => return context.handle_error(err),
        };

        let return_value = context
            .vm
            .get_promise_capability()
            .ok()
            .map(|cap| JsValue::from(cap.promise))
            .unwrap_or_default();

        let data = Rc::new(AwaitData {
            continuation: RefCell::new(None),
            evaluation: context.vm.frame().module_evaluation.clone(),
        });
        let captures = match context.runtime().heap.allocate(AwaitState(data.clone())) {
            Ok(captures) => captures,
            Err(_) => {
                return context.handle_error(crate::error::EngineResourceError::Allocation.into());
            }
        };
        let state = AwaitRef {
            edge: captures.record_ref(),
            data: Rc::downgrade(&data),
        };
        let action = data.evaluation.as_ref().map(|evaluation| {
            let Some(crate::vm::ActiveRunnable::Module(module)) =
                &context.vm.frame().active_runnable
            else {
                unreachable!("actual module invocation has its module")
            };
            evaluation.evaluation.suspended(state.clone());
            ModuleAction::resume(module, state.clone())
        });
        let r#gen = GeneratorContext::from_current(context, None);
        *data.continuation.borrow_mut() = Some(r#gen.stored());
        // Keep the detached stack rooted until its reactions are installed.
        // 3. Let fulfilledClosure be a new Abstract Closure with parameters (value) that captures asyncContext and performs the following steps when called:
        // 4. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 1, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, captures, context| {
                    if let Some(evaluation) = &captures.0.evaluation {
                        evaluation.evaluation.consume(&AwaitRef {
                            edge: captures.record_ref(),
                            data: Rc::downgrade(&captures.0),
                        });
                    }
                    captures.resume(
                        args.get_or_undefined(0).clone(),
                        GeneratorResumeKind::Normal,
                        context,
                    )
                },
                captures.clone(),
            ),
        )
        .name(js_string!())
        .length(1)
        .build()
        .expect("await reaction retains the current runtime");

        // 5. Let rejectedClosure be a new Abstract Closure with parameters (reason) that captures asyncContext and performs the following steps when called:
        // 6. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, captures, context| {
                    if let Some(evaluation) = &captures.0.evaluation {
                        evaluation.evaluation.consume(&AwaitRef {
                            edge: captures.record_ref(),
                            data: Rc::downgrade(&captures.0),
                        });
                    }
                    captures.resume(
                        args.get_or_undefined(0).clone(),
                        GeneratorResumeKind::Throw,
                        context,
                    )
                },
                captures.clone(),
            ),
        )
        .name(js_string!())
        .length(1)
        .build()
        .expect("await reaction retains the current runtime");

        // 7. Perform PerformPromiseThen(promise, onFulfilled, onRejected).
        if let Err(error) = Promise::perform_promise_then_module(
            &promise,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            action,
            context,
        ) {
            // Withdraw a reaction that was never run before reclaiming its
            // segment. An inline reaction may already own a newer suspension.
            let unconsumed = captures.0.continuation.borrow_mut().take();
            if unconsumed.is_some() {
                context.vm.stack.append(r#gen.stack);
                context.vm.frame_mut().restore_stack();
                return context.handle_error(error);
            }
            // Running a catch handler here would use the detached enclosing
            // frame's storage, after its continuation already advanced.
            return ControlFlow::Break(CompletionRecord::Throw(error));
        }

        context.vm.set_return_value(return_value);
        context.handle_yield()
    }
}

impl Operation for Await {
    const NAME: &'static str = "Await";
    const INSTRUCTION: &'static str = "INST - Await";
    const COST: u8 = 5;
}

/// `CreatePromiseCapability` implements the Opcode Operation for `Opcode::CreatePromiseCapability`
///
/// Operation:
///  - Create a promise capacity for an async function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreatePromiseCapability;

impl CreatePromiseCapability {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .js_expect("cannot fail per spec")?;

        context.vm.set_promise_capability(promise_capability)
    }
}

impl Operation for CreatePromiseCapability {
    const NAME: &'static str = "CreatePromiseCapability";
    const INSTRUCTION: &'static str = "INST - CreatePromiseCapability";
    const COST: u8 = 8;
}
