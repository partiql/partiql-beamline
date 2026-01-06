# Shape Commands

The `beamline infer-shape` command analyzes Ion scripts to infer the data schemas without generating full datasets. This is useful for understanding data structures, creating database schemas, and validating script configurations.

## Command Syntax

```bash
beamline infer-shape [OPTIONS]
```

## Required Options

Shape inference uses the same core configuration as data generation:

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

### Output Format
```bash
--output-format <FORMAT>       # Shape output format (default: text)
```

Available formats:
- `text` - Human-readable debug format (default)
- `basic-ddl` - SQL DDL format for database schemas
- `beamline-json` - Beamline JSON format for testing

### Nullability and Optionality
```bash
--default-nullable <true|false>    # Set default nullability behavior
--pct-null <PERCENTAGE>            # Percentage of NULL values (0.0-1.0)
--default-optional <true|false>    # Set default optionality behavior  
--pct-optional <PERCENTAGE>        # Percentage of MISSING values (0.0-1.0)
```

## Output Formats

### Text Format (Default)

Provides detailed type information in Rust debug format:

```bash
$ beamline infer-shape --seed-auto --start-auto --script-path sensors.ion
Seed: 17685918364143248531
Start: 2022-12-12T19:52:29.000000000Z
{
    "sensors": PartiqlType(
        Bag(
            BagType {
                element_type: PartiqlType(
                    Struct(
                        StructType {
                            constraints: {
                                Fields(
                                    {
                                        StructField {
                                            name: "d",
                                            ty: PartiqlType(
                                                DecimalP(2, 0),
                                            ),
                                        },
                                        StructField {
                                            name: "f",
                                            ty: PartiqlType(
                                                Float64,
                                            ),
                                        },
                                        StructField {
                                            name: "i8",
                                            ty: PartiqlType(
                                                Int64,
                                            ),
                                        },
                                    },
                                ),
                            },
                        },
                    ),
                ),
            },
        ),
    ),
}
```

**Use Cases:**
- Development and debugging
- Understanding complex data structures
- Validating script configurations

### Basic DDL Format

Generates SQL DDL statements for database schema creation:

```bash
$ beamline infer-shape \
    --seed 7844265201457918498 \
    --start-auto \
    --script-path sensors-nested.ion \
    --output-format basic-ddl

-- Seed: 7844265201457918498
-- Start: 2024-01-01T06:53:06.000000000Z
-- Syntax: partiql_datatype_syntax.0.1
-- Dataset: sensors
"f" DOUBLE,
"i8" INT8,
"id" INT,
"sub" STRUCT<"f": DOUBLE,"o": INT8>,
"tick" INT8
```

**Use Cases:**
- Creating database tables
- Database schema documentation
- SQL migration scripts
- Data warehouse setup

### Beamline JSON Format

Structured JSON format used by PartiQL testing tools:

```bash
$ beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion \
    --output-format beamline-json

{
  seed: -3711181901898679775,
  start: 2022-05-22T13:49:57.000000000+00:00,
  shapes: {
    sensors: partiql::shape::v0::{
      type: "bag",
      items: {
        type: "struct",
        constraints: [
          ordered,
          closed
        ],
        fields: [
          {
            name: "d",
            type: "decimal(2, 0)"
          },
          {
            name: "f",
            type: "double"
          },
          {
            name: "i8",
            type: "int8"
          },
          {
            name: "tick",
            type: "int8"
          },
          {
            name: "w",
            type: "decimal(5, 4)"
          }
        ]
      }
    }
  }
}
```

**Use Cases:**
- PartiQL conformance testing
- Tool integration
- Automated schema validation

## Examples

### Basic Shape Inference

```bash
# Get basic shape information
beamline infer-shape \
  --seed-auto \
  --start-auto \
  --script-path my_data.ion

# Get reproducible shape with specific seed
beamline infer-shape \
  --seed 12345 \
  --start-auto \
  --script-path my_data.ion \
  --output-format text
```

### Database Schema Generation

```bash
# Generate SQL DDL for database creation
beamline infer-shape \
  --seed 100 \
  --start-auto \
  --script-path ecommerce.ion \
  --output-format basic-ddl > schema.sql

# Use in database creation
psql -d mydb -f schema.sql
```

### Multiple Dataset Schemas

```bash
# Infer shapes for complex multi-dataset scripts
beamline infer-shape \
  --seed 42 \
  --start-auto \
  --script-path client-service.ion \
  --output-format basic-ddl
```

This outputs schemas for all datasets defined in the script:
```sql
-- Dataset: service
"Account" VARCHAR,
"Operation" VARCHAR,
"Program" VARCHAR,
"Request" VARCHAR,
"StartTime" TIMESTAMP,
"client" VARCHAR,
"success" BOOL

-- Dataset: client_0
"id" VARCHAR,
"request_id" VARCHAR,
"request_time" TIMESTAMP,
"success" BOOL

-- Dataset: client_1
"id" VARCHAR,
"request_id" VARCHAR,
"request_time" TIMESTAMP,
"success" BOOL
```

