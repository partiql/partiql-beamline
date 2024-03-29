use crate::gen::{
    bounded_bool, bounded_f64, bounded_i16, bounded_i32, bounded_i64, bounded_i8, bounded_u16,
    bounded_u32, bounded_u64, bounded_u8, simple_choose, ArrivalTime, ConstantGenerator,
    DataGenerationError, HomogeneousPoisson, RandomProcess, RandomProcesses, SimpleProcess,
    SimpleRandomData, SimpleScriptVariableKind, ValueGenerator,
};
use ion_rs::lazy::any_encoding::AnyEncoding;
use ion_rs::lazy::r#struct::LazyStruct;
use ion_rs::{IonError, IonResult, IonType, SymbolRef};
use std::collections::hash_map::Entry;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::vec;

use ion_rs::lazy::reader::LazyReader;
use ion_rs::lazy::sequence::LazyList;
use ion_rs::lazy::value::LazyValue;
use ion_rs::lazy::value_ref::ValueRef;
use partiql_value::Value;
use rand::{Rng, SeedableRng};
use rand_distr::num_traits::ToPrimitive;
use rand_pcg::Pcg64Mcg;

use crate::sim::context::SimContext;
use regex::Regex;

use thiserror::Error;
use time::Duration;

use crate::primitives::DataSetName;
use once_cell::sync::Lazy;

static FORMAT_STRING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([^\\])\{(.+?)\}").expect("FORMAT STRING REGEX"));

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
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

#[derive(Debug)]
pub enum EnvBindingValue {
    Value(Value),
    Generator(Box<dyn ValueGenerator>),
    Arrival(Box<dyn ArrivalTime>),
}

impl From<Value> for EnvBindingValue {
    fn from(value: Value) -> Self {
        EnvBindingValue::Value(value)
    }
}

impl From<Box<dyn ValueGenerator>> for EnvBindingValue {
    fn from(value: Box<dyn ValueGenerator>) -> Self {
        EnvBindingValue::Generator(value)
    }
}

impl From<Box<dyn ArrivalTime>> for EnvBindingValue {
    fn from(value: Box<dyn ArrivalTime>) -> Self {
        EnvBindingValue::Arrival(value)
    }
}

#[derive(Debug)]
pub enum SymbolType {
    VarRef(String),
    Str(String),
}

type EnvBindings = HashMap<String, EnvBindingValue>;

#[derive(Debug)]
struct Env {
    vars: Vec<(String, EnvBindings)>,
}

pub(crate) trait EnvLookup {
    fn find(&self, name: &str) -> Option<&EnvBindingValue>;

    fn get(&self, name: &str) -> ProcessConfigResult<&EnvBindingValue> {
        self.find(name)
            .ok_or_else(|| ProcessConfigError::Other(format!("Unknown variable `{name}`")))
    }
}

impl Env {
    pub fn new() -> Self {
        Self { vars: vec![] }
    }

    pub fn push<S: Into<String>>(&mut self, name: S) {
        self.vars.push((name.into(), Default::default()));
    }

    pub fn pop(&mut self) -> ProcessConfigResult<String> {
        self.vars
            .pop()
            .map(|(name, _)| name)
            .ok_or_else(|| ProcessConfigError::Fatal("Env Stack Underflow".to_string()))
    }

    fn curr(&mut self) -> ProcessConfigResult<&mut (String, EnvBindings)> {
        self.vars
            .last_mut()
            .ok_or_else(|| ProcessConfigError::Fatal("Env Stack Underflow".to_string()))
    }

    pub fn assign<S: Into<String>, V: Into<EnvBindingValue>>(
        &mut self,
        name: S,
        val: V,
    ) -> ProcessConfigResult<&mut EnvBindingValue> {
        let name = name.into();
        let (_scope, vars) = self.curr()?;
        match vars.entry(name) {
            Entry::Occupied(e) => Err(ProcessConfigError::Other(format!(
                "`{0}` bindings already exist",
                e.key()
            ))),
            Entry::Vacant(e) => Ok(e.insert(val.into())),
        }
    }
}

impl EnvLookup for Env {
    fn find(&self, name: &str) -> Option<&EnvBindingValue> {
        self.vars
            .iter()
            .rev()
            .find_map(|(_, bindings)| bindings.get(name))
    }
}

pub struct ProcessParser {
    registry: ValueGeneratorRegistry<Pcg64Mcg>,

    rng: Vec<Pcg64Mcg>,
    env: Env,
    sim_context: SimContext,

