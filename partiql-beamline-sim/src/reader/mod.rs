use crate::gen::DataGenerationError;
use ion_rs::{AnyEncoding, IonError, LazyStruct, ValueRef};
use partiql_value::Value;

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

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProcessConfigError {
    #[error("Read error: `{0}`")]
    ReadError(#[from] IonError),

    #[error("Format string error: `{0}`")]
    FormatStringError(String),

    #[error("Random Variable error: `{0}`")]
    RandomVariableError(#[from] DataGenerationError),

    #[error("No $arrival for random process")]
    NoArrival,

    #[error("No data for random process")]
    NoData,

    #[error("Error: `{0}`")]
    UnknownGenerator(String),

    #[error("Error: {0}")]
    NoConfig(String),

    #[error("Error: `{0}`")]
    UnknownParser(String),

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
        .and_then(|c| c.get("nullable").transpose())
        .transpose()?;
    let optional_config = config
        .and_then(|c| c.get("optional").transpose())
        .transpose()?;

    let null = symbol_parser.default_nullability()?;
    let opt = symbol_parser.default_optionality()?;

    let nullable = if let Some(nullable) = nullable_config {
        to_pct(nullable, symbol_parser)?
    } else {
        null
    };
    let optional = if let Some(optional) = optional_config {
        to_pct(optional, symbol_parser)?
    } else {
        opt
    };

    let mut present = 1.0 - nullable.unwrap_or(0.0) - optional.unwrap_or(0.0);

    if present < 0.0 || present > 1.0 {
        present = 0.0;
    }

    Ok(Density::new(nullable, optional, present)?)
}

pub(crate) fn to_pct(
    val: ValueRef<AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Option<f64>> {
    match val {
        ValueRef::Bool(b) => Ok(b.then(|| 0.0)),
        other => {
            let pct = to_f64(other, symbol_parser)?;
            if 0.0 <= pct && pct <= 1.0 {
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
        ValueRef::Int(i) => Ok(i.as_i64().expect("integer") as i64),
        ValueRef::Symbol(sym) => Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
            Value::Integer(i) => i as i64,
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gen::process::RandomProcesses;
    use crate::reader::process::ProcessParser;
    use crate::sim::context::SimContext;
    use crate::sim::{SimConfigBuilder, SimConfigResult, SimResult};
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
