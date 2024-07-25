use crate::generator::{AstGeneratorBoxed, BasicSFW, DynAstGenerator, FromTable, SelectStar};
use crate::strategy::{Strategy, StrategyResult, TableFilter};
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct SelectAllFromTable {}

impl Strategy<NameAndShape, ast::Query> for SelectAllFromTable {
    fn build(
        &self,
        NameAndShape { name, shape }: NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Query>> {
        let project = SelectStar {}.agboxed();
        let from = FromTable { name }.agboxed();
        let where_clause = None;
        let select_star = BasicSFW {
            project,
            from,
            where_clause,
        }
        .agboxed();
        Ok(select_star)
    }
}

#[derive(Debug, Clone)]
pub struct SelectAllFromFilteredTable {
    pub table_filter: TableFilter,
}

impl Strategy<NameAndShape, ast::Query> for SelectAllFromFilteredTable {
    fn build(
        &self,
        input: NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::Query>> {
        let crng = Pcg64Mcg::from_rng(rng)?;
        let project = SelectStar {}.agboxed();
        let from = FromTable {
            name: input.name.clone(),
        }
        .agboxed();
        let where_clause = Some(self.table_filter.build(input, crng)?);
        let select_star = BasicSFW {
            project,
            from,
            where_clause,
        }
        .agboxed();
        Ok(select_star)
    }
}
