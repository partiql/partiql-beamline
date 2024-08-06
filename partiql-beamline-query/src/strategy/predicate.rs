use crate::generator::{
    DatasetPaths, DatasetPredicates, PathPredicates, PathPredicatesSet, Predicate,
    PredicateDirection,
};
use crate::strategy::path::PathGenSpec;
use crate::strategy::{StrategyError, StrategyResult};
use bitflags::bitflags;
use bitvec::order::Msb0;
use bitvec::BitArr;
use derive_builder::Builder;
use itertools::Itertools;
use partiql_beamline::sim::NameAndShape;
use partiql_types::{PartiqlShape, Static};
use std::fmt::Debug;
use std::ops::RangeBounds;

type PredicateBits = BitArr!(for 17, in u32, Msb0);

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct PredicateFlags: u32 {
        const NOT = 1 << 0;
        const EQ = 1 << 1;
        const NEQ = 1 << 2;

        const LTE = 1 << 3;
        const LT = 1 << 4;
        const GTE = 1 << 5;
        const GT = 1 << 6;

        const IS_NULL = 1 << 7;
        const IS_NOT_NULL = 1 << 8;

        const IS_MISSING = 1 << 9;
        const IS_NOT_MISSING = 1 << 10;

        const LIKE = 1 << 11;
        const NOT_LIKE = 1 << 12;

        const IN = 1 << 13;
        const NOT_IN = 1 << 14;

        const BETWEEN = 1 << 15;

        const AND = 1 << 16;
        const OR = 1 << 17;
    }
}

const AND_BITS: u32 = PredicateFlags::AND.bits();
const OR_BITS: u32 = PredicateFlags::OR.bits();
const NOT_BITS: u32 = PredicateFlags::NOT.bits();

const EQ_BITS: u32 = PredicateFlags::EQ.bits();
const NEQ_BITS: u32 = PredicateFlags::NEQ.bits();

const LTE_BITS: u32 = PredicateFlags::LTE.bits();
const LT_BITS: u32 = PredicateFlags::LT.bits();
const GTE_BITS: u32 = PredicateFlags::GTE.bits();
const GT_BITS: u32 = PredicateFlags::GT.bits();

const IS_NULL_BITS: u32 = PredicateFlags::IS_NULL.bits();
const IS_NOT_NULL_BITS: u32 = PredicateFlags::IS_NOT_NULL.bits();

const IS_MISSING_BITS: u32 = PredicateFlags::IS_MISSING.bits();
const IS_NOT_MISSING_BITS: u32 = PredicateFlags::IS_NOT_MISSING.bits();

const LIKE_BITS: u32 = PredicateFlags::LIKE.bits();
const NOT_LIKE_BITS: u32 = PredicateFlags::NOT_LIKE.bits();

const IN_BITS: u32 = PredicateFlags::IN.bits();
const NOT_IN_BITS: u32 = PredicateFlags::NOT_IN.bits();

const BETWEEN_BITS: u32 = PredicateFlags::BETWEEN.bits();

impl PredicateFlags {
    pub const fn flags_nullable() -> Self {
        Self::IS_NULL.union(Self::IS_NOT_NULL)
    }

    pub const fn flags_optional() -> Self {
        Self::IS_MISSING.union(Self::IS_NOT_MISSING)
    }

    pub const fn flags_absent() -> Self {
        Self::flags_nullable().union(Self::flags_optional())
    }

    pub const fn flags_equality() -> Self {
        Self::EQ.union(Self::NEQ)
    }

    pub const fn flags_comparison() -> Self {
        Self::BETWEEN
            .union(Self::LTE.union(Self::LT))
            .union(Self::GTE.union(Self::GT))
    }

    pub const fn flags_numeric() -> Self {
        Self::flags_equality().union(Self::flags_comparison())
    }

    pub const fn flags_like() -> Self {
        Self::LIKE.union(Self::NOT_LIKE)
    }

    pub const fn flags_in() -> Self {
        Self::IN.union(Self::NOT_IN)
    }

    pub const fn flags_logical_connectives() -> Self {
        Self::AND.union(Self::OR)
    }

    pub const fn flags_logical() -> Self {
        Self::flags_logical_connectives().union(Self::NOT)
    }
}

