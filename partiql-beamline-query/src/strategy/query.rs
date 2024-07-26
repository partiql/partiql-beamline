use crate::generator::{AstGeneratorBoxed, BasicSFW, DynAstGenerator, FromTable};
use crate::strategy::project::ProjectStar;
use crate::strategy::{
    Exclusions, Projections, Strategy, StrategyBoxed, StrategyResult, TableFilter,
};
use derive_builder::Builder;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone, Builder)]
pub struct SelectFromWhere {
    pub projections: Projections,
    #[builder(setter(into, strip_option), default)]
    pub exclusions: Option<Exclusions>,
    #[builder(setter(into, strip_option), default)]
    pub table_filter: Option<TableFilter>,
}

impl SelectFromWhereBuilder {
    pub fn select_all() -> Self {
        let mut bld = Self::default();
        bld.projections(ProjectStar {}.sboxed());
        bld
    }
}

impl Strategy<NameAndShape, ast::Query> for SelectFromWhere {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Query>> {
        let prng = Pcg64Mcg::from_rng(rng.clone())?;
        let project = self.projections.build(data, prng)?;
        let erng = Pcg64Mcg::from_rng(rng.clone())?;
        let exclude = self
            .exclusions
            .as_ref()
            .map(|ex| ex.build(data, erng))
            .transpose()?;
        let from = FromTable {
            name: data.name.clone(),
        }
        .agboxed();
        let wrng = Pcg64Mcg::from_rng(rng.clone())?;
        let where_clause = self
            .table_filter
            .as_ref()
            .map(|tf| tf.build(data, wrng))
            .transpose()?;
        let select_star = BasicSFW {
            project,
            exclude,
            from,
            where_clause,
        }
        .agboxed();
        Ok(select_star)
    }
}
