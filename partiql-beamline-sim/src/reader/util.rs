use crate::gen::distributions::{Density, Meta};
use crate::gen::ValueGenerator;
use crate::reader::error::{
    ConfigValueError, GeneratorConfigError, ProcessConfigError, ProcessConfigResult, Sourceable,
};
use crate::reader::registry::ValueGeneratorParser;
use crate::reader::symbol::EnvSymbolParser;
use ion_rs::{
    AnyEncoding, HasRange, IonError, LazyField, LazyList, LazyStruct, LazyValue, ValueRef,
};
use ion_rs_old::external::bigdecimal::ToPrimitive;
use miette::SourceSpan;
use partiql_value::Value;
use rand::Rng;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Range;

pub(crate) const CONFIG_KEY_NULLABLE: &str = "nullable";
pub(crate) const CONFIG_KEY_OPTIONAL: &str = "optional";
pub(crate) const CONFIG_KEYS_DENSITY: [&str; 2] = [CONFIG_KEY_NULLABLE, CONFIG_KEY_OPTIONAL];

pub(crate) struct BasicValueGeneratorParser<T, R>
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParserImpl<R>,
{
    inner: T,
    marker: PhantomData<R>,
}

pub(crate) trait ValueGeneratorParserImpl<R>
where
    R: Rng + Sized + 'static,
{
    fn parse_with_config(
        &self,
        _rng: R,
        _meta: Meta,
        _density: Density,
        _config: LazyStruct<'_, AnyEncoding>,
        _symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        Err(ProcessConfigError::ConfigUnexpected)
    }

    fn parse_default(
        &self,
        _rng: R,
        _meta: Meta,
        _density: Density,
        _symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        Err(ProcessConfigError::ConfigExpected)
    }

    fn possible_config_keys(&self) -> &[&'static str] {
        &[]
    }
}

impl<T, R> From<T> for BasicValueGeneratorParser<T, R>
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParserImpl<R>,
{
    fn from(inner: T) -> Self {
        let marker = PhantomData;
        BasicValueGeneratorParser { inner, marker }
    }
}

impl<T, R> BasicValueGeneratorParser<T, R>
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParserImpl<R>,
{
    fn parse_and_handle(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let generator = meta.name.clone();
        self.parse(rng, meta, config, symbol_parser).map_err(|err| {
            ProcessConfigError::GeneratorConfig(Box::new(GeneratorConfigError { generator, err }))
        })
    }

    fn parse(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        let inner = &self.inner;
        let inner_keys = inner.possible_config_keys();
        let status = validate_config_keys(config, inner_keys)?;

        let density = parse_density(config.as_ref(), symbol_parser)?;

        let config = if status.local_keys { config } else { None };
        match config {
            None => inner.parse_default(rng, meta, density, symbol_parser),
            Some(config) => {
                let source_span = config.source_span();
                inner
                    .parse_with_config(rng, meta, density, config, symbol_parser)
                    .map_err(|err| err.with_context(source_span))
            }
        }
    }
}

impl<T, R> ValueGeneratorParser<R> for BasicValueGeneratorParser<T, R>
where
    R: Rng + Sized + 'static,
    T: ValueGeneratorParserImpl<R>,
{
    fn parse_generator(
        &self,
        rng: R,
        meta: Meta,
        config: Option<LazyStruct<'_, AnyEncoding>>,
        symbol_parser: &dyn EnvSymbolParser,
    ) -> ProcessConfigResult<Box<dyn ValueGenerator>> {
        self.parse_and_handle(rng, meta, config, symbol_parser)
    }
}

#[inline]
pub(crate) fn require_key<'a>(
    config: LazyStruct<'a, AnyEncoding>,
    key: &'static str,
) -> ProcessConfigResult<ValueRef<'a, AnyEncoding>> {
    if let Ok(Some(value)) = config.find(key) {
        value.read().map_err(|e| {
            ProcessConfigError::ConfigValue(Box::new(ConfigValueError {
                key: key.to_string(),
                err: e.into(),
            }))
        })
    } else {
        Err(ProcessConfigError::ConfigMissingKey(key.to_string()))
    }
}

pub(crate) fn parse_density(
    config: Option<&LazyStruct<'_, AnyEncoding>>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Density> {
    let nullable_config = config
        .and_then(|c| c.get(CONFIG_KEY_NULLABLE).transpose())
        .transpose()?;
    let optional_config = config
        .and_then(|c| c.get(CONFIG_KEY_OPTIONAL).transpose())
        .transpose()?;

    let null = symbol_parser.default_nullability()?;
    let opt = symbol_parser.default_optionality()?;

    let (nullable, nullable_default) = if let Some(nullable) = nullable_config {
        (to_pct(nullable, symbol_parser)?, false)
    } else {
        (null, true)
    };
    let (optional, optional_default) = if let Some(optional) = optional_config {
        (to_pct(optional, symbol_parser)?, false)
    } else {
        (opt, true)
    };

    let pct_absent = nullable.unwrap_or(0.0) + optional.unwrap_or(0.0);
    let present = 1.0 - pct_absent;

    if !(0.0..=1.0).contains(&present) {
        let fmt_msg = |name: &str, val: Option<f64>, default: bool| {
            format!(
                "{}: `{}`{}",
                name,
                val.unwrap_or(0.0),
                if default {
                    "(from simulation default)"
                } else {
                    ""
                }
            )
        };

        let nullability = fmt_msg(CONFIG_KEY_NULLABLE, nullable, nullable_default);
        let optionality = fmt_msg(CONFIG_KEY_OPTIONAL, optional, optional_default);

        let msg = format!(
            "Combined Nullability and Optionality Percents must be between 0.0 and 1.0; {}; {}.",
            nullability, optionality
        );
        Err(ProcessConfigError::DensityError(msg))?
    } else {
        Ok(Density::new(nullable, optional, present)?)
    }
}

