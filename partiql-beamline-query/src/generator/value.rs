use crate::generator::AstGenContext;
use lipsum::lipsum_words_with_rng;
use partiql_ast::ast;
use partiql_ast::ast::{CaseSensitivity, SymbolPrimitive};
use partiql_types::{PartiqlShape, Static, StructType};
use rand::distributions::Distribution;
use rand::prelude::IteratorRandom;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::{Bernoulli, DiscreteUniform, Uniform};

pub struct SimpleRandomValueGenerator {
    bool_dist: Bernoulli,
    int_dist: DiscreteUniform,
    float_dist: Uniform,
    txt_len_dist: DiscreteUniform,
    array_len_dist: DiscreteUniform,
}

impl SimpleRandomValueGenerator {
    pub fn new() -> Self {
        Self {
            bool_dist: Bernoulli::new(0.5).expect("sample"),
            int_dist: DiscreteUniform::new(-50, 50).expect("sample"),
            float_dist: Uniform::new(-50.0, 50.0).expect("sample"),
            txt_len_dist: DiscreteUniform::new(1, 3).expect("sample"),
            array_len_dist: DiscreteUniform::new(2, 5).expect("sample"),
        }
    }

    pub fn rand_bool(&self, rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        Box::new(ast::Expr::Lit(
            ctx.node(ast::Lit::BoolLit(self.bool_dist.sample(rng) > 0f64)),
        ))
    }

    pub fn rand_int(&self, rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        Box::new(ast::Expr::Lit(
            ctx.node(ast::Lit::Int64Lit(self.int_dist.sample(rng) as i64)),
        ))
    }

    pub fn rand_float(&self, rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        Box::new(ast::Expr::Lit(
            ctx.node(ast::Lit::DoubleLit(self.float_dist.sample(rng))),
        ))
    }

    pub fn rand_text(&self, rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        let len = self.txt_len_dist.sample(rng) as usize;
        let txt = lipsum_words_with_rng(rng, len);
        Box::new(ast::Expr::Lit(ctx.node(ast::Lit::CharStringLit(txt))))
    }

    pub fn rand_datetime(&self, _rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        // TODO datetime
        let utcnow = SymbolPrimitive {
            value: "UTCNOW".to_string(),
            case: CaseSensitivity::CaseInsensitive,
        };
        Box::new(ast::Expr::Call(ctx.node(ast::Call {
            func_name: utcnow,
            args: vec![],
        })))
    }

    pub fn rand_struct(
        &self,
        strct: &StructType,
        rng: &mut Pcg64Mcg,
        ctx: &AstGenContext,
    ) -> Box<ast::Expr> {
        let fields = strct
            .fields()
            .map(|field| {
                let name = field.name();
                let ty = field.ty();
                let first = Box::new(ast::Expr::Lit(
                    ctx.node(ast::Lit::CharStringLit(name.to_string())),
                ));
                let second = self.rand_val_from_shape(ty, rng, ctx);
                ast::ExprPair { first, second }
            })
            .collect();
        Box::new(ast::Expr::Struct(ctx.node(ast::Struct { fields })))
    }

    pub fn rand_bag(
        &self,
        shape: &PartiqlShape,
        rng: &mut Pcg64Mcg,
        ctx: &AstGenContext,
    ) -> Box<ast::Expr> {
        let len = self.array_len_dist.sample(rng) as usize;
        let values = std::iter::repeat_with(|| self.rand_val_from_shape(shape, rng, ctx))
            .take(len)
            .collect();
        Box::new(ast::Expr::List(ctx.node(ast::List { values })))
    }

    pub fn rand_static_array(
        &self,
        shape: &PartiqlShape,
        rng: &mut Pcg64Mcg,
        ctx: &AstGenContext,
    ) -> Box<ast::Expr> {
        let len = self.array_len_dist.sample(rng) as usize;
        let values = std::iter::repeat_with(|| self.rand_val_from_shape(shape, rng, ctx))
            .take(len)
            .collect();
        Box::new(ast::Expr::List(ctx.node(ast::List { values })))
    }

    pub fn rand_array(
        &self,
        shape: &PartiqlShape,
        rng: &mut Pcg64Mcg,
        ctx: &AstGenContext,
    ) -> Box<ast::Expr> {
        let len = self.array_len_dist.sample(rng) as usize;
        let values = std::iter::repeat_with(|| self.rand_val_from_shape(shape, rng, ctx))
            .take(len)
            .collect();
        Box::new(ast::Expr::List(ctx.node(ast::List { values })))
    }

    pub fn rand_val(&self, ty: &Static, rng: &mut Pcg64Mcg, ctx: &AstGenContext) -> Box<ast::Expr> {
        match ty {
            Static::Int | Static::Int8 | Static::Int16 | Static::Int32 | Static::Int64 => {
                self.rand_int(rng, ctx)
            }
            Static::Bool => self.rand_bool(rng, ctx),
            Static::Decimal | Static::DecimalP(_, _) => self.rand_float(rng, ctx),
            Static::Float32 | Static::Float64 => self.rand_float(rng, ctx),
            Static::String | Static::StringFixed(_) | Static::StringVarying(_) => {
                self.rand_text(rng, ctx)
            }
            Static::DateTime => self.rand_datetime(rng, ctx),
            Static::Struct(strct) => self.rand_struct(strct, rng, ctx),
            Static::Bag(bag) => self.rand_bag(bag.element_type(), rng, ctx),
            Static::Array(array) => self.rand_array(array.element_type(), rng, ctx),
        }
    }

    pub fn rand_val_from_shape(
        &self,
        shape: &PartiqlShape,
        rng: &mut Pcg64Mcg,
        ctx: &AstGenContext,
    ) -> Box<ast::Expr> {
        let ty = crate::generator::types::flatten_types(shape)
            .into_iter()
            .choose(rng)
            .unwrap();
        self.rand_val(ty, rng, ctx)
    }
}
