use dyn_clone::DynClone;
use partiql_ast::ast;
use partiql_ast::ast::AstNode;
use std::cell::RefCell;
use std::fmt::Debug;

use partiql_ast::builder::NodeBuilderWithAutoId;

mod expr;
mod from_clause;
mod query;
mod select_clause;
mod where_clause;

pub use expr::*;
pub use from_clause::*;
pub use query::*;
pub use select_clause::*;
pub use where_clause::*;

use partiql_beamline::sim::SimContext;

pub struct AstGenContext {
    ctx: SimContext,
    bld: RefCell<NodeBuilderWithAutoId>,
}

impl AstGenContext {
    pub fn new(ctx: SimContext) -> Self {
        let bld = RefCell::new(NodeBuilderWithAutoId::default());
        AstGenContext { ctx, bld }
    }
    pub fn node<T>(&self, node: T) -> AstNode<T> {
        self.bld.borrow_mut().node(node)
    }

    pub fn sim_ctx(&self) -> &SimContext {
        &self.ctx
    }
}

pub trait AstGenerator<Ast>: Debug + DynClone {
    fn gen_node(&self, ctx: &AstGenContext) -> ast::AstNode<Ast> {
        let ast = self.gen_ast(ctx);
        ctx.node(ast)
    }
    fn gen_ast(&self, ctx: &AstGenContext) -> Ast;
}

pub trait AstGeneratorBoxed<Ast>: AstGenerator<Ast>
where
    Self: 'static,
{
    fn agboxed(self) -> Box<dyn AstGenerator<Ast>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T, Ast> AstGeneratorBoxed<Ast> for T where T: AstGenerator<Ast> + 'static {}

pub type DynAstGenerator<Ast> = Box<dyn AstGenerator<Ast>>;

pub type QueryGenerator = DynAstGenerator<ast::Query>;
dyn_clone::clone_trait_object!(AstGenerator<ast::Query>);
pub type QuerySetGenerator = DynAstGenerator<ast::QuerySet>;
dyn_clone::clone_trait_object!(AstGenerator<ast::QuerySet>);
pub type OrderByGenerator = DynAstGenerator<ast::OrderByExpr>;
dyn_clone::clone_trait_object!(AstGenerator<ast::OrderByExpr>);
pub type LimitOffsetGenerator = DynAstGenerator<ast::LimitOffsetClause>;
dyn_clone::clone_trait_object!(AstGenerator<ast::LimitOffsetClause>);
pub type ProjectionGenerator = DynAstGenerator<ast::Projection>;
dyn_clone::clone_trait_object!(AstGenerator<ast::Projection>);
pub type FromClauseGenerator = DynAstGenerator<ast::FromClause>;
dyn_clone::clone_trait_object!(AstGenerator<ast::FromClause>);
pub type WhereClauseGenerator = DynAstGenerator<ast::WhereClause>;
dyn_clone::clone_trait_object!(AstGenerator<ast::WhereClause>);
pub type ExprGenerator = DynAstGenerator<ast::Expr>;
dyn_clone::clone_trait_object!(AstGenerator<ast::Expr>);
