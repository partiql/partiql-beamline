use crate::gen::ValueGenerator;
use crate::sim::context::SimContext;
use partiql_types::{PartiqlType, StructConstraint, StructField, StructType};
use partiql_value::{Tuple, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

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

    fn value_type(&self) -> PartiqlType {
        match self {
            SimpleRandomData::Single(rv) => rv.value_type(),
            SimpleRandomData::Collection(kvs) => {
                let fields = kvs
                    .iter()
                    .map(|(k, v)| StructField::new(k, v.value_type()))
                    .collect();
                PartiqlType::new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
