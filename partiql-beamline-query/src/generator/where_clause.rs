use crate::generator::path::DatasetPaths;
use crate::generator::{
    AstGenContext, AstGenerator, DatasetPredicates, ExprGenerator, PathAndShape, PathPredicates,
    Predicate, PredicateDirection,
};
use crate::generator::{AstGeneratorBoxed, BinOp};
use indexmap::IndexSet;
use itertools::Itertools;
use lipsum::{lipsum_with_rng, lipsum_words_with_rng};
use log::log;
use once_cell::sync::Lazy;
use partiql_ast::ast;
use partiql_ast::ast::{CaseSensitivity, Expr, SymbolPrimitive};
use partiql_beamline::gen::text::LoremIpsumImpl;
use partiql_types::{PartiqlShape, Static};
use rand::distributions::Distribution;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use statrs::distribution::{Bernoulli, DiscreteUniform, Uniform};
use std::cell::{OnceCell, RefCell};
use std::ops::DerefMut;
use std::sync::{Mutex, RwLock};

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

        let bool_dist = Bernoulli::new(0.5).expect("sample");
        let int_dist = DiscreteUniform::new(-50, 50).expect("sample");
        let float_dist = Uniform::new(-50.0, 50.0).expect("sample");
        let len_dist = DiscreteUniform::new(1, 3).expect("sample");

        let mut rand_bool = |rng: &mut Pcg64Mcg| {
            Box::new(ast::Expr::Lit(
                ctx.node(ast::Lit::BoolLit(bool_dist.sample(rng) > 0f64)),
            ))
        };
        let mut rand_int = |rng: &mut Pcg64Mcg| {
            Box::new(ast::Expr::Lit(
                ctx.node(ast::Lit::Int64Lit(int_dist.sample(rng) as i64)),
            ))
        };
        let mut rand_float = |rng: &mut Pcg64Mcg| {
            Box::new(ast::Expr::Lit(
                ctx.node(ast::Lit::DoubleLit(float_dist.sample(rng))),
            ))
        };
        let mut rand_text = |rng: &mut Pcg64Mcg| {
            let len = len_dist.sample(rng) as usize;
            let txt = lipsum_words_with_rng(rng, len);
            Box::new(ast::Expr::Lit(ctx.node(ast::Lit::CharStringLit(txt))))
        };
        let mut rand_datetime = |rng: &mut Pcg64Mcg| {
            // TODO datetime
            let utcnow = SymbolPrimitive {
                value: "UTCNOW".to_string(),
                case: CaseSensitivity::CaseInsensitive,
            };
            Box::new(ast::Expr::Call(ctx.node(ast::Call {
                func_name: utcnow,
                args: vec![],
            })))
        };
        let mut rand_struct = |rng: &mut Pcg64Mcg| {
            // TODO struct
            Box::new(ast::Expr::Struct(ctx.node(ast::Struct { fields: vec![] })))
        };
        let mut rand_bag = |rng: &mut Pcg64Mcg| {
            // TODO bag
            Box::new(ast::Expr::Bag(ctx.node(ast::Bag { values: vec![] })))
        };
        let mut rand_array = |rng: &mut Pcg64Mcg| {
            // TODO array
            Box::new(ast::Expr::List(ctx.node(ast::List { values: vec![] })))
        };
        let mut rand_val = |ty: &Static, rng: &mut Pcg64Mcg| match ty {
            Static::Int | Static::Int8 | Static::Int16 | Static::Int32 | Static::Int64 => {
                rand_int(rng)
            }
            Static::Bool => rand_bool(rng),
            Static::Decimal | Static::DecimalP(_, _) => rand_float(rng),
            Static::Float32 | Static::Float64 => rand_float(rng),
            Static::String | Static::StringFixed(_) | Static::StringVarying(_) => rand_text(rng),
            Static::DateTime => rand_datetime(rng),
            Static::Struct(_) => rand_struct(rng),
            Static::Bag(_) => rand_bag(rng),
            Static::Array(_) => rand_array(rng),
        };
        let between =
            |value, from, to| ast::Expr::Between(ctx.node(ast::Between { value, from, to }));
        let like = |value, pattern, escape| {
            ast::Expr::Like(ctx.node(ast::Like {
                value,
                pattern,
                escape,
            }))
        };
        let binop = |kind, lhs, rhs| ast::Expr::BinOp(ctx.node(ast::BinOp { kind, lhs, rhs }));
        let uniop = |kind, expr| ast::Expr::UniOp(ctx.node(ast::UniOp { kind, expr }));
        let direction = |flag: &_, op| match flag {
            PredicateDirection::Normal => op,
            PredicateDirection::Inverted => uniop(ast::UniOpKind::Not, Box::new(op)),
        };

        match (predicate, shape) {
            (Predicate::AND, _) => binop(ast::BinOpKind::And, path, rand_bool(rng)),
            (Predicate::OR, _) => binop(ast::BinOpKind::Or, path, rand_bool(rng)),
            (Predicate::NOT, _) => uniop(ast::UniOpKind::Not, path),
            (Predicate::EQ, shape) => {
                let ty = flatten_types(shape).into_iter().choose(rng).unwrap();
                binop(ast::BinOpKind::Eq, path, rand_val(ty, rng))
            }
            (Predicate::NEQ, _) => {
                let ty = flatten_types(shape).into_iter().choose(rng).unwrap();
                binop(ast::BinOpKind::Ne, path, rand_val(ty, rng))
            }
            (Predicate::LTE, _) => {
                let ty = flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Lte, path, rand_val(ty, rng))
            }
            (Predicate::LT, _) => {
                let ty = flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Lt, path, rand_val(ty, rng))
            }
            (Predicate::GTE, _) => {
                let ty = flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Gte, path, rand_val(ty, rng))
            }
            (Predicate::GT, _) => {
                let ty = flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                binop(ast::BinOpKind::Gt, path, rand_val(ty, rng))
            }
            (Predicate::BETWEEN, _) => {
                let ty = flatten_types(shape)
                    .into_iter()
                    .filter(|ty| ty.is_comparable())
                    .choose(rng)
                    .unwrap();
                between(path, rand_val(ty, rng), rand_val(ty, rng))
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
                let pattern = rand_text(rng);
                let like = like(path, pattern, None);
                direction(flag, like)
            }
            (Predicate::IN(flag), _) => {
                //
                todo!()
            }
        }
    }
}

