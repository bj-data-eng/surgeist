//! Boa's implementation of ECMAScript's global `Generator` object.
//!
//! A Generator is an instance of a generator function and conforms to both the Iterator and Iterable interfaces.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-generator-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator

use crate::{
    Context, JsArgs, JsData, JsError, JsExpect, JsResult, JsString,
    builtins::iterable::create_iter_result_object_trusted,
    context::intrinsics::Intrinsics,
    error::{JsNativeError, RootingError},
    js_string,
    object::{CONSTRUCTOR, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::JsValue,
    vm::call_frame::StoredCallFrame,
    vm::{CallFrame, CallFrameFlags, CompletionRecord, GeneratorResumeKind, Stack, StoredStack},
};

use super::{BuiltInBuilder, IntrinsicObject};

/// Indicates the state of a generator.
#[derive(Debug, surgeist_js_gc::heap::Trace)]
pub(crate) enum GeneratorState {
    SuspendedStart {
        /// The `[[GeneratorContext]]` internal slot.
        context: StoredGeneratorContext,
    },
    SuspendedYield {
        /// The `[[GeneratorContext]]` internal slot.
        context: StoredGeneratorContext,
    },
    Executing,
    Completed,
}

/// Holds all information that a generator needs to continue it's execution.
///
/// All of the fields must be changed with those that are currently present in the
/// context/vm before the generator execution starts/resumes and after it has ended/yielded.
#[derive(Debug)]
pub(crate) struct GeneratorContext {
    pub(crate) stack: Stack,
    pub(crate) call_frame: Option<CallFrame>,
}

#[derive(Debug, Clone, surgeist_js_gc::heap::Trace)]
pub(crate) struct StoredGeneratorContext {
    stack: StoredStack,
    call_frame: Option<StoredCallFrame>,
}

impl StoredGeneratorContext {
    pub(crate) fn to_context(&self) -> Result<GeneratorContext, RootingError> {
        Ok(GeneratorContext {
            stack: self.stack.to_stack()?,
            call_frame: self
                .call_frame
                .as_ref()
                .map(StoredCallFrame::to_frame)
                .transpose()?,
        })
    }
}

impl GeneratorContext {
    pub(crate) fn continuation_finish(&self) -> JsResult<crate::vm::opcode::ContinuationFinish> {
        self.call_frame.as_ref().map_or(
            Err(crate::error::EngineError::InvalidEntryPermit.into()),
            CallFrame::continuation_finish,
        )
    }
    pub(crate) fn resume_from_await(
        &mut self,
        value: Option<JsValue>,
        kind: GeneratorResumeKind,
        context: &mut Context,
        origin: crate::vm::opcode::AwaitRoot,
    ) -> CompletionRecord {
        let Some(frame) = self.call_frame.as_mut() else {
            return CompletionRecord::Throw(crate::error::EngineError::InvalidEntryPermit.into());
        };
        if let Err(error) = frame.set_await_origin(origin) {
            return CompletionRecord::Throw(error);
        }
        self.resume(value, kind, context)
    }
    /// Await can transfer this invocation's stack while its frame is still
    /// unwinding. Native snapshots have storage but no publication authority.
    pub(crate) fn owns_continuation(&self) -> bool {
        self.call_frame
            .as_ref()
            .is_some_and(CallFrame::owns_continuation)
    }

    pub(crate) fn stored(&self) -> StoredGeneratorContext {
        assert!(
            self.call_frame.as_ref().is_some_and(CallFrame::has_stack),
            "cannot store a transferred continuation"
        );
        StoredGeneratorContext {
            stack: self.stack.stored(),
            call_frame: self.call_frame.as_ref().map(CallFrame::stored),
        }
    }

    /// Creates a new `GeneratorContext` from the current `Context` state.
    pub(crate) fn from_current(
        context: &mut Context,
        async_generator: Option<JsObject>,
    ) -> JsResult<Self> {
        context.vm.frame().check_suspension()?;
        let mut frame = context.vm.frame().clone();
        frame.environments = context.vm.frame().environments.clone();
        frame.realm = context.realm().clone();

        // Split the stack at fp. The split-off portion starts at what was fp,
        // so adjust rp and fp to be relative to the new base.
        let mut stack = context.vm.stack.split_off_frame(&frame);
        context.vm.frame_mut().transfer_stack()?;
        frame.rp -= frame.fp;
        frame.fp = 0;

        // NOTE: Since we get a pre-built call frame with stack, and we reuse them.
        //       So we don't need to push the registers in subsequent calls.
        frame.flags |= CallFrameFlags::REGISTERS_ALREADY_PUSHED;

        if let Some(async_generator) = async_generator {
            stack.set_register(
                &frame,
                CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX,
                async_generator.into(),
            );
        }

        Ok(Self {
            call_frame: Some(frame),
            stack,
        })
    }

    /// Copies ambient state for native callbacks without suspending the caller
    /// or granting permission to publish its bytecode continuation.
    #[cfg(feature = "experimental")]
    pub(crate) fn snapshot_current(context: &Context) -> Self {
        let mut frame = context.vm.frame().clone();
        frame.mark_native_snapshot();
        frame.realm = context.realm().clone();
        let mut stack = context.vm.stack.clone();
        let stack = stack.split_off_frame(&frame);
        frame.rp -= frame.fp;
        frame.fp = 0;
        frame.flags |= CallFrameFlags::REGISTERS_ALREADY_PUSHED;
        Self {
            stack,
            call_frame: Some(frame),
        }
    }

    /// Attaches owned continuation storage for one callback. The saved caller
    /// stack remains rooted in the guard and is restored before its append
    /// checkpoint on both normal return and Rust unwind.
    ///
    /// Only normal return recovers a callee continuation. Unwind abandons this
    /// invocation without guest cleanup or revoking custody already transferred
    /// to accepted reactions; request terminality is a separate lifecycle policy.
    pub(crate) fn with_attached<R>(
        &mut self,
        context: &mut Context,
        call: impl FnOnce(&mut Context) -> R,
    ) -> JsResult<R> {
        // Stored captures always have a frame. Take it before moving any stack,
        // leaving this context unable to publish if the callback unwinds.
        let frame = self
            .call_frame
            .take()
            .ok_or(crate::error::EngineError::InvalidEntryPermit)?;
        let fp = frame.fp;
        let rp = frame.rp;
        let checkpoint = context.vm.call_checkpoint();
        let caller_stack =
            std::mem::replace(&mut context.vm.stack, std::mem::take(&mut self.stack));
        let mut active = context.guard(move |context| {
            // The checkpoint cannot reconstruct a moved caller prefix. Reinstall
            // its actual storage before releasing callee frames and shadow state.
            context.vm.stack = caller_stack;
            context.vm.restore_call(checkpoint);
        });
        active.vm.push_frame(frame);
        active.vm.frame_mut().fp = fp;
        active.vm.frame_mut().rp = rp;

        let result = call(&mut active);

        self.stack = std::mem::take(&mut active.vm.stack);
        self.call_frame = active.vm.pop_frame();
        let recovered = self.call_frame.is_some();
        drop(active);
        if recovered {
            Ok(result)
        } else {
            Err(crate::error::EngineError::InvalidEntryPermit.into())
        }
    }

    /// Resumes execution with `GeneratorContext` as the current execution context.
    pub(crate) fn resume(
        &mut self,
        value: Option<JsValue>,
        resume_kind: GeneratorResumeKind,
        context: &mut Context,
    ) -> CompletionRecord {
        if !self.owns_continuation() {
            return CompletionRecord::Throw(crate::error::EngineError::InvalidEntryPermit.into());
        }
        let Some(frame) = self.call_frame.as_mut() else {
            return CompletionRecord::Throw(crate::error::EngineError::InvalidEntryPermit.into());
        };
        frame.set_exit_early(true);
        let mut first_error = None;
        let attached = self.with_attached(context, |context| {
            if let Some(value) = value {
                context.vm.stack.push(value);
            }
            context.vm.stack.push(resume_kind);
            let result = context.run();
            if let CompletionRecord::Throw(error) = &result {
                first_error = Some(error.clone());
            }
            result
        });
        match attached {
            Ok(result) => result,
            Err(later) => CompletionRecord::Throw(first_error.unwrap_or(later)),
        }
    }

    /// Returns the async generator object, if the function that this [`GeneratorContext`] is from an async generator, [`None`] otherwise.
    pub(crate) fn async_generator_object(&self) -> JsResult<Option<JsObject>> {
        let Some(frame) = self.call_frame.as_ref() else {
            return Ok(None);
        };

        if !frame.code_block().is_async_generator() {
            return Ok(None);
        }

        let val = self
            .stack
            .get_register(frame, CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX)
            .js_expect("registers must have an async generator object")?;

        val.as_object()
            .map(Some)
            .ok_or_else(|| crate::error::EngineError::InvalidEntryPermit.into())
    }
}

/// The internal representation of a `Generator` object.
#[derive(Debug, JsData, surgeist_js_gc::heap::Trace)]
pub struct Generator {
    /// The `[[GeneratorState]]` internal slot.
    pub(crate) state: GeneratorState,
}

impl IntrinsicObject for Generator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)?
            .prototype(realm.intrinsics().constructors().iterator().prototype()?)?
            .static_method(Self::next, js_string!("next"), 1)?
            .static_method(Self::r#return, js_string!("return"), 1)?
            .static_method(Self::throw, js_string!("throw"), 1)?
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )?
            .static_property(
                CONSTRUCTOR,
                realm
                    .intrinsics()
                    .constructors()
                    .generator_function()
                    .prototype()?,
                Attribute::CONFIGURABLE,
            )?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> crate::JsResult<JsObject> {
        Ok(intrinsics.objects().generator()?)
    }
}

