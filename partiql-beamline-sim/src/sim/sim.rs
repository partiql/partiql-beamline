use indexmap::IndexMap;
use std::default::Default;
use std::error::Error;

use crate::gen;
use miette::Diagnostic;
use partiql_types::{PartiqlShape, PartiqlNoIdShapeBuilder};

use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use thiserror::Error;
use time::format_description::well_known::Iso8601;

use crate::gen::process::RandomDataSets;
use crate::gen::DataSamplingError;
use crate::primitives::{DataSetId, DataSetName, Event, ProcessId, Sample, Tick};
use crate::reader::error::ProcessParseError;
use crate::reader::ProcessParser;
use crate::sim::context::{ConstantBindingValue, SimContext, SimContextError};
use crate::sim::timeline::Timeline;
use crate::sim::{SimConfig, SimConfigBuilderError, SimConfigError, SimConfigResult};
use crate::source::{SimSource, SimSourceError};

pub const DATETIME_FORMAT: Iso8601 = Iso8601::DEFAULT;

/// Error during simulation
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum SimError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ConfigError(#[from] SimConfigError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SimSourceError(#[from] SimSourceError),

    #[error("Config error: {0}")]
    ConfigBuilderError(#[from] SimConfigBuilderError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ProcessError(#[from] ProcessParseError),

    #[error("Rand error: {0}")]
    RandError(#[from] rand::Error),

    #[error("Time error: {0}")]
    TimeError(#[from] time::error::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ContextError(SimContextError),

    #[error("Unknown Process: {0:?}")]
    UnknownProcess(ProcessId),

    #[error("Unknown DataSet: {0:?}")]
    UnknownDataSet(DataSetId),

    #[error("Unknown DataSet name: {0:?}")]
    UnknownDataSetName(DataSetName),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ProcessSamplingError(#[from] DataSamplingError),

    #[error("Unknown Error: {0}")]
    UnknownError(#[from] Box<dyn Error + Send + Sync + 'static>),
}

pub type SimResult<T> = Result<T, SimError>;

pub type SimIterator = dyn Iterator<Item=SimResult<Sample>>;

#[derive(Debug, Clone)]
pub struct DatasetTypeMapping {
    mapping: IndexMap<String, PartiqlShape>,
}

impl FromIterator<(String, PartiqlShape)> for DatasetTypeMapping {
    fn from_iter<T: IntoIterator<Item=(String, PartiqlShape)>>(iter: T) -> Self {
        let mapping = iter.into_iter().collect();
        Self { mapping }
    }
}

impl<const N: usize> From<[(String, PartiqlShape); N]> for DatasetTypeMapping {
    fn from(value: [(String, PartiqlShape); N]) -> Self {
        let mapping = IndexMap::from(value);
        Self { mapping }
    }
}

impl DatasetTypeMapping {
    pub fn get_dataset(&self, key: &str) -> Option<NameAndShape> {
        self.mapping.get(key).map(|shp| {
            let name = key.to_string();
            let shape = shp.clone();
            NameAndShape { name, shape }
        })
    }
    pub fn get_shape(&self, key: &str) -> Option<&PartiqlShape> {
        self.mapping.get(key)
    }

    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    pub fn shapes(&self) -> impl Iterator<Item=&PartiqlShape> {
        self.mapping.values()
    }

    pub fn names(&self) -> impl Iterator<Item=&String> {
        self.mapping.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &PartiqlShape)> {
        self.mapping.iter()
    }
}

impl IntoIterator for DatasetTypeMapping {
    type Item = (String, PartiqlShape);
    type IntoIter = indexmap::map::IntoIter<String, PartiqlShape>;

    fn into_iter(self) -> Self::IntoIter {
        self.mapping.into_iter()
    }
}

impl<'a> IntoIterator for &'a DatasetTypeMapping {
    type Item = (&'a String, &'a PartiqlShape);
    type IntoIter = indexmap::map::Iter<'a, String, PartiqlShape>;

    fn into_iter(self) -> Self::IntoIter {
        self.mapping.iter()
    }
}

#[derive(Debug, Clone)]
pub struct NameAndShape {
    pub name: String,
    pub shape: PartiqlShape,
}

#[derive(Debug, Clone)]
pub struct SimBuilder {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    t0: Tick,
    processes: RandomDataSets,
}

impl SimBuilder {
    /// Create a [`SimBuilder`] from the provided [`SimConfig`]
    pub fn from_config(config: SimConfig, source: SimSource) -> SimResult<Self> {
        let seed = config.seed;
        let root_rng = Pcg64Mcg::seed_from_u64(seed);
        let t0 = Tick(0);
        let mut context = SimContext::new(config)?;
        // Set the initial bindings
        context.overwrite_binding(gen::CURRENT_TICK, &ConstantBindingValue::Tick(t0));

        let processes = Self::parse_processes(seed, source, &context)?;

        Ok(SimBuilder {
            context,
            processes,
            root_rng,
            t0,
        })
    }

    fn parse_processes(
        seed: u64,
        source: SimSource,
        ctx: &SimContext,
    ) -> SimConfigResult<RandomDataSets> {
        let registry = Default::default();
        let parser = ProcessParser::new(seed, registry, ctx)?;
        Ok(parser.parse(source)?)
    }

    pub fn build_time_ordered(self) -> SimResult<Sim> {
        Sim::from_builder(self)
    }

    pub fn build_multi_dataset(self) -> SimResult<MultiSim> {
        MultiSim::from_builder(self)
    }
}

pub trait ISim {
    /// The [`SimConfig`] used to construct this.
    fn config(&self) -> &SimConfig;

    /// The [`DatasetTypeMapping`] that defines the `shape` of this sim.
    fn shape(&self) -> DatasetTypeMapping;

    /// The 'dataset's represented by this simulation.
    fn datasets(&self) -> Vec<(DataSetId, DataSetName)>;

    /// Get a 'dataset' id from its name.
    fn get_dataset_id(&self, name: &DataSetName) -> Option<DataSetId> {
        self.datasets()
            .iter()
            .find(|(_, ds_name)| ds_name == name)
            .map(|(id, _)| *id)
    }
}

#[derive(Debug, Clone)]
pub struct Sim {
    context: SimContext,

    #[allow(unused)]
    root_rng: Pcg64Mcg,

    processes: RandomDataSets,

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

impl ISim for Sim {
    fn config(&self) -> &SimConfig {
        self.context.config()
    }

    fn shape(&self) -> DatasetTypeMapping {
        self.processes.shape(&mut PartiqlNoIdShapeBuilder::default())
    }

    fn datasets(&self) -> Vec<(DataSetId, DataSetName)> {
        self.processes.datasets()
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

#[derive(Debug, Clone)]
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

        let build = |p| {
            SimBuilder {
                context: context.clone(),
                root_rng: root_rng.clone(),
                t0,
                processes: p,
            }
                .build_time_ordered()
        };

        let shape = processes.shape(&mut PartiqlNoIdShapeBuilder::default());

        let processes: Result<Vec<_>, _> = processes
            .decompose()
            .into_iter()
            .map(|(n, p)| build(p).map(|sim| (n, sim)))
            .collect();
        let (datasets, sims): (Vec<_>, Vec<_>) = processes?.into_iter().unzip();

        Ok(MultiSim {
            context,
            root_rng,
            dataset_shapes: shape,
            datasets,
            sims,
        })
    }

    pub fn for_dataset(&mut self, id: DataSetId) -> SimResult<&mut Sim> {
        self.sims
            .get_mut(id.0)
            .ok_or_else(|| SimError::UnknownDataSet(id))
    }

    pub fn for_dataset_name<'a>(&mut self, name: impl Into<DataSetName>) -> SimResult<&mut Sim> {
        let name = name.into();
        self.get_dataset_id(&name)
            .ok_or_else(|| SimError::UnknownDataSetName(name))
            .and_then(|id| self.for_dataset(id))
    }

    pub fn into_dataset_sims<P>(self, predicate: P) -> Vec<(DataSetName, Sim)>
    where
        Self: Sized,
        P: Fn(&DataSetName) -> bool,
    {
        self.datasets
            .into_iter()
            .zip(self.sims.into_iter())
            .filter(|(n, s)| predicate(n))
            .collect()
    }

    pub fn dataset_sims<P>(&mut self, predicate: P) -> Vec<(&DataSetName, &mut Sim)>
    where
        Self: Sized,
        P: Fn(&DataSetName) -> bool,
    {
        self.datasets
            .iter()
            .zip(self.sims.iter_mut())
            .filter(|(n, s)| predicate(n))
            .collect()
    }

    /// Generate the next sample from this simulation
    fn next_sample(&mut self, id: DataSetId) -> SimResult<Option<Sample>> {
        self.sims[id.0].next_sample()
    }
}

impl ISim for MultiSim {
    fn config(&self) -> &SimConfig {
        self.context.config()
    }
    fn shape(&self) -> DatasetTypeMapping {
        self.dataset_shapes.clone()
    }

    fn datasets(&self) -> Vec<(DataSetId, DataSetName)> {
        self.datasets
            .iter()
            .enumerate()
            .map(|(i, d)| (DataSetId(i), d.clone()))
            .collect()
    }

    fn get_dataset_id(&self, name: &DataSetName) -> Option<DataSetId> {
        self.datasets.iter().position(|d| d == name).map(DataSetId)
    }
}
