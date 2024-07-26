use crate::generator::{AstGenContext, AstGenerator, ExprGenerator, QueryGenerator};
use partiql_ast::ast;
use partiql_beamline::gen::ValueGenerator;
use partiql_value::Value;
use rust_decimal::Decimal as RustDecimal;

#[derive(Clone, Debug)]
pub struct Expression {
    pub expr: ExprGenerator,
}

impl AstGenerator<ast::Expr> for Expression {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        self.expr.gen_ast(ctx)
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedLiteral {
    pub value: Box<dyn ValueGenerator>,
}

impl AstGenerator<ast::Expr> for GeneratedLiteral {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let _ty = self.value.value_type();
        let val = self.value.present_value(ctx.sim_ctx());
        let lit = match val {
            Value::Null => ast::Lit::Null,
            Value::Missing => ast::Lit::Missing,
            Value::Boolean(b) => ast::Lit::BoolLit(b),
            Value::Integer(i) => ast::Lit::Int64Lit(i),
            Value::Real(r) => ast::Lit::DoubleLit(r.0),
            Value::Decimal(d) => ast::Lit::DecimalLit(*d),
            Value::String(s) => ast::Lit::CharStringLit(*s.clone()),
            Value::Blob(_) => todo!("blob lit from value"),
            Value::DateTime(_) => todo!("DateTime lit from value"),
            Value::List(_) => todo!("List lit from value"),
            Value::Bag(_) => todo!("Bag lit from value"),
            Value::Tuple(_) => todo!("Tuple lit from value"),
        };
        ast::Expr::Lit(ctx.node(lit))
    }
}

#[derive(Clone, Debug)]
pub enum ConstantLiteral {
    Null,
    Missing,
    Int8Lit(i8),
    Int16Lit(i16),
    Int32Lit(i32),
    Int64Lit(i64),
    DecimalLit(RustDecimal),
    NumericLit(RustDecimal),
    RealLit(f32),
    FloatLit(f32),
    DoubleLit(f64),
    BoolLit(bool),
    IonStringLit(String),
    CharStringLit(String),
    NationalCharStringLit(String),
    BitStringLit(String),
    HexStringLit(String),
    StructLit(/*TODO*/),
    BagLit(/*TODO*/),
    ListLit(/*TODO*/),
    TypedLit(/*TODO*/),
}

impl AstGenerator<ast::Expr> for ConstantLiteral {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let lit = match self {
            ConstantLiteral::Null => ast::Lit::Null,
            ConstantLiteral::Missing => ast::Lit::Missing,
            ConstantLiteral::Int8Lit(l) => ast::Lit::Int8Lit(*l),
            ConstantLiteral::Int16Lit(l) => ast::Lit::Int16Lit(*l),
            ConstantLiteral::Int32Lit(l) => ast::Lit::Int32Lit(*l),
            ConstantLiteral::Int64Lit(l) => ast::Lit::Int64Lit(*l),
            ConstantLiteral::DecimalLit(l) => ast::Lit::DecimalLit(*l),
            ConstantLiteral::NumericLit(l) => ast::Lit::NumericLit(*l),
            ConstantLiteral::RealLit(l) => ast::Lit::RealLit(*l),
            ConstantLiteral::FloatLit(l) => ast::Lit::FloatLit(*l),
            ConstantLiteral::DoubleLit(l) => ast::Lit::DoubleLit(*l),
            ConstantLiteral::BoolLit(l) => ast::Lit::BoolLit(*l),
            ConstantLiteral::IonStringLit(l) => ast::Lit::IonStringLit(l.clone()),
            ConstantLiteral::CharStringLit(l) => ast::Lit::CharStringLit(l.clone()),
            ConstantLiteral::NationalCharStringLit(l) => ast::Lit::NationalCharStringLit(l.clone()),
            ConstantLiteral::BitStringLit(l) => ast::Lit::BitStringLit(l.clone()),
            ConstantLiteral::HexStringLit(l) => ast::Lit::HexStringLit(l.clone()),
            _ => todo!(),
        };

        ast::Expr::Lit(ctx.node(lit))
    }
}

#[derive(Clone, Debug)]
pub struct BinOp {
    pub kind: ast::BinOpKind,
    pub lhs: ExprGenerator,
    pub rhs: ExprGenerator,
}

impl AstGenerator<ast::Expr> for BinOp {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let kind = self.kind.clone();
        let lhs = Box::new(self.lhs.gen_ast(ctx));
        let rhs = Box::new(self.rhs.gen_ast(ctx));
        let node = ctx.node(ast::BinOp { kind, lhs, rhs });
        ast::Expr::BinOp(node)
    }
}