pub(crate) fn to_pct(
    val: ValueRef<'_, AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<Option<f64>> {
    match val {
        ValueRef::Bool(b) => Ok(b.then_some(0.0)),
        other => {
            let pct = to_f64(other, symbol_parser)?;
            if (0.0..=1.0).contains(&pct) {
                Ok(Some(pct))
            } else {
                Err(ProcessConfigError::Other(
                    "Percent must be between 0.0 and 1.0".to_string(),
                ))?
            }
        }
    }
}

pub(crate) fn to_i64(
    val: ValueRef<'_, AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<i64> {
    match val {
        ValueRef::Int(i) => Ok(i.as_i64().expect("integer")),
        ValueRef::Symbol(sym) => Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
            Value::Integer(i) => i,
            other => todo!("non-numeric i64 param {other:?}"),
        }),
        _ => todo!("non-numeric float64 param {val:?}"),
    }
}

pub(crate) fn to_f64(
    val: ValueRef<'_, AnyEncoding>,
    symbol_parser: &dyn EnvSymbolParser,
) -> ProcessConfigResult<f64> {
    match val {
        ValueRef::Int(i) => Ok(i.as_i64().expect("integer") as f64),
        ValueRef::Float(f) => Ok(f),
        ValueRef::Decimal(d) => match d.to_string().parse::<f64>() {
            Ok(f) => Ok(f),
            Err(e) => Err(ProcessConfigError::Other(e.to_string())),
        },
        ValueRef::Symbol(sym) => Ok(match symbol_parser.parse_symbol_as_value(&sym)? {
            Value::Integer(i) => i as f64,
            Value::Real(f) => f.0,
            Value::Decimal(d) => d.to_f64().unwrap(),
            other => todo!("non-numeric float64 param {other:?}"),
        }),
        _ => todo!("non-numeric float64 param {val:?}"),
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct KeyValidation {
    global_keys: bool,
    local_keys: bool,
}

pub(crate) fn validate_config_keys(
    config: Option<LazyStruct<'_, AnyEncoding>>,
    allowed_keys: &[&'static str],
) -> ProcessConfigResult<KeyValidation> {
    let global_keys: HashSet<&'static str> = CONFIG_KEYS_DENSITY.into_iter().by_ref().collect();
    let local_keys: HashSet<&'static str> = allowed_keys.iter().copied().collect();

    validate_config_keyset(config, global_keys, local_keys)
}

fn validate_config_keyset(
    config: Option<LazyStruct<'_, AnyEncoding>>,
    global_keys: HashSet<&'static str>,
    local_keys: HashSet<&'static str>,
) -> ProcessConfigResult<KeyValidation> {
    let mut status = KeyValidation::default();
    let mut seen: HashSet<String> = HashSet::default();
    if let Some(config) = config {
        for s in config.iter() {
            let s = s?;
            let name = s.name()?;
            let name = name.text().unwrap_or("");

            if global_keys.contains(name) {
                status.global_keys = true;
            } else if local_keys.contains(name) {
                status.local_keys = true;
            } else {
                return Err(ProcessConfigError::ConfigInvalidKey(name.to_string()));
            }

            if !seen.insert(name.to_string()) {
                return Err(ProcessConfigError::ConfigDuplicateKey(name.to_string()));
            }
        }
    }
    Ok(status)
}

pub(crate) trait ToSourceSpan {
    fn source_span(&self) -> Option<SourceSpan>;
}

impl<'a, T> ToSourceSpan for T
where
    T: IonRange,
{
    #[inline]
    fn source_span(&self) -> Option<SourceSpan> {
        self.ion_range().map(SourceSpan::from)
    }
}

// TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
pub(crate) trait IonRange {
    fn ion_range(&self) -> Option<Range<usize>>;
}

impl<'a> IonRange for LazyValue<'a, AnyEncoding> {
    #[inline]
    fn ion_range(&self) -> Option<Range<usize>> {
        // TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
        self.raw().as_ref().map(HasRange::range)
    }
}

impl<'a> IonRange for LazyStruct<'a, AnyEncoding> {
    #[inline]
    fn ion_range(&self) -> Option<Range<usize>> {
        // TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
        self.as_value().ion_range()
    }
}

impl<'a> IonRange for LazyList<'a, AnyEncoding> {
    #[inline]
    fn ion_range(&self) -> Option<Range<usize>> {
        // TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
        None
    }
}

impl<'a> IonRange for LazyField<'a, AnyEncoding> {
    #[inline]
    fn ion_range(&self) -> Option<Range<usize>> {
        // TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
        None
    }
}

impl<'a> IonRange for ValueRef<'a, AnyEncoding> {
    #[inline]
    fn ion_range(&self) -> Option<Range<usize>> {
        // TODO fix if/when addressed: https://github.com/amazon-ion/ion-rust/issues/810
        None
    }
}

impl ToSourceSpan for IonError {
    #[inline]
    fn source_span(&self) -> Option<SourceSpan> {
        let pos = match &self {
            IonError::Incomplete(e) => e.position(),
            IonError::Decoding(e) => e.position()?,
            _ => None?,
        };

        let start = pos.byte_offset();
        let len = pos.byte_length().unwrap_or(0);
        Some(SourceSpan::new(start.into(), len))
    }
}
