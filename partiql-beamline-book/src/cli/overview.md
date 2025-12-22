# CLI Overview

The PartiQL Beamline Command Line Interface (CLI) provides access to all of Beamline's core functionality: data generation, query generation, schema inference, and database creation. The CLI is built using Rust and follows a simple, consistent command structure.

## Installation and Setup

### Building from Source

The CLI is built as part of the PartiQL Beamline project:

```bash
# Clone the repository
git clone https://github.com/partiql/partiql-beamline.git
cd partiql-beamline

# Build the project (includes CLI)
cargo build --release

# The CLI binary will be available at:
./target/release/beamline
```

### Verification

After building, verify the CLI is working:

```bash
# Check version
./target/release/beamline --version

# View help
./target/release/beamline --help
```

## Command Structure

All Beamline CLI commands follow this structure:

```bash
beamline <COMMAND> [SUBCOMMAND] [OPTIONS]
```

## Available Commands

The CLI provides four main commands:

### 1. `gen` - Data and Database Generation

Generate synthetic data and create databases.

**Subcommands:**
- `data` - Generate synthetic data from Ion scripts
- `db beamline-lite` - Create BeamlineLite database with data and schemas

**Example:**
```bash
beamline gen data --seed-auto --start-auto --sample-count 100 --script-path my_script.ion
```

### 2. `infer-shape` - Schema Inference

Infer data schemas from Ion scripts without generating full datasets.

**Example:**
```bash
beamline infer-shape --seed-auto --start-auto --script-path my_script.ion --output-format basic-ddl
```

### 3. `query` - Query Generation

Generate PartiQL queries that match your data structures.

**Subcommands:**
- `basic` - Basic query generation with configurable strategies

**Example:**
```bash
beamline query basic --seed 1234 --start-auto --script-path data_script.ion --sample-count 5 rand-select-all-fw --tbl-flt-rand-min 1 --tbl-flt-rand-max 1 --pred-lt
```

### 4. `help` - Help Information

Display help for commands and subcommands.

## Common Options

Several options are shared across multiple commands:

### Seed Configuration (Required)
Control reproducibility through seeding:

```bash
--seed-auto                    # Generate random seed automatically
--seed <SEED>                  # Use specific seed (e.g., --seed 12345)
```

### Start Time Configuration (Required)
Set the simulation start time:

```bash
--start-auto                   # Generate random start time
--start-epoch-ms <EPOCH_MS>    # Use Unix timestamp in milliseconds
--start-iso <ISO_8601>         # Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)
```

### Script Configuration (Required)
Provide the Ion script defining data generation:

```bash
--script-path <PATH>           # Path to Ion script file
--script <SCRIPT_DATA>         # Inline Ion script content
```

### Sample Count
Control how much data to generate:

```bash
--sample-count <COUNT>         # Number of samples (default: 10)
```

### Nullability and Optionality
Configure NULL and MISSING value generation:

```bash
--default-nullable <true|false>    # Make types nullable by default
--pct-null <PERCENTAGE>            # Percentage of NULL values (0.0-1.0)
--default-optional <true|false>    # Make types optional by default  
--pct-optional <PERCENTAGE>        # Percentage of MISSING values (0.0-1.0)
```

## Output Formats

### Data Generation Formats

For `gen data`, specify output format with `--output-format`:

| Format | Description | Use Case |
|--------|-------------|----------|
| `text` | Human-readable text (default) | Debugging, inspection |
| `ion` | Compact Amazon Ion binary | Efficient storage |
| `ion-pretty` | Pretty-printed Ion text | Human-readable Ion |
| `ion-binary` | Binary Ion format | Most compact |

**Example:**
```bash
beamline gen data --seed-auto --start-auto --script-path data.ion --output-format ion-pretty
```

### Shape Inference Formats

For `infer-shape`, specify format with `--output-format`:

| Format | Description | Use Case |
|--------|-------------|----------|
| `text` | Debug format (default) | Development |
| `basic-ddl` | SQL DDL format | Database schema |
| `beamline-json` | Beamline JSON format | Testing |

## Basic Usage Examples

### Generate Data
```bash
# Simple data generation
beamline gen data \
  --seed-auto \
  --start-auto \
  --sample-count 1000 \
  --script-path sensors.ion

# Reproducible generation with specific seed
beamline gen data \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --sample-count 500 \
  --script-path user_data.ion \
  --output-format ion-pretty
```

### Filter Datasets
Generate data for specific datasets only:

```bash
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path client_service.ion \
  --dataset service \
  --dataset client_1 \
  --sample-count 100
```

### Infer Schema
```bash
# Get SQL DDL schema
beamline infer-shape \
  --seed-auto \
  --start-auto \
  --script-path my_script.ion \
  --output-format basic-ddl

# Get detailed shape information  
beamline infer-shape \
  --seed 1234 \
  --start-auto \
  --script-path complex_data.ion \
  --output-format text
```

