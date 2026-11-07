use crate::JsResult;
use crate::bytecompiler::ByteCompiler;
use surgeist_js_ast::statement::If;

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_if(&mut self, node: &If, use_expr: bool) -> JsResult<()> {
        self.compile_if_else(
            node.cond(),
            |compiler| {
                compiler.compile_stmt(node.body(), use_expr, true)?;
                Ok(())
            },
            |compiler| {
                if let Some(else_body) = node.else_node() {
                    compiler.compile_stmt(else_body, use_expr, true)?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}
