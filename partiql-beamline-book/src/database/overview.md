# Database Overview

PartiQL Beamline provides powerful database generation capabilities through the **BeamlineLite** system. Unlike simple data generation, database generation creates complete, self-contained databases with both data and schemas, making them ideal for testing, development, and demonstrations.

## What is BeamlineLite?

**BeamlineLite** is PartiQL Beamline's database generation system that creates filesystem-based databases containing:

- **Generated data** in Ion format  
- **Inferred schemas** in both Ion and SQL DDL formats
- **Metadata** about generation parameters
- **Original scripts** for reproducibility
- **Complete catalogs** ready for use with database systems

## Database vs Data Generation

### Data Generation (gen data)

```bash
partiql-beamline-cli gen data \
  --seed 42 \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 1000 \
  --output-format ion-pretty
```

**Output:** Stream of data records to stdout or file

**Use cases:** Data processing pipelines, API testing, analysis

### Database Generation (gen db)

```bash
partiql-beamline-cli gen db beamline-lite \
  --seed 42 \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 1000
```

**Output:** Complete database directory with data + schemas

**Use cases:** Local development databases, testing environments, demos

## BeamlineLite Database Structure

### Catalog Directory Layout

A BeamlineLite database creates a **catalog directory** with this structure:

```
beamline-catalog/
├── .beamline-manifest          # Generation metadata (JSON)
├── .beamline-script           # Original Ion script
├── <dataset>.ion              # Data files (one per dataset)
├── <dataset>.shape.ion        # Ion format schemas
└── <dataset>.shape.sql        # SQL DDL schemas
```

### Real Example from client-service.ion

```bash
$ partiql-beamline-cli gen db beamline-lite \
    --seed-auto \
    --start-auto \
    --script-path client-service.ion \
    --sample-count 1000

writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!

$ tree beamline-catalog/
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

## Database Files Deep Dive

### Manifest File (.beamline-manifest)

Contains generation metadata in JSON format:

```bash
$ cat beamline-catalog/.beamline-manifest
{"seed": "949665520117506306", "start": "2023-02-06T12:52:29.000000000Z", "ddl_syntax.version": "partiql_datatype_syntax.0.1"}
```

**Contents:**
- **seed**: Random seed used for generation (for reproducibility)
- **start**: Simulation start timestamp
- **ddl_syntax.version**: SQL DDL syntax version used in .shape.sql files

### Script File (.beamline-script)

Preserved copy of the original Ion script:

```bash
$ cat beamline-catalog/.beamline-script
rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },
    
    // A generator for client ids
    $id_gen: UUID,
    
    // ... rest of original script
}
```

**Purpose:**
- **Reproducibility**: Regenerate identical database later
- **Documentation**: What script created this database
- **Version control**: Track script changes over time

### Data Files (dataset.ion)

Contains generated data in compact Ion format:

```bash
$ cat beamline-catalog/client_0.ion
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "0de35d1e-a87c-e540-734d-6f2a4fa410c3", request_time: 2021-01-05T03:55:01.035000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "3539cdf0-6f7e-6bdc-c25a-4e0b7d8f8bac", request_time: 2021-01-05T03:55:01.182000000+00:00}
```

**Characteristics:**
- **One record per line**: Newline-delimited Ion format
- **Complete type information**: All Ion types preserved
- **Temporal ordering**: Records ordered by generation time

### Schema Files (dataset.shape.ion)

Ion format schema definitions:

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

**Use cases:**
- **PartiQL validation**: Validate queries against schema
- **Type checking**: Ensure data types match expectations
- **Tool integration**: Ion-aware tools can use schema information

### Schema Files (dataset.shape.sql)

SQL DDL format schemas:

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

**Use cases:**
- **Database creation**: Create tables in SQL databases
- **Schema documentation**: Human-readable schema reference
- **Migration scripts**: Database schema evolution

## Database Generation Process

### Step-by-Step Process

1. **Script Parsing**: Validate and parse Ion script
2. **Simulation Setup**: Initialize random processes and generators
3. **Catalog Creation**: Create catalog directory structure
4. **Manifest Generation**: Write generation metadata
5. **Script Preservation**: Copy original script to catalog
6. **Schema Inference**: Analyze script to determine data types
7. **Schema Export**: Generate Ion and SQL schema files
8. **Data Generation**: Generate and write data files
9. **Completion**: Database ready for use

### Process Output

```bash
$ partiql-beamline-cli gen db beamline-lite \
    --seed 12345 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path ecommerce.ion \
    --sample-count 10000

writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!
```

## Database Configuration Options

### Catalog Configuration

```bash
--catalog_name <NAME>          # Catalog directory name (default: "beamline-catalog")
--catalog_path <PATH>          # Where to create catalog (default: ".")
--force                        # Overwrite existing catalog with backup
```

### Examples

```bash
# Default catalog location
partiql-beamline-cli gen db beamline-lite \
  --seed 1000 \
  --start-auto \
  --script-path data.ion

# Creates: ./beamline-catalog/

# Custom catalog name and location
partiql-beamline-cli gen db beamline-lite \
  --seed 2000 \
  --start-auto \
  --script-path analytics.ion \
  --catalog_name analytics-db-v2 \
  --catalog_path ./databases/

# Creates: ./databases/analytics-db-v2/
```

### Backup and Overwrite Behavior

The system protects existing catalogs by default:

```bash
$ partiql-beamline-cli gen db beamline-lite --seed-auto --start-auto --script-path data.ion
creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17)
```

Use `--force` for safe overwrite with automatic backup:

```bash
$ partiql-beamline-cli gen db beamline-lite \
    --seed-auto \
    --start-auto \
    --script-path updated_data.ion \
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

## Database Use Cases

### Local Development Database

Create a lightweight database for development:

```bash
partiql-beamline-cli gen db beamline-lite \
  --seed 1000 \
  --start-auto \
  --script-path dev_data.ion \
  --sample-count 5000 \
  --catalog_name dev-local-db
```

**Benefits:**
- **Fast setup**: Generated in seconds
- **Realistic data**: Follows statistical distributions  
- **Complete schemas**: Both Ion and SQL formats
- **Reproducible**: Same seed creates identical database

### Testing Database

Create comprehensive test databases:

```bash
partiql-beamline-cli gen db beamline-lite \
  --seed 2024001 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path integration_test.ion \
  --sample-count 25000 \
  --catalog_name integration-test-db \
  --default-nullable false \
  --default-optional false
```

**Benefits:**
- **Consistent**: Exact reproduction for regression testing
- **Complete**: Multiple datasets with relationships
- **Clean**: Controlled nullability for predictable testing
- **Versioned**: Manifest tracks generation parameters

### Demo and Training Database

Create databases for demonstrations and training:

```bash
partiql-beamline-cli gen db beamline-lite \
  --seed 12345 \
  --start-iso "2023-01-01T00:00:00Z" \
  --script-path demo_ecommerce.ion \
  --sample-count 50000 \
  --catalog_name demo-ecommerce-2024
```

**Benefits:**
- **Realistic**: Data follows real-world patterns
- **Self-contained**: Everything needed in one directory
- **Documented**: Schemas and metadata included
- **Portable**: Easy to share and deploy

## Working with Generated Databases

### Database Analysis

```bash
# Examine database structure
ls -la beamline-catalog/
tree beamline-catalog/

# Check generation metadata
cat beamline-catalog/.beamline-manifest

# Count records per dataset
for f in beamline-catalog/*.ion; do
  if [[ "$f" != *".shape.ion" ]]; then
    echo "$(basename "$f" .ion): $(wc -l < "$f") records"
  fi
done

# Examine schemas
ls beamline-catalog/*.shape.sql
cat beamline-catalog/users.shape.sql
```

### Data Inspection

```bash
# View sample data from each dataset
for f in beamline-catalog/*.ion; do
  if [[ "$f" != *".shape.ion" ]]; then
    echo "=== $(basename "$f" .ion) ==="
    head -3 "$f"
    echo ""
  fi
done

# Look for specific patterns
grep "error" beamline-catalog/events.ion | head -5
grep -c "premium" beamline-catalog/users.ion
```

