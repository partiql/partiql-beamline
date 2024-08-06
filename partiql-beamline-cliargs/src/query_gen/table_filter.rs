use crate::query_gen::IntoStrategy;
use clap::Args;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use partiql_beamline_query::strategy::path::{
    PathGenSpec, PathGenSpecBuilder, PathStepFlags, PathTypeFlags,
};
use partiql_beamline_query::strategy::predicate::PredicateFlags;
use partiql_beamline_query::strategy::where_clause::RandomRowPredicateBuilder;
use partiql_beamline_query::strategy::StrategyError::ProjectPaths;
use partiql_beamline_query::strategy::{DynStrategy, StrategyBoxed, StrategyResult};
use std::collections::Bound;
use std::ops::RangeInclusive;

/// Table Filter Generation (i.e., `WHERE`)
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct TableFilterArg {
    #[command(flatten)]
    pub filter_count: TableFilterCountArg,

    #[command(flatten)]
    pub pred_flags: PredicateFlagArgs,

    #[command(flatten)]
    pub path_spec: TableFilterPathSpecArg,
}

impl IntoStrategy<NameAndShape, ast::WhereClause> for TableFilterArg {
    fn into_strategy(self) -> StrategyResult<DynStrategy<NameAndShape, ast::WhereClause>> {
        let (min, max) = self.filter_count.range()?.into_inner();
        let pred_spec = self.pred_flags.flags();
        let path_spec = self.path_spec.spec()?;
        let strat = RandomRowPredicateBuilder::default()
            .min_items(min)
            .max_items(max)
            .path_spec(path_spec)
            .allowed_predicates(pred_spec)
            .build()?;
        Ok(strat.sboxed())
    }
}

/// Count of table filter predicates to generate
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct TableFilterCountArg {
    /// Minimum number of predicates in the filter. Valid values: 1-255
    #[arg(long)]
    pub tbl_flt_rand_min: u8,
    /// Maximum number of predicates in the filter. Valid values: 1-255
    #[arg(long)]
    pub tbl_flt_rand_max: u8,
}

impl TableFilterCountArg {
    pub fn range(&self) -> StrategyResult<RangeInclusive<u8>> {
        let min = self.tbl_flt_rand_min;
        let max = self.tbl_flt_rand_max;

        if min == 0 {
            return Err(ProjectPaths(
                "--tbl-flt-rand-min must be greater than 0".to_string(),
            ));
        }

        if min > max {
            return Err(ProjectPaths(
                "--tbl-flt-rand-max must be greater than or equal to --tbl-flt-rand-min"
                    .to_string(),
            ));
        }

        Ok(min..=max)
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = true)]
pub struct TableFilterPathSpecArg {
    /// Minimum path depth. Valid values: 1-255; Default unbounded
    #[arg(long)]
    pub tbl_flt_path_depth_min: Option<u8>,
    /// Maximum path depth. Valid values: 1-255; Default unbounded
    #[arg(long)]
    pub tbl_flt_path_depth_max: Option<u8>,

    /// Enable generation of all variants of internal path steps
    #[arg(long)]
    pub tbl_flt_pathstep_internal_all: bool,
    /// Enable generation of projection-type internal path steps (e.g., the `.foo` in `variable.foo`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_internal_all")]
    pub tbl_flt_pathstep_internal_project: bool,
    /// Enable generation of index-type internal path steps (e.g., the `[1]` in `variable[1]`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_internal_all")]
    pub tbl_flt_pathstep_internal_index: bool,
    /// Enable generation of for-each-type internal path steps (e.g., the `[*]` in `variable[*]`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_internal_all")]
    pub tbl_flt_pathstep_internal_foreach: bool,
    /// Enable generation of unpivot-type internal path steps (e.g., the `.*` in `variable.*`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_internal_all")]
    pub tbl_flt_pathstep_internal_unpivot: bool,

    /// Enable generation of all variants of final path steps
    #[arg(long)]
    pub tbl_flt_pathstep_final_all: bool,
    /// Enable generation of projection-type final path steps (e.g., the `.foo` in `variable.foo`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_final_all")]
    pub tbl_flt_pathstep_final_project: bool,
    /// Enable generation of index-type final path steps (e.g., the `[1]` in `variable[1]`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_final_all")]
    pub tbl_flt_pathstep_final_index: bool,
    /// Enable generation of for-each-type final path steps (e.g., the `[*]` in `variable[*]`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_final_all")]
    pub tbl_flt_pathstep_final_foreach: bool,
    /// Enable generation of unpivot-type final path steps (e.g., the `.*` in `variable.*`)
    #[arg(long, conflicts_with = "tbl_flt_pathstep_final_all")]
    pub tbl_flt_pathstep_final_unpivot: bool,

