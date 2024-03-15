use crate::primitives::Tick;
use miette::Diagnostic;
use std::collections::HashMap;
use thiserror::Error;
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

#[derive(Default, Clone)]
pub struct SimContext {
    bindings: HashMap<UniCase<String>, BindingValue>,
}

impl SimContext {
    pub fn new() -> SimContext {
        SimContext {
            bindings: Default::default(),
        }
    }

    pub fn add_binding(&mut self, key: &str, value: &BindingValue) -> SimContextResult<()> {
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

    pub fn overwrite_binding(&mut self, key: &str, value: &BindingValue) {
        let key = UniCase::new(key.to_string());
        self.bindings
            .insert(UniCase::new(key.to_string()), value.clone());
    }

    pub fn get_binding(&self, key: &str) -> SimContextResult<&BindingValue> {
        Ok(self
            .bindings
            .get(&UniCase::new(key.to_string()))
            .expect("binding value"))
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum BindingValue {
    String(String),
    UInt8(u8),
    UInt64(u64),
    Tick(Tick),
}

pub type SimContextResult<T> = Result<T, SimContextError>;
