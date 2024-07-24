use crate::generator::{AstGeneratorBoxed, DynAstGenerator, GeneratedLiteral, RowFilter};
use crate::strategy::{Strategy, StrategyResult};
use partiql_ast::ast;
use partiql_beamline::gen::distributions::{Density, Meta};
use partiql_beamline::gen::simple::SimpleBool;
use partiql_beamline::gen::ValueGeneratorBoxed;
use partiql_beamline::sim::NameAndShape;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct RandomRowFilter {}

impl Strategy<NameAndShape, ast::WhereClause> for RandomRowFilter {
    fn build(
        &self,
        _input: NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::WhereClause>> {
        let meta = Meta {
            script_path: "".to_string(),
        };
        let density = Density::new(Some(0.0), None, 1.0).unwrap();
        let b = SimpleBool::new(0.5, rng, meta, density)?;
        let expr = GeneratedLiteral { value: b.boxed() }.agboxed();
        Ok(RowFilter { expr }.agboxed())
    }
}
