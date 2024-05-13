mod cli;
mod kolliderdb;

use crate::cli::{encode_ion_text, IonPrintMode};
use clap::{Parser, Subcommand};
use ion_rs::element::writer::TextKind;
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{DataSetName, Sample, Tick};
use partiql_beamline::sim::{SimBuilder, DATETIME_FORMAT};
use partiql_beamline_cliargs::{
    parse_args, DataOutputFormat, DbArgs, DbTarget, SampleCount, ShapeOutputFormat, SimSpec,
};
use partiql_extension_ion::Encoding;
use std::io::stdout;
use std::ops::Add;

use crate::kolliderdb::{
    catalog_full_path, create_catalog_dir, create_kollider_db, create_manifest_file,
    create_script_file,
};
use partiql_beamline_serde::serde::{PartiqlDataSetsEncoder, PartiqlKolliderEncoder};
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

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.commands {
        Commands::Gen(gen) => match gen {
            Gen::Data {
                spec:
                    SimSpec {
                        seed,
                        start_time,
                        script,
                    },
                sample_count,
                output_format,
                datasets,
            } => {
                let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
                let script = script.extract().into_diagnostic()?;
                let t0 = cfg.t0;

                let sample_count = sample_count.sample_count;

                match output_format {
                    DataOutputFormat::Text => {
                        println!("Seed: {}", cfg.seed);
                        println!("Start: {}", t0.format(&DATETIME_FORMAT).expect("t0 print"));

                        let mut sim =
                            SimBuilder::from_config(cfg.clone(), script.clone().as_bytes())
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
                                    })) = sim.for_dataset(id).next_sample()
                                    {
                                        let time = t0.add(Duration::milliseconds(t as i64));
                                        let name = name.clone().0;
                                        println!("[{time}] : {name:?} {value:?}");
                                    }
                                }
                            }
                        } else {
                            for dataset in datasets {
                                if let Some(id) = sim.get_dataset_id(&DataSetName(dataset.clone()))
                                {
                                    for _c in 0..sample_count {
                                        if let Ok(Some(Sample {
                                            tick: Tick(t),
                                            value,
                                        })) = sim.for_dataset(id).next_sample()
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
            Gen::Db(db) => {
                match db {
                    Db::Kollider {
                        spec:
                            SimSpec {
                                seed,
                                start_time,
                                script,
                            },
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
                            let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
                            let script = script.extract().into_diagnostic()?;
                            let sample_count = sample_count.sample_count;
                            let catalog_full_path = catalog_full_path(&catalog_name, &catalog_path);

                            create_catalog_dir(force, &catalog_name, &catalog_path)?;
                            create_manifest_file(&cfg, &catalog_full_path)?;
                            create_script_file(&catalog_full_path, &script)?;
                            create_kollider_db(
                                &cfg,
                                &catalog_name,
                                &catalog_path,
                                &script,
                                sample_count,
                            )?
                        } else {
                            todo!("Generating database on a target other than filesystem is unsupported")
                        }
                    }
                }
            }
        },
        Commands::InferShape {
            spec:
                SimSpec {
                    seed,
                    start_time,
                    script,
                },
            output_format,
        } => {
            let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
            let script = script.extract().into_diagnostic()?;
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
                _ => {
                    todo!("Unsupported output format")
                }
            }
        }
    }

    Ok(())
}
