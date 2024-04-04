mod cli;

use crate::cli::{encode_ion_text, IonPrintMode, DATETIME_FORMAT};
use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{DataSetName, Sample, Tick};
use partiql_beamline::sim::SimBuilder;
use partiql_beamline_cliargs::{parse_args, OutputFormat, SampleCount, SimSpec};
use partiql_extension_ion::Encoding;
use std::ops::Add;
use time::Duration;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the data generator
    Gen {
        #[command(flatten)]
        spec: SimSpec,

        #[command(flatten)]
        sample_count: SampleCount,

        #[clap(short = 'f', long = "output-format", value_enum, default_value_t=OutputFormat::Text)]
        output_format: OutputFormat,

        #[clap(short = 'd', long = "dataset")]
        datasets: Vec<String>,
    },
    Schema {
        #[command(flatten)]
        spec: SimSpec,
    },
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Gen {
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
                OutputFormat::Text => {
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
                            if let Some(id) = sim.get_dataset_id(&DataSetName(dataset.clone())) {
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
                OutputFormat::Ion => {
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
                OutputFormat::IonPretty => {
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
        Commands::Schema {
            spec:
                SimSpec {
                    seed,
                    start_time,
                    script,
                },
        } => {
            let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
            let script = script.extract().into_diagnostic()?;
            let t0 = cfg.t0;

            println!("Seed: {}", cfg.seed);
            println!("Start: {}", t0.format(&DATETIME_FORMAT).expect("t0 print"));

            let sim = SimBuilder::from_config(cfg.clone(), script.clone().as_bytes())
                .expect("auto sim")
                .build_multi_dataset()
                .expect("auto sim");

            println!("{:#?}", sim.schema())
        }
    }

    Ok(())
}
