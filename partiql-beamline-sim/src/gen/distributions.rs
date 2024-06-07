use crate::gen::{DataGenerationResult, ValueGenerator};
use crate::sim::context::SimContext;
use partiql_types::PartiqlType;
use partiql_value::Value;
use rand::Rng;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::DerefMut;

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
    inner: Inner,
}

impl<R, Inner> RandomVariable<R, Inner>
where
    R: Rng + Sized + Clone,
    Inner: InnerValueGenerator<R>,
{
    pub(crate) fn create(rng: R, inner: Inner) -> DataGenerationResult<Self> {
        let rng = RefCell::new(rng);
        Ok(RandomVariable { rng, inner })
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
        self.inner.gen_value(rng, ctx)
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