### Schema Analysis

```bash
# Compare schemas across datasets
diff beamline-catalog/client_0.shape.sql beamline-catalog/client_1.shape.sql

# Count fields per dataset
for f in beamline-catalog/*.shape.sql; do
  echo "$(basename "$f" .shape.sql): $(wc -l < "$f") fields"
done
```

## Database Integration Patterns

### SQL Database Integration

```bash
#!/bin/bash
# Load BeamlineLite database into PostgreSQL

CATALOG="beamline-catalog"
DB_NAME="test_db"

# Create database
createdb $DB_NAME

# Create tables from schemas
for schema in $CATALOG/*.shape.sql; do
  table_name=$(basename "$schema" .shape.sql)
  echo "CREATE TABLE $table_name (" > temp_schema.sql
  cat "$schema" >> temp_schema.sql
  echo ");" >> temp_schema.sql
  psql -d $DB_NAME -f temp_schema.sql
done

# Load data (would need custom Ion-to-CSV converter)
# for data in $CATALOG/*.ion; do
#   table_name=$(basename "$data" .ion)  
#   ion-to-csv "$data" | psql -d $DB_NAME -c "COPY $table_name FROM STDIN CSV HEADER"
# done

echo "Database $DB_NAME created with schemas"
```

### Application Testing Integration

```bash
#!/bin/bash
# Setup test environment with BeamlineLite database

APP_ENV="testing"
DB_CATALOG="test-data-v3"

echo "Setting up test environment..."

# Generate fresh test database
partiql-beamline-cli gen db beamline-lite \
  --seed 202412 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path test_scenarios.ion \
  --sample-count 10000 \
  --catalog-name $DB_CATALOG \
  --force

# Set environment variables for application
export TEST_DATA_PATH="$(pwd)/$DB_CATALOG"
export TEST_DATA_SEED="202412"
export TEST_DATA_RECORDS="10000"

echo "Test database created: $DB_CATALOG"
echo "Records per dataset:"
for f in $DB_CATALOG/*.ion; do
  if [[ "$f" != *".shape.ion" ]]; then
    echo "  $(basename "$f" .ion): $(wc -l < "$f")"
  fi
done

# Run application tests
# ./run_tests.sh
```

### Multi-Environment Database Generation

```bash
#!/bin/bash
# Generate databases for different environments

BASE_SCRIPT="app_simulation.ion"
BASE_SEED=2024

# Development environment - small data
partiql-beamline-cli gen db beamline-lite \
  --seed $BASE_SEED \
  --start-iso "2024-01-01T08:00:00Z" \
  --script-path $BASE_SCRIPT \
  --sample-count 1000 \
  --catalog-name dev-db \
  --catalog-path ./environments/

# Staging environment - medium data
partiql-beamline-cli gen db beamline-lite \
  --seed $((BASE_SEED + 1)) \
  --start-iso "2024-01-01T08:00:00Z" \
  --script-path $BASE_SCRIPT \
  --sample-count 25000 \
  --catalog-name staging-db \
  --catalog-path ./environments/

# Production-like environment - large data
partiql-beamline-cli gen db beamline-lite \
  --seed $((BASE_SEED + 2)) \
  --start-iso "2024-01-01T08:00:00Z" \
  --script-path $BASE_SCRIPT \
  --sample-count 500000 \
  --catalog-name prod-like-db \
  --catalog-path ./environments/

echo "Generated databases:"
echo "- Development: $(wc -l environments/dev-db/*.ion | tail -1 | awk '{print $1}') total records"
echo "- Staging: $(wc -l environments/staging-db/*.ion | tail -1 | awk '{print $1}') total records"  
echo "- Prod-like: $(wc -l environments/prod-like-db/*.ion | tail -1 | awk '{print $1}') total records"
```

## Database Versioning and Management

### Database Versioning Strategy

