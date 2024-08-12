use crate::generator::{AstGenContext, QueryGenerator};
use crate::strategy::{QueryStrategy, StrategyError};
use derive_builder::Builder;
use miette::Diagnostic;
use partiql_ast::pretty::{ToPretty, ToPrettyError};
use partiql_beamline::sim::{ISim, SimBuilder, SimConfig, SimConfigError, SimContext, SimError};
use partiql_beamline::source::SimSource;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("Query Generation Error")]
#[non_exhaustive]
pub enum QueryGenError {
    #[error("QueryGenerator: {0}")]
    QueryGenerator(#[from] QueryTextGeneratorConfigBuilderError),
    #[error("StrategyError: {0}")]
    Strategy(#[from] StrategyError),
    #[error("SimConfigError: {0}")]
    SimConfig(#[from] SimConfigError),
    #[error("SimError: {0}")]
    Sim(#[from] SimError),
    #[error("ToPrettyError: {0}")]
    ToPrettyError(#[from] ToPrettyError),
    #[error("Unknown: {0}")]
    Unknown(String),
}

/// Result of a Query Generation operation
pub type QueryGenResult<T> = Result<T, QueryGenError>;

#[derive(Debug, Clone, Builder)]
pub struct QueryTextGeneratorConfig {
    config: SimConfig,
    script: SimSource,
    strategy: QueryStrategy,
}

impl QueryTextGeneratorConfig {
    pub fn to_generator(self) -> QueryGenResult<QueryTextGenerator> {
        let root_rng = Pcg64Mcg::seed_from_u64(self.config.seed);
        let shape = SimBuilder::from_config(self.config.clone(), self.script)?
            .build_multi_dataset()?
            .shape();
        let ast_gen = self.strategy.build(&shape, root_rng)?;
        let ctx = AstGenContext::new(SimContext::new(self.config)?);
        Ok(QueryTextGenerator { ctx, ast_gen })
    }
}

pub struct QueryTextGenerator {
    pub ctx: AstGenContext,
    pub ast_gen: QueryGenerator,
}

impl QueryTextGenerator {
    pub fn generate(&self, width: usize) -> QueryGenResult<String> {
        let node = self.ast_gen.gen_node(&self.ctx);
        Ok(node.to_pretty_string(width)?)
    }
}

pub mod generator;
pub mod strategy;
