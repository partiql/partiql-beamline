use clap::{Args, Subcommand};
use partiql_ast::ast;
use partiql_beamline::sim::DatasetTypeMapping;
use partiql_beamline_query::strategy::query::SelectFromWhereBuilder;
use partiql_beamline_query::strategy::{DynStrategy, QueryStrategy, StrategyBoxed, StrategyResult};

mod exclude;
mod project;
mod table_filter;

pub use exclude::*;
use partiql_beamline_query::strategy::project::ProjectStarBuilder;
pub use project::*;
pub use table_filter::*;

pub trait IntoStrategy<Input, Ast> {
    fn into_strategy(self) -> StrategyResult<DynStrategy<Input, Ast>>;
}

/// Query Generation Strategy to use to generate queries
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum QueryGenStrategy {
    /// A `SELECT *` query with randomly generated filters
    RandSelectAllFW(QueryGenStratBasicRandomSelectAllFW),
    /// A Select-From-Where query with randomly generated projections and filters
    RandSFW(QueryGenStratBasicRandomSFW),
    /// A `SELECT *` query with randomly generated excludes and filters
    RandSelectAllEFW(QueryGenStratBasicRandomSelectAllEFW),
    /// A Select-Exclude-From-Where query with randomly generated projections, excludes, and filters
    RandSEFW(QueryGenStratBasicRandomSEFW),
}

impl IntoStrategy<DatasetTypeMapping, ast::Query> for QueryGenStrategy {
    fn into_strategy(self) -> StrategyResult<QueryStrategy> {
        match self {
            QueryGenStrategy::RandSelectAllFW(inner) => inner.into_strategy(),
            QueryGenStrategy::RandSFW(inner) => inner.into_strategy(),
            QueryGenStrategy::RandSelectAllEFW(inner) => inner.into_strategy(),
            QueryGenStrategy::RandSEFW(inner) => inner.into_strategy(),
        }
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct QueryGenStratBasicRandomSelectAllFW {
    #[command(flatten)]
    pub table_filter: TableFilterArg,
}

impl IntoStrategy<DatasetTypeMapping, ast::Query> for QueryGenStratBasicRandomSelectAllFW {
    fn into_strategy(self) -> StrategyResult<QueryStrategy> {
        let projections = ProjectStarBuilder::default().build()?.sboxed();
        let table_filter = self.table_filter.into_strategy()?;
        let sfw = SelectFromWhereBuilder::default()
            .projections(projections)
            .table_filter(table_filter)
            .build()?;
        Ok(sfw.sboxed())
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct QueryGenStratBasicRandomSFW {
    #[command(flatten)]
    pub projections: ProjectListArg,
    #[command(flatten)]
    pub table_filter: TableFilterArg,
}

impl IntoStrategy<DatasetTypeMapping, ast::Query>
    for crate::query_gen::QueryGenStratBasicRandomSFW
{
    fn into_strategy(self) -> StrategyResult<QueryStrategy> {
        let projections = self.projections.into_strategy()?;
        let table_filter = self.table_filter.into_strategy()?;
        let sfw = SelectFromWhereBuilder::default()
            .projections(projections)
            .table_filter(table_filter)
            .build()?;
        Ok(sfw.sboxed())
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct QueryGenStratBasicRandomSelectAllEFW {
    #[command(flatten)]
    pub exclusions: ExcludeListArg,
    #[command(flatten)]
    pub table_filter: TableFilterArg,
}

impl IntoStrategy<DatasetTypeMapping, ast::Query> for QueryGenStratBasicRandomSelectAllEFW {
    fn into_strategy(self) -> StrategyResult<QueryStrategy> {
        let projections = ProjectStarBuilder::default().build()?.sboxed();
        let exclusions = self.exclusions.into_strategy()?;
        let table_filter = self.table_filter.into_strategy()?;
        let sfw = SelectFromWhereBuilder::default()
            .projections(projections)
            .exclusions(exclusions)
            .table_filter(table_filter)
            .build()?;
        Ok(sfw.sboxed())
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct QueryGenStratBasicRandomSEFW {
    #[command(flatten)]
    pub projections: ProjectListArg,
    #[command(flatten)]
    pub exclusions: ExcludeListArg,
    #[command(flatten)]
    pub table_filter: TableFilterArg,
}

impl IntoStrategy<DatasetTypeMapping, ast::Query> for QueryGenStratBasicRandomSEFW {
    fn into_strategy(self) -> StrategyResult<QueryStrategy> {
        let projections = self.projections.into_strategy()?;
        let exclusions = self.exclusions.into_strategy()?;
        let table_filter = self.table_filter.into_strategy()?;
        let sfw = SelectFromWhereBuilder::default()
            .projections(projections)
            .exclusions(exclusions)
            .table_filter(table_filter)
            .build()?;
        Ok(sfw.sboxed())
    }
}
