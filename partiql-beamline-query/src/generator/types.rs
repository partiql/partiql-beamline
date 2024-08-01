use indexmap::IndexSet;
use partiql_types::{PartiqlShape, Static};

static SimpleDynamicVariants: [Static; 9] = [
    Static::Int,
    Static::Int8,
    Static::Int16,
    Static::Int32,
    Static::Int64,
    Static::Bool,
    Static::Decimal,
    Static::Float64,
    Static::String,
];

pub(crate) fn flatten_types(shape: &PartiqlShape) -> IndexSet<&Static> {
    let mut dynamic = false;
    let mut candidates = vec![shape];
    let mut flat = IndexSet::new();
    while let Some(candidate) = candidates.pop() {
        match candidate {
            PartiqlShape::Dynamic => dynamic = true,
            PartiqlShape::AnyOf(any) => candidates.extend(any.types()),
            PartiqlShape::Static(sty) => {
                flat.insert(sty.ty());
            }
            PartiqlShape::Undefined => dynamic = true,
        }
    }

    if dynamic {
        flat.extend(SimpleDynamicVariants.iter());
    }

    flat
}

pub(crate) trait StaticTypeOperations {
    fn is_comparable(&self) -> bool;
}

impl StaticTypeOperations for Static {
    fn is_comparable(&self) -> bool {
        matches!(
            self,
            Static::Int
                | Static::Int8
                | Static::Int16
                | Static::Int32
                | Static::Int64
                | Static::Bool
                | Static::Decimal
                | Static::DecimalP(_, _)
                | Static::Float32
                | Static::Float64
                | Static::DateTime
        )
    }
}
