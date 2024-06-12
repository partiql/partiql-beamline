use crate::gen::DataGenerationError;
use ion_rs::{AnyEncoding, IonError, LazyStruct, ValueRef};
use partiql_value::Value;
use std::collections::HashSet;

use thiserror::Error;

use ion_rs_old::external::bigdecimal::ToPrimitive;
mod env;
mod process;
mod registry;
mod simple;
mod symbol;
mod text;

/// By default, all types are nullable, but nulls are generated with 0% chance
pub(crate) const DEFAULT_NULLABILITY: Option<f64> = Some(0.0);
/// By default, no types are optional (i.e. will never be missing)
pub(crate) const DEFAULT_OPTIONALITY: Option<f64> = None;

pub(crate) const CONFIG_KEY_NULLABLE: &'static str = "nullable";
pub(crate) const CONFIG_KEY_OPTIONAL: &'static str = "optional";
pub(crate) const CONFIG_KEYS_DENSITY: [&'static str; 2] =
    [CONFIG_KEY_NULLABLE, CONFIG_KEY_OPTIONAL];

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProcessConfigError {
    #[error("Read error: `{0}`")]
    ReadError(#[from] IonError),

    #[error("Format string error: `{0}`")]
    FormatStringError(String),

    #[error("Nullability/Optionality config error: `{0}`")]
    DensityError(String),

    #[error("Random Variable error: `{0}`")]
    RandomVariableError(#[from] DataGenerationError),

    #[error("No $arrival for random process")]
    NoArrival,

    #[error("No data for random process")]
    NoData,

    #[error("Duplicate Configuration key: `{0}`")]
    ConfigDuplicateKey(String),

    #[error("Unexpected Configuration key: `{0}`")]
    ConfigInvalidKey(String),

    #[error("Error: `{0}`")]
    UnknownGenerator(String),

    #[error("Error: `{0}`")]
    UnknownArrival(String),

    #[error("Error: `{0}`")]
    UnknownImmediate(String),

    #[error("Error: {0}")]
    NoConfig(String),

    #[error("Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

use crate::gen::distributions::Density;
use crate::reader::symbol::EnvSymbolParser;
pub use process::ProcessParser;

pub(crate) fn parse_density(
    config: Option<&LazyStruct<AnyEncoding>>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Density> {
    let nullable_config = config
        .and_then(|c| c.get(CONFIG_KEY_NULLABLE).transpose())
        .transpose()?;
    let optional_config = config
        .and_then(|c| c.get(CONFIG_KEY_OPTIONAL).transpose())
        .transpose()?;

    let null = symbol_parser.default_nullability()?;
    let opt = symbol_parser.default_optionality()?;

    let (nullable, nullable_default) = if let Some(nullable) = nullable_config {
        (to_pct(nullable, symbol_parser)?, false)
    } else {
        (null, true)
    };
    let (optional, optional_default) = if let Some(optional) = optional_config {
        (to_pct(optional, symbol_parser)?, false)
    } else {
        (opt, true)
    };

    let pct_absent = nullable.unwrap_or(0.0) + optional.unwrap_or(0.0);
    let present = 1.0 - pct_absent;

    if !(0.0..=1.0).contains(&present) {
        let fmt_msg = |name: &str, val: Option<f64>, default: bool| {
            format!(
                "{}: `{}`{}",
                name,
                val.unwrap_or(0.0),
                if default {
                    "(from simulation default)"
                } else {
                    ""
                }
            )
        };
        let nullability = fmt_msg(CONFIG_KEY_NULLABLE, nullable, nullable_default);
        let optionality = fmt_msg(CONFIG_KEY_OPTIONAL, optional, optional_default);
        let msg = format!(
            "Combined Nullability and Optionality Percents must be between 0.0 and 1.0; {}; {}.",
            nullability, optionality
        );
        Err(ProcessConfigError::DensityError(msg))?
    } else {
        Ok(Density::new(nullable, optional, present)?)
    }
}

pub(crate) fn to_pct(
    val: ValueRef<AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Option<f64>> {
    match val {
        ValueRef::Bool(b) => Ok(b.then_some(0.0)),
        other => {
            let pct = to_f64(other, symbol_parser)?;
            if (0.0..=1.0).contains(&pct) {
                Ok(Some(pct))
            } else {
                Err(ProcessConfigError::Other(
                    "Percent must be between 0.0 and 1.0".to_string(),
                ))?
            }
        }
    }
}

pub(crate) fn to_i64(
    val: ValueRef<AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<i64> {
    match val {
        ValueRef::Int(i) => Ok(i.as_i64().expect("integer")),
        ValueRef::Symbol(sym) => Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
            Value::Integer(i) => i,
            other => todo!("non-numeric i64 param {other:?}"),
        }),
        _ => todo!("non-numeric float64 param {val:?}"),
    }
}
pub(crate) fn to_f64(
    val: ValueRef<AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<f64> {
    match val {
        ValueRef::Int(i) => Ok(i.as_i64().expect("integer") as f64),
        ValueRef::Float(f) => Ok(f),
        ValueRef::Decimal(d) => match d.to_string().parse::<f64>() {
            Ok(f) => Ok(f),
            Err(e) => Err(ProcessConfigError::Other(e.to_string())),
        },
        ValueRef::Symbol(sym) => Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
            Value::Integer(i) => i as f64,
            Value::Real(f) => f.0,
            Value::Decimal(d) => d.to_f64().unwrap(),
            other => todo!("non-numeric float64 param {other:?}"),
        }),
        _ => todo!("non-numeric float64 param {val:?}"),
    }
}

pub(crate) fn validate_config_keys<const N: usize>(
    config: Option<LazyStruct<AnyEncoding>>,
    allowed_keys: [&[&'static str]; N],
) -> ProcessConfigResult<()> {
    let keys: HashSet<&'static str> = allowed_keys.into_iter().flatten().map(|s| *s).collect();
    validate_config_keyset(config, keys)
}
pub(crate) fn validate_config_keyset(
    config: Option<LazyStruct<AnyEncoding>>,
    allowed_keys: HashSet<&'static str>,
) -> ProcessConfigResult<()> {
    let mut seen: HashSet<String> = HashSet::default();
    if let Some(config) = config {
        for s in config.iter() {
            let s = s?;
            let name = s.name()?;
            let name = name.expect_text()?;
            if !allowed_keys.contains(name) {
                return Err(ProcessConfigError::ConfigInvalidKey(name.to_string()));
            }
            if !seen.insert(name.to_string()) {
                return Err(ProcessConfigError::ConfigDuplicateKey(name.to_string()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::gen::process::RandomProcesses;
    use crate::reader::process::ProcessParser;
    use crate::sim::context::SimContext;
    use crate::sim::{SimConfigBuilder, SimConfigResult};
    use ion_rs::{AnyEncoding, Element, Reader};

    #[track_caller]
    fn parse(ion_data: &str) -> SimConfigResult<RandomProcesses> {
        let mut ion_bytes: Vec<u8> = vec![];
        Element::read_one(ion_data)?.encode_to(&mut ion_bytes, ion_rs::v1_0::Binary)?;
        let mut reader = Reader::new(AnyEncoding, ion_bytes.as_slice())?;

        let registry = Default::default();
        let seed = 5; // Chosen via roll of a fair die.

        let config = SimConfigBuilder::default().build().expect("config");
        let ctx = SimContext::new(config)?;

        let parser = ProcessParser::new(seed, registry, &ctx)?;
        Ok(parser.parse(&mut reader)?)
    }

    #[test]
    fn sensors() -> SimConfigResult<()> {
        let ion_data = include_str!("../..//tests/scripts/sensors.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn sensors_alternate() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/sensors-alternate.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn client_service() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/client-service.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 14 * 2); // 14 clients; 14 instances of service

        Ok(())
    }
}
