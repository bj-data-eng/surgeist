use crate::{
    JsResult,
    bytecompiler::{
        ByteCompiler, Label, Register,
        control::{CompilerControl, CompilerWork, thin_push, thin_with_capacity},
    },
    js_string,
    vm::{GeneratorResumeKind, opcode::Address},
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn generator_next(
        &mut self,
        value: &Register,
        resume_kind: &Register,
    ) -> JsResult<()> {
        self.control.admit(CompilerWork::Visit)?;
        // NOTE: +4 to jump past the index operand.
        let jump_table_index = self.next_opcode_location()?.checked_add(4)?;
        let table = resume_table(&mut self.control)?;
        self.bytecode
            .emit_jump_table(resume_kind.index(), table, &mut self.control)?;
        // Return branch
        self.bytecode
            .emit_set_accumulator(value.variable(), &mut self.control)?;
        self.bytecode.emit_re_throw(&mut self.control)?;

        // Normal branch
        let normal = self.next_opcode_location()?;
        let end = self.jump()?;

        // Throw branch
        let throw = self.next_opcode_location()?;
        self.bytecode
            .emit_throw(value.variable(), &mut self.control)?;

        self.patch_jump(end)?;
        self.bytecode
            .patch_jump_table(jump_table_index, &[normal, throw], &mut self.control)?;
        Ok(())
    }

    pub(crate) fn generator_delegate_next(
        &mut self,
        value: &Register,
        resume_kind: &Register,
        is_return: &Register,
    ) -> JsResult<(Label, Label)> {
        self.control.admit(CompilerWork::Visit)?;
        let iterator = self.register_allocator.alloc(&mut self.control)?;
        let next = self.register_allocator.alloc(&mut self.control)?;
        self.bytecode
            .emit_iterator_pop(iterator.variable(), next.variable(), &mut self.control)?;

        // NOTE: +4 to jump past the index operand.
        let jump_table_index = self.next_opcode_location()?.checked_add(4)?;
        let table = resume_table(&mut self.control)?;
        self.bytecode
            .emit_jump_table(resume_kind.index(), table, &mut self.control)?;

        // GeneratorResumeKind::Return
        let name_index = self.get_or_insert_string(js_string!("return"))?;
        self.bytecode
            .emit_move(is_return.variable(), iterator.variable(), &mut self.control)?;

        self.bytecode.emit_get_method(
            is_return.variable(),
            name_index.into(),
            &mut self.control,
        )?;

        // If the return method is unavailable, jump outside the resume.
        let return_method_undefined = self.jump_if_null_or_undefined(is_return)?;

        // Return method is available. Call and set `is_return = true` and
        // `value = returned value`.
        self.push_from_register(&iterator)?;
        self.push_from_register(is_return)?;
        self.push_from_register(value)?;
        self.bytecode.emit_call(1u8.into(), &mut self.control)?;
        self.bytecode
            .emit_store_true(is_return.variable(), &mut self.control)?;
        self.pop_into_register(value)?;
        let return_jump = self.jump()?;

        // GeneratorResumeKind::Normal
        let normal = self.next_opcode_location()?;
        self.push_from_register(&iterator)?;
        self.push_from_register(&next)?;
        self.push_from_register(value)?;
        self.bytecode.emit_call(1u8.into(), &mut self.control)?;
        self.bytecode
            .emit_store_false(is_return.variable(), &mut self.control)?;
        self.pop_into_register(value)?;
        let normal_jump = self.jump()?;

        // GeneratorResumeKind::Throw
        let throw = self.next_opcode_location()?;
        let name_index = self.get_or_insert_string(js_string!("throw"))?;
        self.bytecode
            .emit_move(is_return.variable(), iterator.variable(), &mut self.control)?;
        self.bytecode.emit_get_method(
            is_return.variable(),
            name_index.into(),
            &mut self.control,
        )?;

        let skip_push = self.jump_if_not_undefined(is_return)?;
        self.bytecode.emit_iterator_push(
            iterator.variable(),
            next.variable(),
            &mut self.control,
        )?;
        // If the return method is unavailable, jump outside the resume.
        let throw_method_undefined = self.jump()?;

        self.patch_jump(skip_push)?;

        // Throw method is available. Call and set `is_return = true` and
        // `value = returned value`.
        self.push_from_register(&iterator)?;
        self.push_from_register(is_return)?;
        self.push_from_register(value)?;
        self.bytecode.emit_call(1u8.into(), &mut self.control)?;
        self.bytecode
            .emit_store_false(is_return.variable(), &mut self.control)?;
        self.pop_into_register(value)?;

        self.bytecode
            .patch_jump_table(jump_table_index, &[normal, throw], &mut self.control)?;
        self.patch_jump(return_jump)?;
        self.patch_jump(normal_jump)?;

        self.bytecode.emit_iterator_push(
            iterator.variable(),
            next.variable(),
            &mut self.control,
        )?;
        self.register_allocator
            .dealloc(iterator, &mut self.control)?;
        self.register_allocator.dealloc(next, &mut self.control)?;

        Ok((return_method_undefined, throw_method_undefined))
    }

    pub(crate) fn generator_delegate_resume(
        &mut self,
        value: &Register,
        resume_kind: &Register,
        is_return: &Register,
    ) -> JsResult<(Label, Label)> {
        self.control.admit(CompilerWork::Visit)?;
        let not_throw = self.jump_if_not_resume_kind(GeneratorResumeKind::Throw, resume_kind)?;

        // resume_kind is throw. Pop the active iterator and raise the error.
        self.bytecode.emit_iterator_pop(
            resume_kind.variable(),
            resume_kind.variable(),
            &mut self.control,
        )?;
        self.bytecode
            .emit_throw(value.variable(), &mut self.control)?;
        self.patch_jump(not_throw)?;

        // resume_kind is not throw. Try to update the iterator result
        self.bytecode
            .emit_iterator_update_result(value.variable(), &mut self.control)?;

        // `value` should contain a boolean with `iterator.done()` at this point.
        // skip pop if the iterator is not done yet
        let skip_pop = self.jump_if_false(value)?;

        self.bytecode
            .emit_iterator_value(value.variable(), &mut self.control)?;
        self.bytecode.emit_iterator_pop(
            resume_kind.variable(),
            resume_kind.variable(),
            &mut self.control,
        )?;
        let return_gen = self.jump_if_true(is_return)?;
        let exit = self.jump()?;

        self.patch_jump(skip_pop)?;

        Ok((return_gen, exit))
    }
}

/// Storage for the two explicit resume branches; return remains the default.
fn resume_table(control: &mut CompilerControl<'_>) -> JsResult<thin_vec::ThinVec<Address>> {
    let mut table = thin_with_capacity(2, control)?;
    thin_push(&mut table, ByteCompiler::DUMMY_ADDRESS, control)?;
    thin_push(&mut table, ByteCompiler::DUMMY_ADDRESS, control)?;
    Ok(table)
}
