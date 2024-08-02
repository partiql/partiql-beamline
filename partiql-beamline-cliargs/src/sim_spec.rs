use clap::Args;
use partiql_beamline::sim::{SimConfig, SimConfigBuildResult, SimConfigBuilder};
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;
use time::OffsetDateTime;

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

impl Seed {
    pub fn extract(&self) -> Option<u64> {
        match (&self.seed, &self.seed_auto) {
            (Some(seed), false) => Some(*seed),
            (None, true) => None,
            _ => unreachable!(),
        }
    }
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

/// Default nullability (i.e., ability & chance to generate `NULL`) for the simulation.
#[derive(Args, Debug, Clone, PartialEq)]
#[group(required = false, multiple = false)]
pub struct Nullability {
    /// If true, value types will be nullable by default; Else if false, not-nullable by default.
    #[arg(long)]
    pub default_nullable: Option<bool>,

    /// If specified, value types are nullable by default and will generate `NULL` at the given percentage.
    #[arg(long, value_parser=pct_parser)]
    pub pct_null: Option<f64>,
}

/// Default optionality (i.e., ability & chance to generate `MISSING`) for the simulation.
#[derive(Args, Debug, Clone, PartialEq)]
#[group(required = false, multiple = false)]
pub struct Optionality {
    /// If true, value types will be optional by default; Else if false, not-optional by default.
    #[arg(long)]
    pub default_optional: Option<bool>,

    /// If specified, value types are optional by default and will generate `MISSING` at the given percentage.
    #[arg(long, value_parser=pct_parser)]
    pub pct_optional: Option<f64>,
}

/// Script configuration for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = false)]
pub struct Script {
    /// Provde the path to the script file
    #[arg(long, value_name = "PATH/TO/SCRIPT")]
    pub script_path: Option<PathBuf>,

    /// Provde the script inline
    #[arg(long, value_name = "SCRIPT_DATA")]
    pub script: Option<String>,
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

/// Initial state configuration for the generator.
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

impl SimSpec {
    pub fn to_script_and_config(
        self,
    ) -> (std::io::Result<String>, SimConfigBuildResult<SimConfig>) {
        let SimSpec {
            seed,
            start_time,
            script,
            nullability,
            optionality,
        } = self;

        let script = script.extract();
        let mut cfg = SimConfigBuilder::default();

        if let Some(seed) = seed.extract() {
            cfg.seed(seed);
        }
        if let Some(t0) = start_time.extract() {
            cfg.t0(t0);
        }

        if let Some(nullable) = nullability.default_nullable {
            cfg.nullability(if nullable { Some(0.0) } else { None });
        } else if let Some(null) = nullability.pct_null {
            cfg.nullability(Some(null));
        }

        if let Some(optional) = optionality.default_optional {
            cfg.optionality(if optional { Some(0.0) } else { None });
        } else if let Some(opt) = optionality.pct_optional {
            cfg.optionality(Some(opt));
        }

        (script, cfg.build())
    }
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
    if !(0.0..=1.0).contains(&pct) {
        Err(format!("Percents must be between 0 and 1: `{pct}`"))
    } else {
        Ok(pct)
    }
}