    processes: RandomProcesses,
}

impl ProcessParser {
    pub fn new(
        seed: u64,
        registry: ValueGeneratorRegistry<Pcg64Mcg>,
        ctx: &SimContext,
    ) -> ProcessConfigResult<Self> {
        Ok(Self {
            registry,

            rng: vec![Pcg64Mcg::seed_from_u64(seed)],
            env: Env::new(),
            sim_context: ctx.clone(),

            processes: Default::default(),
        })
    }

    fn curr_rng(&mut self) -> ProcessConfigResult<&mut Pcg64Mcg> {
        self.rng
            .last_mut()
            .ok_or_else(|| ProcessConfigError::Fatal("RNG underflow".to_string()))
    }

    fn child_rng(&mut self) -> ProcessConfigResult<Pcg64Mcg> {
        Pcg64Mcg::from_rng(self.curr_rng()?)
            .map_err(|_e| ProcessConfigError::Fatal("Error allocation RNG".to_string()))
    }
    fn push_scope<S: Into<String>>(&mut self, name: S) -> ProcessConfigResult<()> {
        self.env.push(name);
        let scope_rng = self.child_rng()?;
        self.rng.push(scope_rng);

        Ok(())
    }

    pub fn pop_scope(&mut self) -> ProcessConfigResult<String> {
        self.rng
            .pop()
            .ok_or_else(|| ProcessConfigError::Fatal("Rng Stack Underflow".to_string()))?;
        self.env.pop()
    }

    pub fn parse(mut self, reader: &mut LazyReader) -> ProcessConfigResult<RandomProcesses> {
        let top_lvl = reader.expect_next()?;

        let top_lvl = top_lvl.read()?;
        let config = top_lvl.expect_struct()?;
        config.annotations().expect(["rand_processes"])?;

        self.parse_scope_struct("^", config)?;

        Ok(self.processes)
    }

    fn parse_scope<S: Into<String>>(
        &mut self,
        scope_name: S,
        value: ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        let ion_type = value.ion_type();
        match value.ion_type() {
            IonType::List => self.parse_scope_list(scope_name, value.expect_list()?),
            IonType::Struct => self.parse_scope_struct(scope_name, value.expect_struct()?),
            _ => Err(ProcessConfigError::Other(format!(
                "TODO: unhandled scope type `{ion_type}`"
            ))),
        }
    }

    fn parse_scope_list<S: Into<String>>(
        &mut self,
        scope_name: S,
        l: LazyList<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        let annot = l.annotations().collect::<Result<Vec<_>, _>>()?;

        if let Some(annot) = annot.first() {
            self.parse_list_parameterized(scope_name, l, annot)?;
        } else {
            self.parse_list_unparameterized(scope_name, l)?;
        }
        Ok(())
    }

    fn parse_scope_struct<S: Into<String>>(
        &mut self,
        scope_name: S,
        s: LazyStruct<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        let annot = s.annotations().collect::<Result<Vec<_>, _>>()?;

        if annot
            .first()
            .and_then(|s| s.text())
            .filter(|s| s == &"rand_process")
            .is_some()
        {
            let process = self.parse_process(s)?;
            self.processes.add(DataSetName(scope_name.into()), process);
        } else {
            self.push_scope(scope_name)?;
            self.parse_bindings(s)?;
            self.pop_scope()?;
        }
        Ok(())
    }

