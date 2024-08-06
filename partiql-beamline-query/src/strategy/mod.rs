use crate::generator::DynAstGenerator;
use crate::strategy::path::PathGenSpecBuilderError;
use crate::strategy::predicate::PathPredicateGenSpecBuilderError;
use derive_builder::UninitializedFieldError;
use dyn_clone::DynClone;
use miette::Diagnostic;
use partiql_ast::ast;
use partiql_beamline::gen::DataGenerationError;
use partiql_beamline::sim::{DatasetTypeMapping, NameAndShape};
use rand_pcg::Pcg64Mcg;
use std::fmt::Debug;
use thiserror::Error;

pub mod exclude;
pub mod path;
pub mod predicate;
pub mod project;
pub mod query;
pub mod where_clause;

#[doc = "Error type for Strategy Builders"]
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum StrategyBuilderError {
    #[error("Path Generation error: {0}")]
    PathSpec(#[from] PathGenSpecBuilderError),
    #[error("Uninitialized field `{0}`")]
    UninitializedField(&'static str),
    #[error("Validate error `{0}`")]
    ValidationError(String),
}

impl From<String> for StrategyBuilderError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}
impl From<UninitializedFieldError> for StrategyBuilderError {
    fn from(err: UninitializedFieldError) -> Self {
        Self::UninitializedField(err.field_name())
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Query Strategy Error")]
#[non_exhaustive]
pub enum StrategyError {
    #[error("Dataset Cardinality Error: expected `{expected}`, but was `{actual}`")]
    DataSetCardinality { expected: usize, actual: usize },
    #[error("Path Generation error: {0}")]
    Path(#[from] PathGenSpecBuilderError),
    #[error("Predicate Generation error: {0}")]
    Predicate(#[from] PathPredicateGenSpecBuilderError),
    #[error("Strategy Builder error: {0}")]
    StrategyBuilder(#[from] StrategyBuilderError),
    #[error("Random Generator error: {0}")]
    Rand(#[from] rand::Error),
    #[error("Stats Generator error: {0}")]
    Stats(#[from] statrs::StatsError),
    #[error("Data Generation error: {0}")]
    DataGen(#[from] DataGenerationError),
    #[error("Projection Paths error: {0}")]
    ProjectPaths(String),
    #[error("Exclude Paths error: {0}")]
    ExcludePaths(String),
    #[error("Predicate Paths error: {0}")]
    PredicatePaths(String),
    #[error("Other: {0}")]
    Other(String),
}

pub type StrategyResult<T> = Result<T, StrategyError>;

pub trait Strategy<Input, Ast>: Debug + DynClone {
    fn build(&self, input: &Input, rng: Pcg64Mcg) -> StrategyResult<DynAstGenerator<Ast>>;
}

pub trait StrategyBoxed<Input, Ast>: Strategy<Input, Ast>
where
    Self: 'static,
{
    fn sboxed(self) -> Box<dyn Strategy<Input, Ast>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T, Ast, Input> StrategyBoxed<Input, Ast> for T where T: Strategy<Input, Ast> + 'static {}

pub type DynStrategy<Input, Ast> = Box<dyn Strategy<Input, Ast>>;
pub type QueryStrategy = DynStrategy<DatasetTypeMapping, ast::Query>;
dyn_clone::clone_trait_object!(Strategy<DatasetTypeMapping, ast::Query>);

pub type SingleTableQueryStrategy = DynStrategy<NameAndShape, ast::Query>;
dyn_clone::clone_trait_object!(Strategy<NameAndShape, ast::Query>);

pub type Projections = DynStrategy<NameAndShape, ast::Projection>;
dyn_clone::clone_trait_object!(Strategy<NameAndShape, ast::Projection>);

pub type Exclusions = DynStrategy<NameAndShape, ast::Exclusion>;
dyn_clone::clone_trait_object!(Strategy<NameAndShape, ast::Exclusion>);

pub type TableFilter = DynStrategy<NameAndShape, ast::WhereClause>;
dyn_clone::clone_trait_object!(Strategy<NameAndShape, ast::WhereClause>);

// Blanket impl to wrap all single `SingleTableQueryStrategy` into `QueryStrategy`s
impl<T> Strategy<DatasetTypeMapping, ast::Query> for T
where
    T: Strategy<NameAndShape, ast::Query> + 'static,
{
    fn build(
        &self,
        input: &DatasetTypeMapping,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Query>> {
        if input.len() != 1 {
            Err(StrategyError::DataSetCardinality {
                expected: 1,
                actual: input.len(),
            })
        } else {
            let (name, shape) = input.into_iter().next().unwrap();
            let data = NameAndShape {
                name: name.clone(),
                shape: shape.clone(),
            };
            self.build(&data, rng)
        }
    }
}
