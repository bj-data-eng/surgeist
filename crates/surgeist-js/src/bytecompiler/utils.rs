use super::{ByteCompiler, Literal, Register};
use crate::{JsResult, js_string, vm::GeneratorResumeKind};

impl ByteCompiler<'_, '_> {
    /// Closes an iterator
    ///
    /// This is equivalent to the [`IteratorClose`][iter] and [`AsyncIteratorClose`][async]
    /// operations.
    ///
    /// Iterator Stack:
    ///  - iterator **=>** \<empty\>
    ///
    /// [iter]: https://tc39.es/ecma262/#sec-iteratorclose
    /// [async]: https://tc39.es/ecma262/#sec-asynciteratorclose
    pub(super) fn iterator_close(&mut self, async_: bool) -> JsResult<()> {
        let value = self.register_allocator.alloc(&mut self.control)?;
        let called = self.register_allocator.alloc(&mut self.control)?;
        self.bytecode.emit_iterator_return(
            value.variable(),
            called.variable(),
            &mut self.control,
        )?;

        // `iterator` didn't have a `return` method, is already done or is not on the iterator stack.
        let early_exit = self.jump_if_false(&called)?;
        self.register_allocator.dealloc(called, &mut self.control)?;

        if async_ {
            self.bytecode
                .emit_await(value.variable(), &mut self.control)?;
            let resume_kind = self.register_allocator.alloc(&mut self.control)?;
            self.pop_into_register(&resume_kind)?;
            self.pop_into_register(&value)?;
            self.generator_next(&value, &resume_kind)?;
            self.register_allocator
                .dealloc(resume_kind, &mut self.control)?;
        }

        self.bytecode
            .emit_is_object(value.variable(), &mut self.control)?;
        let skip_throw = self.jump_if_true(&value)?;

        self.register_allocator.dealloc(value, &mut self.control)?;

        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
            "inner result was not an object"
        )))?;
        self.bytecode
            .emit_throw_new_type_error(error_msg.into(), &mut self.control)?;

        self.patch_jump(skip_throw)?;
        self.patch_jump(early_exit)?;
        Ok(())
    }

    /// Closes all active iterators in the current [`CallFrame`][crate::vm::CallFrame].
    pub(super) fn close_active_iterators(&mut self) -> JsResult<()> {
        let start = self.next_opcode_location()?;

        let empty = self.register_allocator.alloc(&mut self.control)?;
        self.bytecode
            .emit_iterator_stack_empty(empty.variable(), &mut self.control)?;
        let exit = self.jump_if_true(&empty)?;
        self.register_allocator.dealloc(empty, &mut self.control)?;

        self.iterator_close(self.is_async_generator())?;
        self.bytecode.emit_jump(start, &mut self.control)?;
        self.patch_jump(exit)?;
        Ok(())
    }

    /// Yields from the current generator.
    ///
    /// This is equivalent to the [`Yield ( value )`][yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** received
    ///
    /// [yield]: https://tc39.es/ecma262/#sec-yield
    pub(super) fn r#yield(&mut self, value: &Register) -> JsResult<()> {
        let resume_kind = self.register_allocator.alloc(&mut self.control)?;

        // 1. Let generatorKind be GetGeneratorKind().
        if self.is_async() {
            // 2. If generatorKind is async, return ? AsyncGeneratorYield(? Await(value)).
            self.bytecode
                .emit_await(value.variable(), &mut self.control)?;
            self.pop_into_register(&resume_kind)?;
            self.pop_into_register(value)?;
            self.generator_next(value, &resume_kind)?;
            self.async_generator_yield(value, &resume_kind)?;
        } else {
            // 3. Otherwise, return ? GeneratorYield(CreateIterResultObject(value, false)).
            self.bytecode.emit_create_iterator_result(
                value.variable(),
                false.into(),
                &mut self.control,
            )?;
            self.bytecode
                .emit_generator_yield(value.variable(), &mut self.control)?;
            self.pop_into_register(&resume_kind)?;
            self.pop_into_register(value)?;
        }

        self.generator_next(value, &resume_kind)?;
        self.register_allocator
            .dealloc(resume_kind, &mut self.control)?;
        Ok(())
    }

    /// Yields from the current async generator.
    ///
    /// This is equivalent to the [`AsyncGeneratorYield ( value )`][async_yield] operation from the spec.
    ///
    /// stack:
    /// - value **=>** `resume_kind`, received
    ///
    /// [async_yield]: https://tc39.es/ecma262/#sec-asyncgeneratoryield
    pub(super) fn async_generator_yield(
        &mut self,
        value: &Register,
        resume_kind: &Register,
    ) -> JsResult<()> {
        self.bytecode
            .emit_async_generator_yield(value.variable(), &mut self.control)?;
        self.pop_into_register(resume_kind)?;
        self.pop_into_register(value)?;

        let non_return_resume =
            self.jump_if_not_resume_kind(GeneratorResumeKind::Return, resume_kind)?;

        self.bytecode
            .emit_await(value.variable(), &mut self.control)?;
        self.pop_into_register(resume_kind)?;
        self.pop_into_register(value)?;

        let non_normal_resume =
            self.jump_if_not_resume_kind(GeneratorResumeKind::Normal, resume_kind)?;

        self.emit_resume_kind(GeneratorResumeKind::Return, resume_kind)?;

        self.patch_jump(non_normal_resume)?;
        self.patch_jump(non_return_resume)?;
        Ok(())
    }
}
