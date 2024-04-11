use crate::primitives::Tick;
use crate::sim::SimConfig;
use miette::Diagnostic;
use std::collections::HashMap;
use thiserror::Error;
use time::OffsetDateTime;
use unicase::UniCase;

#[derive(Debug, Error, Diagnostic)]
#[error("Sim Context Error")]
#[non_exhaustive]
pub enum SimContextError {
    #[error("Add binding error: {0}")]
    AddBindingError(String),

    #[error("Get binding error: {0}")]
    GetBindingError(String),
}

#[derive(Clone, Debug)]
pub struct SimContext {
    config: SimConfig,
    bindings: HashMap<UniCase<String>, ConstantBindingValue>,
}

impl SimContext {
    pub fn new(config: SimConfig) -> SimContext {
        SimContext {
            config,
            bindings: Default::default(),
        }
    }

    pub fn add_binding(&mut self, key: &str, value: &ConstantBindingValue) -> SimContextResult<()> {
        let key = UniCase::new(key.to_string());
        if self.bindings.contains_key(&key) {
            Err(SimContextError::AddBindingError(format!(
                "Binding with key {key} already exists"
            )))
        } else {
            self.bindings
                .insert(UniCase::new(key.to_string()), value.clone());
            Ok(())
        }
    }

    pub fn overwrite_binding(&mut self, key: &str, value: &ConstantBindingValue) {
        let key = UniCase::new(key.to_string());
        self.bindings
            .insert(UniCase::new(key.to_string()), value.clone());
    }

    pub fn get_binding(&self, key: &str) -> SimContextResult<&ConstantBindingValue> {
        Ok(self
            .bindings
            .get(&UniCase::new(key.to_string()))
            .expect("binding value"))
    }

    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    pub fn t0(&self) -> &OffsetDateTime {
        &self.config.t0
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ConstantBindingValue {
    String(String),
    UInt8(u8),
    UInt64(u64),
    Tick(Tick),
}

pub type SimContextResult<T> = Result<T, SimContextError>;
