use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder,
    TimestampMillisecondBuilder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use partiql_beamline::primitives::Sample;
use partiql_beamline::sim::{DataSetSampler, ISim};
use partiql_types::{PartiqlShape, Static};
use partiql_value::{BindingsName, DateTime, Value};
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use crate::writer::{SimWriter, SimWriterError, SimWriterResult};

#[derive(Debug, Error)]
pub enum ParquetError {
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Schema error: {0}")]
    Schema(String),
}

impl From<ParquetError> for SimWriterError {
    fn from(e: ParquetError) -> Self {
        SimWriterError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

pub struct SimWriterParquet {
    pub sampler: DataSetSampler,
    pub output_path: String,
}

impl SimWriter for SimWriterParquet {
    fn write(&mut self) -> SimWriterResult<()> {
        let output_dir = Path::new(&self.output_path);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let shapes = self.sampler.sim().shape();

        for (dataset, samples) in self.sampler.iter_mut() {
            let dataset_name = &dataset.0;
            let shape = shapes.get_shape(dataset_name);

            let rows: Vec<Value> = samples
                .filter_map(|s| s.ok())
                .map(|Sample { value, .. }| value)
                .collect();

            if rows.is_empty() {
                continue;
            }

            let arrow_schema = match shape {
                Some(s) => shape_to_arrow_schema(s)?,
                None => infer_schema_from_values(&rows)?,
            };

            let batch = values_to_record_batch(&rows, &arrow_schema)?;

            let file_path = output_dir.join(format!("{dataset_name}.parquet"));
            let file = File::create(&file_path)?;
            let props = WriterProperties::builder().build();
            let mut writer = ArrowWriter::try_new(file, Arc::new(arrow_schema), Some(props))
                .map_err(ParquetError::from)?;
            writer.write(&batch).map_err(ParquetError::from)?;
            writer.close().map_err(ParquetError::from)?;

            println!(
                "wrote {} row(s) to {}",
                rows.len(),
                file_path.display()
            );
        }

        Ok(())
    }
}

fn shape_to_arrow_schema(shape: &PartiqlShape) -> SimWriterResult<Schema> {
    let fields = shape_to_fields(shape)?;
    Ok(Schema::new(fields))
}

fn shape_to_fields(shape: &PartiqlShape) -> SimWriterResult<Vec<Field>> {
    match shape {
        PartiqlShape::Static(stype) => match stype.ty() {
            Static::Struct(s) => {
                let fields: Vec<Field> = s
                    .fields()
                    .map(|f| {
                        let dt = partiql_type_to_arrow(f.ty());
                        Field::new(f.name(), dt, true)
                    })
                    .collect();
                Ok(fields)
            }
            Static::Bag(b) => shape_to_fields(b.element_type()),
            _ => Err(ParquetError::Schema(
                "top-level shape must be a struct or bag of structs".to_string(),
            )
            .into()),
        },
        PartiqlShape::AnyOf(any_of) => {
            for t in any_of.types() {
                if let Ok(fields) = shape_to_fields(t) {
                    return Ok(fields);
                }
            }
            Err(
                ParquetError::Schema("could not resolve union to struct fields".to_string())
                    .into(),
            )
        }
        _ => Err(
            ParquetError::Schema(format!("unsupported top-level shape: {shape}")).into(),
        ),
    }
}

fn partiql_type_to_arrow(shape: &PartiqlShape) -> DataType {
    match shape {
        PartiqlShape::Static(stype) => match stype.ty() {
            Static::Bool => DataType::Boolean,
            Static::Int | Static::Int64 => DataType::Int64,
            Static::Int8 => DataType::Int64,
            Static::Int16 => DataType::Int64,
            Static::Int32 => DataType::Int64,
            Static::Float32 | Static::Float64 => DataType::Float64,
            Static::Decimal | Static::DecimalP(_, _) => DataType::Utf8,
            Static::String | Static::StringFixed(_) | Static::StringVarying(_) => DataType::Utf8,
            Static::DateTime => DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())),
            Static::Struct(s) => {
                let fields: Vec<Field> = s
                    .fields()
                    .map(|f| Field::new(f.name(), partiql_type_to_arrow(f.ty()), true))
                    .collect();
                DataType::Struct(fields.into())
            }
            Static::Array(_) | Static::Bag(_) => DataType::Utf8,
        },
        PartiqlShape::AnyOf(_) => DataType::Utf8,
        PartiqlShape::Dynamic => DataType::Utf8,
        PartiqlShape::Undefined => DataType::Utf8,
    }
}

