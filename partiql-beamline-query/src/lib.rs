use crate::strategy::StrategyError;
use miette::Diagnostic;
use partiql_beamline::sim::{ISim, MultiSim, SimError};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("Query Generation Error")]
#[non_exhaustive]
pub enum QueryGenError {
    #[error("StrategyError: {0}")]
    Strategy(#[from] StrategyError),
    #[error("SimError: {0}")]
    Sim(#[from] SimError),
    #[error("Unknown: {0}")]
    Unknown(String),
}

/// Result of TODO
pub type QueryGenResult<T> = Result<T, QueryGenError>;

pub mod generator;
pub mod strategy;
