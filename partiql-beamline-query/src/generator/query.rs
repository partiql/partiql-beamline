use crate::generator::{
    AstGenContext, AstGenerator, FromClauseGenerator, ProjectionGenerator, WhereClauseGenerator,
};
use partiql_ast::ast;

#[derive(Clone, Debug)]
pub struct BasicSFW {
    pub project: ProjectionGenerator,
    pub from: FromClauseGenerator,
    pub where_clause: Option<WhereClauseGenerator>,
}

impl AstGenerator<ast::Query> for BasicSFW {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Query {
        let set = self.gen_node(ctx);
        ast::Query {
            set,
            order_by: None,
            limit_offset: None,
        }
    }
}

impl AstGenerator<ast::QuerySet> for BasicSFW {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::QuerySet {
        let select = Box::new(self.gen_node(ctx));
        ast::QuerySet::Select(select)
    }
}

impl AstGenerator<ast::Select> for BasicSFW {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Select {
        let project = self.project.gen_node(ctx);
        let from = Some(self.from.gen_node(ctx));
        let where_clause = self
            .where_clause
            .as_ref()
            .map(|w| Box::new(w.gen_node(ctx)));
        ast::Select {
            project,
            from,
            from_let: None,
            where_clause,
            group_by: None,
            having: None,
        }
    }
}
