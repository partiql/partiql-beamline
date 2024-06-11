use crate::gen::arrival::{HomogeneousPoisson, OnceArrival};
use crate::gen::constant::ConstantGenerator;
use crate::gen::data::SimpleRandomData;
use crate::gen::process::{RandomProcesses, SimpleProcess};
use crate::gen::{ArrivalBoxed, ArrivalTime, RandomProcess, ValueGenerator};
use crate::primitives::{DataSetName, Tick};
use crate::reader::env::{Env, EnvBindingValue, EnvLookup};
use crate::reader::registry::ValueGeneratorRegistry;
use crate::reader::symbol::{EnvSymbolParser, SymbolType};
use crate::reader::{parse_density, ProcessConfigError, ProcessConfigResult};
use crate::sim::context::SimContext;
use ion_rs::{
    AnyEncoding, IonResult, IonType, LazyList, LazyStruct, LazyValue, Reader, SymbolRef, ValueRef,
};
use once_cell::sync::Lazy;
use partiql_value::Value;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use time::Duration;

const PROCESS_KEY_ARRIVAL: &str = "$arrival";
const PROCESS_KEY_DATA: &str = "$data";
const SCRIPT_SECTION_PROCESSES: &str = "rand_processes";
const SCRIPT_SECTION_PROCESS: &str = "rand_process";
const SCRIPT_SECTION_STATICDATA: &str = "static_data";

static FORMAT_STRING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(^|[^\\])\{(.+?)\}").expect("FORMAT STRING REGEX"));

