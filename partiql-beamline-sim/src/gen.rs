use crate::primitives::{ProcessId, Sample, Tick};
use partiql_value::{Tuple, Value};
use rand::distributions::Distribution;
use rand::Rng;
use statrs::distribution::Exp;
use statrs::StatsError;
use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use thiserror::Error;
use time::Duration;

use crate::sim::context::SimContext;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RandomVariableError {
    #[error("Stats Error: `{0}-{0}`")]
    Stats(StatsError),

    #[error("Bounds Error: `{0}-{0}`")]
    Bounds(i64, i64),

    #[error("Error: `{0}`")]
    Other(String),
}

impl From<StatsError> for RandomVariableError {
    fn from(value: StatsError) -> Self {
        RandomVariableError::Stats(value)
    }
}

pub type RandomVariableResult<T> = Result<T, RandomVariableError>;

#[derive(Default)]
pub struct Processes {
    processes: Vec<Box<dyn Process>>,
}

impl Processes {
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }
    pub fn add(&mut self, p: Box<dyn Process>) -> ProcessId {
        let id = ProcessId(self.processes.len());
        self.processes.push(p);
        id
    }

    pub fn get(&self, pid: ProcessId) -> Option<&dyn Process> {
        self.processes.get(pid.0).map(|b| b.as_ref())
    }

    pub fn ids(&self) -> Vec<ProcessId> {
        (0..self.processes.len()).map(ProcessId).collect()
    }
}

pub trait Process {
    fn next_sample(
        &self,
        now: Tick,
        ctx: &SimContext,
    ) -> Option<Result<Sample, Box<dyn std::error::Error + Send + Sync + 'static>>>;
    fn children(&self) -> Option<&[&dyn Process]> {
        None
    }
}

pub trait ArrivalTime: Debug {
    fn next_arrival(&self, now: Tick) -> Tick;
}

/// A stochastic process arrival time generator for homogenenously distributed arrivals.
///
/// see https://en.wikipedia.org/wiki/Poisson_point_process#Homogeneous_Poisson_point_process
pub struct HomogeneousPoisson<R>
where
    R: Rng + Sized,
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
    R: Rng + Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HomogeneousPoisson")
            .field("interarrival_time", &self.interarrival_time)
            .finish()
    }
}

impl<R> HomogeneousPoisson<R>
where
    R: Rng + Sized,
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
    R: Rng + Sized,
{
    fn next_arrival(&self, now: Tick) -> Tick {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let next = self.exp.sample(rng);
        let millis = Duration::seconds_f64(next).whole_milliseconds() as u128;
        Tick(now.0 + millis)
    }
}

pub trait ValueGenerator: Debug {
    fn gen_value(&self, ctx: &SimContext) -> Value;
}

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

#[derive(Debug)]
pub struct ConstantGenerator {
    pub constant: Value,
}

impl ValueGenerator for ConstantGenerator {
    fn gen_value(&self, _ctx: &SimContext) -> Value {
        self.constant.clone()
    }
}

pub enum SimpleRandomVariableKind {
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
}

impl SimpleRandomVariableKind {
    pub fn from_string(s: &str) -> RandomVariableResult<Self> {
        match s {
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
            _ => Err(RandomVariableError::Other(format!(
                "Unknown random variable kind `{s}`"
            ))),
        }
    }
    pub fn create<R>(&self, rng: R) -> RandomVariableResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + 'static,
    {
        match self {
            SimpleRandomVariableKind::String => {
                todo!()
            }
            SimpleRandomVariableKind::UInt8 => Ok(Box::new(simple_u8(rng)?)),
            SimpleRandomVariableKind::UInt16 => Ok(Box::new(simple_u16(rng)?)),
            SimpleRandomVariableKind::UInt32 => Ok(Box::new(simple_u32(rng)?)),
            SimpleRandomVariableKind::UInt64 => Ok(Box::new(simple_u64(rng)?)),
            SimpleRandomVariableKind::Int8 => Ok(Box::new(simple_i8(rng)?)),
            SimpleRandomVariableKind::Int16 => Ok(Box::new(simple_i16(rng)?)),
            SimpleRandomVariableKind::Int32 => Ok(Box::new(simple_i32(rng)?)),
            SimpleRandomVariableKind::Int64 => Ok(Box::new(simple_i64(rng)?)),
            SimpleRandomVariableKind::Float64 => Ok(Box::new(simple_f64(rng)?)),
            SimpleRandomVariableKind::Bool => Ok(Box::new(simple_bool(rng)?)),
        }
    }
}

pub fn simple_choose<R>(
    rng: R,
    choices: Vec<Value>,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if choices.is_empty() {
        return Err(RandomVariableError::Other(
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
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    let name = "UniformBool".into();
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Bernoulli::new(0.5)?;
    let f = move |rng: &mut R| Value::from(dist.sample(rng) > 0f64);
    Ok(SimpleRandomVariable { name, rng, f })
}

pub fn simple_u8<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_u8(rng, u8::MIN as i64, u8::MAX as i64)
}

pub fn simple_u16<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_u16(rng, u16::MIN as i64, u16::MAX as i64)
}

pub fn simple_u32<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_u32(rng, u32::MIN as i64, u32::MAX as i64)
}

pub fn simple_u64<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_u64(rng, u64::MIN as i64, i64::MAX)
}

pub fn simple_i8<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_i8(rng, i8::MIN as i64, i8::MAX as i64)
}

pub fn simple_i16<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_i16(rng, i16::MIN as i64, i16::MAX as i64)
}

pub fn simple_i32<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_i32(rng, i32::MIN as i64, i32::MAX as i64)
}

pub fn simple_i64<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_i64(rng, i64::MIN, i64::MAX)
}

pub fn simple_f64<R>(
    rng: R,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    bounded_f64(rng, i8::MIN as f64, i8::MAX as f64)
}

pub fn bounded_u8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < u8::MIN as i64 || max > u8::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < u16::MIN as i64 || max > u16::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < u32::MIN as i64 || max > u32::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < u64::MIN as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < i8::MIN as i64 || max > i8::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < i16::MIN as i64 || max > i16::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    if min < i32::MIN as i64 || max > i32::MAX as i64 {
        Err(RandomVariableError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
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
) -> RandomVariableResult<SimpleRandomVariable<R, impl Fn(&mut R) -> Value>>
where
    R: Rng + Sized,
{
    let name = format!("UniformF64::{{ low: {min}, high: {max} }}");
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Uniform::new(min, max)?;
    let f = move |rng: &mut R| Value::from(dist.sample(rng));
    Ok(SimpleRandomVariable { name, rng, f })
}

pub struct SimpleRandomVariable<R, F>
where
    R: Rng + Sized,
    F: Fn(&mut R) -> Value,
{
    name: String,

    /// The source of randomness
    rng: RefCell<R>,

    f: F,
}

impl<R, F> Debug for SimpleRandomVariable<R, F>
where
    R: Rng + Sized,
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
    R: Rng + Sized,
    F: Fn(&mut R) -> Value,
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

impl Process for SimpleProcess {
    fn next_sample(
        &self,
        now: Tick,
        ctx: &SimContext,
    ) -> Option<Result<Sample, Box<dyn std::error::Error + Send + Sync + 'static>>> {
        let tick = self.arrival.next_arrival(now);
        let value = self.data.gen_value(ctx);
        Some(Ok(Sample { tick, value }))
    }
}
