#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use super::runtime::*;
use derive_new::new;
use ion_rs::data_source::ToIonDataSource;
use ion_rs::element::*;
use ion_rs::*;
use std::fmt;
use std::io::Write;
use std::process::Output;

pub type ShapeTDynamic = ();
pub type ShapeTUnknown = ();
pub type ShapeTBool = ();
pub type ShapeTInt8 = ();
pub type ShapeTInt16 = ();
pub type ShapeTInt32 = ();
pub type ShapeTInt64 = ();
pub type ShapeTInt = ();
pub type ShapeTDecimal = ();
#[derive(new, Debug, PartialEq)]
pub struct ShapeTNumeric {
    pub precision: i64,
    pub scale: i64,
}
pub type ShapeTFloat32 = ();
pub type ShapeTFloat64 = ();
#[derive(new, Debug, PartialEq)]
pub struct ShapeTFloat {
    pub precision: i64,
}
pub type ShapeTString = ();
#[derive(new, Debug, PartialEq)]
pub struct ShapeTStringFixed {
    pub length: i64,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTStringVarying {
    pub length: i64,
}
pub type ShapeTBlob = ();
pub type ShapeTClob = ();
pub type ShapeTDate = ();
#[derive(new, Debug, PartialEq)]
pub struct ShapeTTime {
    pub precision: i64,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTTimeTz {
    pub precision: i64,
    pub offsetHour: i64,
    pub offsetMinute: i64,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTTimestamp {
    pub precision: i64,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTTimestampTz {
    pub precision: i64,
    pub offsetHour: i64,
    pub offsetMinute: i64,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTBag<'a> {
    pub items: &'a Shape<'a>,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTArray<'a> {
    pub items: &'a Shape<'a>,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTStruct<'a> {
    pub fields: Vec<Field<'a>>,
}
#[derive(new, Debug, PartialEq)]
pub struct ShapeTUnion<'a> {
    pub variants: Vec<Shape<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Shape<'a> {
    TDynamic(ShapeTDynamic),
    TUnknown(ShapeTUnknown),
    TBool(ShapeTBool),
    TInt8(ShapeTInt8),
    TInt16(ShapeTInt16),
    TInt32(ShapeTInt32),
    TInt64(ShapeTInt64),
    TInt(ShapeTInt),
    TDecimal(ShapeTDecimal),
    TNumeric(ShapeTNumeric),
    TFloat32(ShapeTFloat32),
    TFloat64(ShapeTFloat64),
    TFloat(ShapeTFloat),
    TString(ShapeTString),
    TStringFixed(ShapeTStringFixed),
    TStringVarying(ShapeTStringVarying),
    TBlob(ShapeTBlob),
    TClob(ShapeTClob),
    TDate(ShapeTDate),
    TTime(ShapeTTime),
    TTimeTz(ShapeTTimeTz),
    TTimestamp(ShapeTTimestamp),
    TTimestampTz(ShapeTTimestampTz),
    TBag(ShapeTBag<'a>),
    TArray(ShapeTArray<'a>),
    TStruct(ShapeTStruct<'a>),
    TUnion(ShapeTUnion<'a>),
}
#[derive(new, Debug, PartialEq)]
pub struct Field<'a> {
    pub name: String,
    pub shape: &'a Shape<'a>,
}

pub trait LinacWriter {
    fn write_shape<'a>(&mut self, value: &'a Shape<'a>) -> RidlResult<()>;
    fn write_shape_t_dynamic<'a>(&mut self, value: &'a ShapeTDynamic) -> RidlResult<()>;
    fn write_shape_t_unknown<'a>(&mut self, value: &'a ShapeTUnknown) -> RidlResult<()>;
    fn write_shape_t_bool<'a>(&mut self, value: &'a ShapeTBool) -> RidlResult<()>;
    fn write_shape_t_int8<'a>(&mut self, value: &'a ShapeTInt8) -> RidlResult<()>;
    fn write_shape_t_int16<'a>(&mut self, value: &'a ShapeTInt16) -> RidlResult<()>;
    fn write_shape_t_int32<'a>(&mut self, value: &'a ShapeTInt32) -> RidlResult<()>;
    fn write_shape_t_int64<'a>(&mut self, value: &'a ShapeTInt64) -> RidlResult<()>;
    fn write_shape_t_int<'a>(&mut self, value: &'a ShapeTInt) -> RidlResult<()>;
    fn write_shape_t_decimal<'a>(&mut self, value: &'a ShapeTDecimal) -> RidlResult<()>;
    fn write_shape_t_numeric<'a>(&mut self, value: &'a ShapeTNumeric) -> RidlResult<()>;
    fn write_shape_t_float32<'a>(&mut self, value: &'a ShapeTFloat32) -> RidlResult<()>;
    fn write_shape_t_float64<'a>(&mut self, value: &'a ShapeTFloat64) -> RidlResult<()>;
    fn write_shape_t_float<'a>(&mut self, value: &'a ShapeTFloat) -> RidlResult<()>;
    fn write_shape_t_string<'a>(&mut self, value: &'a ShapeTString) -> RidlResult<()>;
    fn write_shape_t_string_fixed<'a>(&mut self, value: &'a ShapeTStringFixed) -> RidlResult<()>;
    fn write_shape_t_string_varying<'a>(
        &mut self,
        value: &'a ShapeTStringVarying,
    ) -> RidlResult<()>;
    fn write_shape_t_blob<'a>(&mut self, value: &'a ShapeTBlob) -> RidlResult<()>;
    fn write_shape_t_clob<'a>(&mut self, value: &'a ShapeTClob) -> RidlResult<()>;
    fn write_shape_t_date<'a>(&mut self, value: &'a ShapeTDate) -> RidlResult<()>;
    fn write_shape_t_time<'a>(&mut self, value: &'a ShapeTTime) -> RidlResult<()>;
    fn write_shape_t_time_tz<'a>(&mut self, value: &'a ShapeTTimeTz) -> RidlResult<()>;
    fn write_shape_t_timestamp<'a>(&mut self, value: &'a ShapeTTimestamp) -> RidlResult<()>;
    fn write_shape_t_timestamp_tz<'a>(&mut self, value: &'a ShapeTTimestampTz) -> RidlResult<()>;
    fn write_shape_t_bag<'a>(&mut self, value: &'a ShapeTBag<'a>) -> RidlResult<()>;
    fn write_shape_t_array<'a>(&mut self, value: &'a ShapeTArray<'a>) -> RidlResult<()>;
    fn write_shape_t_struct<'a>(&mut self, value: &'a ShapeTStruct) -> RidlResult<()>;
    fn write_shape_t_union<'a>(&mut self, value: &'a ShapeTUnion) -> RidlResult<()>;
    fn write_field<'a>(&mut self, value: &'a Field<'a>) -> RidlResult<()>;
}

