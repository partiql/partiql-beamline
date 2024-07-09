use crate::gen::ValueGenerator;
use crate::reader::ProcessConfigResult;
use ion_rs::{AnyEncoding, LazyStruct, SymbolRef};
use partiql_value::Value;

#[derive(Debug)]
pub enum SymbolType {
    VarRef(String),
    Str(String),
}

pub trait EnvSymbolParser {
    fn parse_symbol_as_value(&self, sym: &SymbolRef<'_>) -> ProcessConfigResult<Value>;

    fn parse_symbol_as_generator(
        &self,
        sym: &SymbolRef<'_>,
        cfg: Option<LazyStruct<'_, AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;

    fn parse_symbol_as_text(&self, sym: &SymbolRef<'_>) -> ProcessConfigResult<String>;

    fn format_pattern(&self, pattern: &str) -> ProcessConfigResult<String>;

    fn default_nullability(&self) -> ProcessConfigResult<Option<f64>>;

    fn default_optionality(&self) -> ProcessConfigResult<Option<f64>>;
}
