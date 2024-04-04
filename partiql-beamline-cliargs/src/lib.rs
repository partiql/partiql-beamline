use clap::{Args, ValueEnum};
use partiql_beamline::sim::{SimConfig, SimConfigBuildResult, SimConfigBuilder};
use std::fs;

use std::num::ParseIntError;
use std::path::PathBuf;
use time::OffsetDateTime;

/// Output format
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum OutputFormat {
    Ion,
    IonPretty,
    Text,
}

/// Number of samples to get created
#[derive(Args, Debug, Copy, Clone, PartialEq, Eq)]
#[group(required = false, multiple = false)]
pub struct SampleCount {
    /// Value for the number of samples
    #[arg(long, default_value = "10", value_name = "SAMPLE_COUNT")]
    pub sample_count: u64,
}

/// Seed configuration for the generator.
#[derive(Args, Debug, Copy, Clone, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct Seed {
    /// Use the local machine's entropy to generate a 'random' seed.
    #[arg(long)]
    pub seed_auto: bool,

    /// (Re)play from a specified seed.
    #[arg(long, value_name = "SEED")]
    pub seed: Option<u64>,
}

/// Start date & time for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct StartTime {
    /// Use the local machine's entropy to generate a 'random' start time.
    #[arg(long)]
    pub start_auto: bool,

    /// (Re)play from a specified start time (specified in ms since the unix epoch).
    #[arg(long, value_name = "EPOCH_MS", value_parser = epoch_ms_parser)]
    pub start_epoch_ms: Option<time::OffsetDateTime>,

    /// (Re)play from a specified start time (specified in ms since the unix epoch).
    #[arg(long, value_name = "ISO_8601", value_parser = iso_parser)]
    pub start_iso: Option<time::OffsetDateTime>,
}

/// Seed configuration for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct Script {
    #[arg(long, value_name = "PATH/TO/SCRIPT")]
    pub script_path: Option<PathBuf>,

    /// (Re)play from a specified seed.
    #[arg(long, value_name = "SCRIPT_DATA")]
    pub script: Option<String>,
}

/// Seed configuration for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct SimSpec {
    #[command(flatten)]
    pub seed: Seed,

    #[command(flatten)]
    pub start_time: StartTime,

    #[command(flatten)]
    pub script: Script,
}

fn epoch_ms_parser(arg: &str) -> Result<OffsetDateTime, String> {
    let ms: i64 = arg.parse().map_err(|e: ParseIntError| e.to_string())?;
    time::OffsetDateTime::from_unix_timestamp(ms).map_err(|e| e.to_string())
}

fn iso_parser(arg: &str) -> Result<time::OffsetDateTime, String> {
    let iso8601 = time::format_description::well_known::Iso8601::PARSING;
    time::OffsetDateTime::parse(arg, &iso8601).map_err(|e| e.to_string())
}

impl Seed {
    pub fn extract(&self) -> Option<u64> {
        match (&self.seed, &self.seed_auto) {
            (Some(seed), false) => Some(*seed),
            (None, true) => None,
            _ => unreachable!(),
        }
    }
}

impl StartTime {
    pub fn extract(&self) -> Option<OffsetDateTime> {
        match (&self.start_epoch_ms, &self.start_iso, &self.start_auto) {
            (Some(t0), None, false) => Some(*t0),
            (None, Some(t0), false) => Some(*t0),
            (None, None, true) => None,
            _ => unreachable!(),
        }
    }
}

impl Script {
    pub fn extract(self) -> std::io::Result<String> {
        match (self.script_path, self.script) {
            (None, Some(data)) => Ok(data),
            (Some(path), None) => fs::read_to_string(path),
            _ => unreachable!(),
        }
    }
}

pub fn parse_args(seed: &Seed, t0: &StartTime) -> SimConfigBuildResult<SimConfig> {
    let mut cfg = SimConfigBuilder::default();

    if let Some(seed) = seed.extract() {
        cfg.seed(seed);
    }
    if let Some(t0) = t0.extract() {
        cfg.t0(t0);
    }

    cfg.build()
}
