//! High-level API for generating data from DDL column definitions.
//!
//! This module provides a convenient builder-pattern API for programmatic
//! data generation from DDL, without needing to manually construct Ion scripts,
//! `SimSource`, `SimConfig`, or `SimBuilder`.
//!
//! # Example
//!
//! ```rust
//! use partiql_beamline::ddl_generator::DdlDataGenerator;
//!
//! let ddl = r#""sensor_id" VARCHAR, "temperature" DOUBLE, "active" BOOL"#;
//!
//! let mut generator = DdlDataGenerator::builder()
//!     .ddl(ddl)
//!     .dataset_name("sensors")
//!     .seed(42)
//!     .build()
//!     .expect("Failed to build generator");
//!
//! // Generate 5 samples
//! let samples: Vec<_> = generator.take(5).collect();
//! for sample in &samples {
//!     let sample = sample.as_ref().expect("sample");
//!     println!("{:?}", sample.value);
//! }
//! ```
//!
//! # Supported DDL Types
//!
//! See [`ddl_to_script`](crate::ddl_to_script) for the full list of supported DDL types
//! and their mapping to Beamline generators.

use crate::ddl_to_script::{ddl_to_script, DdlConversionError};
use crate::primitives::Sample;
use crate::sim::{ISim, SimBuilder, SimConfigBuilder, SimError, SimResult};
use crate::source::SimSource;
use miette::Diagnostic;
use thiserror::Error;
use time::OffsetDateTime;