### Create Database
```bash
# Create BeamlineLite database
beamline gen db beamline-lite \
  --seed-auto \
  --start-auto \
  --script-path database_script.ion \
  --sample-count 10000

# Custom catalog location
beamline gen db beamline-lite \
  --seed 2024 \
  --start-auto \
  --script-path data.ion \
  --catalog_name my-catalog \
  --catalog_path ./databases/ \
  --sample-count 5000
```

### Generate Queries
```bash
# Simple query generation
beamline query basic \
  --seed 100 \
  --start-auto \
  --script-path transactions.ion \
  --sample-count 10 \
  rand-select-all-fw \
    --tbl-flt-rand-min 1 \
    --tbl-flt-rand-max 3 \
    --tbl-flt-path-depth-max 2 \
    --pred-all
```

## Configuration with Nullability/Optionality

Control NULL and MISSING value generation:

```bash
# Make all types nullable with 10% NULL values
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --pct-null 0.1 \
  --sample-count 1000

# Make types optional with 5% MISSING values
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --pct-optional 0.05 \
  --sample-count 1000

# Disable nullability and optionality
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --default-nullable false \
  --default-optional false \
  --sample-count 1000
```

## Error Handling

### Common Error Types

#### Script Not Found
```bash
$ beamline gen data --seed-auto --start-auto --script-path missing.ion
Error: Unable to read script file 'missing.ion': No such file or directory
```

#### Invalid Ion Script
```bash
$ beamline gen data --seed-auto --start-auto --script-path invalid.ion
Error: Failed to parse Ion script: Invalid syntax at line 5
```

#### Invalid Seed Value
```bash
$ beamline gen data --seed invalid --start-auto --script-path data.ion
Error: Invalid value 'invalid' for '--seed <SEED>': invalid digit found in string
```

### Debug Output

For troubleshooting, examine the generated seed and start time:

```bash
$ beamline gen data --seed-auto --start-auto --script-path sensors.ion --sample-count 2
Seed: 12328924104731257599
Start: 2024-01-20T20:05:41.000000000Z
[2024-01-20 20:07:46.532 +00:00:00] : "sensors" { 'f': -2.5436390152455175, 'i8': 4, 'tick': 125532 }
[2024-01-20 20:09:19.756 +00:00:00] : "sensors" { 'f': -63.49308817145054, 'i8': 4, 'tick': 218756 }
```

The output shows the seed and start time used, allowing you to reproduce the exact same output later.

## Integration Patterns

### Shell Scripting

```bash
#!/bin/bash
# Generate test data for different scenarios

SEED=12345
START_TIME="2024-01-01T00:00:00Z"

# Generate user data
beamline gen data \
  --seed $SEED \
  --start-iso $START_TIME \
  --script-path users.ion \
  --sample-count 1000 \
  --output-format ion-pretty > users.ion

# Generate transaction data
beamline gen data \
  --seed $((SEED + 1)) \
  --start-iso $START_TIME \
  --script-path transactions.ion \
  --sample-count 5000 \
  --output-format ion-pretty > transactions.ion

echo "Data generation completed!"
```

### Pipeline Integration

```bash
# Generate data and pipe to other tools
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path events.ion \
  --sample-count 1000 \
  --output-format ion-pretty | \
  head -20

# Combine with analysis tools
beamline gen data \
  --seed 100 \
  --start-auto \
  --script-path metrics.ion \
  --sample-count 10000 \
  --output-format text | \
  grep "temperature" | \
  wc -l
```

## Best Practices

### 1. Always Use Seeds for Reproducible Testing

```bash
# Good - explicit seed for test scenarios
beamline gen data --seed 12345 --start-iso "2024-01-01T00:00:00Z" --script-path test.ion

# Avoid - auto seed makes reproduction difficult
beamline gen data --seed-auto --start-auto --script-path test.ion
```

### 2. Start Small, Scale Up

```bash
# Test with small sample first
beamline gen data --seed 1 --start-auto --script-path new_script.ion --sample-count 10

# Scale up after validation
beamline gen data --seed 1 --start-auto --script-path new_script.ion --sample-count 100000
```

### 3. Use Appropriate Output Formats

```bash
# Ion formats for data processing
beamline gen data --script-path data.ion --output-format ion-binary

# Text format for debugging
beamline gen data --script-path data.ion --output-format text --sample-count 5
```

### 4. Validate Schemas Before Large Generation

```bash
# Check schema first
beamline infer-shape --seed-auto --start-auto --script-path data.ion --output-format basic-ddl

# Then generate data
beamline gen data --seed 42 --start-auto --script-path data.ion --sample-count 10000
```

## Next Steps

Now that you understand the CLI overview, explore specific commands:

- [Data Commands](./data-commands.md) - Detailed guide to `gen data` command
- [Query Commands](./query-commands.md) - Comprehensive `query` command reference  
- [Shape Commands](./shape-commands.md) - Using `infer-shape` for schema work
- [Database Commands](./database-commands.md) - Creating databases with `gen db`
