# Data Generation Commands

The `beamline gen data` command generates synthetic data from Ion scripts using stochastic processes. This is the primary command for creating reproducible pseudo-random data in Beamline.

## Command Syntax

```bash
beamline gen data [OPTIONS]
```

## Required Options

All data generation requires these three configuration groups (exactly one option from each group):

### Seed Configuration (Required - choose one)
```bash
--seed-auto                    # Generate random seed automatically
--seed <SEED>                  # Use specific numeric seed for reproducibility
```

### Start Time Configuration (Required - choose one)
```bash
--start-auto                   # Generate random start time
--start-epoch-ms <EPOCH_MS>    # Use Unix timestamp in milliseconds
--start-iso <ISO_8601>         # Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)
```

### Script Configuration (Required - choose one)
```bash
--script-path <PATH>           # Path to Ion script file
--script <SCRIPT_DATA>         # Inline Ion script content
--ddl-path <PATH>              # Path to a DDL file (column definitions)
--ddl <DDL_DATA>               # Inline DDL column definitions
```

### DDL Dataset Name
```bash
--dataset-name <NAME>          # Dataset name for DDL-based generation (default: "data")
```

## Optional Parameters

### Sample Count
```bash
--sample-count <COUNT>         # Number of samples to generate (default: 10)
```

### Output Format
```bash
--output-format <FORMAT>       # Output format (default: text)
```

Available formats:
- `text` - Human-readable text format (default)
- `ion` - Compact Amazon Ion format  
- `ion-pretty` - Pretty-printed Ion text format
- `ion-binary` - Binary Ion format (most compact)
- `parquet` - Apache Parquet columnar format (requires `--output-path`)

### Output Path (for file-based formats)
```bash
--output-path <OUTPUT_PATH>    # Output directory for Parquet files
```

### Dataset Filtering
```bash
--dataset <DATASET_NAME>       # Include only specific dataset(s)
                              # Can be used multiple times for multiple datasets
```

### Nullability Configuration (Optional - choose one)
```bash
--default-nullable <true|false>    # Set default nullability behavior
--pct-null <PERCENTAGE>            # Percentage of NULL values (0.0-1.0)
```

### Optionality Configuration (Optional - choose one)
```bash
--default-optional <true|false>    # Set default optionality behavior  
--pct-optional <PERCENTAGE>        # Percentage of MISSING values (0.0-1.0)
```

## Basic Examples

### Simple Data Generation

```bash
# Generate 100 samples with automatic seed and start time
beamline gen data \
  --seed-auto \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 100

# Reproducible generation with specific seed
beamline gen data \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path user_data.ion \
  --sample-count 1000
```

### Different Output Formats

```bash
# Text output (human-readable, default)
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --output-format text

# Pretty Ion format
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --output-format ion-pretty

# Compact binary Ion
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --output-format ion-binary

# Parquet format (writes to files)
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --output-format parquet \
  --output-path ./output
```

### DDL-Based Generation

```bash
# Generate data from inline DDL
beamline gen data \
  --seed 42 \
  --start-auto \
  --ddl '"id" VARCHAR, "temperature" DOUBLE, "active" BOOL' \
  --dataset-name sensors \
  --sample-count 100 \
  --output-format ion-pretty

# Generate from a DDL file
beamline gen data \
  --seed 42 \
  --start-auto \
  --ddl-path schema.ddl \
  --dataset-name products \
  --sample-count 1000 \
  --output-format parquet \
  --output-path ./output
```

### Dataset Filtering

Generate data for specific datasets only:

```bash
# Generate data for specific datasets
beamline gen data \
  --seed 45121008347100595 \
  --start-iso "2020-06-16T14:41:51.000000000Z" \
  --script-path client-service.ion \
  --sample-count 10 \
  --dataset service \
  --dataset client_1 \
  --output-format ion-pretty
```

## Advanced Configuration

### Controlling NULL Values

```bash
# Make all types nullable by default with 10% NULL values
beamline gen data \
  --seed 100 \
  --start-auto \
  --script-path data.ion \
  --pct-null 0.1 \
  --sample-count 500

# Disable nullability entirely
beamline gen data \
  --seed 100 \
  --start-auto \
  --script-path data.ion \
  --default-nullable false \
  --sample-count 500
```