    /// Enable generation of all variants of final types in paths
    #[arg(long)]
    pub tbl_flt_type_final_all: bool,
    /// Enable generation of scalar final types in paths (i.e., the type of a full path can be a scalar (e.g., `9`, `'foo'`, etc))
    #[arg(long, conflicts_with = "tbl_flt_type_final_all")]
    pub tbl_flt_type_final_scalar: bool,
    /// Enable generation of sequence final types in paths (i.e., the type of a full path can be a sequence (e.g., `[1,2,3]`, `<1, 'foo'>`, etc))
    #[arg(long, conflicts_with = "tbl_flt_type_final_all")]
    pub tbl_flt_type_final_sequence: bool,
    /// Enable generation of struct final types in paths (i.e., the type of a full path can be a struct (e.g., `{'a': 9, 'b': []}}`, etc))
    #[arg(long, conflicts_with = "tbl_flt_type_final_all")]
    pub tbl_flt_type_final_struct: bool,
}

impl TableFilterPathSpecArg {
    pub fn spec(&self) -> StrategyResult<PathGenSpec> {
        let depth = |n: Option<u8>| {
            n.map(|n| Bound::Included(n as usize))
                .unwrap_or(Bound::Unbounded)
        };
        Ok(PathGenSpecBuilder::default()
            .min_depth(depth(self.tbl_flt_path_depth_min))
            .max_depth(depth(self.tbl_flt_path_depth_max))
            .allowed_internal_steps(self.allowed_internal_steps()?)
            .allowed_final_steps(self.allowed_final_steps()?)
            .allowed_final_types(self.allowed_final_types()?)
            .build()?)
    }

    fn allowed_internal_steps(&self) -> StrategyResult<PathStepFlags> {
        let mut flags = PathStepFlags::empty();
        if self.tbl_flt_pathstep_internal_all {
            flags |= PathStepFlags::all();
        };
        if self.tbl_flt_pathstep_internal_project {
            flags |= PathStepFlags::PathProject;
        };
        if self.tbl_flt_pathstep_internal_index {
            flags |= PathStepFlags::PathIndex;
        };
        if self.tbl_flt_pathstep_internal_foreach {
            flags |= PathStepFlags::PathForEach;
        };
        if self.tbl_flt_pathstep_internal_unpivot {
            flags |= PathStepFlags::PathUnpivot;
        };
        Ok(flags)
    }

    fn allowed_final_steps(&self) -> StrategyResult<PathStepFlags> {
        let mut flags = PathStepFlags::empty();
        if self.tbl_flt_pathstep_final_all {
            flags |= PathStepFlags::all();
        };
        if self.tbl_flt_pathstep_final_project {
            flags |= PathStepFlags::PathProject;
        };
        if self.tbl_flt_pathstep_final_index {
            flags |= PathStepFlags::PathIndex;
        };
        if self.tbl_flt_pathstep_final_foreach {
            flags |= PathStepFlags::PathForEach;
        };
        if self.tbl_flt_pathstep_final_unpivot {
            flags |= PathStepFlags::PathUnpivot;
        };
        Ok(flags)
    }

    fn allowed_final_types(&self) -> StrategyResult<PathTypeFlags> {
        let mut flags = PathTypeFlags::empty();
        if self.tbl_flt_type_final_all {
            flags |= PathTypeFlags::all();
        };
        if self.tbl_flt_type_final_scalar {
            flags |= PathTypeFlags::Scalar;
        };
        if self.tbl_flt_type_final_sequence {
            flags |= PathTypeFlags::Sequence;
        };
        if self.tbl_flt_type_final_struct {
            flags |= PathTypeFlags::Struct;
        };
        Ok(flags)
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = true)]
pub struct PredicateFlagArgs {
    /// Enable all predicates
    #[arg(long, conflicts_with = "pred_none")]
    pub pred_all: bool,
    /// Enable no predicates
    #[arg(long, conflicts_with = "pred_all")]
    pub pred_none: bool,

    /// Enable `IS NULL` and `IS NOT NULL` and `IS MISSING` and `IS NOT MISSING`
    #[arg(long)]
    pub pred_absent: bool,

    /// Enable `IS NULL` and `IS NOT NULL`
    #[arg(long)]
    pub pred_nullable: bool,
    /// Enable `IS NULL`
    #[arg(long)]
    pub pred_is_null: bool,
    /// Enable `IS NOT NULL`
    #[arg(long)]
    pub pred_is_not_null: bool,

    /// Enable `IS MISSING` and `IS NOT MISSING`
    #[arg(long)]
    pub pred_optional: bool,
    /// Enable `IS MISSING`
    #[arg(long)]
    pub pred_is_missing: bool,
    /// Enable `IS NOT MISSING`
    #[arg(long)]
    pub pred_is_not_missing: bool,

