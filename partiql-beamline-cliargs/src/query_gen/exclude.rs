use crate::query_gen::IntoStrategy;
use clap::Args;
use partiql_ast::ast;
use partiql_beamline::sim::NameAndShape;
use partiql_beamline_query::strategy::exclude::RandomExcludeListBuilder;
use partiql_beamline_query::strategy::path::{
    PathGenSpec, PathGenSpecBuilder, PathStepFlags, PathTypeFlags,
};
use partiql_beamline_query::strategy::StrategyError::ExcludePaths;
use partiql_beamline_query::strategy::{DynStrategy, StrategyBoxed, StrategyResult};
use std::collections::Bound;
use std::ops::RangeInclusive;

/// Exclusion List Generation
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ExcludeListArg {
    #[command(flatten)]
    pub exclude_count: ExcludeListCountArg,

    #[command(flatten)]
    pub path_spec: ExcludeListPathSpecArg,
}

impl IntoStrategy<NameAndShape, ast::Exclusion> for ExcludeListArg {
    fn into_strategy(self) -> StrategyResult<DynStrategy<NameAndShape, ast::Exclusion>> {
        let ExcludeListArg {
            exclude_count,
            path_spec,
        } = self;
        let (min, max) = exclude_count.range()?.into_inner();
        Ok(RandomExcludeListBuilder::default()
            .min_items(min)
            .max_items(max)
            .path_spec(path_spec.spec()?)
            .build()?
            .sboxed())
    }
}

/// Count of exclusions to generate
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ExcludeListCountArg {
    /// Minimum number of exclusions. Valid values: 1-255
    #[arg(long)]
    pub exclude_rand_min: u8,
    /// Maximum number of exclusions. Valid values: 1-255
    #[arg(long)]
    pub exclude_rand_max: u8,
}

impl ExcludeListCountArg {
    pub fn range(&self) -> StrategyResult<RangeInclusive<u8>> {
        let min = self.exclude_rand_min;
        let max = self.exclude_rand_max;

        if min == 0 {
            return Err(ExcludePaths(
                "--exclude-rand-min must be greater than 0".to_string(),
            ));
        }

        if min > max {
            return Err(ExcludePaths(
                "--exclude-rand-max must be greater than or equal to --exclude-rand-min"
                    .to_string(),
            ));
        }

        Ok(min..=max)
    }
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = true, multiple = true)]
pub struct ExcludeListPathSpecArg {
    /// Minimum path depth. Valid values: 1-255; Default unbounded
    #[arg(long)]
    pub exclude_path_depth_min: Option<u8>,
    /// Maximum path depth. Valid values: 1-255; Default unbounded
    #[arg(long)]
    pub exclude_path_depth_max: Option<u8>,

    /// Enable generation of all variants of internal path steps
    #[arg(long)]
    pub exclude_pathstep_internal_all: bool,
    /// Enable generation of projection-type internal path steps (e.g., the `.foo` in `variable.foo`)
    #[arg(long, conflicts_with = "exclude_pathstep_internal_all")]
    pub exclude_pathstep_internal_project: bool,
    /// Enable generation of index-type internal path steps (e.g., the `[1]` in `variable[1]`)
    #[arg(long, conflicts_with = "exclude_pathstep_internal_all")]
    pub exclude_pathstep_internal_index: bool,
    /// Enable generation of for-each-type internal path steps (e.g., the `[*]` in `variable[*]`)
    #[arg(long, conflicts_with = "exclude_pathstep_internal_all")]
    pub exclude_pathstep_internal_foreach: bool,
    /// Enable generation of unpivot-type internal path steps (e.g., the `.*` in `variable.*`)
    #[arg(long, conflicts_with = "exclude_pathstep_internal_all")]
    pub exclude_pathstep_internal_unpivot: bool,