impl PredicateFlags {
    pub fn matching(ty: &PartiqlShape) -> Self {
        match ty {
            PartiqlShape::Dynamic => Self::all(),
            PartiqlShape::AnyOf(anyof) => anyof
                .types()
                .map(Self::matching)
                .fold(Self::flags_absent(), |x, y| x | y),
            PartiqlShape::Static(sty) => {
                let ty = sty.ty();
                let flags = Self::flags_absent().union(Self::flags_equality());
                match ty {
                    Static::Int | Static::Int8 | Static::Int16 | Static::Int32 | Static::Int64 => {
                        flags.union(PredicateFlags::flags_numeric())
                    }
                    Static::Bool => flags | PredicateFlags::flags_logical(),
                    Static::Decimal | Static::DecimalP(_, _) => {
                        flags.union(PredicateFlags::flags_numeric())
                    }
                    Static::Float32 | Static::Float64 => {
                        flags.union(PredicateFlags::flags_numeric())
                    }
                    Static::String | Static::StringFixed(_) | Static::StringVarying(_) => {
                        flags.union(PredicateFlags::flags_like())
                    }
                    Static::DateTime => flags,
                    Static::Struct(_) => flags,
                    Static::Bag(_) => flags,
                    Static::Array(_) => flags,
                }
            }
            PartiqlShape::Undefined => {
                todo!("undefined type not supported")
            }
        }
        .union(PredicateFlags::flags_in())
    }

    pub fn count(&self) -> usize {
        PredicateBits::from([self.bits()]).count_ones()
    }
}

#[derive(Debug, Clone, Builder)]
pub struct PathPredicateGenSpec {
    pub path_spec: PathGenSpec,
    #[builder(default = "PredicateFlags::all()")]
    pub allowed_predicates: PredicateFlags,
}

impl PathPredicateGenSpec {
    pub fn predicate_paths_for_dataset(
        &self,
        data: &NameAndShape,
    ) -> StrategyResult<DatasetPredicates> {
        let DatasetPaths { name, paths } = self.path_spec.paths_for_dataset(data)?;

        let paths: PathPredicatesSet = paths
            .into_iter()
            .map(|path_and_shape| {
                let predicates = self.predicates_for(&path_and_shape.shape);
                PathPredicates {
                    path_and_shape,
                    predicates,
                }
            })
            .filter(|path_predicates| !path_predicates.predicates.is_empty())
            .collect();

        if paths.is_empty() {
            Err(StrategyError::PredicatePaths(
                "Configuration leaves no valid paths available for use in predicates.".to_string(),
            ))
        } else {
            Ok(DatasetPredicates { name, paths })
        }
    }

    fn predicates_for(&self, shape: &PartiqlShape) -> Vec<Predicate> {
        // determine applicable predicates & mask by allowed predicates
        let candidates = PredicateFlags::matching(shape).intersection(self.allowed_predicates);
        candidates
            .iter_names()
            .map(|(_name, flag)| match flag.bits() {
                AND_BITS => Predicate::AND,
                OR_BITS => Predicate::OR,
                NOT_BITS => Predicate::NOT,
                EQ_BITS => Predicate::EQ,
                NEQ_BITS => Predicate::NEQ,
                LTE_BITS => Predicate::LTE,
                LT_BITS => Predicate::LT,
                GTE_BITS => Predicate::GTE,
                GT_BITS => Predicate::GT,
                IS_NULL_BITS => Predicate::IS_NULL(PredicateDirection::Normal),
                IS_NOT_NULL_BITS => Predicate::IS_NULL(PredicateDirection::Inverted),
                IS_MISSING_BITS => Predicate::IS_MISSING(PredicateDirection::Normal),
                IS_NOT_MISSING_BITS => Predicate::IS_MISSING(PredicateDirection::Inverted),
                LIKE_BITS => Predicate::LIKE(PredicateDirection::Normal),
                NOT_LIKE_BITS => Predicate::LIKE(PredicateDirection::Inverted),
                IN_BITS => Predicate::IN(PredicateDirection::Normal),
                NOT_IN_BITS => Predicate::IN(PredicateDirection::Inverted),
                BETWEEN_BITS => Predicate::BETWEEN,
                _ => unreachable!(),
            })
            .collect()
    }
}
