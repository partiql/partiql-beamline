use partiql_beamline::primitives::Sample;
use partiql_beamline::sim::DataSetSampler;
use partiql_value::{DateTime, Value};
use serde_json::{Map, Number, Value as JsonValue};
use std::io::Write;

use crate::writer::{SimWriter, SimWriterError, SimWriterResult};

pub struct SimWriterJson<W: Write> {
    pub sampler: DataSetSampler,
    pub out: W,
    pub pretty: bool,
    pub coerce_unsupported: bool,
}

impl<W: Write> SimWriter for SimWriterJson<W> {
    fn write(&mut self) -> SimWriterResult<()> {
        let seed = self.sampler.seed();
        let t0_str = self
            .sampler
            .t0()
            .format(&partiql_beamline::sim::DATETIME_FORMAT)?;

        let mut root = Map::new();
        root.insert("seed".to_string(), JsonValue::Number(Number::from(seed)));
        root.insert("start".to_string(), JsonValue::String(t0_str.to_string()));

        let mut data = Map::new();
        for (dataset, samples) in self.sampler.iter_mut() {
            let dataset_name = dataset.0.clone();
            let mut rows = Vec::new();
            for sample in samples {
                let Sample { value, .. } = sample?;
                let json_val = value_to_json(&value, self.coerce_unsupported)?;
                rows.push(json_val);
            }
            data.insert(dataset_name, JsonValue::Array(rows));
        }

        root.insert("data".to_string(), JsonValue::Object(data));

        let output = if self.pretty {
            serde_json::to_string_pretty(&root)
        } else {
            serde_json::to_string(&root)
        }
        .map_err(|e| {
            SimWriterError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?;

        writeln!(self.out, "{output}")?;
        self.out.flush()?;
        Ok(())
    }
}

fn value_to_json(value: &Value, coerce: bool) -> SimWriterResult<JsonValue> {
    match value {
        Value::Null | Value::Missing => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number(Number::from(*i))),
        Value::Real(f) => {
            let v = f.0;
            if v.is_finite() {
                match Number::from_f64(v) {
                    Some(n) => Ok(JsonValue::Number(n)),
                    None => Ok(JsonValue::Null),
                }
            } else {
                Ok(JsonValue::Null)
            }
        }
        Value::Decimal(d) => {
            if coerce {
                Ok(JsonValue::String(d.to_string()))
            } else {
                Err(unsupported_error(
                    "Decimal",
                    "use --coerce-unsupported to represent as string",
                ))
            }
        }
        Value::String(s) => Ok(JsonValue::String(s.as_str().to_string())),
        Value::DateTime(dt) => {
            if coerce {
                Ok(JsonValue::String(datetime_to_string(dt)))
            } else {
                Err(unsupported_error(
                    "DateTime",
                    "use --coerce-unsupported to represent as string",
                ))
            }
        }
        Value::Blob(b) => {
            if coerce {
                Ok(JsonValue::String(format!("<blob:{} bytes>", b.len())))
            } else {
                Err(unsupported_error(
                    "Blob",
                    "use --coerce-unsupported to represent as string",
                ))
            }
        }
        Value::List(l) => {
            let items: Result<Vec<_>, _> = l.iter().map(|v| value_to_json(v, coerce)).collect();
            Ok(JsonValue::Array(items?))
        }
        Value::Bag(b) => {
            let items: Result<Vec<_>, _> = b.iter().map(|v| value_to_json(v, coerce)).collect();
            Ok(JsonValue::Array(items?))
        }
        Value::Tuple(t) => {
            let mut obj = Map::new();
            for (key, val) in t.pairs() {
                obj.insert(key.clone(), value_to_json(val, coerce)?);
            }
            Ok(JsonValue::Object(obj))
        }
    }
}

fn datetime_to_string(dt: &DateTime) -> String {
    match dt {
        DateTime::TimestampWithTz(ts) => format!("{ts}"),
        DateTime::Timestamp(ts) => format!("{ts}"),
        DateTime::Date(d) => format!("{d}"),
        DateTime::Time(t) => format!("{t}"),
        DateTime::TimeWithTz(t, tz) => format!("{t}{tz}"),
    }
}

fn unsupported_error(type_name: &str, hint: &str) -> SimWriterError {
    SimWriterError::IoError(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("JSON output does not natively support {type_name} values; {hint}"),
    ))
}