### Schema with Nullability and Optionality

Configure NULL and MISSING value behavior in schema:

```bash
# Schema with all types nullable and optional
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path data.ion \
  --default-nullable true \
  --default-optional true \
  --output-format basic-ddl

# Output includes nullable/optional markers
"age" OPTIONAL TINYINT,
"name" OPTIONAL VARCHAR NULL,
"active" OPTIONAL BOOL
```

### Schema Validation Workflow

Use shape inference to validate scripts before large data generation:

```bash
# 1. Validate script syntax and structure
beamline infer-shape \
  --seed-auto \
  --start-auto \
  --script-path new_script.ion

# 2. Generate SQL schema
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --output-format basic-ddl > schema.sql

# 3. Generate small sample to verify
beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 5

# 4. Generate full dataset
beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 100000
```

## Integration Patterns

### Database Schema Creation

```bash
#!/bin/bash
# generate-database-schema.sh

SCRIPT="$1"
OUTPUT_DIR="./schemas"

if [ -z "$SCRIPT" ]; then
  echo "Usage: $0 <script.ion>"
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

# Generate DDL schema
echo "Generating database schema for $SCRIPT..."
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path "$SCRIPT" \
  --output-format basic-ddl > "$OUTPUT_DIR/$(basename "$SCRIPT" .ion).sql"

# Generate Beamline JSON for testing
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path "$SCRIPT" \
  --output-format beamline-json > "$OUTPUT_DIR/$(basename "$SCRIPT" .ion).json"

echo "Schemas generated in $OUTPUT_DIR/"
```

### CI/CD Schema Validation

```bash
#!/bin/bash
# validate-schemas.sh - CI pipeline script

set -e

echo "Validating Ion scripts..."

for script in scripts/*.ion; do
  echo "Checking $script..."
  
  # Validate script can generate valid schema
  if ! beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$script" \
    --output-format text > /dev/null; then
    echo "ERROR: Invalid script $script"
    exit 1
  fi
  
  echo "✓ $script is valid"
done

echo "All scripts validated successfully!"
```

### Documentation Generation

```bash
# Generate documentation for all data scripts
for script in data_scripts/*.ion; do
  name=$(basename "$script" .ion)
  
  echo "## $name Dataset" >> SCHEMAS.md
  echo '```sql' >> SCHEMAS.md
  
  beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$script" \
    --output-format basic-ddl >> SCHEMAS.md
    
  echo '```' >> SCHEMAS.md
  echo "" >> SCHEMAS.md
done
```

## Error Handling

### Common Errors

#### Script Syntax Errors
```bash
$ beamline infer-shape --seed-auto --start-auto --script-path invalid.ion
Error: Failed to parse Ion script: Invalid Ion syntax at line 5, column 10
```

#### Missing Required Options
```bash
$ beamline infer-shape --script-path data.ion
Error: One of --seed-auto or --seed is required
Error: One of --start-auto, --start-epoch-ms, or --start-iso is required
```

#### Invalid Output Format
```bash
$ beamline infer-shape --seed-auto --start-auto --script-path data.ion --output-format invalid
Error: 'invalid' isn't a valid value for '--output-format <OUTPUT_FORMAT>'
```

## Performance Considerations

Shape inference is very fast since it doesn't generate actual data:

- **Script Parsing**: Milliseconds for typical scripts
- **Type Inference**: Nearly instantaneous
- **Output Generation**: Minimal overhead

This makes shape inference ideal for:
- Quick script validation
- CI/CD pipeline checks
- Interactive development workflows
- Documentation generation

## Best Practices

### 1. Validate Scripts Early

```bash
# Always infer shape before generating large datasets
beamline infer-shape --seed 1 --start-auto --script-path new_script.ion
```

### 2. Use Appropriate Output Formats

```bash
# DDL for database work
beamline infer-shape --seed 1 --start-auto --script-path data.ion --output-format basic-ddl

# Text for debugging
beamline infer-shape --seed 1 --start-auto --script-path data.ion --output-format text

# JSON for automation
beamline infer-shape --seed 1 --start-auto --script-path data.ion --output-format beamline-json
```

### 3. Document Your Schemas

Save schema outputs for reference and version control:

```bash
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path production_data.ion \
  --output-format basic-ddl > docs/production_schema.sql
```

### 4. Use Consistent Seeds

For reproducible schema documentation:

```bash
# Always use seed 1 for schema documentation
beamline infer-shape --seed 1 --start-auto --script-path data.ion --output-format basic-ddl
```

## Next Steps

- [Database Commands](./database-commands.md) - Create complete databases with schemas
- [Schema Guide](../schema/understanding-shapes.md) - Learn about PartiQL type system
- [Data Generation](./data-commands.md) - Generate data matching your schemas
