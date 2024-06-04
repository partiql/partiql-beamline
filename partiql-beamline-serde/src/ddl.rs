use crate::serde::{PartiqlDataSetsEncoder, PartiqlShapeEncoder, ShapeEncodeResult};
use partiql_types::{AnyOf, ArrayType, BagType, PartiqlType, StructType, TypeKind};

const STRUCT: &str = "STRUCT";
const ARRAY: &str = "ARRAY";

const BAG: &str = "BAG";

#[derive(Debug)]
pub struct PartiqlDdlEncoder {
    pub(crate) output: String,
}

impl<'a> PartiqlDdlEncoder {
    pub fn new() -> Self {
        PartiqlDdlEncoder {
            output: String::from(""),
        }
    }

    pub fn output(&'a self) -> &'a str {
        &self.output
    }
    pub fn write_shape(&mut self, shape: &PartiqlType, top_level: bool) -> ShapeEncodeResult {
        match shape.kind() {
            TypeKind::Any => Ok(self.output.push_str("ANY")),
            TypeKind::AnyOf(any_of) => self.write_union(any_of),
            TypeKind::Int => Ok(self.output.push_str("INT")),
            TypeKind::Int8 => Ok(self.output.push_str("TINYINT")),
            TypeKind::Int16 => Ok(self.output.push_str("SMALLINT")),
            TypeKind::Int32 => Ok(self.output.push_str("INTEGER")),
            TypeKind::Int64 => Ok(self.output.push_str("INT8")),
            TypeKind::Bool => Ok(self.output.push_str("BOOL")),
            TypeKind::Decimal => Ok(self.output.push_str("DECIMAL")),
            TypeKind::DecimalP(p, s) => Ok(self.output.push_str(&format!("DECIMAL({p}, {s})"))),
            TypeKind::DateTime => Ok(self.output.push_str("TIMESTAMP")),
            TypeKind::Float32 => Ok(self.output.push_str("REAL")),
            TypeKind::Float64 => Ok(self.output.push_str("DOUBLE")),
            TypeKind::String => Ok(self.output.push_str("STRING")),
            TypeKind::Struct(s) => self.write_struct(s, top_level),
            TypeKind::Bag(b) => self.write_bag(b, top_level),
            TypeKind::Array(a) => self.write_array(a, top_level),

            // non-exhaustive catch-all
            _ => todo!("handle type for {}", shape.kind()),
        }
    }

    fn write_typename(&mut self, tyn: &str) -> ShapeEncodeResult {
        // self.writer.write_symbol(tyn)?;
        Ok(())
    }

    fn write_bag(&mut self, bag: &BagType, top_level: bool) -> ShapeEncodeResult {
        if top_level {
            self.write_shape(bag.element_type(), top_level)?;
        } else {
            self.output.push_str("BAG<");
            self.write_shape(bag.element_type(), false)?;
            self.output.push_str(">");
        }
        Ok(())
    }

    fn write_array(&mut self, arr: &ArrayType, top_level: bool) -> ShapeEncodeResult {
        if top_level {
            {}
        } else {
            self.output.push_str("ARRAY<");
            self.write_shape(arr.element_type(), false)?;
            self.output.push_str(">");
        }
        Ok(())
    }

    fn write_struct(&mut self, strct: &StructType, top_level: bool) -> ShapeEncodeResult {
        if top_level {
            for field in &strct.fields() {
                self.output.push_str(field.name());
                self.output.push_str(":");
                self.write_shape(field.ty(), false)?;
                self.output.push_str(",\n");
            }
        } else {
            self.output.push_str("STRUCT<");
            for field in &strct.fields() {
                self.output.push_str(field.name());
                self.output.push_str(":");
                self.write_shape(field.ty(), false)?;
                self.output.push_str(",\n");
            }
            self.output.push_str(">");
        }
        Ok(())
    }

    fn write_constrained_decimal(&mut self, p: &usize, s: &usize) -> ShapeEncodeResult {
        // self.writer.write_string(format!("DECIMAL({p}, {s})"))?;
        Ok(())
    }

    fn write_union(&mut self, any_of: &AnyOf) -> ShapeEncodeResult {
        todo!("Union support for PartiQL DDL")
    }
}
