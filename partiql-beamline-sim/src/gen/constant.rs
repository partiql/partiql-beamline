use crate::gen::distributions::{Density, Meta};
use crate::gen::util::ValueTypeInference;
use crate::gen::ValueGenerator;
use crate::sim::SimContext;
use partiql_types::{PartiqlNoIdShapeBuilder, PartiqlShape, PartiqlShapeBuilder};
use partiql_value::Value;

#[derive(Debug, Clone)]
pub struct ConstantGenerator {
    pub meta: Meta,
    pub constant: Value,
}

impl ConstantGenerator {
    pub fn new(meta: Meta, constant: Value) -> Self {
        Self { meta, constant }
    }
}

impl ValueGenerator for ConstantGenerator {
    fn present_value(&self, _ctx: &SimContext) -> Value {
        self.constant.clone()
    }

    fn shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        self.constant.infer_shape(bld)
    }

    fn density(&self) -> Option<Density> {
        None
    }
}
