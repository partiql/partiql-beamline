# Understanding Shapes

In Beamline, **shapes** (also called schemas) describe the structure and types of your generated data. Shape inference analyzes Ion scripts to determine what types of data will be generated, without actually generating the full dataset. This is essential for database schema creation, query validation, and understanding your data structure.

## What are Shapes?

Shapes are PartiQL's way of describing data structure and type information:

- **Type information**: What types each field can contain (INT, VARCHAR, BOOL, etc.)
- **Structure information**: How data is organized (bags, structs, arrays)
- **Constraints**: Whether fields are nullable, optional, or have other constraints
- **Nested relationships**: How complex data structures are organized

## Shape Inference Process

### How Shape Inference Works

1. **Script Analysis**: Parse the Ion script to understand generators
2. **Type Resolution**: Determine PartiQL types for each generator
3. **Structure Mapping**: Build hierarchical type structure
4. **Constraint Analysis**: Determine nullability and optionality
5. **Format Output**: Generate shapes in requested format

### Running Shape Inference

From the README examples, shape inference is done using:

```bash
beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion
```

The seed and start time are needed even though no data is generated, as they may affect type inference for certain generators.

## Shape Output Formats

### Text Format (Default)

Provides detailed type information in Rust debug format:

```bash
beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion
```

**Example Output:**
```
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

**Use Cases:**
- Development and debugging
- Understanding complex nested structures
- Detailed type analysis

### Basic DDL Format

Generates SQL DDL statements ready for database creation:

```bash
beamline infer-shape \
    --seed 7844265201457918498 \
    --start-auto \
    --script-path sensors-nested.ion \
    --output-format basic-ddl
```

**Example OutputE:**
```sql
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
- Schema documentation
- Database migration scripts

### Beamline JSON Format

Structured JSON format used by PartiQL testing tools:

```bash
beamline infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion \
    --output-format beamline-json
```

**Example Output:**
```json
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

**Use Cases:**
- PartiQL conformance testing
- Tool integration
- Automated testing pipelines

## PartiQL Type System

### Basic Types

From the examples and implementation:

| PartiQL Type | Description | Ion Script Generator |
|--------------|-------------|---------------------|
| `INT8` | 8-bit signed integer | `UniformI8` |
| `INT64` | 64-bit signed integer | `UniformI64`, `Tick` |
| `DOUBLE` | 64-bit floating point | `UniformF64`, `NormalF64` |
| `DECIMAL(p,s)` | Fixed-precision decimal | `UniformDecimal` |
| `VARCHAR` | Variable-length string | `UUID`, `LoremIpsumTitle`, `Regex` |
| `BOOL` | Boolean value | `Bool` |
| `TIMESTAMP` | Date and time | `Instant`, `Date` |

### Complex Types

| PartiQL Type | Description | Ion Script Generator |
|--------------|-------------|---------------------|
| `STRUCT<...>` | Object with named fields | Nested `$data` objects |
| `ARRAY<T>` | Array of type T | `UniformArray` |
| `UNION<T1,T2>` | Value can be one of multiple types | `UniformAnyOf` |

## Real Shape Examples

### Simple Sensor Shape

From the `sensors.ion` script:

```ion
rand_processes::{
    $n: UniformU8::{ low: 2, high: 10 },
    sensors: $n::[
        rand_process::{
            $data: {
                tick: Tick,
                i8: UniformI8,
                f: UniformF64,
                d: UniformDecimal::{ low: 0d0, high: 4.2d1, nullable: false }
            }
        }
    ]
}
```

**Inferred Shape (DDL):**
```sql
-- Dataset: sensors
"f" DOUBLE,
"i8" INT8,
"tick" INT8,
"d" DECIMAL(2, 0) NOT NULL
```

### Complex Nested Shape

From the `sensors-nested.ion` script:

```ion
rand_processes::{
    sensors: rand_process::{
        $data: {
            tick: Tick,
            i8: UniformI8,
            f: UniformF64,
            sub: {
                o: UniformI8,
                f: UniformF64
            }
        }
    }
}
```

**Inferred Shape (DDL):**
```sql
-- Dataset: sensors  
"f" DOUBLE,
"i8" INT8,
"sub" STRUCT<"f": DOUBLE,"o": INT8>,
"tick" INT8
```

### Multi-Dataset Shape

From the `client-service.ion` script with multiple datasets:

```bash
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path client-service.ion \
    --output-format basic-ddl
