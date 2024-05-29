use crate::gen::timeline::{InstantGenerator, TickGenerator};
use crate::gen::{DataGenerationError, DataGenerationResult, ValueGenerator};
use crate::reader::ProcessConfigError;
use crate::sim::context::SimContext;
use partiql_types::{ArrayType, PartiqlType, TypeKind, TYPE_BOOL};
use partiql_value::{List, Value};
use rand::distributions::Distribution;
use rand::{Rng, SeedableRng};
use rand_distr::num_traits::FromPrimitive;
use rand_pcg::{Mcg128Xsl64, Pcg64Mcg};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;

pub enum SimpleScriptVariableKind {
    Array,
    Tick,
    Instant,
    String,
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
            SimpleScriptVariableKind::String => {
                todo!()
            }
            SimpleScriptVariableKind::Array => Ok(Box::new(simple_array(rng)?)),
            SimpleScriptVariableKind::Tick => Ok(Box::new(simple_tick())),
            SimpleScriptVariableKind::Instant => Ok(Box::new(simple_instant())),
            SimpleScriptVariableKind::UInt8 => Ok(Box::new(simple_u8(rng)?)),
            SimpleScriptVariableKind::UInt16 => Ok(Box::new(simple_u16(rng)?)),
            SimpleScriptVariableKind::UInt32 => Ok(Box::new(simple_u32(rng)?)),
            SimpleScriptVariableKind::UInt64 => Ok(Box::new(simple_u64(rng)?)),
            SimpleScriptVariableKind::Int8 => Ok(Box::new(simple_i8(rng)?)),
            SimpleScriptVariableKind::Int16 => Ok(Box::new(simple_i16(rng)?)),
            SimpleScriptVariableKind::Int32 => Ok(Box::new(simple_i32(rng)?)),
            SimpleScriptVariableKind::Int64 => Ok(Box::new(simple_i64(rng)?)),
            SimpleScriptVariableKind::Float64 => Ok(Box::new(simple_f64(rng)?)),
            SimpleScriptVariableKind::Decimal => Ok(Box::new(simple_decimal(rng)?)),
            SimpleScriptVariableKind::Bool => Ok(Box::new(simple_bool(rng)?)),
            SimpleScriptVariableKind::UUID => Ok(Box::new(simple_uuid(rng)?)),
        }
    }
}

pub fn simple_array<R>(
    mut rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_array(
        rng.clone(),
        2i64,
        10i64,
        Box::new(simple_tick()) as Box<dyn ValueGenerator>,
    )
}

pub fn simple_tick() -> TickGenerator {
    TickGenerator {}
}

pub fn simple_instant() -> InstantGenerator {
    InstantGenerator {}
}

pub fn simple_union<R>(
    rng: R,
    generators: Vec<Box<dyn ValueGenerator>>,
    _ctx: SimContext,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let types: Vec<PartiqlType> = generators.iter().map(|gen| gen.value_type()).collect();

    let name = format!("UniformUnion::[ {:?} ]", types);
    let typ = PartiqlType::any_of(types);
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::DiscreteUniform::new(0, (generators.len() - 1) as i64)?;
    let f = move |rng: &mut R, ctx: &SimContext| {
        let idx = dist.sample(rng) as i64;
        let generator = &generators[idx as usize];
        generator.gen_value(ctx)
    };
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn simple_choose<R>(
    rng: R,
    choices: Vec<Value>,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if choices.is_empty() {
        return Err(DataGenerationError::Other(
            "Empty choice vector".to_string(),
        ));
    }

    use crate::gen::util::ValueTypeInference;
    use rand::seq::SliceRandom;

    let name = "UniformChoice".into();
    let typ = PartiqlType::any_of(choices.iter().map(|v| v.infer_type()));
    let rng = RefCell::new(rng);
    let f = move |rng: &mut R, _ctx: &SimContext| choices.as_slice().choose(rng).unwrap().clone();
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn simple_bool<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_bool(rng, 0.5)
}

pub fn simple_uuid<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = "UUID".into();
    let typ = PartiqlType::new(TypeKind::String);
    let rng = RefCell::new(rng);
    let f = move |rng: &mut R, _ctx: &SimContext| {
        let mut uuid_bytes = uuid::Bytes::default();
        rng.fill_bytes(&mut uuid_bytes);
        let id = uuid::Uuid::from_bytes(uuid_bytes);
        Value::from(id.to_string())
    };
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn simple_u8<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u8(rng, u8::MIN as i64, u8::MAX as i64)
}

pub fn simple_u16<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u16(rng, u16::MIN as i64, u16::MAX as i64)
}

