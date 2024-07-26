use crate::generator::{AstGeneratorBoxed, DynAstGenerator, ExcludePaths, SelectPaths, SelectStar};
use crate::strategy::path::{PathGenSpecBuilder, PathStepFlags};
use crate::strategy::{Strategy, StrategyResult};
use derive_builder::Builder;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;
use std::ops::Bound;

#[derive(Debug, Clone, Builder)]
pub struct RandomExcludeList {
    pub min_items: u8,
    pub max_items: u8,
    #[builder(default = " Bound::Unbounded")]
    pub max_depth: Bound<usize>,
}

impl Strategy<NameAndShape, ast::Exclusion> for RandomExcludeList {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Exclusion>> {
        let rng = RefCell::new(Pcg64Mcg::from_rng(rng.clone())?);
        let amount = DiscreteUniform::new(self.min_items as i64, self.max_items as i64)?;
        let paths = PathGenSpecBuilder::default()
            .min_depth(Bound::Included(2))
            .max_depth(self.max_depth)
            .build()?
            .paths_for_dataset(data)?;
        Ok(ExcludePaths { rng, amount, paths }.agboxed())
    }
}
