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

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_beamline::sim::{DataSetFilter, SampleLimit, SimBuilder, SimConfigBuilder};
    use partiql_beamline::source::SimSource;

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

        let filter = DataSetFilter::from_iter(std::iter::empty::<String>());
        let limit = SampleLimit::Constant(sample_count);
        DataSetSampler::new(sim, filter, limit)
    }

    fn run_json(script_path: &str, sample_count: u64, pretty: bool, coerce: bool) -> String {
        let sampler = make_sampler(script_path, sample_count);
        let mut buf = Vec::new();
        let mut writer = SimWriterJson {
            sampler,
            out: &mut buf,
            pretty,
            coerce_unsupported: coerce,
        };
        writer.write().expect("json write");
        String::from_utf8(buf).expect("valid utf-8")
    }

    #[test]
    fn json_gen_nested_struct() {
        let output = run_json(
            "partiql-beamline-sim/tests/scripts/sensors-nested.ion",
            3,
            false,
            false,
        );
        let parsed: JsonValue = serde_json::from_str(&output).expect("valid JSON");
        assert!(parsed["seed"].is_number());
        assert!(parsed["start"].is_string());
        assert!(parsed["data"]["sensors"].is_array());
        let rows = parsed["data"]["sensors"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0]["sub"].is_object());
        assert!(rows[0]["sub"]["o"].is_number());
    }

    #[test]
    fn json_gen_pretty_is_valid() {
        let output = run_json(
            "partiql-beamline-sim/tests/scripts/sensors-nested.ion",
            2,
            true,
            false,
        );
        let parsed: JsonValue = serde_json::from_str(&output).expect("valid pretty JSON");
        assert!(parsed["data"]["sensors"].is_array());
        assert!(output.contains('\n'));
    }

    #[test]
    fn json_gen_errors_on_datetime_without_coerce() {
        let sampler = make_sampler(
            "partiql-beamline-sim/tests/scripts/simple_transactions.ion",
            2,
        );
        let mut buf = Vec::new();
        let mut writer = SimWriterJson {
            sampler,
            out: &mut buf,
            pretty: false,
            coerce_unsupported: false,
        };
        let result = writer.write();
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("DateTime"));
        assert!(err_msg.contains("--coerce-unsupported"));
    }

    #[test]
    fn json_gen_coerces_datetime_and_decimal() {
        let output = run_json(
            "partiql-beamline-sim/tests/scripts/simple_transactions.ion",
            3,
            false,
            true,
        );
        let parsed: JsonValue = serde_json::from_str(&output).expect("valid JSON");
        let rows = parsed["data"]["test_data"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        // DateTime coerced to string
        assert!(rows[0]["created_at"].is_string());
        // Decimal coerced to string
        assert!(rows[0]["price"].is_string());
        // Native types preserved
        assert!(rows[0]["marketplace_id"].is_number());
        assert!(rows[0]["completed"].is_boolean());
        assert!(rows[0]["transaction_id"].is_string());
    }
}
