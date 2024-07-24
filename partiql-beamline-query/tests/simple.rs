use itertools::Itertools;
use miette::IntoDiagnostic;
use partiql_ast::pretty::ToPretty;
use partiql_beamline::sim::{
    DatasetTypeMapping, ISim, SimBuilder, SimConfigBuilder, SimContext, DATETIME_FORMAT,
};
use partiql_beamline_query::generator::{AstGenContext, FromTable};
use partiql_beamline_query::generator::{
    AstGenerator, AstGeneratorBoxed, BasicSFW, QueryGenerator, RowFilter,
};
use partiql_beamline_query::generator::{BinOp, ConstantLiteral};
use partiql_beamline_query::generator::{DynAstGenerator, SelectStar};
use partiql_beamline_query::strategy::query::{SelectAllFromFilteredTable, SelectAllFromTable};
use partiql_beamline_query::strategy::where_clause::RandomRowFilter;
use partiql_beamline_query::strategy::StrategyBoxed;
use partiql_types::{BagType, PartiqlShape, StaticType};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use time::macros::datetime;
use time::OffsetDateTime;

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
    gen: &DynAstGenerator<partiql_ast::ast::Query>,
) -> miette::Result<()> {
    let config = SimConfigBuilder::default().build()?;
    let ctx = SimContext::new(config)?;
    let ctx = AstGenContext::new(ctx);

    let queries = (1..=5)
        .map(|n| gen.gen_node(&ctx).to_pretty_string(20).unwrap())
        .join("\n\n");

    insta::assert_snapshot!(name, queries);

    Ok(())
}

#[test]
fn simple_ast_gen() -> miette::Result<()> {
    use partiql_ast::ast;
    use partiql_ast::builder::NodeBuilderWithAutoId;
    use partiql_ast::pretty::ToPretty;

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
        from,
        where_clause,
    };
    let sfw: QueryGenerator = sfw.agboxed();

    query_text_test("simple_ast_gen", &sfw)
}

#[test]
fn simple_strategy() -> miette::Result<()> {
    let rng = Pcg64Mcg::seed_from_u64(1234);
    let strat = SelectAllFromTable {}.sboxed();

    let shape = PartiqlShape::new_bag(BagType::new(Box::new(PartiqlShape::Static(
        StaticType::new(partiql_types::Static::Int),
    ))));
    let dataset = DatasetTypeMapping::from([("Table".to_string(), shape)]);
    let gen = strat.build(dataset, rng).into_diagnostic()?;

    query_text_test("simple_strategy", &gen)
}

#[track_caller]
fn shape_from_script(script: &[u8]) -> miette::Result<DatasetTypeMapping> {
    let t0 = OffsetDateTime::from_unix_timestamp(1712358177).into_diagnostic()?;
    let cfg = SimConfigBuilder::default()
        .t0(t0)
        .seed(1234)
        .build()
        .into_diagnostic()?;

    let sim = SimBuilder::from_config(cfg.clone(), script)
        .into_diagnostic()?
        .build_multi_dataset()
        .into_diagnostic()?;

    Ok(sim.shape())
}

#[cfg(test)]
const SCRIPT_SIMPLE_TRANSACTIONS: &[u8] =
    include_bytes!("../../partiql-beamline-sim/tests/scripts/simple_transactions.ion");
#[cfg(test)]
const SCRIPT_TRANSACTIONS: &[u8] =
    include_bytes!("../../partiql-beamline-sim/tests/scripts/transactions.ion");

#[test]
fn simple_random_strategy() -> miette::Result<()> {
    let rng = Pcg64Mcg::seed_from_u64(123456);
    let shape = shape_from_script(SCRIPT_SIMPLE_TRANSACTIONS)?;

    let filter_strat = RandomRowFilter {}.sboxed();
    let strat = SelectAllFromFilteredTable {
        table_filter: filter_strat,
    }
    .sboxed();
    let gen = strat.build(shape.clone(), rng).into_diagnostic()?;

    query_text_test("simple_random_strategy", &gen)?;

    Ok(())
}
