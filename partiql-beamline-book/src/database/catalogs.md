# Managing Catalogs

BeamlineLite catalogs are filesystem-based directories that contain complete databases with data, schemas, and metadata. Understanding how to manage, organize, and work with catalogs is essential for effective database operations.

## Catalog Structure Deep Dive

### Standard Catalog Layout

Every BeamlineLite catalog follows a consistent structure:

```
catalog-name/
├── .beamline-manifest          # JSON metadata file
├── .beamline-script           # Original Ion script
├── dataset_1.ion              # Dataset 1 data (Ion format)
├── dataset_1.shape.ion        # Dataset 1 schema (Ion format)  
├── dataset_1.shape.sql        # Dataset 1 schema (SQL DDL)
├── dataset_2.ion              # Dataset 2 data
├── dataset_2.shape.ion        # Dataset 2 schema (Ion)
├── dataset_2.shape.sql        # Dataset 2 schema (SQL)
└── ... (additional datasets)
```

### File Naming Conventions

**Data Files:** `<dataset_name>.ion`
- Contains generated records in newline-delimited Ion format
- One file per dataset defined in the Ion script
- Records ordered chronologically by generation time

**Ion Schema Files:** `<dataset_name>.shape.ion`  
- PartiQL type definitions in Ion format
- Used by Ion-aware tools for validation and processing
- Contains complete type constraint information

**SQL Schema Files:** `<dataset_name>.shape.sql`
- SQL DDL field definitions (not complete CREATE TABLE)
- Ready for integration with SQL databases
- Human-readable schema documentation

**Metadata Files:**
- `.beamline-manifest` - Generation parameters in JSON
- `.beamline-script` - Original Ion script for reproducibility

## Catalog Creation Options

### Basic Catalog Creation

```bash
# Default catalog in current directory
partiql-beamline-cli gen db beamline-lite \
  --seed 42 \
  --start-auto \
  --script-path data.ion

# Creates: ./beamline-catalog/
```

### Custom Catalog Configuration

```bash
# Custom name and location
partiql-beamline-cli gen db beamline-lite \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path ecommerce.ion \
  --sample-count 50000 \
  --catalog-name ecommerce-prod-simulation \
  --catalog-path ./production-databases/

# Creates: ./production-databases/ecommerce-prod-simulation/
```

### Catalog Naming Best Practices

```bash
# Good - descriptive, versioned names
--catalog-name user-analytics-v2-20241201
--catalog-name integration-test-db-sprint-45  
--catalog-name demo-ecommerce-q4-2024

# Avoid - generic names
--catalog-name db
--catalog-name test
--catalog-name data
```

## Catalog Lifecycle Management

### Safe Overwrite with Backup

BeamlineLite protects existing catalogs by default:

```bash
$ partiql-beamline-cli gen db beamline-lite --seed 1 --start-auto --script-path data.ion
creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17)
```

The `--force` option creates automatic backups:

```bash
$ partiql-beamline-cli gen db beamline-lite \
    --seed 1 \
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

**Backup naming pattern:**
`<catalog-name>.<ISO-8601-timestamp>.bkp`

### Manual Backup Management

```bash
#!/bin/bash
# Create manual backup before making changes

CATALOG="important-catalog"
BACKUP_DIR="./catalog-backups"

# Create timestamped backup
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_NAME="${CATALOG}_manual_backup_${TIMESTAMP}"

mkdir -p "$BACKUP_DIR"
cp -r "$CATALOG" "$BACKUP_DIR/$BACKUP_NAME"

echo "Manual backup created: $BACKUP_DIR/$BACKUP_NAME"

# Optional compression
tar -czf "$BACKUP_DIR/$BACKUP_NAME.tar.gz" -C "$BACKUP_DIR" "$BACKUP_NAME"
rm -rf "$BACKUP_DIR/$BACKUP_NAME"

echo "Compressed backup: $BACKUP_DIR/$BACKUP_NAME.tar.gz"
```

### Backup Cleanup

```bash
#!/bin/bash
# Clean up old automatic backups

BACKUP_PATTERN="*.bkp"
DAYS_TO_KEEP=7

echo "Cleaning up backups older than $DAYS_TO_KEEP days..."

