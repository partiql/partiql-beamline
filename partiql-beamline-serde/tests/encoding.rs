use ion_rs::element::writer::TextKind;
use ion_rs::element::Element;
use partiql_beamline::sim::{ISim, SimBuilder, SimConfigBuilder};
use partiql_beamline_serde::kollider::PartiqlKolliderEncoder;
use partiql_beamline_serde::serde::PartiqlDataSetsEncoder;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
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

    let shape = sim.shape();

    let mut buff = vec![];
    let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
        .build(&mut buff)
        .expect("pretty writer");
    let mut encoder = PartiqlKolliderEncoder::new(&mut writer);
    encoder
        .write_datasets(&cfg, shape.clone())
        .expect("encoded value");
    drop(writer);

    let expected = include_str!("shapes/partiql-kollider/sensors-shape.ion");
    let actual = String::from_utf8(buff).expect("valid utf8");

    let elm1 = Element::read_one(expected).unwrap();
    let elm2 = Element::read_one(actual).unwrap();

    let expected_struct = elm1.as_struct().expect("expected element as struct");
    let actual_struct = elm2.as_struct().expect("actual element as struct");

    println!("{:}", &expected_struct);
    println!("{:}", &actual_struct);

    assert_eq!(expected_struct, actual_struct);

    let (_, sensors_ty) = shape.get_key_value("sensors").expect("sensors_type");

    let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
    let ddl_expected = r#""a" UNION<INT8,DECIMAL(5, 4) NOT NULL,DOUBLE,VARCHAR>,"ar1" ARRAY<DECIMAL(2, 1)>,"ar2" ARRAY<VARCHAR>,"ar3" ARRAY<DECIMAL(5, 4)>,"ar4" ARRAY<TINYINT>,"ar5" ARRAY<UNION<INT8,DECIMAL(5, 4) NOT NULL,DOUBLE,VARCHAR>>,"d" DECIMAL(2, 0) NOT NULL,"f" DOUBLE,"i8" TINYINT,"tick" INT8,"w" OPTIONAL DECIMAL(5, 4)"#;
    let ddl_actual = ddl_compact.ddl(sensors_ty).expect("ddl_output");

    println!("{:}", &ddl_expected);
    println!("{:}", &ddl_actual);

    assert_eq!(ddl_actual, ddl_expected);
}