pub struct LinacWriterBuilder {}

impl LinacWriterBuilder {

    //
    // Create the `text` encoder from the given writer.
    //
    pub(crate) fn text<'a, W, I>(writer: &mut I) -> LinacWriterText<W, I> 
    where 
        W: 'a,
        I: IonWriter<Output = W>,
    {
        LinacWriterText { writer: RidlWriter::new(writer) }
    }
}

pub struct LinacWriterText<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>,
{
    writer: RidlWriter<'a, W, I>,
}

impl<'b, W, I> LinacWriter for LinacWriterText<'b, W, I>
where
    W: 'b,
    I: IonWriter<Output = W>,
{

    fn write_shape<'a>(&mut self, value: &'a Shape<'a>) -> RidlResult<()> {
        match value {
            Shape::TDynamic(v) => self.write_shape_t_dynamic(v),
            Shape::TUnknown(v) => self.write_shape_t_unknown(v),
            Shape::TBool(v) => self.write_shape_t_bool(v),
            Shape::TInt8(v) => self.write_shape_t_int8(v),
            Shape::TInt16(v) => self.write_shape_t_int16(v),
            Shape::TInt32(v) => self.write_shape_t_int32(v),
            Shape::TInt64(v) => self.write_shape_t_int64(v),
            Shape::TInt(v) => self.write_shape_t_int(v),
            Shape::TDecimal(v) => self.write_shape_t_decimal(v),
            Shape::TNumeric(v) => self.write_shape_t_numeric(v),
            Shape::TFloat32(v) => self.write_shape_t_float32(v),
            Shape::TFloat64(v) => self.write_shape_t_float64(v),
            Shape::TFloat(v) => self.write_shape_t_float(v),
            Shape::TString(v) => self.write_shape_t_string(v),
            Shape::TStringFixed(v) => self.write_shape_t_string_fixed(v),
            Shape::TStringVarying(v) => self.write_shape_t_string_varying(v),
            Shape::TBlob(v) => self.write_shape_t_blob(v),
            Shape::TClob(v) => self.write_shape_t_clob(v),
            Shape::TDate(v) => self.write_shape_t_date(v),
            Shape::TTime(v) => self.write_shape_t_time(v),
            Shape::TTimeTz(v) => self.write_shape_t_time_tz(v),
            Shape::TTimestamp(v) => self.write_shape_t_timestamp(v),
            Shape::TTimestampTz(v) => self.write_shape_t_timestamp_tz(v),
            Shape::TBag(v) => self.write_shape_t_bag(v),
            Shape::TArray(v) => self.write_shape_t_array(v),
            Shape::TStruct(v) => self.write_shape_t_struct(v),
            Shape::TUnion(v) => self.write_shape_t_union(v),
        }
    }

    fn write_shape_t_dynamic<'a>(&mut self, value: &'a ShapeTDynamic) -> RidlResult<()> {
        self.writer.set_tag("shape.t_dynamic");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_unknown<'a>(&mut self, value: &'a ShapeTUnknown) -> RidlResult<()> {
        self.writer.set_tag("shape.t_unknown");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_bool<'a>(&mut self, value: &'a ShapeTBool) -> RidlResult<()> {
        self.writer.set_tag("shape.t_bool");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int8<'a>(&mut self, value: &'a ShapeTInt8) -> RidlResult<()> {
        self.writer.set_tag("shape.t_int8");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int16<'a>(&mut self, value: &'a ShapeTInt16) -> RidlResult<()> {
        self.writer.set_tag("shape.t_int16");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int32<'a>(&mut self, value: &'a ShapeTInt32) -> RidlResult<()> {
        self.writer.set_tag("shape.t_int32");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int64<'a>(&mut self, value: &'a ShapeTInt64) -> RidlResult<()> {
        self.writer.set_tag("shape.t_int64");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int<'a>(&mut self, value: &'a ShapeTInt) -> RidlResult<()> {
        self.writer.set_tag("shape.t_int");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_decimal<'a>(&mut self, value: &'a ShapeTDecimal) -> RidlResult<()> {
        self.writer.set_tag("shape.t_decimal");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_numeric<'a>(&mut self, value: &'a ShapeTNumeric) -> RidlResult<()> {
        self.writer.set_tag("shape.t_numeric");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("scale");
        self.writer.write_int(value.scale)?;
        self.writer.step_out()
    }

    fn write_shape_t_float32<'a>(&mut self, value: &'a ShapeTFloat32) -> RidlResult<()> {
        self.writer.set_tag("shape.t_float32");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_float64<'a>(&mut self, value: &'a ShapeTFloat64) -> RidlResult<()> {
        self.writer.set_tag("shape.t_float64");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_float<'a>(&mut self, value: &'a ShapeTFloat) -> RidlResult<()> {
        self.writer.set_tag("shape.t_float");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_string<'a>(&mut self, value: &'a ShapeTString) -> RidlResult<()> {
        self.writer.set_tag("shape.t_string");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_string_fixed<'a>(&mut self, value: &'a ShapeTStringFixed) -> RidlResult<()> {
        self.writer.set_tag("shape.t_string_fixed");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("length");
        self.writer.write_int(value.length)?;
        self.writer.step_out()
    }

    fn write_shape_t_string_varying<'a>(
        &mut self,
        value: &'a ShapeTStringVarying,
    ) -> RidlResult<()> {
        self.writer.set_tag("shape.t_string_varying");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("length");
        self.writer.write_int(value.length)?;
        self.writer.step_out()
    }

    fn write_shape_t_blob<'a>(&mut self, value: &'a ShapeTBlob) -> RidlResult<()> {
        self.writer.set_tag("shape.t_blob");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_clob<'a>(&mut self, value: &'a ShapeTClob) -> RidlResult<()> {
        self.writer.set_tag("shape.t_clob");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_date<'a>(&mut self, value: &'a ShapeTDate) -> RidlResult<()> {
        self.writer.set_tag("shape.t_date");
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_time<'a>(&mut self, value: &'a ShapeTTime) -> RidlResult<()> {
        self.writer.set_tag("shape.t_time");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_time_tz<'a>(&mut self, value: &'a ShapeTTimeTz) -> RidlResult<()> {
        self.writer.set_tag("shape.t_time_tz");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("offset_hour");
        self.writer.write_int(value.offsetHour)?;
        self.writer.set_field_name("offset_minute");
        self.writer.write_int(value.offsetMinute)?;
        self.writer.step_out()
    }

    fn write_shape_t_timestamp<'a>(&mut self, value: &'a ShapeTTimestamp) -> RidlResult<()> {
        self.writer.set_tag("shape.t_timestamp");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_timestamp_tz<'a>(&mut self, value: &'a ShapeTTimestampTz) -> RidlResult<()> {
        self.writer.set_tag("shape.t_timestamp_tz");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("offset_hour");
        self.writer.write_int(value.offsetHour)?;
        self.writer.set_field_name("offset_minute");
        self.writer.write_int(value.offsetMinute)?;
        self.writer.step_out()
    }

    fn write_shape_t_bag<'a>(&mut self, value: &'a ShapeTBag<'a>) -> RidlResult<()> {
        self.writer.set_tag("shape.t_bag");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("items");
        self.write_shape(&value.items)?;
        self.writer.step_out()
    }

    fn write_shape_t_array<'a>(&mut self, value: &'a ShapeTArray<'a>) -> RidlResult<()> {
        self.writer.set_tag("shape.t_array");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("items");
        self.write_shape(&value.items)?;
        self.writer.step_out()
    }

    fn write_shape_t_struct<'a>(&mut self, value: &'a ShapeTStruct) -> RidlResult<()> {
        self.writer.set_tag("shape.t_struct");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("fields");
        self.writer.write_array_start(&value.fields, None)?;
        for v_0 in &value.fields {
            self.write_field(v_0)?;
        }
        self.writer.write_array_end()?;
        self.writer.step_out()
    }

    fn write_shape_t_union<'a>(&mut self, value: &'a ShapeTUnion) -> RidlResult<()> {
        self.writer.set_tag("shape.t_union");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("variants");
        self.writer.write_array_start(&value.variants, None)?;
        for v_0 in &value.variants {
            self.write_shape(v_0)?;
        }
        self.writer.write_array_end()?;
        self.writer.step_out()
    }

    fn write_field<'a>(&mut self, value: &'a Field<'a>) -> RidlResult<()> {
        self.writer.set_tag("field");
        self.writer.step_in(IonType::Struct)?;
        self.writer.set_field_name("name");
        self.writer.write_string(&value.name)?;
        self.writer.set_field_name("shape");
        self.write_shape(&value.shape)?;
        self.writer.step_out()
    }
}

