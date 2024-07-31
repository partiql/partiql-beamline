use crate::generator::{DatasetPaths, PathAndShape, PathAndShapeSet, PathGenStep};
use crate::strategy::StrategyResult;
use bitflags::bitflags;
use derive_builder::Builder;
use indexmap::IndexMap;
use itertools::Itertools;
use partiql_beamline::sim::NameAndShape;
use partiql_types::{PartiqlShape, Static, StaticType};
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct PathTypeFlags: u32 {
        const Scalar = 1 << 0;
        const Sequence = 1 << 1;
        const Struct = 1 << 2;
    }
}

impl PathTypeFlags {
    pub fn matches(&self, ty: &PartiqlShape) -> bool {
        match ty {
            PartiqlShape::Dynamic => {
                todo!("dynamic type not supported yet")
            }
            PartiqlShape::AnyOf(anyof) => anyof.types().any(|ty| self.matches(ty)),
            PartiqlShape::Static(ty) => {
                (ty.is_scalar() && self.contains(PathTypeFlags::Scalar))
                    || (ty.is_sequence() && self.contains(PathTypeFlags::Sequence))
                    || (ty.is_struct() && self.contains(PathTypeFlags::Struct))
            }
            PartiqlShape::Undefined => {
                todo!("undefined type not supported")
            }
        }
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct PathStepFlags: u32 {
        const  PathProject = 1 << 0; // e.g., `.foo`
        const  PathIndex = 1 << 1;   // e.g., `[3]`
        const  PathForEach = 1 << 2; // e.g., `[*]`
        const  PathUnpivot = 1 << 3; // e.g., `.*`
    }
}

impl PathStepFlags {
    pub fn matches(&self, step: &PathGenStep) -> bool {
        match step {
            PathGenStep::PathProject(_) => self.contains(PathStepFlags::PathProject),
            PathGenStep::PathIndex(_) => self.contains(PathStepFlags::PathIndex),
            PathGenStep::PathForEach => self.contains(PathStepFlags::PathForEach),
            PathGenStep::PathUnpivot => self.contains(PathStepFlags::PathUnpivot),
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct PathGenSpec {
    #[builder(default = "PathStepFlags::all()")]
    pub allowed_internal_steps: PathStepFlags,
    #[builder(default = "PathTypeFlags::all()")]
    pub allowed_final_types: PathTypeFlags,
    #[builder(default = "PathStepFlags::all()")]
    pub allowed_final_steps: PathStepFlags,
    #[builder(default = "Bound::Unbounded")]
    pub min_depth: Bound<usize>,
    #[builder(default = "Bound::Unbounded")]
    pub max_depth: Bound<usize>,
}

impl PathGenSpec {
    pub fn paths_for_dataset(&self, dataset: &NameAndShape) -> StrategyResult<DatasetPaths> {
        let NameAndShape { name, shape } = dataset;
        let paths = self.paths_for_shape(shape)?;
        Ok(DatasetPaths {
            name: name.clone(),
            paths,
        })
    }

    pub fn paths_for_shape(&self, ty: &PartiqlShape) -> StrategyResult<PathAndShapeSet> {
        let mut paths = Vec::default();
        self.path_for_type(ty, &mut paths)?;
        paths.reverse();
        Ok(paths)
    }

    fn allowed_step(&self, step: PathStepFlags) -> bool {
        self.allowed_internal_steps.contains(step) || self.allowed_final_steps.contains(step)
    }

    pub fn path_for_type(
        &self,
        shape: &PartiqlShape,
        paths: &mut Vec<PathAndShape>,
    ) -> StrategyResult<()> {
        fn append(steps: &[PathGenStep], step: PathGenStep, shape: PartiqlShape) -> PathAndShape {
            PathAndShape {
                steps: steps.iter().cloned().chain(std::iter::once(step)).collect(),
                shape,
            }
        }

        let min_depth = match self.min_depth {
            Bound::Included(n) => n,
            Bound::Excluded(n) => n.saturating_sub(1),
            Bound::Unbounded => usize::MIN,
        };

        let max_depth = match self.max_depth {
            Bound::Included(n) => n,
            Bound::Excluded(n) => n.saturating_sub(1),
            Bound::Unbounded => usize::MAX,
        };

        let mut candidates: Vec<(usize, PathAndShape)> = vec![(
            0,
            PathAndShape {
                steps: vec![],
                shape: shape.clone(),
            },
        )];

        'candidate_queue: while let Some((depth, PathAndShape { steps, shape })) = candidates.pop()
        {
            // reject candidates over the max path depth
            if depth > max_depth {
                continue 'candidate_queue;
            }

            let accept_depth = depth >= min_depth;
            let accept_type = self.allowed_final_types.matches(&shape);
            let accept_step = steps
                .last()
                .map_or(true, |step| self.allowed_final_steps.matches(step));
            let valid_internal_step = steps
                .last()
                .map_or(true, |step| self.allowed_internal_steps.matches(step));

            // if this step is a valid internal step, inspect it and generate new candidates
            if valid_internal_step {
                match &shape {
                    PartiqlShape::Dynamic => {
                        todo!("dynamic type not supported yet")
                    }
                    PartiqlShape::AnyOf(anyof) => {
                        // sort the children types by depth of path
                        let by_depth: IndexMap<(Bound<usize>, Bound<usize>), Vec<&PartiqlShape>> =
                            anyof.types().map(|s| (s.path_depth(), s)).fold(
                                IndexMap::default(),
                                |mut lookup, (depth, shape)| {
                                    lookup.entry(depth).or_default().push(shape);
                                    lookup
                                },
                            );

                        // if all children have the same path depth, we can accept self
                        // if differing path depths, push inspection of each path depth and not self
                        if by_depth.len() > 1 {
                            for dep in by_depth.into_values() {
                                let shape = if dep.len() == 1 {
                                    dep.into_iter().next().unwrap().clone()
                                } else {
                                    PartiqlShape::AnyOf(dep.into_iter().cloned().collect())
                                };
                                let steps = steps.clone();
                                candidates.push((depth, PathAndShape { steps, shape }));
                            }
                            continue 'candidate_queue;
                        }
                    }
                    PartiqlShape::Static(sty) => {
                        match sty.ty() {
                            Static::Struct(s) => {
                                // Handle path unpivots for structs (e.g. the `.*` in `path.*.a.b.c`)
                                if self.allowed_step(PathStepFlags::PathUnpivot) {
                                    // Collect the types of all fields
                                    let field_types: Vec<_> =
                                        s.fields().map(|f| f.ty()).unique().cloned().collect();

                                    if field_types.len() == 1 {
                                        // If there's only a single type, push it
                                        let step_in = append(
                                            &steps,
                                            PathGenStep::PathUnpivot,
                                            field_types.into_iter().next().unwrap(),
                                        );
                                        candidates.push((depth + 1, step_in));
                                    } else {
                                        // If there are multiple, combine them into an `AnyOf`
                                        let step_in = append(
                                            &steps,
                                            PathGenStep::PathUnpivot,
                                            PartiqlShape::AnyOf(field_types.into_iter().collect()),
                                        );
                                        candidates.push((depth + 1, step_in));
                                    }
                                }

                                // Handle projecting single named keys (e.g., the `.foo` in `path.foo`)
                                if self.allowed_step(PathStepFlags::PathProject) {
                                    for field in s.fields() {
                                        let step_in = append(
                                            &steps,
                                            PathGenStep::PathProject(field.name().to_string()),
                                            field.ty().clone(),
                                        );
                                        candidates.push((depth + 1, step_in));
                                    }
                                }
                            }
                            Static::Bag(b) => {
                                // handle for each over bags (e.g. the `[*]` in `path[*].a`)
                                if self.allowed_step(PathStepFlags::PathForEach) {
                                    let step_in = append(
                                        &steps,
                                        PathGenStep::PathForEach,
                                        b.element_type().clone(),
                                    );
                                    candidates.push((depth + 1, step_in));
                                }
                            }
                            Static::Array(l) => {
                                // handle for each over list (e.g. the `[*]` in `path[*].a`)
                                if self.allowed_step(PathStepFlags::PathForEach) {
                                    let step_in = append(
                                        &steps,
                                        PathGenStep::PathForEach,
                                        l.element_type().clone(),
                                    );
                                    candidates.push((depth + 1, step_in));
                                }
                                // TODO handle explicitly indexed paths (e.g., the `[5]` in `path[5].a`)
                                if self.allowed_step(PathStepFlags::PathIndex) {
                                    todo!("path index")
                                }
                            }
                            _ => {}
                        };
                    }
                    PartiqlShape::Undefined => {
                        todo!("undefined type not supported")
                    }
                }
            }

            // if this step is a valid termination point, add it to generated path list
            if accept_depth && accept_type && accept_step {
                paths.push(PathAndShape {
                    steps: steps.clone(),
                    shape: shape.clone(),
                })
            }
        }

        Ok(())
    }
}

trait PathDepth {
    fn path_depth(&self) -> (Bound<usize>, Bound<usize>);
}

impl PathDepth for PartiqlShape {
    fn path_depth(&self) -> (Bound<usize>, Bound<usize>) {
        match self {
            PartiqlShape::Dynamic => (Bound::Unbounded, Bound::Unbounded),
            PartiqlShape::AnyOf(any) => any
                .types()
                .fold((Bound::Included(0), Bound::Included(0)), |range, ty| {
                    merge_range(range, ty.path_depth())
                }),
            PartiqlShape::Static(ty) => ty.path_depth(),
            PartiqlShape::Undefined => (Bound::Unbounded, Bound::Unbounded),
        }
    }
}

#[inline]
fn merge_bound<F>(l: Bound<&usize>, r: Bound<&usize>, f: F) -> Bound<usize>
where
    F: FnOnce(usize, usize) -> usize,
{
    match (l, r) {
        (Bound::Unbounded, _) => Bound::Unbounded,
        (_, Bound::Unbounded) => Bound::Unbounded,
        (Bound::Included(v1), Bound::Included(v2)) => Bound::Included(f(*v1, *v2)),
        (Bound::Excluded(v1), Bound::Excluded(v2)) => Bound::Excluded(f(*v1, *v2)),
        (Bound::Excluded(_), Bound::Included(_)) => {
            todo!()
        }
        (Bound::Included(_), Bound::Excluded(_)) => {
            todo!()
        }
    }
}

#[inline]
fn merge_range(
    l: impl RangeBounds<usize>,
    r: impl RangeBounds<usize>,
) -> (Bound<usize>, Bound<usize>) {
    let start = merge_bound(l.start_bound(), r.start_bound(), std::cmp::min);
    let end = merge_bound(l.end_bound(), r.end_bound(), std::cmp::max);
    (start, end)
}

impl PathDepth for StaticType {
    fn path_depth(&self) -> (Bound<usize>, Bound<usize>) {
        if self.ty().is_scalar() {
            (Bound::Included(0), Bound::Included(0))
        } else {
            let (s, e) = match self.ty() {
                Static::Struct(st) => st
                    .fields()
                    .fold((Bound::Included(0), Bound::Included(0)), |range, f| {
                        merge_range(range, f.ty().path_depth())
                    }),
                Static::Bag(seq) => seq.element_type().path_depth(),
                Static::Array(seq) => seq.element_type().path_depth(),
                _ => unreachable!(),
            };

            let e = match e {
                Bound::Included(n) => Bound::Included(n + 1),
                Bound::Excluded(n) => Bound::Excluded(n + 1),
                Bound::Unbounded => Bound::Unbounded,
            };

            (s, e)
        }
    }
}
