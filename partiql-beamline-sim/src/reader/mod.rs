use crate::gen::DataGenerationError;
use ion_rs::IonError;

use thiserror::Error;

mod env;
mod process;
mod registry;
mod simple;
mod symbol;
mod text;

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

    #[error("Error: `{0}`")]
    UnknownParser(String),

    #[error("Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

pub use process::ProcessParser;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gen::process::RandomProcesses;
    use crate::reader::process::ProcessParser;
    use crate::sim::context::SimContext;
    use crate::sim::SimConfigBuilder;
    use ion_rs::{AnyEncoding, Element, Reader};

    #[track_caller]
    fn parse(ion_data: &str) -> ProcessConfigResult<RandomProcesses> {
        let mut ion_bytes: Vec<u8> = vec![];
        Element::read_one(ion_data)?.encode_to(&mut ion_bytes, ion_rs::v1_0::Binary)?;
        let mut reader = Reader::new(AnyEncoding, ion_bytes.as_slice())?;

        let registry = Default::default();
        let seed = 5; // Chosen via roll of a fair die.

        let config = SimConfigBuilder::default().build().expect("config");
        let ctx = SimContext::new(config);

        let parser = ProcessParser::new(seed, registry, &ctx)?;
        parser.parse(&mut reader)
    }

    #[test]
    fn sensors() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../..//tests/scripts/sensors.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn sensors_alternate() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/sensors-alternate.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn client_service() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/client-service.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 14 * 2); // 14 clients; 14 instances of service

        Ok(())
    }
}
