use crate::serde::{ShapeEncodeResult, ShapeEncodingError};
use partiql_types::{AnyOf, ArrayType, BagType, PartiqlType, StructType, TypeKind};
use std::fmt::{Display, Formatter};
use std::string::ToString;

const PARTIQL_DATA_TYPE_SYNTAX: &str = "partiql_datatype_syntax";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DdlFormat {
    Compact,
    Pretty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlSyntax {
    name: String,
    version: DdlSyntaxVersion,
}

impl Display for DdlSyntax {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}.{}",
            self.name, self.version.major, self.version.minor
        )
    }
}

impl DdlSyntax {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn version(&self) -> String {
        format!("{}.{}", self.version.major, self.version.minor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdlSyntaxVersion {
    major: u8,
    minor: u8,
}

pub trait PartiqlDdlEncoder {
    type Output;

    fn ddl(&self, ty: &PartiqlType) -> ShapeEncodeResult<Self::Output>;

    fn syntax(&self) -> DdlSyntax;
}

#[derive(Debug, Clone)]
pub struct PartiqlBasicDdlEncoder {
    format: DdlFormat,
    syntax: DdlSyntax,
}

impl PartiqlBasicDdlEncoder {
    pub fn new(format: DdlFormat) -> Self {
        PartiqlBasicDdlEncoder {
            format,
            syntax: DdlSyntax {
                name: PARTIQL_DATA_TYPE_SYNTAX.to_string(),
                version: DdlSyntaxVersion { major: 0, minor: 1 },
            },
        }
    }

    fn write_shape(&self, shape: &PartiqlType) -> ShapeEncodeResult<String> {
        Ok(match shape.kind() {
            TypeKind::Any => "ANY".to_string(),
            TypeKind::AnyOf(any_of) => self.write_union(any_of)?,
            TypeKind::Int => "INT".to_string(),
            TypeKind::Int8 => "TINYINT".to_string(),
            TypeKind::Int16 => "SMALLINT".to_string(),
            TypeKind::Int32 => "INTEGER".to_string(),
            TypeKind::Int64 => "INT8".to_string(),
            TypeKind::Bool => "BOOL".to_string(),
            TypeKind::Decimal => "DECIMAL".to_string(),
            TypeKind::DecimalP(p, s) => format!("DECIMAL({p}, {s})"),
            TypeKind::DateTime => "TIMESTAMP".to_string(),
            TypeKind::Float32 => "REAL".to_string(),
            TypeKind::Float64 => "DOUBLE".to_string(),
            TypeKind::String => "VARCHAR".to_string(),
            TypeKind::Struct(s) => self.write_struct(s)?,
            TypeKind::Bag(b) => self.write_bag(b)?,
            TypeKind::Array(a) => self.write_array(a)?,

            // non-exhaustive catch-all
            _ => todo!("handle type for {}", shape.kind()),
        })
    }

    fn write_bag(&self, bag: &BagType) -> ShapeEncodeResult<String> {
        Ok(format!("BAG<{}>", self.write_shape(bag.element_type())?))
    }

    fn write_array(&self, arr: &ArrayType) -> ShapeEncodeResult<String> {
        Ok(format!("ARRAY<{}>", self.write_shape(arr.element_type())?))
    }

    fn write_struct(&self, strct: &StructType) -> ShapeEncodeResult<String> {
        let mut struct_out = String::from("STRUCT<");

        let fields = strct.fields();
        let mut fields = fields.iter().peekable();
        while let Some(field) = fields.next() {
            struct_out.push_str(&format!("\"{}\": ", field.name()));
            struct_out.push_str(&self.write_shape(field.ty())?);
            if fields.peek().is_some() {
                struct_out.push_str(",");
            }
        }

        struct_out.push_str(">");
        Ok(struct_out)
    }

    fn write_union(&self, any_of: &AnyOf) -> ShapeEncodeResult<String> {
        let mut union_out = String::from("UNION<");
        let mut types = any_of.types().peekable();
        while let Some(ty) = types.next() {
            union_out.push_str(&self.write_shape(ty)?);
            if types.peek().is_some() {
                union_out.push_str(",");
            }
        }
        union_out.push_str(">");
        Ok(union_out)
    }

    fn write_line(&self) -> ShapeEncodeResult<String> {
        Ok(if self.format == DdlFormat::Pretty {
            "\n".to_string()
        } else {
            "".to_string()
        })
    }
}

impl PartiqlDdlEncoder for PartiqlBasicDdlEncoder {
    type Output = String;

    fn ddl(&self, ty: &PartiqlType) -> ShapeEncodeResult<String> {
        let mut output = String::new();

        if let TypeKind::Bag(bag) = ty.kind() {
            if let TypeKind::Struct(s) = bag.element_type().kind() {
                let fields = s.fields();
                let mut fields = fields.iter().peekable();
                while let Some(field) = fields.next() {
                    output.push_str(&format!("\"{}\" ", field.name()));
                    output.push_str(&self.write_shape(field.ty())?);
                    if fields.peek().is_some() {
                        output.push_str(",");
                        output.push_str(&self.write_line()?);
                    }
                }
                Ok(output)
            } else {
                Err(ShapeEncodingError::UnsupportedEncoding(format!(
                    "Unsupported top level element type {:?}",
                    bag.element_type()
                )))
            }
        } else {
            Err(ShapeEncodingError::UnsupportedEncoding(format!(
                "Unsupported top level type {:?}",
                ty.kind()
            )))
        }
    }

    fn syntax(&self) -> DdlSyntax {
        self.syntax.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_types::{array, bag, f64, int8, r#struct, str, struct_fields, StructConstraint};
    use std::collections::BTreeSet;

    #[test]
    fn ddl_test() {
        let nested_attrs = struct_fields![
            (
                "a",
                PartiqlType::any_of(vec![
                    PartiqlType::new(TypeKind::DecimalP(5, 4)),
                    PartiqlType::new(TypeKind::Int8),
                ])
            ),
            ("b", array![str![]]),
            ("c", f64!()),
        ];
        let details = r#struct![BTreeSet::from([nested_attrs])];

        let fields = struct_fields![
            ("employee_id", int8![]),
            ("full_name", str![]),
            ("salary", PartiqlType::new(TypeKind::DecimalP(8, 2))),
            ("details", details),
            ("dependents", array![str![]])
        ];
        let ty = bag![r#struct![BTreeSet::from([
            fields,
            StructConstraint::Open(false)
        ])]];

        let expected_compact = r#""dependents" ARRAY<VARCHAR>,"details" STRUCT<"a": UNION<TINYINT,DECIMAL(5, 4)>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,"employee_id" TINYINT,"full_name" VARCHAR,"salary" DECIMAL(8, 2)"#;
        let expected_pretty = r#""dependents" ARRAY<VARCHAR>,
"details" STRUCT<"a": UNION<TINYINT,DECIMAL(5, 4)>,"b": ARRAY<VARCHAR>,"c": DOUBLE>,
"employee_id" TINYINT,
"full_name" VARCHAR,
"salary" DECIMAL(8, 2)"#;

        let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
        assert_eq!(ddl_compact.ddl(&ty).expect("write shape"), expected_compact);

        let ddl_pretty = PartiqlBasicDdlEncoder::new(DdlFormat::Pretty);
        assert_eq!(ddl_pretty.ddl(&ty).expect("write shape"), expected_pretty);
    }
}
