use partiql_types::{
    ArrayType, BagType, PartiqlShape, StructConstraint, StructField, StructType, TYPE_BOOL,
    TYPE_DATETIME, TYPE_DECIMAL, TYPE_INT, TYPE_REAL, TYPE_STRING,
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
            Value::Boolean(_) => TYPE_BOOL,
            Value::Integer(_) => TYPE_INT,
            Value::Real(_) => TYPE_REAL,
            Value::Decimal(_) => TYPE_DECIMAL,
            Value::String(_) => TYPE_STRING,
            Value::Blob(_) => PartiqlShape::Undefined, // TODO BLOB
            Value::DateTime(_) => TYPE_DATETIME,
            Value::List(l) => {
                let types = l.iter().map(|v| v.infer_shape());
                PartiqlShape::new_array(ArrayType::new(Box::new(PartiqlShape::any_of(types))))
            }
            Value::Bag(b) => {
                let types = b.iter().map(|v| v.infer_shape());
                PartiqlShape::new_bag(BagType::new(Box::new(PartiqlShape::any_of(types))))
            }
            Value::Tuple(t) => {
                let fields = t
                    .pairs()
                    .map(|(k, v)| StructField::new(k, v.infer_shape()))
                    .collect();
                PartiqlShape::new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
