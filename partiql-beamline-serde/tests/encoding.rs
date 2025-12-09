use ion_rs::element::writer::TextKind;
use partiql_beamline::sim::{ISim, NameAndShape, SimBuilder, SimConfigBuilder};
use partiql_beamline::source::SimSource;
use partiql_beamline_serde::beamline_json::BeamlineJsonEncoder;
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

    let source = SimSource::new("sensors.ion", script).expect("source");
    let sim = SimBuilder::from_config(cfg.clone(), source)
        .expect("auto sim")
        .build_multi_dataset()
        .expect("auto sim");

    let shape = sim.shape();

    let mut buff = vec![];
    let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Pretty)
        .build(&mut buff)
        .expect("pretty writer");
    let mut encoder = BeamlineJsonEncoder::new(&mut writer);
    encoder
        .write_datasets(&cfg, shape.clone())
        .expect("encoded value");
    drop(writer);

    let actual = String::from_utf8(buff).expect("valid utf8");
    insta::assert_snapshot!("verify_correct_encoding__shape", actual);

    let NameAndShape {
        name: _,
        shape: sensors_ty,
    } = shape.get_dataset("sensors").expect("sensors_type");

    let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
    let ddl_actual = ddl_compact.ddl(&sensors_ty).expect("ddl_output");

    insta::assert_snapshot!("verify_correct_encoding__ddl", ddl_actual);
}