static Test: [Static; 9] = [
    Static::Int,
    Static::Int8,
    Static::Int16,
    Static::Int32,
    Static::Int64,
    Static::Bool,
    Static::Decimal,
    Static::Float64,
    Static::String,
];

fn flatten_types(shape: &PartiqlShape) -> IndexSet<&Static> {
    let mut dynamic = false;
    let mut candidates = vec![shape];
    let mut flat = IndexSet::new();
    while let Some(candidate) = candidates.pop() {
        match candidate {
            PartiqlShape::Dynamic => dynamic = true,
            PartiqlShape::AnyOf(any) => candidates.extend(any.types()),
            PartiqlShape::Static(sty) => {
                flat.insert(sty.ty());
            }
            PartiqlShape::Undefined => dynamic = true,
        }
    }

    if dynamic {
        flat.extend(Test.iter());
    }

    flat
}

trait StaticTypeOperations {
    fn is_comparable(&self) -> bool;
}

impl StaticTypeOperations for Static {
    fn is_comparable(&self) -> bool {
        matches!(
            self,
            Static::Int
                | Static::Int8
                | Static::Int16
                | Static::Int32
                | Static::Int64
                | Static::Bool
                | Static::Decimal
                | Static::DecimalP(_, _)
                | Static::Float32
                | Static::Float64
                | Static::DateTime
        )
    }
}
