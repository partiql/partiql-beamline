use miette::IntoDiagnostic;
use partiql_beamline::sim::{ISim, SimBuilder, SimConfigBuilder, SimResult, DATETIME_FORMAT};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding::PartiqlEncodedAsIon;
use partiql_value::{tuple, BindingsName, List, Value};
use std::collections::HashMap;
use time::macros::datetime;

#[track_caller]
fn repeatable_sims(script: &[u8]) -> SimResult<(SimBuilder, SimBuilder)> {
    let config = SimConfigBuilder::default().build()?;
    let t0 = config.t0;
    let seed = config.seed;
    let config2 = SimConfigBuilder::default().t0(t0).seed(seed).build()?;

    Ok((
        SimBuilder::from_config(config, script)?,
        SimBuilder::from_config(config2, script)?,
    ))
}

#[track_caller]
fn verify_repeatable(script: &[u8]) -> SimResult<()> {
    let (sim, sim2) = repeatable_sims(script)?;
    let mut sim = sim.build_time_ordered()?;
    let mut sim2 = sim2.build_time_ordered()?;

    for (sample1, sample2) in sim.iter_mut().zip(sim2.iter_mut()).take(100) {
        assert_eq!(sample1?, sample2?);
    }

    let _test_final = sim.next_sample();

    Ok(())
}

#[track_caller]
fn verify_repeatable_multi(script: &[u8]) -> SimResult<()> {
    let (sim, sim2) = repeatable_sims(script)?;
    let mut sim = sim.build_multi_dataset()?;
    let mut sim2 = sim2.build_multi_dataset()?;

    let ds1 = sim.datasets();
    let ds2 = sim2.datasets();
    assert_eq!(ds1, ds2);

    for (id, _n) in ds1 {
        for (sample1, sample2) in
            std::iter::zip(sim.for_dataset(id)?, sim2.for_dataset(id)?).take(100)
        {
            assert_eq!(sample1?, sample2?);
        }
    }

    Ok(())
}

pub(crate) fn encode_ion_text(value: &Value) -> Result<String, IonEncodeError> {
    let mut buff = vec![];
    let mut writer = ion_rs_old::TextWriterBuilder::pretty().build(&mut buff)?;

    let mut encoder =
        IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(PartiqlEncodedAsIon))
            .build(&mut writer)?;

    encoder.write_value(value)?;

    drop(encoder);
    drop(writer);

    Ok(String::from_utf8(buff).expect("string"))
}

