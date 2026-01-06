# Schema Output Formats

Beamline supports three distinct output formats for schema information, each optimized for different use cases. Understanding these formats helps you choose the right one for your workflow and integrate schemas effectively with your tools and processes.

## Available Schema Formats

The `infer-shape` command supports three output formats via `--output-format`:

| Format | Description | Use Case | Performance |
|--------|-------------|----------|-------------|
| `text` | Rust debug format with detailed type information | Development, debugging | Fast |
| `basic-ddl` | SQL DDL statements ready for database creation | Database integration | Fast |
| `beamline-json` | Structured JSON for PartiQL testing tools | Tool integration, automation | Fast |

## Text Format (Default)

### Characteristics

- **Detailed type information**: Complete PartiQL type system representation
- **Rust debug format**: Shows internal type structures
- **Development focused**: Ideal for understanding complex type relationships
- **Human readable**: With some practice, easy to understand

### Example Output

From the README example with sensors.ion:

```bash
$ beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion

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
                                                DecimalP(
                                                    2,
                                                    0,
                                                ),
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
                                        StructField {
                                            name: "tick",
                                            ty: PartiqlType(
                                                Int64,
                                            ),
                                        },
                                        StructField {
                                            name: "w",
                                            ty: PartiqlType(
                                                DecimalP(
                                                    5,
                                                    4,
                                                ),
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

### Understanding Text Format Structure

- **`PartiqlType`**: Root type wrapper
- **`Bag`**: Collection type (dataset)
- **`BagType`**: Container for element type information
- **`Struct`**: Record structure  
- **`StructType`**: Detailed structure information
- **`StructField`**: Individual field with name and type
- **`DecimalP(5,4)`**: Decimal with precision 5, scale 4
- **`Float64`**: 64-bit floating point
- **`Int64`**: 64-bit integer

### Use Cases

- **Development**: Understanding complex type relationships
- **Debugging**: Detailed analysis of type inference
- **Learning**: Understanding PartiQL type system
- **Tool development**: Building PartiQL-aware tools

## Basic DDL Format

### Characteristics

- **SQL-ready**: Can be used directly in CREATE TABLE statements
- **Human readable**: Easy to understand for database developers
- **Production focused**: Ready for database integration
- **Compact**: Concise representation

### Example Output

From the README example with sensors-nested.ion:

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

### Format Structure

- **Header comments**: Generation metadata for reproducibility
- **Syntax version**: DDL syntax version identifier  
- **Dataset sections**: `-- Dataset: name` separates different datasets
- **Field definitions**: SQL column definitions ready for CREATE TABLE
- **Type specifications**: Precise SQL types with dimensions

### DDL Type Mapping

| PartiQL Internal | DDL Output | Description |
|------------------|------------|-------------|
| `Int64` | `INT8`, `TINYINT`, `INT`, `BIGINT` | Size depends on generator range |
| `Float64` | `DOUBLE` | 64-bit floating point |
| `DecimalP(p,s)` | `DECIMAL(p,s)` | Fixed precision decimal |
| `String` | `VARCHAR` | Variable length string |
| `Bool` | `BOOL` | Boolean type |
| `DateTime` | `TIMESTAMP` | Date and time |
| `Struct` | `STRUCT<...>` | Nested object structure |
| `Array` | `ARRAY<T>` | Array of type T |
| `Union` | `UNION<T1,T2,...>` | One of several types |

### Complete Database Creation

```bash
#!/bin/bash
# Create complete database from DDL output

SCRIPT="ecommerce.ion"
DB_NAME="ecommerce_test"

# Generate complete DDL
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl > schema.ddl

# Create database
createdb "$DB_NAME"

# Extract and create each table
grep "-- Dataset:" schema.ddl | while read -r line; do
    dataset=$(echo "$line" | cut -d: -f2 | xargs)
    
    {
        echo "CREATE TABLE $dataset ("
        # Extract fields until next dataset or end of file
        sed -n "/-- Dataset: $dataset/,/-- Dataset:/p" schema.ddl | \
        grep '^"' | \
        sed '$ s/,$//' # Remove trailing comma from last line
        echo ");"
    } > "${dataset}_table.sql"
    
    echo "Creating table: $dataset"
    psql -d "$DB_NAME" -f "${dataset}_table.sql"
    
    rm "${dataset}_table.sql"
done

echo "Database $DB_NAME created successfully"
```

### Use Cases

- **Database creation**: Direct CREATE TABLE generation
- **Schema documentation**: Human-readable reference
- **Migration scripts**: Database schema evolution
- **SQL integration**: Compatible with SQL databases

## Beamline JSON Format

### Characteristics

- **Machine readable**: Structured JSON for programmatic processing
- **Tool integration**: Designed for PartiQL testing frameworks
- **Versioned**: Includes format version information
- **Complete metadata**: Full type and constraint information

### Example Output

From the README example:

```bash
$ beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion \
    --output-format beamline-json

