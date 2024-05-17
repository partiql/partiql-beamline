# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Changed the command line hierarchy to separate data generation and shape inference.
### Added
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
