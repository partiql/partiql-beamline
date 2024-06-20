//#![deny(rust_2018_idioms)]

pub mod sim;

pub mod gen;
pub mod primitives;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::primitives::{Sample, Tick};

    use crate::reader::ProcessConfigError;
    use crate::sim::{
        ISim, Sim, SimBuilder, SimConfigBuilder, SimConfigError, SimError, SimResult,
    };
    use assert_matches::assert_matches;
    use partiql_types::{StaticTypeVariant, StructField};
    use partiql_value::{list, tuple, Value};
    use std::ops::Add;
    use time::macros::datetime;
    use time::Duration;

    fn format_simple_script(config: &'static str) -> String {
        format!(
            r#"
            rand_processes::{{
              $n:UniformU8::{{
                low:2,
                high:4
              }},
              sensors:$n::[
                rand_process::{{
                  $r:Uniform::{{ choices: [2, 3] }},
                  $arrival:HomogeneousPoisson::{{
                    interarrival:minutes::$r
                  }},
                  $data:{{
                    i8:UniformI8::{config},
                  }}
                }}
              ]
            }}
        "#
        )
    }

    #[test]
    fn config_error_invalid_key_low() {
        assert_matches!(
            sim_from_script(&format_simple_script("{low_val: 5, high_val: 6}")),
            Err(SimError::ConfigError(SimConfigError::ProcessConfig(
                ProcessConfigError::ConfigInvalidKey(msg),
            ))) if msg == "low_val"
        );
    }

    #[test]
    fn config_error_invalid_key_high() {
        assert_matches!(
            sim_from_script(&format_simple_script("{low: 5, high_val: 6}")),
            Err(SimError::ConfigError(SimConfigError::ProcessConfig(
                ProcessConfigError::ConfigInvalidKey(msg),
            ))) if msg == "high_val"
        );
    }

    #[test]
    fn config_error_duplicate_key_high() {
        assert_matches!(
            sim_from_script(&format_simple_script("{low: 5, high: 6, high: 9}")),
            Err(SimError::ConfigError(SimConfigError::ProcessConfig(
                ProcessConfigError::ConfigDuplicateKey(msg),
            ))) if msg == "high"
        );
    }

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

        let sim = SimBuilder::from_config(config, script.as_bytes())?.build_time_ordered()?;
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
            println!("[{time}] : {value:?}");
        }

        let expected = tuple!(
            ("tick", 3960823),
            ("id", 1),
            ("i8", 113),
            ("f", 81.65777117537067),
            ("d", 35),
            ("sub", tuple!(("f", 119.14662526189088), ("o", -23))),
            ("w", 2.5932),
            ("variant", "a06a5f14-b622-43ef-a9ae-1cd5fdacc829"),
            ("tick_array", list!(3960823, 3960823)),
            ("weight_array", list!(4.2710, 3.9625, 4.9851)),
            ("decimal_array", list!(3.6373, 2.5320)),
        );

        let sample_101 = sim.next_sample().unwrap().unwrap();
        println!("{:?}", sample_101);
        assert_eq!(Value::from(expected), sample_101.value);
    }

    #[test]
    fn sensors_shape() {
        let sim = sensor_sim();
        let _t0 = sim.config().t0;

        dbg!(sim.shape());
    }

    #[test]
    fn sensors_shape_decimals() {
        let sim = sensor_sim();
        let _t0 = sim.config().t0;
        let datasets_mappings = sim.shape();
        let sensors_shape = datasets_mappings.get("sensors").expect("sensors shape");
        assert!(sensors_shape.is_bag());

        let stype = sensors_shape.expect_static().expect("static type");

        if let StaticTypeVariant::Bag(bag) = stype.ty() {
            if let Ok(struct_type) = bag.element_type().expect_struct() {
                let fields: Vec<StructField> = struct_type
                    .fields()
                    .into_iter()
                    .filter(|f| f.name() == "w" || f.name() == "d")
                    .collect();
                assert_eq!(fields.len(), 2);
                fields.into_iter().for_each(|f| {
                    let stype = f.ty().expect_static().expect("struct type");
                    if f.name() == "w" {
                        assert_eq!(stype.ty(), StaticTypeVariant::DecimalP(5, 4));
                    } else {
                        assert_eq!(stype.ty(), StaticTypeVariant::DecimalP(2, 0));
                    }
                });
            } else {
                panic!("not a struct type for sensors shape element")
            }
        } else {
            panic!("not a bag type for sensors shape")
        }
    }
}
