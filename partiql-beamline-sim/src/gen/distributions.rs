use crate::gen::{DataGenerationResult, ValueGenerator};
use crate::sim::context::SimContext;
use partiql_types::PartiqlShape;
use partiql_value::Value;
use rand::Rng;
use rand_distr::Distribution;
use statrs::distribution::Categorical;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;

#[derive(Copy, Clone, Debug)]
pub enum Presence {
    Null,
    Missing,
    Present,
}

impl Presence {
    pub fn to_value<V>(self, present: V) -> Value
    where
        V: Into<Value>,
    {
        match self {
            Presence::Null => Value::Null,
            Presence::Missing => Value::Missing,
            Presence::Present => present.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Density {
    null: Option<f64>,
    missing: Option<f64>,
    present: f64,

    dist: Categorical,
}

impl Density {
    pub fn new(
        null_prob_mass: Option<f64>,
        missing_prob_mass: Option<f64>,
        present_prob_mass: f64,
    ) -> DataGenerationResult<Self> {
        let sum =
            null_prob_mass.unwrap_or(0.0) + missing_prob_mass.unwrap_or(0.0) + present_prob_mass;

        let null = null_prob_mass.map(|n| n / sum);
        let missing = missing_prob_mass.map(|n| n / sum);
        let present = present_prob_mass / sum;
        let dist = Categorical::new(&[null.unwrap_or(0.0), missing.unwrap_or(0.0), present])?;
        Ok(Self {
            null,
            missing,
            present,
            dist,
        })
    }

    pub fn nullability(&self) -> Option<f64> {
        self.null
    }

    pub fn optionality(&self) -> Option<f64> {
        self.missing
    }

    pub fn sample<R>(&self, rng: &mut R) -> Presence
    where
        R: Rng + Sized + Clone,
    {
        let i = self.dist.sample(rng) as u8;
        match i {
            0 => Presence::Null,
            1 => Presence::Missing,
            2 => Presence::Present,
            _ => unreachable!(),
        }
    }
}

pub trait InnerValueGenerator<R>: Debug + Clone
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, ctx: &SimContext) -> Value;
    fn value_type(&self) -> PartiqlShape;
}

pub struct RandomVariable<R, Inner>
where
    R: Rng + Sized + Clone,
    Inner: InnerValueGenerator<R>,
{
    /// The source of randomness
    rng: RefCell<R>,

    /// The source of Null | Missing
    density: Density,

    /// The value generator
    inner: Inner,
}

impl<R, Inner> RandomVariable<R, Inner>
where
    R: Rng + Sized + Clone,
    Inner: InnerValueGenerator<R>,
{
    pub(crate) fn create(rng: R, density: Density, inner: Inner) -> DataGenerationResult<Self> {
        let rng = RefCell::new(rng);
        Ok(RandomVariable {
            rng,
            density,
            inner,
        })
    }

    fn presence_and_value(&self, ctx: &SimContext) -> (Presence, Value) {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();

        // Always draw from *both* density and the actual value generator.
        // This assures that values are stable across differing 'density' configurations.
        let presence = self.density.sample(rng);
        let value = self.inner.present_value(rng, ctx);
        (presence, value)
    }
}

impl<R, Impl> Clone for RandomVariable<R, Impl>
where
    R: Rng + Sized + Clone,
    Impl: InnerValueGenerator<R> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            rng: self.rng.clone(),
            density: self.density.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<R, Impl> Debug for RandomVariable<R, Impl>
where
    R: Rng + Sized + Clone,
    Impl: InnerValueGenerator<R>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<R, Impl> ValueGenerator for RandomVariable<R, Impl>
where
    R: Rng + Sized + Clone,
    Impl: InnerValueGenerator<R>,
{
    fn gen_value(&self, ctx: &SimContext) -> Value {
        let (presence, value) = self.presence_and_value(ctx);
        presence.to_value(value)
    }

    fn present_value(&self, ctx: &SimContext) -> Value {
        // Deliberately disregard Null and Missing, as a non-absent value was requested.
        let (_, value) = self.presence_and_value(ctx);
        value
    }

    fn value_type(&self) -> PartiqlShape {
        self.inner.value_type()
    }
}
