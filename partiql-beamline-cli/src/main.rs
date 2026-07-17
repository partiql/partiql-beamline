mod beamlinelite_db;
mod json_writer;
mod parquet_writer;
mod writer;

use crate::beamlinelite_db::{
    catalog_full_path, create_catalog_dir, create_db, create_manifest_file, create_script_file,
};
use crate::writer::{
    DataSetFiltersBuilder, SimSamplesBuilder, SimWriter, WriterIonBinaryBuilder,
    WriterIonCompactBuilder, WriterIonPrettyBuilder, WriterSimBuilder, WriterTextBuilder,
};
use clap::{Args, Parser, Subcommand};
use ion_rs::element::writer::TextKind;
use itertools::{Itertools, Position};
use miette::IntoDiagnostic;
use partiql_beamline::sim::{ISim, SimBuilder, DATETIME_FORMAT};
use partiql_beamline_cliargs::data_gen::{
    DataOutputFormat, DbArgs, DbTarget, SampleCount, ShapeOutputFormat,
};
use partiql_beamline_cliargs::query_gen::{IntoStrategy, QueryGenStrategy};
use partiql_beamline_cliargs::sim_spec::SimSpec;
use partiql_beamline_query::{QueryTextGenerator, QueryTextGeneratorConfigBuilder};
use partiql_beamline_serde::beamline_json::BeamlineJsonEncoder;
use partiql_beamline_serde::serde::PartiqlDataSetsEncoder;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};

use std::io::stdout;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(subcommand)]
    /// Run the generator
    Gen(Gen),
    /// Run the script shape inference
    InferShape {
        #[command(flatten)]
        spec: SimSpec,
        #[clap(short = 'f', long = "output-format", value_enum, default_value_t=ShapeOutputFormat::Text
        )]
        output_format: ShapeOutputFormat,
    },
    /// Run the query generator
    #[clap(subcommand)]
    Query(QueryGen),
}

#[derive(Subcommand)]
pub enum Gen {
    /// Run the data generator
    Data(Data),

    #[clap(subcommand)]
    /// Run the Db generator with both data and schema(s)
    Db(Db),
}

#[derive(Args)]
/// Run the data generator
pub struct Data {
    #[command(flatten)]
    spec: SimSpec,

    #[command(flatten)]
    sample_count: SampleCount,

    #[clap(short = 'f', long = "output-format", value_enum, default_value_t=DataOutputFormat::Text
    )]
    output_format: DataOutputFormat,

    #[clap(short = 'd', long = "dataset")]
    datasets: Vec<String>,

    #[clap(short = 'o', long = "output-path", value_name = "OUTPUT_PATH")]
    output_path: Option<String>,

    /// Coerce unsupported types (e.g., DateTime, Decimal) to strings instead of erroring
    #[clap(long = "coerce-unsupported", default_value = "false")]
    coerce_unsupported: bool,
}

#[derive(Subcommand)]
pub enum Db {
    BeamlineLite {
        #[command(flatten)]
        spec: SimSpec,

        #[command(flatten)]
        sample_count: SampleCount,

        #[command(flatten)]
        db_args: DbArgs,
    },
}

#[derive(Subcommand)]
pub enum QueryGen {
    Basic {
        #[command(flatten)]
        samples: SampleCount,
        #[command(flatten)]
        source: SimSpec,
        #[clap(subcommand)]
        strat: QueryGenStrategy,
    },
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.commands {
        Commands::Gen(gen) => handle_gen(gen)?,
        Commands::InferShape {
            spec,
            output_format,
        } => handle_infer(spec, output_format)?,
        Commands::Query(query) => handle_query(query)?,
    }

    Ok(())
}
fn handle_gen(gen: Gen) -> miette::Result<()> {
    match gen {
        Gen::Data(data) => handle_gen_data(data),
        Gen::Db(db) => handle_gen_db(db),
    }
}

