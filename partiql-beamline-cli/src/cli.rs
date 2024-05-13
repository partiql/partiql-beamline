use ion_rs::element::writer::TextKind;
use miette::IntoDiagnostic;
use partiql_beamline::primitives::{DataSetId, DataSetName};
use partiql_beamline::sim::{MultiSim, SimBuilder, SimConfig, SimResult, DATETIME_FORMAT};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::{tuple, List, Value};

pub(crate) fn execute(
    cfg: SimConfig,
    script: String,
    sample_count: u64,
    datasets: Vec<String>,
) -> miette::Result<Value> {
    let seed = cfg.seed;
    let start = cfg
        .t0
        .format(&DATETIME_FORMAT)
        .expect("start datetime string");

    let mut sim = SimBuilder::from_config(cfg.clone(), script.clone().as_bytes())
        .expect("multi sim")
        .build_multi_dataset()
        .expect("multi sim with datasets");

    let tp = get_values(&mut sim, sample_count as usize, datasets).expect("tuple value");

    Ok(Value::Tuple(Box::new(tuple![
        ("seed", seed),
        ("start", start),
        ("data", tp)
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

pub fn get_values(
    sim: &mut MultiSim,
    sample_count: usize,
    datasets: Vec<String>,
) -> miette::Result<Value> {
    let datasets: Vec<(DataSetId, DataSetName)> = if datasets.is_empty() {
        sim.datasets()
    } else {
        datasets
            .into_iter()
            .map(DataSetName)
            .filter_map(|ds| sim.get_dataset_id(&ds).map(|id| (id, ds)))
            .collect()
    };

    let mut tp = tuple!();
    for (ds_id, ds_n) in datasets {
        let sim = sim.for_dataset(ds_id);
        let name = ds_n.0.as_str();
        let vals: Result<Vec<_>, _> = sim
            .iter_mut()
            .take(sample_count)
            .map(|s| s.map(|s| s.value))
            .collect();
        tp.insert(name, List::from(vals.into_diagnostic()?).into())
    }

    Ok(Value::from(tp))
}

pub(crate) fn get_multi_sim(cfg: &SimConfig, script: &str) -> SimResult<MultiSim> {
    SimBuilder::from_config(cfg.clone(), script.as_bytes())
        .expect("auto sim")
        .build_multi_dataset()
}
