use crate::JsResult;
use crate::bytecompiler::{BindingAccessOpcode, ByteCompiler, Register, ToJsString};
use surgeist_js_ast::Expression;
use surgeist_js_ast::expression::operator::{Unary, unary::UnaryOp};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, dst: &Register) -> JsResult<()> {
        match unary.op() {
            UnaryOp::Delete => {
                let mut position = self.position_guard(unary)?;
                let compiler = &mut *position;

                if let Some(access) = compiler.access_expression(unary.target())? {
                    compiler.access_delete(access, dst)?;
                } else if let Expression::Optional(opt) = unary.target() {
                    compiler.compile_optional_delete(opt, dst)?;
                } else {
                    compiler.compile_expr(unary.target(), dst)?;
                    compiler
                        .bytecode
                        .emit_store_true(dst.variable(), &mut compiler.control)?;
                }
                position.finish()?;
            }
            UnaryOp::Minus => {
                self.compile_expr(unary.target(), dst)?;
                self.bytecode.emit_neg(dst.variable(), &mut self.control)?;
            }
            UnaryOp::Plus => {
                self.compile_expr(unary.target(), dst)?;
                self.bytecode.emit_pos(dst.variable(), &mut self.control)?;
            }
            UnaryOp::Not => {
                self.compile_expr(unary.target(), dst)?;
                self.bytecode
                    .emit_logical_not(dst.variable(), &mut self.control)?;
            }
            UnaryOp::Tilde => {
                self.compile_expr(unary.target(), dst)?;
                self.bytecode
                    .emit_bit_not(dst.variable(), &mut self.control)?;
            }
            UnaryOp::TypeOf => {
                match self.flatten_expression(unary.target())? {
                    Expression::Identifier(identifier) => {
                        let identifier =
                            identifier.try_to_js_string(&*self.interner, &mut self.control)?;
                        let binding = self.lexical_identifier_reference(identifier)?;
                        let index = self.get_binding(&binding)?;
                        self.emit_binding_access(
                            BindingAccessOpcode::GetNameOrUndefined,
                            &index,
                            dst,
                        )?;
                    }
                    expr => self.compile_expr(expr, dst)?,
                }
                self.bytecode
                    .emit_type_of(dst.variable(), &mut self.control)?;
            }
            UnaryOp::Void => {
                self.compile_expr(unary.target(), dst)?;
                self.bytecode
                    .emit_store_undefined(dst.variable(), &mut self.control)?;
            }
        }
        Ok(())
    }
}
