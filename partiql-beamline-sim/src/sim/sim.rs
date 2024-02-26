use std::default::Default;
use std::error::Error;

use ion_rs::lazy::reader::LazyReader;
use miette::Diagnostic;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;

use crate::gen::Processes;
use crate::primitives::{Event, ProcessId, Sample, Tick};
use crate::reader::ProcessParser;
use crate::sim::timeline::Timeline;
use crate::sim::{SimConfig, SimConfigError, SimConfigResult};

/// Error during simulation
#[derive(Debug, Error, Diagnostic)]
#[error("Sim Config Error")]
#[non_exhaustive]
pub enum SimError {
    #[error("Config error: {0}")]
    ConfigError(SimConfigError),

    #[error("Rand error: {0}")]
    RandError(rand::Error),

    #[error("Rand error: {0}")]
    TimeError(time::error::Error),

    #[error("Unknown Process: {0:?}")]
    UnknownProcess(ProcessId),

    #[error("Unknown Error: {0}")]
    UnknownError(Box<dyn Error + Send + Sync + 'static>),
}

impl From<rand::Error> for SimError {
    fn from(e: rand::Error) -> Self {
        Self::RandError(e)
    }
}

impl From<time::error::Error> for SimError {
    fn from(e: time::error::Error) -> Self {
        Self::TimeError(e)
    }
}

impl From<SimConfigError> for SimError {
    fn from(e: SimConfigError) -> Self {
        Self::ConfigError(e)
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for SimError {
    fn from(e: Box<dyn Error + Send + Sync + 'static>) -> Self {
        Self::UnknownError(e)
    }
}

pub type SimResult<T> = Result<T, SimError>;

pub struct Sim {
    #[allow(unused)]
    config: SimConfig,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    time: Tick,
    timeline: Timeline,
    processes: Processes,
}

impl Sim {
    /// Create a [`Sim`] from the provided [`SimConfig`]
    pub fn from_config(config: SimConfig, script: &[u8]) -> SimResult<Self> {
        let seed = config.seed;
        let mut sim = Sim {
            config,
            root_rng: Pcg64Mcg::seed_from_u64(seed),
            time: Tick(0),
            timeline: Timeline::default(),
            processes: Self::parse_processes(seed, script)?,
        };

        // Populate the timeline with initial events for each process
        for pid in sim.processes.ids() {
            sim.schedule_process(pid)?
        }

        Ok(sim)
    }

    fn parse_processes(seed: u64, script: &[u8]) -> SimConfigResult<Processes> {
        let parser = ProcessParser::new(seed)?;
        let mut reader = LazyReader::new(script);
        Ok(parser.parse(&mut reader)?)
    }

    /// Generate the next sample from this simulation
    pub fn next_sample(&mut self) -> SimResult<Option<Sample>> {
        match self.timeline.pop() {
            None => Ok(None),
            Some(Event { pid, sample }) => {
                // update simulation's current tick to the tick for this event
                self.time = sample.tick;

                // schedule sampling the process again
                self.schedule_process(pid)?;

                Ok(Some(sample))
            }
        }
    }

    fn schedule_process(&mut self, pid: ProcessId) -> SimResult<()> {
        // find the process
        let proc = self
            .processes
            .get(pid)
            .ok_or(SimError::UnknownProcess(pid))?;

        // generate its next sample
        let next_sample = proc.next_sample(self.time).transpose()?;

        // add the sample to the timeline
        if let Some(sample) = next_sample {
            self.timeline.push(Event { pid, sample });
        }

        Ok(())
    }
}
