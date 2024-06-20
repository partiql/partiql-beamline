use crate::serde::{PartiqlDataSetsEncoder, PartiqlShapeEncoder, ShapeEncodeResult};
use ion_rs::element::writer::ElementWriter;
use ion_rs::element::Element;
use ion_rs::{IonType, IonWriter};
use partiql_beamline::sim::{DatasetTypeMapping, SimConfig, DATETIME_FORMAT};
use partiql_types::{AnyOf, ArrayType, BagType, PartiqlShape, StaticTypeVariant, StructType};

#[derive(Debug)]
pub struct PartiqlKolliderEncoder<'a, W, I>
where
    I: IonWriter<Output = W>,
{
    pub(crate) writer: &'a mut I,
}

impl<'a, W, I> PartiqlKolliderEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    pub fn new(writer: &'a mut I) -> Self {
        PartiqlKolliderEncoder { writer }
    }
}

impl<'a, W, I> PartiqlShapeEncoder<W, I> for PartiqlKolliderEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn writer(&mut self) -> &mut I {
        self.writer
    }
    fn write_shape(&mut self, shape: &PartiqlShape) -> ShapeEncodeResult<()> {
        match shape {
            PartiqlShape::Dynamic => self.write_typename("any"),
            PartiqlShape::AnyOf(any_of) => self.write_union(any_of),
            PartiqlShape::Static(stype) => match stype.ty() {
                StaticTypeVariant::Int => self.write_typename("int"),
                StaticTypeVariant::Int8 => self.write_typename("tinyint"),
                StaticTypeVariant::Int16 => self.write_typename("smallint"),
                StaticTypeVariant::Int32 => self.write_typename("integer"),
                StaticTypeVariant::Int64 => self.write_typename("int8"),
                StaticTypeVariant::Bool => self.write_typename("bool"),
                StaticTypeVariant::Decimal => self.write_typename("decimal"),
                StaticTypeVariant::DecimalP(p, s) => self.write_constrained_decimal(&p, &s),
                StaticTypeVariant::DateTime => self.write_typename("timestamp"),
                StaticTypeVariant::Float32 => self.write_typename("real"),
                StaticTypeVariant::Float64 => self.write_typename("double"),
                StaticTypeVariant::String => self.write_typename("string"),
                StaticTypeVariant::StringFixed(_) => todo!("handle type for {}", stype),
                StaticTypeVariant::StringVarying(_) => todo!("handle type for {}", stype),
                StaticTypeVariant::Struct(s) => self.write_struct(&s),
                StaticTypeVariant::Bag(b) => self.write_bag(&b),
                StaticTypeVariant::Array(a) => self.write_list(&a),
            },
            PartiqlShape::Undefined => todo!("handle type for {}", shape),
        }
    }
}

impl<'a, W, I> PartiqlKolliderEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    fn write_typename(&mut self, tyn: &str) -> ShapeEncodeResult<()> {
        self.writer.write_string(tyn)?;
        Ok(())
    }

    fn write_bag(&mut self, bag: &BagType) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("type");
            self.writer.write_string("bag")?;

            self.writer.set_field_name("items");
            self.write_shape(bag.element_type())?;
        }
        self.writer.step_out()?;
        Ok(())
    }

    fn write_list(&mut self, arr: &ArrayType) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("type");
            self.writer.write_string("list")?;

            self.writer.set_field_name("items");
            self.write_shape(arr.element_type())?;
        }
        self.writer.step_out()?;
        Ok(())
    }

    fn write_struct(&mut self, strct: &StructType) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("type");
            self.writer.write_string("struct")?;

            self.writer.set_field_name("constraints");
            self.writer.step_in(IonType::List)?;
            {
                self.writer.write_symbol("ordered")?;
                self.writer.write_symbol("closed")?;
            }
            self.writer.step_out()?;

            self.writer.set_field_name("fields");
            self.writer.step_in(IonType::List)?;
            for field in &strct.fields() {
                self.writer.step_in(IonType::Struct)?;
                {
                    self.writer.set_field_name("name");
                    self.writer.write_string(field.name())?;

                    self.writer.set_field_name("type");
                    self.write_shape(field.ty())?;
                }
                self.writer.step_out()?;
            }
            self.writer.step_out()?;
        }
        self.writer.step_out()?;
        Ok(())
    }

    fn write_constrained_decimal(&mut self, p: &usize, s: &usize) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("name");
            self.writer.write_string("decimal")?;
            self.writer.set_field_name("precision");
            self.writer.write_i64(*p as i64)?;
            self.writer.set_field_name("scale");
            self.writer.write_i64(*s as i64)?;
        }
        self.writer.step_out()?;
        Ok(())
    }

    fn write_union(&mut self, any_of: &AnyOf) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("name");
            self.writer.write_string("union")?;
            self.writer.set_field_name("types");
            self.writer.step_in(IonType::List)?;
            let types = any_of.types();
            for t in types.into_iter() {
                match self.write_shape(t) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            self.writer.step_out()?;
        }
        self.writer.step_out()?;
        Ok(())
    }
}

impl<'a, W, I> PartiqlDataSetsEncoder<W, I> for PartiqlKolliderEncoder<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    type Output = ();

    fn writer(&mut self) -> &mut I {
        self.writer
    }

    fn write_datasets(
        &mut self,
        cfg: &SimConfig,
        shapes: DatasetTypeMapping,
    ) -> ShapeEncodeResult<()> {
        self.writer.step_in(IonType::Struct)?;
        {
            self.writer.set_field_name("seed");
            self.writer.write_i64(cfg.seed as i64)?;

            self.writer.set_field_name("start");
            self.writer
                .write_element(&Element::read_one(cfg.t0.format(&DATETIME_FORMAT)?)?)?;

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