struct LinacWriterPacked<'a, W, I>
where
    W: 'a,
    I: IonWriter<Output = W>
{
    writer: RidlWriter<'a, W, I>,
}

impl<'b, W, I> LinacWriter for LinacWriterPacked<'b, W, I>
where
    W: 'b,
    I: IonWriter<Output = W>,
{
    fn write_shape<'a>(&mut self, value: &'a Shape<'a>) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        match value {
            Shape::TDynamic(v) => {
                self.writer.write_int(0)?;
                self.write_shape_t_dynamic(v)?;
            }
            Shape::TUnknown(v) => {
                self.writer.write_int(1)?;
                self.write_shape_t_unknown(v)?;
            }
            Shape::TBool(v) => {
                self.writer.write_int(2)?;
                self.write_shape_t_bool(v)?;
            }
            Shape::TInt8(v) => {
                self.writer.write_int(3)?;
                self.write_shape_t_int8(v)?;
            }
            Shape::TInt16(v) => {
                self.writer.write_int(4)?;
                self.write_shape_t_int16(v)?;
            }
            Shape::TInt32(v) => {
                self.writer.write_int(5)?;
                self.write_shape_t_int32(v)?;
            }
            Shape::TInt64(v) => {
                self.writer.write_int(6)?;
                self.write_shape_t_int64(v)?;
            }
            Shape::TInt(v) => {
                self.writer.write_int(7)?;
                self.write_shape_t_int(v)?;
            }
            Shape::TDecimal(v) => {
                self.writer.write_int(8)?;
                self.write_shape_t_decimal(v)?;
            }
            Shape::TNumeric(v) => {
                self.writer.write_int(9)?;
                self.write_shape_t_numeric(v)?;
            }
            Shape::TFloat32(v) => {
                self.writer.write_int(10)?;
                self.write_shape_t_float32(v)?;
            }
            Shape::TFloat64(v) => {
                self.writer.write_int(11)?;
                self.write_shape_t_float64(v)?;
            }
            Shape::TFloat(v) => {
                self.writer.write_int(12)?;
                self.write_shape_t_float(v)?;
            }
            Shape::TString(v) => {
                self.writer.write_int(13)?;
                self.write_shape_t_string(v)?;
            }
            Shape::TStringFixed(v) => {
                self.writer.write_int(14)?;
                self.write_shape_t_string_fixed(v)?;
            }
            Shape::TStringVarying(v) => {
                self.writer.write_int(15)?;
                self.write_shape_t_string_varying(v)?;
            }
            Shape::TBlob(v) => {
                self.writer.write_int(16)?;
                self.write_shape_t_blob(v)?;
            }
            Shape::TClob(v) => {
                self.writer.write_int(17)?;
                self.write_shape_t_clob(v)?;
            }
            Shape::TDate(v) => {
                self.writer.write_int(18)?;
                self.write_shape_t_date(v)?;
            }
            Shape::TTime(v) => {
                self.writer.write_int(19)?;
                self.write_shape_t_time(v)?;
            }
            Shape::TTimeTz(v) => {
                self.writer.write_int(20)?;
                self.write_shape_t_time_tz(v)?;
            }
            Shape::TTimestamp(v) => {
                self.writer.write_int(21)?;
                self.write_shape_t_timestamp(v)?;
            }
            Shape::TTimestampTz(v) => {
                self.writer.write_int(22)?;
                self.write_shape_t_timestamp_tz(v)?;
            }
            Shape::TBag(v) => {
                self.writer.write_int(23)?;
                self.write_shape_t_bag(v)?;
            }
            Shape::TArray(v) => {
                self.writer.write_int(24)?;
                self.write_shape_t_array(v)?;
            }
            Shape::TStruct(v) => {
                self.writer.write_int(25)?;
                self.write_shape_t_struct(v)?;
            }
            Shape::TUnion(v) => {
                self.writer.write_int(26)?;
                self.write_shape_t_union(v)?;
            }
        }
        self.writer.step_out()
    }

    fn write_shape_t_dynamic<'a>(&mut self, value: &'a ShapeTDynamic) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_unknown<'a>(&mut self, value: &'a ShapeTUnknown) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_bool<'a>(&mut self, value: &'a ShapeTBool) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int8<'a>(&mut self, value: &'a ShapeTInt8) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int16<'a>(&mut self, value: &'a ShapeTInt16) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int32<'a>(&mut self, value: &'a ShapeTInt32) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int64<'a>(&mut self, value: &'a ShapeTInt64) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_int<'a>(&mut self, value: &'a ShapeTInt) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_decimal<'a>(&mut self, value: &'a ShapeTDecimal) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_numeric<'a>(&mut self, value: &'a ShapeTNumeric) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("scale");
        self.writer.write_int(value.scale)?;
        self.writer.step_out()
    }

    fn write_shape_t_float32<'a>(&mut self, value: &'a ShapeTFloat32) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_float64<'a>(&mut self, value: &'a ShapeTFloat64) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_float<'a>(&mut self, value: &'a ShapeTFloat) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_string<'a>(&mut self, value: &'a ShapeTString) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_string_fixed<'a>(&mut self, value: &'a ShapeTStringFixed) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("length");
        self.writer.write_int(value.length)?;
        self.writer.step_out()
    }

    fn write_shape_t_string_varying<'a>(
        &mut self,
        value: &'a ShapeTStringVarying,
    ) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("length");
        self.writer.write_int(value.length)?;
        self.writer.step_out()
    }

    fn write_shape_t_blob<'a>(&mut self, value: &'a ShapeTBlob) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_clob<'a>(&mut self, value: &'a ShapeTClob) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_date<'a>(&mut self, value: &'a ShapeTDate) -> RidlResult<()> {
        self.writer.write_symbol("unit")
    }

    fn write_shape_t_time<'a>(&mut self, value: &'a ShapeTTime) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_time_tz<'a>(&mut self, value: &'a ShapeTTimeTz) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("offset_hour");
        self.writer.write_int(value.offsetHour)?;
        self.writer.set_field_name("offset_minute");
        self.writer.write_int(value.offsetMinute)?;
        self.writer.step_out()
    }

    fn write_shape_t_timestamp<'a>(&mut self, value: &'a ShapeTTimestamp) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.step_out()
    }

    fn write_shape_t_timestamp_tz<'a>(&mut self, value: &'a ShapeTTimestampTz) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("precision");
        self.writer.write_int(value.precision)?;
        self.writer.set_field_name("offset_hour");
        self.writer.write_int(value.offsetHour)?;
        self.writer.set_field_name("offset_minute");
        self.writer.write_int(value.offsetMinute)?;
        self.writer.step_out()
    }

    fn write_shape_t_bag<'a>(&mut self, value: &'a ShapeTBag<'a>) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("items");
        self.write_shape(&value.items)?;
        self.writer.step_out()
    }

    fn write_shape_t_array<'a>(&mut self, value: &'a ShapeTArray<'a>) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("items");
        self.write_shape(&value.items)?;
        self.writer.step_out()
    }

    fn write_shape_t_struct<'a>(&mut self, value: &'a ShapeTStruct) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("fields");
        self.writer.write_array_start(&value.fields, None)?;
        for v_0 in &value.fields {
            self.write_field(v_0)?;
        }
        self.writer.write_array_end()?;
        self.writer.step_out()
    }

    fn write_shape_t_union<'a>(&mut self, value: &'a ShapeTUnion) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("variants");
        self.writer.write_array_start(&value.variants, None)?;
        for v_0 in &value.variants {
            self.write_shape(v_0)?;
        }
        self.writer.write_array_end()?;
        self.writer.step_out()
    }

    fn write_field<'a>(&mut self, value: &'a Field<'a>) -> RidlResult<()> {
        self.writer.step_in(IonType::SExp)?;
        self.writer.set_field_name("name");
        self.writer.write_string(&value.name)?;
        self.writer.set_field_name("shape");
        self.write_shape(&value.shape)?;
        self.writer.step_out()
    }
}
pub trait LinacReader<'a> {
    fn read_shape(&mut self) -> RidlResult<&'a Shape<'a>>;
    fn read_shape_t_dynamic(&mut self) -> RidlResult<&'a ShapeTDynamic>;
    fn read_shape_t_unknown(&mut self) -> RidlResult<&'a ShapeTUnknown>;
    fn read_shape_t_bool(&mut self) -> RidlResult<&'a ShapeTBool>;
    fn read_shape_t_int8(&mut self) -> RidlResult<&'a ShapeTInt8>;
    fn read_shape_t_int16(&mut self) -> RidlResult<&'a ShapeTInt16>;
    fn read_shape_t_int32(&mut self) -> RidlResult<&'a ShapeTInt32>;
    fn read_shape_t_int64(&mut self) -> RidlResult<&'a ShapeTInt64>;
    fn read_shape_t_int(&mut self) -> RidlResult<&'a ShapeTInt>;
    fn read_shape_t_decimal(&mut self) -> RidlResult<&'a ShapeTDecimal>;
    fn read_shape_t_numeric(&mut self) -> RidlResult<&'a ShapeTNumeric>;
    fn read_shape_t_float32(&mut self) -> RidlResult<&'a ShapeTFloat32>;
    fn read_shape_t_float64(&mut self) -> RidlResult<&'a ShapeTFloat64>;
    fn read_shape_t_float(&mut self) -> RidlResult<&'a ShapeTFloat>;
    fn read_shape_t_string(&mut self) -> RidlResult<&'a ShapeTString>;
    fn read_shape_t_string_fixed(&mut self) -> RidlResult<&'a ShapeTStringFixed>;
    fn read_shape_t_string_varying(&mut self) -> RidlResult<&'a ShapeTStringVarying>;
    fn read_shape_t_blob(&mut self) -> RidlResult<&'a ShapeTBlob>;
    fn read_shape_t_clob(&mut self) -> RidlResult<&'a ShapeTClob>;
    fn read_shape_t_date(&mut self) -> RidlResult<&'a ShapeTDate>;
    fn read_shape_t_time(&mut self) -> RidlResult<&'a ShapeTTime>;
    fn read_shape_t_time_tz(&mut self) -> RidlResult<&'a ShapeTTimeTz>;
    fn read_shape_t_timestamp(&mut self) -> RidlResult<&'a ShapeTTimestamp>;
    fn read_shape_t_timestamp_tz(&mut self) -> RidlResult<&'a ShapeTTimestampTz>;
    fn read_shape_t_bag(&mut self) -> RidlResult<&'a ShapeTBag<'a>>;
    fn read_shape_t_array(&mut self) -> RidlResult<&'a ShapeTArray<'a>>;
    fn read_shape_t_struct(&mut self) -> RidlResult<&'a ShapeTStruct>;
    fn read_shape_t_union(&mut self) -> RidlResult<&'a ShapeTUnion>;
    fn read_field(&mut self) -> RidlResult<&'a Field<'a>>;
}

