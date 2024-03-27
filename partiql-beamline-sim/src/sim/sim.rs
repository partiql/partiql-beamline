use std::default::Default;
use std::error::Error;

use crate::gen;
use ion_rs::lazy::reader::LazyReader;
use miette::Diagnostic;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;

use crate::gen::{DataSamplingError, RandomProcesses};
use crate::primitives::{Event, ProcessId, Sample, Tick};
use crate::reader::{ProcessConfigError, ProcessParser};
use crate::sim::context::{ConstantBindingValue, SimContext, SimContextError};
use crate::sim::timeline::Timeline;
use crate::sim::{SimConfig, SimConfigError, SimConfigResult};

/// Error during simulation
#[derive(Debug, Error, Diagnostic)]
#[error("Sim Error")]
#[non_exhaustive]
pub enum SimError {
    #[error("Config error: {0}")]
    ConfigError(#[from] SimConfigError),

    #[error("Config error: {0}")]
    ProcessConfigError(#[from] ProcessConfigError),

    #[error("Rand error: {0}")]
    RandError(#[from] rand::Error),

    #[error("Rand error: {0}")]
    TimeError(#[from] time::error::Error),

    #[error("Rand error: {0}")]
    ContextError(SimContextError),

    #[error("Unknown Process: {0:?}")]
    UnknownProcess(ProcessId),

    #[error("Unknown Process: {0:?}")]
    ProcessSamplingError(#[from] DataSamplingError),

    #[error("Unknown Error: {0}")]
    UnknownError(#[from] Box<dyn Error + Send + Sync + 'static>),
}

pub type SimResult<T> = Result<T, SimError>;

pub struct Sim {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    time: Tick,
    timeline: Timeline,
    processes: RandomProcesses,
}

impl Sim {
    /// Create a [`Sim`] from the provided [`SimConfig`]
    pub fn from_config(config: SimConfig, script: &[u8]) -> SimResult<Self> {
        let seed = config.seed;
        let context = SimContext::new(config);

        let processes = Self::parse_processes(seed, script, &context)?;
        let mut sim = Sim {
            context,
            processes,
            root_rng: Pcg64Mcg::seed_from_u64(seed),
            time: Tick(0),
            timeline: Timeline::default(),
        };

        // Populate the timeline with initial events for each process
        for pid in sim.processes.ids() {
            sim.schedule_pid(pid)?
        }

        // Set the initial bindings
        sim.context
            .overwrite_binding(gen::CURRENT_TICK, &ConstantBindingValue::Tick(sim.time));

        Ok(sim)
    }

    pub fn add_binding_to_context(
        &mut self,
        key: &str,
        value: &ConstantBindingValue,
    ) -> SimResult<()> {
        match self.context.add_binding(key, value) {
            Ok(_) => Ok(()),
            Err(e) => Err(SimError::ContextError(e)),
        }
    }

    fn parse_processes(
        seed: u64,
        script: &[u8],
        ctx: &SimContext,
    ) -> SimConfigResult<RandomProcesses> {
        let registry = Default::default();
        let parser = ProcessParser::new(seed, registry, ctx)?;
        let mut reader = LazyReader::new(script);
        Ok(parser.parse(&mut reader)?)
    }

    /// Generate the next sample from this simulation
    pub fn next_sample(&mut self) -> SimResult<Option<Sample>> {
        match self.timeline.pop() {
            None => Ok(None),
            Some(Event { pid, tick }) => {
                // update simulation's current tick to the tick for this event
                if tick > self.time {
                    self.time = tick;
                    self.context.overwrite_binding(
                        gen::CURRENT_TICK,
                        &ConstantBindingValue::Tick(self.time),
                    );
                }

                // find the process
                let proc = self
                    .processes
                    .get(pid)
                    .ok_or(SimError::UnknownProcess(pid))?;

                // re-schedule sampling the process again
                let tick = proc.next_arrival(self.time, &self.context);
                self.timeline.push(Event { pid, tick });

                // generate the process's sample
                Ok(proc.next_sample(&self.context).transpose()?)
            }
        }
    }

    fn schedule_pid(&mut self, pid: ProcessId) -> SimResult<()> {
        // find the process
        let proc = self
            .processes
            .get(pid)
            .ok_or(SimError::UnknownProcess(pid))?;

        let tick = proc.next_arrival(self.time, &self.context);
        self.timeline.push(Event { pid, tick });
        Ok(())
    }
}
