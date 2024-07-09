use crate::gen::distributions::{Density, Meta};
use crate::gen::util::ValueTypeInference;
use crate::gen::ValueGenerator;
use crate::sim::context::SimContext;
use partiql_types::PartiqlShape;
use partiql_value::Value;

#[derive(Debug, Clone)]
pub struct ConstantGenerator {
    pub meta: Meta,
    pub constant: Value,
    pub typ: PartiqlShape,
}

impl ConstantGenerator {
    pub fn new(meta: Meta, constant: Value) -> Self {
        let typ = constant.infer_shape();
        Self {
            meta,
            constant,
            typ,
        }
    }
}

impl ValueGenerator for ConstantGenerator {
    fn present_value(&self, _ctx: &SimContext) -> Value {
        self.constant.clone()
    }

    fn value_type(&self) -> PartiqlShape {
        self.typ.clone()
    }

    fn density(&self) -> Option<Density> {
        None
    }
}
