# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Changed references of Kollider to Beamline JSON and DB.
- Changed the command line hierarchy to separate data generation and shape inference.
### Added
- Added Parquet output format (`--output-format parquet --output-path <dir>`) for columnar data generation; supports scalars, nested structs, and native List columns for arrays with scalar elements.
- Added DDL-based data generation (`--ddl` / `--ddl-path`) allowing data generation directly from SQL-like column definitions without writing Ion scripts.
- Added `DdlDataGenerator` programmatic API for DDL-based generation from Rust code.
- Added Weibull, Normal, LogNormal, Exponential distributions
- Added timestamp generators
- Added generation of queries based on the shape of the data generator data
- Added ability to script 'density' (i.e. `NULL` or `MISSING` possibility and probability)
- Added Basic DDL to KolliderDB output as `.shape.sql` files and `infer-shape` as `basic-ddl`. 
- Added `UniformAnyOf` support for `UniformArray`
- Added `UniformArray` for generating values with Array type.
- Added `LoremIpsum`, `LoremIpsumTitle`, and `Regex` generators.
- Added `static_data` generation.
- Added `UniformAnyOf` for generating values with Union type.
- Added `gen db kollider` for creating KolliderDB database.
- Added `schema` to `shape` for both CLI and other constructs.
- Added `UniformDecimal` type to scripts.
- Added `--partiql-kollider` that generates a [PartiQL Kollider](https://github.com/partiql/partiql-lang-kotlin/tree/main/plugins/partiql-local) shape (schema) representation.
- Added `schema` command to the CLI that infers the schema of data that a script generates
- Added --output-format to the CLI with Ion, Ion Pretty, and TEXT (default) format
- Added --sample-count to CLI along with a default value
- Added current tick generation capability with `Tick` keyword in scripts
- Added initial partiql-beamline data generator
- Added CLI

### Fixes
