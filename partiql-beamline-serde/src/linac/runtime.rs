use bumpalo::Bump;
use ion_rs::{Decimal, IonReader, IonWriter, IonType, Reader, Str, Symbol};
use ion_rs::types::Bytes;

use ion_rs::IonError;
use thiserror::Error;

pub type RidlResult<T> = Result<T, RidlError>;

#[derive(Error, Debug, PartialEq)]
pub enum RidlError {
    #[error("An error reading the input occurred, most likely due to malformed Ion: {0}")]
    Ion(IonError),

    #[error("{0}")]
    ReadError(String),

    #[error("{0}")]
    WriteError(String),
}

impl From<IonError> for RidlError {
    fn from(e: IonError) -> Self {
        RidlError::Ion(e)
    }
}

pub struct Arena {
    bump: Bump,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Arena { bump: Bump::new() }
    }

    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump.alloc(val)
    }
}

pub struct RidlReader<'a> {
    pub reader: Reader<'a>,
}

#[inline]
pub fn read_err<T>(description: &str) -> RidlResult<T> {
    return Err(RidlError::ReadError(description.to_string()));
}

impl<'a> RidlReader<'a> {

    pub fn new(reader: Reader) -> RidlReader {
        RidlReader { reader }
    }

    pub fn assert_type(&self, ion_type: IonType) -> RidlResult<()> {
        let actual = self.reader.ion_type().ok_or(RidlError::ReadError("Unexpected end of value.".to_string()))?;
        if actual != ion_type {
            return read_err("Unexpected type");
        }
        Ok(())
    }

    pub fn assert_field(&self, field: String) -> RidlResult<()> {
        self.reader.current();
        let symbol = self.reader.field_name()?;
        let actual = match symbol.text_or_error() {
            Ok(v) => v,
            Err(e) => return Err(RidlError::from(e)),
        };
        if field != actual {
            return read_err("Field name did not match expected field name.");
        }
        Ok(())
    }

    pub fn assert_tag(&self, tag: String) -> RidlResult<()> {
        let annotations: Vec<Symbol> = self.reader.annotations()
            .filter_map(|s| s.ok())
            .collect();
        if annotations.len() != 1 {
            return read_err("Expected a type annotation, but found none.");
        }
        let actual = annotations[0].text().ok_or(RidlError::ReadError("Failed to read annotation".to_string()))?;
        if actual != tag {
            return read_err("Type annotation did not match expected value.");
        }
        Ok(())
    }

    pub fn read_bool(&mut self) -> RidlResult<bool> {
        self.reader.read_bool().map_err(|e| RidlError::from(e))
    }

    pub fn read_int(&mut self) -> RidlResult<i64> {
        self.reader.read_i64().map_err(|e| RidlError::from(e))
    }

    pub fn read_float(&mut self) -> RidlResult<f64> {
        self.reader.read_f64().map_err(|e| RidlError::from(e))
    }

    pub fn read_decimal(&mut self, precision: Option<u64>, exponent: Option<i64>) -> RidlResult<Decimal> {
        let value = self.reader.read_decimal().map_err(|e| RidlError::from(e))?;
        if precision.is_some() && precision.unwrap() < value.precision() {
            return read_err("Decimal value precision exceeded the type's precision constraint");
        }
        if exponent.is_some() && -exponent.unwrap() < value.scale() {
            return read_err("Decimal value exponent exceeded the type's exponent constraint");
        }
        Ok(value)
    }

    pub fn read_string(&mut self) -> RidlResult<Str> {
        self.reader.read_string().map_err(|e| RidlError::from(e))
    }

    pub fn read_blob(&mut self, size: u64) -> RidlResult<Bytes> {
        let bytes = self.reader.read_blob().map_err(|e| RidlError::from(e))?.0;
        let len = bytes.as_ref().len() as u64;
        if len != size {
            return read_err("Blob size does not match definition size constraint.");
        }
        Ok(bytes)
    }

    pub fn read_clob(&mut self, size: u64) -> RidlResult<Bytes> {
        let bytes = self.reader.read_clob().map_err(|e| RidlError::from(e))?.0;
        let len = bytes.as_ref().len() as u64;
        if len != size {
            return read_err("Clob size does not match definition size constraint.");
        }
        Ok(bytes)
    }
}

pub struct RidlWriter<W> {
    writer: W,
}

#[inline]
fn write_err<T>(description: &str) -> RidlResult<T> {
    return Err(RidlError::WriteError(description.to_string()));
}

impl<W: IonWriter> RidlWriter<W> {

    pub fn new(w: W) -> RidlWriter<W> {
        RidlWriter { writer: w }
    }

    pub fn set_tag(&mut self, tag: &str) {
        let annotations: Vec<&str> = vec![tag];
        self.writer.set_annotations(annotations);
    }

    pub fn set_field_name(&mut self, field: &str) {
        self.writer.set_field_name(field);
    }

    pub fn write_bool(&mut self, value: bool) -> RidlResult<()> {
        self.writer.write_bool(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_int(&mut self, value: i64) -> RidlResult<()> {
        self.writer.write_i64(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_float(&mut self, value: f64) -> RidlResult<()> {
        self.writer.write_f64(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_decimal(&mut self, value: &Decimal, precision: Option<u64>, exponent: Option<i64>) -> RidlResult<()> {
        if precision.is_some() && precision.unwrap() < value.precision() {
            return write_err("Decimal value precision exceeded the type's precision constraint");
        }
        if exponent.is_some() && -exponent.unwrap() < value.scale() {
            return write_err("Decimal value exponent exceeded the type's exponent constraint");
        }
        if let Err(e) = self.writer.write_decimal(value) {
            return Err(RidlError::from(e));
        }
        Ok(())
    }

    pub fn write_string<T: AsRef<str>>(&mut self, value: T) -> RidlResult<()> {
        self.writer.write_string(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_symbol<T: AsRef<str>>(&mut self, value: T) -> RidlResult<()> {
        self.writer.write_symbol(value.as_ref()).map_err(|e| RidlError::from(e))
    }

    pub fn write_blob<T: AsRef<[u8]>>(&mut self, value: T, size: Option<u64>) -> RidlResult<()> {
        let bytes = value.as_ref();
        if size.is_some() && size.unwrap() != bytes.len() as u64 {
            return write_err("Blob size does not match type size constraint");
        }
        self.writer.write_blob(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_clob<T: AsRef<[u8]>>(&mut self, value: T, size: Option<u64>) -> RidlResult<()> {
        let bytes = value.as_ref();
        if size.is_some() && size.unwrap() != bytes.len() as u64 {
            return write_err("Clob size does not match type size constraint");
        }
        self.writer.write_clob(value).map_err(|e| RidlError::from(e))
    }

    pub fn write_array_start<T>(&mut self, array: &Vec<T>, size: Option<usize>) -> RidlResult<()> {
        self.writer.step_in(IonType::List)?;
        if size.is_some() && size.unwrap() != array.len() {
            return write_err("Vec size does not match array size constraint")
        }
        Ok(())
    }

    pub fn write_array_end(&mut self) -> RidlResult<()> {
        self.step_out()
    }

    pub fn step_in(&mut self, kind: IonType) -> RidlResult<()> {
        self.writer.step_in(kind).map_err(|e| RidlError::from(e))
    }

    pub fn step_out(&mut self) -> RidlResult<()> {
        self.writer.step_out().map_err(|e| RidlError::from(e))
    }
}
