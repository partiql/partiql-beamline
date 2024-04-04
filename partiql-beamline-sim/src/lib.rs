//#![deny(rust_2018_idioms)]

pub mod sim;

pub mod gen;
pub mod primitives;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::primitives::{Sample, Tick};
    use crate::sim::{Sim, SimBuilder, SimConfigBuilder};
    use partiql_value::{tuple, Value};
    use std::ops::Add;
    use time::macros::datetime;
    use time::Duration;

    fn sensor_script() -> &'static str {
        r#"
            rand_processes::{
                $n: UniformU8::{ low: 2, high: 4 },

                sensors: $n::[
                    rand_process::{
                        $r: Uniform::[5,10],
                        $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
                        $data: {
                            tick: Tick,
                            id: '$@n',
                            i8: UniformI8,
                            f: UniformF64,
                            sub: {
                                o:UniformI8,
                                f:UniformF64,
                            }
                        }
                    }
                ],
            }
        "#
    }

    fn sensor_sim() -> Sim {
        let script = sensor_script();

        let t0 = datetime!(2013-11-07 00:00:01-05:00);
        let config = SimConfigBuilder::default()
            .seed(5) // Chosen via roll of a fair die.
            .t0(t0)
            .build()
            .expect("config");

        let sim = SimBuilder::from_config(config, script.as_bytes())
            .expect("sim")
            .build_time_ordered()
            .expect("sim");
        sim
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
            ("tick", 16238568),
            ("id", 1),
            ("i8", -17),
            ("f", 92.25197734368527),
            ("sub", tuple!(("f", -37.74527277209394), ("o", 67)))
        );

        let sample_101 = sim.next_sample().unwrap().unwrap();

        assert_eq!(Value::from(expected), sample_101.value);
    }

    #[test]
    fn sensors_schema() {
        let sim = sensor_sim();
        let _t0 = sim.config().t0;

        dbg!(sim.schema());
    }
}
