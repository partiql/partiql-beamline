use crate::gen::util::ValueTypeInference;
use crate::gen::ValueGenerator;
use crate::sim::context::SimContext;
use partiql_types::PartiqlType;
use partiql_value::Value;

#[derive(Debug, Clone)]
pub struct ConstantGenerator {
    pub constant: Value,
    pub typ: PartiqlType,
}

impl ConstantGenerator {
    pub fn new(constant: Value) -> Self {
        let typ = constant.infer_type();
        Self { typ, constant }
    }
}

impl ValueGenerator for ConstantGenerator {
    fn present_value(&self, ctx: &SimContext) -> Value {
        self.constant.clone()
    }

    fn value_type(&self) -> PartiqlType {
        self.typ.clone()
    }
}
