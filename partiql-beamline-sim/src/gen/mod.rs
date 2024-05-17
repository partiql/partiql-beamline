use crate::primitives::{Sample, Tick};
use crate::sim::context::SimContext;
use dyn_clone::DynClone;
use partiql_types::PartiqlType;
use partiql_value::Value;
use statrs::StatsError;
use std::fmt::Debug;
use thiserror::Error;

pub mod arrival;
pub mod constant;
pub mod data;
pub mod distributions;
pub mod process;
pub mod timeline;
mod util;

pub const CURRENT_TICK: &str = "current_tick";

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DataGenerationError {
    #[error("Stats Error: `{0}-{0}`")]
    Stats(#[from] StatsError),

    #[error("Bounds Error: `{0}-{0}`")]
    Bounds(i64, i64),

    #[error("Error: `{0}`")]
    Other(String),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DataSamplingError {
    #[error("Invalid Tick Error: {0}")]
    InvalidTick(String),
}

pub type DataSamplingResult<T> = Result<T, DataSamplingError>;

/// A Random Process (aka Stochastic Process) is
/// > a mathematical models of systems and phenomena that appear to vary in a random manner.
///  -- from: https://en.wikipedia.org/wiki/Stochastic_process
pub trait RandomProcess: Debug {
    fn next_sample(&self, ctx: &SimContext) -> Option<DataSamplingResult<Sample>>;

    /// Returns [`Some(Tick)`] representing the next arrival tick for this process's samples
    /// Returns [`None`] when the process is 'finished'.
    fn next_arrival(&self, now: Tick, ctx: &SimContext) -> Option<Tick>;

    fn shape(&self) -> PartiqlType;
}

pub type DataGenerationResult<T> = Result<T, DataGenerationError>;

pub trait ValueGenerator: Debug + DynClone {
    fn gen_value(&self, ctx: &SimContext) -> Value;
    fn value_type(&self) -> PartiqlType;
}

dyn_clone::clone_trait_object!(ValueGenerator);

/// A 'generator' of arrival times for the events from a Random Process (aka Stochastic Process)
pub trait ArrivalTime: Debug + DynClone {
    /// Returns [`Some(Tick)`] representing the next arrival tick for this process's samples
    /// Returns [`None`] when the process is 'finished'.
    fn next_arrival(&self, now: Tick) -> Option<Tick>;
}
dyn_clone::clone_trait_object!(ArrivalTime);

pub trait ArrivalBoxed: ArrivalTime
where
    Self: 'static,
{
    fn boxed(self) -> Box<dyn ArrivalTime>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T> ArrivalBoxed for T where T: ArrivalTime + 'static {}