### Controlling MISSING Values

```bash
# Make all types optional with 5% MISSING values
beamline gen data \
  --seed 200 \
  --start-auto \
  --script-path data.ion \
  --pct-optional 0.05 \
  --sample-count 500

# Disable optionality entirely
beamline gen data \
  --seed 200 \
  --start-auto \
  --script-path data.ion \
  --default-optional false \
  --sample-count 500
```

### Inline Scripts

For small scripts, you can provide the Ion script content directly:

```bash
beamline gen data \
  --seed 300 \
  --start-auto \
  --script 'rand_processes::{ test: rand_process::{ $arrival: HomogeneousPoisson:: { interarrival: seconds::1 }, $data: { id: UniformU8, value: UniformF64 } } }' \
  --sample-count 5 \
  --output-format text
```

## Reproducibility Examples

### Exact Reproduction

```bash
# First run - note the seed and start time
beamline gen data \
  --seed-auto \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 2

# Output shows:
# Seed: 12328924104731257599
# Start: 2024-01-20T20:05:41.000000000Z
# [data follows...]

# Reproduce exactly the same data
beamline gen data \
  --seed 12328924104731257599 \
  --start-iso "2024-01-20T20:05:41.000000000Z" \
  --script-path sensors.ion \
  --sample-count 2
```

### Reproducible with Different Start Times

```bash
# Same seed, different start time gives same data pattern at different times
beamline gen data \
  --seed 12345 \
  --start-iso "2023-01-01T00:00:00Z" \
  --script-path events.ion \
  --sample-count 5

beamline gen data \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path events.ion \
  --sample-count 5
```

## Output Format Details

### Text Format (Default)

Human-readable format with timestamps and dataset names:

```bash
$ beamline gen data --seed 1234 --start-auto --script-path sensors.ion --sample-count 2
Seed: 1234
Start: 2019-08-01T00:00:01.000000000-07:00
[2019-08-01 7:26:21.964 -07:00:00] : "sensors" { 'f': -2.5436390152455175, 'i8': 4, 'tick': 125532 }
[2019-08-10 5:46:15.24 -07:00:00] : "sensors" { 'f': -63.49308817145054, 'i8': 4, 'tick': 218756 }
```

### Ion Pretty Format

Pretty-printed Ion with metadata:

```bash
$ beamline gen data --seed 1234 --start-auto --script-path sensors.ion --sample-count 2 --output-format ion-pretty
{
  seed: 1234,
  start: "2019-08-01T00:00:01.000000000-07:00",
  data: {
    sensors: [
      {
        f: -2.5436390152455175e0,
        i8: 4,
        tick: 125532
      },
      {
        f: -63.49308817145054e0,
        i8: 4,
        tick: 218756
      }
    ]
  }
}
```

### Ion and Ion Binary Formats

- `ion` - Compact text Ion without pretty printing
- `ion-binary` - Binary Ion format (most space-efficient)

Both formats preserve all Ion type information and are suitable for programmatic processing.

## Static Data Generation

Beamline supports static data generation (data generated before simulation starts):

```bash
# Generate data with static customer table and dynamic orders
beamline gen data \
  --seed 1234 \
  --start-iso "2019-08-01T00:00:01-07:00" \
  --script-path orders.ion \
  --sample-count 30 \
  --output-format text
```

Static data appears first with the same timestamp, followed by temporally-distributed dynamic data.

## Error Handling

### Common Error Scenarios

#### Missing Script File
```bash
$ beamline gen data --seed-auto --start-auto --script-path nonexistent.ion
Error: Failed to read script file 'nonexistent.ion': No such file or directory (os error 2)
```

#### Invalid Ion Syntax
```bash
$ beamline gen data --seed-auto --start-auto --script-path invalid.ion
Error: Failed to parse Ion script: Invalid Ion syntax at line 5, column 10
```

#### Missing Required Arguments
```bash
$ beamline gen data --script-path data.ion
Error: One of --seed-auto or --seed is required
Error: One of --start-auto, --start-epoch-ms, or --start-iso is required
```

#### Invalid Percentage Values
```bash
$ beamline gen data --seed-auto --start-auto --script-path data.ion --pct-null 1.5
Error: Percents must be between 0 and 1: `1.5`
```

