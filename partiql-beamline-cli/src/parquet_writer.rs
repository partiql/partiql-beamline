use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, ListBuilder, StringBuilder,
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
    #[error("Unsupported type for Parquet: {0}")]
    UnsupportedType(String),
}

impl From<ParquetError> for SimWriterError {
    fn from(e: ParquetError) -> Self {
        SimWriterError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        ))
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

            let mut rows: Vec<Value> = Vec::new();
            for sample in samples {
                let Sample { value, .. } = sample?;
                rows.push(value);
            }

            if rows.is_empty() {
                continue;
            }

            let arrow_schema = match shape {
                Some(s) => shape_to_arrow_schema(s)?,
                None => infer_schema_from_values(&rows)?,
            };

            let batch = values_to_record_batch(&rows, &arrow_schema)?;

            let sanitized_name = sanitize_filename(dataset_name);
            let file_path = output_dir.join(format!("{sanitized_name}.parquet"));
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

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | '\0' => '_',
            _ => c,
        })
        .collect()
}

fn shape_to_arrow_schema(shape: &PartiqlShape) -> SimWriterResult<Schema> {
    let fields = shape_to_fields(shape)?;
    Ok(Schema::new(fields))
}

fn shape_to_fields(shape: &PartiqlShape) -> SimWriterResult<Vec<Field>> {
    match shape {
        PartiqlShape::Static(stype) => match stype.ty() {
            Static::Struct(s) => {
                let mut fields = Vec::new();
                for f in s.fields() {
                    let dt = partiql_type_to_arrow(f.ty())?;
                    fields.push(Field::new(f.name(), dt, true));
                }
                Ok(fields)
            }
            Static::Bag(b) => shape_to_fields(b.element_type()),
            _ => Err(ParquetError::Schema(
                "top-level shape must be a struct or bag of structs".to_string(),
            )
            .into()),
        },
        PartiqlShape::AnyOf(_) => Err(ParquetError::UnsupportedType(
            "AnyOf (union) types are not supported in Parquet output; \
             all fields must have a single concrete type"
                .to_string(),
        )
        .into()),
        _ => Err(
            ParquetError::Schema(format!("unsupported top-level shape: {shape}")).into(),
        ),
    }
}

fn partiql_type_to_arrow(shape: &PartiqlShape) -> Result<DataType, ParquetError> {
    match shape {
        PartiqlShape::Static(stype) => match stype.ty() {
            Static::Bool => Ok(DataType::Boolean),
            Static::Int | Static::Int64 => Ok(DataType::Int64),
            Static::Int8 => Ok(DataType::Int64),
            Static::Int16 => Ok(DataType::Int64),
            Static::Int32 => Ok(DataType::Int64),
            Static::Float32 | Static::Float64 => Ok(DataType::Float64),
            Static::Decimal | Static::DecimalP(_, _) => Ok(DataType::Utf8),
            Static::String | Static::StringFixed(_) | Static::StringVarying(_) => Ok(DataType::Utf8),
            Static::DateTime => {
                Ok(DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())))
            }
            Static::Struct(s) => {
                let mut fields = Vec::new();
                for f in s.fields() {
                    let dt = partiql_type_to_arrow(f.ty())?;
                    fields.push(Field::new(f.name(), dt, true));
                }
                Ok(DataType::Struct(fields.into()))
            }
            Static::Array(a) => {
                let elem_type = partiql_type_to_arrow(a.element_type())?;
                Ok(DataType::List(Arc::new(Field::new("item", elem_type, true))))
            }
            Static::Bag(b) => {
                let elem_type = partiql_type_to_arrow(b.element_type())?;
                Ok(DataType::List(Arc::new(Field::new("item", elem_type, true))))
            }
        },
        PartiqlShape::AnyOf(_) => Err(ParquetError::UnsupportedType(
            "AnyOf (union) types are not supported in Parquet output; \
             all fields must have a single concrete type"
                .to_string(),
        )),
        PartiqlShape::Dynamic => Err(ParquetError::UnsupportedType(
            "Dynamic types are not supported in Parquet output".to_string(),
        )),
        PartiqlShape::Undefined => Err(ParquetError::UnsupportedType(
            "Undefined types are not supported in Parquet output".to_string(),
        )),
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
        Value::List(l) => {
            let elem_type = l.iter().next().map(infer_arrow_type).unwrap_or(DataType::Utf8);
            DataType::List(Arc::new(Field::new("item", elem_type, true)))
        }
        Value::Bag(b) => {
            let elem_type = b.iter().next().map(infer_arrow_type).unwrap_or(DataType::Utf8);
            DataType::List(Arc::new(Field::new("item", elem_type, true)))
        }
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
                .map(|f| build_struct_child(name, f.name(), f.data_type(), rows))
                .collect::<SimWriterResult<Vec<_>>>()?;

            let struct_array =
                arrow::array::StructArray::try_new(fields.clone(), child_arrays, None)
                    .map_err(ParquetError::Arrow)?;
            Ok(Arc::new(struct_array))
        }
        DataType::List(inner_field) => build_list_column(name, inner_field.data_type(), rows),
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

