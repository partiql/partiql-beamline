use crate::generator::path::DatasetPaths;
use crate::generator::{AstGenContext, AstGenerator};
use partiql_ast::ast;
use rand::distributions::Distribution;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;
use std::ops::DerefMut;

#[derive(Clone, Debug)]
pub struct SelectStar {}

impl AstGenerator<ast::Projection> for SelectStar {
    fn gen_ast(&self, _ctx: &AstGenContext) -> ast::Projection {
        ast::Projection {
            kind: ast::ProjectionKind::ProjectStar,
            setq: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectPaths {
    pub paths: DatasetPaths,
    pub amount: statrs::distribution::DiscreteUniform,
    pub rng: RefCell<Pcg64Mcg>,
}

impl AstGenerator<ast::Projection> for SelectPaths {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Projection {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let num = self.amount.sample(rng) as usize;
        let name = self.paths.name.as_str();

        let path_sel =
            DiscreteUniform::new(0, (self.paths.paths.len() - 1) as i64).expect("sample");
        let mut items = Vec::with_capacity(num);
        for _ in 0..num {
            let idx = path_sel.sample(rng) as usize;
            let path_and_shape = &self.paths.paths[idx];
            items.push((name, path_and_shape).gen_node(ctx));
        }

        ast::Projection {
            kind: ast::ProjectionKind::ProjectList(items),
            setq: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExcludePaths {
    pub paths: DatasetPaths,
    pub amount: statrs::distribution::DiscreteUniform,
    pub rng: RefCell<Pcg64Mcg>,
}

impl AstGenerator<ast::Exclusion> for ExcludePaths {
    fn gen_ast(&self, ctx: &AstGenContext) -> ast::Exclusion {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let num = self.amount.sample(rng) as usize;
        let name = self.paths.name.as_str();

        let path_sel =
            DiscreteUniform::new(0, (self.paths.paths.len() - 1) as i64).expect("sample");
        let mut items = Vec::with_capacity(num);
        for _ in 0..num {
            let idx = path_sel.sample(rng) as usize;
            let path_and_shape = &self.paths.paths[idx];
            items.push((name, path_and_shape).gen_node(ctx));
        }

        ast::Exclusion { items }
    }
}