```bash
# Use meaningful catalog names with versions
partiql-beamline-cli gen db beamline-lite \
  --seed 2024001 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path user_analytics_v2.ion \
  --sample-count 100000 \
  --catalog-name user-analytics-v2-20241201 \
  --catalog-path ./db-versions/

# Document database version
echo "User Analytics Database v2.0
Generated: $(date)
Script: user_analytics_v2.ion
Seed: 2024001  
Start: 2024-01-01T00:00:00Z
Records: 100,000
Purpose: User behavior analysis for Q4 2024
Team: Analytics Engineering
" > db-versions/user-analytics-v2-20241201/README.md
```

### Database Archiving

```bash
#!/bin/bash
# Archive old database versions

ARCHIVE_DIR="./archived-databases"
CURRENT_DB="beamline-catalog"

if [ -d "$CURRENT_DB" ]; then
  # Create archive with timestamp
  TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
  ARCHIVE_NAME="$(basename "$CURRENT_DB")_$TIMESTAMP"
  
  mkdir -p "$ARCHIVE_DIR"
  mv "$CURRENT_DB" "$ARCHIVE_DIR/$ARCHIVE_NAME"
  
  echo "Archived current database to: $ARCHIVE_DIR/$ARCHIVE_NAME"
else
  echo "No current database to archive"
fi
```

## Advanced Database Generation

### Large Database Generation

For large databases, consider performance and storage:

```bash
# Generate large database with monitoring
time partiql-beamline-cli gen db beamline-lite \
  --seed 999999 \
  --start-iso "2023-01-01T00:00:00Z" \
  --script-path large_simulation.ion \
  --sample-count 5000000 \
  --catalog-name large-dataset-5m

# Monitor disk usage during generation
watch -n 5 'du -sh large-dataset-5m/'
```

### Database Validation

```bash
#!/bin/bash
# Validate generated database

CATALOG="$1"

if [ ! -d "$CATALOG" ]; then
  echo "Catalog directory $CATALOG not found"
  exit 1
fi

echo "Validating database: $CATALOG"

# Check required files exist
for required in ".beamline-manifest" ".beamline-script"; do
  if [ ! -f "$CATALOG/$required" ]; then
    echo "ERROR: Missing required file $required"
    exit 1
  fi
done

# Check data files have corresponding schemas
for data_file in $CATALOG/*.ion; do
  if [[ "$data_file" != *".shape.ion" ]]; then
    dataset=$(basename "$data_file" .ion)
    
    if [ ! -f "$CATALOG/${dataset}.shape.ion" ]; then
      echo "ERROR: Missing Ion schema for dataset $dataset"
    fi
    
    if [ ! -f "$CATALOG/${dataset}.shape.sql" ]; then
      echo "ERROR: Missing SQL schema for dataset $dataset"  
    fi
    
    record_count=$(wc -l < "$data_file")
    echo "✓ Dataset $dataset: $record_count records"
  fi
done

echo "Database validation completed"
```

## Performance Considerations

### Database Size Planning

```bash
# Estimate database size with small sample
partiql-beamline-cli gen db beamline-lite \
  --seed 1 \
  --start-auto \
  --script-path production_sim.ion \
  --sample-count 1000 \
  --catalog-name size-estimate

# Check size
du -sh size-estimate/
echo "Estimated size for 1M records: $(($(du -sk size-estimate/ | cut -f1) * 1000))KB"

# Clean up
rm -rf size-estimate/
```

### Memory and Disk Optimization

```bash
# For very large databases
partiql-beamline-cli gen db beamline-lite \
  --seed 123456 \
  --start-auto \
  --script-path optimized_sim.ion \
  --sample-count 10000000 \
  --catalog-name huge-db \
  --catalog-path /fast-ssd-storage/  # Use fast storage
```

### Concurrent Database Generation

```bash
#!/bin/bash
# Generate multiple databases in parallel

SCRIPT="simulation.ion"
BASE_SEED=10000

# Generate multiple test databases concurrently
for i in {1..4}; do
  (
    partiql-beamline-cli gen db beamline-lite \
      --seed $((BASE_SEED + i)) \
      --start-auto \
      --script-path $SCRIPT \
      --sample-count 50000 \
      --catalog-name "parallel-db-$i" \
      --catalog-path ./parallel-dbs/ &
  )
done

wait  # Wait for all background jobs
echo "All parallel databases generated"
```

