use crate::gen::distributions::Density;
use crate::gen::simple::{
    bounded_array, bounded_bool, bounded_choose, bounded_decimal, bounded_f64, bounded_i16,
    bounded_i32, bounded_i64, bounded_i8, bounded_u16, bounded_u32, bounded_u64, bounded_u8,
    bounded_union, simple_uuid,
};
use crate::gen::timeline::{InstantGenerator, TickGenerator};
use crate::gen::{DataGenerationError, DataGenerationResult, ValueGenerator};
use crate::reader;
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::{
    to_f64, to_i64, validate_config_keys, ProcessConfigError, ProcessConfigResult,
    CONFIG_KEYS_DENSITY,
};
use ion_rs::{AnyEncoding, LazyStruct, SymbolRef, ValueRef};
use partiql_value::Value;
use rand::Rng;
use std::collections::{HashMap, HashSet};

pub(crate) const DEFAULT_UINT8: (i64, i64) = (u8::MIN as i64, u8::MAX as i64);
pub(crate) const DEFAULT_UINT16: (i64, i64) = (u16::MIN as i64, u16::MAX as i64);
pub(crate) const DEFAULT_UINT32: (i64, i64) = (u32::MIN as i64, u32::MAX as i64);
pub(crate) const DEFAULT_UINT64: (i64, i64) = (u64::MIN as i64, i64::MAX);
pub(crate) const DEFAULT_INT8: (i64, i64) = (i8::MIN as i64, i8::MAX as i64);
pub(crate) const DEFAULT_INT16: (i64, i64) = (i16::MIN as i64, i16::MAX as i64);
pub(crate) const DEFAULT_INT32: (i64, i64) = (i32::MIN as i64, i32::MAX as i64);
pub(crate) const DEFAULT_INT64: (i64, i64) = (i64::MIN, i64::MAX);
pub(crate) const DEFAULT_FLOAT: (f64, f64) = (i8::MIN as f64, i8::MAX as f64);
pub(crate) const DEFAULT_BOOL: f64 = 0.5;
pub(crate) const CONFIG_KEY_RANGE_LOW: &'static str = "low";
pub(crate) const CONFIG_KEY_RANGE_HIGH: &'static str = "high";
pub(crate) const CONFIG_KEYS_RANGE: [&'static str; 2] =
    [CONFIG_KEY_RANGE_LOW, CONFIG_KEY_RANGE_HIGH];

#[derive(Debug)]
pub enum SimpleScriptVariableKind {
    AnyOf,
    Array,
    Tick,
    Instant,
    Choice,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float64,
    Bool,
    UUID,
    Decimal,
}

impl SimpleScriptVariableKind {
    pub fn named() -> DataGenerationResult<Vec<(String, SimpleScriptVariableKind)>> {
        [
            "Tick",
            "Instant",
            "Uniform",
            "UniformAnyOf",
            "UniformArray",
            "UniformU8",
            "UniformU16",
            "UniformU32",
            "UniformU64",
            "UniformI8",
            "UniformI16",
            "UniformI32",
            "UniformI64",
            "UniformF64",
            "UniformDecimal",
            "Bool",
            "UUID",
        ]
        .iter()
        .map(|s| Self::from_string(s).map(|k| (s.to_string(), k)))
        .collect()
    }

    pub fn from_string(s: &str) -> DataGenerationResult<Self> {
        match s {
            "Tick" => Ok(Self::Tick),
            "Instant" => Ok(Self::Instant),
            "Uniform" => Ok(Self::Choice),
            "UniformAnyOf" => Ok(Self::AnyOf),
            "UniformArray" => Ok(Self::Array),
            "UniformU8" => Ok(Self::UInt8),
            "UniformU16" => Ok(Self::UInt16),
            "UniformU32" => Ok(Self::UInt32),
            "UniformU64" => Ok(Self::UInt64),
            "UniformI8" => Ok(Self::Int8),
            "UniformI16" => Ok(Self::Int16),
            "UniformI32" => Ok(Self::Int32),
            "UniformI64" => Ok(Self::Int64),
            "UniformF64" => Ok(Self::Float64),
            "UniformDecimal" => Ok(Self::Decimal),
            "Bool" => Ok(Self::Bool),
            "UUID" => Ok(Self::UUID),
            _ => Err(DataGenerationError::Other(format!(
                "Unknown random variable kind `{s}`"
            ))),
        }
    }
}

fn range(
    config: Option<LazyStruct<AnyEncoding>>,
) -> ProcessConfigResult<Option<(ValueRef<AnyEncoding>, ValueRef<AnyEncoding>)>> {
    let low = config
        .and_then(|c| c.get(CONFIG_KEY_RANGE_LOW).transpose())
        .transpose()?;
    let high = config
        .and_then(|c| c.get(CONFIG_KEY_RANGE_HIGH).transpose())
        .transpose()?;

    match (low, high) {
        (Some(low), Some(high)) => Ok(Some((low, high))),
        (None, None) => Ok(None),
        _ => Err(ProcessConfigError::Other(
            "If specifying range, both 'low' and 'high' are required".to_string(),
        )),
    }
}

fn range_i64(
    config: Option<LazyStruct<AnyEncoding>>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Option<(i64, i64)>> {
    let range = range(config)?;
    if let Some((low, high)) = range {
        Ok(Some((
            to_i64(low, symbol_parser)?,
            to_i64(high, symbol_parser)?,
        )))
    } else {
        Ok(None)
    }
}

fn range_f64(
    config: Option<LazyStruct<AnyEncoding>>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Option<(f64, f64)>> {
    let range = range(config)?;
    if let Some((low, high)) = range {
        Ok(Some((
            to_f64(low, symbol_parser)?,
            to_f64(high, symbol_parser)?,
        )))
    } else {
        Ok(None)
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
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let density = reader::parse_density(config.as_ref(), symbol_parser)?;

        let gen: Box<dyn ValueGenerator> = match self {
            SimpleScriptVariableKind::AnyOf => {
                self.parse_any_of(rng, density, symbol_parser, config)?
            }
            SimpleScriptVariableKind::Choice => {
                self.parse_choice(rng, density, symbol_parser, config)?
            }
            SimpleScriptVariableKind::Array => {
                self.parse_array(rng, density, symbol_parser, config)?
            }
            SimpleScriptVariableKind::Tick => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY])?;
                Box::new(TickGenerator::new(rng, density)?)
            }
            SimpleScriptVariableKind::Instant => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY])?;
                Box::new(InstantGenerator::new(rng, density)?)
            }
            SimpleScriptVariableKind::UInt8 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_UINT8);
                Box::new(bounded_u8(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::UInt16 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_UINT16);
                Box::new(bounded_u16(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::UInt32 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_UINT32);
                Box::new(bounded_u32(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::UInt64 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_UINT64);
                Box::new(bounded_u64(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Int8 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_INT8);
                Box::new(bounded_i8(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Int16 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_INT16);
                Box::new(bounded_i16(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Int32 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_INT32);
                Box::new(bounded_i32(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Int64 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_i64(config, symbol_parser)?.unwrap_or(DEFAULT_INT64);
                Box::new(bounded_i64(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Float64 => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_f64(config, symbol_parser)?.unwrap_or(DEFAULT_FLOAT);
                Box::new(bounded_f64(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Decimal => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &CONFIG_KEYS_RANGE])?;
                let (low, high) = range_f64(config, symbol_parser)?.unwrap_or(DEFAULT_FLOAT);
                Box::new(bounded_decimal(rng, density, low, high)?)
            }
            SimpleScriptVariableKind::Bool => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &["p"]])?;
                let p = if let Some(p) = config.and_then(|c| c.get("p").transpose()).transpose()? {
                    to_f64(p, symbol_parser)?
                } else {
                    DEFAULT_BOOL
                };
                Box::new(bounded_bool(rng, density, p)?)
            }
            SimpleScriptVariableKind::UUID => {
                validate_config_keys(config, [&CONFIG_KEYS_DENSITY])?;
                Box::new(simple_uuid(rng, density)?)
            }
        };
        Ok(gen)
    }
}

impl SimpleScriptVariableKind {
    fn parse_any_of<R>(
        &self,
        rng: R,
        density: Density,
        symbol_parser: &dyn EnvSymbolParser,
        config: Option<LazyStruct<AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + Clone + 'static,
    {
        validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &["types"]])?;
        let config = config.ok_or_else(|| {
            ProcessConfigError::NoConfig(format!("Usage of {self:?} with no config is unsupported"))
        })?;

        let lst = config.get_expected("types")?.expect_list()?;

        let mut generators = vec![];
        for gen in lst.into_iter() {
            let gen_value = gen?.read()?;

            let gen = match gen_value {
                ValueRef::Symbol(sym) => symbol_parser.parse_symbol_as_generator(&sym, None)?,
                ValueRef::Struct(strct) => {
                    let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;
                    if annot.is_empty() {
                        Err(ProcessConfigError::Other(format!(
                            "Unsupported type for {strct:?}"
                        )))?
                    } else {
                        symbol_parser.parse_symbol_as_generator(&annot[0], Some(strct))?
                    }
                }
                _ => Err(ProcessConfigError::Other(format!(
                    "Unsupported `type` {gen_value:?} in `UniformAnyOf` definition"
                )))?,
            };

            generators.push(gen);
        }

        Ok(Box::new(bounded_union(rng, density, generators)?))
    }

    fn parse_choice<R>(
        &self,
        rng: R,
        density: Density,
        _symbol_parser: &dyn EnvSymbolParser,
        config: Option<LazyStruct<AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + Clone + 'static,
    {
        validate_config_keys(config, [&CONFIG_KEYS_DENSITY, &["choices"]])?;
        let config = config.ok_or_else(|| {
            ProcessConfigError::NoConfig(format!("Usage of {self:?} with no config is unsupported"))
        })?;

        let choices = config.get_expected("choices")?.expect_list()?;
        let mut choice_values = vec![];
        for choice in choices.iter() {
            let choice = choice?.read()?;
            let ion_type = choice.ion_type();
            let value: Value = match choice {
                ValueRef::Bool(b) => Ok(b.into()),
                ValueRef::Int(i) => Ok(i.as_i64().unwrap().into()),
                ValueRef::Float(f) => Ok(f.into()),
                ValueRef::String(s) => Ok(s.text().into()),
                _ => Err(ProcessConfigError::Other(format!(
                    "Unsupported Type for `Uniform` `{ion_type}`"
                ))),
            }?;

            choice_values.push(value);
        }

        Ok(Box::new(bounded_choose(rng, density, choice_values)?))
    }
    fn parse_array<R>(
        &self,
        rng: R,
        density: Density,
        symbol_parser: &dyn EnvSymbolParser,
        config: Option<LazyStruct<AnyEncoding>>,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + Clone + 'static,
    {
        validate_config_keys(
            config,
            [
                &CONFIG_KEYS_DENSITY,
                &["element_type", "min_size", "max_size"],
            ],
        )?;
        let config = config.ok_or_else(|| {
            ProcessConfigError::NoConfig(format!("Usage of {self:?} with no config is unsupported"))
        })?;

        let min_size = config.get_expected("min_size")?;
        let max_size = config.get_expected("max_size")?;

        let get_generator = |sym: &SymbolRef,
                             cfg: Option<LazyStruct<AnyEncoding>>|
         -> ProcessConfigResult<Box<dyn ValueGenerator>> {
            let gen = symbol_parser.parse_symbol_as_generator(sym, cfg)?;

            Ok(Box::new(bounded_array(
                rng,
                density,
                min_size.expect_i64()?,
                max_size.expect_i64()?,
                gen,
            )?) as Box<dyn ValueGenerator>)
        };

        let elem_type = config.get_expected("element_type")?;

        Ok(match elem_type {
            ValueRef::Symbol(sym) => get_generator(&sym, None)?,
            ValueRef::Struct(strct) => {
                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;
                if annot.is_empty() {
                    Err(ProcessConfigError::Other(format!(
                        "Unsupported type for {strct:?}"
                    )))?
                } else {
                    get_generator(&annot[0], Some(strct))?
                }
            }
            _ => Err(ProcessConfigError::Other(format!(
                "Unsupported `element_type` {elem_type:?} in `UniformArray` definition"
            )))?,
        })
    }
}