{
  seed: -3711181901898679775,
  start: "2022-05-22T13:49:57.000000000+00:00",
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

### JSON Format Structure

- **`seed`**: Random seed used for inference
- **`start`**: Start timestamp used for inference
- **`shapes`**: Dictionary of dataset name to shape definition
- **`partiql::shape::v0::`**: Format version identifier
- **`type: "bag"`**: Collection type (dataset)
- **`items`**: Element type definition for bag contents
- **`constraints`**: Structural constraints (`ordered`, `closed`)
- **`fields`**: Array of field definitions
- **Field objects**: `name` and `type` for each field

### Processing JSON Format

```bash
# Extract dataset names
beamline infer-shape --seed 1 --start-auto --script-path multi.ion --output-format beamline-json | \
jq -r '.shapes | keys[]'

# Count fields in each dataset
beamline infer-shape --seed 1 --start-auto --script-path multi.ion --output-format beamline-json | \
jq -r '.shapes | to_entries[] | "\(.key): \(.value.items.fields | length) fields"'

# Extract field types for specific dataset
beamline infer-shape --seed 1 --start-auto --script-path data.ion --output-format beamline-json | \
jq -r '.shapes.users.items.fields[] | "\(.name): \(.type)"'
```

### Use Cases

- **Automated testing**: PartiQL conformance test suites
- **Tool integration**: Schema-aware development tools
- **CI/CD pipelines**: Automated schema validation
- **Documentation generation**: Programmatic documentation creation

## Format Comparison

### Size and Performance

For the same schema with multiple datasets:

```bash
# Generate all formats for comparison
beamline infer-shape --seed 1 --start-auto --script-path complex.ion --output-format text > schema.txt
beamline infer-shape --seed 1 --start-auto --script-path complex.ion --output-format basic-ddl > schema.sql
beamline infer-shape --seed 1 --start-auto --script-path complex.ion --output-format beamline-json > schema.json

# Compare sizes
ls -lh schema.*
# Example results:
# -rw-r--r-- 1 user user 8.2K schema.txt  (most detailed)
# -rw-r--r-- 1 user user 1.5K schema.sql  (most compact)  
# -rw-r--r-- 1 user user 3.1K schema.json (structured)
```

### Information Density

| Format | Type Detail | Structure Info | Metadata | Processability |
|--------|-------------|----------------|----------|----------------|
| `text` | Very High | Very High | High | Low |
| `basic-ddl` | Medium | Medium | Medium | High (SQL) |
| `beamline-json` | High | High | High | High (JSON) |

## Format-Specific Integration

### Text Format Analysis

```bash
# Analyze complex type structures
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path nested_structures.ion \
    --output-format text | \
grep -A 20 "StructField"  # Extract field information

# Count nesting levels
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path deep_nesting.ion \
    --output-format text | \
grep -c "Struct("  # Count nested structures
```

### DDL Format Database Integration

```bash
#!/bin/bash
# Complete database integration workflow

SCRIPT="$1"
DATABASE="$2"

# Generate DDL schema
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl > full_schema.sql

# Create database
createdb "$DATABASE"

# Process each dataset into a table
current_dataset=""
while IFS= read -r line; do
    if [[ $line == *"-- Dataset:"* ]]; then
        # Start new dataset
        if [[ -n "$current_dataset" ]]; then
            echo ");" >> "${current_dataset}.sql"
            psql -d "$DATABASE" -f "${current_dataset}.sql"
            rm "${current_dataset}.sql"
        fi
        
        current_dataset=$(echo "$line" | cut -d: -f2 | xargs)
        echo "CREATE TABLE $current_dataset (" > "${current_dataset}.sql"
        
    elif [[ $line == \"* ]]; then
        # Add field to current table
        echo "  $line" >> "${current_dataset}.sql"
    fi
done < full_schema.sql

# Handle last dataset
if [[ -n "$current_dataset" ]]; then
    echo ");" >> "${current_dataset}.sql"
    psql -d "$DATABASE" -f "${current_dataset}.sql"
    rm "${current_dataset}.sql"
fi

echo "Database $DATABASE created with all tables"
```

### JSON Format Automation

```bash
#!/bin/bash
# Automated schema processing with JSON format

SCRIPT="$1"

echo "Analyzing schema from $SCRIPT..."

# Generate JSON schema
schema_json=$(beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format beamline-json)

# Extract metadata
seed=$(echo "$schema_json" | jq -r '.seed')
start=$(echo "$schema_json" | jq -r '.start')
echo "Schema generated with seed: $seed, start: $start"

# Analyze each dataset
echo "$schema_json" | jq -r '.shapes | keys[]' | while read -r dataset; do
    echo ""
    echo "Dataset: $dataset"
    
    # Get field count
    field_count=$(echo "$schema_json" | jq ".shapes.${dataset}.items.fields | length")
    echo "  Fields: $field_count"
    
    # List all fields with types
    echo "  Field details:"
    echo "$schema_json" | jq -r ".shapes.${dataset}.items.fields[] | \"    \(.name): \(.type)\""
    
    # Check for complex types
    complex_count=$(echo "$schema_json" | jq -r ".shapes.${dataset}.items.fields[] | select(.type | contains(\"STRUCT\") or contains(\"ARRAY\") or contains(\"UNION\")) | .name" | wc -l)
    if [ "$complex_count" -gt 0 ]; then
        echo "  Complex types: $complex_count fields"
    fi
done
```

## Multi-Dataset Schema Handling

### Separating Dataset Schemas

Different formats handle multiple datasets differently:

#### Text Format
All datasets in single output, nested under their names:

```
{
    "service": PartiqlType(Bag(...)),
    "client_0": PartiqlType(Bag(...)),
    "client_1": PartiqlType(Bag(...))
}
```

#### Basic DDL Format  
Datasets separated by comments:

```sql
-- Dataset: service
"Account" VARCHAR,
"Request" VARCHAR,
-- Dataset: client_0  
"id" VARCHAR,
"request_id" VARCHAR,
-- Dataset: client_1
"id" VARCHAR,
"request_id" VARCHAR,
```

#### JSON Format
Datasets as separate objects in `shapes` dictionary:

```json
{
  "shapes": {
    "service": { "type": "bag", "items": {...} },
    "client_0": { "type": "bag", "items": {...} },
    "client_1": { "type": "bag", "items": {...} }
  }
}
```

### Dataset-Specific Schema Extraction

```bash
#!/bin/bash
# Extract schema for specific dataset

SCRIPT="$1"
DATASET="$2"
FORMAT="$3"

case $FORMAT in
    "ddl")
        beamline infer-shape \
            --seed 1 \
            --start-auto \
            --script-path "$SCRIPT" \
            --output-format basic-ddl | \
        sed -n "/-- Dataset: $DATASET/,/-- Dataset:/p" | \
        head -n -1  # Remove next dataset header
        ;;
        
    "json")
        beamline infer-shape \
            --seed 1 \
            --start-auto \
            --script-path "$SCRIPT" \
            --output-format beamline-json | \
        jq ".shapes.${DATASET}"
        ;;
        
    *)
        echo "Usage: $0 <script.ion> <dataset_name> <ddl|json>"
        exit 1
        ;;
esac
```

## Nullability and Optionality in Formats

### How Each Format Represents Absent Values

#### Text Format
```
StructField {
    name: "nullable_field",
    ty: PartiqlType(Int64),  // Type doesn't show nullability directly
}
```

#### DDL Format
```sql
"required_field" VARCHAR NOT NULL,     -- nullable: false
"nullable_field" INT,                  -- nullable: true (default)
"optional_field" OPTIONAL VARCHAR      -- optional: true
```

#### JSON Format
```json
{
  "name": "nullable_field", 
  "type": "int64"  // Nullability not directly visible
}
```

**Note**: DDL format provides the clearest nullability information.

### CLI Defaults Impact on Formats

```bash
# With CLI nullability defaults
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path data.ion \
    --default-nullable true \
    --default-optional true \
    --output-format basic-ddl
```

**Result shows CLI impact:**
```sql
-- All fields affected by CLI defaults
"field1" OPTIONAL VARCHAR,      -- Made optional by CLI
"field2" OPTIONAL INT NOT NULL, -- Made optional, but explicit nullable: false overrides
"field3" OPTIONAL BOOL          -- Both CLI defaults applied
```

## Advanced Format Usage

### Schema Evolution Tracking

```bash
#!/bin/bash
# Track schema changes across versions

OLD_SCRIPT="model_v1.ion"
NEW_SCRIPT="model_v2.ion"

echo "Schema Evolution Analysis"
echo "========================"

# Generate schemas in DDL format for comparison
beamline infer-shape --seed 1 --start-auto --script-path "$OLD_SCRIPT" --output-format basic-ddl > v1_schema.sql
beamline infer-shape --seed 1 --start-auto --script-path "$NEW_SCRIPT" --output-format basic-ddl > v2_schema.sql

# Show changes
echo "Schema changes:"
diff -u v1_schema.sql v2_schema.sql

# Also generate JSON for programmatic analysis
beamline infer-shape --seed 1 --start-auto --script-path "$OLD_SCRIPT" --output-format beamline-json > v1_schema.json
beamline infer-shape --seed 1 --start-auto --script-path "$NEW_SCRIPT" --output-format beamline-json > v2_schema.json

# Count field changes
v1_fields=$(jq -r '.shapes | to_entries[] | .value.items.fields[].name' v1_schema.json | sort)
v2_fields=$(jq -r '.shapes | to_entries[] | .value.items.fields[].name' v2_schema.json | sort)

added_fields=$(comm -13 <(echo "$v1_fields") <(echo "$v2_fields"))
removed_fields=$(comm -23 <(echo "$v1_fields") <(echo "$v2_fields"))

echo ""
echo "Field changes:"
if [ -n "$added_fields" ]; then
    echo "Added: $(echo "$added_fields" | tr '\n' ' ')"
fi
if [ -n "$removed_fields" ]; then
    echo "Removed: $(echo "$removed_fields" | tr '\n' ' ')"
fi
```

### Multi-Format Documentation

```bash
#!/bin/bash
# Generate comprehensive schema documentation

SCRIPT="$1"
BASE_NAME=$(basename "$SCRIPT" .ion)

echo "# Schema Documentation: $BASE_NAME"
echo "Generated: $(date)"
echo ""

# Generate metadata
metadata=$(beamline infer-shape --seed 1 --start-auto --script-path "$SCRIPT" --output-format basic-ddl | head -3)
echo "## Generation Metadata"
echo '```'
echo "$metadata"
echo '```'
echo ""

# SQL DDL for database developers
echo "## SQL DDL Schema"
echo '```sql'
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl | tail -n +4  # Skip metadata lines
echo '```'
echo ""

# JSON for tool developers
echo "## JSON Schema (for tools)"
echo '```json'
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format beamline-json | jq '.shapes'
echo '```'
echo ""

# Analysis summary
echo "## Schema Analysis"
schema_json=$(beamline infer-shape --seed 1 --start-auto --script-path "$SCRIPT" --output-format beamline-json)
dataset_count=$(echo "$schema_json" | jq '.shapes | length')
total_fields=$(echo "$schema_json" | jq '[.shapes[].items.fields | length] | add')

echo "- **Datasets**: $dataset_count"
echo "- **Total fields**: $total_fields"
echo "- **Source script**: \`$SCRIPT\`"
```

## Best Practices

### 1. Choose Format for Purpose

```bash
# Understanding complex types
beamline infer-shape --script-path complex.ion --output-format text

# Database creation
beamline infer-shape --script-path db_model.ion --output-format basic-ddl

# Automated processing
beamline infer-shape --script-path data.ion --output-format beamline-json
```

### 2. Use Consistent Parameters

```bash
# Always use same seed for reproducible schema generation
beamline infer-shape --seed 1 --start-auto --script-path script.ion --output-format basic-ddl
```

### 3. Version Schema Output

```bash
# Include format in version control
git add schemas/model_v2.sql schemas/model_v2.json
git commit -m "Add schema v2 in SQL and JSON formats

- SQL DDL for database creation
- JSON for automated tooling"
```

### 4. Validate Format Consistency

```bash
# Ensure all formats represent same schema
beamline infer-shape --seed 42 --start-auto --script-path test.ion --output-format basic-ddl > test.sql
beamline infer-shape --seed 42 --start-auto --script-path test.ion --output-format beamline-json > test.json

# Extract field count from both formats  
sql_fields=$(grep '^"' test.sql | wc -l)
json_fields=$(jq '[.shapes[].items.fields | length] | add' test.json)

if [ "$sql_fields" -eq "$json_fields" ]; then
    echo "✅ Schema formats consistent: $sql_fields fields"
else
    echo "❌ Format mismatch: SQL=$sql_fields, JSON=$json_fields"
fi
```

## Format Selection Guidelines

### By Use Case

| Use Case | Recommended Format | Alternative |
|----------|-------------------|-------------|
| **Database creation** | `basic-ddl` | N/A |
| **Development debugging** | `text` | `basic-ddl` |
| **Tool integration** | `beamline-json` | N/A |
| **Documentation** | `basic-ddl` | `beamline-json` |
| **Schema comparison** | `basic-ddl` | `beamline-json` |
| **CI/CD automation** | `beamline-json` | `basic-ddl` |

### By Consumer

| Consumer | Recommended Format | Rationale |
|----------|-------------------|-----------|
| **SQL Database** | `basic-ddl` | Direct CREATE TABLE usage |
| **PartiQL Tools** | `beamline-json` | Native PartiQL format |
| **Human Review** | `basic-ddl` | Most readable |
| **Development Tools** | `beamline-json` | Machine processable |
| **Documentation** | `basic-ddl` | Clear and concise |

## Next Steps

Now that you understand all schema output formats:

- [CLI Shape Commands](../cli/shape-commands.md) - Complete CLI reference for shape operations
- [Database Integration](../database/overview.md) - Using schemas for database creation
- [Understanding Shapes](./understanding-shapes.md) - Fundamental concepts and type mappings
