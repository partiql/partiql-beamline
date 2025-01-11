use crate::gen::distributions::Meta;
use crate::gen::ValueGenerator;
use crate::reader::error::{ProcessConfigError, ProcessConfigResult};
use crate::reader::simple::{
    ExpF64Read, LogNormalF64Read, NormalF64Read, SimpleScriptVariableKind, WeibullF64Read,
};
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::text::{Formatter, LoremIpsum, LoremIpsumTitle, RegexFormatter};
use crate::reader::timeline::{DateRead, TimestampRead};
use crate::reader::util::BasicValueGeneratorParser;
use ion_rs::{AnyEncoding, LazyStruct};
use rand::Rng;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Arc;

pub trait ValueGeneratorParser<R>
where
    R: Rng + Sized + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &mut dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;
}

pub trait ValueGeneratorParserBoxed<R>
where
    R: Rng + Sized + 'static,
{
    fn vgpboxed(self) -> Arc<dyn ValueGeneratorParser<R>>
    where
        Self: Sized + 'static;
}

impl<R, T> ValueGeneratorParserBoxed<R> for T
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParser<R>,
{
    fn vgpboxed(self) -> Arc<dyn ValueGeneratorParser<R>>
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }
}

pub struct ValueGeneratorRegistry<R>
where
    R: Rng + Sized + 'static,
{
    generators: HashMap<String, Arc<dyn ValueGeneratorParser<R>>>,
}

impl<R> Default for ValueGeneratorRegistry<R>
where
    R: Rng + Sized + Clone + 'static,
{
    fn default() -> Self {
        let mut registry = ValueGeneratorRegistry::new();

        for (k, v) in SimpleScriptVariableKind::named().expect("static registry creation") {
            registry
                .add_parser(&k, Arc::new(v))
                .expect("static registry creation");
        }

        for (k, v) in [
            (
                "Format",
                BasicValueGeneratorParser::from(Formatter {}).vgpboxed(),
            ),
            (
                "Regex",
                BasicValueGeneratorParser::from(RegexFormatter {}).vgpboxed(),
            ),
            (
                "LoremIpsum",
                BasicValueGeneratorParser::from(LoremIpsum {}).vgpboxed(),
            ),
            (
                "LoremIpsumTitle",
                BasicValueGeneratorParser::from(LoremIpsumTitle {}).vgpboxed(),
            ),
            (
                "Timestamp",
                BasicValueGeneratorParser::from(TimestampRead {}).vgpboxed(),
            ),
            (
                "Date",
                BasicValueGeneratorParser::from(DateRead {}).vgpboxed(),
            ),
            (
                "NormalF64",
                BasicValueGeneratorParser::from(NormalF64Read {}).vgpboxed(),
            ),
            (
                "LogNormalF64",
                BasicValueGeneratorParser::from(LogNormalF64Read {}).vgpboxed(),
            ),
            (
                "ExpF64",
                BasicValueGeneratorParser::from(ExpF64Read {}).vgpboxed(),
            ),
            (
                "WeibullF64",
                BasicValueGeneratorParser::from(WeibullF64Read {}).vgpboxed(),
            ),
        ] {
            registry.add_parser(k, v).expect("static registry creation");
        }

        registry
    }
}

impl<R> ValueGeneratorRegistry<R>
where
    R: Rng + Sized + 'static,
{
    fn new() -> Self {
        Self {
            generators: Default::default(),
        }
    }

    pub fn add_parser(
        &mut self,
        name: &str,
        parser: Arc<dyn ValueGeneratorParser<R>>,
    ) -> ProcessConfigResult<&Arc<dyn ValueGeneratorParser<R>>> {
        match self.generators.entry(name.to_string()) {
            Occupied(_) => Err(ProcessConfigError::other(format!(
                "`{name}` parser already exists"
            ))),
            Vacant(entry) => Ok(entry.insert(parser)),
        }
    }

    pub fn get_parser(&self, name: &str) -> Option<Arc<dyn ValueGeneratorParser<R>>> {
        self.generators.get(name).cloned()
    }

    pub fn has_parser(&self, name: &str) -> bool {
        self.generators.contains_key(name)
    }
}