fn handle_gen_data(
    Data {
        spec,
        sample_count,
        output_format,
        datasets,
        output_path,
        coerce_unsupported,
    }: Data,
) -> miette::Result<()> {
    let (script, cfg) = spec.to_script_and_config();
    let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);

    let samples = SimSamplesBuilder::default()
        .count(sample_count.sample_count)
        .build()?;
    let filters = DataSetFiltersBuilder::default().filters(datasets).build()?;
    let wspec = WriterSimBuilder::default()
        .cfg(cfg)
        .script(script)
        .samples(samples)
        .dataset_filters(filters)
        .build()?;

    if output_format == DataOutputFormat::Parquet {
        let output_dir = output_path.unwrap_or_else(|| ".".to_string());
        let sampler = wspec.to_sampler().into_diagnostic()?;
        let mut writer = parquet_writer::SimWriterParquet {
            sampler,
            output_path: output_dir,
        };
        writer.write()?;
        return Ok(());
    }

    if matches!(output_format, DataOutputFormat::Json | DataOutputFormat::JsonPretty) {
        let sampler = wspec.to_sampler().into_diagnostic()?;
        let out = stdout().lock();
        let mut writer = json_writer::SimWriterJson {
            sampler,
            out,
            pretty: output_format == DataOutputFormat::JsonPretty,
            coerce_unsupported,
        };
        writer.write()?;
        return Ok(());
    }

    let out = stdout().lock();
    let mut writer = match output_format {
        DataOutputFormat::Text => WriterTextBuilder::default()
            .spec(wspec)
            .build()?
            .to_writer(out)?,
        DataOutputFormat::Ion => WriterIonCompactBuilder::default()
            .spec(wspec)
            .build()?
            .to_writer(out)?,
        DataOutputFormat::IonPretty => WriterIonPrettyBuilder::default()
            .spec(wspec)
            .build()?
            .to_writer(out)?,
        DataOutputFormat::IonBinary => WriterIonBinaryBuilder::default()
            .spec(wspec)
            .build()?
            .to_writer(out)?,
        DataOutputFormat::Parquet | DataOutputFormat::Json | DataOutputFormat::JsonPretty => {
            unreachable!()
        }
    };

    writer.write()?;
    drop(writer);

    Ok(())
}

fn handle_gen_db(db: Db) -> miette::Result<()> {
    match db {
        Db::BeamlineLite {
            spec,
            db_args:
                DbArgs {
                    catalog_name,
                    catalog_path,
                    force,
                    target: DbTarget::Filesystem,
                },
            sample_count,
        } => {
            let (script, cfg) = spec.to_script_and_config();
            let (script, cfg) = (script?, cfg?);
            let sample_count = sample_count.sample_count;
            let catalog_full_path = catalog_full_path(&catalog_name, &catalog_path);

            create_catalog_dir(force, &catalog_name, &catalog_path)?;

            let ddl_encoder = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
            create_manifest_file(&cfg, &catalog_full_path, &ddl_encoder.syntax())?;

            create_script_file(&catalog_full_path, &script)?;
            create_db(
                cfg,
                &catalog_name,
                &catalog_path,
                script,
                sample_count,
                &ddl_encoder,
            )?
        }
    }

    Ok(())
}

// TODO rather than all the `.expect`s below, we should use miette errors/diagnostics for better error reporting
fn handle_infer(spec: SimSpec, output_format: ShapeOutputFormat) -> miette::Result<()> {
    let (script, cfg) = spec.to_script_and_config();
    let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);

    let t0 = cfg.t0;

    let sim = SimBuilder::from_config(cfg.clone(), script)?.build_multi_dataset()?;

    match output_format {
        ShapeOutputFormat::BeamlineJson => {
            let shape = sim.shape();

            let mut out = stdout().lock();
            let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
                .build(&mut out)
                .expect("pretty writer");
            let mut encoder = BeamlineJsonEncoder::new(&mut writer);
            encoder.write_datasets(&cfg, shape).expect("encoded value");
            drop(writer);
            drop(out);
        }
        ShapeOutputFormat::Text => {
            println!("Seed: {}", cfg.seed);
            println!("Start: {}", t0.format(&DATETIME_FORMAT).expect("t0 print"));

            println!("{:#?}", sim.shape())
        }
        ShapeOutputFormat::BasicDdl => {
            println!("-- Seed: {}", cfg.seed);
            println!(
                "-- Start: {}",
                t0.format(&DATETIME_FORMAT).expect("t0 print")
            );

            let basic_ddl_encoder = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
            println!("-- Syntax: {}", basic_ddl_encoder.syntax());
            for (k, t) in sim.shape() {
                println!("-- Dataset: {k}");
                println!("{}", basic_ddl_encoder.ddl(&t)?)
            }
        }
        _ => {
            todo!("Unsupported output format")
        }
    }
    Ok(())
}