fn infer_schema_from_values(rows: &[Value]) -> SimWriterResult<Schema> {
    let first = rows.iter().find(|v| matches!(v, Value::Tuple(_)));
    match first {
        Some(Value::Tuple(t)) => {
            let fields: Vec<Field> = t
                .pairs()
                .map(|(name, val)| Field::new(name.as_str(), infer_arrow_type(val), true))
                .collect();
            Ok(Schema::new(fields))
        }
        _ => Err(
            ParquetError::Schema("cannot infer schema - no tuple rows found".to_string()).into(),
        ),
    }
}

fn infer_arrow_type(val: &Value) -> DataType {
    match val {
        Value::Boolean(_) => DataType::Boolean,
        Value::Integer(_) => DataType::Int64,
        Value::Real(_) => DataType::Float64,
        Value::Decimal(_) => DataType::Utf8,
        Value::String(_) => DataType::Utf8,
        Value::DateTime(_) => DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())),
        Value::Tuple(_) => DataType::Utf8,
        Value::List(_) => DataType::Utf8,
        Value::Bag(_) => DataType::Utf8,
        Value::Blob(_) => DataType::Binary,
        Value::Null | Value::Missing => DataType::Utf8,
    }
}

fn values_to_record_batch(rows: &[Value], schema: &Schema) -> SimWriterResult<RecordBatch> {
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len());

    for field in schema.fields() {
        let array = build_column(field.name(), field.data_type(), rows)?;
        columns.push(array);
    }

    RecordBatch::try_new(Arc::new(schema.clone()), columns)
        .map_err(|e| ParquetError::Arrow(e).into())
}