    fn parse_process(
        &mut self,
        processes: LazyStruct<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn RandomProcess>> {
        let mut data = None;
        let mut arrival = None;
        for field in processes.iter() {
            let field = field?;
            let name = self.parse_symbol_type(&field.name()?)?;
            let value = field.value().read()?;

            match name {
                SymbolType::VarRef(name) => {
                    match name.as_str() {
                        "$arrival" => {
                            arrival = Some(self.parse_arrival(&value)?);
                        }
                        "$data" => {
                            data = Some(self.parse_generator(&value)?);
                        }
                        _ => {
                            // variable definition
                            let val = self.parse_binding_value(&value)?;
                            self.env.assign(name, val)?;
                        }
                    }
                }
                SymbolType::Str(name) => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unexpected scope in process `{name}`"
                    )));
                }
            }
        }

        if arrival.is_none() {
            if let Some(EnvBindingValue::Arrival(binding)) = self.env.find("$arrival") {
                arrival = Some(binding.clone());
            }
        }

        if data.is_none() {
            if let Some(EnvBindingValue::Generator(binding)) = self.env.find("$data") {
                data = Some(binding.clone());
            }
        }

        let arrival = arrival.ok_or_else(|| ProcessConfigError::NoArrival)?;
        let data = data.ok_or_else(|| ProcessConfigError::NoData)?;

        Ok(Box::new(SimpleProcess { arrival, data }))
    }

    fn parse_bindings(&mut self, processes: LazyStruct<AnyEncoding>) -> ProcessConfigResult<()> {
        for field in processes.iter() {
            let field = field?;
            let name = self.parse_symbol_type(&field.name()?)?;
            let value = field.value();
            let value = value.read()?;

            match name {
                SymbolType::VarRef(name) => {
                    // variable definition
                    let val = self.parse_binding_value(&value)?;
                    self.env.assign(name, val)?;
                }
                SymbolType::Str(name) => {
                    self.parse_scope(name, value)?;
                }
            }
        }
        Ok(())
    }

    fn parse_binding_value(
        &mut self,
        value: &ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<EnvBindingValue> {
        if let Ok(arrival) = self.parse_arrival(value) {
            Ok(arrival.into())
        } else if let Ok(generator) = self.parse_generator(value) {
            Ok(generator.into())
        } else if let Ok(immediate) = self.parse_immediate(value) {
            Ok(immediate.into())
        } else {
            Err(ProcessConfigError::Other(format!(
                "Unknown binding `{value:?}`"
            )))
        }
    }

    fn parse_list_parameterized<S: Into<String>>(
        &mut self,
        scope_name: S,
        list: LazyList<AnyEncoding>,
        parameterization: &SymbolRef,
    ) -> ProcessConfigResult<()> {
        let list_param = self.env.get(&self.parse_symbol_text(parameterization)?)?;
        let list_param = match list_param {
            EnvBindingValue::Value(v) => v.clone(),
            EnvBindingValue::Generator(g) => g.gen_value(&self.sim_context),
            EnvBindingValue::Arrival(_) => todo!("error arrival for list param"),
        };
        match self.parse_symbol_type(parameterization)? {
            SymbolType::VarRef(name) => match list_param {
                Value::Integer(n) if n > 0 => {
                    let scope_name: String = scope_name.into();
                    let index = name.replace('$', "$@");
                    for i in 0i64..n {
                        self.push_scope(&scope_name)?;
                        let index = index.as_str();
                        self.env.assign(index, Value::from(i))?;
                        for val in list.iter() {
                            self.parse_scope(&scope_name, val?.read()?)?;
                        }
                        self.pop_scope()?;
                    }
                }
                _ => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unsupported list parameterization `{list_param:?}`"
                    )));
                }
            },
            SymbolType::Str(_) => {
                todo!("list str symbol")
            }
        }
        Ok(())
    }
    fn parse_list_unparameterized<S: Into<String>>(
        &mut self,
        _scope_name: S,
        _list: LazyList<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        todo!("parse_list_unparameterized list")
    }

    fn parse_immediate(&mut self, value: &ValueRef<AnyEncoding>) -> ProcessConfigResult<Value> {
        let ion_type = value.ion_type();

        match value {
            ValueRef::Bool(b) => Ok((*b).into()),
            ValueRef::Int(i) => Ok((i.as_i64().unwrap()).into()),
            ValueRef::Float(f) => Ok((*f).into()),
            ValueRef::String(s) => Ok((s.text()).into()),
            ValueRef::SExp(sexp) => {
                let annot = sexp.annotations().collect::<IonResult<Vec<_>>>()?;
                assert_eq!(annot.len(), 1usize);
                match self.env.get(annot.first().unwrap().text().unwrap())? {
                    EnvBindingValue::Value(v) => Ok(v.clone()),
                    EnvBindingValue::Generator(g) => Ok(g.gen_value(&self.sim_context)),
                    EnvBindingValue::Arrival(_) => {
                        todo!("arrival in immediate reference")
                    }
                }
            }
            _ => Err(ProcessConfigError::Other(format!(
                "TODO: unhandled immediate type `{ion_type}`"
            ))),
        }
    }

    fn parse_duration(&self, duration: &LazyValue<AnyEncoding>) -> ProcessConfigResult<Duration> {
        let ion_type = duration.ion_type();
        let mut annot = duration.annotations();
        let duration = duration.read()?;
        let duration: i64 = match ion_type {
            IonType::Int => duration.expect_i64()?,
            IonType::Symbol => match self.parse_symbol_type(&duration.expect_symbol()?)? {
                SymbolType::VarRef(var) => match self.env.get(&var)? {
                    EnvBindingValue::Value(Value::Integer(i)) => *i,
                    EnvBindingValue::Generator(gen) => {
                        let v = gen.gen_value(&self.sim_context);
                        match v {
                            Value::Integer(i) => i,
                            other => {
                                return Err(ProcessConfigError::Other(format!(
                                    "Unexpected generated variable type in duration `{other:?}`"
                                )));
                            }
                        }
                    }
                    other => {
                        return Err(ProcessConfigError::Other(format!(
                            "Unexpected variable type in duration `{other:?}`"
                        )));
                    }
                },
                SymbolType::Str(name) => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unexpected symbol in duration `{name}`"
                    )));
                }
            },
            _ => {
                return Err(ProcessConfigError::Other(format!(
                    "TODO: unhandled duration type `{ion_type}`"
                )));
            }
        };

        match annot.next() {
            Some(Ok(duration_type)) => match self.parse_symbol_text(&duration_type)?.as_str() {
                "hours" => Ok(Duration::hours(duration)),
                "minutes" => Ok(Duration::minutes(duration)),
                "seconds" => Ok(Duration::seconds(duration)),
                "milliseconds" => Ok(Duration::milliseconds(duration)),
                "microseconds" => Ok(Duration::microseconds(duration)),
                unknown => Err(ProcessConfigError::Other(format!(
                    "Bad duration type `{unknown}`"
                ))),
            },
            _ => Err(ProcessConfigError::Other("Bad duration type".to_string())),
        }
    }

    fn parse_arrival(
        &mut self,
        value: &ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn ArrivalTime>> {
        let ion_type = value.ion_type();
        match value {
            ValueRef::Struct(strct) => {
                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;

                let kind = annot
                    .first()
                    .ok_or_else(|| ProcessConfigError::Other("No Arrival type".to_string()))?;
                let kind = self.parse_symbol_text(kind)?;
                match kind.as_str() {
                    "HomogeneousPoisson" => {
                        let interarrival =
                            self.parse_duration(&strct.find_expected("interarrival")?)?;

                        Ok(Box::new(HomogeneousPoisson::from_interarrival_time(
                            self.child_rng()?,
                            interarrival,
                        )))
                    }
                    _ => Err(ProcessConfigError::Other(format!(
                        "Unknown arrival type `{kind}`"
                    ))),
                }
            }
            _ => Err(ProcessConfigError::Other(format!(
                "TODO: unhandled arrival type `{ion_type}`"
            ))),
        }
    }

    fn parse_generator(
        &mut self,
        value: &ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        match value {
            ValueRef::Symbol(sym) => match self.parse_symbol_type(sym)? {
                SymbolType::VarRef(var) => {
                    let gen: Box<dyn ValueGenerator> = match self.env.get(&var)? {
                        EnvBindingValue::Value(v) => {
                            let constant = v.clone();
                            Box::new(ConstantGenerator { constant })
                        }
                        EnvBindingValue::Generator(g) => g.clone(),
                        EnvBindingValue::Arrival(_) => todo!("arrival generator reference"),
                    };
                    Ok(gen)
                }
                SymbolType::Str(name) => {
                    if !self.registry.has_parser(&name) {
                        Err(ProcessConfigError::UnknownGenerator(name))
                    } else {
                        let crng = self.child_rng()?;
                        let parser = self.registry.get_parser(&name).unwrap();
                        parser.parse_generator(crng, None, self, &self.sim_context)
                    }
                }
            },
            ValueRef::Struct(strct) => {
                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;
                if annot.is_empty() {
                    self.push_scope("data")?;
                    let mut kvs: HashMap<String, Box<_>> = Default::default();
                    for field in strct.iter() {
                        let field = field?;
                        let name = self.parse_symbol_text(&field.name()?)?.to_string();
                        let value_ref = field.value().read()?;
                        let value = self.parse_generator(&value_ref)?;
                        kvs.insert(name, value);
                    }
                    self.pop_scope()?;
                    Ok(Box::new(SimpleRandomData::Collection(kvs)) as Box<dyn ValueGenerator>)
                } else {
                    let kind = self.parse_symbol_type(&annot[0])?;
                    match kind {
                        SymbolType::VarRef(_) => {
                            todo!("struct varref")
                        }
                        SymbolType::Str(name) => {
                            if !self.registry.has_parser(&name) {
                                Err(ProcessConfigError::UnknownGenerator(name))
                            } else {
                                let crng = self.child_rng()?;
                                let parser = self.registry.get_parser(&name).unwrap();

                                parser.parse_generator(
                                    crng,
                                    Some(strct.clone()),
                                    self,
                                    &self.sim_context,
                                )
                            }
                        }
                    }
                }
            }
            ValueRef::List(l) => {
                let crng = self.child_rng()?;

                let mut choices = vec![];

                let annot = l.annotations().collect::<Result<Vec<_>, _>>()?;

                let n = if let Some(SymbolType::VarRef(name)) = annot
                    .first()
                    .map(|param| self.parse_symbol_type(param))
                    .transpose()?
                {
                    let list_param = self.env.get(&name)?;
                    let list_param = match list_param {
                        EnvBindingValue::Value(v) => v,
                        EnvBindingValue::Generator(_) => {
                            todo!("error generator for list param")
                        }
                        EnvBindingValue::Arrival(_) => todo!("error arrival for list param"),
                    };
                    match list_param {
                        Value::Integer(n) if *n > 0 => *n,
                        _ => {
                            return Err(ProcessConfigError::Other(format!(
                                "Unsupported list parameterization `{list_param:?}`"
                            )));
                        }
                    }
                } else {
                    1
                };

                for _i in 0..n {
                    for li in l.iter() {
                        choices.push(self.parse_immediate(&li?.read()?)?);
                    }
                }
                Ok(Box::new(simple_choose(crng, choices)?) as Box<dyn ValueGenerator>)
            }
            other => {
                let constant = self.parse_immediate(other)?;
                Ok(Box::new(ConstantGenerator { constant }))
            }
        }
    }

    fn parse_symbol_type(&self, sym: &SymbolRef) -> ProcessConfigResult<SymbolType> {
        self.parse_symbol_text(sym).map(|sym| {
            if sym.starts_with('$') {
                SymbolType::VarRef(sym.to_string())
            } else {
                SymbolType::Str(sym.to_string())
            }
        })
    }

    fn parse_symbol_text(&self, sym: &SymbolRef) -> ProcessConfigResult<String> {
        let txt = sym
            .text()
            .ok_or_else(|| ProcessConfigError::Other("Non-text symbol".to_string()))?;

        self.format_str(txt)
    }

    fn format_str(&self, pattern: &str) -> ProcessConfigResult<String> {
        let re = &FORMAT_STRING_RE;
        let txt = if re.is_match(pattern) {
            let mut new = String::with_capacity(pattern.len());
            let mut last_match = 0;
            for caps in re.captures_iter(pattern) {
                let total_match = caps.get(0).unwrap();
                let look_ahead_char = caps.get(1).unwrap().as_str();
                let variable_ref = caps.get(2).unwrap().as_str().trim();
                new.push_str(&pattern[last_match..total_match.start()]);
                new.push_str(look_ahead_char);

                let replacement = match self.env.get(variable_ref)? {
                    EnvBindingValue::Value(v) => {
                        format!("{v:?}")
                    }
                    EnvBindingValue::Generator(_) => todo!("error: generator in format string"),
                    EnvBindingValue::Arrival(_) => todo!("error: arrival in format string"),
                };
                new.push_str(&replacement);
                last_match = total_match.end();
            }
            new.push_str(&pattern[last_match..]);
            new
        } else {
            pattern.to_string()
        };

        Ok(txt)
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

        registry
            .add_parser("Format", Box::new(Formatter {}))
            .expect("static registry creation");

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

pub trait EnvSymbolParser {
    fn parse_symbol(&self, sym: &SymbolRef) -> ProcessConfigResult<Value>;

    fn format_pattern(&self, pattern: &str) -> ProcessConfigResult<String>;
}

impl EnvSymbolParser for ProcessParser {
    fn parse_symbol(&self, sym: &SymbolRef) -> ProcessConfigResult<Value> {
        match self.parse_symbol_type(sym)? {
            SymbolType::VarRef(name) => match self.env.get(&name)? {
                EnvBindingValue::Value(v) => Ok(v.clone()),
                EnvBindingValue::Generator(gen) => Ok(gen.gen_value(&self.sim_context)),
                EnvBindingValue::Arrival(_) => todo!("arrival in generator config"),
            },
            SymbolType::Str(_) => {
                todo!("bare symbol in generator config")
            }
        }
    }

    fn format_pattern(&self, pattern: &str) -> ProcessConfigResult<String> {
        self.format_str(pattern)
    }
}

pub trait ValueGeneratorParser<R>
where
    R: Rng + Sized + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        config: Option<LazyStruct<AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
        ctx: &SimContext,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;
}

struct Formatter {}

impl<R> ValueGeneratorParser<R> for Formatter
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        _rng: R,
        config: Option<LazyStruct<AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
        _ctx: &SimContext,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        if let Some(config) = config {
            if let Ok(pattern) = config.get_expected("pattern") {
                let patt = pattern.expect_string()?;
                let patt = patt.text();
                let constant = Value::from(symbol_parser.format_pattern(patt)?);
                let gen = ConstantGenerator { constant };
                return Ok(Box::new(gen));
            }
        }
        Err(ProcessConfigError::FormatStringError(
            "no 'pattern' supplied".to_string(),
        ))
    }
}

