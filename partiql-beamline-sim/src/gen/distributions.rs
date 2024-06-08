use crate::gen::{DataGenerationResult, ValueGenerator};
use crate::sim::context::SimContext;
use partiql_types::PartiqlType;
use partiql_value::Value;
use rand::Rng;
use rand_distr::Distribution;
use statrs::distribution::Categorical;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
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
    null: f64,
    missing: f64,
    present: f64,

    dist: Categorical,
}

impl Density {
    pub fn new(
        null_prob_mass: f64,
        missing_prob_mass: f64,
        present_prob_mass: f64,
    ) -> DataGenerationResult<Self> {
        let sum = null_prob_mass + missing_prob_mass + present_prob_mass;

        let null = null_prob_mass / sum;
        let missing = missing_prob_mass / sum;
        let present = present_prob_mass / sum;
        let dist = Categorical::new(&[null, missing, present])?;
        Ok(Self {
            null,
            missing,
            present,
            dist,
        })
    }

    fn prob_mass(&self) -> [f64; 3] {
        [self.null, self.missing, self.present]
    }

    fn sample<R>(&self, rng: &mut R) -> Presence
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
    fn gen_value(&self, rng: &mut R, ctx: &SimContext) -> Value;
    fn value_type(&self) -> PartiqlType;
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
    pub(crate) fn create(rng: R, inner: Inner) -> DataGenerationResult<Self> {
        let rng = RefCell::new(rng);

        // TODO: allow configuring null & missing probability
        let density = Density::new(0.0, 0.0, 1.0)?;

        Ok(RandomVariable {
            rng,
            density,
            inner,
        })
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
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();

        // Always draw from *both* density and the actual value generator.
        // This assures that values are stable across differing 'density' configurations.
        let presence = self.density.sample(rng);
        let present = self.inner.gen_value(rng, ctx);
        presence.to_value(present)
    }

    fn value_type(&self) -> PartiqlType {
        self.inner.value_type()
    }
}

pub struct SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value,
{
    pub(crate) name: String,
    pub(crate) typ: PartiqlType,
    pub(crate) f: F,
    rng: PhantomData<R>,
}

pub type SimpleRandomVariable<R, F> = RandomVariable<R, SimpleRandomVariableImpl<R, F>>;

impl<R, F> SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    pub fn new(rng: R, name: String, typ: PartiqlType, f: F) -> DataGenerationResult<Self> {
        let inner = SimpleRandomVariableImpl {
            name,
            typ,
            f,
            rng: PhantomData,
        };
        RandomVariable::create(rng, inner)
    }
}

impl<R, F> Clone for SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            typ: self.typ.clone(),
            f: self.f.clone(),
            rng: self.rng,
        }
    }
}

impl<R, F> Debug for SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleRandomVariable")
            .field("name", &self.name)
            .finish()
    }
}

impl<R, F> InnerValueGenerator<R> for SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn gen_value(&self, rng: &mut R, ctx: &SimContext) -> Value {
        (self.f)(rng, ctx)
    }

    fn value_type(&self) -> PartiqlType {
        self.typ.clone()
    }
}