### Debugging Tips

1. **Start Small**: Use `--sample-count 5` to quickly test scripts
2. **Use Text Format**: Default text format is easiest to read for debugging
3. **Check Seeds**: Note auto-generated seeds for reproduction
4. **Validate Scripts**: Use `infer-shape` to check script syntax first

## Integration Patterns

### Shell Scripting

```bash
#!/bin/bash
set -e

SCRIPT_PATH="simulation.ion"
OUTPUT_DIR="./generated_data"
SEED=12345

mkdir -p "$OUTPUT_DIR"

# Generate different datasets
echo "Generating user data..."
beamline gen data \
  --seed $SEED \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path "$SCRIPT_PATH" \
  --dataset users \
  --sample-count 1000 \
  --output-format ion-pretty > "$OUTPUT_DIR/users.ion"

echo "Generating transaction data..."
beamline gen data \
  --seed $((SEED + 1)) \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path "$SCRIPT_PATH" \
  --dataset transactions \
  --sample-count 5000 \
  --output-format ion-pretty > "$OUTPUT_DIR/transactions.ion"

echo "Data generation completed!"
```

### Pipeline Processing

```bash
# Generate and process data in pipeline
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path metrics.ion \
  --sample-count 1000 \
  --output-format text | \
  grep "temperature" | \
  awk '{ print $NF }' | \
  head -10

# Generate multiple formats simultaneously
beamline gen data \
  --seed 100 \
  --start-auto \
  --script-path data.ion \
  --sample-count 1000 \
  --output-format ion-pretty | \
  tee data.ion | \
  head -20
```

### Testing Workflows

```bash
# Generate test data with specific characteristics
generate_test_data() {
  local seed=$1
  local sample_count=$2
  local script=$3
  
  beamline gen data \
    --seed "$seed" \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path "$script" \
    --sample-count "$sample_count" \
    --default-nullable false \
    --default-optional false \
    --output-format ion-pretty
}

# Use in tests
generate_test_data 12345 100 "test_users.ion" > test_users.ion
generate_test_data 12346 50 "test_orders.ion" > test_orders.ion
```

## Performance Considerations

### Sample Count Impact

- Small counts (`--sample-count 10-100`): Near-instantaneous
- Medium counts (`--sample-count 1000-10000`): Seconds
- Large counts (`--sample-count 100000+`): Minutes, depending on script complexity

### Output Format Performance

1. `text` - Moderate performance, human-readable
2. `ion-binary` - Fastest and most compact
3. `ion` - Fast, compact text format
4. `ion-pretty` - Slowest due to formatting overhead

### Memory Usage

Beamline streams data generation, so memory usage stays constant regardless of sample count. Large datasets are processed incrementally.

## Best Practices

### 1. Use Specific Seeds for Testing
```bash
# Good - reproducible
beamline gen data --seed 12345 --start-iso "2024-01-01T00:00:00Z" --script-path test.ion

# Avoid - non-reproducible
beamline gen data --seed-auto --start-auto --script-path test.ion
```

### 2. Start with Small Sample Counts
```bash
# Validate script first
beamline gen data --seed 1 --start-auto --script-path new_script.ion --sample-count 5

# Scale up after validation
beamline gen data --seed 1 --start-auto --script-path new_script.ion --sample-count 10000
```

### 3. Use Appropriate Output Formats
```bash
# Human inspection
beamline gen data --script-path data.ion --output-format text --sample-count 10

# Data processing
beamline gen data --script-path data.ion --output-format ion-binary --sample-count 100000

# Configuration files
beamline gen data --script-path data.ion --output-format ion-pretty --sample-count 1000
```

### 4. Document Your Seeds
```bash
# Good practice - document seeds used
# User test data: seed 2024001
# Integration test data: seed 2024002  
# Performance test data: seed 2024003

beamline gen data --seed 2024001 --start-auto --script-path users.ion
```

## Next Steps

Now that you understand data generation commands, explore:

- [Query Commands](./query-commands.md) - Generate PartiQL queries for your data
- [Shape Commands](./shape-commands.md) - Infer and work with data schemas
- [Database Commands](./database-commands.md) - Create complete databases with data and schemas
- [Data Generation Guide](../data-generation/overview.md) - Learn about Ion scripts and generators
