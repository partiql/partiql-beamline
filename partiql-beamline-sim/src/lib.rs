#![deny(rust_2018_idioms)]

pub mod sim;

pub mod ddl_generator;
pub mod ddl_to_script;
pub mod gen;
pub mod primitives;
pub mod reader;
pub mod source;

#[cfg(test)]
mod tests {
    use crate::primitives::{Sample, Tick};

    use crate::sim::{ISim, Sim, SimBuilder, SimConfigBuilder, SimResult};
    use crate::source::SimSource;

    use partiql_types::{Static, StructField};
    use partiql_value::{list, tuple, Value};
    use std::ops::Add;
    use time::macros::datetime;
    use time::Duration;

    fn sensor_script() -> &'static str {
        r#"
            rand_processes::{
              $n:UniformU8::{
                low:2,
                high:4
              },
              sensors:$n::[
                rand_process::{
                  $r:Uniform::{ choices: [2, 3] },
                  $arrival:HomogeneousPoisson::{
                    interarrival:minutes::$r
                  },
                  $weight:UniformDecimal::{
                    low:1.995,
                    high:4.9999
                  },
                  $anyof:UniformAnyOf::{ types: [
                    UUID,
                    Tick,
                    UniformDecimal::{
                      low:32.2,
                      high:43.5
                    },
                    UniformI8
                  ]},
                  $tick_array:UniformArray::{
                    min_size:2,
                    max_size:5,
                    element_type:Tick
                  },
                  $data:{
                    tick:Tick,
                    id:'$@n',
                    i8:UniformI8,
                    f:UniformF64,
                    sub:{
                      o:UniformI8,
                      f:UniformF64
                    },
                    w:$weight,
                    d:UniformDecimal::{
                      low:0.,
                      high:42.
                    },
                    variant:$anyof,
                    tick_array:$tick_array,
                    weight_array:UniformArray::{
                      min_size:3,
                      max_size:3,
                      element_type:$weight
                    },
                    decimal_array:UniformArray::{
                      min_size:2,
                      max_size:2,
                      element_type:UniformDecimal::{
                        low:1.995,
                        high:4.9999
                      }
                    }
                  }
                }
              ]
            }
        "#
    }

    fn sim_from_script(script: &str) -> SimResult<Sim> {
        let t0 = datetime!(2013-11-07 00:00:01-05:00);
        let config = SimConfigBuilder::default()
            .seed(5) // Chosen via roll of a fair die.
            .t0(t0)
            .build()?;

        let source = SimSource::new("test_data", script)?;
        let sim = SimBuilder::from_config(config, source)?.build_time_ordered()?;
        Ok(sim)
    }

    fn sensor_sim() -> Sim {
        sim_from_script(sensor_script()).expect("sim creation")
    }

    #[test]
    fn sensors() {
        let mut sim = sensor_sim();
        let t0 = sim.config().t0;

        for sample in sim.iter_mut().take(100) {
            let Sample {
                tick: Tick(t),
                value,
            } = sample.expect("next_sample");
            let time = t0.add(Duration::milliseconds(t as i64));
        }

        let sample_101 = sim.next_sample().unwrap().unwrap();
        insta::assert_debug_snapshot!("sensors_data", sample_101);
    }

    #[test]
    fn sensors_shape() {
        let sim = sensor_sim();
        let _t0 = sim.config().t0;

        let shapes = sim.shape();
        insta::assert_debug_snapshot!("sensors_shape", shapes);
    }

    #[test]
    fn sensors_shape_decimals() {
        let sim = sensor_sim();
        let _t0 = sim.config().t0;
        let datasets_mappings = sim.shape();
        let sensors_shape = datasets_mappings
            .get_shape("sensors")
            .expect("sensors shape");
        insta::assert_debug_snapshot!("sensors_shape_decimals", sensors_shape);
    }

    /// Ion structs permit duplicate field names and the reader iterates every
    /// occurrence, but the reader funnels fields through an `IndexMap<String, _>`
    /// (last-write-wins) before a `Tuple` is ever built. So a `$data` struct that
    /// declares `dup` twice generates a tuple with a single `dup` attribute
    /// carrying the last declaration's value. The deterministic single-choice
    /// generators make the surviving value assertable.
    #[test]
    fn duplicate_struct_field_names_dedupe() {
        use partiql_value::BindingsName;
        use std::borrow::Cow;

        let script = r#"
            rand_processes::{
                $r: Uniform::{ choices: [1] },
                dupes: rand_process::{
                    $arrival: HomogeneousPoisson::{ interarrival: minutes::$r },
                    $data: {
                        dup: Uniform::{ choices: [1] },
                        other: Uniform::{ choices: [2] },
                        dup: Uniform::{ choices: [9] },
                    }
                },
            }
        "#;

        let mut sim = sim_from_script(script).expect("sim creation");
        let Sample { value, .. } = sim
            .next_sample()
            .expect("a sample")
            .expect("sample without error");

        let tuple = match value {
            Value::Tuple(t) => t,
            other => panic!("expected a tuple, got {other:?}"),
        };

        // Exactly one `dup` and one `other` — the duplicate collapsed.
        assert_eq!(tuple.len(), 2);
        let dup_count = tuple.pairs().filter(|(k, _)| *k == "dup").count();
        assert_eq!(dup_count, 1, "duplicate field name was not deduplicated");

        // Last declaration (`choices: [9]`) wins over the first (`[1]`).
        let dup = tuple
            .get(&BindingsName::CaseSensitive(Cow::Borrowed("dup")))
            .expect("dup attribute");
        assert_eq!(*dup, Value::from(9));
        let other = tuple
            .get(&BindingsName::CaseSensitive(Cow::Borrowed("other")))
            .expect("other attribute");
        assert_eq!(*other, Value::from(2));
    }
}