pub fn simple_u32<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u32(rng, u32::MIN as i64, u32::MAX as i64)
}

pub fn simple_u64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_u64(rng, u64::MIN as i64, i64::MAX)
}

pub fn simple_i8<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i8(rng, i8::MIN as i64, i8::MAX as i64)
}

pub fn simple_i16<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i16(rng, i16::MIN as i64, i16::MAX as i64)
}

pub fn simple_i32<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i32(rng, i32::MIN as i64, i32::MAX as i64)
}

pub fn simple_i64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_i64(rng, i64::MIN, i64::MAX)
}

pub fn simple_f64<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_f64(rng, i8::MIN as f64, i8::MAX as f64)
}

pub fn simple_decimal<R>(
    rng: R,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    bounded_decimal(rng, i8::MIN as f64, i8::MAX as f64)
}

pub fn bounded_array<R>(
    rng: R,
    min: i64,
    max: i64,
    elem_generator: Box<dyn ValueGenerator>,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min > max {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        let elem_type = elem_generator.value_type();

        let name = format!(
            "UniformArray::{{ min_size: {min}, max_size: {max}, element_type: {elem_type:?} }}"
        );

        let rng = RefCell::new(rng);
        let dist = statrs::distribution::DiscreteUniform::new(min, max)?;
        let typ = PartiqlType::new_array(ArrayType::new(Box::new(elem_type.clone())));
        let f = move |rng: &mut R, ctx: &SimContext| {
            let array_length = dist.sample(rng) as usize;
            let array: Vec<_> = std::iter::repeat_with(|| elem_generator.gen_value(&ctx))
                .take(array_length)
                .collect();
            Value::List(Box::new(List::from(array)))
        };
        Ok(SimpleRandomVariable { name, typ, rng, f })
    }
}

pub fn bounded_bool<R>(
    rng: R,
    p: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformBool::{{ p: {p} }}");
    let typ = TYPE_BOOL;
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Bernoulli::new(p)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng) > 0f64);
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn bounded_u8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u8::MIN as i64 || max > u8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u16::MIN as i64 || max > u16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u32::MIN as i64 || max > u32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_u64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u64::MIN as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i8<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i8::MIN as i64 || max > i8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i16<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i16::MIN as i64 || max > i16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i32<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i32::MIN as i64 || max > i32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, min, max)
    }
}

pub fn bounded_i64<R>(
    rng: R,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformI64::{{ low: {min}, high: {max} }}");
    let typ = PartiqlType::new(TypeKind::Int64);
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::DiscreteUniform::new(min, max)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng) as i64);
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn bounded_f64<R>(
    rng: R,
    min: f64,
    max: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformF64::{{ low: {min}, high: {max} }}");
    let typ = PartiqlType::new(TypeKind::Float64);
    let rng = RefCell::new(rng);
    let dist = statrs::distribution::Uniform::new(min, max)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng));
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub fn bounded_decimal<R>(
    rng: R,
    min: f64,
    max: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformDecimal::{{low: {min}, high: {max} }}");

    let p_max_dec = rust_decimal::Decimal::from_f64(max).unwrap();
    let p_max_dec_precision = p_max_dec
        .mantissa()
        .unsigned_abs()
        .checked_ilog10()
        .unwrap_or_default()
        + 1;

    let p_max_scale = p_max_dec.scale();

    let typ = PartiqlType::new(TypeKind::DecimalP(
        p_max_dec_precision as usize,
        p_max_scale as usize,
    ));
    let rng = RefCell::new(rng);

    let dist = statrs::distribution::Uniform::new(min, max)?;

    let f = move |rng: &mut R, _ctx: &SimContext| {
        let mut out_dec =
            rust_decimal::Decimal::from_f64_retain(dist.sample(rng)).expect("decimal value");
        out_dec.rescale(p_max_scale);
        Value::Decimal(Box::new(out_dec))
    };
    Ok(SimpleRandomVariable { name, typ, rng, f })
}

pub struct SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value,
{
    name: String,
    typ: PartiqlType,

    /// The source of randomness
    rng: RefCell<R>,

    f: F,
}

impl<R, F> Clone for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            typ: self.typ.clone(),
            rng: self.rng.clone(),
            f: self.f.clone(),
        }
    }
}

impl<R, F> Debug for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleRandomVariable")
            .field("name", &self.name)
            .finish()
    }
}

impl<R, F> ValueGenerator for SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn gen_value(&self, ctx: &SimContext) -> Value {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        (self.f)(rng, ctx)
    }

    fn value_type(&self) -> PartiqlType {
        self.typ.clone()
    }
}
