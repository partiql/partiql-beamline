# Managing Catalogs

BeamlineLite catalogs are filesystem-based directories that contain complete databases with data, schemas, and metadata. 
Understanding how to manage, organize, and work with catalogs is essential for effective database operations.

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
beamline gen db beamline-lite \
  --seed 42 \
  --start-auto \
  --script-path data.ion

# Creates: ./beamline-catalog/
```

### Custom Catalog Configuration

```bash
# Custom name and location
beamline gen db beamline-lite \
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
$ beamline gen db beamline-lite --seed 1 --start-auto --script-path data.ion
creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17)
```

The `--force` option creates automatic backups:

```bash
$ beamline gen db beamline-lite \
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