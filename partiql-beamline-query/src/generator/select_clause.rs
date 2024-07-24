use crate::generator::{AstGenContext, AstGenerator};
use partiql_ast::ast;

#[derive(Clone, Debug)]
pub struct SelectStar {}

impl AstGenerator<ast::Projection> for SelectStar {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Projection {
        ast::Projection {
            kind: ast::ProjectionKind::ProjectStar,
            setq: None,
        }
    }
}
