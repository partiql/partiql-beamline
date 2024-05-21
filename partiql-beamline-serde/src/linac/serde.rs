use ion_rs::{element::Element, IonType, IonWriter};
use partiql_beamline::sim::{DatasetTypeMapping, SimConfig, DATETIME_FORMAT};

use super::{runtime::{Arena, RidlError}, shape::*};
use crate::serde::{PartiqlDataSetsEncoder, PartiqlShapeEncoder, ShapeEncodeResult, ShapeEncodingError};
use partiql_types::{AnyOf, ArrayType, BagType, StructType, TypeKind};

pub struct LinacShapeEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    writer: &'a mut I,
}

impl<'a, W, I> LinacShapeEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    pub fn new(writer: &'a mut I) -> Self {
        LinacShapeEncoder { writer }
    }
}

impl<'a, W, I> PartiqlShapeEncoder<W, I> for LinacShapeEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn writer(&mut self) -> &mut I {
        self.writer
    }

    fn write_shape(&mut self, shape: &partiql_types::PartiqlType) -> ShapeEncodeResult {
        let arena = Arena::new();
        let model = self.to_shape(&arena, shape);
        let mut writer = LinacWriterBuilder::text(self.writer);
        writer.write_shape(&model).map_err(|e| {
            match e {
                RidlError::Ion(e) => ShapeEncodingError::IonEncodingError(e),
                RidlError::ReadError(_) => ShapeEncodingError::UnsupportedEncoding,
                RidlError::WriteError(_) => ShapeEncodingError::UnsupportedEncoding,
            }
        })
    }
}

impl<'a, W, I> LinacShapeEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn to_shape(&self, arena: &'a Arena, shape: &partiql_types::PartiqlType) -> Shape<'a> {
        match shape.kind() {
            // Any
            TypeKind::Any => Shape::TDynamic(()),

            // Unknown
            TypeKind::Null | TypeKind::Missing | TypeKind::Undefined => Shape::TUnknown(()),

            // Logical
            TypeKind::Bool => Shape::TBool(()),

            // Exact Numeric
            TypeKind::Int => Shape::TInt(()),
            TypeKind::Int8 => Shape::TInt8(()),
            TypeKind::Int16 => Shape::TInt16(()),
            TypeKind::Int32 => Shape::TInt32(()),
            TypeKind::Int64 => Shape::TInt64(()),
            TypeKind::Decimal => Shape::TDecimal(()),
            TypeKind::DecimalP(p, s) => Shape::TNumeric(ShapeTNumeric {
                precision: *p as i64,
                scale: *s as i64,
            }),

            // Approximate Numeric
            TypeKind::Float32 => Shape::TFloat32(()),
            TypeKind::Float64 => Shape::TFloat64(()),

            // Strings
            TypeKind::String => Shape::TString(()),
            TypeKind::StringFixed(l) => {
                Shape::TStringFixed(ShapeTStringFixed { length: *l as i64 })
            }
            TypeKind::StringVarying(l) => {
                Shape::TStringVarying(ShapeTStringVarying { length: *l as i64 })
            }

            // Datetime
            TypeKind::DateTime => Shape::TTimestamp(ShapeTTimestamp { precision: 0 }),

            // Containers
            TypeKind::Bag(t) => Shape::TBag(self.to_bag(arena, t)),
            TypeKind::Array(t) => Shape::TArray(self.to_array(arena, t)),
            TypeKind::Struct(t) => Shape::TStruct(self.to_struct(arena, t)),

            // Union
            TypeKind::AnyOf(t) => Shape::TUnion(self.to_any_of(arena, t)),

            _ => todo!("handle type for {}", shape.kind()),
        }
    }

    fn to_any_of(&self, arena: &'a Arena, t: &AnyOf) -> ShapeTUnion<'a> {
        let variants: Vec<Shape> = t.types().map(|v| self.to_shape(arena, v)).collect();
        ShapeTUnion { variants }
    }

    fn to_bag(&self, arena: &'a Arena, t: &BagType) -> ShapeTBag<'a> {
        let items = self.to_shape(arena, t.element_type());
        ShapeTBag {
            items: arena.alloc(items),
        }
    }

    fn to_array(&self, arena: &'a Arena, t: &ArrayType) -> ShapeTArray<'a> {
        let items = self.to_shape(arena, t.element_type());
        ShapeTArray {
            items: arena.alloc(items),
        }
    }

    fn to_struct(&self, arena: &'a Arena, t: &StructType) -> ShapeTStruct<'a> {
        let fields: Vec<Field> = t
            .fields()
            .iter()
            .map(|f| {
                let name = f.name().to_string();
                let shape = self.to_shape(arena, f.ty());
                Field {
                    name,
                    shape: arena.alloc(shape),
                }
            })
            .collect();
        ShapeTStruct { fields }
    }
}

impl<'a, W, I> PartiqlDataSetsEncoder<W, I> for LinacShapeEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn writer(&mut self) -> &mut I {
        self.writer
    }

    fn write_datasets(&mut self, cfg: &SimConfig, shapes: DatasetTypeMapping) -> ShapeEncodeResult {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("seed");
            self.writer.write_i64(cfg.seed as i64)?;

            // self.writer.set_field_name("start");
            // self.writer
            //     .write_element(&Element::read_one(cfg.t0.format(&DATETIME_FORMAT)?)?)?;

            self.writer.set_field_name("shapes");
            self.writer.step_in(IonType::Struct)?;
            for (dataset, ty) in shapes.into_iter() {
                self.writer.set_field_name(dataset);
                self.writer.set_annotations(vec!["partiql", "shape", "v0"]);
                self.write_shape(&ty)?;
            }
            self.writer.step_out()?;
        }
        self.writer.step_out()?;
        Ok(())
    }
}