## Database Comparison and Testing

### Schema Evolution Testing

```bash
#!/bin/bash  
# Test schema changes between versions

OLD_SCRIPT="schema_v1.ion"
NEW_SCRIPT="schema_v2.ion"

# Generate databases with same seed for comparison
partiql-beamline-cli gen db beamline-lite \
  --seed 100 \
  --start-auto \
  --script-path $OLD_SCRIPT \
  --catalog-name schema-v1-test

partiql-beamline-cli gen db beamline-lite \
  --seed 100 \
  --start-auto \
  --script-path $NEW_SCRIPT \
  --catalog-name schema-v2-test

# Compare schemas
echo "Schema differences:"
for dataset in schema-v1-test/*.shape.sql; do
  table_name=$(basename "$dataset" .shape.sql)
  if [ -f "schema-v2-test/$table_name.shape.sql" ]; then
    echo "=== $table_name ==="
    diff "$dataset" "schema-v2-test/$table_name.shape.sql"
  else
    echo "=== $table_name ==="
    echo "Table removed in v2"
  fi
done

# Check for new tables in v2
for dataset in schema-v2-test/*.shape.sql; do
  table_name=$(basename "$dataset" .shape.sql)
  if [ ! -f "schema-v1-test/$table_name.shape.sql" ]; then
    echo "=== $table_name ==="
    echo "New table in v2"
  fi
done
```

### Data Quality Assessment

```bash
#!/bin/bash
# Assess generated database quality

CATALOG="$1"

echo "Database Quality Report: $CATALOG"
echo "Generated: $(date)"
echo "=================================="

# Basic statistics
echo "Dataset Summary:"
for data in $CATALOG/*.ion; do
  if [[ "$data" != *".shape.ion" ]]; then
    dataset=$(basename "$data" .ion)
    records=$(wc -l < "$data")
    size=$(du -h "$data" | cut -f1)
    echo "  $dataset: $records records ($size)"
  fi
done

echo ""
echo "Schema Summary:"
for schema in $CATALOG/*.shape.sql; do
  dataset=$(basename "$schema" .shape.sql)  
  fields=$(wc -l < "$schema")
  echo "  $dataset: $fields fields"
done

# Check for data quality issues
echo ""
echo "Data Quality Checks:"

# Check for obvious issues in data files
for data in $CATALOG/*.ion; do
  if [[ "$data" != *".shape.ion" ]]; then
    dataset=$(basename "$data" .ion)
    
    # Check for empty records
    empty_count=$(grep -c '^{}$' "$data" 2>/dev/null || echo "0")
    if [ "$empty_count" -gt 0 ]; then
      echo "  WARNING: $dataset has $empty_count empty records"
    fi
    
    # Check for null-only records
    null_only=$(grep -c '^{[^}]*null[^}]*}$' "$data" 2>/dev/null || echo "0") 
    if [ "$null_only" -gt "$(wc -l < "$data")" ] && [ "$null_only" -gt 100 ]; then
      echo "  INFO: $dataset has high null value rate"
    fi
  fi
done

echo "Quality assessment completed"
```

## Database Backup and Recovery

### Manual Backup Strategy

```bash
#!/bin/bash
# Manual database backup

SOURCE_CATALOG="$1"
BACKUP_DIR="./database-backups"

if [ ! -d "$SOURCE_CATALOG" ]; then
  echo "Source catalog $SOURCE_CATALOG not found"
  exit 1
fi

# Create backup with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_NAME="$(basename "$SOURCE_CATALOG")_backup_$TIMESTAMP"

mkdir -p "$BACKUP_DIR"
cp -r "$SOURCE_CATALOG" "$BACKUP_DIR/$BACKUP_NAME"

echo "Database backed up to: $BACKUP_DIR/$BACKUP_NAME"

# Optionally compress backup
tar -czf "$BACKUP_DIR/$BACKUP_NAME.tar.gz" -C "$BACKUP_DIR" "$BACKUP_NAME"
rm -rf "$BACKUP_DIR/$BACKUP_NAME"

echo "Compressed backup: $BACKUP_DIR/$BACKUP_NAME.tar.gz"
```