impl Generator {
    const NAME: JsString = StaticJsStrings::GENERATOR;

    /// `Generator.prototype.next ( value )`
    ///
    /// The `next()` method returns an object with two properties done and value.
    /// You can also provide a parameter to the next method to send a value to the generator.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.next
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/next
    pub(crate) fn next(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GeneratorResume(this value, value, empty).
        Self::generator_resume(this, args.get_or_undefined(0).clone(), context)
    }

    /// `Generator.prototype.return ( value )`
    ///
    /// The `return()` method returns the given value and finishes the generator.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.return
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/return
    pub(crate) fn r#return(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let g be the this value.
        // 2. Let C be Completion { [[Type]]: return, [[Value]]: value, [[Target]]: empty }.
        // 3. Return ? GeneratorResumeAbrupt(g, C, empty).
        Self::generator_resume_abrupt(this, Ok(args.get_or_undefined(0).clone()), context)
    }

    /// `Generator.prototype.throw ( exception )`
    ///
    /// The `throw()` method resumes the execution of a generator by throwing an error into it
    /// and returns an object with two properties done and value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator.prototype.throw
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/throw
    pub(crate) fn throw(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let g be the this value.
        // 2. Let C be ThrowCompletion(exception).
        // 3. Return ? GeneratorResumeAbrupt(g, C, empty).
        Self::generator_resume_abrupt(
            this,
            Err(JsError::from_opaque(args.get_or_undefined(0).clone())?),
            context,
        )
    }

    /// `27.5.3.3 GeneratorResume ( generator, value, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresume
    pub(crate) fn generator_resume(
        r#gen: &JsValue,
        value: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        context.with_generator_request(|context| {
            context.runtime().check_value(r#gen)?;
            context.runtime().check_value(&value)?;
            // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
            let Some(generator_obj) = r#gen.as_object() else {
                return Err(JsNativeError::typ()
                    .with_message("Generator method called on non generator")
                    .into());
            };
            let mut r#gen = generator_obj.downcast_mut::<Self>()?.ok_or_else(|| {
                JsNativeError::typ().with_message("generator resumed on non generator object")
            })?;

            // 4. Let genContext be generator.[[GeneratorContext]].
            // 5. Let methodContext be the running execution context.
            // 6. Suspend methodContext.
            // 7. Set generator.[[GeneratorState]] to executing.
            let (mut generator_context, first_execution) = match &r#gen.state {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is completed, return CreateIterResultObject(undefined, true).
                GeneratorState::Completed => {
                    drop(r#gen);
                    return create_iter_result_object_trusted(JsValue::undefined(), true, context);
                }
                // 3. Assert: state is either suspendedStart or suspendedYield.
                GeneratorState::SuspendedStart { context } => (context.to_context()?, true),
                GeneratorState::SuspendedYield { context } => (context.to_context()?, false),
            };

            let previous = std::mem::replace(&mut r#gen.state, GeneratorState::Executing);
            drop(r#gen);
            drop(previous);

            let record = generator_context.resume(
                (!first_execution).then_some(value),
                GeneratorResumeKind::Normal,
                context,
            );

            let mut r#gen = generator_obj
                .downcast_mut::<Self>()
                .and_then(|guard| guard.ok_or(crate::error::ObjectAccessError::WrongNativeBrand))
                .map_err(|later| match &record {
                    CompletionRecord::Throw(error) if error.as_engine().is_some() => error.clone(),
                    _ => JsError::from(later),
                })?;

            // 8. Push genContext onto the execution context stack; genContext is now the running execution context.
            // 9. Resume the suspended evaluation of genContext using NormalCompletion(value) as the result of the operation that suspended it. Let result be the value returned by the resumed computation.
            // 10. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
            // 11. Return Completion(result).
            match record {
                CompletionRecord::Normal(value) => {
                    r#gen.state = GeneratorState::SuspendedYield {
                        context: generator_context.stored(),
                    };
                    Ok(value)
                }
                CompletionRecord::Return(value) => {
                    r#gen.state = GeneratorState::Completed;
                    create_iter_result_object_trusted(value, true, context)
                }
                CompletionRecord::Throw(err) => {
                    r#gen.state = GeneratorState::Completed;
                    Err(err)
                }
            }
        })
    }

    /// `27.5.3.4 GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorresumeabrupt
    pub(crate) fn generator_resume_abrupt(
        r#gen: &JsValue,
        abrupt_completion: JsResult<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        context.with_generator_request(|context| {
            context.runtime().check_value(r#gen)?;
            match &abrupt_completion {
                Ok(value) => context.runtime().check_value(value)?,
                Err(error) => {
                    if let Some(value) = error.as_opaque() {
                        context.runtime().check_value(value)?;
                    }
                }
            }
            // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
            let Some(generator_obj) = r#gen.as_object() else {
                return Err(JsNativeError::typ()
                    .with_message("Generator method called on non generator")
                    .into());
            };
            let mut r#gen = generator_obj.downcast_mut::<Self>()?.ok_or_else(|| {
                JsNativeError::typ().with_message("generator resumed on non generator object")
            })?;

            // 4. Assert: state is suspendedYield.
            // 5. Let genContext be generator.[[GeneratorContext]].
            // 6. Let methodContext be the running execution context.
            // 7. Suspend methodContext.
            // 8. Set generator.[[GeneratorState]] to executing.
            let mut generator_context = match &r#gen.state {
                GeneratorState::Executing => {
                    return Err(JsNativeError::typ()
                        .with_message("Generator should not be executing")
                        .into());
                }
                // 2. If state is suspendedStart, then
                // 3. If state is completed, then
                GeneratorState::SuspendedStart { .. } | GeneratorState::Completed => {
                    // a. Set generator.[[GeneratorState]] to completed.
                    let previous = std::mem::replace(&mut r#gen.state, GeneratorState::Completed);
                    drop(r#gen);
                    drop(previous);

                    // b. Once a generator enters the completed state it never leaves it and its
                    // associated execution context is never resumed. Any execution state associated
                    // with generator can be discarded at this point.

                    // a. If abruptCompletion.[[Type]] is return, then
                    if let Ok(value) = abrupt_completion {
                        // i. Return CreateIterResultObject(abruptCompletion.[[Value]], true).
                        let value = create_iter_result_object_trusted(value, true, context)?;
                        return Ok(value);
                    }

                    // b. Return Completion(abruptCompletion).
                    return abrupt_completion;
                }
                GeneratorState::SuspendedYield { context } => context.to_context()?,
            };

            let previous = std::mem::replace(&mut r#gen.state, GeneratorState::Executing);

            // 9. Push genContext onto the execution context stack; genContext is now the running execution context.
            // 10. Resume the suspended evaluation of genContext using abruptCompletion as the result of the operation that suspended it. Let result be the completion record returned by the resumed computation.
            // 11. Assert: When we return here, genContext has already been removed from the execution context stack and methodContext is the currently running execution context.
            // 12. Return Completion(result).
            drop(r#gen);
            drop(previous);

            let (value, resume_kind) = match abrupt_completion {
                Ok(value) => (value, GeneratorResumeKind::Return),
                Err(err) => (
                    err.into_opaque(context.access())?,
                    GeneratorResumeKind::Throw,
                ),
            };

            let record = generator_context.resume(Some(value), resume_kind, context);

            let mut r#gen = generator_obj
                .downcast_mut::<Self>()
                .and_then(|guard| guard.ok_or(crate::error::ObjectAccessError::WrongNativeBrand))
                .map_err(|later| match &record {
                    CompletionRecord::Throw(error) if error.as_engine().is_some() => error.clone(),
                    _ => JsError::from(later),
                })?;

            match record {
                CompletionRecord::Normal(value) => {
                    r#gen.state = GeneratorState::SuspendedYield {
                        context: generator_context.stored(),
                    };
                    Ok(value)
                }
                CompletionRecord::Return(value) => {
                    r#gen.state = GeneratorState::Completed;
                    create_iter_result_object_trusted(value, true, context)
                }
                CompletionRecord::Throw(err) => {
                    r#gen.state = GeneratorState::Completed;
                    Err(err)
                }
            }
        })
    }
}
