use crate::sim_spec::{Script, Seed, StartTime};
use clap::{Args, ValueEnum};

/// Output format for the generated data
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataOutputFormat {
    Ion,
    IonPretty,
    IonBinary,
    Text,
    Parquet,
    Json,
    JsonPretty,
    /// One compact JSON value per line, no envelope. Row data only.
    /// `seed`, `start`, and the `data` wrapper are omitted. Use `json` or
    /// `json-pretty` if you need the run metadata. The invoking `--seed`
    /// and `--start-iso` remain the reproducibility source.
    Jsonl,
    /// A single top-level JSON array of row values, no envelope. Same
    /// row-only contract as `jsonl`. `seed`, `start`, and `data` are omitted.
    JsonArray,
}

/// Output format for the generated shape of data
#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShapeOutputFormat {
    BeamlineJson,
    Text,
    BasicDdl,
}

/// Number of samples to create
#[derive(Args, Debug, Copy, Clone, PartialEq, Eq)]
#[group(required = false, multiple = false)]
pub struct SampleCount {
    /// Value for the number of samples
    #[arg(long, default_value = "10", value_name = "SAMPLE_COUNT")]
    pub sample_count: u64,
}

/// Initial state configuration for the generator.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct BeamlineLite {
    #[command(flatten)]
    pub seed: Seed,

    #[command(flatten)]
    pub start_time: StartTime,

    #[command(flatten)]
    pub script: Script,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct DbArgs {
    #[clap(short = 'c', long = "catalog_name", default_value = "beamline-catalog")]
    pub catalog_name: String,

    #[clap(short = 'p', long = "catalog_path", default_value = ".")]
    pub catalog_path: String,

    #[clap(long = "force", default_value = "false")]
    pub force: bool,

    #[clap(short = 't', long = "target", default_value = "filesystem")]
    pub target: DbTarget,
}

/// Output target for the generated Database
#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum DbTarget {
    Filesystem,
}
