use crate::gen::{ArrivalTime, ValueGenerator};
use crate::reader::error::{NotKnownError, OtherError, ProcessConfigResult};
use itertools::Itertools;
use partiql_value::Value;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

type EnvBindings = HashMap<String, EnvBindingValue>;

#[derive(Debug)]
pub struct Env {
    vars: Vec<(String, EnvBindings)>,
}

pub(crate) trait EnvLookup {
    fn find(&self, name: &str) -> Option<&EnvBindingValue>;

    fn get(&self, name: &str) -> ProcessConfigResult<&EnvBindingValue> {
        self.find(name)
            .ok_or_else(|| NotKnownError::Variable(name.to_string()).into())
    }
}

impl Env {
    pub fn new() -> Self {
        Self { vars: vec![] }
    }

    pub fn push_scope<S: Into<String>>(&mut self, name: S) {
        self.vars.push((name.into(), Default::default()));
    }

    pub fn pop(&mut self) -> ProcessConfigResult<String> {
        self.vars
            .pop()
            .map(|(name, _)| name)
            .ok_or_else(|| OtherError::Fatal("Env Stack Underflow".to_string()).into())
    }

    fn curr(&mut self) -> ProcessConfigResult<&mut (String, EnvBindings)> {
        self.vars
            .last_mut()
            .ok_or_else(|| OtherError::Fatal("Env Stack Underflow".to_string()).into())
    }

    pub fn assign<S: Into<String>, V: Into<EnvBindingValue>>(
        &mut self,
        name: S,
        val: V,
    ) -> ProcessConfigResult<&mut EnvBindingValue> {
        let name = name.into();
        let (_scope, vars) = self.curr()?;
        match vars.entry(name) {
            Entry::Occupied(e) => {
                Err(OtherError::Other(format!("`{0}` bindings already exist", e.key())).into())
            }
            Entry::Vacant(e) => Ok(e.insert(val.into())),
        }
    }

    pub fn curr_path(&self, name: Option<&str>) -> ProcessConfigResult<String> {
        Ok(self
            .vars
            .iter()
            .map(|(n, _e)| n.as_str())
            .chain(name)
            .join("."))
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