/// Errors that can occur when building or using a [`DdlDataGenerator`].
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum DdlGeneratorError {
    /// Error converting DDL to a Beamline script.
    #[error("DDL conversion error: {0}")]
    DdlConversion(#[from] DdlConversionError),

    /// Error in simulation configuration or execution.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Sim(#[from] SimError),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

pub type DdlGeneratorResult<T> = Result<T, DdlGeneratorError>;

/// Builder for constructing a [`DdlDataGenerator`].
///
/// # Required Fields
/// - `ddl`: The DDL column definitions string
///
/// # Optional Fields
/// - `dataset_name`: Name for the generated dataset (default: `"data"`)
/// - `seed`: Random seed for reproducible generation (default: auto-generated)
/// - `t0`: Start time for the simulation (default: auto-generated)
///
/// # Example
///
/// ```rust
/// use partiql_beamline::ddl_generator::DdlDataGenerator;
///
/// let generator = DdlDataGenerator::builder()
///     .ddl(r#""id" VARCHAR, "value" DOUBLE"#)
///     .seed(42)
///     .build()
///     .expect("build");
/// ```
#[derive(Debug, Clone)]
pub struct DdlDataGeneratorBuilder {
    ddl: Option<String>,
    dataset_name: String,
    seed: Option<u64>,
    t0: Option<OffsetDateTime>,
    nullability: Option<f64>,
    optionality: Option<f64>,
}

impl Default for DdlDataGeneratorBuilder {
    fn default() -> Self {
        Self {
            ddl: None,
            dataset_name: "data".to_string(),
            seed: None,
            t0: None,
            nullability: None,
            optionality: None,
        }
    }
}

impl DdlDataGeneratorBuilder {
    /// Set the DDL column definitions string.
    ///
    /// This is the format output by Beamline's `infer-shape --output-format basic-ddl`:
    /// ```text
    /// "column_name" TYPE,
    /// "another_column" TYPE
    /// ```
    pub fn ddl(mut self, ddl: impl Into<String>) -> Self {
        self.ddl = Some(ddl.into());
        self
    }

    /// Set the dataset name (default: `"data"`).
    pub fn dataset_name(mut self, name: impl Into<String>) -> Self {
        self.dataset_name = name.into();
        self
    }

    /// Set a specific random seed for reproducible generation.
    /// If not set, a random seed will be auto-generated.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the simulation start time.
    /// If not set, a start time will be auto-generated from the seed.
    pub fn t0(mut self, t0: OffsetDateTime) -> Self {
        self.t0 = Some(t0);
        self
    }

    /// Set the default nullability percentage (0.0 to 1.0).
    /// If set, generated values may be `NULL` at the given rate.
    pub fn nullability(mut self, pct: f64) -> Self {
        self.nullability = Some(pct);
        self
    }

    /// Set the default optionality percentage (0.0 to 1.0).
    /// If set, generated values may be `MISSING` at the given rate.
    pub fn optionality(mut self, pct: f64) -> Self {
        self.optionality = Some(pct);
        self
    }

    /// Build the [`DdlDataGenerator`].
    ///
    /// # Errors
    /// Returns an error if:
    /// - `ddl` was not set
    /// - The DDL cannot be parsed
    /// - The simulation cannot be configured
    pub fn build(self) -> DdlGeneratorResult<DdlDataGenerator> {
        let ddl = self
            .ddl
            .ok_or(DdlGeneratorError::MissingField("ddl"))?;

        // Convert DDL to Ion script
        let script = ddl_to_script(&ddl, &self.dataset_name)?;

        // Build SimConfig
        let mut cfg_builder = SimConfigBuilder::default();
        if let Some(seed) = self.seed {
            cfg_builder.seed(seed);
        }
        if let Some(t0) = self.t0 {
            cfg_builder.t0(t0);
        }
        if let Some(null_pct) = self.nullability {
            cfg_builder.nullability(Some(null_pct));
        }
        if let Some(opt_pct) = self.optionality {
            cfg_builder.optionality(Some(opt_pct));
        }
        let config = cfg_builder.build().map_err(|e| SimError::ConfigBuilderError(e))?;

        // Build SimSource and Sim
        let source = SimSource::new("<ddl>", script).map_err(SimError::SimSourceError)?;
        let sim = SimBuilder::from_config(config, source)?.build_time_ordered()?;

        Ok(DdlDataGenerator { sim })
    }
}

/// A data generator that produces samples from DDL column definitions.
///
/// Created via [`DdlDataGenerator::builder()`]. Implements [`Iterator`] over
/// [`SimResult<Sample>`], where each [`Sample`] contains a `tick` and a `value`
/// (a [`partiql_value::Value`]).
///
/// # Example
///
/// ```rust
/// use partiql_beamline::ddl_generator::DdlDataGenerator;
///
/// let mut gen = DdlDataGenerator::builder()
///     .ddl(r#""id" VARCHAR, "temp" DOUBLE"#)
///     .dataset_name("sensors")
///     .seed(42)
///     .build()
///     .unwrap();
///
/// // Take 10 samples
/// for sample in gen.take(10) {
///     let sample = sample.unwrap();
///     println!("tick={:?}, value={:?}", sample.tick, sample.value);
/// }
/// ```
pub struct DdlDataGenerator {
    sim: crate::sim::Sim,
}

impl DdlDataGenerator {
    /// Create a new builder for configuring a [`DdlDataGenerator`].
    pub fn builder() -> DdlDataGeneratorBuilder {
        DdlDataGeneratorBuilder::default()
    }

    /// Generate the next sample.
    ///
    /// Returns `Ok(Some(sample))` if a sample was generated,
    /// `Ok(None)` if the simulation has ended (unlikely for DDL-based generation),
    /// or `Err(...)` if an error occurred.
    pub fn next_sample(&mut self) -> SimResult<Option<Sample>> {
        self.sim.next_sample()
    }

    /// Get the seed used for this generator.
    pub fn seed(&self) -> u64 {
        self.sim.config().seed
    }

    /// Get the start time (t0) used for this generator.
    pub fn t0(&self) -> OffsetDateTime {
        self.sim.config().t0
    }

    /// Get the shape (type mapping) of the generated datasets.
    pub fn shape(&self) -> crate::sim::DatasetTypeMapping {
        use crate::sim::ISim;
        self.sim.shape()
    }
}

impl Iterator for DdlDataGenerator {
    type Item = SimResult<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sim.next_sample().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_value::Value;

    #[test]
    fn test_basic_generation() {
        let mut gen = DdlDataGenerator::builder()
            .ddl(r#""sensor_id" VARCHAR, "temperature" DOUBLE, "active" BOOL"#)
            .dataset_name("sensors")
            .seed(42)
            .build()
            .expect("build");

        let samples: Vec<_> = gen.take(5).collect();
        assert_eq!(samples.len(), 5);

        for sample in &samples {
            let sample = sample.as_ref().expect("sample should be Ok");
            // Each sample value should be a Tuple (struct) with 3 fields
            match &sample.value {
                Value::Tuple(t) => {
                    // Tuple should have 3 pairs (sensor_id, temperature, active)
                    let pairs: Vec<_> = t.pairs().collect();
                    assert_eq!(pairs.len(), 3, "Expected 3 fields in tuple");
                }
                other => panic!("Expected Tuple, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_reproducible_with_seed() {
        let collect_values = |seed: u64| -> Vec<Value> {
            DdlDataGenerator::builder()
                .ddl(r#""id" VARCHAR, "value" DOUBLE"#)
                .seed(seed)
                .build()
                .unwrap()
                .take(3)
                .map(|s| s.unwrap().value)
                .collect()
        };

        let run1 = collect_values(123);
        let run2 = collect_values(123);
        let run3 = collect_values(456);

        assert_eq!(run1, run2, "Same seed should produce same values");
        assert_ne!(run1, run3, "Different seeds should produce different values");
    }

    #[test]
    fn test_default_dataset_name() {
        let gen = DdlDataGenerator::builder()
            .ddl(r#""x" DOUBLE"#)
            .seed(1)
            .build()
            .expect("build");

        // The default dataset name is "data"
        let shape = gen.shape();
        assert!(shape.get_shape("data").is_some(), "default dataset name should be 'data'");
    }

    #[test]
    fn test_custom_dataset_name() {
        let gen = DdlDataGenerator::builder()
            .ddl(r#""x" DOUBLE"#)
            .dataset_name("my_table")
            .seed(1)
            .build()
            .expect("build");

        let shape = gen.shape();
        assert!(shape.get_shape("my_table").is_some());
    }

    #[test]
    fn test_complex_ddl_types() {
        let ddl = r#"
            "id" VARCHAR,
            "count" INT,
            "price" DECIMAL(5, 2),
            "timestamp" TIMESTAMP,
            "tags" ARRAY<VARCHAR>,
            "active" BOOL
        "#;

        let mut gen = DdlDataGenerator::builder()
            .ddl(ddl)
            .dataset_name("products")
            .seed(99)
            .build()
            .expect("build");

        let sample = gen.next_sample().expect("no error").expect("has sample");
        match &sample.value {
            Value::Tuple(t) => {
                let pairs: Vec<_> = t.pairs().collect();
                assert_eq!(pairs.len(), 6, "Expected 6 fields in tuple");
                // Verify field names
                let field_names: Vec<_> = pairs.iter().map(|(k, _)| k.to_string()).collect();
                assert!(field_names.contains(&"id".to_string()));
                assert!(field_names.contains(&"count".to_string()));
                assert!(field_names.contains(&"price".to_string()));
                assert!(field_names.contains(&"timestamp".to_string()));
                assert!(field_names.contains(&"tags".to_string()));
                assert!(field_names.contains(&"active".to_string()));
            }
            other => panic!("Expected Tuple, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_ddl_error() {
        let result: DdlGeneratorResult<DdlDataGenerator> = DdlDataGenerator::builder()
            .seed(1)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ddl_error() {
        let result = DdlDataGenerator::builder()
            .ddl("")
            .seed(1)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_seed_and_t0_accessors() {
        let gen = DdlDataGenerator::builder()
            .ddl(r#""x" DOUBLE"#)
            .seed(42)
            .build()
            .expect("build");

        assert_eq!(gen.seed(), 42);
        // t0 should be auto-generated but deterministic from seed
        let _t0 = gen.t0();
    }

    #[test]
    fn test_explicit_t0() {
        use time::macros::datetime;

        let t0 = datetime!(2024-01-01 00:00:00 UTC);
        let gen = DdlDataGenerator::builder()
            .ddl(r#""x" DOUBLE"#)
            .seed(42)
            .t0(t0)
            .build()
            .expect("build");

        assert_eq!(gen.t0(), t0);
    }

    #[test]
    fn test_iterator_is_lazy() {
        // Just building shouldn't generate any data
        let gen = DdlDataGenerator::builder()
            .ddl(r#""x" DOUBLE"#)
            .seed(1)
            .build()
            .expect("build");

        // Can take any number of samples
        let samples: Vec<_> = gen.into_iter().take(100).collect();
        assert_eq!(samples.len(), 100);
    }
}