find . -name "$BACKUP_PATTERN" -type d -mtime +$DAYS_TO_KEEP | while read -r backup; do
  echo "Removing old backup: $backup"
  rm -rf "$backup"
done

echo "Backup cleanup completed"
```

## Catalog Organization Strategies

### Project-Based Organization

```bash
# Organize catalogs by project
mkdir -p projects/{analytics,ecommerce,iot}/databases

# Analytics project databases
partiql-beamline-cli gen db beamline-lite \
  --script-path analytics_v1.ion \
  --catalog-name analytics-dev \
  --catalog-path ./projects/analytics/databases/

partiql-beamline-cli gen db beamline-lite \
  --script-path analytics_v1.ion \
  --catalog-name analytics-staging \
  --catalog-path ./projects/analytics/databases/

# E-commerce project databases  
partiql-beamline-cli gen db beamline-lite \
  --script-path ecommerce_sim.ion \
  --catalog-name ecommerce-integration-test \
  --catalog-path ./projects/ecommerce/databases/
```

### Environment-Based Organization

```bash
# Organize by deployment environment
mkdir -p environments/{dev,staging,prod-like}/databases

BASE_SCRIPT="application.ion"
BASE_SEED=2024

# Development environment
partiql-beamline-cli gen db beamline-lite \
  --seed $BASE_SEED \
  --start-auto \
  --script-path $BASE_SCRIPT \
  --sample-count 1000 \
  --catalog-name app-db \
  --catalog-path ./environments/dev/databases/

# Staging environment  
partiql-beamline-cli gen db beamline-lite \
  --seed $((BASE_SEED + 1)) \
  --start-auto \
  --script-path $BASE_SCRIPT \
  --sample-count 25000 \
  --catalog-name app-db \
  --catalog-path ./environments/staging/databases/

# Production-like environment
partiql-beamline-cli gen db beamline-lite \
  --seed $((BASE_SEED + 2)) \
  --start-auto \
  --script-path $BASE_SCRIPT \
  --sample-count 500000 \
  --catalog-name app-db \
  --catalog-path ./environments/prod-like/databases/
```

### Version-Based Organization

```bash
# Organize by database version
mkdir -p database-versions/{v1.0,v1.1,v2.0}

# Version 1.0 databases
partiql-beamline-cli gen db beamline-lite \
  --script-path schema_v1.ion \
  --catalog-name production-simulation \
  --catalog-path ./database-versions/v1.0/

# Version 2.0 databases (new schema)
partiql-beamline-cli gen db beamline-lite \
  --script-path schema_v2.ion \
  --catalog-name production-simulation \
  --catalog-path ./database-versions/v2.0/