    /// Enable \[`=`, `<>``\]
    #[arg(long)]
    pub pred_equality: bool,
    /// Enable `=`
    #[arg(long)]
    pub pred_eq: bool,
    /// Enable `<>`
    #[arg(long)]
    pub pred_neq: bool,

    /// Enable \[`<`, `<=`, `>`, `>=`, `BETWEEN`\]
    #[arg(long)]
    pub pred_comparison: bool,
    /// Enable `<`
    #[arg(long)]
    pub pred_lt: bool,
    /// Enable `<=`
    #[arg(long)]
    pub pred_lte: bool,
    /// Enable `>`
    #[arg(long)]
    pub pred_gt: bool,
    /// Enable `>=`
    #[arg(long)]
    pub pred_gte: bool,
    /// Enable `BETWEEN`
    #[arg(long)]
    pub pred_between: bool,

    /// Enable \[`=`, `<>`, `<`, `<=`, `>`, `>=`, `BETWEEN`\]
    #[arg(long)]
    pub pred_numeric: bool,

    /// Enable `LIKE` and `NOT LIKE`
    #[arg(long)]
    pub pred_like_all: bool,
    /// Enable `LIKE`
    #[arg(long)]
    pub pred_like: bool,
    /// Enable `NOT LIKE`
    #[arg(long)]
    pub pred_not_like: bool,

    /// Enable `IN` and `NOT IN`
    #[arg(long)]
    pub pred_in_all: bool,
    /// Enable `IN`
    #[arg(long)]
    pub pred_in: bool,
    /// Enable `NOT IN`
    #[arg(long)]
    pub pred_not_in: bool,

    /// Enable `AND` and `OR` and `NOT`
    #[arg(long)]
    pub pred_logical_all: bool,
    /// Enable `AND`
    #[arg(long)]
    pub pred_logical_and: bool,
    /// Enable `OR`
    #[arg(long)]
    pub pred_logical_or: bool,
    /// Enable `NOT`
    #[arg(long)]
    pub pred_logical_not: bool,
}

impl PredicateFlagArgs {
    pub fn flags(&self) -> PredicateFlags {
        let mut flags = PredicateFlags::empty();

        if self.pred_all {
            flags |= PredicateFlags::all();
        }

        if self.pred_absent {
            flags |= PredicateFlags::flags_absent();
        }

        if self.pred_nullable {
            flags |= PredicateFlags::flags_nullable();
        }
        if self.pred_is_null {
            flags |= PredicateFlags::IS_NULL;
        }
        if self.pred_is_not_null {
            flags |= PredicateFlags::IS_NOT_NULL;
        }

        if self.pred_optional {
            flags |= PredicateFlags::flags_optional();
        }
        if self.pred_is_missing {
            flags |= PredicateFlags::IS_MISSING;
        }
        if self.pred_is_not_missing {
            flags |= PredicateFlags::IS_NOT_MISSING;
        }

        if self.pred_equality {
            flags |= PredicateFlags::flags_equality();
        }
        if self.pred_eq {
            flags |= PredicateFlags::EQ;
        }
        if self.pred_neq {
            flags |= PredicateFlags::NEQ;
        }

        if self.pred_comparison {
            flags |= PredicateFlags::flags_comparison();
        }
        if self.pred_lt {
            flags |= PredicateFlags::LT;
        }
        if self.pred_lte {
            flags |= PredicateFlags::LTE;
        }
        if self.pred_gt {
            flags |= PredicateFlags::GT;
        }
        if self.pred_gte {
            flags |= PredicateFlags::GTE;
        }
        if self.pred_between {
            flags |= PredicateFlags::BETWEEN;
        }

        if self.pred_numeric {
            flags |= PredicateFlags::flags_numeric();
        }

        if self.pred_like_all {
            flags |= PredicateFlags::flags_like();
        }
        if self.pred_like {
            flags |= PredicateFlags::LIKE;
        }
        if self.pred_not_like {
            flags |= PredicateFlags::NOT_LIKE;
        }

        if self.pred_in_all {
            flags |= PredicateFlags::flags_in();
        }
        if self.pred_in {
            flags |= PredicateFlags::IN;
        }
        if self.pred_not_in {
            flags |= PredicateFlags::NOT_IN;
        }

        if self.pred_logical_all {
            flags |= PredicateFlags::flags_logical();
        }
        if self.pred_logical_and {
            flags |= PredicateFlags::AND;
        }
        if self.pred_logical_or {
            flags |= PredicateFlags::OR;
        }
        if self.pred_logical_not {
            flags |= PredicateFlags::NOT;
        }

        flags
    }
}