pub struct LinacReaderBuilder {}

impl LinacReaderBuilder {

    pub fn text<'a, I: 'a + ToIonDataSource>(arena: &'a Arena, input: I) -> LinacReaderText {
        let reader = ReaderBuilder::default()
            .build(input)
            .expect("Failed to instantiate IonReader.");
        LinacReaderText {
            arena,
            reader: RidlReader::new(reader),
        }
    }

    fn packed<'a, I: 'a + ToIonDataSource>(arena: &'a Arena, input: I) -> LinacReaderPacked {
        let reader = ReaderBuilder::default()
            .build(input)
            .expect("Failed to instantiate IonReader.");
        LinacReaderPacked {
            arena,
            reader: RidlReader::new(reader),
        }
    }
}

struct LinacReaderText<'a> {
    arena: &'a Arena,
    reader: RidlReader<'a>,
}

impl<'a> LinacReader<'a> for LinacReaderText<'a> {
    fn read_shape(&mut self) -> RidlResult<&'a Shape<'a>> {
        todo!()
    }

    fn read_shape_t_dynamic(&mut self) -> RidlResult<&'a ShapeTDynamic> {
        todo!()
    }

    fn read_shape_t_unknown(&mut self) -> RidlResult<&'a ShapeTUnknown> {
        todo!()
    }

    fn read_shape_t_bool(&mut self) -> RidlResult<&'a ShapeTBool> {
        todo!()
    }

    fn read_shape_t_int8(&mut self) -> RidlResult<&'a ShapeTInt8> {
        todo!()
    }

    fn read_shape_t_int16(&mut self) -> RidlResult<&'a ShapeTInt16> {
        todo!()
    }

    fn read_shape_t_int32(&mut self) -> RidlResult<&'a ShapeTInt32> {
        todo!()
    }

    fn read_shape_t_int64(&mut self) -> RidlResult<&'a ShapeTInt64> {
        todo!()
    }

    fn read_shape_t_int(&mut self) -> RidlResult<&'a ShapeTInt> {
        todo!()
    }

    fn read_shape_t_decimal(&mut self) -> RidlResult<&'a ShapeTDecimal> {
        todo!()
    }

    fn read_shape_t_numeric(&mut self) -> RidlResult<&'a ShapeTNumeric> {
        todo!()
    }

    fn read_shape_t_float32(&mut self) -> RidlResult<&'a ShapeTFloat32> {
        todo!()
    }

    fn read_shape_t_float64(&mut self) -> RidlResult<&'a ShapeTFloat64> {
        todo!()
    }

    fn read_shape_t_float(&mut self) -> RidlResult<&'a ShapeTFloat> {
        todo!()
    }

    fn read_shape_t_string(&mut self) -> RidlResult<&'a ShapeTString> {
        todo!()
    }

    fn read_shape_t_string_fixed(&mut self) -> RidlResult<&'a ShapeTStringFixed> {
        todo!()
    }

    fn read_shape_t_string_varying(&mut self) -> RidlResult<&'a ShapeTStringVarying> {
        todo!()
    }

    fn read_shape_t_blob(&mut self) -> RidlResult<&'a ShapeTBlob> {
        todo!()
    }

    fn read_shape_t_clob(&mut self) -> RidlResult<&'a ShapeTClob> {
        todo!()
    }

    fn read_shape_t_date(&mut self) -> RidlResult<&'a ShapeTDate> {
        todo!()
    }

    fn read_shape_t_time(&mut self) -> RidlResult<&'a ShapeTTime> {
        todo!()
    }

    fn read_shape_t_time_tz(&mut self) -> RidlResult<&'a ShapeTTimeTz> {
        todo!()
    }

    fn read_shape_t_timestamp(&mut self) -> RidlResult<&'a ShapeTTimestamp> {
        todo!()
    }

    fn read_shape_t_timestamp_tz(&mut self) -> RidlResult<&'a ShapeTTimestampTz> {
        todo!()
    }

    fn read_shape_t_bag(&mut self) -> RidlResult<&'a ShapeTBag<'a>> {
        todo!()
    }

    fn read_shape_t_array(&mut self) -> RidlResult<&'a ShapeTArray<'a>> {
        todo!()
    }

    fn read_shape_t_struct(&mut self) -> RidlResult<&'a ShapeTStruct> {
        todo!()
    }

    fn read_shape_t_union(&mut self) -> RidlResult<&'a ShapeTUnion> {
        todo!()
    }

    fn read_field(&mut self) -> RidlResult<&'a Field<'a>> {
        todo!()
    }
}

