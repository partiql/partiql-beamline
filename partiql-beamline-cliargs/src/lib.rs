use clap::{Args, ValueEnum};
use partiql_beamline::sim::{SimConfig, SimConfigBuildResult, SimConfigBuilder};
use std::fs;

use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;
use time::OffsetDateTime;

/// Output format for the generated data
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DataOutputFormat {
    Ion,
    IonPretty,
    Text,
}

/// Output format for the generated shape of data
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ShapeOutputFormat {
    PartiqlKollider,
    Text,
    BasicDdl,
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

/// Default nullability for the simulation.
#[derive(Args, Debug, Clone, PartialEq)]
#[group(required = false, multiple = false)]
pub struct Nullability {
    /// If false, value types will be non-nullable by default
    #[arg(long, default_value = "false")]
    pub default_not_null: bool,

    /// If specifed, value types are nullable by default and will generate `NULL` at the given percentage.
    #[arg(long, value_parser=pct_parser)]
    pub pct_null: Option<f64>,
}

/// Default nullability for the simulation.
#[derive(Args, Debug, Clone, PartialEq)]
#[group(required = false, multiple = false)]
pub struct Optionality {
    /// If false, value types will be non-nullable by default
    #[arg(long, default_value = "true")]
    pub default_not_optional: bool,

    /// If specifed, value types are nullable by default and will generate `MISSING` at the given percentage.
    #[arg(long,  value_parser=pct_parser)]
    pub pct_optional: Option<f64>,
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
#[derive(Args, Debug, Clone, PartialEq)]
pub struct SimSpec {
    #[command(flatten)]
    pub seed: Seed,

    #[command(flatten)]
    pub start_time: StartTime,

    #[command(flatten)]
    pub script: Script,

    #[command(flatten)]
    pub nullability: Nullability,

    #[command(flatten)]
    pub optionality: Optionality,
}

/// Seed configuration for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct KolliderDb {
    #[command(flatten)]
    pub seed: Seed,

    #[command(flatten)]
    pub start_time: StartTime,

    #[command(flatten)]
    pub script: Script,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct DbArgs {
    #[clap(
        short = 'c',
        long = "output-format",
        default_value = "beamline-catalog"
    )]
    pub catalog_name: String,

    #[clap(short = 'p', long = "output-format", default_value = ".")]
    pub catalog_path: String,

    #[clap(long = "force", default_value = "false")]
    pub force: bool,

    #[clap(short = 'p', long = "output-format", default_value = "filesystem")]
    pub target: DbTarget,
}

/// Output target for the generated Database
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DbTarget {
    Filesystem,
}

fn epoch_ms_parser(arg: &str) -> Result<OffsetDateTime, String> {
    let ms: i64 = arg.parse().map_err(|e: ParseIntError| e.to_string())?;
    time::OffsetDateTime::from_unix_timestamp(ms).map_err(|e| e.to_string())
}

fn iso_parser(arg: &str) -> Result<time::OffsetDateTime, String> {
    let iso8601 = time::format_description::well_known::Iso8601::PARSING;
    time::OffsetDateTime::parse(arg, &iso8601).map_err(|e| e.to_string())
}

fn pct_parser(arg: &str) -> Result<f64, String> {
    let pct = f64::from_str(arg).map_err(|e| e.to_string())?;
    if pct < 0.0 || 1.0 < pct {
        Err(format!("Percents must be between 0 and 1: `{pct}`"))
    } else {
        Ok(pct)
    }
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

pub fn parse_args(
    seed: &Seed,
    t0: &StartTime,
    null: Nullability,
    opt: Optionality,
) -> SimConfigBuildResult<SimConfig> {
    let mut cfg = SimConfigBuilder::default();

    if let Some(seed) = seed.extract() {
        cfg.seed(seed);
    }
    if let Some(t0) = t0.extract() {
        cfg.t0(t0);
    }

    if null.default_not_null {
        cfg.nullability(None);
    } else if let Some(null) = null.pct_null {
        cfg.nullability(Some(null));
    }

    if opt.default_not_optional {
        cfg.optionality(None);
    } else if let Some(opt) = opt.pct_optional {
        cfg.optionality(Some(opt));
    }

    cfg.build()
}
