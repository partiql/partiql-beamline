use crate::gen::constant::ConstantGenerator;
use crate::gen::distributions::Meta;
use crate::gen::text::{LoremIpsumGenerator, LoremIpsumTitleGenerator, RegexGenerator};
use crate::gen::ValueGenerator;
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::util::parse_density;
use crate::reader::{ProcessConfigError, ProcessConfigResult};
use ion_rs::{AnyEncoding, LazyStruct};
use partiql_value::Value;
use rand::Rng;

pub struct Formatter {}

pub struct RegexFormatter {}

pub struct LoremIpsum {}

pub struct LoremIpsumTitle {}

impl<R> ValueGeneratorParser<R> for Formatter
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        _rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        if let Some(config) = config {
            if let Ok(pattern) = config.get_expected("pattern") {
                let patt = pattern.expect_string()?;
                let patt = patt.text();
                let constant = Value::from(symbol_parser.format_pattern(patt)?);
                let gen = ConstantGenerator::new(meta, constant);
                return Ok(Box::new(gen));
            }
        }
        Err(ProcessConfigError::FormatStringError(
            "no 'pattern' supplied".to_string(),
        ))
    }
}

impl<R> ValueGeneratorParser<R> for RegexFormatter
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let density = parse_density(config.as_ref(), symbol_parser)?;
        if let Some(config) = config {
            if let Ok(pattern) = config.get_expected("pattern") {
                let patt = pattern.expect_string()?;
                let patt = patt.text();
                let gen = RegexGenerator::new(rng, meta, density, patt)?;
                return Ok(Box::new(gen));
            }
        }
        Err(ProcessConfigError::FormatStringError(
            "no 'pattern' supplied".to_string(),
        ))
    }
}

impl<R> ValueGeneratorParser<R> for LoremIpsum
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let density = parse_density(config.as_ref(), symbol_parser)?;
        if let Some(config) = config {
            let min = config.get_expected("min_words");
            let max = config.get_expected("max_words");

            if let (Ok(min), Ok(max)) = (min, max) {
                let min = min.expect_i64()? as u8;
                let max = max.expect_i64()? as u8;
                return Ok(Box::new(LoremIpsumGenerator::new(
                    rng, meta, density, min, max,
                )?));
            }
        }

        Err(ProcessConfigError::Other(
            "Configuration error for LoremIpsum".to_string(),
        ))
    }
}

impl<R> ValueGeneratorParser<R> for LoremIpsumTitle
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let density = parse_density(config.as_ref(), symbol_parser)?;

        Ok(Box::new(LoremIpsumTitleGenerator::new(rng, meta, density)?))
    }
}