### Database Restoration

```bash
#!/bin/bash
# Restore database from backup

BACKUP_FILE="$1"
RESTORE_DIR="$2"

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Backup file $BACKUP_FILE not found"
  exit 1
fi

# Extract backup
tar -xzf "$BACKUP_FILE" -C "$RESTORE_DIR"

echo "Database restored to: $RESTORE_DIR"
ls -la "$RESTORE_DIR"
```

## Database Migration and Updates

### Schema Migration

```bash
#!/bin/bash
# Migrate to new schema version

OLD_CATALOG="production-db-v1"
NEW_SCRIPT="production_v2.ion"

# Backup current database
cp -r "$OLD_CATALOG" "$OLD_CATALOG.backup"

# Get generation parameters from old database
OLD_SEED=$(jq -r '.seed' "$OLD_CATALOG/.beamline-manifest")
OLD_START=$(jq -r '.start' "$OLD_CATALOG/.beamline-manifest")

# Generate new database with same parameters
partiql-beamline-cli gen db beamline-lite \
  --seed "$OLD_SEED" \
  --start-iso "$OLD_START" \
  --script-path "$NEW_SCRIPT" \
  --catalog-name production-db-v2

# Compare schemas
echo "Schema changes:"
diff -r "$OLD_CATALOG"/*.shape.sql production-db-v2/*.shape.sql

echo "Migration completed"
echo "Old database backed up to: $OLD_CATALOG.backup"
echo "New database created: production-db-v2"
```

## Best Practices

### 1. Use Meaningful Catalog Names

```bash
# Good - descriptive names with versions and dates
partiql-beamline-cli gen db beamline-lite \
  --script-path user_behavior.ion \
  --catalog-name user-behavior-v3-20241201

# Avoid - generic names
partiql-beamline-cli gen db beamline-lite \
  --script-path data.ion \
  --catalog-name db
```

### 2. Document Database Purpose

```bash
# Create database with documentation
partiql-beamline-cli gen db beamline-lite \
  --seed 54321 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path analytics.ion \
  --sample-count 75000 \
  --catalog-name analytics-q4-2024

# Add comprehensive README
cat > analytics-q4-2024/README.md << EOF
# Analytics Database Q4 2024

## Purpose
Customer behavior analysis for quarterly business review

## Generation Details
- **Script**: analytics.ion
- **Seed**: 54321  
- **Start Time**: 2024-01-01T00:00:00Z
- **Sample Count**: 75,000
- **Generated**: $(date)

## Datasets
- **customers**: Customer profiles and demographics
- **sessions**: User session data with engagement metrics
- **events**: Detailed user interaction events
- **conversions**: Purchase and conversion events

## Usage
This database supports queries for:
- Customer segmentation analysis
- User journey mapping
- Conversion funnel analysis
- Engagement pattern identification

## Contact
Analytics Engineering Team <analytics@company.com>
EOF
```

### 3. Plan Storage Requirements

```bash
# Estimate storage needs
partiql-beamline-cli gen db beamline-lite \
  --seed 1 \
  --start-auto \
  --script-path large_sim.ion \
  --sample-count 10000 \
  --catalog-name storage-test

# Check size and project
size_kb=$(du -sk storage-test | cut -f1)
echo "10K records = ${size_kb}KB"
echo "1M records ≈ $((size_kb * 100))KB = $((size_kb * 100 / 1024))MB"

# Clean up test
rm -rf storage-test/
```

### 4. Use Version Control for Metadata

```bash
# Track database generation metadata in git
git add beamline-catalog/.beamline-manifest
git add beamline-catalog/.beamline-script  
git add beamline-catalog/README.md
git commit -m "Add database generation manifest for analytics-v2.1

Generated 50K records for user analytics testing
Seed: 2024001, Start: 2024-01-01T00:00:00Z"
```

### 5. Validate Generated Databases

```bash
#!/bin/bash
# Standard database validation

CATALOG="$1"

echo "Validating BeamlineLite database: $CATALOG"

# Structural validation
if [ ! -d "$CATALOG" ]; then
