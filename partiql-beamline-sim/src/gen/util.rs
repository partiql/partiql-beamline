use partiql_types::{
    ArrayType, BagType, PartiqlType, StructConstraint, StructField, StructType, TypeKind,
    TYPE_BOOL, TYPE_DATETIME, TYPE_DECIMAL, TYPE_INT, TYPE_MISSING, TYPE_NULL, TYPE_REAL,
    TYPE_STRING,
};
use partiql_value::Value;

pub trait ValueTypeInference {
    fn infer_type(&self) -> PartiqlType;
}

impl ValueTypeInference for Value {
    fn infer_type(&self) -> PartiqlType {
        match self {
            Value::Null => TYPE_NULL,
            Value::Missing => TYPE_MISSING,
            Value::Boolean(_) => TYPE_BOOL,
            Value::Integer(_) => TYPE_INT,
            Value::Real(_) => TYPE_REAL,
            Value::Decimal(_) => TYPE_DECIMAL,
            Value::String(_) => TYPE_STRING,
            Value::Blob(_) => PartiqlType::new(TypeKind::Undefined), // TODO BLOB
            Value::DateTime(_) => TYPE_DATETIME,
            Value::List(l) => {
                let types = l.iter().map(|v| v.infer_type());
                PartiqlType::new_array(ArrayType::new(Box::new(PartiqlType::any_of(types))))
            }
            Value::Bag(b) => {
                let types = b.iter().map(|v| v.infer_type());
                PartiqlType::new_bag(BagType::new(Box::new(PartiqlType::any_of(types))))
            }
            Value::Tuple(t) => {
                let fields = t
                    .pairs()
                    .map(|(k, v)| StructField::new(k, v.infer_type()))
                    .collect();
                PartiqlType::new_struct(StructType::new([StructConstraint::Fields(fields)].into()))
            }
        }
    }
}
