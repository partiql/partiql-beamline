# Database Commands

The `beamline gen db beamline-lite` command creates complete BeamlineLite databases containing both synthetic data and inferred schemas. This provides a complete local database for testing and development.

## Command Syntax

```bash
beamline gen db beamline-lite [OPTIONS]
```

## Required Options

Database generation uses the same core configuration as data generation:

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
```

## Optional Parameters

### Sample Count
```bash
--sample-count <COUNT>         # Number of samples to generate (default: 10)
```

### Catalog Configuration
```bash
--catalog_name <NAME>          # Name of the catalog directory (default: "beamline-catalog")
--catalog_path <PATH>          # Path where catalog will be created (default: ".")
--force                        # Overwrite existing catalog (creates backup first)
```

### Target
```bash
--target filesystem            # Create filesystem-based database (default and only option)
```

### Nullability and Optionality
```bash
--default-nullable <true|false>    # Set default nullability behavior
--pct-null <PERCENTAGE>            # Percentage of NULL values (0.0-1.0)
--default-optional <true|false>    # Set default optionality behavior  
--pct-optional <PERCENTAGE>        # Percentage of MISSING values (0.0-1.0)
```

## What Gets Created

A BeamlineLite database consists of multiple files in a catalog directory:

### Catalog Structure

```
beamline-catalog/
├── .beamline-manifest          # Metadata (seed, start time, DDL syntax version)
├── .beamline-script           # Original Ion script used for generation
├── <dataset_name>.ion         # Data files (one per dataset)
├── <dataset_name>.shape.ion   # Schema files in Ion format
└── <dataset_name>.shape.sql   # Schema files in SQL DDL format
```

### Example Catalog Contents

After running:
```bash
beamline gen db beamline-lite \
  --seed-auto \
  --start-auto \
  --script-path client-service.ion \
  --sample-count 1000
```

**Generated files:**
```
beamline-catalog/
├── .beamline-manifest
├── .beamline-script  
├── service.ion
├── service.shape.ion
├── service.shape.sql
├── client_0.ion
├── client_0.shape.ion
├── client_0.shape.sql
├── client_1.ion
├── client_1.shape.ion
├── client_1.shape.sql
└── ... (more client datasets)
```

## File Contents

### Manifest File

Contains generation metadata:

```bash
$ cat beamline-catalog/.beamline-manifest
{"seed": "949665520117506306", "start": "2023-02-06T12:52:29.000000000Z", "ddl_syntax.version": "partiql_datatype_syntax.0.1"}
```

### Script File

Original Ion script used for generation:

```bash
$ cat beamline-catalog/.beamline-script
rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },
    
    // A generator for client ids
    $id_gen: UUID,
    
    // ... rest of script
}
```

### Data Files

Generated synthetic data in Ion format:

```bash
$ cat beamline-catalog/client_0.ion
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "0de35d1e-a87c-e540-734d-6f2a4fa410c3", request_time: 2021-01-05T03:55:01.035000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "3539cdf0-6f7e-6bdc-c25a-4e0b7d8f8bac", request_time: 2021-01-05T03:55:01.182000000+00:00}
```

### Schema Files

Ion format schema:
```bash
$ cat beamline-catalog/client_0.shape.ion
{
  type: "bag",
  items: {
    type: "struct",
    constraints: [ordered, closed],
    fields: [
      { name: "id", type: "string" },
      { name: "request_id", type: "string" },
      { name: "request_time", type: "datetime" },
      { name: "success", type: "bool" }
    ]
  }
}
```

SQL DDL format schema:
```bash
$ cat beamline-catalog/service.shape.sql
"Account" VARCHAR,
"Distance" DECIMAL(2, 0),
"Operation" VARCHAR,
"Program" VARCHAR,
"Request" VARCHAR,
"StartTime" TIMESTAMP,
"Weight" DECIMAL(5, 4),
"client" VARCHAR,
"success" BOOL
```

## Examples

### Basic Database Creation

```bash
# Create database with default settings
beamline gen db beamline-lite \
  --seed-auto \
  --start-auto \
  --script-path my_data.ion \
  --sample-count 1000

# Creates ./beamline-catalog/ with all files
```

### Custom Catalog Configuration

```bash
# Create database in custom location
beamline gen db beamline-lite \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path production_sim.ion \
  --sample-count 50000 \
  --catalog_name production-data \
  --catalog_path ./databases/
  
# Creates ./databases/production-data/ with all files
```

### Reproducible Database Creation

```bash
# Create reproducible test database
beamline gen db beamline-lite \
  --seed 2024 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path test_suite.ion \
  --sample-count 10000 \
  --catalog_name test-db-2024 \
  --default-nullable false \
  --default-optional false
