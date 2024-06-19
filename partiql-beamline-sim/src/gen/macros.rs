use partiql_types::{
    ArrayType, BagType, PartiqlType, StructConstraint, StructField, StructType, TypeKind,
    TYPE_BOOL, TYPE_DATETIME, TYPE_DECIMAL, TYPE_INT, TYPE_MISSING, TYPE_NULL, TYPE_REAL,
    TYPE_STRING,
};
use partiql_value::Value;

// TODO use concat_idents! when stable so $inner doesn't need to be passed

macro_rules! rv_typedef {
    ($name: ident, $inner: ident) => {
        pub type $name<R> = RandomVariable<R, $inner>;
    };

    ($(#[$outer:meta])* $name: ident, $inner: ident) => {
        $(#[$outer])*
        pub type $name<R> = RandomVariable<R, $inner>;
    };
}
pub(crate) use rv_typedef;
macro_rules! rv_stateless {
    ($inner: ident) => {
        #[derive(Debug, Clone, Default)]
        #[doc(hidden)]
        pub struct $inner {}
    };
}
pub(crate) use rv_stateless;
macro_rules! rv_default_new {
    ($name: ident) => {
        impl<R> $name<R>
        where
            R: Rng + Sized + Clone,
        {
            pub fn new(rng: R, density: Density) -> DataGenerationResult<Self> {
                RandomVariable::create(rng, density, Default::default())
            }
        }
    };
}
pub(crate) use rv_default_new;

macro_rules! make_rv_stateless {
    ($name: ident, $inner: ident) => {
        rv_typedef!($name, $inner);
        rv_stateless!($inner);
        rv_default_new!($name);
    };

    ($(#[$outer:meta])* $name: ident, $inner: ident) => {
        rv_typedef!($(#[$outer])* $name, $inner);
        rv_stateless!($inner);
        rv_default_new!($name);
    };
}
pub(crate) use make_rv_stateless;
