use crate::gen::ValueGenerator;
use crate::reader::simple::SimpleScriptVariableKind;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::text::{Formatter, LoremIpsum, LoremIpsumTitle, RegexFormatter};
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
        config: Option<LazyStruct<AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;
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

        let ps: [(&str, Box<dyn ValueGeneratorParser<R>>); 4] = [
            ("Format", Box::new(Formatter {})),
            ("Regex", Box::new(RegexFormatter {})),
            ("LoremIpsum", Box::new(LoremIpsum {})),
            ("LoremIpsumTitle", Box::new(LoremIpsumTitle {})),
        ];
        for (k, v) in ps {
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
