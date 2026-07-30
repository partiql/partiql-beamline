use base64::Engine;
use partiql_beamline::primitives::Sample;
use partiql_beamline::sim::{DataSetSampler, SimResult};
use partiql_value::{DateTime, Value};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::{Map, Number, Value as JsonValue};
use std::cell::RefCell;
use std::io::Write;

use crate::writer::{SimWriter, SimWriterError, SimWriterResult};

/// Number of top-level fields in the root document: `seed`, `start`, and `data`.
const ROOT_FIELD_COUNT: usize = 3;

pub struct SimWriterJson<W: Write> {
    pub sampler: DataSetSampler,
    pub out: W,
    pub pretty: bool,
    pub coerce_unsupported: bool,
}

impl<W: Write> SimWriter for SimWriterJson<W> {
    fn write(&mut self) -> SimWriterResult<()> {
        // Borrow the fields disjointly so the serializer (holding `out`) and the
        // sampler can be borrowed mutably at the same time.
        let SimWriterJson {
            sampler,
            out,
            pretty,
            coerce_unsupported,
        } = self;

        let seed = sampler.seed();
        let start = sampler
            .t0()
            .format(&partiql_beamline::sim::DATETIME_FORMAT)?
            .to_string();

        // Stream the document straight to `out` so we never materialize the full
        // JSON string, and only one row is held in memory at a time.
        let root = RootSer {
            seed,
            start: &start,
            coerce: *coerce_unsupported,
            sampler: RefCell::new(sampler),
        };

        if *pretty {
            let mut ser = serde_json::Serializer::pretty(&mut *out);
            root.serialize(&mut ser).map_err(map_serde_err)?;
        } else {
            let mut ser = serde_json::Serializer::new(&mut *out);
            root.serialize(&mut ser).map_err(map_serde_err)?;
        }

        writeln!(out)?;
        out.flush()?;
        Ok(())
    }
}

/// Serializes the top-level `{ seed, start, data }` document, streaming datasets
/// and rows on demand rather than buffering them.
struct RootSer<'a> {
    seed: u64,
    start: &'a str,
    coerce: bool,
    sampler: RefCell<&'a mut DataSetSampler>,
}

impl Serialize for RootSer<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(ROOT_FIELD_COUNT))?;
        map.serialize_entry("seed", &self.seed)?;
        map.serialize_entry("start", self.start)?;
        map.serialize_entry(
            "data",
            &DataSer {
                coerce: self.coerce,
                sampler: &self.sampler,
            },
        )?;
        map.end()
    }
}

/// Serializes the `data` object as `{ <dataset>: [rows...] }`, one dataset at a time.
struct DataSer<'r, 'a> {
    coerce: bool,
    sampler: &'r RefCell<&'a mut DataSetSampler>,
}

impl Serialize for DataSer<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sampler = self.sampler.borrow_mut();
        let mut map = serializer.serialize_map(None)?;
        for (dataset, samples) in sampler.iter_mut() {
            map.serialize_entry(
                dataset.0.as_str(),
                &RowsSer {
                    coerce: self.coerce,
                    rows: RefCell::new(samples),
                },
            )?;
        }
        map.end()
    }
}

/// Serializes a dataset's rows as a JSON array, writing each row as it is sampled.
struct RowsSer<I> {
    coerce: bool,
    rows: RefCell<I>,
}

impl<I> Serialize for RowsSer<I>
where
    I: Iterator<Item = SimResult<Sample>>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = self.rows.borrow_mut();
        let mut seq = serializer.serialize_seq(None)?;
        for sample in &mut *rows {
            let Sample { value, .. } = sample.map_err(serde::ser::Error::custom)?;
            let json_val = value_to_json(&value, self.coerce).map_err(serde::ser::Error::custom)?;
            seq.serialize_element(&json_val)?;
        }
        seq.end()
    }
}

