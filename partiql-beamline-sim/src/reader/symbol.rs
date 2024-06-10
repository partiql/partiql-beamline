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
    fn parse_symbol_as_value(&self, sym: &SymbolRef) -> ProcessConfigResult<Value>;

    fn parse_symbol_as_generator(
        &self,
        sym: &SymbolRef,
        cfg: Option<LazyStruct<AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>;

    fn parse_symbol_as_text(&self, sym: &SymbolRef) -> ProcessConfigResult<String>;

    fn format_pattern(&self, pattern: &str) -> ProcessConfigResult<String>;
}
