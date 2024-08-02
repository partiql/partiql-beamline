use crate::generator::types::StaticTypeOperations;
use crate::generator::value::SimpleRandomValueGenerator;
use crate::generator::{
    AstGenContext, AstGenerator, DatasetPredicates, ExprGenerator, PathPredicates, Predicate,
    PredicateDirection,
};
use partiql_ast::ast;
use partiql_ast::ast::Expr;
use partiql_types::PartiqlShape;
use rand::distributions::Distribution;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use statrs::distribution::{Bernoulli, DiscreteUniform};
use std::cell::RefCell;
use std::ops::DerefMut;

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

#[derive(Clone, Debug)]
pub struct PathRowFilter {
    pub paths: DatasetPredicates,
    pub amount: statrs::distribution::DiscreteUniform,
    pub rng: RefCell<Pcg64Mcg>,
}

impl AstGenerator<ast::WhereClause> for PathRowFilter {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::WhereClause {
        let mut rng = self.rng.borrow_mut();
        let mut rng = rng.deref_mut();
        let num = self.amount.sample(rng) as usize;
        let name = self.paths.name.as_str();

        let logical_sel = Bernoulli::new(0.5).expect("sample");
        let path_sel =
            DiscreteUniform::new(0, (self.paths.paths.len() - 1) as i64).expect("sample");
        let idxs: Vec<usize> = std::iter::repeat_with(|| path_sel.sample(rng) as usize)
            .take(num)
            .collect();
        let mut exprs: Vec<_> = idxs
            .into_iter()
            .map(|idx| {
                let rng = RefCell::new(Pcg64Mcg::from_rng(&mut rng).unwrap());
                let predicates = self.paths.paths[idx].clone();
                Box::new(
                    PredicateGen {
                        dataset: name.to_string(),
                        predicates,
                        rng,
                    }
                    .gen_ast(ctx),
                )
            })
            .collect();

        while exprs.len() >= 2 {
            let next = rng.gen_range(0..exprs.len());
            let lhs = exprs.remove(next);
            let next = rng.gen_range(0..exprs.len());
            let rhs = exprs.remove(next);
            let is_and = logical_sel.sample(rng) > 0f64;
            let kind = if is_and {
                ast::BinOpKind::And
            } else {
                ast::BinOpKind::Or
            };
            let expr = Box::new(ast::Expr::BinOp(ctx.node(ast::BinOp { kind, lhs, rhs })));
            exprs.push(expr);
        }

        let expr = exprs.into_iter().next().unwrap_or_else(|| {
            let is_true = logical_sel.sample(rng) > 0f64;
            Box::new(ast::Expr::Lit(ctx.node(ast::Lit::BoolLit(is_true))))
        });

        ast::WhereClause { expr }
    }
}

#[derive(Clone, Debug)]
pub struct PredicateGen {
    pub dataset: String,
    pub predicates: PathPredicates,
    pub rng: RefCell<Pcg64Mcg>,
}

impl AstGenerator<ast::Expr> for PredicateGen {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Expr {
        let mut rng_borrow = self.rng.borrow_mut();
        let rng = rng_borrow.deref_mut();

        let PathPredicates {
            path_and_shape,
            predicates,
        } = &self.predicates;
        let pred_sel = DiscreteUniform::new(0, (predicates.len() - 1) as i64).expect("sample");
        let pred_idx = pred_sel.sample(rng) as usize;
        let predicate = &predicates[pred_idx];

        let path: Box<ast::Expr> = Box::new((self.dataset.as_str(), path_and_shape).gen_ast(ctx));
        let shape = &path_and_shape.shape;
        drop(rng_borrow);

        self.test_for(path, shape, predicate, ctx)
    }
}

impl PredicateGen {
    fn test_for(
        &self,
        path: Box<Expr>,
        shape: &PartiqlShape,
        predicate: &Predicate,
        ctx: &AstGenContext,
    ) -> ast::Expr {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();

        let val_gen = SimpleRandomValueGenerator::new();

        let between =
            |value, from, to| ast::Expr::Between(ctx.node(ast::Between { value, from, to }));
        let like = |value, pattern, escape| {
            ast::Expr::Like(ctx.node(ast::Like {
                value,
                pattern,
                escape,
            }))
        };
        let in_expr = |lhs, rhs| ast::Expr::In(ctx.node(ast::In { lhs, rhs }));
        let binop = |kind, lhs, rhs| ast::Expr::BinOp(ctx.node(ast::BinOp { kind, lhs, rhs }));
        let uniop = |kind, expr| ast::Expr::UniOp(ctx.node(ast::UniOp { kind, expr }));
        let direction = |flag: &_, op| match flag {
            PredicateDirection::Normal => op,
            PredicateDirection::Inverted => uniop(ast::UniOpKind::Not, Box::new(op)),
        };

        match (predicate, shape) {
            (Predicate::AND, _) => binop(ast::BinOpKind::And, path, val_gen.rand_bool(rng, ctx)),
            (Predicate::OR, _) => binop(ast::BinOpKind::Or, path, val_gen.rand_bool(rng, ctx)),
            (Predicate::NOT, _) => uniop(ast::UniOpKind::Not, path),
            (Predicate::EQ, shape) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Eq, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::NEQ, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Ne, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::LTE, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Lte, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::LT, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Lt, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::GTE, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Gte, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::GT, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Gt, path, val_gen.rand_val(ty, rng, ctx))
            }
            (Predicate::BETWEEN, _) => {
                let ty = crate::generator::types::flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                between(
                    path,
                    val_gen.rand_val(ty, rng, ctx),
                    val_gen.rand_val(ty, rng, ctx),
                )
            }
            (Predicate::IS_NULL(flag), _) => {
                let null = Box::new(ast::Expr::Lit(ctx.node(ast::Lit::Null)));
                let op = binop(ast::BinOpKind::Is, path, null);
                direction(flag, op)
            }
            (Predicate::IS_MISSING(flag), _) => {
                let null = Box::new(ast::Expr::Lit(ctx.node(ast::Lit::Missing)));
                let op = binop(ast::BinOpKind::Is, path, null);
                direction(flag, op)
            }
            (Predicate::LIKE(flag), _) => {
                let pattern = val_gen.rand_text(rng, ctx);
                let like = like(path, pattern, None);
                direction(flag, like)
            }
            (Predicate::IN(flag), _) => {
                let in_expr = in_expr(path, val_gen.rand_array(shape, rng, ctx));
                direction(flag, in_expr)
            }
        }
    }
}
