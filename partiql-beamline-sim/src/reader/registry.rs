use crate::gen::distributions::{Density, Meta};
use crate::gen::ValueGenerator;
use crate::reader::simple::SimpleScriptVariableKind;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::text::{Formatter, LoremIpsum, LoremIpsumTitle, RegexFormatter};
use crate::reader::util::BasicValueGeneratorParser;
use crate::reader::{ProcessConfigError, ProcessConfigResult};
use ion_rs::{AnyEncoding, LazyStruct};
use rand::Rng;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

pub trait ValueGeneratorParser<R>
where
    R: Rng + Sized + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;
}

pub trait ValueGeneratorParserBoxed<R>
where
    R: Rng + Sized + 'static,
{
    fn vgpboxed(self) -> Box<dyn ValueGeneratorParser<R>>
    where
        Self: Sized + 'static;
}

impl<R, T> ValueGeneratorParserBoxed<R> for T
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParser<R>,
{
    fn vgpboxed(self) -> Box<dyn ValueGeneratorParser<R>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub struct ValueGeneratorRegistry<R>
where
    R: Rng + Sized + 'static,
{
    generators: HashMap<String, Box<dyn ValueGeneratorParser<R>>>,
}

impl<R> Default for ValueGeneratorRegistry<R>
where
    R: Rng + Sized + Clone + 'static,
{
    fn default() -> Self {
        let mut registry = ValueGeneratorRegistry::new();

        for (k, v) in SimpleScriptVariableKind::named().expect("static registry creation") {
            registry
                .add_parser(&k, Box::new(v))
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
        parser: Box<dyn ValueGeneratorParser<R>>,
    ) -> ProcessConfigResult<&Box<dyn ValueGeneratorParser<R>>> {
        match self.generators.entry(name.to_string()) {
            Occupied(_) => Err(ProcessConfigError::Other(format!(
                "`{name}` parser already exists"
            ))),
            Vacant(entry) => Ok(entry.insert(parser)),
        }
    }

    pub fn get_parser(&self, name: &str) -> Option<&Box<dyn ValueGeneratorParser<R>>> {
        self.generators.get(name)
    }

    pub fn has_parser(&self, name: &str) -> bool {
        self.generators.contains_key(name)
    }
}
