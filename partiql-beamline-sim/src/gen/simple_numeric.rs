use crate::gen::distributions::{Density, InnerValueGenerator, Meta, RandomVariable};
use crate::gen::{DataGenerationError, DataGenerationResult};
use crate::sim::SimContext;

use partiql_types::{
    PartiqlShape, Static, TYPE_DOUBLE, TYPE_INT16, TYPE_INT32, TYPE_INT64, TYPE_INT8,
};
use partiql_value::Value;
use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::num_traits::FromPrimitive;
use std::fmt::Debug;

use crate::gen::macros::*;
use debug_ignore::DebugIgnore;
use statrs::distribution::Continuous;

macro_rules! rv_ranged {
    ($inner: ident, $ty: ty, $dist_ty: ty) => {
        #[derive(Debug, Clone)]
        #[doc(hidden)]
        pub struct $inner {
            /// The specified minimum value for generation
            #[allow(dead_code)]
            pub(crate) min: $ty,
            /// The specified maximum value for generation
            #[allow(dead_code)]
            pub(crate) max: $ty,
            pub(crate) dist: DebugIgnore<$dist_ty>,
        }
    };
}

macro_rules! rv_ranged_parameterized {
    ($inner: ident, $param_ty: ty, $dist_ty: ty) => {
        #[derive(Debug, Clone)]
        #[doc(hidden)]
        pub struct $inner {
            /// The distribution's parameters
            #[allow(dead_code)]
            pub(crate) params: $param_ty,
            pub(crate) dist: DebugIgnore<$dist_ty>,
        }
    };
}

macro_rules! rv_ranged_ivg {
    ($inner: ident, $ty: ty, $pq_ty: expr) => {
        impl<R> InnerValueGenerator<R> for $inner
        where
            R: Rng + Sized + Clone,
        {
            fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
                Value::from(self.dist.sample(rng) as $ty)
            }

            fn value_type(&self) -> PartiqlShape {
                $pq_ty
            }
        }
    };
}
macro_rules! rv_ranged_discrete_new {
    ($name: ident, $inner: ident, $ty: ty, $bounds_ty: ty, $min: expr, $max: expr) => {
        impl<R> $name<R>
        where
            R: Rng + Sized + Clone,
        {
            #[doc = concat!("Creates a generator that yields uniformly distributed ", stringify!($ty), " between `min` and `max` with the specified [`Density`].")]
            pub fn new<I>(min: I, max: I, rng: R, meta: Meta, density: Density) -> DataGenerationResult<Self>
            where
                I: Into<i64>,
            {
                let min: i64 = min.into();
                let max: i64 = max.into();
                if min < $min.into() || max > $max.into() {
                    Err(DataGenerationError::Bounds(min.into(), max.into()))
                } else {
                    let dist = statrs::distribution::DiscreteUniform::new(min, max)?.into();
                    let min = <$ty>::try_from(min)?;
                    let max = <$ty>::try_from(max)?;
                    let inner = $inner { min, max, dist };
                    RandomVariable::create(rng, meta, density, inner)
                }
            }
        }
    };
}
macro_rules! make_rv_ranged_discrete {
    ($name: ident, $inner: ident, $ty: ty, $pq_ty: expr, $bounds_ty: ty, $min: expr, $max: expr) => {
        rv_ranged!($inner, $ty, statrs::distribution::DiscreteUniform);
        rv_typedef!(
            #[doc = concat!("Yields discrete uniformly distributed values of type ", stringify!($ty))]
            $name,
            $inner
        );
        rv_ranged_ivg!($inner, $ty, $pq_ty);
        rv_ranged_discrete_new!($name, $inner, $ty, $bounds_ty, $min, $max);
    };
    ($name: ident, $inner: ident, $ty: ty, $pq_ty: expr, $min: expr, $max: expr) => {
        make_rv_ranged_discrete!($name, $inner, $ty, $pq_ty, $ty, $min, $max);
    };
}

// TODO fix in partiql-type
pub const TYPE_UINT8: PartiqlShape = PartiqlShape::new(Static::Int64);
pub const TYPE_UINT16: PartiqlShape = PartiqlShape::new(Static::Int64);
pub const TYPE_UINT32: PartiqlShape = PartiqlShape::new(Static::Int64);
pub const TYPE_UINT64: PartiqlShape = PartiqlShape::new(Static::Int64);