fn map_serde_err(e: serde_json::Error) -> SimWriterError {
    SimWriterError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
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
                // Preserve the bytes by base64-encoding them rather than dropping
                // them behind a placeholder.
                Ok(JsonValue::String(
                    base64::engine::general_purpose::STANDARD.encode(b.as_slice()),
                ))
            } else {
                Err(unsupported_error(
                    "Blob",
                    "use --coerce-unsupported to represent as a base64 string",
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
                if matches!(val, Value::Missing) {
                    continue;
                }
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
        assert!(output.contains("\n  \"data\""));
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

    #[test]
    fn blob_coerces_to_base64() {
        let blob = Value::Blob(Box::new(b"hello".to_vec()));
        let json = value_to_json(&blob, true).expect("coerced blob");
        // "hello" base64-encoded round-trips back to the original bytes.
        assert_eq!(json, JsonValue::String("aGVsbG8=".to_string()));
    }

    #[test]
    fn blob_errors_without_coerce() {
        let blob = Value::Blob(Box::new(b"hello".to_vec()));
        let err = value_to_json(&blob, false).expect_err("blob without coerce errors");
        let msg = format!("{err}");
        assert!(msg.contains("Blob"));
        assert!(msg.contains("--coerce-unsupported"));
    }

    #[test]
    fn special_chars_in_keys_and_strings_are_escaped() {
        use partiql_value::Tuple;

        // Keys and string values with characters that must be JSON-escaped:
        // double quote, backslash, tab, newline, and braces.
        let tuple = Tuple::from([
            ("a \"b\" {c}", Value::from("he said \"hi\"\t{x}")),
            ("back\\slash", Value::from("line1\nline2")),
        ]);
        let json = value_to_json(&Value::Tuple(Box::new(tuple)), false).expect("serialize tuple");

        // Serialize through serde and confirm it round-trips to the same values,
        // which is only possible if the output is valid, properly-escaped JSON.
        let s = serde_json::to_string(&json).expect("valid JSON string");
        let reparsed: JsonValue = serde_json::from_str(&s).expect("re-parse escaped JSON");
        assert_eq!(reparsed["a \"b\" {c}"], JsonValue::String("he said \"hi\"\t{x}".into()));
        assert_eq!(reparsed["back\\slash"], JsonValue::String("line1\nline2".into()));
    }

    /// Test that JSON generated end-to-end from a script with a duplicated struct
    /// field name has a single, unique key. The dedup itself happens upstream in
    /// the sim reader (see the sim crate's `duplicate_struct_field_names_dedupe`
    /// test); this asserts the whole parse -> generate -> JSON pipeline preserves
    /// that invariant, emitting one `dup` key with the last declaration's value.
    #[test]
    fn json_gen_emits_unique_keys_for_duplicate_fields() {
        let output = run_json(
            "partiql-beamline-sim/tests/scripts/duplicate_keys.ion",
            2,
            false,
            false,
        );
        let parsed: JsonValue = serde_json::from_str(&output).expect("valid JSON");
        let rows = parsed["data"]["dupes"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        for row in rows {
            let obj = row.as_object().unwrap();
            // Only one `dup` entry survives; the sibling `other` is untouched.
            assert_eq!(obj.len(), 2);
            // The last declaration (`choices: [9]`) wins over the first (`[1]`).
            assert_eq!(obj["dup"], JsonValue::Number(9.into()));
            assert_eq!(obj["other"], JsonValue::Number(2.into()));
        }
    }

    /// Documents the writer's behavior if a duplicate-keyed `Tuple` ever reached it
    /// directly (bypassing the generator's dedup): `serde_json::Map::insert` is
    /// last-write-wins, so the entries collapse to one. This is not reachable via
    /// normal generation (see `json_gen_deduplicates_struct_field_names`) but pins
    /// the writer's contract in case a future value source changes the invariant.
    #[test]
    fn duplicate_tuple_keys_collapse_last_wins() {
        use partiql_value::Tuple;

        let tuple = Tuple::from([("k", Value::from(1)), ("k", Value::from(2))]);
        let json =
            value_to_json(&Value::Tuple(Box::new(tuple)), false).expect("serialize tuple");
        let obj = json.as_object().expect("json object");
        assert_eq!(obj.len(), 1);
        assert_eq!(obj["k"], JsonValue::Number(2.into()));
    }
}
