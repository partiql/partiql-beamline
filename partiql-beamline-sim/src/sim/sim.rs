use std::collections::BTreeMap;
use std::default::Default;
use std::error::Error;

use crate::gen;
use ion_rs::{AnyEncoding, IonError, Reader};
use miette::Diagnostic;
use partiql_types::PartiqlType;

use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;
use time::format_description::well_known::Iso8601;

use crate::gen::process::RandomProcesses;
use crate::gen::DataSamplingError;
use crate::primitives::{DataSetId, DataSetName, Event, ProcessId, Sample, Tick};
use crate::reader::ProcessConfigError;
use crate::reader::ProcessParser;
use crate::sim::context::{ConstantBindingValue, SimContext, SimContextError};
use crate::sim::timeline::Timeline;
use crate::sim::{SimConfig, SimConfigBuilderError, SimConfigError, SimConfigResult};

pub const DATETIME_FORMAT: Iso8601 = Iso8601::DEFAULT;

/// Error during simulation
#[derive(Debug, Error, Diagnostic)]
#[error("Sim Error")]
#[non_exhaustive]
pub enum SimError {
    #[error("Read error: `{0}`")]
    ReadError(#[from] IonError),

    #[error("Config error: {0}")]
    ConfigError(#[from] SimConfigError),

    #[error("Config error: {0}")]
    ConfigBuilderError(#[from] SimConfigBuilderError),

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

pub type SimIterator = dyn Iterator<Item = SimResult<Sample>>;

pub type DatasetTypeMapping = BTreeMap<String, PartiqlType>;

pub struct SimBuilder {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    t0: Tick,
    processes: RandomProcesses,
}

impl SimBuilder {
    /// Create a [`SimBuilder`] from the provided [`SimConfig`]
    pub fn from_config(config: SimConfig, script: &[u8]) -> SimResult<Self> {
        let seed = config.seed;
        let root_rng = Pcg64Mcg::seed_from_u64(seed);
        let t0 = Tick(0);
        let mut context = SimContext::new(config)?;
        // Set the initial bindings
        context.overwrite_binding(gen::CURRENT_TICK, &ConstantBindingValue::Tick(t0));

        let processes = Self::parse_processes(seed, script, &context)?;

        Ok(SimBuilder {
            context,
            processes,
            root_rng,
            t0,
        })
    }

    fn parse_processes(
        seed: u64,
        script: &[u8],
        ctx: &SimContext,
    ) -> SimConfigResult<RandomProcesses> {
        let registry = Default::default();
        let parser = ProcessParser::new(seed, registry, ctx)?;
        let mut reader = Reader::new(AnyEncoding, script)?;
        Ok(parser.parse(&mut reader)?)
    }

    pub fn build_time_ordered(self) -> SimResult<Sim> {
        Sim::from_builder(self)
    }

    pub fn build_multi_dataset(self) -> SimResult<MultiSim> {
        MultiSim::from_builder(self)
    }
}

#[derive(Debug)]
pub struct Sim {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    processes: RandomProcesses,

    time: Tick,
    timeline: Timeline,
}

impl Sim {
    /// Create a [`Sim`] from the provided [`SimConfig`]
    fn from_builder(builder: SimBuilder) -> SimResult<Self> {
        let SimBuilder {
            context,
            processes,
            root_rng,
            t0: time,
        } = builder;

        // Populate the timeline with initial events for each process
        let mut timeline = Timeline::default();
        for pid in processes.ids() {
            let (_dataset, proc) = processes.get(pid).ok_or(SimError::UnknownProcess(pid))?;

            if let Some(tick) = proc.next_arrival(time, &context) {
                timeline.push(Event { pid, tick })
            } else {
                todo!("No initial arrival for process")
            }
        }

        Ok(Sim {
            context,
            root_rng,
            processes,
            time,
            timeline,
        })
    }

    pub fn config(&self) -> &SimConfig {
        self.context.config()
    }

    pub fn shape(&self) -> DatasetTypeMapping {
        self.processes.shape()
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
                let (_dataset, proc) = self
                    .processes
                    .get(pid)
                    .ok_or(SimError::UnknownProcess(pid))?;

                // re-schedule sampling the process again
                if let Some(tick) = proc.next_arrival(self.time, &self.context) {
                    self.timeline.push(Event { pid, tick });
                }

                // generate the process's sample
                Ok(proc.next_sample(&self.context).transpose()?)
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> SimIterMut<'_> {
        SimIterMut(self)
    }
}

pub struct SimIterMut<'a>(&'a mut Sim);

impl<'a> Iterator for SimIterMut<'a> {
    type Item = SimResult<Sample>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_sample().transpose()
    }
}

impl<'a> IntoIterator for &'a mut Sim {
    type Item = SimResult<Sample>;
    type IntoIter = SimIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct SimIntoIter(Sim);

impl Iterator for SimIntoIter {
    type Item = SimResult<Sample>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_sample().transpose()
    }
}

impl IntoIterator for Sim {
    type Item = SimResult<Sample>;
    type IntoIter = SimIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        SimIntoIter(self)
    }
}

#[derive(Debug)]
pub struct MultiSim {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    dataset_shapes: DatasetTypeMapping,

    datasets: Vec<DataSetName>,
    sims: Vec<Sim>,
}

impl MultiSim {
    /// Create a [`Sim`] from the provided [`SimConfig`]
    fn from_builder(builder: SimBuilder) -> SimResult<Self> {
        let SimBuilder {
            context,
            processes,
            root_rng,
            t0,
        } = builder;

        let shape = processes.shape();
        let mut processes: Vec<_> = processes.decompose().into_iter().collect();
        processes.sort_by(|(ld, _), (rd, _)| ld.cmp(rd));

        let mut datasets = Vec::default();
        let mut sims = Vec::default();
        for (d, p) in processes {
            datasets.push(d);

            let bld = SimBuilder {
                context: context.clone(),
                root_rng: root_rng.clone(),
                t0,
                processes: p,
            };
            sims.push(bld.build_time_ordered()?);
        }

        Ok(MultiSim {
            context,
            root_rng,
            dataset_shapes: shape,
            datasets,
            sims,
        })
    }

    pub fn config(&self) -> &SimConfig {
        self.context.config()
    }

    pub fn datasets(&self) -> Vec<(DataSetId, DataSetName)> {
        self.datasets
            .iter()
            .enumerate()
            .map(|(i, d)| (DataSetId(i), d.clone()))
            .collect()
    }

    pub fn get_dataset_id(&self, name: &DataSetName) -> Option<DataSetId> {
        self.datasets.iter().position(|d| d == name).map(DataSetId)
    }

    /// Generate the next sample from this simulation
    pub fn next_sample(&mut self, id: DataSetId) -> SimResult<Option<Sample>> {
        self.sims[id.0].next_sample()
    }

    pub fn for_dataset(&mut self, id: DataSetId) -> &mut Sim {
        &mut self.sims[id.0]
    }

    pub fn shape(&self) -> DatasetTypeMapping {
        self.dataset_shapes.clone()
    }
}
