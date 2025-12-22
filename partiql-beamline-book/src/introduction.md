# Introduction

Welcome to the **Beamline Guide** — your comprehensive resource for mastering synthetic data and query generation 
for your  AI/ML, testing, and simulation use-cases.

## What You'll Learn

This guide will take you on a journey from understanding the basics of Beamline to becoming proficient in generating
sophisticated synthetic datasets and queries. Whether you are an AI/ML researcher, a developer looking to test your 
implementations, or a data scientist needing realistic test data, this guide has you covered.

## How This Guide is Organized

The guide is structured to gradually build your understanding and skills:

1. **Getting Started** — Learn what Beamline is and get your first data generation running
2. **Understanding the Basics** — Grasp core concepts like random processes and reproducible generation
3. **Data Generation** — Master the art of creating synthetic data with various types and patterns
4. **Query Generation** — Learn to generate PartiQL queries that match your data shapes
5. **Schema and Shape Inference** — Understand how to work with data schemas and type inference
6. **Database Generation** — Create local BeamlineLite database with both data and schemas
7. **Command Line Interface** — Become proficient with all CLI commands and options
10. **Examples and Tutorials** — Hands-on tutorials with real-world scenarios

## What is Beamline?

Beamline is a tool for **fast data generation**. 
It generates **reproducible pseudo-random data** using a **stochastic approach** and **statistical distributions**, 
meaning you can create realistic datasets that follow specific mathematical patterns. This makes the data both random
enough to be useful for  AI/ML model training, simulation, and testing purposing, while remaining deterministic enough 
to be reproducible for debugging and validation.

The tool's ability to generate data based on statistical distributions makes it particularly valuable for AI model 
training scenarios where you need synthetic data that resembles specific population distributions or statistical characteristics.

### Key Features

- **Reproducible Data Generation**: Use seeds to generate the same data every time
- **Stochastic Processes**: Model real-world data patterns using mathematical distributions
- **Query Generation**: Automatically generate PartiQL/SQL-like queries that match your data shapes
- **Schema Inference**: Automatically infer and export data schemas in various formats
- **Multiple Output Formats**: Support for [Amazon Ion](https://amazon-ion.github.io/ion-docs/), JSON, and SQL DDL
- **Database Generation**: Create complete local copies of the generated data with both data and schemas
- **Flexible Configuration**: Highly configurable through Ion-based scripts

## Prerequisites

This guide assumes basic familiarity with:
- Command-line interfaces
- Ion/JSON data formats
- Basic understanding of databases and queries
- Rust programming language (for building from source)

Don't worry if you're new to some of these concepts — we'll explain everything you need to know as we go!

## Getting Help

If you encounter issues or have questions while following this guide:
- Check the [Troubleshooting](./reference/troubleshooting.md) section
- Review the [Command Reference](./reference/commands.md) for detailed CLI options
- Visit the [PartiQL Beamline GitHub repository](https://github.com/partiql/partiql-beamline) for the latest updates and community support

Let's begin your journey with PartiQL Beamline!
