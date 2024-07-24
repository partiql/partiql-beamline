use crate::generator::{AstGenContext, AstGenerator};
use partiql_ast::ast;

#[derive(Clone, Debug)]
pub struct FromTable {
    pub name: String,
}

impl AstGenerator<ast::FromClause> for FromTable {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::FromClause {
        let name = ast::SymbolPrimitive {
            value: self.name.clone(),
            case: ast::CaseSensitivity::CaseInsensitive,
        };
        let table = ctx.node(ast::VarRef {
            name: name.clone(),
            qualifier: ast::ScopeQualifier::Unqualified,
        });
        let expr = Box::new(ast::Expr::VarRef(table));
        let as_alias = Some(name);
        let from_let = ctx.node(ast::FromLet {
            expr,
            kind: ast::FromLetKind::Scan,
            as_alias,
            at_alias: None,
            by_alias: None,
        });
        let source = ast::FromSource::FromLet(from_let);
        ast::FromClause { source }
    }
}