fn build_column(name: &str, data_type: &DataType, rows: &[Value]) -> SimWriterResult<ArrayRef> {
    match data_type {
        DataType::Boolean => {
            let mut builder = BooleanBuilder::with_capacity(rows.len());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::Boolean(b)) => builder.append_value(*b),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Int64 => {
            let mut builder = Int64Builder::with_capacity(rows.len());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::Integer(i)) => builder.append_value(*i),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Float64 => {
            let mut builder = Float64Builder::with_capacity(rows.len());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::Real(f)) => builder.append_value(f.0),
                    Some(Value::Integer(i)) => builder.append_value(*i as f64),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Utf8 => {
            let mut builder = StringBuilder::new();
            for row in rows {
                match get_field(row, name) {
                    Some(Value::String(s)) => builder.append_value(s.as_str()),
                    Some(Value::Decimal(d)) => builder.append_value(d.to_string()),
                    Some(Value::List(l)) => {
                        builder.append_value(format!("{l:?}"));
                    }
                    Some(Value::Bag(b)) => {
                        builder.append_value(format!("{b:?}"));
                    }
                    Some(Value::Null) | Some(Value::Missing) | None => builder.append_null(),
                    Some(other) => builder.append_value(format!("{other:?}")),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            let mut builder = TimestampMillisecondBuilder::with_capacity(rows.len());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::DateTime(dt)) => {
                        let millis = datetime_to_epoch_millis(dt);
                        builder.append_value(millis);
                    }
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish().with_timezone("UTC")))
        }
        DataType::Struct(fields) => {
            let child_arrays: Vec<ArrayRef> = fields
                .iter()
                .map(|f| build_nested_column(name, f.name(), f.data_type(), rows))
                .collect::<SimWriterResult<Vec<_>>>()?;

            let struct_array =
                arrow::array::StructArray::try_new(fields.clone(), child_arrays, None)
                    .map_err(ParquetError::Arrow)?;
            Ok(Arc::new(struct_array))
        }
        _ => {
            let mut builder = StringBuilder::new();
            for row in rows {
                match get_field(row, name) {
                    Some(Value::String(s)) => builder.append_value(s.as_str()),
                    Some(Value::Null) | Some(Value::Missing) | None => builder.append_null(),
                    Some(v) => builder.append_value(format!("{v:?}")),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
    }
}

fn build_nested_column(
    parent_name: &str,
    child_name: &str,
    data_type: &DataType,
    rows: &[Value],
) -> SimWriterResult<ArrayRef> {
    match data_type {
        DataType::Boolean => {
            let mut builder = BooleanBuilder::with_capacity(rows.len());
            for row in rows {
                match get_nested_field(row, parent_name, child_name) {
                    Some(Value::Boolean(b)) => builder.append_value(*b),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Int64 => {
            let mut builder = Int64Builder::with_capacity(rows.len());
            for row in rows {
                match get_nested_field(row, parent_name, child_name) {
                    Some(Value::Integer(i)) => builder.append_value(*i),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Float64 => {
            let mut builder = Float64Builder::with_capacity(rows.len());
            for row in rows {
                match get_nested_field(row, parent_name, child_name) {
                    Some(Value::Real(f)) => builder.append_value(f.0),
                    Some(Value::Integer(i)) => builder.append_value(*i as f64),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            let mut builder = TimestampMillisecondBuilder::with_capacity(rows.len());
            for row in rows {
                match get_nested_field(row, parent_name, child_name) {
                    Some(Value::DateTime(dt)) => {
                        builder.append_value(datetime_to_epoch_millis(dt));
                    }
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish().with_timezone("UTC")))
        }
        _ => {
            let mut builder = StringBuilder::new();
            for row in rows {
                match get_nested_field(row, parent_name, child_name) {
                    Some(Value::String(s)) => builder.append_value(s.as_str()),
                    Some(Value::Decimal(d)) => builder.append_value(d.to_string()),
                    Some(Value::Null) | Some(Value::Missing) | None => builder.append_null(),
                    Some(v) => builder.append_value(format!("{v:?}")),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
    }
}

fn get_field<'a>(row: &'a Value, name: &str) -> Option<&'a Value> {
    match row {
        Value::Tuple(t) => {
            let binding = BindingsName::CaseInsensitive(Cow::Borrowed(name));
            t.get(&binding)
        }
        _ => None,
    }
}

fn get_nested_field<'a>(row: &'a Value, parent: &str, child: &str) -> Option<&'a Value> {
    let parent_val = get_field(row, parent)?;
    match parent_val {
        Value::Tuple(t) => {
            let binding = BindingsName::CaseInsensitive(Cow::Borrowed(child));
            t.get(&binding)
        }
        _ => None,
    }
}

fn datetime_to_epoch_millis(dt: &DateTime) -> i64 {
    match dt {
        DateTime::TimestampWithTz(ts) => {
            let unix_epoch = time::OffsetDateTime::UNIX_EPOCH;
            let duration = *ts - unix_epoch;
            duration.whole_milliseconds() as i64
        }
        DateTime::Timestamp(ts) => {
            let unix_epoch = time::OffsetDateTime::UNIX_EPOCH;
            let odt = ts.assume_utc();
            let duration = odt - unix_epoch;
            duration.whole_milliseconds() as i64
        }
        DateTime::Date(d) => {
            let midnight = d.midnight().assume_utc();
            let unix_epoch = time::OffsetDateTime::UNIX_EPOCH;
            let duration = midnight - unix_epoch;
            duration.whole_milliseconds() as i64
        }
        DateTime::Time(_) | DateTime::TimeWithTz(_, _) => 0,
    }
}
