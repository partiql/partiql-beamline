use partiql_types::{
    type_bool, type_datetime, type_decimal, type_float64, type_int, type_string, ArrayType,
    BagType, PartiqlShape, PartiqlShapeBuilder, StructConstraint, StructField, StructType,
};
use partiql_value::Value;

pub trait ValueTypeInference {
    fn infer_shape(&self) -> PartiqlShape;
}

impl ValueTypeInference for Value {
    fn infer_shape(&self) -> PartiqlShape {
        match self {
            Value::Null => PartiqlShape::Undefined,
            Value::Missing => PartiqlShape::Undefined,
            Value::Boolean(_) => type_bool!(),
            Value::Integer(_) => type_int!(),
            Value::Real(_) => type_float64!(),
            Value::Decimal(_) => type_decimal!(),
            Value::String(_) => type_string!(),
            Value::Blob(_) => PartiqlShape::Undefined, // TODO BLOB
            Value::DateTime(_) => type_datetime!(),
            Value::List(l) => {
                let types = l.iter().map(|v| v.infer_shape());
                PartiqlShapeBuilder::init_or_get().new_array(ArrayType::new(Box::new(
                    PartiqlShapeBuilder::init_or_get().any_of(types),
                )))
            }
            Value::Bag(b) => {
                let types = b.iter().map(|v| v.infer_shape());
                PartiqlShapeBuilder::init_or_get().new_bag(BagType::new(Box::new(
                    PartiqlShapeBuilder::init_or_get().any_of(types),
                )))
            }
            Value::Tuple(t) => {
                let fields = t
                    .pairs()
                    .map(|(k, v)| StructField::new(k, v.infer_shape()))
                    .collect();
                PartiqlShapeBuilder::init_or_get()
                    .new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
