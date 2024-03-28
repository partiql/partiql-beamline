use crate::primitives::{DataSetName, ProcessId, Sample, Tick};
use partiql_value::{DateTime, Tuple, Value};
use rand::distributions::Distribution;
use rand::Rng;
use statrs::distribution::Exp;
use statrs::StatsError;
use std::cell::RefCell;
use std::collections::HashMap;

use dyn_clone::DynClone;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, DerefMut};
use thiserror::Error;
use time::Duration;

use crate::sim::context::{ConstantBindingValue, SimContext};

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

pub type DataGenerationResult<T> = Result<T, DataGenerationError>;

#[derive(Default)]
pub struct RandomProcesses {
    processes: Vec<(DataSetName, Box<dyn RandomProcess>)>,
}

impl RandomProcesses {
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    pub fn add(&mut self, d: DataSetName, p: Box<dyn RandomProcess>) -> ProcessId {
        let id = ProcessId(self.processes.len());
        self.processes.push((d, p));
        id
    }

    pub fn get(&self, pid: ProcessId) -> Option<(&DataSetName, &dyn RandomProcess)> {
        self.processes.get(pid.0).map(|(d, rp)| (d, rp.as_ref()))
    }

    pub fn ids(&self) -> Vec<ProcessId> {
        (0..self.processes.len()).map(ProcessId).collect()
    }

    pub fn decompose(self) -> HashMap<DataSetName, RandomProcesses> {
        let mut procs: HashMap<DataSetName, RandomProcesses> = HashMap::default();

        for (d, p) in self.processes {
            let rp = procs
                .entry(d.clone())
                .or_insert_with(|| RandomProcesses { processes: vec![] });
            rp.processes.push((d, p));
        }

        procs
    }
}

/// A Random Process (or Stochastic Process) is
/// > a mathematical models of systems and phenomena that appear to vary in a random manner.
///  -- from: https://en.wikipedia.org/wiki/Stochastic_process
pub trait RandomProcess {
    fn next_sample(&self, ctx: &SimContext) -> Option<Result<Sample, DataSamplingError>>;

    fn next_arrival(&self, now: Tick, ctx: &SimContext) -> Tick;

    fn children(&self) -> Option<&[&dyn RandomProcess]> {
        None
    }
}

pub trait ArrivalTime: Debug + DynClone {
    fn next_arrival(&self, now: Tick) -> Tick;
}
dyn_clone::clone_trait_object!(ArrivalTime);

/// A stochastic process arrival time generator for homogenenously distributed arrivals.
///
/// see https://en.wikipedia.org/wiki/Poisson_point_process#Homogeneous_Poisson_point_process
#[derive(Clone)]
pub struct HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    /// The mean time between arrivals
    interarrival_time: Duration,

    /// An exponential distribution used to generate the next interarrival time
    exp: Exp,

    /// The source of randomness
    rng: RefCell<R>,
}

impl<R> Debug for HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HomogeneousPoisson")
            .field("interarrival_time", &self.interarrival_time)
            .finish()
    }
}

impl<R> HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    pub fn from_interarrival_time(rng: R, interarrival_time: Duration) -> Self {
        let rate = interarrival_time.as_seconds_f64();
        let exp = Exp::new(1f64 / rate).expect("dist");
        let rng = RefCell::new(rng);

        HomogeneousPoisson {
            interarrival_time,
            exp,
            rng,
        }
    }
}

impl<R> ArrivalTime for HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    fn next_arrival(&self, now: Tick) -> Tick {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let next = self.exp.sample(rng);
        let millis = Duration::seconds_f64(next).whole_milliseconds() as u128;
        Tick(now.0 + millis)
    }
}

pub trait ValueGenerator: Debug + DynClone {
    fn gen_value(&self, ctx: &SimContext) -> Value;
}
dyn_clone::clone_trait_object!(ValueGenerator);

#[derive(Clone)]
pub enum SimpleRandomData {
    Single(Box<dyn ValueGenerator>),
    Collection(HashMap<String, Box<dyn ValueGenerator>>),
}

impl Debug for SimpleRandomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleRandomData::Single(s) => Debug::fmt(s, f),
            SimpleRandomData::Collection(data) => {
                let mut map = f.debug_map();
                for (k, v) in data {
                    map.entry(k, v);
                }
                map.finish()
            }
        }
    }
}

impl From<Box<dyn ValueGenerator>> for SimpleRandomData {
    fn from(value: Box<dyn ValueGenerator>) -> Self {
        Self::Single(value)
    }
}

impl<const N: usize, S, V> From<[(S, V); N]> for SimpleRandomData
where
    S: Into<String>,
    V: Into<Box<dyn ValueGenerator>>,
{
    #[inline]
    fn from(arr: [(S, V); N]) -> Self {
        let map = arr.into_iter().map(|(s, v)| (s.into(), v.into())).collect();
        SimpleRandomData::Collection(map)
    }
}

