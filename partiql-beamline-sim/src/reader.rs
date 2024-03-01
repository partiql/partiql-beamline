use crate::gen::{
    bounded_f64, bounded_i16, bounded_i32, bounded_i64, bounded_i8, bounded_u16, bounded_u32,
    bounded_u64, bounded_u8, simple_choose, ArrivalTime, ConstantGenerator, HomogeneousPoisson,
    Process, Processes, RandomVariableError, SimpleProcess, SimpleRandomData,
    SimpleRandomVariableKind, ValueGenerator,
};
use ion_rs::lazy::any_encoding::AnyEncoding;
use ion_rs::lazy::r#struct::LazyStruct;
use ion_rs::{IonError, IonType, SymbolRef};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::vec;

use ion_rs::lazy::reader::LazyReader;
use ion_rs::lazy::sequence::LazyList;
use ion_rs::lazy::value::LazyValue;
use ion_rs::lazy::value_ref::ValueRef;
use partiql_value::Value;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::reader::util::{SymbolParser, SymbolType};

use crate::sim::context::SimContext;
use thiserror::Error;
use time::Duration;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProcessConfigError {
    #[error("Read error: `{0}`")]
    ReadError(IonError),

    #[error("Random Variable error: `{0}`")]
    RandomVariableError(RandomVariableError),

    #[error("No $arrival for process")]
    NoArrival,

    #[error("No data for process")]
    NoData,

    #[error("Error: `{0}`")]
    Other(String),

    #[error("Fatal Internal Error: `{0}`")]
    Fatal(String),
}

impl From<IonError> for ProcessConfigError {
    fn from(err: IonError) -> Self {
        ProcessConfigError::ReadError(err)
    }
}

impl From<RandomVariableError> for ProcessConfigError {
    fn from(err: RandomVariableError) -> Self {
        ProcessConfigError::RandomVariableError(err)
    }
}

type ProcessConfigResult<T> = Result<T, ProcessConfigError>;

type EnvBindings = HashMap<String, Value>;

#[derive(Debug)]
struct Env {
    vars: Vec<(String, EnvBindings)>,
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

    pub fn assign<S: Into<String>, V: Into<Value>>(
        &mut self,
        name: S,
        val: V,
    ) -> ProcessConfigResult<&mut Value> {
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

    pub fn find(&self, name: &str) -> Option<&Value> {
        self.vars
            .iter()
            .rev()
            .find_map(|(_, bindings)| bindings.get(name))
    }

    pub fn get(&self, name: &str) -> ProcessConfigResult<&Value> {
        self.find(name)
            .ok_or_else(|| ProcessConfigError::Other(format!("Unknown variable `{name}`")))
    }
}

pub struct ProcessParser {
    rng: Vec<Pcg64Mcg>,
    env: Env,
    sim_context: SimContext,

    processes: Processes,
}

impl ProcessParser {
    pub fn new(seed: u64) -> ProcessConfigResult<Self> {
        Ok(Self {
            rng: vec![Pcg64Mcg::seed_from_u64(seed)],
            env: Env::new(),
            sim_context: Default::default(),

            processes: Default::default(),
        })
    }
}

impl ProcessParser {
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

    pub fn parse(mut self, reader: &mut LazyReader) -> ProcessConfigResult<Processes> {
        let top_lvl = reader.expect_next()?;
        let config = top_lvl.read()?.expect_struct()?;
        config.annotations().expect(["processes"])?;

        self.push_scope("^")?;
        self.parse_processes(config)?;
        self.pop_scope()?;

        Ok(self.processes)
    }

    fn parse_processes(&mut self, processes: LazyStruct<AnyEncoding>) -> ProcessConfigResult<()> {
        if processes.annotations().are(["process"])? {
            let process = self.parse_process(processes)?;
            self.processes.add(process);
        } else {
            self.parse_bindings(processes)?;
        }
        Ok(())
    }

