use crate::gen::distributions::{Density, InnerValueGenerator, RandomVariable};
use crate::gen::{DataGenerationResult, ValueGenerator};
use crate::sim::context::SimContext;
use partiql_types::{PartiqlShape, StructConstraint, StructField, StructType};
use partiql_value::{Tuple, Value};
use rand::Rng;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub enum SimpleRandomDataImpl {
    Single(Box<dyn ValueGenerator>),
    Collection(HashMap<String, Box<dyn ValueGenerator>>),
}

impl Debug for SimpleRandomDataImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleRandomDataImpl::Single(s) => Debug::fmt(s, f),
            SimpleRandomDataImpl::Collection(data) => {
                let mut map = f.debug_map();
                for (k, v) in data {
                    map.entry(k, v);
                }
                map.finish()
            }
        }
    }
}

impl From<Box<dyn ValueGenerator>> for SimpleRandomDataImpl {
    fn from(value: Box<dyn ValueGenerator>) -> Self {
        Self::Single(value)
    }
}

impl<S, V> From<HashMap<S, V>> for SimpleRandomDataImpl
where
    S: Into<String>,
    V: Into<Box<dyn ValueGenerator>>,
{
    #[inline]
    fn from(kvs: HashMap<S, V>) -> Self {
        let map = kvs.into_iter().map(|(s, v)| (s.into(), v.into())).collect();
        SimpleRandomDataImpl::Collection(map)
    }
}

impl<const N: usize, S, V> From<[(S, V); N]> for SimpleRandomDataImpl
where
    S: Into<String>,
    V: Into<Box<dyn ValueGenerator>>,
{
    #[inline]
    fn from(arr: [(S, V); N]) -> Self {
        let map = arr.into_iter().map(|(s, v)| (s.into(), v.into())).collect();
        SimpleRandomDataImpl::Collection(map)
    }
}

pub type SimpleRandomData<R> = RandomVariable<R, SimpleRandomDataImpl>;

impl<R> SimpleRandomData<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new<I>(rng: R, density: Density, inner: I) -> DataGenerationResult<Self>
    where
        I: Into<SimpleRandomDataImpl>,
    {
        let inner = inner.into();
        RandomVariable::create(rng, density, inner)
    }
}

impl<R> InnerValueGenerator<R> for SimpleRandomDataImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, _rng: &mut R, ctx: &SimContext) -> Value {
        match self {
            SimpleRandomDataImpl::Single(rv) => rv.gen_value(ctx),
            SimpleRandomDataImpl::Collection(kvs) => {
                let it = kvs.iter().filter_map(|(k, v)| match v.gen_value(ctx) {
                    Value::Missing => None,
                    v => Some((k, v)),
                });
                Tuple::from_iter(it).into()
            }
        }
    }

    fn value_type(&self) -> PartiqlShape {
        match self {
            SimpleRandomDataImpl::Single(rv) => rv.value_type(),
            SimpleRandomDataImpl::Collection(kvs) => {
                let fields = kvs
                    .iter()
                    .map(|(k, v)| {
                        if let Some(d) = v.density() {
                            if let Some(_) = d.optionality() {
                                StructField::new_optional(k, v.value_type())
                            } else {
                                StructField::new(k, v.value_type())
                            }
                        } else {
                            StructField::new(k, v.value_type())
                        }
                    })
                    .collect();
                PartiqlShape::new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
