use crate::JsResult;
use crate::bytecompiler::control::{CompilerWork, size_overflow};
use crate::bytecompiler::{
    ByteCompiler,
    jump_control::{JumpRecord, JumpRecordAction, JumpRecordKind, record_action, reverse_actions},
};
use surgeist_js_ast::statement::Break;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Break`] `surgeist_js_ast` node
    pub(crate) fn compile_break(&mut self, node: Break, _use_expr: bool) -> JsResult<()> {
        let actions = self.break_jump_record_actions(node)?;

        JumpRecord::new(JumpRecordKind::Break, actions).perform_actions(Self::DUMMY_ADDRESS, self)
    }

    fn break_jump_record_actions(&mut self, node: Break) -> JsResult<Vec<JumpRecordAction>> {
        let mut actions = Vec::default();
        for (i, info) in self.jump_info.iter().enumerate().rev() {
            self.control.admit(CompilerWork::Search)?;
            let count = self.jump_info_open_environment_count(i);
            record_action(
                &mut actions,
                JumpRecordAction::PopEnvironments { count },
                &mut self.control,
            )?;

            if !info.in_finally()
                && let Some((finally_throw_flag, finally_throw_index)) = info.finally_throw
            {
                record_action(
                    &mut actions,
                    JumpRecordAction::HandleFinally {
                        index: u32::try_from(info.jumps.len()).map_err(|_| size_overflow())?,
                        finally_throw_flag,
                        finally_throw_index,
                    },
                    &mut self.control,
                )?;
                record_action(
                    &mut actions,
                    JumpRecordAction::Transfer {
                        index: u32::try_from(i).map_err(|_| size_overflow())?,
                    },
                    &mut self.control,
                )?;
            }

            if let Some(label) = node.label() {
                if info.label() == Some(label) {
                    record_action(
                        &mut actions,
                        JumpRecordAction::Transfer {
                            index: u32::try_from(i).map_err(|_| size_overflow())?,
                        },
                        &mut self.control,
                    )?;
                    break;
                }
            } else if info.is_loop() || info.is_switch() {
                record_action(
                    &mut actions,
                    JumpRecordAction::Transfer {
                        index: u32::try_from(i).map_err(|_| size_overflow())?,
                    },
                    &mut self.control,
                )?;
                break;
            }
        }

        reverse_actions(&mut actions, &mut self.control)?;
        Ok(actions)
    }
}