```

**Generated Output:**
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

Notice how each dataset gets its own schema section.

## Nullability in Shapes

### Nullable vs Non-Nullable Fields

Shape inference detects nullability configuration from scripts:

```ion
rand_processes::{
    test_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            required_field: UUID::{ nullable: false },
            nullable_field: UniformI32::{ nullable: 0.2, low: 1, high: 100 },
            optional_field: UniformDecimal::{ optional: 0.1, low: 0.0, high: 100.0 }
        }
    }
}
```

**Inferred Shape:**
```sql
-- Dataset: test_data
"required_field" VARCHAR NOT NULL,        -- nullable: false
"nullable_field" INT,                     -- nullable: 0.2 (can be NULL)
"optional_field" OPTIONAL DECIMAL(3, 1)   -- optional: 0.1 (can be MISSING)
```

### CLI Nullability Defaults

Global CLI defaults affect inferred shapes:

```bash
# With default nullability
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path simple_data.ion \
    --default-nullable true \
    --default-optional true \
    --output-format basic-ddl
```

**Result:**
```sql
-- All fields become nullable and optional by default
"field1" OPTIONAL INT,
"field2" OPTIONAL VARCHAR,
"field3" OPTIONAL BOOL
```

## Shape Inference Workflow

### Development Workflow

```bash
#!/bin/bash
# Shape-driven development workflow

SCRIPT="new_data_model.ion"

echo "1. Creating initial Ion script..."
cat > $SCRIPT << 'EOF'
rand_processes::{
    user_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            user_id: UUID,
            age: UniformU8::{ low: 18, high: 80 },
            email: Format::{ pattern: "user{UUID}@example.com" },
            active: Bool::{ p: 0.8 }
        }
    }
}
EOF

echo "2. Inferring shape..."
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path $SCRIPT \
    --output-format basic-ddl > schema.sql

echo "3. Generated schema:"
cat schema.sql

echo "4. Testing with small sample..."
beamline gen data \
    --seed 1 \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 5 \
    --output-format text

echo "Shape-driven development complete!"
```

### Schema Validation

```bash
# Validate schema matches expectations
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path production_schema.ion \
    --output-format basic-ddl > expected_schema.sql

# Compare with previous version
diff previous_schema.sql expected_schema.sql

# Generate sample data to verify
beamline gen data \
    --seed 1 \
    --start-auto \
    --script-path production_schema.ion \
    --sample-count 10
```

## Complex Shape Examples

### Arrays and Union Types

```ion
rand_processes::{
    complex_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            measurements: UniformArray::{
                min_size: 2,
                max_size: 5,
                element_type: UniformF64::{ low: 0.0, high: 100.0 }
            },
            mixed_value: UniformAnyOf::{
                types: [
                    UUID,
                    UniformI32::{ low: 1, high: 1000 },
                    Bool
                ]
            }
        }
    }
}
```

**Inferred Shape:**
```sql
-- Dataset: complex_data
"measurements" ARRAY<DOUBLE>,
"mixed_value" UNION<VARCHAR,INT,BOOL>
```

### Deeply Nested Structures

```ion
rand_processes::{
    nested_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            user: {
                profile: {
                    personal: {
                        name: LoremIpsumTitle,
                        age: UniformU8::{ low: 18, high: 80 }
                    },
                    preferences: {
                        theme: Uniform::{ choices: ["light", "dark"] },
                        notifications: Bool
                    }
                },
                stats: {
                    login_count: UniformU32,
                    last_seen: Instant
                }
            }
        }
    }
}
```

**Inferred Shape:**
```sql
-- Dataset: nested_data  
"user" STRUCT<
  "profile": STRUCT<
    "personal": STRUCT<"age": TINYINT,"name": VARCHAR>,
    "preferences": STRUCT<"notifications": BOOL,"theme": VARCHAR>
  >,
  "stats": STRUCT<"last_seen": TIMESTAMP,"login_count": INT>
>
```

## Shape Analysis and Validation

### Schema Consistency Checking

```bash
# Infer shapes from multiple related scripts
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path user_v1.ion \
    --output-format basic-ddl > user_v1_schema.sql

beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path user_v2.ion \
    --output-format basic-ddl > user_v2_schema.sql

# Compare schemas for compatibility
echo "Schema changes between versions:"
diff user_v1_schema.sql user_v2_schema.sql
```

### Multi-Dataset Schema Analysis

```bash
# Analyze all datasets in a complex script
beamline infer-shape \
    --seed 42 \
    --start-auto \
    --script-path client-service.ion \
    --output-format basic-ddl > all_schemas.sql

# Extract individual dataset schemas
grep -A 20 "-- Dataset: service" all_schemas.sql > service_schema.sql
grep -A 20 "-- Dataset: client_0" all_schemas.sql > client_schema.sql
```

## Shape-Based Development

### Database Schema Generation

```bash
#!/bin/bash
# Generate database schemas from Ion scripts