#[derive(Clone, Debug)]
pub struct UniOp {
    pub kind: ast::UniOpKind,
    pub expr: ExprGenerator,
}

impl AstGenerator<ast::Expr> for UniOp {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let kind = self.kind.clone();
        let expr = Box::new(self.expr.gen_ast(ctx));
        let node = ctx.node(ast::UniOp { kind, expr });
        ast::Expr::UniOp(node)
    }
}

#[derive(Clone, Debug)]
pub struct Like {
    pub value: ExprGenerator,
    pub pattern: ExprGenerator,
    pub escape: Option<ExprGenerator>,
}

impl AstGenerator<ast::Expr> for Like {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let value = Box::new(self.value.gen_ast(ctx));
        let pattern = Box::new(self.pattern.gen_ast(ctx));
        let escape = self.escape.as_ref().map(|esc| Box::new(esc.gen_ast(ctx)));
        let node = ctx.node(ast::Like {
            value,
            pattern,
            escape,
        });
        ast::Expr::Like(node)
    }
}

#[derive(Clone, Debug)]
pub struct Between {
    pub value: ExprGenerator,
    pub from: ExprGenerator,
    pub to: ExprGenerator,
}

impl AstGenerator<ast::Expr> for Between {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let value = Box::new(self.value.gen_ast(ctx));
        let from = Box::new(self.from.gen_ast(ctx));
        let to = Box::new(self.to.gen_ast(ctx));
        let node = ctx.node(ast::Between { value, from, to });
        ast::Expr::Between(node)
    }
}

#[derive(Clone, Debug)]
pub struct In {
    pub lhs: ExprGenerator,
    pub rhs: ExprGenerator,
}

impl AstGenerator<ast::Expr> for In {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let lhs = Box::new(self.lhs.gen_ast(ctx));
        let rhs = Box::new(self.rhs.gen_ast(ctx));
        let node = ctx.node(ast::In { lhs, rhs });
        ast::Expr::In(node)
    }
}

#[derive(Clone, Debug)]
pub struct ExprPair {
    pub first: ExprGenerator,
    pub second: ExprGenerator,
}

impl AstGenerator<ast::ExprPair> for ExprPair {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::ExprPair {
        let first = Box::new(self.first.gen_ast(ctx));
        let second = Box::new(self.second.gen_ast(ctx));
        ast::ExprPair { first, second }
    }
}

#[derive(Clone, Debug)]
pub struct SimpleCase {
    pub expr: ExprGenerator,
    pub cases: Vec<ExprPair>,
    pub default: Option<ExprGenerator>,
}

impl AstGenerator<ast::Expr> for SimpleCase {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let expr = Box::new(self.expr.gen_ast(ctx));
        let cases = self.cases.iter().map(|ep| ep.gen_ast(ctx)).collect();
        let default = self
            .default
            .as_ref()
            .map(|default| Box::new(default.gen_ast(ctx)));
        let node = ast::SimpleCase {
            expr,
            cases,
            default,
        };
        let node = ctx.node(ast::Case::SimpleCase(node));
        ast::Expr::Case(node)
    }
}

#[derive(Clone, Debug)]
pub struct SearchedCase {
    pub cases: Vec<ExprPair>,
    pub default: Option<ExprGenerator>,
}

impl AstGenerator<ast::Expr> for SearchedCase {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let cases = self.cases.iter().map(|ep| ep.gen_ast(ctx)).collect();
        let default = self
            .default
            .as_ref()
            .map(|default| Box::new(default.gen_ast(ctx)));
        let node = ast::SearchedCase { cases, default };
        let node = ctx.node(ast::Case::SearchedCase(node));
        ast::Expr::Case(node)
    }
}

// TODO VarRef

#[derive(Clone, Debug)]
pub struct StructExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for StructExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Struct(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct BagExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for BagExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Bag(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct ListExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for ListExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::List(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct SexpExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for SexpExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Sexp(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct PathExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for PathExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Path(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for CallExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::Call(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct CallAggExpr {
    // TODO
}
impl AstGenerator<ast::Expr> for CallAggExpr {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Expr {
        ast::Expr::CallAgg(todo!())
    }
}

#[derive(Clone, Debug)]
pub struct QueryExpr {
    query: QueryGenerator,
}
impl AstGenerator<ast::Expr> for QueryExpr {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let query = self.query.gen_ast(ctx);
        let node = ctx.node(query);
        ast::Expr::Query(node)
    }
}
