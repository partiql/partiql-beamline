use ion_rs::element::writer::TextKind;
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{Sample, Tick};
use partiql_beamline::sim::{Sim, SimConfig};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::{tuple, List, Value};
use std::ops::Add;
use time::format_description::well_known::Iso8601;
use time::Duration;

pub(crate) const DATETIME_FORMAT: Iso8601 = Iso8601::DEFAULT;

pub(crate) fn execute(cfg: SimConfig, script: String, sample_count: u64) -> miette::Result<Value> {
    let mut sim = Sim::from_config(cfg.clone(), script.as_bytes()).into_diagnostic()?;
    let mut values = vec![];

    for _ in 0..sample_count {
        if let Ok(Some(Sample {
            tick: Tick(t),
            value,
        })) = sim.next_sample()
        {
            let dt = cfg
                .t0
                .add(Duration::milliseconds(t as i64))
                .format(&DATETIME_FORMAT)
                .expect("datetime string");
            values.push(partiql_value::Value::Tuple(Box::new(tuple![
                ("datetime", format!("{dt}")),
                ("value", value)
            ])));
        }
    }

    let seed = cfg.seed;
    let start = cfg
        .t0
        .format(&DATETIME_FORMAT)
        .expect("start datetime string");

    Ok(Value::Tuple(Box::new(tuple![
        ("seed", seed),
        ("start", start),
        ("values", Value::List(Box::new(List::from(values))))
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
