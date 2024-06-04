use std::error::Error;

use derive_builder::Builder;
use ion_rs::IonError;
use miette::Diagnostic;
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;
use time::macros::datetime;
use time::OffsetDateTime;

use crate::reader::ProcessConfigError;

/// Error in simulation configuration.
#[derive(Debug, Error, Diagnostic)]
#[error("Sim Config Error")]
#[non_exhaustive]
pub enum SimConfigError {
    #[error("Rand error: {0}")]
    RandError(rand::Error),

    #[error("Process Configuration error: {0}")]
    ProcessConfig(ProcessConfigError),

    #[error("Rand error: {0}")]
    TimeError(time::error::Error),

    #[error("Unknown Error: {0}")]
    UnknownError(Box<dyn Error + Send + Sync + 'static>),
}

impl From<rand::Error> for SimConfigError {
    fn from(e: rand::Error) -> Self {
        SimConfigError::RandError(e)
    }
}

impl From<time::error::Error> for SimConfigError {
    fn from(e: time::error::Error) -> Self {
        SimConfigError::TimeError(e)
    }
}

impl From<ProcessConfigError> for SimConfigError {
    fn from(e: ProcessConfigError) -> Self {
        SimConfigError::ProcessConfig(e)
    }
}

impl From<IonError> for SimConfigError {
    fn from(e: IonError) -> Self {
        let pce: ProcessConfigError = e.into();
        pce.into()
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for SimConfigError {
    fn from(e: Box<dyn Error + Send + Sync + 'static>) -> Self {
        SimConfigError::UnknownError(e)
    }
}

impl From<SimConfigError> for SimConfigBuilderError {
    fn from(e: SimConfigError) -> Self {
        SimConfigBuilderError::from(e.to_string())
    }
}

pub type SimConfigResult<T> = Result<T, SimConfigError>;
pub type SimConfigBuildResult<T> = Result<T, SimConfigBuilderError>;

#[derive(Builder, Clone, Debug)]
#[builder(build_fn(skip))]
pub struct SimConfig {
    /// Simulation root seed.
    pub seed: u64,

    /// Simulation time zero
    pub t0: OffsetDateTime,
}

impl SimConfigBuilder {
    pub fn build(&self) -> Result<SimConfig, SimConfigBuilderError> {
        let seed = match self.seed {
            Some(seed) => seed,
            None => auto_seed()?,
        };

        let t0 = match self.t0 {
            Some(t0) => t0,
            None => auto_t0(seed)?,
        };

        Ok(SimConfig { seed, t0 })
    }
}

fn auto_seed() -> SimConfigResult<u64> {
    Ok(ChaCha8Rng::from_rng(thread_rng())?.gen())
}

fn auto_t0(seed: u64) -> SimConfigResult<OffsetDateTime> {
    let s = datetime!(2019-08-01 00:00:01-07:00);
    let e = OffsetDateTime::now_utc();
    let range = s.unix_timestamp()..=e.unix_timestamp();

    let mut rng = Pcg64Mcg::seed_from_u64(seed);

    let time: Result<_, time::error::Error> =
        OffsetDateTime::from_unix_timestamp(rng.gen_range(range)).map_err(|e| e.into());
    Ok(time?)
}
