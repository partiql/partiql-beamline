use ion_rs::element::writer::{ElementWriter, TextKind};
use ion_rs::element::Element;
use ion_rs::{ion_struct, IonError, IonType, IonWriter};
use miette::Diagnostic;
use partiql_beamline::sim::{DatasetTypeMapping, SimConfig, DATETIME_FORMAT};
use partiql_types::{PartiqlType, TypeKind};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("ShapeEncodingError Error")]
#[non_exhaustive]
pub enum ShapeEncodingError {
    UnsupportedEncoding,
    #[error("IonEncodingError: {0}")]
    IonEncodingError(#[from] IonError),
    #[error("DateTimeEncodingError e: {0}")]
    DateTimeEncodingError(#[from] time::error::Format),
}

pub trait PartiqlShapeEncoding {
    fn print(
        &self,
        cfg: &SimConfig,
        shapes: DatasetTypeMapping,
    ) -> Result<String, ShapeEncodingError>;
}

#[derive(Default, Debug)]
pub struct PartiqlKolliderEncoding {}

impl PartiqlShapeEncoding for PartiqlKolliderEncoding {
    fn print(
        &self,
        cfg: &SimConfig,
        shapes: DatasetTypeMapping,
    ) -> Result<String, ShapeEncodingError> {
        let mut buff = vec![];
        let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
            .build(&mut buff)
            .expect("pretty writer");

        writer.step_in(IonType::Struct)?;
        writer.set_field_name("seed");
        writer.write_element(&(cfg.seed as i64).into())?;
        writer.set_field_name("start");
        writer.write_element(&Element::read_one(cfg.t0.format(&DATETIME_FORMAT)?)?)?;
        writer.set_field_name("shapes");
        writer.step_in(IonType::Struct)?;
        for (dataset, ty) in shapes.into_iter() {
            writer.set_field_name(dataset);
            writer.set_annotations(vec!["partiql", "shape", "v0"]);
            writer.step_in(IonType::Struct)?;
            if let TypeKind::Bag(bag) = ty.kind() {
                writer.set_field_name("type");
                writer.write_element(&Element::read_one("\"bag\"")?)?;
                writer.set_field_name("items");
                writer.step_in(IonType::Struct)?;
                writer.set_field_name("type");
                writer.write_element(&Element::read_one("\"struct\"")?)?;
                writer.set_field_name("constraints");
                writer.step_in(IonType::List)?;
                writer.write_element(&Element::read_one("ordered")?)?;
                writer.write_element(&Element::read_one("closed")?)?;
                writer.step_out()?;
                writer.set_field_name("fields");
                writer.step_in(IonType::List)?;

                for elem in get_shape_field_names(bag.element_type()) {
                    writer.write_element(&elem)?;
                }
                writer.step_out()?;
                writer.step_out()?;
            }
            writer.step_out()?;
        }
        writer.step_out()?;

        writer.step_out()?;
        drop(writer);
        Ok(String::from_utf8(buff).expect("from ut8"))
    }
}

fn get_shape_field_names(ty: &PartiqlType) -> Vec<Element> {
    if let TypeKind::Struct(struct_type) = ty.kind() {
        let out: Vec<Element> = struct_type
            .fields()
            .into_iter()
            .map(|field| {
                let field_name = field.name();
                let field_type = match field.ty().kind() {
                    TypeKind::Any => "any",
                    TypeKind::Null => "null",
                    TypeKind::Int => "int",
                    TypeKind::Int8 => "tinyint",
                    TypeKind::Int16 => "smallint",
                    TypeKind::Int32 => "integer",
                    TypeKind::Int64 => "int8",
                    TypeKind::Bool => "bool",
                    TypeKind::Decimal => "decimal",
                    TypeKind::DateTime => "datetime",
                    TypeKind::Float32 => "real",
                    TypeKind::Float64 => "double",
                    TypeKind::String => "string",
                    TypeKind::Undefined => "undefined",
                    _ => todo!("unsupported shape element type"),
                };
                let row = ion_struct! {
                    "name": field_name,
                    "type": field_type
                };
                row.into()
            })
            .collect();
        out
    } else {
        todo!("unsupported shape")
    }
}
