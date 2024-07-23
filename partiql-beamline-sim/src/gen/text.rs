use crate::gen::distributions::{Density, InnerValueGenerator, Meta, RandomVariable};
use crate::gen::{DataGenerationError, DataGenerationResult};
use crate::sim::SimContext;
use lipsum::{lipsum_title_with_rng, lipsum_with_rng};
use partiql_types::{PartiqlShape, TYPE_STRING};
use partiql_value::Value;
use rand::distributions::Distribution;
use rand::Rng;
use regex_syntax::hir::{Class, Hir, HirKind, Repetition};
use regex_syntax::ParserBuilder;
use statrs::distribution::DiscreteUniform;
use std::fmt::Debug;
use std::io::Write;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct LoremIpsumImpl {
    len: DiscreteUniform,
}

pub type LoremIpsumGenerator<R> = RandomVariable<R, LoremIpsumImpl>;

impl<R> LoremIpsumGenerator<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(
        rng: R,
        meta: Meta,
        density: Density,
        min: u8,
        max: u8,
    ) -> DataGenerationResult<Self> {
        let len = statrs::distribution::DiscreteUniform::new(min as i64, max as i64)?;
        RandomVariable::create(rng, meta, density, LoremIpsumImpl { len })
    }
}

impl<R> InnerValueGenerator<R> for LoremIpsumImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        let n = self.len.sample(rng) as usize;
        Value::from(lipsum_with_rng(rng, n))
    }

    fn value_type(&self) -> PartiqlShape {
        TYPE_STRING
    }
}

#[derive(Debug, Clone)]
pub struct LoremIpsumTitleImpl {}

pub type LoremIpsumTitleGenerator<R> = RandomVariable<R, LoremIpsumTitleImpl>;

impl<R> LoremIpsumTitleGenerator<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(rng: R, meta: Meta, density: Density) -> DataGenerationResult<Self> {
        RandomVariable::create(rng, meta, density, LoremIpsumTitleImpl {})
    }
}

impl<R> InnerValueGenerator<R> for LoremIpsumTitleImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        Value::from(lipsum_title_with_rng(rng))
    }

    fn value_type(&self) -> PartiqlShape {
        TYPE_STRING
    }
}

#[derive(Debug, Clone)]
pub struct RegexImpl {
    re_strategy: ReStrategy,
}

pub type RegexGenerator<R> = RandomVariable<R, RegexImpl>;

fn regex_err(error: &str) -> DataGenerationError {
    DataGenerationError::Other(format!("Regex Error: {error}"))
}
fn unsupported<T>(error: &str) -> Result<T, DataGenerationError> {
    Err(regex_err(&format!("Unsupported Regex: {error}")))
}
impl From<regex_syntax::Error> for DataGenerationError {
    fn from(value: regex_syntax::Error) -> Self {
        regex_err(&value.to_string())
    }
}

impl<R> RegexGenerator<R>
where
    R: Rng + Sized + Clone,
{
    pub fn new(rng: R, meta: Meta, density: Density, regex: &str) -> DataGenerationResult<Self> {
        let hir = ParserBuilder::new().build().parse(regex)?;
        let re_strategy = regex_gen(&hir)?;
        RandomVariable::create(rng, meta, density, RegexImpl { re_strategy })
    }
}

impl<R> InnerValueGenerator<R> for RegexImpl
where
    R: Rng + Sized + Clone,
{
    fn present_value(&self, rng: &mut R, _ctx: &SimContext) -> Value {
        let mut out: Vec<u8> = vec![];
        self.re_strategy.write_to(rng, &mut out);
        Value::from(String::from_utf8(out).expect("valid utf-8"))
    }

    fn value_type(&self) -> PartiqlShape {
        TYPE_STRING
    }
}

#[derive(Debug, Clone)]
enum ReStrategy {
    Lit(ReStrategyLit),
    Repetition(ReStrategyRepetition),
    Range(ReStrategyRange),
    Concat(ReStrategyConcat),
    Choose(ReStrategyChoose),
}

trait ReGen {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write;
}
impl ReGen for ReStrategy {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        match self {
            ReStrategy::Lit(inner) => inner.write_to(rng, out),
            ReStrategy::Repetition(inner) => inner.write_to(rng, out),
            ReStrategy::Range(inner) => inner.write_to(rng, out),
            ReStrategy::Concat(inner) => inner.write_to(rng, out),
            ReStrategy::Choose(inner) => inner.write_to(rng, out),
        }
    }
}

#[derive(Debug, Clone)]
struct ReStrategyLit {
    bytes: Vec<u8>,
}
impl ReGen for ReStrategyLit {
    fn write_to<R, W>(&self, _rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        out.write_all(self.bytes.as_slice()).expect("write");
    }
}

#[derive(Debug, Clone)]
struct ReStrategyRepetition {
    len: DiscreteUniform,
    inner: Box<ReStrategy>,
}
impl ReGen for ReStrategyRepetition {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        let len = self.len.sample(rng) as usize;
        for _i in 0..len {
            self.inner.write_to(rng, out);
        }
    }
}

#[derive(Debug, Clone)]
struct ReStrategyConcat {
    segments: Vec<ReStrategy>,
}
impl ReGen for ReStrategyConcat {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        for inner in &self.segments {
            inner.write_to(rng, out);
        }
    }
}