```

## Overwriting and Backup

### Safe Overwrite with Backup

The CLI protects existing catalogs by default:

```bash
$ beamline gen db beamline-lite --seed-auto --start-auto --script-path data.ion
creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17)
```

Use `--force` to overwrite with automatic backup:

```bash
$ beamline gen db beamline-lite \
    --seed-auto \
    --start-auto \
    --script-path data.ion \
    --force

command is using --force ...
Beamline catalog ./beamline-catalog/ exists, backing it up to "beamline-catalog.2024-05-10T22:15:54.019316000Z.bkp"...
back up completed
writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!
```

## Database Structure Analysis

### Examine Generated Database

```bash
# View catalog structure
tree beamline-catalog/

# Examine manifest
cat beamline-catalog/.beamline-manifest

# Check a data file
head -5 beamline-catalog/service.ion

# Check schema
cat beamline-catalog/service.shape.sql
```

### Validate Database Consistency

```bash
# Count records in each dataset
for data_file in beamline-catalog/*.ion; do
  if [[ "$data_file" != *".shape.ion"* ]]; then
    echo "$(basename "$data_file"): $(wc -l < "$data_file") records"
  fi
done
```

## Integration Patterns

### Testing Database Setup

```bash
#!/bin/bash
# setup-test-database.sh

TEST_SEED=12345
TEST_START="2024-01-01T00:00:00Z"
TEST_SAMPLES=10000

echo "Creating test database..."

# Clean up any existing test database
rm -rf test-database/

# Generate test database
beamline gen db beamline-lite \
  --seed $TEST_SEED \
  --start-iso $TEST_START \
  --script-path test_data_spec.ion \
  --sample-count $TEST_SAMPLES \
  --catalog_name test-database \
  --catalog_path . \
  --default-nullable false

echo "Test database created in ./test-database/"
echo "Records generated: $TEST_SAMPLES"
echo "Seed used: $TEST_SEED"
echo "Start time: $TEST_START"
```

### Multi-Environment Database Generation

```bash
#!/bin/bash
# generate-env-databases.sh

SCRIPT="simulation.ion"
BASE_SEED=2024

# Development environment
beamline gen db beamline-lite \
  --seed $BASE_SEED \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path $SCRIPT \
  --sample-count 1000 \
  --catalog_name dev-db \
  --catalog_path ./environments/

# Staging environment  
beamline gen db beamline-lite \
  --seed $((BASE_SEED + 1)) \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path $SCRIPT \
  --sample-count 10000 \
  --catalog_name staging-db \
  --catalog_path ./environments/

# Production-like environment
beamline gen db beamline-lite \
  --seed $((BASE_SEED + 2)) \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path $SCRIPT \
  --sample-count 100000 \
  --catalog_name prod-like-db \
  --catalog_path ./environments/
```

### Database Migration Testing

```bash
#!/bin/bash
# test-schema-migration.sh

OLD_SCRIPT="data_v1.ion"
NEW_SCRIPT="data_v2.ion"

# Generate database with old schema
beamline gen db beamline-lite \
  --seed 100 \
  --start-auto \
  --script-path $OLD_SCRIPT \
  --catalog_name old-schema \
  --sample-count 1000

# Generate database with new schema
beamline gen db beamline-lite \
  --seed 100 \
  --start-auto \
  --script-path $NEW_SCRIPT \
  --catalog_name new-schema \
  --sample-count 1000

# Compare schemas
diff old-schema/*.shape.sql new-schema/*.shape.sql
```

## Performance Considerations

Database creation involves:
1. **Script parsing** (milliseconds)
2. **Data generation** (scales with `--sample-count`)
3. **Schema inference** (nearly instantaneous)
4. **File I/O** (depends on dataset size and disk speed)

### Performance Tips

```bash
# For large databases, monitor progress
time beamline gen db beamline-lite \
  --seed 1 \
  --start-auto \
  --script-path large_sim.ion \
  --sample-count 1000000

# Use faster storage for temporary operations
beamline gen db beamline-lite \
  --seed 1 \
  --start-auto \
  --script-path data.ion \
  --catalog_path /tmp/fast-storage/
```

## Best Practices

### 1. Use Meaningful Catalog Names

```bash
# Good - descriptive names
beamline gen db beamline-lite \
  --script-path user_analytics.ion \
  --catalog_name user-analytics-2024 \
  --catalog_path ./databases/

# Avoid - generic names
beamline gen db beamline-lite \
  --script-path data.ion \
  --catalog_name db
```

### 2. Document Generation Parameters

```bash
# Create documentation alongside database
beamline gen db beamline-lite \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path simulation.ion \
  --sample-count 50000 \
  --catalog_name analytics-db-v1

# Document the generation
echo "Analytics Database v1
Generated: $(date)
Seed: 12345
Start: 2024-01-01T00:00:00Z
Sample Count: 50000
Script: simulation.ion" > analytics-db-v1/README.txt
```

### 3. Use Version Control for Catalog Manifests

Track database generation metadata:

```bash
# Add manifest files to version control
git add beamline-catalog/.beamline-manifest
git add beamline-catalog/.beamline-script
git commit -m "Add database generation manifest for test-db v2.1"
```

### 4. Backup Before --force Operations

```bash
# The CLI creates backups automatically with --force, but verify
ls -la beamline-catalog*.bkp

# Manual backup before --force if desired
cp -r beamline-catalog manual-backup-$(date +%Y%m%d)
beamline gen db beamline-lite --script-path updated.ion --force
```

## Use Cases

### Local Development Database

```bash
# Create local database for development
beamline gen db beamline-lite \
  --seed 1000 \
  --start-auto \
  --script-path dev_data.ion \
  --sample-count 5000 \
  --catalog_name dev-local
```

### Test Suite Database

```bash
# Create comprehensive test database
beamline gen db beamline-lite \
  --seed 2024001 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path comprehensive_test.ion \
  --sample-count 50000 \
  --catalog_name integration-test-db \
  --default-nullable false \
  --default-optional false
```

### Performance Benchmark Database

```bash
# Create large database for performance testing
beamline gen db beamline-lite \
  --seed 999999 \
  --start-auto \
  --script-path performance_test.ion \
  --sample-count 1000000 \
  --catalog_name perf-benchmark \
  --catalog_path ./benchmarks/
```

## Database Analysis

### Examine Database Contents

```bash
# Check database size
du -sh beamline-catalog/

# Count records per dataset
for f in beamline-catalog/*.ion; do
  if [[ "$f" != *".shape.ion"* ]]; then
    echo "$(basename "$f" .ion): $(wc -l < "$f") records"
  fi
done

# View sample data
head -3 beamline-catalog/service.ion

# View schema
cat beamline-catalog/service.shape.sql
```

### Validate Database Integrity

```bash
# Verify manifest matches generation
cat beamline-catalog/.beamline-manifest

# Verify script is preserved
diff original_script.ion beamline-catalog/.beamline-script

# Check all datasets have corresponding schemas
for data in beamline-catalog/*.ion; do
  if [[ "$data" != *".shape.ion"* ]]; then
    dataset=$(basename "$data" .ion)
    if [[ ! -f "beamline-catalog/${dataset}.shape.ion" ]]; then
      echo "Missing schema for $dataset"
    fi
  fi
done
```

## Error Handling

### Common Errors

#### Catalog Directory Exists
```bash
$ beamline gen db beamline-lite --seed-auto --start-auto --script-path data.ion
creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17)

# Solution: Use --force or different catalog name
beamline gen db beamline-lite --seed-auto --start-auto --script-path data.ion --force
```

#### Script Parse Errors
```bash
$ beamline gen db beamline-lite --seed-auto --start-auto --script-path invalid.ion
Error: Failed to parse Ion script: Invalid Ion syntax at line 8
```

#### Insufficient Disk Space
```bash
# Check available space before large database creation
df -h .
beamline gen db beamline-lite --script-path huge_data.ion --sample-count 10000000
```

## Best Practices

### 1. Plan Storage Requirements

```bash
# Estimate database size with small sample first
beamline gen db beamline-lite \
  --seed 1 \
  --start-auto \
  --script-path data.ion \
  --sample-count 100 \
  --catalog_name size-test

# Check size and extrapolate
du -sh size-test/
# If 100 samples = 1MB, then 100,000 samples ≈ 1GB
```

### 2. Use Consistent Naming Conventions

```bash
# Good naming convention
beamline gen db beamline-lite \
  --script-path ecommerce_v2.ion \
  --catalog_name ecommerce-v2-20241201 \
  --catalog_path ./databases/

# Include date, version, purpose in catalog name
```

### 3. Document Database Generation

```bash
# Create database with documentation
beamline gen db beamline-lite \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path analytics.ion \
  --sample-count 25000 \
  --catalog_name analytics-q4-2024

# Add README
echo "Analytics Database Q4 2024
Purpose: Customer behavior analysis
Generated: $(date)
Script: analytics.ion  
Seed: 12345
Records: 25000
Contact: analytics-team@company.com" > analytics-q4-2024/README.txt
```

### 4. Validate Generated Databases

```bash
# Verify database creation was successful
ls -la beamline-catalog/
cat beamline-catalog/.beamline-manifest
wc -l beamline-catalog/*.ion
```

## Next Steps

Now that you understand all CLI commands:

- [CLI Overview](./overview.md) - Review complete CLI capabilities
- [Data Generation Guide](../data-generation/overview.md) - Learn about Ion scripts and generators
- [Database Guide](../database/overview.md) - Learn about database concepts and usage
- [Examples](../examples/sensors.md) - See complete workflows in action