pub struct ProcessParser {
    registry: ValueGeneratorRegistry<Pcg64Mcg>,
    rng_stack: Rc<RefCell<Vec<Pcg64Mcg>>>,
    env_stack: Env,
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
            rng_stack: Rc::new(RefCell::new(vec![Pcg64Mcg::seed_from_u64(seed)])),
            env_stack: Env::new(),
            sim_context: ctx.clone(),
            processes: Default::default(),
        })
    }

    fn curr_nullability(&self) -> ProcessConfigResult<Option<f64>> {
        Ok(self.sim_context.density().nullability())
    }

    fn curr_optionality(&self) -> ProcessConfigResult<Option<f64>> {
        Ok(self.sim_context.density().optionality())
    }

    fn curr_rng(&self) -> ProcessConfigResult<RefCell<Pcg64Mcg>> {
        let mut rng = self.rng_stack.borrow_mut();
        Ok(RefCell::new(rng.last_mut().cloned().ok_or_else(|| {
            ProcessConfigError::Fatal("RNG underflow".to_string())
        })?))
    }

    fn child_rng(&self) -> ProcessConfigResult<Pcg64Mcg> {
        let curr = self.curr_rng()?;
        Pcg64Mcg::from_rng(curr.into_inner())
            .map_err(|_e| ProcessConfigError::Fatal("Error allocation RNG".to_string()))
    }
    fn push_scope<S: Into<String>>(&mut self, name: S) -> ProcessConfigResult<()> {
        self.env_stack.push_scope(name);
        let scope_rng = self.child_rng()?;
        self.rng_stack.borrow_mut().push(scope_rng);

        Ok(())
    }

    pub fn pop_scope(&mut self) -> ProcessConfigResult<String> {
        self.rng_stack
            .borrow_mut()
            .pop()
            .ok_or_else(|| ProcessConfigError::Fatal("Rng Stack Underflow".to_string()))?;
        self.env_stack.pop()
    }

    pub fn parse(
        mut self,
        reader: &mut Reader<AnyEncoding, &[u8]>,
    ) -> ProcessConfigResult<RandomProcesses> {
        let top_lvl = reader.expect_next()?;

        let top_lvl = top_lvl.read()?;
        let config = top_lvl.expect_struct()?;
        config.annotations().expect([SCRIPT_SECTION_PROCESSES])?;

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

        if let Some(annot) = annot.first().and_then(|s| s.text()) {
            match annot {
                SCRIPT_SECTION_PROCESS => {
                    let process = self.parse_process(s)?;
                    self.processes.add(DataSetName(scope_name.into()), process);
                    return Ok(());
                }
                SCRIPT_SECTION_STATICDATA => {
                    self.push_scope(SCRIPT_SECTION_STATICDATA)?;
                    {
                        self.env_stack
                            .assign(PROCESS_KEY_ARRIVAL, OnceArrival::new(Tick(0)).boxed())?;
                        let process = self.parse_process(s)?;
                        self.processes.add(DataSetName(scope_name.into()), process);
                    }
                    self.pop_scope()?;
                    return Ok(());
                }
                SCRIPT_SECTION_PROCESSES => (), // fall through
                _ => todo!("unrecognized annotation {}", annot),
            }
        }

        self.push_scope(scope_name)?;
        self.parse_bindings(s)?;
        self.pop_scope()?;

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
                        PROCESS_KEY_ARRIVAL => {
                            arrival = Some(self.parse_arrival(&value)?);
                        }
                        PROCESS_KEY_DATA => {
                            data = Some(self.parse_generator(&value)?);
                        }
                        _ => {
                            // variable definition
                            let val = self.parse_binding_value(&value)?;
                            self.env_stack.assign(name, val)?;
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
            if let Some(EnvBindingValue::Arrival(binding)) =
                self.env_stack.find(PROCESS_KEY_ARRIVAL)
            {
                arrival = Some(binding.clone());
            }
        }

        if data.is_none() {
            if let Some(EnvBindingValue::Generator(binding)) = self.env_stack.find(PROCESS_KEY_DATA)
            {
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
                    self.env_stack.assign(name, val)?;
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
                "Unknown binding `{value:?}`" // TODO Can panic here due to https://github.com/amazon-ion/ion-rust/issues/770
            )))
        }
    }

    fn parse_list_parameterized<S: Into<String>>(
        &mut self,
        scope_name: S,
        list: LazyList<AnyEncoding>,
        parameterization: &SymbolRef,
    ) -> ProcessConfigResult<()> {
        let list_param = self
            .env_stack
            .get(&self.parse_symbol_text(parameterization)?)?;
        let list_param = match list_param {
            EnvBindingValue::Value(v) => v.clone(),
            EnvBindingValue::Generator(g) => g.present_value(&self.sim_context),
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
                        self.env_stack.assign(index, Value::from(i))?;
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
            ValueRef::Int(i) => Ok(i.as_i64().unwrap().into()),
            ValueRef::Float(f) => Ok((*f).into()),
            ValueRef::String(s) => Ok(s.text().into()),
            ValueRef::SExp(sexp) => {
                let annot = sexp.annotations().collect::<IonResult<Vec<_>>>()?;
                assert_eq!(annot.len(), 1usize);
                match self.env_stack.get(annot.first().unwrap().text().unwrap())? {
                    EnvBindingValue::Value(v) => Ok(v.clone()),
                    EnvBindingValue::Generator(g) => Ok(g.present_value(&self.sim_context)),
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
                SymbolType::VarRef(var) => match self.env_stack.get(&var)? {
                    EnvBindingValue::Value(Value::Integer(i)) => *i,
                    EnvBindingValue::Generator(gen) => {
                        let v = gen.present_value(&self.sim_context);
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
                "weeks" => Ok(Duration::weeks(duration)),
                "days" => Ok(Duration::days(duration)),
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
                    let gen: Box<dyn ValueGenerator> = match self.env_stack.get(&var)? {
                        EnvBindingValue::Value(v) => Box::new(ConstantGenerator::new(v.clone())),
                        EnvBindingValue::Generator(g) => g.clone(),
                        EnvBindingValue::Arrival(_) => todo!("arrival generator reference"),
                    };
                    Ok(gen)
                }
                SymbolType::Str(name) => {
                    if self.registry.has_parser(&name) {
                        let crng = self.child_rng()?;
                        if let Some(parser) = self.registry.get_parser(&name) {
                            parser.parse_generator(crng, None, self)
                        } else {
                            Err(ProcessConfigError::UnknownParser(name))
                        }
                    } else {
                        Err(ProcessConfigError::UnknownGenerator(name))
                    }
                }
            },
            ValueRef::Struct(strct) => {
                let density = parse_density(Some(strct), self)?;

                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;

                if annot.is_empty() {
                    self.push_scope("data")?;
                    let mut kvs: HashMap<String, Box<_>> = Default::default();
                    for field in strct.iter() {
                        let field = field?;
                        let name = self.parse_symbol_text(&field.name()?)?.to_string();
                        let value_ref = field.value().read()?;
                        let value_generator = self.parse_generator(&value_ref)?;
                        kvs.insert(name, value_generator);
                    }
                    self.pop_scope()?;
                    let rng = self.child_rng()?;
                    Ok(Box::new(SimpleRandomData::new(rng, density, kvs)?)
                        as Box<dyn ValueGenerator>)
                } else {
                    let kind = self.parse_symbol_type(&annot[0])?;
                    match kind {
                        SymbolType::VarRef(_) => {
                            todo!("struct varref")
                        }
                        SymbolType::Str(name) => {
                            if self.registry.has_parser(&name) {
                                let rng = self.child_rng()?;
                                let parser = self.registry.get_parser(&name).unwrap();
                                parser.parse_generator(rng, Some(*strct), self)
                            } else {
                                Err(ProcessConfigError::UnknownGenerator(name))
                            }
                        }
                    }
                }
            }
            ValueRef::List(lst) => Err(ProcessConfigError::Other(format!(
                "Unable to parse `{lst:?}`"
            ))),
            other => {
                let constant = self.parse_immediate(other)?;
                Ok(Box::new(ConstantGenerator::new(constant)))
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

                let replacement = match self.env_stack.get(variable_ref)? {
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

impl EnvSymbolParser for ProcessParser {
    fn parse_symbol_as_value(&self, sym: &SymbolRef) -> ProcessConfigResult<Value> {
        match self.parse_symbol_type(sym)? {
            SymbolType::VarRef(name) => match self.env_stack.get(&name)? {
                EnvBindingValue::Value(v) => Ok(v.clone()),
                EnvBindingValue::Generator(gen) => Ok(gen.present_value(&self.sim_context)),
                EnvBindingValue::Arrival(_) => todo!("arrival in generator config"),
            },
            SymbolType::Str(s) => Ok(Value::String(Box::new(s))),
        }
    }
    fn parse_symbol_as_generator(
        &self,
        sym: &SymbolRef,
        cfg: Option<LazyStruct<AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        match self.parse_symbol_type(sym)? {
            SymbolType::VarRef(name) => match self.env_stack.get(&name)? {
                EnvBindingValue::Generator(gen) => Ok(gen.clone()),
                _ => todo!(),
            },
            SymbolType::Str(name) => {
                if self.registry.has_parser(&name) {
                    let crng = self.child_rng()?;
                    if let Some(parser) = self.registry.get_parser(&name) {
                        parser.parse_generator(crng, cfg, self)
                    } else {
                        Err(ProcessConfigError::UnknownParser(name))
                    }
                } else {
                    Err(ProcessConfigError::UnknownGenerator(name))
                }
            }
        }
    }

    fn parse_symbol_as_text(&self, sym: &SymbolRef) -> ProcessConfigResult<String> {
        self.parse_symbol_text(sym)
    }

    fn format_pattern(&self, pattern: &str) -> ProcessConfigResult<String> {
        self.format_str(pattern)
    }

    fn default_nullability(&self) -> ProcessConfigResult<Option<f64>> {
        self.curr_nullability()
    }

    fn default_optionality(&self) -> ProcessConfigResult<Option<f64>> {
        self.curr_optionality()
    }
}
