use partiql_beamline::sim::{Sim, SimConfigBuilder};

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

    let mut sim = Sim::from_config(config, script).expect("auto sim");
    let mut sim2 = Sim::from_config(config2, script).expect("repetition sim");

    for _ in 0..1000 {
        assert_eq!(
            sim.next_sample().expect("sample"),
            sim2.next_sample().expect("sample2")
        );
    }
}

#[test]
fn verify_repeatable_sensors() {
    let script = include_bytes!("scripts/sensors.ion");
    verify_repeatable(script);
}