    /// Enable generation of all variants of final path steps
    #[arg(long)]
    pub exclude_pathstep_final_all: bool,
    /// Enable generation of projection-type final path steps (e.g., the `.foo` in `variable.foo`)
    #[arg(long, conflicts_with = "exclude_pathstep_final_all")]
    pub exclude_pathstep_final_project: bool,
    /// Enable generation of index-type final path steps (e.g., the `[1]` in `variable[1]`)
    #[arg(long, conflicts_with = "exclude_pathstep_final_all")]
    pub exclude_pathstep_final_index: bool,
    /// Enable generation of for-each-type final path steps (e.g., the `[*]` in `variable[*]`)
    #[arg(long, conflicts_with = "exclude_pathstep_final_all")]
    pub exclude_pathstep_final_foreach: bool,
    /// Enable generation of unpivot-type final path steps (e.g., the `.*` in `variable.*`)
    #[arg(long, conflicts_with = "exclude_pathstep_final_all")]
    pub exclude_pathstep_final_unpivot: bool,

    /// Enable generation of all variants of final types in paths
    #[arg(long)]
    pub exclude_type_final_all: bool,
    /// Enable generation of scalar final types in paths (i.e., the type of a full path can be a scalar (e.g., `9`, `'foo'`, etc))
    #[arg(long, conflicts_with = "exclude_type_final_all")]
    pub exclude_type_final_scalar: bool,
    /// Enable generation of sequence final types in paths (i.e., the type of a full path can be a sequence (e.g., `[1,2,3]`, `<1, 'foo'>`, etc))
    #[arg(long, conflicts_with = "exclude_type_final_all")]
    pub exclude_type_final_sequence: bool,
    /// Enable generation of struct final types in paths (i.e., the type of a full path can be a struct (e.g., `{'a': 9, 'b': []}}`, etc))
    #[arg(long, conflicts_with = "exclude_type_final_all")]
    pub exclude_type_final_struct: bool,
}

impl ExcludeListPathSpecArg {
    pub fn spec(&self) -> StrategyResult<PathGenSpec> {
        let depth = |n: Option<u8>| {
            n.map(|n| Bound::Included(n as usize))
                .unwrap_or(Bound::Unbounded)
        };
        Ok(PathGenSpecBuilder::default()
            .min_depth(depth(self.exclude_path_depth_min))
            .max_depth(depth(self.exclude_path_depth_max))
            .allowed_internal_steps(self.allowed_internal_steps()?)
            .allowed_final_steps(self.allowed_final_steps()?)
            .allowed_final_types(self.allowed_final_types()?)
            .build()?)
    }

    fn allowed_internal_steps(&self) -> StrategyResult<PathStepFlags> {
        let mut flags = PathStepFlags::empty();
        if self.exclude_pathstep_internal_all {
            flags |= PathStepFlags::all();
        };
        if self.exclude_pathstep_internal_project {
            flags |= PathStepFlags::PathProject;
        };
        if self.exclude_pathstep_internal_index {
            flags |= PathStepFlags::PathIndex;
        };
        if self.exclude_pathstep_internal_foreach {
            flags |= PathStepFlags::PathForEach;
        };
        if self.exclude_pathstep_internal_unpivot {
            flags |= PathStepFlags::PathUnpivot;
        };
        Ok(flags)
    }

    fn allowed_final_steps(&self) -> StrategyResult<PathStepFlags> {
        let mut flags = PathStepFlags::empty();
        if self.exclude_pathstep_final_all {
            flags |= PathStepFlags::all();
        };
        if self.exclude_pathstep_final_project {
            flags |= PathStepFlags::PathProject;
        };
        if self.exclude_pathstep_final_index {
            flags |= PathStepFlags::PathIndex;
        };
        if self.exclude_pathstep_final_foreach {
            flags |= PathStepFlags::PathForEach;
        };
        if self.exclude_pathstep_final_unpivot {
            flags |= PathStepFlags::PathUnpivot;
        };
        Ok(flags)
    }

    fn allowed_final_types(&self) -> StrategyResult<PathTypeFlags> {
        let mut flags = PathTypeFlags::empty();
        if self.exclude_type_final_all {
            flags |= PathTypeFlags::all();
        };
        if self.exclude_type_final_scalar {
            flags |= PathTypeFlags::Scalar;
        };
        if self.exclude_type_final_sequence {
            flags |= PathTypeFlags::Sequence;
        };
        if self.exclude_type_final_struct {
            flags |= PathTypeFlags::Struct;
        };
        Ok(flags)
    }
}
