use itertools::Itertools;
use miette::IntoDiagnostic;
use partiql_beamline::sim::{
    DatasetTypeMapping, ISim, NameAndShape, SimBuilder, SimConfigBuilder, SimContext,
};
use partiql_beamline::source::SimSource;
use partiql_beamline_query::generator::{AstGenContext, FromTable};
use partiql_beamline_query::generator::{AstGeneratorBoxed, BasicSFW, QueryGenerator, RowFilter};
use partiql_beamline_query::generator::{BinOp, ConstantLiteral};
use partiql_beamline_query::generator::{DynAstGenerator, SelectStar};
use partiql_beamline_query::strategy::exclude::RandomExcludeListBuilder;
use partiql_beamline_query::strategy::path::{
    PathGenSpec, PathGenSpecBuilder, PathStepFlags, PathTypeFlags,
};
use partiql_beamline_query::strategy::predicate::PredicateFlags;
use partiql_beamline_query::strategy::project::{ProjectStar, RandomProjectListBuilder};
use partiql_beamline_query::strategy::query::SelectFromWhereBuilder;
use partiql_beamline_query::strategy::where_clause::{RandomRowFilter, RandomRowPredicateBuilder};
use partiql_beamline_query::strategy::{QueryStrategy, StrategyBoxed};
use partiql_beamline_query::{QueryTextGenerator, QueryTextGeneratorConfigBuilder};
use partiql_types::{BagType, PartiqlShape, StaticType};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use std::collections::Bound;
use std::ops::Sub;
use time::OffsetDateTime;

#[cfg(test)]
const SCRIPT_SIMPLE_TRANSACTIONS: &[u8] =
    include_bytes!("../../partiql-beamline-sim/tests/scripts/simple_transactions.ion");
#[cfg(test)]
const SCRIPT_TRANSACTIONS: &[u8] =
    include_bytes!("../../partiql-beamline-sim/tests/scripts/transactions.ion");

macro_rules! script_data {
    ($file_basename:expr $(,)?) => {
        include_bytes!(concat!(
            "../../partiql-beamline-sim/tests/scripts/",
            $file_basename,
            ".ion"
        ))
    };
}

macro_rules! exemplar_data {
    ($file_basename:expr $(,)?) => {
        include_bytes!(concat!(
            "../../partiql-beamline-sim/tests/data/exemplar/",
            $file_basename,
            ".ion"
        ))
    };
}

macro_rules! test_data {
    ($file_basename:expr $(,)?) => {
        (script_data!($file_basename), exemplar_data!($file_basename))
    };
}

#[track_caller]
#[inline]
fn query_text_test(
    name: &str,
    ast_gen: DynAstGenerator<partiql_ast::ast::Query>,
) -> miette::Result<()> {
    let config = SimConfigBuilder::default().build()?;
    let ctx = SimContext::new(config)?;
    let ctx = AstGenContext::new(ctx);

    let gen = QueryTextGenerator { ast_gen, ctx };
    query_text_gen_test(name, gen)
}

#[track_caller]
fn get_generator(
    seed: u64,
    name: &str,
    script: &[u8],
    strat: QueryStrategy,
) -> miette::Result<QueryTextGenerator> {
    let config = SimConfigBuilder::default().seed(seed).build()?;
    let source = SimSource::new(name, script)?;

    QueryTextGeneratorConfigBuilder::default()
        .config(config)
        .script(source)
        .strategy(strat)
        .build()
        .into_diagnostic()?
        .to_generator()
        .into_diagnostic()
}

#[track_caller]
#[inline]
fn query_text_gen_test(name: &str, gen: QueryTextGenerator) -> miette::Result<()> {
    let queries = (1..=5).map(|_| gen.generate(20).unwrap()).join("\n\n");

    insta::assert_snapshot!(name, queries);

    Ok(())
}

