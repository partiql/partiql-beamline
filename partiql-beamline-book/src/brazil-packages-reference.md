# Brazil Packages Reference

This document provides references to all Brazil packages used in PartiQL Beamline, available on code.amazon.com.

## Overview

PartiQL Beamline depends on several internal Brazil packages that are part of the PartiQL ecosystem. These packages are maintained on Amazon's internal code repository and are essential for the functionality of PartiQL Beamline.

## PartiQL Core Packages

### partiql-ast (v0.10.1)
**Purpose**: PartiQL Abstract Syntax Tree representation and manipulation  
**Code Repository**: [code.amazon.com/packages/PartiQL-AST](https://code.amazon.com/packages/PartiQL-AST)  
**Used by**: `partiql-beamline-query`, `partiql-beamline-cliargs`  
**Description**: Provides the AST data structures for representing PartiQL queries and expressions.

### partiql-parser (v0.10)
**Purpose**: PartiQL query parsing functionality  
**Code Repository**: [code.amazon.com/packages/PartiQL-Parser](https://code.amazon.com/packages/PartiQL-Parser)  
**Used by**: `partiql-beamline-query`  
**Description**: Handles parsing of PartiQL query strings into AST representations.

### partiql-types (v0.10)
**Purpose**: PartiQL type system definitions and utilities  
**Code Repository**: [code.amazon.com/packages/PartiQL-Types](https://code.amazon.com/packages/PartiQL-Types)  
**Used by**: `partiql-beamline-sim`, `partiql-beamline-query`, `partiql-beamline-serde`  
**Description**: Defines the type system used throughout PartiQL, including type inference and validation.

### partiql-value (v0.10)
**Purpose**: PartiQL value representation and manipulation  
**Code Repository**: [code.amazon.com/packages/PartiQL-Value](https://code.amazon.com/packages/PartiQL-Value)  
**Used by**: `partiql-beamline-cli`, `partiql-beamline-sim`, `partiql-beamline-query`  
**Description**: Core value types and operations for PartiQL data values.

### partiql-extension-ddl (v0.10)
**Purpose**: Data Definition Language extensions for PartiQL  
**Code Repository**: [code.amazon.com/packages/PartiQL-Extension-DDL](https://code.amazon.com/packages/PartiQL-Extension-DDL)  
**Used by**: `partiql-beamline-cli`, `partiql-beamline-serde`  
**Description**: Provides DDL capabilities for creating and managing PartiQL schemas.

### partiql-extension-ion (v0.10)
**Purpose**: Ion format integration for PartiQL  
**Code Repository**: [code.amazon.com/packages/PartiQL-Extension-Ion](https://code.amazon.com/packages/PartiQL-Extension-Ion)  
**Used by**: `partiql-beamline-cli`, `partiql-beamline-sim`, `partiql-beamline-query`  
**Description**: Enables PartiQL to work with Amazon Ion data format, including serialization and deserialization.

## Internal PartiQL Beamline Packages

### partiql-beamline-cli
**Purpose**: Command-line interface for PartiQL Beamline  
**Code Repository**: [code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-cli](https://code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-cli)  
**Description**: Main CLI application providing data generation, query generation, and schema inference capabilities.

### partiql-beamline-sim
**Purpose**: Core simulation and data generation engine  
**Code Repository**: [code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-sim](https://code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-sim)  
**Description**: Contains the core data generation algorithms, Ion script processing, and simulation engine.

### partiql-beamline-query
**Purpose**: Query generation and inference functionality  
**Code Repository**: [code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-query](https://code.amazon.com/packages/PartiQL-Beamline/trees/mainline/--/partiql-beamline-query)  
