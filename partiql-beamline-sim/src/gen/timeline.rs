use crate::gen::distributions::{InnerValueGenerator, RandomVariable};
use crate::gen::{DataGenerationResult, CURRENT_TICK};
use crate::primitives::Tick;
use crate::sim::{ConstantBindingValue, SimContext};
use partiql_types::{type_datetime, type_int64, PartiqlShape, PartiqlShapeBuilder};
use partiql_value::{DateTime, Value};
use rand::Rng;
use std::ops::Add;
use time::Duration;

use crate::gen::macros::*;

make_rv_stateless!(
    /// Yields the simulation's current [`Tick`] when a value is generated.
    TickGenerator, TickGeneratorImpl);

impl<R> InnerValueGenerator<R> for TickGeneratorImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, _rng: &mut R, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");
        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            // TODO Remove `as usize` once https://github.com/partiql/partiql-lang-rust/pull/449 is released
            (*t as usize).into()
        } else {
            todo!("handle unexpected value for Tick")
        }
    }

    fn shape(&self, bld: &mut PartiqlShapeBuilder) -> PartiqlShape {
        type_int64!(bld)
    }
}

make_rv_stateless!(
    /// Yields the simulation's current 'Time' when a value is generated.
    ///
    /// The current time is calculated by adding the current [`Tick`] to the simulation's start time (`t0`).
    InstantGenerator, InstantGeneratorImpl
);

impl<R> InnerValueGenerator<R> for InstantGeneratorImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, _rng: &mut R, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");

        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            let t0 = ctx.t0();
            let time = t0.add(Duration::milliseconds(*t as i64));
            DateTime::TimestampWithTz(time).into()
        } else {
            todo!("handle unexpected value for Tick")
        }
    }

    fn shape(&self, bld: &mut PartiqlShapeBuilder) -> PartiqlShape {
        type_datetime!(bld)
    }
}
