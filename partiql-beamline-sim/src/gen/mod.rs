use crate::gen::distributions::Density;
use crate::primitives::{Sample, Tick};
use crate::sim::SimContext;
use dyn_clone::DynClone;
use miette::Diagnostic;
use partiql_types::PartiqlShape;
use partiql_value::Value;
use statrs::StatsError;
use std::convert::Infallible;
use std::fmt::Debug;
use std::num::TryFromIntError;
use thiserror::Error;

pub mod arrival;
pub mod constant;
pub mod data;
pub mod distributions;
mod macros;
pub mod process;
pub mod simple;
pub mod simple_numeric;
pub mod text;
pub mod timeline;
mod util;

pub const CURRENT_TICK: &str = "current_tick";

#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum DataGenerationError {
    #[error("Stats Error: `{0}-{0}`")]
    Stats(#[from] StatsError),

    #[error("Bounds Error: `{0}-{0}`")]
    Bounds(i64, i64),

    #[error("Bounds Error: `{0}-{0}`")]
    BoundsF(f64, f64),

    #[error("Integer Conversion Error: {0}")]
    IntConversionError(#[from] TryFromIntError),

    #[error("Regex Error: {0}")]
    Regex(String),

    #[error("Unknown Error: {0}")]
    Other(String),
}

impl From<Infallible> for DataGenerationError {
    fn from(_value: Infallible) -> Self {
        unreachable!();
    }
}

#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum DataSamplingError {
    #[error("Invalid Tick Error: {0}")]
    InvalidTick(String),
}

pub type DataSamplingResult<T> = Result<T, DataSamplingError>;

/// A Random Process (aka Stochastic Process) is
/// > a mathematical models of systems and phenomena that appear to vary in a random manner.
///  -- from: https://en.wikipedia.org/wiki/Stochastic_process
pub trait RandomProcess: Debug + DynClone {
    fn next_sample(&self, ctx: &SimContext) -> Option<DataSamplingResult<Sample>>;

    /// Returns [`Some(Tick)`] representing the next arrival tick for this process's samples
    /// Returns [`None`] when the process is 'finished'.
    fn next_arrival(&self, now: Tick, ctx: &SimContext) -> Option<Tick>;

    fn shape(&self) -> PartiqlShape;
}

dyn_clone::clone_trait_object!(RandomProcess);

pub type DataGenerationResult<T> = Result<T, DataGenerationError>;

pub trait ValueGenerator: Debug + DynClone {
    /// Generates a [`Value`].
    ///
    /// The value may be [`Value::Null`] or [`Value::Missing`] depending on the scripted nullability
    /// and optionality.
    ///
    /// For only non-absent values, see [`Self::present_value`].
    fn gen_value(&self, ctx: &SimContext) -> Value {
        self.present_value(ctx)
    }

    /// Generates non-absent [`Value`] (i.e., not [`Value::Null`] and not [`Value::Missing`]).
    fn present_value(&self, ctx: &SimContext) -> Value;
    fn value_type(&self) -> PartiqlShape;

    // TODO Change to `fn density(&self) -> Density;` as part of https://github.com/partiql/partiql-beamline/issues/27
    fn density(&self) -> Option<Density>;
}

dyn_clone::clone_trait_object!(ValueGenerator);

pub trait ValueGeneratorBoxed: ValueGenerator
where
    Self: 'static,
{
    fn boxed(self) -> Box<dyn ValueGenerator>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T> ValueGeneratorBoxed for T where T: ValueGenerator + 'static {}

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