#[derive(Debug, Clone)]
struct ReStrategyChoose {
    len: DiscreteUniform,
    choices: Vec<ReStrategy>,
}
impl ReGen for ReStrategyChoose {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        let idx = self.len.sample(rng) as usize;
        self.choices[idx].write_to(rng, out);
    }
}

#[derive(Debug, Clone)]
struct ReStrategyRange {
    len: DiscreteUniform,
    offsets: Vec<usize>,
    ranges: Vec<ReStrategyRangeClass>,
}

#[derive(Debug, Clone)]
enum ReStrategyRangeClass {
    Unicode(RangeInclusive<char>),
    Bytes(RangeInclusive<u8>),
}

impl ReGen for ReStrategyRange {
    fn write_to<R, W>(&self, rng: &mut R, out: &mut W)
    where
        R: Rng + Sized + Clone,
        W: Write,
    {
        let draw = self.len.sample(rng) as usize;

        let next_larger = self
            .offsets
            .iter()
            .enumerate()
            .find(|(_idx, offset)| **offset > draw)
            .map(|(idx, _)| idx);

        let one_past_idx = next_larger.unwrap_or(self.offsets.len());

        assert!(one_past_idx > 0);
        let idx = one_past_idx - 1;
        let offset = self.offsets[idx];
        match &self.ranges[idx] {
            ReStrategyRangeClass::Unicode(range) => {
                let mut bytes = [0; 4];
                let x = range.clone().nth(draw - offset).unwrap();
                let b = x.encode_utf8(&mut bytes);
                out.write_all(b.as_bytes()).expect("unicode byte write");
            }
            ReStrategyRangeClass::Bytes(range) => {
                let b = range.clone().nth(draw - offset).unwrap();
                out.write_all(&[b]).expect("byte write");
            }
        }
    }
}

fn regex_gen(expr: &Hir) -> DataGenerationResult<ReStrategy> {
    Ok(match expr.kind() {
        HirKind::Empty => ReStrategy::Lit(ReStrategyLit { bytes: vec![] }),
        HirKind::Literal(lit) => ReStrategy::Lit(ReStrategyLit {
            bytes: lit.0.to_vec(),
        }),
        HirKind::Class(class) => match class {
            Class::Unicode(unicode) => {
                let ranges: Vec<_> = unicode.iter().map(|r| r.start()..=r.end()).collect();
                let (len, offsets) =
                    ranges
                        .iter()
                        .fold((0, vec![]), |(len, mut offsets), range| {
                            offsets.push(len);
                            (len + range.size_hint().0, offsets)
                        });
                let ranges = ranges
                    .into_iter()
                    .map(ReStrategyRangeClass::Unicode)
                    .collect();

                let len = DiscreteUniform::new(0, (len - 1) as i64)?;
                ReStrategy::Range(ReStrategyRange {
                    len,
                    offsets,
                    ranges,
                })
            }
            Class::Bytes(bytes) => {
                let ranges: Vec<_> = bytes.iter().map(|r| r.start()..=r.end()).collect();
                let (len, offsets) =
                    ranges
                        .iter()
                        .fold((0, vec![]), |(len, mut offsets), range| {
                            offsets.push(len);
                            (len + range.size_hint().0, offsets)
                        });
                let ranges = ranges
                    .into_iter()
                    .map(ReStrategyRangeClass::Bytes)
                    .collect();

                let len = DiscreteUniform::new(0, (len - 1) as i64)?;
                ReStrategy::Range(ReStrategyRange {
                    len,
                    offsets,
                    ranges,
                })
            }
        },
        HirKind::Repetition(rep) => {
            let len = to_len(rep)?;
            let inner = Box::new(regex_gen(&rep.sub)?);
            ReStrategy::Repetition(ReStrategyRepetition { len, inner })
        }
        HirKind::Capture(capture) => regex_gen(&capture.sub)?,
        HirKind::Concat(concat) => {
            let segments: Result<Vec<_>, _> = concat.iter().map(regex_gen).collect();
            ReStrategy::Concat(ReStrategyConcat {
                segments: segments?,
            })
        }
        HirKind::Alternation(alts) => {
            let len = DiscreteUniform::new(0, (alts.len() - 1) as i64)?;
            let choices: Result<Vec<_>, _> = alts.iter().map(regex_gen).collect();

            ReStrategy::Choose(ReStrategyChoose {
                len,
                choices: choices?,
            })
        }
        HirKind::Look(_) => return unsupported("Look-around not supported"),
    })
}

fn to_len(rep: &Repetition) -> DataGenerationResult<DiscreteUniform> {
    const REP_MAX: i64 = 42; // Some default length to cap at
    let rep_check = |n: i64| {
        if n < REP_MAX {
            Ok(n)
        } else {
            unsupported(&format!("Max length is {REP_MAX}"))
        }
    };
    let (min, max) = match (rep.min, rep.max) {
        // `*` - zero or more
        (0, None) => (0, REP_MAX),
        // `+` - one or more
        (1, None) => (1, REP_MAX),
        // `?` - zero or one
        (0, Some(1)) => (0, 1),
        // `{n}` - exactly `n`
        (min, Some(max)) if min == max => {
            let n = rep_check(min as i64)?;
            (n, n)
        }
        // `{n,m}` - at least `n` and at most `m`
        (min, Some(max)) => (rep_check(min as i64)?, rep_check(max as i64)?),
        // `{n,}` - at least `n`
        (min, None) => (rep_check(min as i64)?, REP_MAX),
    };
    Ok(DiscreteUniform::new(min, max)?)
}