struct LinacReaderPacked<'a> {
    arena: &'a Arena,
    reader: RidlReader<'a>,
}

impl<'a> LinacReader<'a> for LinacReaderPacked<'a> {
    fn read_shape(&mut self) -> RidlResult<&'a Shape<'a>> {
        todo!()
    }

    fn read_shape_t_dynamic(&mut self) -> RidlResult<&'a ShapeTDynamic> {
        todo!()
    }

    fn read_shape_t_unknown(&mut self) -> RidlResult<&'a ShapeTUnknown> {
        todo!()
    }

    fn read_shape_t_bool(&mut self) -> RidlResult<&'a ShapeTBool> {
        todo!()
    }

    fn read_shape_t_int8(&mut self) -> RidlResult<&'a ShapeTInt8> {
        todo!()
    }

    fn read_shape_t_int16(&mut self) -> RidlResult<&'a ShapeTInt16> {
        todo!()
    }

    fn read_shape_t_int32(&mut self) -> RidlResult<&'a ShapeTInt32> {
        todo!()
    }

    fn read_shape_t_int64(&mut self) -> RidlResult<&'a ShapeTInt64> {
        todo!()
    }

    fn read_shape_t_int(&mut self) -> RidlResult<&'a ShapeTInt> {
        todo!()
    }

    fn read_shape_t_decimal(&mut self) -> RidlResult<&'a ShapeTDecimal> {
        todo!()
    }

    fn read_shape_t_numeric(&mut self) -> RidlResult<&'a ShapeTNumeric> {
        todo!()
    }

    fn read_shape_t_float32(&mut self) -> RidlResult<&'a ShapeTFloat32> {
        todo!()
    }

    fn read_shape_t_float64(&mut self) -> RidlResult<&'a ShapeTFloat64> {
        todo!()
    }

    fn read_shape_t_float(&mut self) -> RidlResult<&'a ShapeTFloat> {
        todo!()
    }

    fn read_shape_t_string(&mut self) -> RidlResult<&'a ShapeTString> {
        todo!()
    }

    fn read_shape_t_string_fixed(&mut self) -> RidlResult<&'a ShapeTStringFixed> {
        todo!()
    }

    fn read_shape_t_string_varying(&mut self) -> RidlResult<&'a ShapeTStringVarying> {
        todo!()
    }

    fn read_shape_t_blob(&mut self) -> RidlResult<&'a ShapeTBlob> {
        todo!()
    }

    fn read_shape_t_clob(&mut self) -> RidlResult<&'a ShapeTClob> {
        todo!()
    }

    fn read_shape_t_date(&mut self) -> RidlResult<&'a ShapeTDate> {
        todo!()
    }

    fn read_shape_t_time(&mut self) -> RidlResult<&'a ShapeTTime> {
        todo!()
    }

    fn read_shape_t_time_tz(&mut self) -> RidlResult<&'a ShapeTTimeTz> {
        todo!()
    }

    fn read_shape_t_timestamp(&mut self) -> RidlResult<&'a ShapeTTimestamp> {
        todo!()
    }

    fn read_shape_t_timestamp_tz(&mut self) -> RidlResult<&'a ShapeTTimestampTz> {
        todo!()
    }

    fn read_shape_t_bag(&mut self) -> RidlResult<&'a ShapeTBag<'a>> {
        todo!()
    }

    fn read_shape_t_array(&mut self) -> RidlResult<&'a ShapeTArray<'a>> {
        todo!()
    }

    fn read_shape_t_struct(&mut self) -> RidlResult<&'a ShapeTStruct> {
        todo!()
    }

    fn read_shape_t_union(&mut self) -> RidlResult<&'a ShapeTUnion> {
        todo!()
    }

    fn read_field(&mut self) -> RidlResult<&'a Field<'a>> {
        todo!()
    }
}
