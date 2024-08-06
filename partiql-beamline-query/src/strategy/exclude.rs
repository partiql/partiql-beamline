use crate::generator::{AstGeneratorBoxed, DynAstGenerator, ExcludePaths};
use crate::strategy::path::{PathGenSpec, PathGenSpecBuilder, PathGenSpecBuilderError};
use crate::strategy::{Strategy, StrategyResult};
use crate::strategy::{StrategyBuilderError, StrategyError};
use derive_builder::Builder;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;
use std::ops::Bound;

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(error = "StrategyBuilderError", validate = "Self::validate"))]
pub struct RandomExcludeList {
    pub min_items: u8,
    pub max_items: u8,

    #[builder(default = "Self::default_paths()?.build()?")]
    pub path_spec: PathGenSpec,
}

impl RandomExcludeListBuilder {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(min) = self.min_items {
            if min < 1 {
                return Err("Exclude list minimum must be greater than 0".to_string());
            }
        }

        Ok(())
    }

    pub fn default_paths() -> Result<PathGenSpecBuilder, PathGenSpecBuilderError> {
        let mut builder: PathGenSpecBuilder = PathGenSpecBuilder::default();
        builder
            .min_depth(Bound::Included(2))
            .max_depth(Bound::Unbounded);
        Ok(builder)
    }
}

impl Strategy<NameAndShape, ast::Exclusion> for RandomExcludeList {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Exclusion>> {
        let rng = RefCell::new(Pcg64Mcg::from_rng(rng.clone())?);
        let amount = DiscreteUniform::new(self.min_items as i64, self.max_items as i64)?;
        let paths = self.path_spec.paths_for_dataset(data)?;

        if paths.paths.is_empty() {
            Err(StrategyError::ExcludePaths(
                "Configuration leaves no valid paths available for use in exclusions.".to_string(),
            ))
        } else {
            Ok(ExcludePaths { rng, amount, paths }.agboxed())
        }
    }
}