fn handle_query(query: QueryGen) -> miette::Result<()> {
    match query {
        QueryGen::Basic {
            samples,
            source,
            strat,
        } => {
            let (script, cfg) = source.to_script_and_config();
            let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);
            let qg: QueryTextGenerator = QueryTextGeneratorConfigBuilder::default()
                .config(cfg.clone())
                .script(script)
                .strategy(strat.into_strategy().into_diagnostic()?)
                .build()
                .into_diagnostic()?
                .to_generator()
                .into_diagnostic()?;

            let queries: Result<Vec<_>, _> = std::iter::repeat_with(|| qg.generate(80))
                .take(samples.sample_count as usize)
                .collect();

            for (pos, query) in queries?.iter().with_position() {
                match pos {
                    Position::First | Position::Only => {
                        println!("{query}");
                    }
                    Position::Middle | Position::Last => {
                        println!("\n\n{query}");
                    }
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;
    use std::fmt::Debug;

    #[inline]
    #[track_caller]
    fn assert_args<I, T>(args: I)
    where
        I: IntoIterator<Item = T> + Debug,
        T: Into<OsString> + Clone,
    {
        let result = Cli::try_parse_from(args);
        if let Err(err) = result {
            panic!("Expected to parse, but found: {err}");
        }
    }

    #[inline]
    #[track_caller]
    fn assert_cmdline(cmd: impl AsRef<str>) {
        assert_args(cmd.as_ref().split_whitespace())
    }

    #[test]
    fn parse_data_gen_cmdline() {
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed-auto --start-auto --sample-count 2
                        --script-path partiql-beamline-sim/tests/scripts/sensors.ion"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed 12328924104731257599 --start-auto --sample-count 2
                        --script-path partiql-beamline-sim/tests/scripts/sensors.ion"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed 12328924104731257599  --start-iso 2024-01-20T20:51:02.000000000Z --sample-count 2
                        --script-path partiql-beamline-sim/tests/scripts/sensors.ion"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed-auto --start-auto --sample-count 3
                        --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion
                        --output-format ion-pretty"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed 45121008347100595
                        --start-iso 2020-06-16T14:41:51.000000000Z
                        --script-path partiql-beamline-sim/tests/scripts/client-service.ion
                        --sample-count 10
                        --dataset service --dataset client_1
                        --output-format ion-pretty"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed 1234
                        --start-iso 2019-08-01T00:00:01-07:00
                        --script-path ./partiql-beamline-sim/tests/scripts/orders.ion
                        --sample-count 30
                        --output-format text"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen data
                        --seed 1234
                        --start-iso 2024-01-01T00:00:00.000000000Z
                        --script-path ./partiql-beamline-sim/tests/scripts/sensors.ion
                        --sample-count 5
                        --output-format parquet
                        --output-path ./test-output"##,
        );
    }

    #[test]
    fn parse_infer_shape_cmdline() {
        assert_cmdline(
            r##"partiql-beamline infer-shape
                        --seed-auto --start-auto
                        --script-path ./partiql-beamline-sim/tests/scripts/sensors.ion"##,
        );
        assert_cmdline(
            r##"partiql-beamline infer-shape
                      --seed 7844265201457918498
                      --start-auto 
                      --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion
                      --output-format basic-ddl"##,
        );
    }

    #[test]
    fn parse_gen_db_cmdline() {
        assert_cmdline(
            r##"partiql-beamline gen db beamline-lite
                       --seed-auto --start-auto 
                       --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion"##,
        );
        assert_cmdline(
            r##"partiql-beamline gen db beamline-lite
                       --seed-auto --start-auto
                       --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion"##,
        );
    }

    #[test]
    fn parse_query_cmdline() {
        assert_cmdline(
            r##"partiql-beamline query 
                        basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-select-all-fw
                                  --tbl-flt-rand-min 1 --tbl-flt-rand-max 1
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all
                                      --tbl-flt-pathstep-final-project
                                      --tbl-flt-type-final-scalar
                                      --pred-lt   "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-select-all-fw
                                  --tbl-flt-rand-min 1 --tbl-flt-rand-max 1
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all
                                      --tbl-flt-pathstep-final-project
                                      --tbl-flt-type-final-scalar
                                      --pred-lt
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-select-all-fw
                                  --tbl-flt-rand-min 3 --tbl-flt-rand-max 10
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-all
                                      --pred-all
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-sfw
                                  --project-rand-min 2 --project-rand-max 5
                                      --project-path-depth-min 1 --project-path-depth-max 1
                                      --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all
                                  --tbl-flt-rand-min 2 --tbl-flt-rand-max 5
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar
                                      --pred-all
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-sefw
                                  --project-rand-min 2 --project-rand-max 5
                                      --project-path-depth-min 1 --project-path-depth-max 1
                                      --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all
                                  --tbl-flt-rand-min 2 --tbl-flt-rand-max 5
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar
                                      --pred-all
                                  --exclude-rand-min 1 --exclude-rand-max 3
                                      --exclude-path-depth-min 1 --exclude-path-depth-max 1
                                      --exclude-pathstep-internal-all --exclude-pathstep-final-all --exclude-type-final-all
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion
                               --sample-count 3
                        rand-select-all-efw
                                  --tbl-flt-rand-min 1 --tbl-flt-rand-max 1
                                      --tbl-flt-path-depth-max 1
                                      --tbl-flt-pathstep-internal-all
                                      --tbl-flt-pathstep-final-project
                                      --tbl-flt-type-final-scalar
                                      --pred-lt
                                  --exclude-rand-min 1 --exclude-rand-max 3
                                      --exclude-path-depth-min 1 --exclude-path-depth-max 1
                                      --exclude-pathstep-internal-all --exclude-pathstep-final-all --exclude-type-final-all
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/transactions.ion
                               --sample-count 3
                        rand-sefw
                                  --project-rand-min 2 --project-rand-max 5
                                      --project-path-depth-min 1 --project-path-depth-max 10
                                      --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all
                                  --tbl-flt-rand-min 2 --tbl-flt-rand-max 5
                                      --tbl-flt-path-depth-max 10
                                      --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar
                                      --pred-all
                                  --exclude-rand-min 1 --exclude-rand-max 2
                                      --exclude-path-depth-min 3 --exclude-path-depth-max 4
                                      --exclude-pathstep-internal-all --exclude-pathstep-final-unpivot --exclude-type-final-all
            "##,
        );
        assert_cmdline(
            r##"partiql-beamline query
                        basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/transactions.ion
                               --sample-count 3
                        rand-sefw
                                  --project-rand-min 2 --project-rand-max 5
                                      --project-path-depth-min 1 --project-path-depth-max 3
                                      --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all
                                  --tbl-flt-rand-min 2 --tbl-flt-rand-max 5
                                      --tbl-flt-path-depth-max 10
                                      --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar
                                      --pred-all
                                  --exclude-rand-min 1 --exclude-rand-max 2
                                      --exclude-path-depth-min 3 --exclude-path-depth-max 4
                                      --exclude-pathstep-internal-all --exclude-pathstep-final-unpivot --exclude-type-final-all
            "##,
        );
    }
}
