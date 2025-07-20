# Introduction

Welcome to the **PartiQL Beamline Guide** – your comprehensive resource for mastering synthetic data and query generation for PartiQL testing and experimentation.

## What You'll Learn

This guide will take you on a journey from understanding the basics of PartiQL Beamline to becoming proficient in generating sophisticated synthetic datasets and queries. Whether you're a developer looking to test your PartiQL implementations, a data engineer needing realistic test data, or a researcher exploring query patterns, this guide has you covered.

## How This Guide is Organized

The guide is structured to gradually build your understanding and skills:

1. **Getting Started** - Learn what PartiQL Beamline is and get your first data generation running
2. **Understanding the Basics** - Grasp core concepts like random processes and reproducible generation
3. **Data Generation** - Master the art of creating synthetic data with various types and patterns
4. **Query Generation** - Learn to generate PartiQL queries that match your data shapes
5. **Schema and Shape Inference** - Understand how to work with data schemas and type inference
6. **Database Generation** - Create complete databases with both data and schemas
7. **Command Line Interface** - Become proficient with all CLI commands and options
8. **Advanced Topics** - Explore custom generators, performance optimization, and integration patterns
9. **Reference** - Quick access to all generators, configurations, and commands
10. **Examples and Tutorials** - Hands-on tutorials with real-world scenarios

## What is PartiQL Beamline?

PartiQL Beamline is a powerful tool for **fast data generation** designed specifically for PartiQL testing and experimentation. It generates **reproducible pseudo-random data** using a **stochastic approach**, meaning you can create realistic datasets that are both random enough to be useful for testing and deterministic enough to be reproducible for debugging and validation.

### Key Features

- **Reproducible Data Generation**: Use seeds to generate the same data every time
- **Stochastic Processes**: Model real-world data patterns using mathematical distributions
- **Query Generation**: Automatically generate PartiQL queries that match your data shapes
- **Schema Inference**: Automatically infer and export data schemas in various formats
- **Multiple Output Formats**: Support for Ion, JSON, SQL DDL, and more
- **Database Generation**: Create complete databases with both data and schemas
- **Flexible Configuration**: Highly configurable through Ion-based scripts

### Why Use PartiQL Beamline?

- **Testing**: Generate realistic test data for your PartiQL implementations
- **Benchmarking**: Create consistent datasets for performance testing
- **Development**: Quickly prototype with realistic data without manual data creation
- **Research**: Explore query patterns and data relationships
- **Education**: Learn PartiQL with hands-on examples and realistic datasets

## Prerequisites

This guide assumes basic familiarity with:
- Command-line interfaces
- JSON/Ion data formats
- Basic understanding of databases and queries
- Rust programming language (for building from source)

Don't worry if you're new to some of these concepts – we'll explain everything you need to know as we go!

## Getting Help

If you encounter issues or have questions while following this guide:
- Check the [Troubleshooting](./reference/troubleshooting.md) section
- Review the [Command Reference](./reference/commands.md) for detailed CLI options
- Visit the [PartiQL Beamline GitHub repository](https://github.com/partiql/partiql-beamline) for the latest updates and community support

Let's begin your journey with PartiQL Beamline!
