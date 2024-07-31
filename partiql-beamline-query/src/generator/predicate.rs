use crate::generator::{AstGenContext, AstGenerator, PathAndShape, PathGenStep};
use partiql_ast::ast;
use partiql_types::PartiqlShape;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub enum PredicateDirection {
    Normal,
    Inverted,
}

#[derive(Debug, Clone)]
pub enum Predicate {
    AND,
    OR,
    NOT,

    EQ,
    NEQ,

    LTE,
    LT,
    GTE,
    GT,

    BETWEEN,

    IS_NULL(PredicateDirection),
    IS_MISSING(PredicateDirection),
    LIKE(PredicateDirection),
    IN(PredicateDirection),
}

#[derive(Clone)]
pub struct PathPredicates {
    pub path_and_shape: PathAndShape,
    pub predicates: Vec<Predicate>,
}

impl Debug for PathPredicates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let PathPredicates {
            path_and_shape,
            predicates,
        } = self;
        write!(f, "{path_and_shape:?} ({predicates:?})")
    }
}

pub type PathPredicatesSet = Vec<PathPredicates>;

#[derive(Debug, Clone)]
pub struct DatasetPredicates {
    pub name: String,
    pub paths: PathPredicatesSet,
}
