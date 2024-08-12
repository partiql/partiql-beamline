use crate::gen::{ArrivalTime, DataSamplingError, RandomProcess, ValueGenerator, CURRENT_TICK};
use crate::primitives::{DataSetId, DataSetName, ProcessId, Sample, Tick};
use crate::sim::{ConstantBindingValue, DatasetTypeMapping, SimContext};
use indexmap::map::Entry;
use indexmap::IndexMap;
use partiql_types::{BagType, PartiqlShape};

#[derive(Debug, Clone)]
pub struct SimpleProcess {
    pub arrival: Box<dyn ArrivalTime>,
    pub data: Box<dyn ValueGenerator>,
}

impl RandomProcess for SimpleProcess {
    fn next_sample(&self, ctx: &SimContext) -> Option<Result<Sample, DataSamplingError>> {
        let tick_binding_value = ctx.get_binding(CURRENT_TICK).expect("tick binding value");
        if let &ConstantBindingValue::Tick(tick) = tick_binding_value {
            let value = self.data.present_value(ctx);
            Some(Ok(Sample { tick, value }))
        } else {
            Some(Err(DataSamplingError::InvalidTick(format!(
                "{tick_binding_value:?}"
            ))))
        }
    }

    fn next_arrival(&self, now: Tick, _ctx: &SimContext) -> Option<Tick> {
        self.arrival.next_arrival(now)
    }

    fn shape(&self) -> PartiqlShape {
        self.data.value_type()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RandomDataSets {
    processes: Vec<(DataSetName, Box<dyn RandomProcess>)>,
}

impl RandomDataSets {
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    pub fn add(&mut self, d: DataSetName, p: Box<dyn RandomProcess>) -> ProcessId {
        let id = ProcessId(self.processes.len());
        self.processes.push((d, p));
        id
    }

    #[inline]
    pub fn get(&self, pid: ProcessId) -> Option<(&DataSetName, &dyn RandomProcess)> {
        self.processes.get(pid.0).map(|(d, rp)| (d, rp.as_ref()))
    }

    pub fn ids(&self) -> Vec<ProcessId> {
        (0..self.processes.len()).map(ProcessId).collect()
    }

    pub fn datasets(&self) -> Vec<(DataSetId, DataSetName)> {
        self.processes
            .iter()
            .enumerate()
            .map(|(id, (name, _))| (DataSetId(id), name.clone()))
            .collect()
    }

    pub fn decompose(self) -> IndexMap<DataSetName, RandomDataSets> {
        let mut procs = IndexMap::default();

        for (d, p) in self.processes {
            let rp = procs
                .entry(d.clone())
                .or_insert_with(|| RandomDataSets { processes: vec![] });
            rp.processes.push((d, p));
        }

        procs
    }

    pub fn shape(&self) -> DatasetTypeMapping {
        let mut kvs: IndexMap<&str, _> = IndexMap::default();
        for (d, rp) in &self.processes {
            match kvs.entry(&d.0) {
                Entry::Occupied(mut e) => {
                    let x: &mut PartiqlShape = e.get_mut();
                    let y = rp.shape();
                    let u = x.clone().union_with(y); // todo make not need clone
                    *x = u;
                }
                Entry::Vacant(e) => {
                    e.insert(rp.shape());
                }
            }
        }

        kvs.into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    PartiqlShape::new_bag(BagType::new(Box::new(v))),
                )
            })
            .collect()
    }
}