#[test]
fn simple_ast_gen() -> miette::Result<()> {
    use partiql_ast::ast;

    let project = SelectStar {}.agboxed();
    let from = FromTable {
        name: "Foo".to_string(),
    }
    .agboxed();
    let five = ConstantLiteral::Int8Lit(5).agboxed();
    let eight = ConstantLiteral::Int8Lit(8).agboxed();
    let sum = BinOp {
        kind: ast::BinOpKind::Add,
        lhs: five,
        rhs: eight,
    }
    .agboxed();
    let gt = BinOp {
        kind: ast::BinOpKind::Gt,
        lhs: sum,
        rhs: ConstantLiteral::Int8Lit(7).agboxed(),
    }
    .agboxed();
    let and = BinOp {
        kind: ast::BinOpKind::And,
        lhs: gt.clone(),
        rhs: gt,
    }
    .agboxed();
    let where_clause = Some(RowFilter { expr: and }.agboxed());
    let sfw = BasicSFW {
        project,
        exclude: None,
        from,
        where_clause,
    };
    let sfw: QueryGenerator = sfw.agboxed();

    query_text_test("simple_ast_gen", sfw)
}

#[test]
fn simple_strategy() -> miette::Result<()> {
    let rng = Pcg64Mcg::seed_from_u64(1234);
    let strat = SelectFromWhereBuilder::select_all()
        .build()
        .into_diagnostic()?
        .sboxed();

    let shape = PartiqlShape::new_bag(BagType::new(Box::new(PartiqlShape::Static(
        StaticType::new(partiql_types::Static::Int),
    ))));
    let dataset = DatasetTypeMapping::from([("Table".to_string(), shape)]);
    let gen = strat.build(&dataset, rng).into_diagnostic()?;

    query_text_test("simple_strategy", gen)
}

#[track_caller]
fn shape_from_script(name: &str, script: &[u8]) -> miette::Result<DatasetTypeMapping> {
    let t0 = OffsetDateTime::from_unix_timestamp(1712358177).into_diagnostic()?;
    let cfg = SimConfigBuilder::default()
        .t0(t0)
        .seed(1234)
        .build()
        .into_diagnostic()?;

    let source = SimSource::new(name, script)?;
    let sim = SimBuilder::from_config(cfg.clone(), source)
        .into_diagnostic()?
        .build_multi_dataset()
        .into_diagnostic()?;

    Ok(sim.shape())
}

