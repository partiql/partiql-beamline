use crate::generator::{AstGeneratorBoxed, DynAstGenerator, SelectPaths, SelectStar};
use crate::strategy::path::{PathGenSpecBuilder, PathStepFlags};
use crate::strategy::{Strategy, StrategyResult};
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct ProjectStar {}

impl Strategy<NameAndShape, ast::Projection> for ProjectStar {
    fn build(
        &self,
        _: &NameAndShape,
        _: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Projection>> {
        Ok(SelectStar {}.agboxed())
    }
}

#[derive(Debug, Clone)]
pub struct RandomProjectList {
    pub min_items: u8,
    pub max_items: u8,
}

impl Strategy<NameAndShape, ast::Projection> for RandomProjectList {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Projection>> {
        let rng = RefCell::new(Pcg64Mcg::from_rng(rng.clone())?);
        let amount = DiscreteUniform::new(self.min_items as i64, self.max_items as i64)?;
        let paths = PathGenSpecBuilder::default()
            .allowed_internal_steps(PathStepFlags::all() - PathStepFlags::PathUnpivot)
            .build()?
            .paths_for_dataset(data)?;
        Ok(SelectPaths { rng, amount, paths }.agboxed())
    }
}
