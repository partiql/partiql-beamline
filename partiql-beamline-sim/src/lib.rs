pub mod sim;

pub mod gen;
pub mod primitives;
pub mod reader;

#[cfg(test)]
mod tests {

    use crate::primitives::{Sample, Tick};
    use crate::sim::{Sim, SimConfigBuilder};
    use partiql_value::{tuple, Value};
    use std::ops::Add;
    use time::macros::datetime;
    use time::Duration;

    #[test]
    fn sensors() {
        let script = r#"
            processes::{
                $n: UniformU8::{ low: 2, high: 10 },
            
                sensors: $n::[
                    process::{
                        $r: Uniform::[5,10],
                        $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
                        $data: {
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
        "#;

        let t0 = datetime!(2013-11-07 00:00:01-05:00);
        let config = SimConfigBuilder::default()
            .seed(5) // Chosen via roll of a fair die.
            .t0(t0)
            .build()
            .expect("config");

        let mut sim = Sim::from_config(config, script.as_bytes()).expect("sim");

        for _ in 0..100 {
            let Sample {
                tick: Tick(t),
                value,
            } = sim.next_sample().expect("next_sample").unwrap();
            let time = t0.add(Duration::milliseconds(t as i64));
            println!("[{time}] : {value:?}");
        }

        let expected = tuple!(
            ("id", 1),
            ("i8", -69),
            ("f", -42.80960192722216),
            ("sub", tuple!(("f", -6.082133258561541), ("o", -62)))
        );
        let expected = Value::from(expected);
        let sample_101 = sim.next_sample().unwrap().unwrap();

        assert_eq!(expected, sample_101.value);
    }
}