#[test]
fn simple_random_strategy() -> miette::Result<()> {
    let filter_strat = RandomRowFilter {}.sboxed();
    let strat = SelectFromWhereBuilder::select_all()
        .table_filter(filter_strat)
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_random_strategy",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}

#[track_caller]
#[inline]
fn path_gen_test(
    name: &str,
    data: &DatasetTypeMapping,
    path_spec: &PathGenSpec,
) -> miette::Result<()> {
    for (ds_name, shape) in data {
        let name = format!("path_test_{name}_{ds_name}");
        let dataset = NameAndShape {
            name: name.clone(),
            shape: shape.clone(),
        };
        let paths = path_spec.paths_for_dataset(&dataset).unwrap();
        let paths = paths.paths;

        let output = format!("{paths:#?}");
        insta::assert_snapshot!(name, output);
    }

    Ok(())
}

#[test]
fn path_gen_tests() -> miette::Result<()> {
    let simple = shape_from_script("simple_transactions", SCRIPT_SIMPLE_TRANSACTIONS)?;
    let complex = shape_from_script("transactions", SCRIPT_TRANSACTIONS)?;

    let scalars = PathGenSpecBuilder::default()
        .allowed_internal_steps(PathStepFlags::PathProject | PathStepFlags::PathForEach)
        .allowed_final_types(PathTypeFlags::Scalar)
        .build()
        .unwrap();
    path_gen_test("simple-scalars", &simple, &scalars)?;
    path_gen_test("complex-scalars", &complex, &scalars)?;

    let full = PathGenSpecBuilder::default().build().unwrap();
    path_gen_test("simple-full", &simple, &full)?;
    path_gen_test("complex-full", &complex, &full)?;

    let full_project = PathGenSpecBuilder::default()
        .allowed_internal_steps(PathStepFlags::PathProject | PathStepFlags::PathForEach)
        .build()
        .unwrap();
    path_gen_test("simple-full-project", &simple, &full_project)?;
    path_gen_test("complex-full-project", &complex, &full_project)?;

    let full_depth2 = PathGenSpecBuilder::default()
        .max_depth(Bound::Included(1))
        .build()
        .unwrap();
    path_gen_test("simple-full-depth2", &simple, &full_depth2)?;
    path_gen_test("complex-full-depth2", &complex, &full_depth2)?;

    Ok(())
}

#[test]
fn simple_project_strategy() -> miette::Result<()> {
    let projections = RandomProjectListBuilder::default()
        .min_items(2)
        .max_items(5)
        .build()?;
    let strat = SelectFromWhereBuilder::default()
        .projections(projections.sboxed())
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_project_strategy",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}

#[test]
fn simple_exclude_strategy() -> miette::Result<()> {
    let projections = ProjectStar {};
    let exclusions = RandomExcludeListBuilder::default()
        .min_items(2)
        .max_items(5)
        .path_spec(
            RandomExcludeListBuilder::default_paths()
                .into_diagnostic()?
                .min_depth(Bound::Included(2))
                .build()
                .into_diagnostic()?,
        )
        .build()
        .into_diagnostic()?;
    let strat = SelectFromWhereBuilder::default()
        .projections(projections.sboxed())
        .exclusions(exclusions.sboxed())
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_exclude_strategy",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}

#[test]
fn simple_shallow_exclude_strategy() -> miette::Result<()> {
    let projections = ProjectStar {};
    let path_spec = RandomExcludeListBuilder::default_paths()
        .into_diagnostic()?
        .max_depth(Bound::Included(1))
        .min_depth(Bound::Included(2))
        .build()
        .into_diagnostic()?;
    let exclusions = RandomExcludeListBuilder::default()
        .min_items(2)
        .max_items(5)
        .path_spec(path_spec)
        .build()
        .into_diagnostic()?;
    let strat = SelectFromWhereBuilder::default()
        .projections(projections.sboxed())
        .exclusions(exclusions.sboxed())
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_shallow_exclude_strategy",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}

#[test]
fn simple_random_row_filters() -> miette::Result<()> {
    let projections = ProjectStar {};

    let path_spec = PathGenSpecBuilder::default()
        .min_depth(Bound::Included(2))
        .max_depth(Bound::Unbounded)
        .build()
        .into_diagnostic()?;
    let allowed_predicates = PredicateFlags::all()
        .sub(PredicateFlags::flags_absent())
        .sub(PredicateFlags::flags_logical_connectives());
    let row_filters = RandomRowPredicateBuilder::default()
        .allowed_predicates(allowed_predicates)
        .min_items(2)
        .max_items(5)
        .path_spec(path_spec)
        .build()
        .into_diagnostic()?;
    let strat = SelectFromWhereBuilder::default()
        .projections(projections.sboxed())
        .table_filter(row_filters.sboxed())
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_random_row_filters",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}

#[test]
fn simple_random_scalar_row_filters() -> miette::Result<()> {
    let projections = ProjectStar {};
    let path_spec = PathGenSpecBuilder::default()
        .min_depth(Bound::Included(1))
        .max_depth(Bound::Included(1))
        .allowed_final_types(PathTypeFlags::Scalar)
        .build()
        .into_diagnostic()?;
    let allowed_predicates = PredicateFlags::all()
        .sub(PredicateFlags::flags_absent())
        .sub(PredicateFlags::flags_logical_connectives());
    let row_filters = RandomRowPredicateBuilder::default()
        .allowed_predicates(allowed_predicates)
        .min_items(2)
        .max_items(10)
        .path_spec(path_spec)
        .build()
        .into_diagnostic()?;
    let strat = SelectFromWhereBuilder::default()
        .projections(projections.sboxed())
        .table_filter(row_filters.sboxed())
        .build()
        .into_diagnostic()?
        .sboxed();

    query_text_gen_test(
        "simple_random_scalar_row_filters",
        get_generator(123456, "transactions", SCRIPT_TRANSACTIONS, strat)?,
    )?;

    Ok(())
}
