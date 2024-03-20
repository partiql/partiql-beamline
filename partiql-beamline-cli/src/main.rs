mod cli;

use crate::cli::{encode_ion_text, IonPrintMode, DATETIME_FORMAT};
use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{Sample, Tick};
use partiql_beamline::sim::Sim;
use partiql_beamline_cliargs::{parse_args, OutputFormat, SampleCount, Script, Seed, StartTime};
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
        sample_count: SampleCount,

        #[command(flatten)]
        seed: Seed,

        #[command(flatten)]
        start_time: StartTime,

        #[command(flatten)]
        script: Script,

        #[clap(short = 'f', long = "output-format", value_enum, default_value_t=OutputFormat::Text)]
        output_format: OutputFormat,
    },
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Gen {
            sample_count,
            seed,
            start_time,
            script,
            output_format,
        } => {
            let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
            let script = script.extract().into_diagnostic()?;
            let t0 = cfg.t0;

            let sample_count = sample_count.sample_count;

            match output_format {
                OutputFormat::Text => {
                    println!("Seed: {}", cfg.seed);
                    println!("Start: {}", t0.format(&DATETIME_FORMAT).expect("t0 print"));

                    let mut sim = Sim::from_config(cfg, script.as_bytes()).into_diagnostic()?;

                    for _ in 0..sample_count {
                        if let Ok(Some(Sample {
                            tick: Tick(t),
                            value,
                        })) = sim.next_sample()
                        {
                            let time = t0.add(Duration::milliseconds(t as i64));
                            println!("[{time}] : {value:?}");
                        }
                    }
                }
                OutputFormat::Ion => {
                    let res = encode_ion_text(
                        IonPrintMode::Compact,
                        &cli::execute(cfg, script, sample_count)?,
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
                        &cli::execute(cfg, script, sample_count)?,
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
    }

    Ok(())
}
