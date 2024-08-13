use crate::gen::constant::ConstantGenerator;
use crate::gen::distributions::{Density, Meta};
use crate::gen::text::{LoremIpsumGenerator, LoremIpsumTitleGenerator, RegexGenerator};
use crate::gen::{ValueGenerator, ValueGeneratorBoxed};
use crate::reader::error::{ProcessConfigError, ProcessConfigResult, Sourceable};
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::util::{parse_density, require_key, ValueGeneratorParserImpl};
use ion_rs::{AnyEncoding, LazyStruct};
use partiql_value::Value;
use rand::Rng;

pub struct Formatter {}

pub struct RegexFormatter {}

pub struct LoremIpsum {}

pub struct LoremIpsumTitle {}

const KEY_PATTERN: &str = "pattern";
const KEY_MIN_WORDS: &str = "min_words";
const KEY_MAX_WORDS: &str = "max_words";

impl<R> ValueGeneratorParserImpl<R> for Formatter
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_with_config(
        &self,
        _rng: R,
        meta: Meta,
        _density: Density,
        config: LazyStruct<'_, AnyEncoding>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let (pattern, span) = require_key(config, KEY_PATTERN)?;
        let pattern = pattern.expect_string()?;
        let formatted = symbol_parser
            .format_pattern(pattern.text())
            .with_context(span)?;
        let constant = Value::from(formatted);
        let gen = ConstantGenerator::new(meta, constant);
        Ok(Box::new(gen))
    }

    fn possible_config_keys(&self) -> &[&'static str] {
        &[KEY_PATTERN]
    }
}

impl<R> ValueGeneratorParserImpl<R> for RegexFormatter
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
        let (pattern, span) = require_key(config, KEY_PATTERN)?;
        let pattern = pattern.expect_string()?;
        let gen = RegexGenerator::new(rng, meta, density, pattern.text())
            .map_err(ProcessConfigError::from)
            .with_context(span)?;

        Ok(Box::new(gen))
    }

    fn possible_config_keys(&self) -> &[&'static str] {
        &[KEY_PATTERN]
    }
}

impl<R> ValueGeneratorParserImpl<R> for LoremIpsum
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_with_config(
        &self,
        rng: R,
        meta: Meta,
        density: Density,
        config: LazyStruct<'_, AnyEncoding>,
        _symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let min = require_key(config, KEY_MIN_WORDS)?.0.expect_i64()? as u8;
        let max = require_key(config, KEY_MAX_WORDS)?.0.expect_i64()? as u8;

        Ok(LoremIpsumGenerator::new(rng, meta, density, min, max)?.boxed())
    }

    fn possible_config_keys(&self) -> &[&'static str] {
        &[KEY_MIN_WORDS, KEY_MAX_WORDS]
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

impl<R> ValueGeneratorParserImpl<R> for LoremIpsumTitle
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
        Ok(Box::new(LoremIpsumTitleGenerator::new(rng, meta, density)?))
    }
}
