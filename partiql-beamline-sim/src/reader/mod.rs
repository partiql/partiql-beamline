mod env;
pub mod error;
mod process;
mod registry;
mod simple;
mod symbol;
mod text;
mod util;

/// By default, all types are nullable, but nulls are generated with 0% chance
pub(crate) const DEFAULT_NULLABILITY: Option<f64> = Some(0.0);
/// By default, no types are optional (i.e. will never be missing)
pub(crate) const DEFAULT_OPTIONALITY: Option<f64> = None;

pub use process::ProcessParser;

#[cfg(test)]
mod tests {
    use crate::gen::process::RandomDataSets;
    use crate::reader::process::ProcessParser;
    use crate::sim::SimContext;
    use crate::sim::{SimConfigBuilder, SimConfigResult};
    use crate::source::SimSource;
    use ion_rs::Element;

    #[track_caller]
    fn parse(name: &str, ion_data: &str) -> SimConfigResult<RandomDataSets> {
        let mut ion_bytes: Vec<u8> = vec![];
        Element::read_one(ion_data)
            .expect("ion decode")
            .encode_to(&mut ion_bytes, ion_rs::v1_0::Binary)
            .expect("ion encode");

        let source = SimSource::new(name, ion_bytes)?;

        let registry = Default::default();
        let seed = 5; // Chosen via roll of a fair die.

        let config = SimConfigBuilder::default().build().expect("config");
        let ctx = SimContext::new(config)?;

        let parser = ProcessParser::new(seed, registry, &ctx)?;
        Ok(parser.parse(source)?)
    }

    #[test]
    fn sensors() -> SimConfigResult<()> {
        let ion_data = include_str!("../..//tests/scripts/sensors.ion");
        let processes = parse("sensors.ion", ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn sensors_alternate() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/sensors-alternate.ion");
        let processes = parse("sensors-alternate.ion", ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn client_service() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/client-service.ion");
        let processes = parse("client-service.ion", ion_data)?;
        assert_eq!(processes.ids().len(), 14 * 2); // 14 clients; 14 instances of service

        Ok(())
    }

    #[test]
    fn transactions() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/transactions.ion");
        let processes = parse("transactions.ion", ion_data)?;
        assert_eq!(processes.ids().len(), 1);

        Ok(())
    }

    #[test]
    fn orders() -> SimConfigResult<()> {
        let ion_data = include_str!("../../tests/scripts/orders.ion");
        let processes = parse("orders.ion", ion_data)?;
        assert_eq!(processes.ids().len(), 28);

        Ok(())
    }
}
