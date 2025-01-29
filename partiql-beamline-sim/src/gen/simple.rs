use crate::gen::distributions::{Density, InnerValueGenerator, Meta, RandomVariable};
use crate::gen::{DataGenerationError, DataGenerationResult, ValueGenerator};
use crate::sim::SimContext;
use partiql_types::{type_bool, type_string, type_array, PartiqlShape, PartiqlShapeBuilder, PartiqlNoIdShapeBuilder};

use partiql_value::{List, Value};
use rand::distributions::Distribution;
use rand::Rng;
use std::fmt::Debug;

use crate::gen::macros::*;
use crate::gen::util::ValueTypeInference;
use debug_ignore::DebugIgnore;
use rand::seq::SliceRandom;

rv_typedef!(
    /// Generates a single value by using a Discrete Uniform to choose amongst inner generators.
    SimpleAnyOf, SimpleAnyOfImpl);
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct SimpleAnyOfImpl {
    generators: Vec<Box<dyn ValueGenerator>>,
    dist: DebugIgnore<statrs::distribution::DiscreteUniform>,
}

impl<R> SimpleAnyOf<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(
        generators: Vec<Box<dyn ValueGenerator>>,
        rng: R,
        meta: Meta,
        density: Density,
    ) -> DataGenerationResult<Self> {
        let dist =
            statrs::distribution::DiscreteUniform::new(0, (generators.len() - 1) as i64)?.into();
        RandomVariable::create(rng, meta, density, SimpleAnyOfImpl { generators, dist })
    }
}
impl<R> InnerValueGenerator<R> for SimpleAnyOfImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, ctx: &SimContext) -> Value {
        let idx = self.dist.sample(rng) as i64;
        let generator = &self.generators[idx as usize];
        generator.gen_value(ctx)
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        let types: Vec<PartiqlShape> = self.generators.iter().map(|g| g.shape(bld)).collect();
        bld.any_of(types)
    }
}

rv_typedef!(
    /// Generates a single value by using a Discrete Uniform to choose amongst inner generators.
    SimpleChoose, SimpleChooseImpl);
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct SimpleChooseImpl {
    choices: Vec<Value>,
}

impl<R> SimpleChoose<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(
        choices: Vec<Value>,
        rng: R,
        meta: Meta,
        density: Density,
    ) -> DataGenerationResult<Self> {
        if choices.is_empty() {
            return Err(DataGenerationError::Other(
                "Empty choice vector".to_string(),
            ));
        }
        RandomVariable::create(rng, meta, density, SimpleChooseImpl { choices })
    }
}

impl<R> InnerValueGenerator<R> for SimpleChooseImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        self.choices.as_slice().choose(rng).unwrap().clone()
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        let types: Vec<PartiqlShape> = self.choices.iter().map(|v| v.infer_shape(bld)).collect();
        bld.any_of(types)
    }
}

rv_typedef!(
    /// Uses a Discrete Uniform to generate a length and uses the inner generator for each element
    SimpleArray, SimpleArrayImpl);
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct SimpleArrayImpl {
    min: i64,
    max: i64,
    elem_generator: Box<dyn ValueGenerator>,
    dist: DebugIgnore<statrs::distribution::DiscreteUniform>,
}

impl<R> SimpleArray<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(
        min: i64,
        max: i64,
        elem_generator: Box<dyn ValueGenerator>,
        rng: R,
        meta: Meta,
        density: Density,
    ) -> DataGenerationResult<Self> {
        if min > max {
            Err(DataGenerationError::Bounds(min, max))
        } else {
            let dist = statrs::distribution::DiscreteUniform::new(min, max)?.into();
            RandomVariable::create(
                rng,
                meta,
                density,
                crate::gen::simple::SimpleArrayImpl {
                    min,
                    max,
                    elem_generator,
                    dist,
                },
            )
        }
    }
}
impl<R> InnerValueGenerator<R> for crate::gen::simple::SimpleArrayImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, ctx: &SimContext) -> Value {
        let array_length = self.dist.sample(rng) as usize;
        let array: Vec<_> = std::iter::repeat_with(|| self.elem_generator.gen_value(ctx))
            .take(array_length)
            .collect();
        Value::List(Box::new(List::from(array)))
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        type_array!(bld, self.elem_generator.shape(bld))
    }
}

rv_typedef!(
    /// Generates a boolean based on a percent likelihood
    SimpleBool, SimpleBoolImpl);
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct SimpleBoolImpl {
    pct: f64,
    dist: DebugIgnore<statrs::distribution::Bernoulli>,
}

impl<R> SimpleBool<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(pct: f64, rng: R, meta: Meta, density: Density) -> DataGenerationResult<Self> {
        let dist = statrs::distribution::Bernoulli::new(pct)?.into();
        RandomVariable::create(rng, meta, density, SimpleBoolImpl { pct, dist })
    }
}
impl<R> InnerValueGenerator<R> for SimpleBoolImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        // `dist.sample` returns either 0.0 or 1.0
        Value::from(self.dist.sample(rng) > 0f64)
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        type_bool!(bld)
    }
}

make_rv_stateless!(
    /// Yields a Version 4 UUID
    Uuid, UuidImpl);
impl<R> InnerValueGenerator<R> for UuidImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        let mut uuid_bytes = uuid::Bytes::default();
        rng.fill_bytes(&mut uuid_bytes);
        let id = uuid::Builder::from_random_bytes(uuid_bytes).into_uuid();
        Value::from(id.to_string())
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        type_string!(bld)
    }
}