    fn parse_process(
        &mut self,
        processes: LazyStruct<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn Process>> {
        let mut data = None;
        let mut arrival = None;
        for field in processes.iter() {
            let field = field?;
            let name = field.name()?.parse_symbol_type()?;
            let value = field.value().read()?;

            match name {
                SymbolType::VarRef(name) => {
                    match name.as_str() {
                        "$arrival" => {
                            arrival = Some(self.parse_arrival(value)?);
                        }
                        "$data" => {
                            data = Some(self.parse_generator(value)?);
                        }
                        _ => {
                            // variable definition
                            let val = self.parse_immediate(value)?;
                            self.env.assign(name, val)?;
                        }
                    }
                }
                SymbolType::Str(name) => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unexpected scope in process `{name}`"
                    )))
                }
            }
        }

        let arrival = arrival.ok_or_else(|| ProcessConfigError::NoArrival)?;
        let data = data.ok_or_else(|| ProcessConfigError::NoData)?;

        Ok(Box::new(SimpleProcess { arrival, data }))
    }

    fn parse_bindings(
        &mut self,
        processes: LazyStruct<AnyEncoding>,
    ) -> Result<(), ProcessConfigError> {
        for field in processes.iter() {
            let field = field?;
            let name = field.name()?.parse_symbol_type()?;
            let value = field.value().read()?;

            match name {
                SymbolType::VarRef(name) => {
                    // variable definition
                    let val = self.parse_immediate(value)?;
                    self.env.assign(name, val)?;
                }
                SymbolType::Str(name) => {
                    self.parse_scope(name, value)?;
                }
            }
        }
        Ok(())
    }

    fn parse_scope(
        &mut self,
        scope_name: String,
        value: ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        let ion_type = value.ion_type();
        match value.ion_type() {
            IonType::List => {
                let l = value.expect_list()?;
                let annot = l.annotations().collect::<Result<Vec<_>, _>>()?;

                if let Some(annot) = annot.first() {
                    self.parse_list_parameterized(scope_name, l, annot)?;
                } else {
                    self.parse_list_unparameterized(scope_name, l)?;
                }
            }
            IonType::Struct => {
                todo!("scope struct {:?}", &value)
            }
            _ => {
                return Err(ProcessConfigError::Other(format!(
                    "TODO: unhandled scope type `{ion_type}`"
                )))
            }
        }
        Ok(())
    }

    fn parse_list_parameterized(
        &mut self,
        scope_name: String,
        list: LazyList<AnyEncoding>,
        parameterization: &SymbolRef,
    ) -> ProcessConfigResult<()> {
        let list_param = self.env.get(parameterization.parse_symbol_text()?)?;
        match parameterization.parse_symbol_type()? {
            SymbolType::VarRef(name) => match list_param {
                Value::Integer(n) if *n > 0 => {
                    let index = name.replace('$', "$@");
                    for i in 0i64..*n {
                        self.push_scope(&scope_name)?;
                        self.env.assign(index.as_str(), Value::from(i))?;
                        for val in list.iter() {
                            self.parse_processes(val?.read()?.expect_struct()?)?;
                        }
                        self.pop_scope()?;
                    }
                }
                _ => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unsupported list parameterization `{list_param:?}`"
                    )))
                }
            },
            SymbolType::Str(_) => {
                todo!("list str symbol")
            }
        }
        Ok(())
    }
    fn parse_list_unparameterized(
        &mut self,
        _scope_name: String,
        _list: LazyList<AnyEncoding>,
    ) -> ProcessConfigResult<()> {
        todo!("parse_list_unparameterized list")
    }

    fn parse_immediate(&mut self, value: ValueRef<AnyEncoding>) -> ProcessConfigResult<Value> {
        let ion_type = value.ion_type();
        let val = match ion_type {
            IonType::Bool => value.expect_bool()?.into(),
            IonType::Int => value.expect_i64()?.into(),
            IonType::Float => value.expect_float()?.into(),
            IonType::String => value.expect_string()?.text().into(),

            IonType::Symbol | IonType::List | IonType::SExp | IonType::Struct => self
                .parse_generator(value)
                .map_err(|_e| {
                    ProcessConfigError::Other(format!("Invalid immediate type `{ion_type}`"))
                })
                .map(|r| r.gen_value(&self.sim_context))?,

            _ => {
                return Err(ProcessConfigError::Other(format!(
                    "TODO: unhandled immediate type `{ion_type}`"
                )))
            }
        };

        Ok(val)
    }

    fn parse_duration(&self, duration: LazyValue<AnyEncoding>) -> ProcessConfigResult<Duration> {
        let ion_type = duration.ion_type();
        let mut annot = duration.annotations();
        let duration = duration.read()?;
        let duration: i64 = match ion_type {
            IonType::Int => duration.expect_i64()?,
            IonType::Symbol => match duration.expect_symbol()?.parse_symbol_type()? {
                SymbolType::VarRef(var) => match self.env.get(&var)? {
                    Value::Integer(i) => *i,
                    other => {
                        return Err(ProcessConfigError::Other(format!(
                            "Unexpected variable type in duration `{other:?}`"
                        )))
                    }
                },
                SymbolType::Str(name) => {
                    return Err(ProcessConfigError::Other(format!(
                        "Unexpected symbol in duration `{name}`"
                    )))
                }
            },
            _ => {
                return Err(ProcessConfigError::Other(format!(
                    "TODO: unhandled duration type `{ion_type}`"
                )))
            }
        };

        match annot.next() {
            Some(Ok(duration_type)) => match duration_type.parse_symbol_text()? {
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
        value: ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn ArrivalTime>> {
        let ion_type = value.ion_type();
        match ion_type {
            IonType::Struct => {
                let crng = self.child_rng()?;
                let strct = value.expect_struct()?;
                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;

                let kind = annot
                    .first()
                    .ok_or_else(|| ProcessConfigError::Other("No Arrival type".to_string()))?
                    .parse_symbol_text()?;
                match kind {
                    "HomogeneousPoisson" => {
                        let interarrival =
                            self.parse_duration(strct.find_expected("interarrival")?)?;

                        Ok(Box::new(HomogeneousPoisson::from_interarrival_time(
                            crng,
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
        value: ValueRef<AnyEncoding>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let ion_type = value.ion_type();

        match ion_type {
            IonType::Symbol => match value.expect_symbol()?.parse_symbol_type()? {
                SymbolType::VarRef(var) => Ok(Box::new(ConstantGenerator {
                    constant: self.env.get(&var)?.clone(),
                }) as Box<dyn ValueGenerator>),
                SymbolType::Str(s) => {
                    let crng = self.child_rng()?;
                    SimpleRandomVariableKind::from_string(s.as_str())?.parse_generator(
                        crng,
                        None,
                        &self.sim_context,
                    )
                }
            },
            IonType::Struct => {
                let strct = value.expect_struct()?;
                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;
                if annot.is_empty() {
                    self.push_scope("data")?;
                    let mut kvs: HashMap<String, Box<_>> = Default::default();
                    for field in strct.iter() {
                        let field = field?;
                        let name = field.name()?.parse_symbol_text()?.to_string();
                        let value = self.parse_generator(field.value().read()?)?;
                        kvs.insert(name, value);
                    }
                    self.pop_scope()?;
                    Ok(Box::new(SimpleRandomData::Collection(kvs)) as Box<dyn ValueGenerator>)
                } else {
                    let crng = self.child_rng()?;
                    let kind = annot[0].parse_symbol_type()?;
                    match kind {
                        SymbolType::VarRef(_) => {
                            todo!("struct varref")
                        }
                        SymbolType::Str(s) => {
                            let kind = SimpleRandomVariableKind::from_string(&s)?;
                            kind.parse_generator(crng, Some(strct), &self.sim_context)
                        }
                    }
                }
            }
            IonType::List => {
                let crng = self.child_rng()?;
                let l = value.expect_list()?;
                let mut choices = vec![];
                for li in l.iter() {
                    choices.push(self.parse_immediate(li?.read()?)?);
                }
                Ok(Box::new(simple_choose(crng, choices)?) as Box<dyn ValueGenerator>)
            }
            _ => Err(ProcessConfigError::Other(format!(
                "TODO: unhandled generator type `{ion_type}`"
            ))),
        }
    }
}

trait ValueGeneratorParser {
    fn parse_generator<R>(
        &self,
        rng: R,
        config: Option<LazyStruct<AnyEncoding>>,
        ctx: &SimContext,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + 'static;
}

impl ValueGeneratorParser for SimpleRandomVariableKind {
    fn parse_generator<R>(
        &self,
        rng: R,
        config: Option<LazyStruct<AnyEncoding>>,
        _ctx: &SimContext,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + 'static,
    {
        if let Some(config) = config {
            let low = config.get_expected("low")?;
            let high = config.get_expected("high")?;
            let gen: Box<dyn ValueGenerator> = match self {
                SimpleRandomVariableKind::String => {
                    todo!("bounded string generator")
                }
                SimpleRandomVariableKind::UInt8 => {
                    Box::new(bounded_u8(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::UInt16 => {
                    Box::new(bounded_u16(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::UInt32 => {
                    Box::new(bounded_u32(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::UInt64 => {
                    Box::new(bounded_u64(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::Int8 => {
                    Box::new(bounded_i8(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::Int16 => {
                    Box::new(bounded_i16(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::Int32 => {
                    Box::new(bounded_i32(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::Int64 => {
                    Box::new(bounded_i64(rng, low.expect_i64()?, high.expect_i64()?)?)
                }
                SimpleRandomVariableKind::Float64 => {
                    Box::new(bounded_f64(rng, low.expect_float()?, high.expect_float()?)?)
                }
                SimpleRandomVariableKind::Bool => {
                    todo!("bounded bool generator")
                }
            };
            Ok(gen)
        } else {
            Ok(self.create(rng)?)
        }
    }
}

mod util {
    use crate::reader::{ProcessConfigError, ProcessConfigResult};

    use ion_rs::SymbolRef;

    pub enum SymbolType {
        VarRef(String),
        Str(String),
    }

    pub trait SymbolParser {
        fn parse_symbol_type(&self) -> ProcessConfigResult<SymbolType>;
        fn parse_symbol_text(&self) -> ProcessConfigResult<&str>;
    }

    impl<'a> SymbolParser for SymbolRef<'a> {
        fn parse_symbol_type(&self) -> ProcessConfigResult<SymbolType> {
            self.parse_symbol_text().map(|sym| {
                if sym.starts_with('$') {
                    SymbolType::VarRef(sym.to_string())
                } else {
                    SymbolType::Str(sym.to_string())
                }
            })
        }

        fn parse_symbol_text(&self) -> ProcessConfigResult<&str> {
            self.text()
                .ok_or_else(|| ProcessConfigError::Other("Non-text symbol".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ion_rs::Element;

    #[test]
    fn parse() -> ProcessConfigResult<()> {
        let ion_data = r#"
            processes::{
                $n: UniformU8::{ low: 2, high: 10 },
            
                sensors: $n::[
                    process::{
                        $r: Uniform::[5,10],
                        $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
                        $data: {
                            id: '$@n',
                            i8: UniformI8,
                            f: UniformF64,
                            sub: {
                                o:UniformI8,
                                f:UniformF64,
                            }
                        }
                    }
                ],
            }
        "#;
        let ion_bytes = Element::read_one(ion_data)?.to_binary()?;
        let mut reader = LazyReader::new(&ion_bytes);

        let seed = 5; // Chosen via roll of a fair die.
        let parser = ProcessParser::new(seed)?;
        let processes = parser.parse(&mut reader)?;
        assert_eq!(processes.ids().len(), 7);

        Ok(())
    }
}