impl From<SimpleRandomData> for Box<dyn ValueGenerator> {
    fn from(value: SimpleRandomData) -> Self {
        Box::new(value)
    }
}

impl ValueGenerator for SimpleRandomData {
    fn gen_value(&self, ctx: &SimContext) -> Value {
        match self {
            SimpleRandomData::Single(rv) => rv.gen_value(ctx),
            SimpleRandomData::Collection(kvs) => {
                let it = kvs.iter().map(|(k, v)| (k, v.gen_value(ctx)));
                Tuple::from_iter(it).into()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantGenerator {
    pub constant: Value,
}

impl ValueGenerator for ConstantGenerator {
    fn gen_value(&self, _ctx: &SimContext) -> Value {
        self.constant.clone()
    }
}

#[derive(Debug, Copy, Clone)]
/// Yields the simulation's current [`Tick`] when a value is generated.
pub struct TickGenerator {}

impl ValueGenerator for TickGenerator {
    fn gen_value(&self, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");
        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            // TODO Remove `as usize` once https://github.com/partiql/partiql-lang-rust/pull/449 is released
            (*t as usize).into()
        } else {
            todo!("handle unexpected value for Tick")
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// Yields the simulation's current 'Time' when a value is generated.
///
/// The current time is calculated by adding the current [`Tick`] to the simulation's start time (`t0`).
pub struct InstantGenerator {}

impl ValueGenerator for InstantGenerator {
    fn gen_value(&self, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");

        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            let t0 = ctx.t0();
            let time = t0.add(Duration::milliseconds(*t as i64));
            DateTime::TimestampWithTz(time).into()
        } else {
            todo!("handle unexpected value for Tick")
        }
    }
}

pub enum SimpleScriptVariableKind {
    Tick,
    Instant,
    String,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float64,
    Bool,
    UUID,
}

impl SimpleScriptVariableKind {
    pub fn named() -> DataGenerationResult<Vec<(String, SimpleScriptVariableKind)>> {
        [
            "Tick",
            "Instant",
            "String",
            "UniformU8",
            "UniformU16",
            "UniformU32",
            "UniformU64",
            "UniformI8",
            "UniformI16",
            "UniformI32",
            "UniformI64",
            "UniformF64",
            "Bool",
            "UUID",
        ]
        .iter()
        .map(|s| Self::from_string(s).map(|k| (s.to_string(), k)))
        .collect()
    }

    pub fn from_string(s: &str) -> DataGenerationResult<Self> {
        match s {
            "Tick" => Ok(Self::Tick),
            "Instant" => Ok(Self::Instant),
            "String" => Ok(Self::String),
            "UniformU8" => Ok(Self::UInt8),
            "UniformU16" => Ok(Self::UInt16),
            "UniformU32" => Ok(Self::UInt32),
            "UniformU64" => Ok(Self::UInt64),
            "UniformI8" => Ok(Self::Int8),
            "UniformI16" => Ok(Self::Int16),
            "UniformI32" => Ok(Self::Int32),
            "UniformI64" => Ok(Self::Int64),
            "UniformF64" => Ok(Self::Float64),
            "Bool" => Ok(Self::Bool),
            "UUID" => Ok(Self::UUID),
            _ => Err(DataGenerationError::Other(format!(
                "Unknown random variable kind `{s}`"
            ))),
        }
    }

    pub fn create<R>(
        &self,
        rng: R,
        _ctx: &SimContext,
    ) -> DataGenerationResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + Clone + 'static,
    {
        match self {
            SimpleScriptVariableKind::String => {
                todo!()
            }
            SimpleScriptVariableKind::Tick => Ok(Box::new(simple_tick())),
            SimpleScriptVariableKind::Instant => Ok(Box::new(simple_instant())),
            SimpleScriptVariableKind::UInt8 => Ok(Box::new(simple_u8(rng)?)),
            SimpleScriptVariableKind::UInt16 => Ok(Box::new(simple_u16(rng)?)),
            SimpleScriptVariableKind::UInt32 => Ok(Box::new(simple_u32(rng)?)),
            SimpleScriptVariableKind::UInt64 => Ok(Box::new(simple_u64(rng)?)),
            SimpleScriptVariableKind::Int8 => Ok(Box::new(simple_i8(rng)?)),
            SimpleScriptVariableKind::Int16 => Ok(Box::new(simple_i16(rng)?)),
            SimpleScriptVariableKind::Int32 => Ok(Box::new(simple_i32(rng)?)),
            SimpleScriptVariableKind::Int64 => Ok(Box::new(simple_i64(rng)?)),
            SimpleScriptVariableKind::Float64 => Ok(Box::new(simple_f64(rng)?)),
            SimpleScriptVariableKind::Bool => Ok(Box::new(simple_bool(rng)?)),
            SimpleScriptVariableKind::UUID => Ok(Box::new(simple_uuid(rng)?)),
        }
    }
}

pub fn simple_tick() -> TickGenerator {
    TickGenerator {}
}

pub fn simple_instant() -> InstantGenerator {
    InstantGenerator {}
}

pub fn simple_choose<R>(
    rng: R,
    choices: Vec<Value>,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if choices.is_empty() {
        return Err(DataGenerationError::Other(
            "Empty choice vector".to_string(),
        ));
    }

    use rand::seq::SliceRandom;

    let name = "UniformChoice".into();
    let rng = RefCell::new(rng);
    let f = move |rng: &mut R| choices.as_slice().choose(rng).unwrap().clone();
    Ok(SimpleRandomVariable { name, rng, f })
}

pub fn simple_bool<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_bool(rng, 0.5)
}

pub fn simple_uuid<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = "UUID".into();
    let rng = RefCell::new(rng);
    let f = move |rng: &mut R| {
        let mut uuid_bytes = uuid::Bytes::default();
        rng.fill_bytes(&mut uuid_bytes);
        let id = uuid::Uuid::from_bytes(uuid_bytes);
        Value::from(id.to_string())
    };
    Ok(SimpleRandomVariable { name, rng, f })
}

pub fn simple_u8<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u8(rng, u8::MIN as i64, u8::MAX as i64)
}

pub fn simple_u16<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u16(rng, u16::MIN as i64, u16::MAX as i64)
}

pub fn simple_u32<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u32(rng, u32::MIN as i64, u32::MAX as i64)
}

