use partiql_types::{type_bool, type_datetime, type_decimal, type_float64, type_int, type_string, type_array, type_bag, BagType, PartiqlShape, PartiqlShapeBuilder, StructConstraint, StructField, StructType, PartiqlNoIdShapeBuilder};
use partiql_value::Value;

pub trait ValueTypeInference {
    fn infer_shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape;
}

impl ValueTypeInference for Value {
    fn infer_shape(&self, bld: &mut PartiqlNoIdShapeBuilder) -> PartiqlShape {
        match self {
            Value::Null => PartiqlShape::Undefined,
            Value::Missing => PartiqlShape::Undefined,
            Value::Boolean(_) => type_bool!(bld),
            Value::Integer(_) => type_int!(bld),
            Value::Real(_) => type_float64!(bld),
            Value::Decimal(_) => type_decimal!(bld),
            Value::String(_) => type_string!(bld),
            Value::Blob(_) => PartiqlShape::Undefined, // TODO BLOB
            Value::DateTime(_) => type_datetime!(bld),
            Value::List(l) => {
                let types: Vec<PartiqlShape> = l.iter().map(|v| v.infer_shape(bld)).collect();
                type_array!(bld, bld.any_of(types))
            }
            Value::Bag(b) => {
                let types: Vec<PartiqlShape> = b.iter().map(|v| v.infer_shape(bld)).collect();
                type_bag!(bld, bld.any_of(types))
            }
            Value::Tuple(t) => {
                let fields = t
                    .pairs()
                    .map(|(k, v)| StructField::new(k, v.infer_shape(bld)))
                    .collect();
                bld.new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
