use crate::gen::distributions::{Density, InnerValueGenerator, RandomVariable};
use crate::gen::timeline::{InstantGenerator, TickGenerator};
use crate::gen::{DataGenerationError, DataGenerationResult, ValueGenerator};
use crate::sim::context::SimContext;
use partiql_types::{ArrayType, PartiqlType, TypeKind, TYPE_BOOL};
use partiql_value::{List, Value};
use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::num_traits::FromPrimitive;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub struct SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value,
{
    pub(crate) name: String,
    pub(crate) typ: PartiqlType,
    pub(crate) f: F,
    rng: PhantomData<R>,
}

pub type SimpleRandomVariable<R, F> = RandomVariable<R, SimpleRandomVariableImpl<R, F>>;

impl<R, F> SimpleRandomVariable<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    pub fn new(
        rng: R,
        density: Density,
        name: String,
        typ: PartiqlType,
        f: F,
    ) -> DataGenerationResult<Self> {
        let inner = SimpleRandomVariableImpl {
            name,
            typ,
            f,
            rng: PhantomData,
        };
        RandomVariable::create(rng, density, inner)
    }
}

impl<R, F> Clone for SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            typ: self.typ.clone(),
            f: self.f.clone(),
            rng: self.rng,
        }
    }
}

impl<R, F> Debug for SimpleRandomVariableImpl<R, F>
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

impl<R, F> InnerValueGenerator<R> for SimpleRandomVariableImpl<R, F>
where
    R: Rng + Sized + Clone,
    F: Fn(&mut R, &SimContext) -> Value + Clone,
{
    fn present_value(&self, rng: &mut R, ctx: &SimContext) -> Value {
        (self.f)(rng, ctx)
    }

    fn value_type(&self) -> PartiqlType {
        self.typ.clone()
    }
}

pub fn bounded_union<R>(
    rng: R,
    density: Density,
    generators: Vec<Box<dyn ValueGenerator>>,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let types: Vec<PartiqlType> = generators.iter().map(|gen| gen.value_type()).collect();

    let name = format!("UniformUnion::[ {:?} ]", types);
    let typ = PartiqlType::any_of(types);

    let dist = statrs::distribution::DiscreteUniform::new(0, (generators.len() - 1) as i64)?;
    let f = move |rng: &mut R, ctx: &SimContext| {
        let idx = dist.sample(rng) as i64;
        let generator = &generators[idx as usize];
        generator.gen_value(ctx)
    };
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn bounded_choose<R>(
    rng: R,
    density: Density,
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

    let f = move |rng: &mut R, _ctx: &SimContext| choices.as_slice().choose(rng).unwrap().clone();
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn simple_uuid<R>(
    rng: R,
    density: Density,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = "UUID".into();
    let typ = PartiqlType::new(TypeKind::String);

    let f = move |rng: &mut R, _ctx: &SimContext| {
        let mut uuid_bytes = uuid::Bytes::default();
        rng.fill_bytes(&mut uuid_bytes);
        let id = uuid::Uuid::from_bytes(uuid_bytes);
        Value::from(id.to_string())
    };
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn bounded_array<R>(
    rng: R,
    density: Density,
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

        let dist = statrs::distribution::DiscreteUniform::new(min, max)?;
        let typ = PartiqlType::new_array(ArrayType::new(Box::new(elem_type.clone())));
        let f = move |rng: &mut R, ctx: &SimContext| {
            let array_length = dist.sample(rng) as usize;
            let array: Vec<_> = std::iter::repeat_with(|| elem_generator.gen_value(ctx))
                .take(array_length)
                .collect();
            Value::List(Box::new(List::from(array)))
        };
        SimpleRandomVariable::new(rng, density, name, typ, f)
    }
}

pub fn bounded_bool<R>(
    rng: R,
    density: Density,
    p: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformBool::{{ p: {p} }}");
    let typ = TYPE_BOOL;

    let dist = statrs::distribution::Bernoulli::new(p)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng) > 0f64);
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn bounded_u8<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u8::MIN as i64 || max > u8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_u16<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u16::MIN as i64 || max > u16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_u32<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u32::MIN as i64 || max > u32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_u64<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < u64::MIN as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_i8<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i8::MIN as i64 || max > i8::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_i16<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i16::MIN as i64 || max > i16::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_i32<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    if min < i32::MIN as i64 || max > i32::MAX as i64 {
        Err(DataGenerationError::Bounds(min, max))
    } else {
        bounded_i64(rng, density, min, max)
    }
}

pub fn bounded_i64<R>(
    rng: R,
    density: Density,
    min: i64,
    max: i64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformI64::{{ low: {min}, high: {max} }}");
    let typ = PartiqlType::new(TypeKind::Int64);

    let dist = statrs::distribution::DiscreteUniform::new(min, max)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng) as i64);
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn bounded_f64<R>(
    rng: R,
    density: Density,
    min: f64,
    max: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let name = format!("UniformF64::{{ low: {min}, high: {max} }}");
    let typ = PartiqlType::new(TypeKind::Float64);

    let dist = statrs::distribution::Uniform::new(min, max)?;
    let f = move |rng: &mut R, _ctx: &SimContext| Value::from(dist.sample(rng));
    SimpleRandomVariable::new(rng, density, name, typ, f)
}

pub fn bounded_decimal<R>(
    rng: R,
    density: Density,
    min: f64,
    max: f64,
) -> DataGenerationResult<SimpleRandomVariable<R, impl Fn(&mut R, &SimContext) -> Value + Clone>>
where
    R: Rng + Sized + Clone,
{
    let p_and_s = |n| {
        let dec = rust_decimal::Decimal::from_f64(n).unwrap();
        let precision = dec
            .mantissa()
            .unsigned_abs()
            .checked_ilog10()
            .unwrap_or_default()
            + 1;

        let scale = dec.scale();
        (precision, scale)
    };

    let name = format!("UniformDecimal::{{low: {min}, high: {max} }}");

    let (p_max_dec_precision, p_max_scale) = p_and_s(max);
    let (p_min_dec_precision, p_min_scale) = p_and_s(min);

    let precision = p_max_dec_precision.max(p_min_dec_precision);
    let scale = p_max_scale.max(p_min_scale);

    let typ = PartiqlType::new(TypeKind::DecimalP(precision as usize, scale as usize));

    let dist = statrs::distribution::Uniform::new(min, max)?;

    let f = move |rng: &mut R, _ctx: &SimContext| {
        let mut out_dec =
            rust_decimal::Decimal::from_f64_retain(dist.sample(rng)).expect("decimal value");
        out_dec.rescale(scale);
        Value::Decimal(Box::new(out_dec))
    };
    SimpleRandomVariable::new(rng, density, name, typ, f)
}
