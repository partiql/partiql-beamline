use ion_rs::element::Element;
use partiql_beamline::sim::{SimBuilder, SimConfigBuilder};
use partiql_beamline_serde::serde::{PartiqlKolliderEncoding, PartiqlShapeEncoding};
use time::OffsetDateTime;

#[test]
pub fn verify_correct_encoding() {
    let script = include_bytes!("../../partiql-beamline-sim/tests/scripts/sensors.ion");
    let t0 = OffsetDateTime::from_unix_timestamp(1712358177).expect("offset date time");
    let cfg = SimConfigBuilder::default()
        .t0(t0)
        .seed(1234)
        .build()
        .expect("sim config");

    let sim = SimBuilder::from_config(cfg.clone(), script)
        .expect("auto sim")
        .build_multi_dataset()
        .expect("auto sim");

    let shape = sim.schema();
    let shape_encoding: &dyn PartiqlShapeEncoding = &PartiqlKolliderEncoding::default();
    let actual = format!(
        "{:}",
        shape_encoding.print(&cfg, shape).expect("encoded value")
    );

    let script_as_byte_array =
        Vec::from(include_bytes!("shapes/partiql-kollider/sensors-shape.ion"));
    let expected = String::from_utf8(script_as_byte_array).expect("utf 8");

    let elm1 = Element::read_one(expected).unwrap();
    let elm2 = Element::read_one(actual).unwrap();

    let expected_struct = elm1.as_struct().expect("expected element as struct");
    let actual_struct = elm2.as_struct().expect("actual element as struct");

    println!("{:}", &expected_struct);
    println!("{:}", &actual_struct);

    assert_eq!(expected_struct, actual_struct);
}
