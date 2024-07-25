use crate::generator::{AstGenContext, AstGenerator, ExprGenerator};
use partiql_ast::ast;

#[derive(Clone, Debug)]
pub struct RowFilter {
    pub expr: ExprGenerator,
}

impl AstGenerator<ast::WhereClause> for RowFilter {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::WhereClause {
        let expr = Box::new(self.expr.gen_ast(ctx));
        ast::WhereClause { expr }
    }
}
