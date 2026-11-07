use crate::JsResult;
use crate::bytecompiler::ByteCompiler;
use surgeist_js_ast::statement::Block;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] `surgeist_js_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) -> JsResult<()> {
        let scope = self.push_declarative_scope(block.scope())?;
        self.block_declaration_instantiation(block)?;
        self.compile_statement_list(block.statement_list(), use_expr, true)?;
        self.pop_declarative_scope(scope)?;
        Ok(())
    }
}
