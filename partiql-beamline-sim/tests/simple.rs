use partiql_beamline::sim::{Sim, SimBuilder, SimConfigBuilder};

#[track_caller]
fn verify_repeatable(script: &[u8]) {
    let config = SimConfigBuilder::default().build().expect("auto config");

    let t0 = config.t0;
    let seed = config.seed;
    let config2 = SimConfigBuilder::default()
        .t0(t0)
        .seed(seed)
        .build()
        .expect("repetition config");

    let mut sim = SimBuilder::from_config(config, script)
        .expect("auto sim")
        .build_time_ordered()
        .expect("auto sim");
    let mut sim2 = SimBuilder::from_config(config2, script)
        .expect("repetition sim")
        .build_time_ordered()
        .expect("repetition sim");

    for _ in 0..1000 {
        assert_eq!(
            sim.next_sample().expect("sample"),
            sim2.next_sample().expect("sample2")
        );
    }
}

#[track_caller]
fn verify_repeatable_multi(script: &[u8]) {
    let config = SimConfigBuilder::default().build().expect("auto config");

    let t0 = config.t0;
    let seed = config.seed;
    let config2 = SimConfigBuilder::default()
        .t0(t0)
        .seed(seed)
        .build()
        .expect("repetition config");

    let mut sim = SimBuilder::from_config(config, script)
        .expect("auto sim")
        .build_multi_dataset()
        .expect("auto sim");
    let mut sim2 = SimBuilder::from_config(config2, script)
        .expect("repetition sim")
        .build_multi_dataset()
        .expect("repetition sim");

    let ds1 = sim.datasets();
    let ds2 = sim2.datasets();
    assert_eq!(ds1, ds2);

    for (id, _n) in ds1 {
        for _ in 0..1000 {
            assert_eq!(
                sim.next_sample(id).expect("sample"),
                sim2.next_sample(id).expect("sample2")
            );
        }
    }
}

#[test]
fn verify_repeatable_sensors() {
    let script = include_bytes!("scripts/sensors.ion");
    verify_repeatable(script);
    verify_repeatable_multi(script);
}

#[test]
fn verify_repeatable_sensors_alternate() {
    let script = include_bytes!("scripts/sensors_alternate.ion");
    verify_repeatable(script);
    verify_repeatable_multi(script);
}

#[test]
fn verify_repeatable_client_service() {
    let script = include_bytes!("scripts/client_service.ion");
    verify_repeatable(script);
    verify_repeatable_multi(script);
}
