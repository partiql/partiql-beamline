use crate::gen::distributions::{Density, InnerValueGenerator, Meta, RandomVariable};
use crate::gen::{DataGenerationResult, CURRENT_TICK};
use crate::primitives::Tick;
use crate::sim::{ConstantBindingValue, SimContext};
use partiql_types::{PartiqlShape, TYPE_DATETIME, TYPE_INT64};
use partiql_value::{DateTime, Value};
use rand::Rng;
use statrs::distribution::DiscreteUniform;
use std::ops::Add;
use time::{Duration, PrimitiveDateTime};

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

    fn value_type(&self) -> PartiqlShape {
        TYPE_INT64
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

    fn value_type(&self) -> PartiqlShape {
        TYPE_DATETIME
    }
}

rv_typedef!(
    /// Yields the simulation's current 'Time' as a timestamp when a value is generated.
    ///
    /// The current time is calculated by adding the current [`Tick`] to the simulation's start time (`t0`).
    Timestamp,
    TimestampImpl
);

#[derive(Debug, Clone)]
pub enum TimestampPrecision {
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
}

impl<R> Timestamp<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(
        rng: R,
        meta: Meta,
        density: Density,
        with_timezone: bool,
        precision: TimestampPrecision,
    ) -> DataGenerationResult<Self> {
        RandomVariable::create(
            rng,
            meta,
            density,
            TimestampImpl {
                with_timezone,
                precision,
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct TimestampImpl {
    with_timezone: bool,
    precision: TimestampPrecision,
}

impl<R> InnerValueGenerator<R> for TimestampImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, _rng: &mut R, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");

        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            let t0 = ctx.t0();
            let time = t0.add(Duration::milliseconds(*t as i64));
            let time = match self.precision {
                TimestampPrecision::Day => time
                    .replace_hour(0)
                    .unwrap()
                    .replace_minute(0)
                    .unwrap()
                    .replace_second(0)
                    .unwrap()
                    .replace_millisecond(0)
                    .unwrap(),
                TimestampPrecision::Hour => time
                    .replace_minute(0)
                    .unwrap()
                    .replace_second(0)
                    .unwrap()
                    .replace_millisecond(0)
                    .unwrap(),
                TimestampPrecision::Minute => time
                    .replace_second(0)
                    .unwrap()
                    .replace_millisecond(0)
                    .unwrap(),
                TimestampPrecision::Second => time.replace_millisecond(0).unwrap(),
                TimestampPrecision::Millisecond => time,
                TimestampPrecision::Microsecond => time,
            };
            if self.with_timezone {
                DateTime::TimestampWithTz(time).into()
            } else {
                let time = PrimitiveDateTime::new(time.date(), time.time());
                DateTime::Timestamp(time).into()
            }
        } else {
            todo!("handle unexpected value for Tick")
        }
    }

    fn value_type(&self) -> PartiqlShape {
        TYPE_DATETIME
    }
}

make_rv_stateless!(
    /// Yields the simulation's current 'Time' as a Date when a value is generated.
    ///
    /// The current time is calculated by adding the current [`Tick`] to the simulation's start time (`t0`).
    Date,
    DateImpl
);

impl<R> InnerValueGenerator<R> for DateImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, _rng: &mut R, ctx: &SimContext) -> Value {
        let tick = ctx.get_binding(CURRENT_TICK).expect("tick binding value");

        if let ConstantBindingValue::Tick(Tick(t)) = tick {
            let t0 = ctx.t0();
            let time = t0.add(Duration::milliseconds(*t as i64));
            DateTime::Date(time.date()).into()
        } else {
            todo!("handle unexpected value for Tick")
        }
    }

    fn value_type(&self) -> PartiqlShape {
        TYPE_DATETIME
    }
}