// ===== Discrete Uniform ===== //
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleUInt8, SimpleUInt8Impl,  u8,  TYPE_UINT8,       u8::MIN,  u8::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleUInt16,SimpleUInt16Impl, u16, TYPE_UINT16,      u16::MIN, u16::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleUInt32,SimpleUInt32Impl, u32, TYPE_UINT32,      u32::MIN, u32::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleUInt64,SimpleUInt64Impl, u64, TYPE_UINT64, i64, u64::MIN as i64, i64::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleInt8,  SimpleInt8Impl,   i8,  TYPE_INT8,        i8::MIN,  i8::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleInt16, SimpleInt16Impl,  i16, TYPE_INT16,       i16::MIN, i16::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleInt32, SimpleInt32Impl,  i32, TYPE_INT32,       i32::MIN, i32::MAX);
#[rustfmt::skip::macros(make_rv_ranged_discrete)]
make_rv_ranged_discrete!(SimpleInt64, SimpleInt64Impl,  i64, TYPE_INT64,       i64::MIN, i64::MAX);

macro_rules! make_rv_ranged_continuous {
    ($name: ident, $inner: ident, $dist_ty: ty, $pq_ty: expr) => {
        rv_ranged!($inner, f64, $dist_ty);
        rv_typedef!(
            #[doc = concat!("Yields ", stringify!($dist_ty), " distributed values of type f64")]
            $name,
            $inner
        );
        rv_ranged_ivg!($inner, f64, $pq_ty);
    };
}

macro_rules! rv_ranged_continuous_new {
    ($name: ident, $inner: ident, $param_ty: ty) => {
        impl<R> $name<R>
        where
            R: Rng + Sized + Clone,
        {
            pub fn new(
                params: $param_ty,
                rng: R,
                meta: Meta,
                density: Density,
            ) -> DataGenerationResult<Self> {
                let dist = params.to_dist()?.into();
                let inner = $inner { params, dist };
                RandomVariable::create(rng, meta, density, inner)
            }
        }
    };
}

macro_rules! make_rv_ranged_continuous2 {
    ($name: ident, $inner: ident, $param_ty: ty, $dist_ty: ty, $pq_ty: expr) => {
        rv_ranged_parameterized!($inner, $param_ty, $dist_ty);
        rv_typedef!(
            #[doc = concat!("Yields ", stringify!($dist_ty), " distributed values of type f64")]
            $name,
            $inner
        );
        rv_ranged_ivg!($inner, f64, $pq_ty);
        rv_ranged_continuous_new!($name, $inner, $param_ty);
    };
}

#[rustfmt::skip::macros(make_rv_ranged_continuous)]
make_rv_ranged_continuous!( SimpleF64, SimpleF64Impl,                   statrs::distribution::Uniform,      TYPE_DOUBLE );
#[rustfmt::skip::macros(make_rv_ranged_continuous2)]
make_rv_ranged_continuous2!( Normal, NormalImpl, NormalParams,          statrs::distribution::Normal,       TYPE_DOUBLE );
#[rustfmt::skip::macros(make_rv_ranged_continuous2)]
make_rv_ranged_continuous2!( LogNormal, LogNormalImpl, LogNormalParams, statrs::distribution::LogNormal,    TYPE_DOUBLE );
#[rustfmt::skip::macros(make_rv_ranged_continuous2)]
make_rv_ranged_continuous2!( Exp, ExpImpl, ExpParams,                   statrs::distribution::Exp,          TYPE_DOUBLE );
#[rustfmt::skip::macros(make_rv_ranged_continuous2)]
make_rv_ranged_continuous2!( Weibull, WeibullImpl, WeibullParams,       statrs::distribution::Weibull,      TYPE_DOUBLE );