fn build_list_column(
    name: &str,
    elem_type: &DataType,
    rows: &[Value],
) -> SimWriterResult<ArrayRef> {
    match elem_type {
        DataType::Int64 => {
            let mut builder = ListBuilder::new(Int64Builder::new());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::List(l)) => {
                        for item in l.iter() {
                            match item {
                                Value::Integer(i) => builder.values().append_value(*i),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    Some(Value::Bag(b)) => {
                        for item in b.iter() {
                            match item {
                                Value::Integer(i) => builder.values().append_value(*i),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    _ => builder.append(false),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Float64 => {
            let mut builder = ListBuilder::new(Float64Builder::new());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::List(l)) => {
                        for item in l.iter() {
                            match item {
                                Value::Real(f) => builder.values().append_value(f.0),
                                Value::Integer(i) => builder.values().append_value(*i as f64),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    Some(Value::Bag(b)) => {
                        for item in b.iter() {
                            match item {
                                Value::Real(f) => builder.values().append_value(f.0),
                                Value::Integer(i) => builder.values().append_value(*i as f64),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    _ => builder.append(false),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Boolean => {
            let mut builder = ListBuilder::new(BooleanBuilder::new());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::List(l)) => {
                        for item in l.iter() {
                            match item {
                                Value::Boolean(b) => builder.values().append_value(*b),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    Some(Value::Bag(b)) => {
                        for item in b.iter() {
                            match item {
                                Value::Boolean(v) => builder.values().append_value(*v),
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    _ => builder.append(false),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            let mut builder = ListBuilder::new(TimestampMillisecondBuilder::new());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::List(l)) => {
                        for item in l.iter() {
                            match item {
                                Value::DateTime(dt) => {
                                    builder.values().append_value(datetime_to_epoch_millis(dt));
                                }
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    Some(Value::Bag(b)) => {
                        for item in b.iter() {
                            match item {
                                Value::DateTime(dt) => {
                                    builder.values().append_value(datetime_to_epoch_millis(dt));
                                }
                                _ => builder.values().append_null(),
                            }
                        }
                        builder.append(true);
                    }
                    _ => builder.append(false),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        _ => {
            let mut builder = ListBuilder::new(StringBuilder::new());
            for row in rows {
                match get_field(row, name) {
                    Some(Value::List(l)) => {
                        for item in l.iter() {
                            match item {
                                Value::String(s) => builder.values().append_value(s.as_str()),
                                Value::Decimal(d) => {
                                    builder.values().append_value(d.to_string());
                                }
                                Value::Null | Value::Missing => builder.values().append_null(),
                                other => {
                                    builder.values().append_value(format!("{other:?}"));
                                }
                            }
                        }
                        builder.append(true);
                    }
                    Some(Value::Bag(b)) => {
                        for item in b.iter() {
                            match item {
                                Value::String(s) => builder.values().append_value(s.as_str()),
                                Value::Decimal(d) => {
                                    builder.values().append_value(d.to_string());
                                }
                                Value::Null | Value::Missing => builder.values().append_null(),
                                other => {
                                    builder.values().append_value(format!("{other:?}"));
                                }
                            }
                        }
                        builder.append(true);
                    }
                    _ => builder.append(false),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
    }
}

/// Builds a child column for a struct field. Recursively handles nested structs and lists.
fn build_struct_child(
    parent_name: &str,
    child_name: &str,
    data_type: &DataType,
    rows: &[Value],
) -> SimWriterResult<ArrayRef> {
    let child_values: Vec<Option<&Value>> = rows
        .iter()
        .map(|row| get_nested_field(row, parent_name, child_name))
        .collect();

    build_from_values(data_type, &child_values)
}

/// Builds an Arrow array from a slice of optional values, dispatching on the target data type.
fn build_from_values(data_type: &DataType, values: &[Option<&Value>]) -> SimWriterResult<ArrayRef> {
    match data_type {
        DataType::Boolean => {
            let mut builder = BooleanBuilder::with_capacity(values.len());
            for val in values {
                match val {
                    Some(Value::Boolean(b)) => builder.append_value(*b),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Int64 => {
            let mut builder = Int64Builder::with_capacity(values.len());
            for val in values {
                match val {
                    Some(Value::Integer(i)) => builder.append_value(*i),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Float64 => {
            let mut builder = Float64Builder::with_capacity(values.len());
            for val in values {
                match val {
                    Some(Value::Real(f)) => builder.append_value(f.0),
                    Some(Value::Integer(i)) => builder.append_value(*i as f64),
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Utf8 => {
            let mut builder = StringBuilder::new();
            for val in values {
                match val {
                    Some(Value::String(s)) => builder.append_value(s.as_str()),
                    Some(Value::Decimal(d)) => builder.append_value(d.to_string()),
                    Some(Value::Null) | Some(Value::Missing) | None => builder.append_null(),
                    Some(other) => builder.append_value(format!("{other:?}")),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            let mut builder = TimestampMillisecondBuilder::with_capacity(values.len());
            for val in values {
                match val {
                    Some(Value::DateTime(dt)) => {
                        builder.append_value(datetime_to_epoch_millis(dt));
                    }
                    _ => builder.append_null(),
                }
            }
            Ok(Arc::new(builder.finish().with_timezone("UTC")))
        }
        DataType::Struct(fields) => {
            let child_arrays: Vec<ArrayRef> = fields
                .iter()
                .map(|f| {
                    let nested: Vec<Option<&Value>> = values
                        .iter()
                        .map(|v| match v {
                            Some(Value::Tuple(t)) => {
                                let binding =
                                    BindingsName::CaseInsensitive(Cow::Borrowed(f.name().as_str()));
                                t.get(&binding)
                            }
                            _ => None,
                        })
                        .collect();
                    build_from_values(f.data_type(), &nested)
                })
                .collect::<SimWriterResult<Vec<_>>>()?;

            let struct_array =
                arrow::array::StructArray::try_new(fields.clone(), child_arrays, None)
                    .map_err(ParquetError::Arrow)?;
            Ok(Arc::new(struct_array))
        }
        DataType::List(inner_field) => {
            build_list_from_values(inner_field.data_type(), values)
        }
        _ => {
            let mut builder = StringBuilder::new();
            for val in values {
                match val {
                    Some(Value::Null) | Some(Value::Missing) | None => builder.append_null(),
                    Some(v) => builder.append_value(format!("{v:?}")),
                }
            }
            Ok(Arc::new(builder.finish()))
        }
    }
}

fn build_list_from_values(
    elem_type: &DataType,
    values: &[Option<&Value>],
) -> SimWriterResult<ArrayRef> {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for val in values {
        match val {
            Some(Value::List(l)) => {
                for item in l.iter() {
                    builder
                        .values()
                        .append_value(value_to_string_for_list(item, elem_type));
                }
                builder.append(true);
            }
            Some(Value::Bag(b)) => {
                for item in b.iter() {
                    builder
                        .values()
                        .append_value(value_to_string_for_list(item, elem_type));
                }
                builder.append(true);
            }
            _ => builder.append(false),
        }
    }
    Ok(Arc::new(builder.finish()))
}

fn value_to_string_for_list(val: &Value, _elem_type: &DataType) -> String {
    match val {
        Value::String(s) => s.as_str().to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.0.to_string(),
        Value::Decimal(d) => d.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null | Value::Missing => String::new(),
        other => format!("{other:?}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_beamline::sim::{SimBuilder, SimConfigBuilder};
    use partiql_beamline::source::SimSource;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_sampler(script_path: &str, sample_count: u64) -> DataSetSampler {
        let path = format!("../{script_path}");
        let source = SimSource::from_path(&path)
            .or_else(|_| SimSource::from_path(script_path))
            .expect("read script");
        let cfg = SimConfigBuilder::default()
            .seed(1234u64)
            .t0(time::OffsetDateTime::UNIX_EPOCH)
            .build()
            .expect("config");
        let sim = SimBuilder::from_config(cfg, source)
            .expect("sim builder")
            .build_multi_dataset()
            .expect("multi sim");

        let filter =
            partiql_beamline::sim::DataSetFilter::from_iter(std::iter::empty::<String>());
        let limit = partiql_beamline::sim::SampleLimit::Constant(sample_count);
        DataSetSampler::new(sim, filter, limit)
    }

    fn run_parquet_gen(script_path: &str, sample_count: u64) -> (TempDir, Vec<PathBuf>) {
        let sampler = make_sampler(script_path, sample_count);
        let tmp_dir = TempDir::new().expect("temp dir");

        let mut writer = SimWriterParquet {
            sampler,
            output_path: tmp_dir.path().to_str().unwrap().to_string(),
        };
        writer.write().expect("parquet write");

        let parquet_files: Vec<PathBuf> = std::fs::read_dir(tmp_dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "parquet").unwrap_or(false))
            .collect();

        (tmp_dir, parquet_files)
    }

    #[test]
    fn parquet_gen_sensors_nested() {
        let (_tmp, files) = run_parquet_gen(
            "partiql-beamline-sim/tests/scripts/sensors-nested.ion",
            5,
        );
        assert!(!files.is_empty(), "should produce at least one parquet file");

        for file in &files {
            let file_reader = File::open(file).expect("open parquet");
            let reader =
                parquet::arrow::arrow_reader::ParquetRecordBatchReader::try_new(file_reader, 1024)
                    .expect("parquet reader");
            let batches: Vec<_> = reader.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
            let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
            assert_eq!(total_rows, 5);
        }
    }

    #[test]
    fn parquet_gen_simple_transactions() {
        let (_tmp, files) = run_parquet_gen(
            "partiql-beamline-sim/tests/scripts/simple_transactions.ion",
            10,
        );
        assert_eq!(files.len(), 1);

        let file_reader = File::open(&files[0]).expect("open parquet");
        let reader =
            parquet::arrow::arrow_reader::ParquetRecordBatchReader::try_new(file_reader, 1024)
                .expect("parquet reader");
        let batches: Vec<_> = reader.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 10);

        let schema = batches[0].schema();
        assert!(schema.field_with_name("transaction_id").is_ok());
        assert!(schema.field_with_name("marketplace_id").is_ok());
        assert!(schema.field_with_name("completed").is_ok());
    }

    #[test]
    fn parquet_gen_rejects_anyof() {
        let sampler = make_sampler("partiql-beamline-sim/tests/scripts/sensors.ion", 3);
        let tmp_dir = TempDir::new().expect("temp dir");

        let mut writer = SimWriterParquet {
            sampler,
            output_path: tmp_dir.path().to_str().unwrap().to_string(),
        };

        let result = writer.write();
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("AnyOf"),
            "error should mention AnyOf: {err_msg}"
        );
    }
}
