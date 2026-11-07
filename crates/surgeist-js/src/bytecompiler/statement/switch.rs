use crate::JsResult;
use crate::bytecompiler::{
    ByteCompiler, Label,
    control::{CompilerWork, reserve},
};
use surgeist_js_ast::statement::Switch;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Switch`] `surgeist_js_ast` node
    pub(crate) fn compile_switch(&mut self, switch: &Switch, use_expr: bool) -> JsResult<()> {
        let value = self.register_allocator.alloc(&mut self.control)?;
        self.compile_expr(switch.val(), &value)?;
        let outer_scope = self.push_declarative_scope(switch.scope())?;

        self.block_declaration_instantiation(switch)?;

        let start_address = self.next_opcode_location()?;
        self.push_switch_control_info(None, start_address, use_expr)?;

        let mut labels = Vec::new();
        reserve(&mut labels, switch.cases().len(), &mut self.control)?;

        let condition = self.register_allocator.alloc(&mut self.control)?;

        for case in switch.cases() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // If it does not have a condition it is the default case.
            let label = if let Some(cond) = case.condition() {
                self.compile_expr(cond, &condition)?;
                self.case(&value, &condition)?
            } else {
                Self::DUMMY_LABEL
            };

            self.control.admit(CompilerWork::MetadataWrite)?;
            self.control.admit(CompilerWork::Copy {
                bytes: size_of::<Label>(),
            })?;
            labels.push(label);
        }

        self.register_allocator
            .dealloc(condition, &mut self.control)?;
        self.register_allocator.dealloc(value, &mut self.control)?;

        let default_label = self.jump()?;
        let mut default_label_set = false;

        for (label, case) in labels.into_iter().zip(switch.cases()) {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // Check if it's the default case.
            let label = if label == Self::DUMMY_LABEL {
                default_label_set = true;
                default_label
            } else {
                label
            };
            self.patch_jump(label)?;

            self.compile_statement_list(case.body(), use_expr, true)?;
        }

        if !default_label_set {
            self.patch_jump(default_label)?;
        }

        self.pop_switch_control_info()?;
        self.pop_declarative_scope(outer_scope)?;
        Ok(())
    }
}