pub fn simple_u64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u64(rng, u64::MIN as i64, i64::MAX)
}

pub fn simple_i8<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i8(rng, i8::MIN as i64, i8::MAX as i64)
}

pub fn simple_i16<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i16(rng, i16::MIN as i64, i16::MAX as i64)
}

pub fn simple_i32<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i32(rng, i32::MIN as i64, i32::MAX as i64)
}

pub fn simple_i64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i64(rng, i64::MIN, i64::MAX)
}

pub fn simple_f64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_f64(rng, i8::MIN as f64, i8::MAX as f64)
}

pub fn bounded_bool<R>(
    rng: R,
    p: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformBool::{{ p: {p} }}");
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Bernoulli::new(p)?;
    let f = move |rng: &mut R| Value::from(dist.sample(rng) > 0f64);
    Ok(SimpleRandomVariable { name, rng, f })
}

pub fn bounded_u8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u8::MIN as i64 || max > u8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u16::MIN as i64 || max > u16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u32::MIN as i64 || max > u32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u64::MIN as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i8::MIN as i64 || max > i8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i16::MIN as i64 || max > i16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i32::MIN as i64 || max > i32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformI64::{{ low: {min}, high: {max} }}");
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::DiscreteUniform::new(min, max)?;
    let f = move |rng: &mut R| Value::from(dist.sample(rng) as i64);
    Ok(SimpleRandomVariable { name, rng, f })
}

pub fn bounded_f64<R>(
    rng: R,
    min: f64,
    max: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformF64::{{ low: {min}, high: {max} }}");
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Uniform::new(min, max)?;
    let f = move |rng: &mut R| Value::from(dist.sample(rng));
    Ok(SimpleRandomVariable { name, rng, f })
}

pub struct SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R) -> Value,
{
    name: String,

    /// The source of randomness
    rng: RefCell<R>,

    f: F,
}

impl<R, F> Clone for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R) -> Value + Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            rng: self.rng.clone(),
            f: self.f.clone(),
        }
    }
}

impl<R, F> Debug for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R) -> Value,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleRandomVariable")
            .field("name", &self.name)
            .finish()
    }
}

impl<R, F> ValueGenerator for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R) -> Value + Clone,
{
    fn gen_value(&self, _ctx: &SimContext) -> Value {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        (self.f)(rng)
    }
}

#[derive(Debug)]
pub struct SimpleProcess {
    pub arrival: Box<dyn ArrivalTime>,
    pub data: Box<dyn ValueGenerator>,
}

impl RandomProcess for SimpleProcess {
    fn next_sample(&self, ctx: &SimContext) -> Option<Result<Sample, DataSamplingError>> {
        let tick_binding_value = ctx.get_binding(CURRENT_TICK).expect("tick binding value");
        if let &ConstantBindingValue::Tick(tick) = tick_binding_value {
            let value = self.data.gen_value(ctx);
            Some(Ok(Sample { tick, value }))
        } else {
            Some(Err(DataSamplingError::InvalidTick(format!(
                "{tick_binding_value:?}"
            ))))
        }
    }

    fn next_arrival(&self, now: Tick, _ctx: &SimContext) -> Tick {
        self.arrival.next_arrival(now)
    }
}
