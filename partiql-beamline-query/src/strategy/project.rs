use crate::generator::{AstGeneratorBoxed, DynAstGenerator, SelectPaths, SelectStar};
use crate::strategy::path::{
    PathGenSpec, PathGenSpecBuilder, PathGenSpecBuilderError, PathStepFlags,
};
use crate::strategy::{Strategy, StrategyResult};
use crate::strategy::{StrategyBuilderError, StrategyError};
use derive_builder::Builder;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "StrategyBuilderError"))]
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

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "StrategyBuilderError"))]
pub struct RandomProjectList {
    pub min_items: u8,
    pub max_items: u8,

    #[builder(default = "Self::default_paths()?.build()?")]
    pub path_spec: PathGenSpec,
}

impl RandomProjectListBuilder {
    pub fn default_paths() -> Result<PathGenSpecBuilder, PathGenSpecBuilderError> {
        let mut builder = PathGenSpecBuilder::default();
        builder.allowed_internal_steps(PathStepFlags::all() - PathStepFlags::PathUnpivot);
        Ok(builder)
    }
}

impl Strategy<NameAndShape, ast::Projection> for RandomProjectList {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Projection>> {
        let rng = RefCell::new(Pcg64Mcg::from_rng(rng.clone())?);
        let amount = DiscreteUniform::new(self.min_items as i64, self.max_items as i64)?;
        let paths = self.path_spec.paths_for_dataset(data)?;

        if paths.paths.is_empty() {
            Err(StrategyError::ProjectPaths(
                "Configuration leaves no valid paths available for use in projections.".to_string(),
            ))
        } else {
            Ok(SelectPaths { rng, amount, paths }.agboxed())
        }
    }
}
