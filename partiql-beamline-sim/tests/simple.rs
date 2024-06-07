use miette::IntoDiagnostic;
use partiql_beamline::sim::{SimBuilder, SimConfigBuilder, DATETIME_FORMAT};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding::PartiqlEncodedAsIon;
use partiql_value::{tuple, BindingsName, List, Value};
use time::macros::datetime;

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

    let sim2 = SimBuilder::from_config(config2, script)
        .expect("repetition sim")
        .build_time_ordered()
        .expect("repetition sim");

    for (sample1, sample2) in std::iter::zip(sim.iter_mut(), sim2).take(100) {
        assert_eq!(sample1.expect("sample"), sample2.expect("sample2"));
    }

    let _test_final = sim.next_sample();
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
        for (sample1, sample2) in
            std::iter::zip(sim.for_dataset(id), sim2.for_dataset(id)).take(100)
        {
            assert_eq!(sample1.expect("sample"), sample2.expect("sample2"));
        }
    }
}

pub(crate) fn encode_ion_text(value: &Value) -> Result<String, IonEncodeError> {
    let mut buff = vec![];
    let mut writer = ion_rs_old::TextWriterBuilder::pretty()
        .build(&mut buff)
        .expect("pretty writer");

    let mut encoder =
        IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(PartiqlEncodedAsIon))
            .build(&mut writer)?;

    encoder.write_value(value)?;

    drop(encoder);
    drop(writer);

    Ok(String::from_utf8(buff).expect("string"))
}

pub(crate) fn decode_ion(buff: &[u8]) -> Result<Value, IonEncodeError> {
    let reader = ion_rs_old::reader::ReaderBuilder::new()
        .build(buff)
        .expect("pretty writer");

    let mut decoder =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(PartiqlEncodedAsIon))
            .build(reader)
            .expect("decoder build");

    let val = decoder
        .next()
        .expect("decoded value")
        .expect("decoded value");
    assert!(decoder.next().is_none());

    Ok(val)
}

#[track_caller]
fn verify_exemplar(script: &[u8], exemplar: &[u8]) -> miette::Result<()> {
    // thanks random.org
    let seed = 90;
    let t0 = datetime!(2024-05-24 20:39:13 UTC);
    let config = SimConfigBuilder::default()
        .t0(t0)
        .seed(seed)
        .build()
        .expect("auto config");

    let skip_count = 75;
    let sample_count = 5;
    let t0 = config.t0;
    let start = t0.format(&DATETIME_FORMAT).expect("start datetime string");
    let seed = config.seed;

    let mut sim = SimBuilder::from_config(config, script)
        .expect("auto sim")
        .build_multi_dataset()
        .expect("auto sim");

    let datasets = sim.datasets();
    let mut tp = tuple!();
    for (ds_id, ds_n) in datasets {
        let sim = sim.for_dataset(ds_id);
        let name = ds_n.0.as_str();
        let vals: Result<Vec<_>, _> = sim
            .iter_mut()
            .skip(skip_count)
            .take(sample_count)
            .map(|s| s.map(|s| s.value))
            .collect();
        tp.insert(name, List::from(vals.into_diagnostic()?).into())
    }

    let data = tuple![("seed", seed), ("start", start), ("data", tp)];
    let value = Value::from(data);
    println!("{}", encode_ion_text(&value).expect("encode"));
    let data = value.as_tuple_ref();

    let exemplar_s = std::str::from_utf8(exemplar).into_diagnostic()?;
    let exemplar = decode_ion(exemplar).into_diagnostic()?;
    let exemplar = exemplar.as_tuple_ref();

    let k_seed = BindingsName::CaseInsensitive("seed".into());
    let k_start = BindingsName::CaseInsensitive("start".into());
    let k_data = BindingsName::CaseInsensitive("data".into());
    assert_eq!(data.get(&k_seed), exemplar.get(&k_seed));
    assert_eq!(data.get(&k_start), exemplar.get(&k_start));
    assert_eq!(data.get(&k_data), exemplar.get(&k_data));

    Ok(())
}

macro_rules! script_data {
    ($file_basename:expr $(,)?) => {
        include_bytes!(concat!("scripts/", $file_basename, ".ion"))
    };
}

macro_rules! exemplar_data {
    ($file_basename:expr $(,)?) => {
        include_bytes!(concat!("data/exemplar/", $file_basename, ".ion"))
    };
}

macro_rules! test_data {
    ($file_basename:expr $(,)?) => {
        (script_data!($file_basename), exemplar_data!($file_basename))
    };
}

#[test]
fn verify_repeatable_transactions() {
    let (script, exemplar) = test_data!("transactions");
    verify_repeatable(script);
    verify_repeatable_multi(script);
    verify_exemplar(script, exemplar);
}

#[test]
fn verify_repeatable_orders() {
    let (script, exemplar) = test_data!("orders");
    verify_repeatable(script);
    verify_repeatable_multi(script);
    verify_exemplar(script, exemplar);
}

#[test]
fn verify_repeatable_sensors() {
    let (script, exemplar) = test_data!("sensors");
    verify_repeatable(script);
    verify_repeatable_multi(script);
    verify_exemplar(script, exemplar);
}

#[test]
fn verify_repeatable_sensors_alternate() {
    let (script, exemplar) = test_data!("sensors-alternate");
    verify_repeatable(script);
    verify_repeatable_multi(script);
    verify_exemplar(script, exemplar);
}

#[test]
fn verify_repeatable_client_service() {
    let (script, exemplar) = test_data!("client-service");
    verify_repeatable(script);
    verify_repeatable_multi(script);
    verify_exemplar(script, exemplar);
}
