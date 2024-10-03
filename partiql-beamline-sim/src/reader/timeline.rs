use crate::gen::constant::ConstantGenerator;
use crate::gen::distributions::{Density, Meta};
use crate::gen::timeline::{Date, DateImpl, Timestamp, TimestampPrecision};
use crate::gen::{ValueGenerator, ValueGeneratorBoxed};
use crate::reader::error::{ProcessConfigError, ProcessConfigResult, Sourceable};
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::util::{parse_density, require_key, ValueGeneratorParserImpl};
use ion_rs::{AnyEncoding, LazyStruct};
use partiql_value::Value;
use rand::Rng;

pub struct TimestampRead {}

const KEY_TIMEZONE: &str = "timezone";
const KEY_PRECISION: &str = "precision";

impl<R> ValueGeneratorParserImpl<R> for TimestampRead
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_with_config(
        &self,
        rng: R,
        meta: Meta,
        density: Density,
        config: LazyStruct<'_, AnyEncoding>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let (with_timezone, _) = require_key(config, KEY_TIMEZONE)?;
        let with_timezone = with_timezone.expect_bool()?;

        let (precision, span) = require_key(config, KEY_PRECISION)?;
        let precision = precision.expect_symbol()?;

        let precision = match precision.text() {
            Some("day") => TimestampPrecision::Day,
            Some("hour") => TimestampPrecision::Hour,
            Some("minute") => TimestampPrecision::Minute,
            Some("second") => TimestampPrecision::Second,
            Some("millisecond") => TimestampPrecision::Millisecond,
            Some("microsecond") => TimestampPrecision::Microsecond,
            Some(prec) => {
                todo!("Unknown precision {prec}")
            }
            None => {
                todo!()
            }
        };

        let gen = Timestamp::new(rng, meta, density, with_timezone, precision)
            .map_err(ProcessConfigError::from)
            .with_context(span)?;

        Ok(Box::new(gen))
    }

    fn possible_config_keys(&self) -> &[&'static str] {
        &[KEY_TIMEZONE, KEY_PRECISION]
    }
}

#[derive(Debug, Clone, Default)]
pub struct DateRead {}

impl<R> ValueGeneratorParserImpl<R> for DateRead
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_default(
        &self,
        rng: R,
        meta: Meta,
        density: Density,
        _symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        Ok(Box::new(Date::new(rng, meta, density)?))
    }
}
