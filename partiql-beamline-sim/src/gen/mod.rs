use crate::primitives::{Sample, Tick};
use crate::sim::context::SimContext;
use dyn_clone::DynClone;
use partiql_types::PartiqlType;
use partiql_value::Value;
use rand::distributions::Distribution;
use rand::Rng;
use rust_decimal::prelude::FromPrimitive;
use statrs::StatsError;
use std::fmt::Debug;
use std::ops::{Add, DerefMut};
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

/// A Random Process (or Stochastic Process) is
/// > a mathematical models of systems and phenomena that appear to vary in a random manner.
///  -- from: https://en.wikipedia.org/wiki/Stochastic_process
pub trait RandomProcess {
    fn next_sample(&self, ctx: &SimContext) -> Option<DataSamplingResult<Sample>>;

    fn next_arrival(&self, now: Tick, ctx: &SimContext) -> Tick;

    fn shape(&self) -> PartiqlType;
}

pub type DataGenerationResult<T> = Result<T, DataGenerationError>;

pub trait ValueGenerator: Debug + DynClone {
    fn gen_value(&self, ctx: &SimContext) -> Value;
    fn value_type(&self) -> PartiqlType;
}
dyn_clone::clone_trait_object!(ValueGenerator);

pub trait ArrivalTime: Debug + DynClone {
    fn next_arrival(&self, now: Tick) -> Tick;
}
dyn_clone::clone_trait_object!(ArrivalTime);
