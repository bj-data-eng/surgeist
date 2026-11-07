use crate::JsResult;
use crate::bytecompiler::{ByteCompiler, NodeKind, control::CompilerWork};
use surgeist_js_ast::{
    Statement,
    statement::{Labelled, LabelledItem},
};

impl ByteCompiler<'_, '_> {
    /// Compile a [`Labelled`] `surgeist_js_ast` node
    pub(crate) fn compile_labelled(&mut self, labelled: &Labelled, use_expr: bool) -> JsResult<()> {
        let labelled_loc = self.next_opcode_location()?;
        self.push_labelled_control_info(labelled.label(), labelled_loc, use_expr)?;

        match labelled.item() {
            LabelledItem::Statement(stmt) => {
                // Labelled loops bypass compile_stmt to retain their label.
                // Count that actual Statement descent here, but let the normal
                // wrapper own every other statement's visit and depth.
                let _depth = if matches!(
                    stmt,
                    Statement::ForLoop(_)
                        | Statement::ForInLoop(_)
                        | Statement::ForOfLoop(_)
                        | Statement::WhileLoop(_)
                        | Statement::DoWhileLoop(_)
                ) {
                    let guard = self.control.enter()?;
                    self.control.admit(CompilerWork::Visit)?;
                    Some(guard)
                } else {
                    None
                };
                match stmt {
                    Statement::ForLoop(for_loop) => {
                        self.compile_for_loop(for_loop, Some(labelled.label()), use_expr)?;
                    }
                    Statement::ForInLoop(for_in_loop) => {
                        self.compile_for_in_loop(for_in_loop, Some(labelled.label()), use_expr)?;
                    }
                    Statement::ForOfLoop(for_of_loop) => {
                        self.compile_for_of_loop(for_of_loop, Some(labelled.label()), use_expr)?;
                    }
                    Statement::WhileLoop(while_loop) => {
                        self.compile_while_loop(while_loop, Some(labelled.label()), use_expr)?;
                    }
                    Statement::DoWhileLoop(do_while_loop) => {
                        self.compile_do_while_loop(
                            do_while_loop,
                            Some(labelled.label()),
                            use_expr,
                        )?;
                    }
                    stmt => self.compile_stmt(stmt, use_expr, true)?,
                }
            }
            LabelledItem::FunctionDeclaration(f) => {
                let dst = self.register_allocator.alloc(&mut self.control)?;
                self.function_with_binding(f.into(), NodeKind::Declaration, &dst)?;
                self.register_allocator.dealloc(dst, &mut self.control)?;
            }
        }

        self.pop_labelled_control_info()?;
        Ok(())
    }
}