impl<R> SimpleF64<R>
where
    R: Rng + Sized + Clone,
{
    /// Creates f64 generator that yields uniformly distributed f64s between `min` and `max`
    /// with the specified [`Density`].
    pub fn new(
        min: f64,
        max: f64,
        rng: R,
        meta: Meta,
        density: Density,
    ) -> DataGenerationResult<Self> {
        if min < f64::MIN || max > f64::MAX {
            Err(DataGenerationError::BoundsF(min, max))
        } else {
            let dist = statrs::distribution::Uniform::new(min, max)?.into();
            let inner = SimpleF64Impl { min, max, dist };
            RandomVariable::create(rng, meta, density, inner)
        }
    }
}

trait ParamsToDist<D: Continuous<f64, f64>> {
    fn to_dist(&self) -> DataGenerationResult<D>;
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct NormalParams {
    pub mean: f64,
    pub std_dev: f64,
}

impl ParamsToDist<statrs::distribution::Normal> for NormalParams {
    fn to_dist(&self) -> DataGenerationResult<statrs::distribution::Normal> {
        Ok(statrs::distribution::Normal::new(self.mean, self.std_dev)?)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ExpParams {
    pub rate: f64,
}

impl ParamsToDist<statrs::distribution::Exp> for ExpParams {
    fn to_dist(&self) -> DataGenerationResult<statrs::distribution::Exp> {
        Ok(statrs::distribution::Exp::new(self.rate)?)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct LogNormalParams {
    pub location: f64,
    pub scale: f64,
}

impl ParamsToDist<statrs::distribution::LogNormal> for LogNormalParams {
    fn to_dist(&self) -> DataGenerationResult<statrs::distribution::LogNormal> {
        Ok(statrs::distribution::LogNormal::new(
            self.location,
            self.scale,
        )?)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct WeibullParams {
    pub shape: f64,
    pub scale: f64,
}

impl ParamsToDist<statrs::distribution::Weibull> for WeibullParams {
    fn to_dist(&self) -> DataGenerationResult<statrs::distribution::Weibull> {
        Ok(statrs::distribution::Weibull::new(self.shape, self.scale)?)
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct SimpleDecimalImpl {
    #[allow(dead_code)]
    pub(crate) min: f64,
    #[allow(dead_code)]
    pub(crate) max: f64,
    pub(crate) precision: u32,
    pub(crate) scale: u32,

    pub(crate) dist: DebugIgnore<statrs::distribution::Uniform>,
}
rv_typedef!(
    /// Generates a decimal
    SimpleDecimal, SimpleDecimalImpl);
impl<R> SimpleDecimal<R>
where
    R: Rng + Sized + Clone,
{
    /// Creates decimal generator that yields uniformly distributed decimals between `min` and `max`
    /// with the specified [`Density`].
    pub fn new(
        min: f64,
        max: f64,
        rng: R,
        meta: Meta,
        density: Density,
    ) -> DataGenerationResult<Self> {
        if min < f64::MIN || max > f64::MAX {
            Err(DataGenerationError::BoundsF(min, max))
        } else {
            let p_and_s = |n| {
                let dec = rust_decimal::Decimal::from_f64(n).unwrap();
                let precision = dec
                    .mantissa()
                    .unsigned_abs()
                    .checked_ilog10()
                    .unwrap_or_default()
                    + 1;

                let scale = dec.scale();
                (precision, scale)
            };

            let (p_max_dec_precision, p_max_scale) = p_and_s(max);
            let (p_min_dec_precision, p_min_scale) = p_and_s(min);

            let precision = p_max_dec_precision.max(p_min_dec_precision);
            let scale = p_max_scale.max(p_min_scale);

            let dist = statrs::distribution::Uniform::new(min, max)?.into();

            let inner = SimpleDecimalImpl {
                min,
                max,
                precision,
                scale,
                dist,
            };
            RandomVariable::create(rng, meta, density, inner)
        }
    }
}
impl<R> InnerValueGenerator<R> for SimpleDecimalImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        let mut out_dec =
            rust_decimal::Decimal::from_f64_retain(self.dist.sample(rng)).expect("decimal value");
        out_dec.rescale(self.scale);
        Value::Decimal(Box::new(out_dec))
    }

    fn value_type(&self) -> PartiqlShape {
        PartiqlShape::new(Static::DecimalP(
            self.precision as usize,
            self.scale as usize,
        ))
    }
}
