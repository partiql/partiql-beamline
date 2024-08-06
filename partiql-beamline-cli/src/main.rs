mod cli;
mod kolliderdb;

use crate::cli::{encode_ion_text, IonPrintMode};
use clap::{Parser, Subcommand};
use ion_rs::element::writer::TextKind;
use itertools::{Itertools, Position};
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{DataSetName, Sample, Tick};
use partiql_beamline::sim::{ISim, SimBuilder, DATETIME_FORMAT};
use partiql_extension_ion::Encoding;
use std::io::stdout;
use std::ops::Add;

use crate::kolliderdb::{
    catalog_full_path, create_catalog_dir, create_kollider_db, create_manifest_file,
    create_script_file,
};
use partiql_beamline_cliargs::data_gen::{
    DataOutputFormat, DbArgs, DbTarget, SampleCount, ShapeOutputFormat,
};
use partiql_beamline_cliargs::query_gen::{IntoStrategy, QueryGenStrategy};
use partiql_beamline_cliargs::sim_spec::SimSpec;
use partiql_beamline_query::{QueryTextGenerator, QueryTextGeneratorConfigBuilder};
use partiql_beamline_serde::kollider::PartiqlKolliderEncoder;
use partiql_beamline_serde::serde::PartiqlDataSetsEncoder;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
use time::Duration;

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
        #[clap(short = 'f', long = "output-format", value_enum, default_value_t=ShapeOutputFormat::Text)]
        output_format: ShapeOutputFormat,
    },
    /// Run the query generator
    #[clap(subcommand)]
    Query(QueryGen),
}

#[derive(Subcommand)]
pub enum Gen {
    /// Run the data generator
    Data {
        #[command(flatten)]
        spec: SimSpec,

        #[command(flatten)]
        sample_count: SampleCount,

        #[clap(short = 'f', long = "output-format", value_enum, default_value_t=DataOutputFormat::Text)]
        output_format: DataOutputFormat,

        #[clap(short = 'd', long = "dataset")]
        datasets: Vec<String>,
    },
    #[clap(subcommand)]
    /// Run the Db generator with both data and schema(s)
    Db(Db),
}

#[derive(Subcommand)]
pub enum Db {
    Kollider {
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
        //#[clap(flatten)]
        //strat: QueryGenStratBasicRandomSFW,
    },
}

// TODO rather than all the `.expect`s below, we should use miette errors/diagnostics for better error reporting
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
        Gen::Data {
            spec,
            sample_count,
            output_format,
            datasets,
        } => {
            let (script, cfg) = spec.to_script_and_config();
            let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);
            let t0 = cfg.t0;

            let sample_count = sample_count.sample_count;

            match output_format {
                DataOutputFormat::Text => {
                    println!("Seed: {}", cfg.seed);
                    println!("Start: {}", t0.format(&DATETIME_FORMAT).expect("t0 print"));

                    let mut sim = SimBuilder::from_config(cfg.clone(), script.clone().as_bytes())
                        .expect("auto sim")
                        .build_multi_dataset()
                        .expect("auto sim");

                    if datasets.is_empty() {
                        let sim_datasets = sim.datasets();
                        for (id, name) in sim_datasets {
                            for _c in 0..sample_count {
                                if let Ok(Some(Sample {
                                    tick: Tick(t),
                                    value,
                                })) = sim.for_dataset(id)?.next_sample()
                                {
                                    let time = t0.add(Duration::milliseconds(t as i64));
                                    let name = name.clone().0;
                                    println!("[{time}] : {name:?} {value:?}");
                                }
                            }
                        }
                    } else {
                        for dataset in datasets {
                            if let Some(id) = sim.get_dataset_id(&DataSetName(dataset.clone())) {
                                for _c in 0..sample_count {
                                    if let Ok(Some(Sample {
                                        tick: Tick(t),
                                        value,
                                    })) = sim.for_dataset(id)?.next_sample()
                                    {
                                        let time = t0.add(Duration::milliseconds(t as i64));
                                        println!("[{time}] : {dataset:?} {value:?}");
                                    }
                                }
                            }
                        }
                    }
                }
                DataOutputFormat::Ion => {
                    let res = encode_ion_text(
                        IonPrintMode::Compact,
                        &cli::execute(cfg, script, sample_count, datasets)?,
                        Encoding::Ion,
                    );
                    match res {
                        Ok(out) => println!("{:}", &out),
                        Err(e) => println!("{:?}", e),
                    }
                }
                DataOutputFormat::IonPretty => {
                    let res = encode_ion_text(
                        IonPrintMode::Pretty,
                        &cli::execute(cfg, script, sample_count, datasets)?,
                        Encoding::Ion,
                    );
                    match res {
                        Ok(out) => println!("{:}", &out),
                        Err(e) => println!("{:?}", e),
                    }
                }
                _ => {
                    todo!("Unsupported output format")
                }
            }
        }
        Gen::Db(db) => match db {
            Db::Kollider {
                spec,
                db_args:
                    DbArgs {
                        catalog_name,
                        catalog_path,
                        force,
                        target,
                    },
                sample_count,
            } => {
                if let DbTarget::Filesystem = target {
                    let (script, cfg) = spec.to_script_and_config();
                    let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);
                    let sample_count = sample_count.sample_count;
                    let catalog_full_path = catalog_full_path(&catalog_name, &catalog_path);

                    create_catalog_dir(force, &catalog_name, &catalog_path)?;

                    let ddl_encoder = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
                    create_manifest_file(&cfg, &catalog_full_path, &ddl_encoder.syntax())?;

                    create_script_file(&catalog_full_path, &script)?;
                    create_kollider_db(
                        cfg,
                        &catalog_name,
                        &catalog_path,
                        &script,
                        sample_count,
                        &ddl_encoder,
                    )?
                } else {
                    todo!("Generating database on a target other than filesystem is unsupported")
                }
            }
        },
    }
    Ok(())
}

fn handle_infer(spec: SimSpec, output_format: ShapeOutputFormat) -> miette::Result<()> {
    let (script, cfg) = spec.to_script_and_config();
    let (script, cfg) = (script.into_diagnostic()?, cfg.into_diagnostic()?);

    let t0 = cfg.t0;

    let sim = SimBuilder::from_config(cfg.clone(), script.clone().as_bytes())
        .expect("auto sim")
        .build_multi_dataset()
        .expect("auto sim");

    match output_format {
        ShapeOutputFormat::PartiqlKollider => {
            let shape = sim.shape();

            let mut out = stdout().lock();
            let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
                .build(&mut out)
                .expect("pretty writer");
            let mut encoder = PartiqlKolliderEncoder::new(&mut writer);
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
