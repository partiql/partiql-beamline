//#![deny(rust_2018_idioms)]

pub mod sim;

pub mod gen;
pub mod primitives;
pub mod reader;

#[cfg(test)]
mod tests {
    use crate::primitives::{Sample, Tick};
    use crate::sim::{Sim, SimBuilder, SimConfigBuilder};
    use partiql_types::{StructField, TypeKind};
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
                        $weight: UniformDecimal::{ low: 1.995, high: 4.9999 },
                        $data: {
                            tick: Tick,
                            id: '$@n',
                            i8: UniformI8,
                            f: UniformF64,
                            w: $weight,
                            d: UniformDecimal::{ low: 0d0, high: 4.2d1 },
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
            ("i8", 107),
            ("f", -60.91545829837153),
            ("d", 33.),
            ("w", 4.0864),
            ("sub", tuple!(("f", 36.91801269407276), ("o", 49)))
        );

        let sample_101 = sim.next_sample().unwrap().unwrap();

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
        if let TypeKind::Bag(bag) = sensors_shape.kind() {
            if let TypeKind::Struct(struct_type) = bag.element_type().kind() {
                let fields: Vec<StructField> = struct_type
                    .fields()
                    .into_iter()
                    .filter(|f| f.name() == "w" || f.name() == "d")
                    .collect();
                assert_eq!(fields.len(), 2);
                fields.into_iter().for_each(|f| {
                    if f.name() == "w" {
                        assert_eq!(f.ty().kind(), &TypeKind::DecimalP(5, 4));
                    } else {
                        assert_eq!(f.ty().kind(), &TypeKind::DecimalP(2, 0));
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
