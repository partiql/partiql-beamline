use crate::generator::{
    AstGeneratorBoxed, DatasetPaths, DatasetPredicates, DynAstGenerator, GeneratedLiteral,
    PathAndShape, PathPredicates, PathRowFilter, RowFilter,
};
use crate::strategy::path::{PathGenSpec, PathGenSpecBuilder};
use crate::strategy::predicate::{
    PathPredicateGenSpec, PathPredicateGenSpecBuilder, PredicateFlags,
};
use crate::strategy::{Strategy, StrategyResult};
use derive_builder::Builder;
use partiql_ast::ast;
use partiql_beamline::gen::distributions::{Density, Meta};
use partiql_beamline::gen::simple::SimpleBool;
use partiql_beamline::gen::ValueGeneratorBoxed;
use partiql_beamline::sim::NameAndShape;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use statrs::distribution::DiscreteUniform;
use std::cell::RefCell;
use std::collections::Bound;

#[derive(Debug, Clone)]
pub struct RandomRowFilter {}

impl Strategy<NameAndShape, ast::WhereClause> for RandomRowFilter {
    fn build(
        &self,
        _input: &NameAndShape,
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

#[derive(Debug, Clone, Builder)]
pub struct RandomRowPredicate {
    #[builder(default = "PredicateFlags::all()")]
    pub allowed_predicates: PredicateFlags,
    pub min_items: u8,
    pub max_items: u8,
    pub path_spec: PathGenSpec,
}

impl Strategy<NameAndShape, ast::WhereClause> for RandomRowPredicate {
    fn build(
        &self,
        data: &NameAndShape,
        rng: Pcg64Mcg,
    ) -> StrategyResult<DynAstGenerator<ast::WhereClause>> {
        let rng = RefCell::new(Pcg64Mcg::from_rng(rng.clone())?);
        let amount = DiscreteUniform::new(self.min_items as i64, self.max_items as i64)?;

        let pred_spec = PathPredicateGenSpecBuilder::default()
            .allowed_predicates(self.allowed_predicates)
            .path_spec(self.path_spec.clone())
            .build()?;
        let paths = pred_spec.predicate_paths_for_dataset(data)?;

        Ok(PathRowFilter { paths, amount, rng }.agboxed())
    }
}