pub(crate) fn decode_ion(buff: &[u8]) -> Result<Value, IonEncodeError> {
    let reader = ion_rs_old::reader::ReaderBuilder::new().build(buff)?;

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
    let seed = 90; // thanks random.org
    let t0 = datetime!(2024-05-24 20:39:13 UTC);
    let config = SimConfigBuilder::default().t0(t0).seed(seed).build()?;

    let skip_count = 75;
    let sample_count = 5;
    let t0 = config.t0;
    let start = t0.format(&DATETIME_FORMAT).expect("start datetime string");
    let seed = config.seed;

    let mut sim = SimBuilder::from_config(config, script)?.build_multi_dataset()?;

    let datasets = sim.datasets();
    let mut tp = tuple!();
    for (ds_id, ds_n) in datasets {
        let sim = sim.for_dataset(ds_id)?;
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

#[track_caller]
fn verify_exemplar_partials(
    script: &[u8],
    exemplar: &[u8],
    max_null: f64,
    max_optional: f64,
) -> miette::Result<()> {
    assert!((0.0..=1.0).contains(&max_null));
    assert!((0.0..=1.0).contains(&max_optional));
    let pcts = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
    for npct in pcts.iter().map(|pct| *pct * max_null) {
        for opct in pcts.iter().map(|pct| *pct * max_optional) {
            let scale = 1.0_f64.max(npct + opct) * 1.0;
            let nullability = npct / scale;
            let optionality = opct / scale;
            verify_exemplar_partial(script, exemplar, Some(nullability), Some(optionality))?;
        }
    }

    Ok(())
}

#[track_caller]
fn verify_exemplar_partial(
    script: &[u8],
    exemplar: &[u8],
    nullability: Option<f64>,
    optionality: Option<f64>,
) -> miette::Result<()> {
    let seed = 90; // thanks random.org
    let t0 = datetime!(2024-05-24 20:39:13 UTC);
    let config = SimConfigBuilder::default()
        .t0(t0)
        .seed(seed)
        .nullability(nullability)
        .optionality(optionality)
        .build()?;

    let skip_count = 75;
    let sample_count = 5;
    let t0 = config.t0;
    let start = t0.format(&DATETIME_FORMAT).expect("start datetime string");
    let seed = config.seed;

    let mut sim = SimBuilder::from_config(config, script)?.build_multi_dataset()?;

    let datasets = sim.datasets();
    let mut tp = tuple!();
    for (ds_id, ds_n) in datasets {
        let sim = sim.for_dataset(ds_id)?;
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
    let data = value.as_tuple_ref();

    let exemplar = decode_ion(exemplar).into_diagnostic()?;
    let exemplar = exemplar.as_tuple_ref();

    let k_seed = BindingsName::CaseInsensitive("seed".into());
    let k_start = BindingsName::CaseInsensitive("start".into());
    let k_data = BindingsName::CaseInsensitive("data".into());
    assert_eq!(data.get(&k_seed), exemplar.get(&k_seed));
    assert_eq!(data.get(&k_start), exemplar.get(&k_start));
    compare_present(data.get(&k_data).unwrap(), exemplar.get(&k_data).unwrap());

    Ok(())
}

fn compare_present(data: &Value, exemplar: &Value) {
    if data.is_absent() && !exemplar.is_absent() {
    } else {
        match (data, exemplar) {
            (Value::Tuple(data), Value::Tuple(exemplar)) => {
                let ekvs: HashMap<_, _> = exemplar.pairs().collect();
                for (k, v) in data.pairs() {
                    //println!("{}", k);
                    let ev = ekvs.get(k);
                    assert!(ev.is_some());
                    if !v.is_absent() {
                        compare_present(v, ev.unwrap());
                    }
                }
            }
            (Value::List(data), Value::List(exemplar)) => {
                assert_eq!(data.len(), exemplar.len());
                for (data, exemplar) in data.iter().zip(exemplar.iter()) {
                    compare_present(data, exemplar);
                }
            }
            (Value::Bag(data), Value::Bag(exemplar)) => {
                assert_eq!(data.len(), exemplar.len());
                for (data, exemplar) in data.iter().zip(exemplar.iter()) {
                    compare_present(data, exemplar);
                }
            }
            (data, exemplar) => {
                assert_eq!(data, exemplar);
            }
        }
    }
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
fn verify_repeatable_transactions() -> miette::Result<()> {
    let (script, _) = test_data!("transactions");
    verify_repeatable(script)?;
    verify_repeatable_multi(script)?;
    Ok(())
}

#[test]
fn verify_repeatable_orders() -> miette::Result<()> {
    let (script, _) = test_data!("orders");
    verify_repeatable(script)?;
    verify_repeatable_multi(script)?;
    Ok(())
}

#[test]
fn verify_repeatable_sensors() -> miette::Result<()> {
    let (script, _) = test_data!("sensors");
    verify_repeatable(script)?;
    verify_repeatable_multi(script)?;
    Ok(())
}

#[test]
fn verify_repeatable_sensors_alternate() -> miette::Result<()> {
    let (script, _) = test_data!("sensors-alternate");
    verify_repeatable(script)?;
    verify_repeatable_multi(script)?;
    Ok(())
}

#[test]
fn verify_repeatable_client_service() -> miette::Result<()> {
    let (script, _) = test_data!("client-service");
    verify_repeatable(script)?;
    verify_repeatable_multi(script)?;
    Ok(())
}

#[test]
fn verify_exemplar_transactions() -> miette::Result<()> {
    let (script, exemplar) = test_data!("transactions");
    verify_exemplar(script, exemplar)?;
    verify_exemplar_partials(script, exemplar, 1.0, 1.0)?;
    Ok(())
}

#[test]
fn verify_exemplar_orders() -> miette::Result<()> {
    let (script, exemplar) = test_data!("orders");
    verify_exemplar(script, exemplar)?;
    verify_exemplar_partials(script, exemplar, 1.0, 1.0)?;
    Ok(())
}

#[test]
fn verify_exemplar_sensors() -> miette::Result<()> {
    let (script, exemplar) = test_data!("sensors");
    verify_exemplar(script, exemplar)?;
    // The sensors script uses a max of 0.75 for null scripting, so cap the optional at 0.25
    verify_exemplar_partials(script, exemplar, 1.0, 0.25 - f64::EPSILON)?;
    Ok(())
}

#[test]
fn verify_exemplar_sensors_alternate() -> miette::Result<()> {
    let (script, exemplar) = test_data!("sensors-alternate");
    verify_exemplar(script, exemplar)?;
    verify_exemplar_partials(script, exemplar, 1.0, 1.0)?;
    Ok(())
}

#[test]
fn verify_exemplar_client_service() -> miette::Result<()> {
    let (script, exemplar) = test_data!("client-service");
    verify_exemplar(script, exemplar)?;
    verify_exemplar_partials(script, exemplar, 1.0, 1.0)?;
    Ok(())
}