# Compare versions
diff -r database-versions/v1.0/production-simulation/*.shape.sql \
       database-versions/v2.0/production-simulation/*.shape.sql
```

## Catalog Analysis and Inspection

### Catalog Inventory

```bash
#!/bin/bash
# Generate catalog inventory report

CATALOG_DIR="$1"

if [ ! -d "$CATALOG_DIR" ]; then
  echo "Catalog directory not found: $CATALOG_DIR"
  exit 1
fi

echo "BeamlineLite Catalog Report"
echo "=========================="
echo "Catalog: $CATALOG_DIR"
echo "Generated: $(date)"
echo ""

# Generation metadata
if [ -f "$CATALOG_DIR/.beamline-manifest" ]; then
  echo "Generation Metadata:"
  echo "  Seed: $(jq -r '.seed' "$CATALOG_DIR/.beamline-manifest")"
  echo "  Start: $(jq -r '.start' "$CATALOG_DIR/.beamline-manifest")"
  echo "  DDL Version: $(jq -r '."ddl_syntax.version"' "$CATALOG_DIR/.beamline-manifest")"
  echo ""
fi

# Dataset summary
echo "Datasets:"
total_records=0
for data_file in "$CATALOG_DIR"/*.ion; do
  if [[ "$data_file" != *".shape.ion" ]]; then
    dataset=$(basename "$data_file" .ion)
    records=$(wc -l < "$data_file")
    size=$(du -h "$data_file" | cut -f1)
    total_records=$((total_records + records))
    
    echo "  $dataset:"
    echo "    Records: $records"
    echo "    Size: $size" 
    
    # Check for schema files
    if [ -f "$CATALOG_DIR/$dataset.shape.ion" ]; then
      echo "    Ion Schema: ✓"
    else
      echo "    Ion Schema: ✗ MISSING"
    fi
    
    if [ -f "$CATALOG_DIR/$dataset.shape.sql" ]; then
      echo "    SQL Schema: ✓"
    else  
      echo "    SQL Schema: ✗ MISSING"
    fi
    echo ""
  fi
done

echo "Total Records: $total_records"
echo "Total Size: $(du -sh "$CATALOG_DIR" | cut -f1)"
```

### Catalog Health Check

```bash
#!/bin/bash
# Validate catalog integrity

CATALOG="$1"

echo "Catalog Health Check: $CATALOG"
echo "================================"

errors=0
warnings=0

# Check required files
for required in ".beamline-manifest" ".beamline-script"; do
  if [ ! -f "$CATALOG/$required" ]; then
    echo "ERROR: Missing required file $required"
    ((errors++))
  else
    echo "✓ Found $required"
  fi
done

# Check data/schema file pairs
for data_file in "$CATALOG"/*.ion; do
  if [[ "$data_file" != *".shape.ion" ]]; then
    dataset=$(basename "$data_file" .ion)
    
    # Check Ion schema
    if [ ! -f "$CATALOG/$dataset.shape.ion" ]; then
      echo "ERROR: Missing Ion schema for dataset $dataset"
      ((errors++))
    fi
    
    # Check SQL schema
    if [ ! -f "$CATALOG/$dataset.shape.sql" ]; then
      echo "ERROR: Missing SQL schema for dataset $dataset"
      ((errors++))
    fi
    
    # Check for empty data files
    if [ ! -s "$data_file" ]; then
      echo "WARNING: Empty data file $dataset.ion"
      ((warnings++))
    fi
  fi
done

# Check for orphaned schema files
for schema_file in "$CATALOG"/*.shape.ion; do
  dataset=$(basename "$schema_file" .shape.ion)
  if [ ! -f "$CATALOG/$dataset.ion" ]; then
    echo "WARNING: Orphaned schema file $schema_file (no corresponding data)"
    ((warnings++))
  fi
done

echo ""
echo "Health Check Results:"
echo "  Errors: $errors"
echo "  Warnings: $warnings"

if [ $errors -eq 0 ]; then
  echo "✓ Catalog is healthy"
  exit 0
else
  echo "✗ Catalog has issues requiring attention"
  exit 1
fi
```

## Catalog Manipulation Operations

### Catalog Merging

```bash
#!/bin/bash
# Merge multiple catalogs (for same schema)

OUTPUT_CATALOG="$1"
shift
INPUT_CATALOGS=("$@")

echo "Merging catalogs into: $OUTPUT_CATALOG"

mkdir -p "$OUTPUT_CATALOG"

# Use first catalog's metadata as base
cp "${INPUT_CATALOGS[0]}/.beamline-manifest" "$OUTPUT_CATALOG/"
cp "${INPUT_CATALOGS[0]}/.beamline-script" "$OUTPUT_CATALOG/"

# Copy schemas from first catalog
cp "${INPUT_CATALOGS[0]}"/*.shape.* "$OUTPUT_CATALOG/"

# Merge data files
for catalog in "${INPUT_CATALOGS[@]}"; do
  echo "Processing catalog: $catalog"
  
  for data_file in "$catalog"/*.ion; do
    if [[ "$data_file" != *".shape.ion" ]]; then
      dataset=$(basename "$data_file" .ion)
      cat "$data_file" >> "$OUTPUT_CATALOG/$dataset.ion"
    fi
  done
done

echo "Catalog merge completed"

# Report merged sizes
for f in "$OUTPUT_CATALOG"/*.ion; do
  if [[ "$f" != *".shape.ion" ]]; then
    echo "$(basename "$f" .ion): $(wc -l < "$f") total records"
  fi
done
```

### Catalog Splitting by Dataset

```bash
#!/bin/bash  
# Split catalog into individual dataset catalogs

SOURCE_CATALOG="$1"
OUTPUT_DIR="$2"

if [ ! -d "$SOURCE_CATALOG" ]; then
  echo "Source catalog not found: $SOURCE_CATALOG"
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

# Split each dataset into its own catalog
for data_file in "$SOURCE_CATALOG"/*.ion; do
  if [[ "$data_file" != *".shape.ion" ]]; then
    dataset=$(basename "$data_file" .ion)
    dataset_catalog="$OUTPUT_DIR/$dataset-catalog"
    
    echo "Creating catalog for dataset: $dataset"
    mkdir -p "$dataset_catalog"
    
    # Copy metadata files
    cp "$SOURCE_CATALOG/.beamline-manifest" "$dataset_catalog/"
    cp "$SOURCE_CATALOG/.beamline-script" "$dataset_catalog/"
    
    # Copy dataset-specific files
    cp "$data_file" "$dataset_catalog/"
    cp "$SOURCE_CATALOG/$dataset.shape.ion" "$dataset_catalog/"
    cp "$SOURCE_CATALOG/$dataset.shape.sql" "$dataset_catalog/"
    
    echo "  Created: $dataset_catalog"
  fi
done

echo "Catalog splitting completed"
```

### Catalog Sampling

```bash
#!/bin/bash
# Create smaller sample catalog from large catalog

SOURCE_CATALOG="$1"
SAMPLE_SIZE="$2"  
OUTPUT_CATALOG="$3"

echo "Creating sample catalog: $OUTPUT_CATALOG"
echo "Sample size: $SAMPLE_SIZE records per dataset"

mkdir -p "$OUTPUT_CATALOG"

# Copy metadata and schemas
cp "$SOURCE_CATALOG/.beamline-manifest" "$OUTPUT_CATALOG/"
cp "$SOURCE_CATALOG/.beamline-script" "$OUTPUT_CATALOG/"
cp "$SOURCE_CATALOG"/*.shape.* "$OUTPUT_CATALOG/"

# Sample data files
for data_file in "$SOURCE_CATALOG"/*.ion; do
  if [[ "$data_file" != *".shape.ion" ]]; then
    dataset=$(basename "$data_file" .ion)
    
    # Take first N records (preserves temporal ordering)
    head -n "$SAMPLE_SIZE" "$data_file" > "$OUTPUT_CATALOG/$dataset.ion"
    
    records=$(wc -l < "$OUTPUT_CATALOG/$dataset.ion")
    echo "  $dataset: $records records sampled"
  fi
done

echo "Sample catalog created: $OUTPUT_CATALOG"
```

## Multi-Catalog Workflows

### Catalog Comparison

```bash
#!/bin/bash
# Compare two catalogs for differences

CATALOG_A="$1"
CATALOG_B="$2"

echo "Comparing catalogs:"
echo "  A: $CATALOG_A"  
echo "  B: $CATALOG_B"
echo ""

# Compare metadata
echo "Metadata Comparison:"
echo "  Seed A: $(jq -r '.seed' "$CATALOG_A/.beamline-manifest")"
echo "  Seed B: $(jq -r '.seed' "$CATALOG_B/.beamline-manifest")"
echo "  Start A: $(jq -r '.start' "$CATALOG_A/.beamline-manifest")"
echo "  Start B: $(jq -r '.start' "$CATALOG_B/.beamline-manifest")"
echo ""

# Compare schemas
echo "Schema Comparison:"
for schema_a in "$CATALOG_A"/*.shape.sql; do
  dataset=$(basename "$schema_a" .shape.sql)
  schema_b="$CATALOG_B/$dataset.shape.sql"
  
  if [ -f "$schema_b" ]; then
    if diff -q "$schema_a" "$schema_b" > /dev/null; then
      echo "  $dataset: ✓ Schemas identical"
    else
      echo "  $dataset: ✗ Schemas differ"
      diff "$schema_a" "$schema_b"
    fi
  else
    echo "  $dataset: ✗ Missing in catalog B"
  fi
done

# Compare record counts
echo ""
echo "Record Count Comparison:"
for data_a in "$CATALOG_A"/*.ion; do
  if [[ "$data_a" != *".shape.ion" ]]; then
    dataset=$(basename "$data_a" .ion)
    data_b="$CATALOG_B/$dataset.ion"
    
    count_a=$(wc -l < "$data_a")
    if [ -f "$data_b" ]; then
      count_b=$(wc -l < "$data_b")
      echo "  $dataset: A=$count_a, B=$count_b"
    else
      echo "  $dataset: A=$count_a, B=missing"
    fi
  fi
done
```

### Catalog Synchronization

```bash
#!/bin/bash
# Synchronize catalog structure (copy schemas, preserve data)

SOURCE_CATALOG="$1"  # Has correct schemas
TARGET_CATALOG="$2"  # Has data to preserve

echo "Synchronizing catalog structure from $SOURCE_CATALOG to $TARGET_CATALOG"

# Update metadata files
cp "$SOURCE_CATALOG/.beamline-manifest" "$TARGET_CATALOG/"
cp "$SOURCE_CATALOG/.beamline-script" "$TARGET_CATALOG/"

# Update schema files
cp "$SOURCE_CATALOG"/*.shape.ion "$TARGET_CATALOG/"
cp "$SOURCE_CATALOG"/*.shape.sql "$TARGET_CATALOG/"

echo "Schema synchronization completed"
echo "Data files in $TARGET_CATALOG preserved"
```

## Catalog Integration Patterns

### CI/CD Integration

```bash
#!/bin/bash
# CI/CD pipeline catalog management

set -e

PROJECT="analytics-pipeline"
BRANCH="${CI_BRANCH:-main}"
BUILD_ID="${CI_BUILD_ID:-local}"

# Create build-specific catalog
CATALOG_NAME="$PROJECT-$BRANCH-$BUILD_ID"

echo "Creating CI/CD test database: $CATALOG_NAME"

# Generate test database
partiql-beamline-cli gen db beamline-lite \
  --seed "$BUILD_ID" \
  --start-auto \
  --script-path "ci/test-data.ion" \
  --sample-count 10000 \
  --catalog-name "$CATALOG_NAME" \
  --catalog-path "./ci-databases/"

# Validate generated catalog
./scripts/validate-catalog.sh "ci-databases/$CATALOG_NAME"

# Set environment for tests
export TEST_CATALOG_PATH="$(pwd)/ci-databases/$CATALOG_NAME"
export TEST_CATALOG_NAME="$CATALOG_NAME"

echo "CI/CD database ready: $CATALOG_NAME"

# Clean up after tests (if desired)
# trap "rm -rf ci-databases/$CATALOG_NAME" EXIT
```

### Development Workflow Integration

```bash
#!/bin/bash
# Development workflow with catalog management

FEATURE="user-analytics-v3"
DEV_CATALOG="dev-$FEATURE"

echo "Setting up development environment for: $FEATURE"

# Clean up any existing dev catalog
if [ -d "$DEV_CATALOG" ]; then
  echo "Archiving existing dev catalog..."
  mv "$DEV_CATALOG" "$DEV_CATALOG.$(date +%Y%m%d_%H%M%S).archive"
fi

# Generate fresh development database
partiql-beamline-cli gen db beamline-lite \
  --seed 1000 \
  --start-auto \
  --script-path "scripts/$FEATURE.ion" \
  --sample-count 5000 \
  --catalog-name "$DEV_CATALOG"

# Setup development environment
export DEV_DATA_PATH="$(pwd)/$DEV_CATALOG"
export DEV_SEED="1000"

echo "Development database ready: $DEV_CATALOG"
echo "Environment variables set:"
echo "  DEV_DATA_PATH=$DEV_DATA_PATH"
echo "  DEV_SEED=$DEV_SEED"

# Start development server with database
# npm run dev
```

## Catalog Monitoring and Maintenance

### Catalog Size Monitoring

```bash
#!/bin/bash
# Monitor catalog disk usage

CATALOGS_DIR="$1"

echo "Catalog Size Report - $(date)"
echo "============================="

find "$CATALOGS_DIR" -name "beamline-catalog*" -o -name "*-catalog" -o -name "*-db" | while read -r catalog; do
  if [ -d "$catalog" ] && [ -f "$catalog/.beamline-manifest" ]; then
    size=$(du -sh "$catalog" | cut -f1)
    records=0
    
    # Count total records
    for data in "$catalog"/*.ion; do
      if [[ "$data" != *".shape.ion" ]] && [ -f "$data" ]; then
        records=$((records + $(wc -l < "$data")))
      fi
    done
    
    echo "$(basename "$catalog"): $size ($records records)"
  fi
done

echo ""
echo "Total disk usage: $(du -sh "$CATALOGS_DIR" | cut -f1)"
```

### Catalog Cleanup

```bash
#!/bin/bash
# Clean up old catalogs and backups

CATALOGS_DIR="$1" 
DAYS_OLD="$2"

echo "Cleaning up catalogs older than $DAYS_OLD days in $CATALOGS_DIR"

# Find and list old catalogs
find "$CATALOGS_DIR" -name "*-catalog" -o -name "*-db" -o -name "*.bkp" | while read -r catalog; do
  if [ -d "$catalog" ] && [ "$(find "$catalog" -mtime +$DAYS_OLD -print -quit)" ]; then
    size=$(du -sh "$catalog" | cut -f1)
    echo "Would remove: $(basename "$catalog") ($size)"
  fi
done

echo ""
read -p "Proceed with cleanup? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
  find "$CATALOGS_DIR" -name "*-catalog" -o -name "*-db" -o -name "*.bkp" | while read -r catalog; do
    if [ -d "$catalog" ] && [ "$(find "$catalog" -mtime +$DAYS_OLD -print -quit)" ]; then
      echo "Removing: $(basename "$catalog")"
      rm -rf "$catalog"
    fi
  done
  echo "Cleanup completed"
else
  echo "Cleanup cancelled"
fi
```

## Advanced Catalog Operations

### Catalog Migration Between Storage Systems

```bash
#!/bin/bash
# Migrate catalog to different storage location

SOURCE_CATALOG="$1"
TARGET_LOCATION="$2"

echo "Migrating catalog from $SOURCE_CATALOG to $TARGET_LOCATION"

# Validate source catalog
if [ ! -d "$SOURCE_CATALOG" ] || [ ! -f "$SOURCE_CATALOG/.beamline-manifest" ]; then
  echo "Invalid source catalog: $SOURCE_CATALOG"
  exit 1
fi

# Create target directory
mkdir -p "$TARGET_LOCATION"
TARGET_CATALOG="$TARGET_LOCATION/$(basename "$SOURCE_CATALOG")"

# Copy catalog
echo "Copying catalog..."
cp -r "$SOURCE_CATALOG" "$TARGET_CATALOG"

# Verify migration
echo "Verifying migration..."
diff -r "$SOURCE_CATALOG" "$TARGET_CATALOG"

if [ $? -eq 0 ]; then
  echo "✓ Migration successful"
  echo "Source: $SOURCE_CATALOG"
  echo "Target: $TARGET_CATALOG"
else
  echo "✗ Migration verification failed"
  exit 1
fi
```

### Catalog Consolidation

```bash
#!/bin/bash
# Consolidate multiple related catalogs

OUTPUT_CATALOG="$1"
shift
INPUT_CATALOGS=("$@")

echo "Consolidating catalogs:"
for cat in "${INPUT_CATALOGS[@]}"; do
  echo "  - $cat"
done
echo "Into: $OUTPUT_CATALOG"

mkdir -p "$OUTPUT_CATALOG"

# Use first catalog as base for metadata
BASE_CATALOG="${INPUT_CATALOGS[0]}"
cp "$BASE_CATALOG/.beamline-manifest" "$OUTPUT_CATALOG/"
cp "$BASE_CATALOG/.beamline-script" "$OUTPUT_CATALOG/"

# Create consolidated manifest
cat > "$OUTPUT_CATALOG/.beamline-manifest" << EOF  
{
  "consolidated": true,
  "created": "$(date -Iseconds)",
  "source_catalogs": [$(printf '"%s",' "${INPUT_CATALOGS[@]}" | sed 's/,$//')]
}
EOF

# Consolidate datasets
all_datasets=()
for catalog in "${INPUT_CATALOGS[@]}"; do
  for data_file in "$catalog"/*.ion; do
    if [[ "$data_file" != *".shape.ion" ]]; then
      dataset=$(basename "$data_file" .ion)
      
      # Add to dataset list if not already present
      if [[ ! " ${all_datasets[@]} " =~ " ${dataset} " ]]; then
        all_datasets+=("$dataset")
      fi
    fi
  done
done

# Merge each dataset
for dataset in "${all_datasets[@]}"; do
  echo "Consolidating dataset: $dataset"
  
  # Merge data files
  for catalog in "${INPUT_CATALOGS[@]}"; do
    if [ -f "$catalog/$dataset.ion" ]; then
      cat "$catalog/$dataset.ion" >> "$OUTPUT_CATALOG/$dataset.ion"
    fi
  done
  
  # Copy schema from first catalog that has this dataset
  for catalog in "${INPUT_CATALOGS[@]}"; do
    if [ -f "$catalog/$dataset.shape.ion" ]; then
      cp "$catalog/$dataset.shape.ion" "$OUTPUT_CATALOG/"
      cp "$catalog/$dataset.shape.sql" "$OUTPUT_CATALOG/"
      break
    fi
  done
done

echo "Consolidation completed"
```

## Catalog Security and Access Control

### Catalog Permissions

```bash
#!/bin/bash
# Set appropriate permissions for catalogs

CATALOG="$1"
MODE="$2"  # "dev", "prod", or "shared"

case $MODE in
  "dev")
    # Development - full access for developer
    chmod -R 755 "$CATALOG"
    echo "Set development permissions (755) for $CATALOG"
    ;;
  "prod")
    # Production - read-only for most users
    chmod -R 644 "$CATALOG"
    chmod 755 "$CATALOG"  # Directory needs execute permission
    echo "Set production permissions (644/755) for $CATALOG"
    ;;
  "shared")
    # Shared - group writable
    chmod -R 664 "$CATALOG"
    chmod -R g+s "$CATALOG"
    chmod 775 "$CATALOG"
    echo "Set shared permissions (664/775) for $CATALOG"
    ;;
  *)
    echo "Usage: $0 <catalog> <dev|prod|shared>"
    exit 1
    ;;
esac
```

### Catalog Integrity Protection

```bash
#!/bin/bash
# Protect catalog from accidental modification

CATALOG="$1"

if [ ! -d "$CATALOG" ]; then
  echo "Catalog not found: $CATALOG"
  exit 1
fi

# Make catalog read-only
chmod -R a-w "$CATALOG"

# Create protection marker
echo "This catalog has been protected from modification.
To unprotect: chmod -R u+w $CATALOG
Protected: $(date)
" > "$CATALOG/.PROTECTED"

chmod 444 "$CATALOG/.PROTECTED"

echo "Catalog protected: $CATALOG"
echo "To unprotect: chmod -R u+w $CATALOG"
```

## Troubleshooting Catalog Issues

### Common Catalog Problems

#### Corrupted Manifest File

```bash
# Symptoms: Can't read generation metadata
$ cat beamline-catalog/.beamline-manifest
cat: beamline-catalog/.beamline-manifest: No such file or directory

# Solution: Regenerate from script if available
if [ -f beamline-catalog/.beamline-script ]; then
  echo "Regenerating catalog from preserved script..."
  
  partiql-beamline-cli gen db beamline-lite \
    --seed 42 \
    --start-auto \
    --script-path beamline-catalog/.beamline-script \
    --catalog-name recovered-catalog
fi
```

#### Missing Schema Files

```bash
# Symptoms: Data files without corresponding schemas
$ ls beamline-catalog/
users.ion  orders.ion  # Missing .shape.ion and .shape.sql files

# Solution: Regenerate schemas from script
partiql-beamline-cli infer-shape \
  --seed 1 \
  --start-auto \
  --script-path beamline-catalog/.beamline-script \
  --output-format basic-ddl > recovered_schema.sql

partiql-beamline-cli infer-shape \
  --seed 1 \
  --start-auto \
  --script-path beamline-catalog/.beamline-script \
  --output-format beamline-json > recovered_schema.json
```

#### Large Catalog Performance