SCRIPT="$1"
OUTPUT_DIR="./schemas"

if [ -z "$SCRIPT" ]; then
    echo "Usage: $0 <script.ion>"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"
BASENAME=$(basename "$SCRIPT" .ion)

echo "Generating schemas for $SCRIPT..."

# Generate SQL DDL schema
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl > "$OUTPUT_DIR/${BASENAME}_schema.sql"

# Generate Beamline JSON for testing tools
beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format beamline-json > "$OUTPUT_DIR/${BASENAME}_schema.json"

echo "Schemas generated:"
echo "  SQL DDL: $OUTPUT_DIR/${BASENAME}_schema.sql"
echo "  JSON: $OUTPUT_DIR/${BASENAME}_schema.json"

# Show summary
echo ""
echo "Schema summary:"
grep "-- Dataset:" "$OUTPUT_DIR/${BASENAME}_schema.sql" | while read -r line; do
    dataset=$(echo "$line" | cut -d: -f2 | xargs)
    field_count=$(grep -A 100 "$line" "$OUTPUT_DIR/${BASENAME}_schema.sql" | grep '^"' | head -20 | wc -l)
    echo "  $dataset: $field_count fields"
done
```

### Schema Documentation

```bash
# Generate schema documentation for all scripts
for script in scripts/*.ion; do
    echo "## $(basename "$script" .ion)" >> SCHEMAS.md
    echo "" >> SCHEMAS.md
    echo "Generated from: \`$script\`" >> SCHEMAS.md
    echo "" >> SCHEMAS.md
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

## Understanding Type Mappings

### Ion Generator to PartiQL Type Mapping

Based on the actual implementation and README:

| Ion Generator | PartiQL Type | DDL Representation |
|---------------|--------------|-------------------|
| `Bool` | `BOOL` | `BOOL` |
| `UniformI8` | `INT64` | `TINYINT` or `INT8` |
| `UniformI16` | `INT64` | `SMALLINT` or `INT16` |
| `UniformI32` | `INT64` | `INT` |  
| `UniformI64` | `INT64` | `BIGINT` |
| `UniformU8` | `INT64` | `TINYINT` |
| `UniformU16` | `INT64` | `SMALLINT` |
| `UniformU32` | `INT64` | `INT` |
| `UniformU64` | `INT64` | `BIGINT` |
| `UniformF64` | `DOUBLE` | `DOUBLE` |
| `UniformDecimal` | `DECIMAL(p,s)` | `DECIMAL(p,s)` |
| `UUID` | `STRING` | `VARCHAR` |
| `LoremIpsumTitle` | `STRING` | `VARCHAR` |
| `Regex` | `STRING` | `VARCHAR` |
| `Format` | `STRING` | `VARCHAR` |
| `Instant` | `DATETIME` | `TIMESTAMP` |
| `Date` | `DATETIME` | `DATE` or `TIMESTAMP` |
| `Tick` | `INT64` | `INT8` or `INT64` |

### Precision and Scale Inference

For decimal types, Beamline infers precision and scale:

```ion
rand_processes::{
    decimal_test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            price: UniformDecimal::{ low: 9.99, high: 999.99 },    // DECIMAL(5,2)
            weight: UniformDecimal::{ low: 0.5, high: 10.9999 },  // DECIMAL(6,4)
            percentage: UniformDecimal::{ low: 0d0, high: 1d2 }   // DECIMAL(3,0)
        }
    }
}
```

**Inferred Shape:**
```sql
-- Dataset: decimal_test
"price" DECIMAL(5, 2),
"weight" DECIMAL(6, 4),  
"percentage" DECIMAL(3, 0)
```

## Schema Evolution and Migration

### Schema Version Comparison

```bash
#!/bin/bash
# Compare schema versions for migration planning

OLD_SCRIPT="data_model_v1.ion"
NEW_SCRIPT="data_model_v2.ion"

# Generate schemas for both versions
beamline infer-shape --seed 1 --start-auto --script-path $OLD_SCRIPT --output-format basic-ddl > v1_schema.sql
beamline infer-shape --seed 1 --start-auto --script-path $NEW_SCRIPT --output-format basic-ddl > v2_schema.sql

echo "Schema Migration Analysis"
echo "========================="

# Show differences
echo "Changes between v1 and v2:"
diff -u v1_schema.sql v2_schema.sql

echo ""
echo "Migration considerations:"

# Check for removed fields (breaking changes)
if grep -v "^--" v1_schema.sql | grep -v "^$" > v1_fields.txt && 
   grep -v "^--" v2_schema.sql | grep -v "^$" > v2_fields.txt; then
   
    removed_fields=$(comm -23 v1_fields.txt v2_fields.txt)
    if [ -n "$removed_fields" ]; then
        echo "⚠️  Breaking changes - removed fields:"
        echo "$removed_fields"
    fi
    
    added_fields=$(comm -13 v1_fields.txt v2_fields.txt)
    if [ -n "$added_fields" ]; then
        echo "✅ Added fields (non-breaking):"
        echo "$added_fields"
    fi
fi

rm -f v1_fields.txt v2_fields.txt
```

### Database Migration Script Generation

```bash
#!/bin/bash
# Generate database migration scripts

OLD_SCHEMA="$1"
NEW_SCHEMA="$2"

echo "-- Database Migration Script"
echo "-- Generated: $(date)"
echo "-- From: $OLD_SCHEMA"
echo "-- To: $NEW_SCHEMA"
echo ""

# This is a simplified example - real migration would be more complex
echo "-- Review changes manually:"
echo "-- $(diff --brief $OLD_SCHEMA $NEW_SCHEMA)"

echo ""
echo "-- Add new columns (example):"
comm -13 <(grep '^"' $OLD_SCHEMA | sort) <(grep '^"' $NEW_SCHEMA | sort) | while read -r field; do
    echo "ALTER TABLE dataset_name ADD COLUMN $field;"
done
```

## Integration Patterns

### CI/CD Schema Validation

```bash
#!/bin/bash
# CI/CD pipeline schema validation

set -e

echo "Validating Ion script schemas..."

for script in scripts/*.ion; do
    echo "Checking $(basename "$script")..."
    
    # Validate script produces valid schema
    if ! beamline infer-shape \
        --seed 1 \
        --start-auto \
        --script-path "$script" \
        --output-format text > /dev/null 2>&1; then
        echo "❌ Error: Invalid script $script"
        exit 1
    fi
    
    echo "✅ $(basename "$script") - valid schema"
done

echo "All schemas validated successfully!"
```

### Documentation Generation

```bash
# Generate schema documentation
generate_schema_docs() {
    local script_dir="$1"
    local output_file="$2"
    
    echo "# Data Model Documentation" > "$output_file"
    echo "" >> "$output_file"
    echo "Generated: $(date)" >> "$output_file"
    echo "" >> "$output_file"
    
    for script in "$script_dir"/*.ion; do
        local name=$(basename "$script" .ion)
        echo "## $name" >> "$output_file"
        echo "" >> "$output_file"
        echo "Script: \`$script\`" >> "$output_file"
        echo "" >> "$output_file"
        echo '```sql' >> "$output_file"
        
        beamline infer-shape \
            --seed 1 \
            --start-auto \
            --script-path "$script" \
            --output-format basic-ddl >> "$output_file"
            
        echo '```' >> "$output_file"
        echo "" >> "$output_file"
    done
}

generate_schema_docs "data_models" "DATA_MODEL_SCHEMAS.md"
```

## Best Practices

### 1. Always Validate Shapes

```bash
# Before generating large datasets, check the shape
beamline infer-shape --seed 1 --start-auto --script-path new_model.ion
```

### 2. Use Appropriate Output Formats

```bash
# DDL for database work
beamline infer-shape --script-path data.ion --output-format basic-ddl

# Text for debugging  
beamline infer-shape --script-path data.ion --output-format text

# JSON for automation
beamline infer-shape --script-path data.ion --output-format beamline-json
```

### 3. Document Schema Changes

```bash
# Track schema evolution
git add schemas/
git commit -m "Update user data model schema

Added:
- user.preferences.theme field
- user.stats.last_login timestamp

Modified:  
- user.profile.age now optional (nullable: 0.1)"
```

### 4. Validate Schema Compatibility

```bash
# Ensure query compatibility with schema changes
beamline infer-shape --seed 1 --start-auto --script-path new_schema.ion --output-format basic-ddl > new_schema.sql

# Generate test queries against new schema
beamline query basic \
    --seed 2 \
    --start-auto \
    --script-path new_schema.ion \
    --sample-count 10 \
    rand-select-all-fw \
    --pred-all > validation_queries.sql

echo "Schema and queries generated for validation testing"
```

## Next Steps

Now that you understand shapes and schema inference:

- [Shape Inference](./shape-inference.md) - Advanced shape inference techniques and analysis
- [Output Formats](./output-formats.md) - Deep dive into all schema output formats
- [CLI Shape Commands](../cli/shape-commands.md) - Complete CLI reference for shape operations
- [Database Integration](../database/overview.md) - Using shapes for database creation
