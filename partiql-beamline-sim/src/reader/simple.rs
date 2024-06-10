use crate::gen::simple::{
    bounded_array, bounded_bool, bounded_choose, bounded_decimal, bounded_f64, bounded_i16,
    bounded_i32, bounded_i64, bounded_i8, bounded_u16, bounded_u32, bounded_u64, bounded_u8,
    bounded_union,
};
use crate::gen::{simple, DataGenerationError, DataGenerationResult, ValueGenerator};
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use crate::reader::{ProcessConfigError, ProcessConfigResult};
use ion_rs::{AnyEncoding, LazyStruct, SymbolRef, ValueRef};
use ion_rs_old::external::bigdecimal::ToPrimitive;
use ion_rs_old::IonReader;
use partiql_value::Value;
use rand::Rng;

#[derive(Debug)]
pub enum SimpleScriptVariableKind {
    AnyOf,
    Array,
    Tick,
    Instant,
    String,
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
            "String",
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
            "String" => Ok(Self::String),
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

    pub fn create<R>(&self, rng: R) -> DataGenerationResult<Box<dyn ValueGenerator>>
    where
        R: Rng + Sized + Clone + 'static,
    {
        match self {
            SimpleScriptVariableKind::Tick => Ok(Box::new(simple::simple_tick())),
            SimpleScriptVariableKind::Instant => Ok(Box::new(simple::simple_instant())),
            SimpleScriptVariableKind::UInt8 => Ok(Box::new(simple::simple_u8(rng)?)),
            SimpleScriptVariableKind::UInt16 => Ok(Box::new(simple::simple_u16(rng)?)),
            SimpleScriptVariableKind::UInt32 => Ok(Box::new(simple::simple_u32(rng)?)),
            SimpleScriptVariableKind::UInt64 => Ok(Box::new(simple::simple_u64(rng)?)),
            SimpleScriptVariableKind::Int8 => Ok(Box::new(simple::simple_i8(rng)?)),
            SimpleScriptVariableKind::Int16 => Ok(Box::new(simple::simple_i16(rng)?)),
            SimpleScriptVariableKind::Int32 => Ok(Box::new(simple::simple_i32(rng)?)),
            SimpleScriptVariableKind::Int64 => Ok(Box::new(simple::simple_i64(rng)?)),
            SimpleScriptVariableKind::Float64 => Ok(Box::new(simple::simple_f64(rng)?)),
            SimpleScriptVariableKind::Decimal => Ok(Box::new(simple::simple_decimal(rng)?)),
            SimpleScriptVariableKind::Bool => Ok(Box::new(simple::simple_bool(rng)?)),
            SimpleScriptVariableKind::UUID => Ok(Box::new(simple::simple_uuid(rng)?)),
            _ => Err(DataGenerationError::NoConfig(format!(
                "Usage of {self:?} with no config is unsupported"
            ))),
        }
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
        if let Some(config) = config {
            fn to_float(
                val: ValueRef<AnyEncoding>,
                symbol_parser: &dyn EnvSymbolParser,
            ) -> ProcessConfigResult<f64> {
                match val {
                    ValueRef::Int(i) => Ok(i.as_i64().expect("integer") as f64),
                    ValueRef::Float(f) => Ok(f),
                    ValueRef::Decimal(d) => match d.to_string().parse::<f64>() {
                        Ok(f) => Ok(f),
                        Err(e) => Err(ProcessConfigError::Other(e.to_string())),
                    },
                    ValueRef::Symbol(sym) => {
                        Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
                            Value::Integer(i) => i as f64,
                            Value::Real(f) => f.0,
                            Value::Decimal(d) => d.to_f64().unwrap(),
                            other => todo!("non-numeric float64 param {other:?}"),
                        })
                    }
                    _ => todo!("non-numeric float64 param {val:?}"),
                }
            }
            fn range(
                config: LazyStruct<AnyEncoding>,
            ) -> ProcessConfigResult<(ValueRef<AnyEncoding>, ValueRef<AnyEncoding>)> {
                Ok((config.get_expected("low")?, config.get_expected("high")?))
            }

            let gen: Box<dyn ValueGenerator> = match self {
                SimpleScriptVariableKind::AnyOf => {
                    let lst = config.get_expected("types")?.expect_list()?;

                    let mut generators = vec![];
                    for gen in lst.into_iter() {
                        let gen_value = gen?.read()?;

                        let gen = match gen_value {
                            ValueRef::Symbol(sym) => {
                                symbol_parser.parse_symbol_as_generator(&sym, None)?
                            }
                            ValueRef::Struct(strct) => {
                                let annot = strct.annotations().collect::<Result<Vec<_>, _>>()?;
                                if annot.is_empty() {
                                    Err(ProcessConfigError::Other(format!(
                                        "Unsupported type for {strct:?}"
                                    )))?
                                } else {
                                    symbol_parser
                                        .parse_symbol_as_generator(&annot[0], Some(strct))?
                                }
                            }
                            _ => Err(ProcessConfigError::Other(format!(
                                "Unsupported `type` {gen_value:?} in `UniformAnyOf` definition"
                            )))?,
                        };

                        generators.push(gen);
                    }

                    Box::new(bounded_union(rng, generators)?)
                }
                SimpleScriptVariableKind::Choice => {
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

                    Box::new(bounded_choose(rng, choice_values)?)
                }
                SimpleScriptVariableKind::Array => {
                    let min_size = config.get_expected("min_size")?;
                    let max_size = config.get_expected("max_size")?;

                    let get_generator =
                        |sym: &SymbolRef,
                         cfg: Option<LazyStruct<AnyEncoding>>|
                         -> ProcessConfigResult<Box<dyn ValueGenerator>> {
                            let gen = symbol_parser.parse_symbol_as_generator(sym, cfg)?;

                            Ok(Box::new(bounded_array(
                                rng,
                                min_size.expect_i64()?,
                                max_size.expect_i64()?,
                                gen,
                            )?) as Box<dyn ValueGenerator>)
                        };

                    let elem_type = config.get_expected("element_type")?;

                    match elem_type {
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
                    }
                }
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
                SimpleScriptVariableKind::Decimal => {
                    let (low, high) = range(config)?;
                    let low = to_float(low, symbol_parser)?;
                    let high = to_float(high, symbol_parser)?;

                    Box::new(bounded_decimal(rng, low, high)?)
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
            Ok(self.create(rng)?)
        }
    }
}
