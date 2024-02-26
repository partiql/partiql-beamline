use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{Sample, Tick};
use partiql_beamline::sim::Sim;
use partiql_beamline_cliargs::{parse_args, Script, Seed, StartTime};
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
        seed: Seed,

        #[command(flatten)]
        start_time: StartTime,

        #[command(flatten)]
        script: Script,
    },
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Gen {
            seed,
            start_time,
            script,
        } => {
            let cfg = parse_args(&seed, &start_time).into_diagnostic()?;
            let t0 = cfg.t0;

            let dt_fmt = time::format_description::well_known::Iso8601::DEFAULT;

            println!("Seed: {}", cfg.seed);
            println!("Start: {}", cfg.t0.format(&dt_fmt).expect("t0 print"));

            let script = script.extract().into_diagnostic()?;
            let mut sim = Sim::from_config(cfg, script.as_bytes()).into_diagnostic()?;

// TODO move to variable iteration as opposed to the current `100` limit
            for _ in 0..100 {
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
    }

    Ok(())
}
