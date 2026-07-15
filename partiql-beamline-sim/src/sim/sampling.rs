use crate::primitives::{DataSetId, DataSetName, Sample};
use crate::sim::{ISim, MultiSim, SimResult};
use derive_builder::Builder;
use std::collections::HashSet;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SampleLimit {
    Constant(u64),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DataSetFilter {
    filters: HashSet<String>,
}

impl DataSetFilter {
    fn filter(&self, name: &DataSetName) -> bool {
        if !self.filters.is_empty() {
            self.filters.contains(name.0.as_str())
        } else {
            true
        }
    }
}

impl FromIterator<String> for DataSetFilter {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            filters: HashSet::from_iter(iter),
        }
    }
}

pub struct DataSetSampler {
    sim: MultiSim,
    filter: DataSetFilter,
    limit: SampleLimit,
}

impl DataSetSampler {
    pub fn new(sim: MultiSim, filter: DataSetFilter, limit: SampleLimit) -> Self {
        DataSetSampler { sim, filter, limit }
    }
    pub fn sim(&self) -> &MultiSim {
        &self.sim
    }

    pub fn seed(&self) -> u64 {
        self.sim.config().seed
    }

    pub fn t0(&self) -> OffsetDateTime {
        self.sim.config().t0
    }

    pub fn into_iter(
        self,
    ) -> impl Iterator<Item = (DataSetName, impl Iterator<Item = SimResult<Sample>>)> {
        let DataSetSampler {
            mut sim,
            ref filter,
            limit,
        } = self;
        match limit {
            SampleLimit::Constant(samples) => {
                let by_dataset = sim.into_dataset_sims(|n| self.filter.filter(n)).into_iter();
                let sampler = by_dataset
                    .map(move |(name, sim)| (name, sim.into_iter().take(samples as usize)));
                DataSetSamplerIntoIter { sampler }
            }
        }
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<
        Item = (
            &'a DataSetName,
            impl Iterator<Item = SimResult<Sample>> + 'a,
        ),
    > {
        match self.limit {
            SampleLimit::Constant(samples) => {
                let by_dataset = self.sim.dataset_sims(|n| self.filter.filter(n)).into_iter();
                let sampler = by_dataset
                    .map(move |(name, sim)| (name, sim.into_iter().take(samples as usize)));
                DataSetSamplerIter { sampler }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataSetSamplerIntoIter<I, II>
where
    II: Iterator<Item = SimResult<Sample>>,
    I: Iterator<Item = (DataSetName, II)>,
{
    sampler: I,
}

impl<I, II> Iterator for DataSetSamplerIntoIter<I, II>
where
    II: Iterator<Item = SimResult<Sample>>,
    I: Iterator<Item = (DataSetName, II)>,
{
    type Item = (DataSetName, II);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.sampler.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.sampler.size_hint()
    }
}

#[derive(Debug, Clone)]
pub struct DataSetSamplerIter<'a, I, II>
where
    II: Iterator<Item = SimResult<Sample>>,
    I: Iterator<Item = (&'a DataSetName, II)>,
{
    sampler: I,
}

impl<'a, I, II> Iterator for DataSetSamplerIter<'a, I, II>
where
    II: Iterator<Item = SimResult<Sample>>,
    I: Iterator<Item = (&'a DataSetName, II)>,
{
    type Item = (&'a DataSetName, II);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.sampler.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.sampler.size_hint()
    }
}
