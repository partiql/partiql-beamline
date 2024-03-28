use ion_rs::element::writer::TextKind;
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{Sample, Tick};
use partiql_beamline::sim::{Sim, SimBuilder, SimConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::{tuple, List, Value};
use std::ops::Add;
use time::format_description::well_known::Iso8601;
use time::Duration;

pub(crate) const DATETIME_FORMAT: Iso8601 = Iso8601::DEFAULT;

pub(crate) fn execute(cfg: SimConfig, script: String, sample_count: u64) -> miette::Result<Value> {
    let mut sim = SimBuilder::from_config(cfg.clone(), script.as_bytes())
        .into_diagnostic()?
        .build_time_ordered()
        .into_diagnostic()?;

    let samples = sim.iter_mut().take(sample_count as usize).map(|result| {
        result.map(|sample| {
            //
            let Sample {
                tick: Tick(t),
                value,
            } = sample;
            let dt = cfg
                .t0
                .add(Duration::milliseconds(t as i64))
                .format(&DATETIME_FORMAT)
                .expect("datetime string");
            Value::from(tuple![("datetime", dt), ("value", value)])
        })
    });

    let seed = cfg.seed;
    let start = cfg
        .t0
        .format(&DATETIME_FORMAT)
        .expect("start datetime string");
    let samples: Result<Vec<_>, _> = samples.collect();
    let samples = samples.into_diagnostic()?;
    Ok(Value::Tuple(Box::new(tuple![
        ("seed", seed),
        ("start", start),
        ("values", Value::from(List::from(samples)))
    ])))
}

pub(crate) fn encode_ion_text(
    print_mode: IonPrintMode,
    value: &Value,
    encoding: Encoding,
) -> Result<String, IonEncodeError> {
    let mut buff = vec![];
    let mut writer = match print_mode {
        IonPrintMode::Compact => ion_rs::TextWriterBuilder::new(TextKind::Compact)
            .build(&mut buff)
            .expect("compact writer"),
        IonPrintMode::Pretty => ion_rs::TextWriterBuilder::pretty()
            .build(&mut buff)
            .expect("pretty writer"),
    };

    let mut encoder = IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(encoding))
        .build(&mut writer)?;

    encoder.write_value(value)?;

    drop(encoder);
    drop(writer);

    Ok(String::from_utf8(buff).expect("string"))
}

pub(crate) enum IonPrintMode {
    Compact,
    Pretty,
}