impl<R> ValueGeneratorParser<R> for SimpleScriptVariableKind
where
    R: Rng + Sized + Clone + 'static,
{
    fn parse_generator(
        &self,
        rng: R,
        config: Option<LazyStruct<AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
        ctx: &SimContext,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        if let Some(config) = config {
            fn to_float(
                val: ValueRef<AnyEncoding>,
                symbol_parser: &dyn EnvSymbolParser,
            ) -> ProcessConfigResult<f64> {
                Ok(match val {
                    ValueRef::Int(i) => i.as_i64().expect("integer") as f64,
                    ValueRef::Float(f) => f,
                    ValueRef::Symbol(sym) => match symbol_parser.parse_symbol(&sym)? {
                        Value::Integer(i) => i as f64,
                        Value::Real(f) => f.0,
                        Value::Decimal(d) => d.to_f64().unwrap(),
                        other => todo!("non-numeric float64 param {other:?}"),
                    },
                    _ => todo!("non-numeric float64 param {val:?}"),
                })
            }
            fn range(
                config: LazyStruct<AnyEncoding>,
            ) -> ProcessConfigResult<(ValueRef<AnyEncoding>, ValueRef<AnyEncoding>)> {
                Ok((config.get_expected("low")?, config.get_expected("high")?))
            }

            let gen: Box<dyn ValueGenerator> = match self {
                SimpleScriptVariableKind::String => {
                    todo!("bounded string generator")
                }
                SimpleScriptVariableKind::Tick => {
                    todo!("bounded tick generator")
                }
                SimpleScriptVariableKind::Instant => {
                    todo!("bounded Instant generator")
                }
                SimpleScriptVariableKind::UInt8 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_u8(rng, low.expect_i64()?, high.expect_i64()?)?)
                }

                SimpleScriptVariableKind::UInt16 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_u16(rng, low.expect_i64()?, high.expect_i64()?)?)
                }

                SimpleScriptVariableKind::UInt32 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_u32(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::UInt64 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_u64(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::Int8 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_i8(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::Int16 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_i16(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::Int32 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_i32(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::Int64 => {
                    let (low, high) = range(config)?;
                    Box::new(bounded_i64(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleScriptVariableKind::Float64 => {
                    let (low, high) = range(config)?;
                    let low = to_float(low, symbol_parser)?;
                    let high = to_float(high, symbol_parser)?;
                    Box::new(bounded_f64(rng, low, high)?)
                }
                SimpleScriptVariableKind::Bool => {
                    let p = to_float(config.get_expected("p")?, symbol_parser)?;
                    Box::new(bounded_bool(rng, p)?)
                }
                SimpleScriptVariableKind::UUID => {
                    todo!("bounded uuid generator")
                }
            };
            Ok(gen)
        } else {
            Ok(self.create(rng, ctx)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sim::SimConfigBuilder;
    use ion_rs::Element;

    #[track_caller]
    fn parse(ion_data: &str) -> ProcessConfigResult<RandomProcesses> {
        let ion_bytes = Element::read_one(ion_data)?.to_binary()?;
        let mut reader = LazyReader::new(&ion_bytes);

        let registry = Default::default();
        let seed = 5; // Chosen via roll of a fair die.

        let config = SimConfigBuilder::default().build().expect("config");
        let ctx = SimContext::new(config);

        let parser = ProcessParser::new(seed, registry, &ctx)?;
        parser.parse(&mut reader)
    }

    #[test]
    fn sensors() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../tests/scripts/sensors.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn sensors_alternate() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../tests/scripts/sensors-alternate.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }

    #[test]
    fn client_service() -> ProcessConfigResult<()> {
        let ion_data = include_str!("../tests/scripts/client-service.ion");
        let processes = parse(ion_data)?;
        assert_eq!(processes.ids().len(), 10 * 2); // 10 clients; 10 instances of service

        Ok(())
    }
}
