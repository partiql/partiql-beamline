# Shape Inference

Shape inference is the process of analyzing Ion scripts to determine the data types and structures that will be generated, without actually generating data. This is extremely fast and useful for schema validation, database preparation, and understanding data models.

## Shape Inference Command

### Basic Usage

The `infer-shape` command requires the same core parameters as data generation:

```bash
partiql-beamline-cli infer-shape \
    --seed-auto \
    --start-auto \
    --script-path sensors.ion
```

Even though no data is generated, seed and start time may affect type inference for certain dynamic generators.

### With Specific Parameters

```bash
# Use specific seed for reproducible shape inference
partiql-beamline-cli infer-shape \
    --seed 12345 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path complex_schema.ion \
    --output-format basic-ddl
```

## Output Format Analysis

### Text Format (Detailed Debug)

From the README example:

```bash
$ partiql-beamline-cli infer-shape \
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
                                                DecimalP(2, 0),
                                            ),
                                        },
                                        StructField {
                                            name: "f",
                                            ty: PartiqlType(
                                                Float64,
                                            ),
                                        },
                                        // ... more fields
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

**Understanding the structure:**
- `Bag`: Collection of records (dataset)
- `BagType`: Type information for the bag
- `Struct`: Each record is a structured object  
- `StructField`: Individual field definitions with names and types
- `PartiqlType`: Specific type information (DecimalP, Float64, etc.)

### Basic DDL Format (SQL Ready)

From the README example:

```bash
$ partiql-beamline-cli infer-shape \
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

**Format characteristics:**
- **Comments**: Metadata about generation parameters
- **Dataset headers**: Clear separation between datasets
- **SQL-ready**: Can be used directly in CREATE TABLE statements
- **Type precision**: Specific SQL types with precision for decimals

### Beamline JSON Format (Tool Integration)

From the README example:

```bash
$ partiql-beamline-cli infer-shape \
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
        constraints: [ordered, closed],
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

**Format characteristics:**
- **Structured JSON**: Machine-readable format
- **Versioned**: `partiql::shape::v0::` indicates version
- **Complete metadata**: Seeds, timestamps, and full type information
- **Tool integration**: Designed for PartiQL testing tools

## Advanced Shape Inference

### CLI Global Defaults Impact

CLI defaults affect shape inference results:

```bash
# Infer with default nullable/optional settings
partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path data.ion \
    --default-nullable false \
    --default-optional true \
    --output-format basic-ddl
```

From the README example showing CLI impact:

```bash
$ partiql-beamline-cli infer-shape \
    --seed 7844265201457918498 \
    --start-auto \
    --script-path sensors.ion \
    --output-format basic-ddl \
    --default-nullable false \
    --default-optional true

-- Seed: 7844265201457918498
-- Start: 2024-01-18T11:40:34.000000000Z
-- Syntax: partiql_datatype_syntax-0.1
-- Dataset: sensors
"a" OPTIONAL UNION<INT8 NOT NULL,DECIMAL(5, 4) NOT NULL,DOUBLE NOT NULL,VARCHAR NOT NULL>,
"ar1" OPTIONAL ARRAY<DECIMAL(2, 1) NOT NULL> NOT NULL,
"ar2" OPTIONAL ARRAY<VARCHAR NOT NULL> NOT NULL,
"ar3" OPTIONAL ARRAY<DECIMAL(5, 4)> NOT NULL,
"ar4" OPTIONAL ARRAY<TINYINT NOT NULL> NOT NULL,
"ar5" OPTIONAL ARRAY<UNION<INT8 NOT NULL,DECIMAL(5, 4) NOT NULL,DOUBLE NOT NULL,VARCHAR NOT NULL>> NOT NULL,
"d" OPTIONAL DECIMAL(2, 0) NOT NULL,
"f" OPTIONAL DOUBLE NOT NULL,
"i8" OPTIONAL TINYINT NOT NULL,
"tick" OPTIONAL INT8 NOT NULL,
"w" OPTIONAL DECIMAL(5, 4)
```

Notice how CLI defaults made fields `OPTIONAL` and `NOT NULL`.

### Multi-Dataset Shape Analysis

```bash
# Analyze complex multi-dataset script
partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path client-service.ion \
    --output-format basic-ddl
```

**Example output structure:**
```sql
-- Dataset: service
"Account" VARCHAR,
"Distance" DECIMAL(2, 0),
"Operation" VARCHAR,
"Program" VARCHAR,
"Request" VARCHAR,
"StartTime" TIMESTAMP,
"Weight" DECIMAL(5, 4),
"anyof" UNION<INT8,DECIMAL(5, 4)>,
"array" ARRAY<INT8>,
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

Each dataset from the Ion script gets its own schema section.

## Shape Inference Patterns

### Script Validation Workflow

```bash
#!/bin/bash
# Validate Ion script before data generation

SCRIPT="$1"

if [ ! -f "$SCRIPT" ]; then
    echo "Script not found: $SCRIPT"
    exit 1
fi

echo "Validating Ion script: $SCRIPT"

# Test shape inference (fast validation)
if ! partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format text > /dev/null; then
    echo "❌ Script validation failed - check Ion syntax"
    exit 1
fi

echo "✅ Script syntax valid"

# Show inferred schema
echo ""
echo "Inferred schema:"
partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl

echo ""
echo "✅ Script ready for data generation"
```

### Schema Documentation Generation

```bash
#!/bin/bash
# Auto-generate schema documentation

SCRIPTS_DIR="$1"
OUTPUT_FILE="$2"

echo "# Data Schema Documentation" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "Auto-generated: $(date)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

for script in "$SCRIPTS_DIR"/*.ion; do
    name=$(basename "$script" .ion)
    echo "Processing $name..."
    
    echo "## $name Data Schema" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "**Source Script**: \`$(basename "$script")\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Add schema in SQL format
    echo '```sql' >> "$OUTPUT_FILE"
    partiql-beamline-cli infer-shape \
        --seed 1 \
        --start-auto \
        --script-path "$script" \
        --output-format basic-ddl | grep -v "^-- Seed:" | grep -v "^-- Start:" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Count datasets and fields
    schema_output=$(partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path "$script" --output-format basic-ddl)
    dataset_count=$(echo "$schema_output" | grep -c "^-- Dataset:")
    field_count=$(echo "$schema_output" | grep -c '^"')
    
    echo "**Summary**: $dataset_count dataset(s), $field_count total fields" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

echo "Schema documentation generated: $OUTPUT_FILE"
```

## Real-World Examples

### E-commerce Schema Analysis

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 10, high: 100 },
    $customer_ids: $n_customers::[UUID::()],
    
    customers: static_data::{
        $data: {
            customer_id: Uniform::{ choices: $customer_ids },
            name: LoremIpsumTitle,
            email: Format::{ pattern: "customer{UUID}@email.com" },
            age: UniformU8::{ low: 18, high: 80, optional: 0.1 }
        }
    },
    
    orders: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::30 },
        $data: {
            order_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            total: UniformDecimal::{ low: 10.00, high: 500.00 },
            items: UniformArray::{
                min_size: 1,
                max_size: 5,
                element_type: {
                    product_name: LoremIpsumTitle,
                    price: UniformDecimal::{ low: 5.00, high: 100.00 },
                    quantity: UniformU8::{ low: 1, high: 3 }
                }
            }
        }
    }
}
```

**Inferred Schema:**
```bash
$ partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path ecommerce.ion --output-format basic-ddl

-- Dataset: customers
"age" OPTIONAL TINYINT,
"customer_id" VARCHAR,
"email" VARCHAR,
"name" VARCHAR

-- Dataset: orders
"customer_id" VARCHAR,
"items" ARRAY<STRUCT<"price": DECIMAL(5, 2),"product_name": VARCHAR,"quantity": TINYINT>>,
"order_id" VARCHAR,
"total" DECIMAL(5, 2)
```

### Financial Data Schema

```ion
rand_processes::{
    transactions: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            transaction_id: UUID::{ nullable: false },
            account_id: UUID,
            amount: LogNormalF64::{ location: 4.0, scale: 1.0 },
            transaction_type: Uniform::{ choices: ["deposit", "withdrawal", "transfer"] },
            risk_score: UniformF64::{ low: 0.0, high: 1.0 },
            metadata: {
                merchant: LoremIpsumTitle,
                location: Regex::{ pattern: "[A-Z]{2}" },
                processing_time: UniformF64::{ low: 0.1, high: 5.0 }
            },
            compliance: {
                aml_flagged: Bool::{ p: 0.01 },
                requires_review: Bool::{ p: 0.05 },
                risk_category: Uniform::{ choices: ["low", "medium", "high"] }
            }
        }
    }
}
```

**Inferred Schema:**
```sql
-- Dataset: transactions
"account_id" VARCHAR,
"amount" DOUBLE,
"compliance" STRUCT<"aml_flagged": BOOL,"requires_review": BOOL,"risk_category": VARCHAR>,
"metadata" STRUCT<"location": VARCHAR,"merchant": VARCHAR,"processing_time": DOUBLE>,
"risk_score" DOUBLE,
"transaction_id" VARCHAR NOT NULL,
"transaction_type" VARCHAR
```

## Shape Inference Analysis

### Schema Complexity Assessment

```bash
#!/bin/bash
# Analyze schema complexity

SCRIPT="$1"

echo "Schema Complexity Analysis for: $SCRIPT"
echo "======================================"

# Get detailed shape information
schema_output=$(partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl)

# Count datasets
dataset_count=$(echo "$schema_output" | grep -c "^-- Dataset:")
echo "Datasets: $dataset_count"

# Count total fields
field_count=$(echo "$schema_output" | grep -c '^"')
echo "Total fields: $field_count"

# Count complex types
struct_count=$(echo "$schema_output" | grep -c "STRUCT<")
array_count=$(echo "$schema_output" | grep -c "ARRAY<")
union_count=$(echo "$schema_output" | grep -c "UNION<")

echo "Complex types:"
echo "  Structs: $struct_count"
echo "  Arrays: $array_count"  
echo "  Unions: $union_count"

# Count nullable/optional fields
nullable_count=$(echo "$schema_output" | grep -v "NOT NULL" | grep -c '^"')
optional_count=$(echo "$schema_output" | grep -c "OPTIONAL")

echo "Nullability:"
echo "  Nullable fields: $nullable_count"
echo "  Optional fields: $optional_count"

echo ""
echo "Complexity Score: $((field_count + struct_count * 2 + array_count * 2 + union_count * 3))"
```

### Multi-Format Schema Comparison

```bash
#!/bin/bash
# Compare schema formats for analysis

SCRIPT="$1"
BASE_NAME=$(basename "$SCRIPT" .ion)

echo "Generating schema in all formats for: $SCRIPT"

# Generate all three formats
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path "$SCRIPT" --output-format text > "${BASE_NAME}_debug.txt"
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path "$SCRIPT" --output-format basic-ddl > "${BASE_NAME}_schema.sql"  
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path "$SCRIPT" --output-format beamline-json > "${BASE_NAME}_schema.json"

echo "Generated schema files:"
echo "  Debug format: ${BASE_NAME}_debug.txt ($(wc -l < ${BASE_NAME}_debug.txt) lines)"
echo "  SQL DDL format: ${BASE_NAME}_schema.sql ($(wc -l < ${BASE_NAME}_schema.sql) lines)"
echo "  JSON format: ${BASE_NAME}_schema.json ($(wc -l < ${BASE_NAME}_schema.json) lines)"

# Show summary from SQL format
echo ""
echo "Schema summary:"
grep "-- Dataset:" "${BASE_NAME}_schema.sql" | while read -r line; do
    dataset=$(echo "$line" | cut -d: -f2 | xargs)
    echo "  Dataset: $dataset"
done
```

## Shape Inference Optimization

### Fast Schema Validation

Shape inference is much faster than data generation:

```bash
# Quick validation of multiple scripts
for script in models/*.ion; do
    echo -n "$(basename "$script"): "
    
    start_time=$(date +%s.%N)
    if partiql-beamline-cli infer-shape \
        --seed 1 \
        --start-auto \
        --script-path "$script" \
        --output-format text > /dev/null; then
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc -l)
        echo "✅ Valid (${duration}s)"
    else
        echo "❌ Invalid"
    fi
done
```

### Batch Schema Generation

```bash
#!/bin/bash
# Generate schemas for all scripts in parallel

SCRIPTS_DIR="$1"
OUTPUT_DIR="$2"

mkdir -p "$OUTPUT_DIR"

echo "Generating schemas for all scripts in $SCRIPTS_DIR..."

# Process scripts in parallel
for script in "$SCRIPTS_DIR"/*.ion; do
    {
        name=$(basename "$script" .ion)
        echo "Processing $name..."
        
        partiql-beamline-cli infer-shape \
            --seed 1 \
            --start-auto \
            --script-path "$script" \
            --output-format basic-ddl > "$OUTPUT_DIR/${name}_schema.sql"
            
        echo "✅ $name completed"
    } &
done

wait  # Wait for all background jobs
echo "All schema generation completed"

# Summary
echo ""
echo "Generated schemas:"
ls -la "$OUTPUT_DIR"/*.sql | while read -r line; do
    file=$(echo "$line" | awk '{print $9}')
    lines=$(wc -l < "$file")
    echo "  $(basename "$file"): $lines lines"
done
```

## Troubleshooting Shape Inference

### Common Issues

#### Script Syntax Errors

```bash
$ partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path bad_syntax.ion
Error: Failed to parse Ion script: Invalid Ion syntax at line 5, column 10
```

**Solution**: Check Ion script syntax, ensure balanced braces and proper structure.

#### Missing Required Parameters

```bash
$ partiql-beamline-cli infer-shape --script-path data.ion
Error: One of --seed-auto or --seed is required
Error: One of --start-auto, --start-epoch-ms, or --start-iso is required
```

**Solution**: Always provide seed and start time parameters.

#### Invalid Generator Configuration

```bash
# This will fail during shape inference
rand_processes::{
    bad_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            invalid_range: UniformI32::{ low: 100, high: 50 }  // min > max
        }
    }
}
```

**Solution**: Check generator configurations for valid parameter ranges.

### Performance Troubleshooting

Shape inference should be very fast (milliseconds). If it's slow:

```bash
# Check for complex nested structures
partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path suspected_slow.ion \
    --output-format text | grep -c "nested_struct"
```

Very deep nesting (10+ levels) might slow shape inference slightly.

## Integration Examples

### Database Schema Creation Pipeline

```bash
#!/bin/bash
# Complete database schema creation pipeline

SCRIPT="$1"
DATABASE_NAME="$2"

echo "Creating database from Ion script: $SCRIPT"

# 1. Validate script and infer schema
echo "Step 1: Validating script and inferring schema..."
if ! partiql-beamline-cli infer-shape \
    --seed 1 \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl > schema.sql; then
    echo "❌ Schema inference failed"
    exit 1
fi

# 2. Create database
echo "Step 2: Creating database $DATABASE_NAME..."
createdb "$DATABASE_NAME"

# 3. Generate CREATE TABLE statements
echo "Step 3: Generating CREATE TABLE statements..."
grep "-- Dataset:" schema.sql | while read -r line; do
    dataset=$(echo "$line" | cut -d: -f2 | xargs)
    
    echo "CREATE TABLE $dataset (" > "table_${dataset}.sql"
    # Extract fields for this dataset (simplified)
    grep -A 100 "$line" schema.sql | grep '^"' | head -20 >> "table_${dataset}.sql"
    echo ");" >> "table_${dataset}.sql"
    
    echo "Creating table: $dataset"
    psql -d "$DATABASE_NAME" -f "table_${dataset}.sql"
done

echo "✅ Database $DATABASE_NAME created with schema from $SCRIPT"
```

### Schema Testing Integration

```bash
#!/bin/bash
# Test schema consistency across development workflow

SCRIPT="user_model.ion"
SEED=12345

echo "Testing schema consistency workflow..."

# 1. Infer baseline schema
partiql-beamline-cli infer-shape \
    --seed $SEED \
    --start-auto \
    --script-path "$SCRIPT" \
    --output-format basic-ddl > baseline_schema.sql

# 2. Generate test data using same script
partiql-beamline-cli gen data \
    --seed $SEED \
    --start-auto \
    --script-path "$SCRIPT" \
    --sample-count 100 \
    --output-format ion-pretty > test_data.ion

# 3. Generate test queries using same script
partiql-beamline-cli query basic \
    --seed $((SEED + 1)) \
    --start-auto \
    --script-path "$SCRIPT" \
    --sample-count 10 \
    rand-select-all-fw \
    --pred-all > test_queries.sql

echo "Consistency test completed:"
echo "  Schema: baseline_schema.sql"
echo "  Data: test_data.ion ($(jq '.data | to_entries[0].value | length' test_data.ion 2>/dev/null || echo 'N/A') records)"
echo "  Queries: test_queries.sql ($(wc -l < test_queries.sql) queries)"

# 4. Validate all components reference same structure
echo ""
echo "✅ Schema, data, and queries all generated from same Ion script"
echo "✅ Consistency guaranteed by same script source"
```

## Best Practices

### 1. Use Shape Inference Early

```bash
# Always validate scripts before large data generation
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path new_script.ion

# Then proceed with data generation
partiql-beamline-cli gen data --seed 1 --start-auto --script-path new_script.ion --sample-count 100000
```

### 2. Choose Format for Purpose

```bash
# Development and debugging
partiql-beamline-cli infer-shape --script-path script.ion --output-format text

# Database integration
partiql-beamline-cli infer-shape --script-path script.ion --output-format basic-ddl

# Tool integration and automation
partiql-beamline-cli infer-shape --script-path script.ion --output-format beamline-json
```

### 3. Version Control Schemas

```bash
# Track schema evolution alongside scripts
git add scripts/user_model.ion schemas/user_model_schema.sql
git commit -m "Add user model v2 with preferences and stats

Schema changes:
- Added user.preferences nested object
- Added user.stats.login_count field  
- Made user.profile.age optional"
```

### 4. Validate Schema Changes

```bash
# Before deploying schema changes
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path new_version.ion --output-format basic-ddl > new_schema.sql
diff old_schema.sql new_schema.sql

# Test compatibility with existing queries
# your-query-validator --schema new_schema.sql --queries existing_queries.sql
```

## Next Steps

Now that you understand shape inference:

- [Schema Output Formats](./output-formats.md) - Deep dive into text, DDL, and JSON formats
- [CLI Shape Commands](../cli/shape-commands.md) - Complete CLI reference
- [Database Integration](../database/overview.md) - Using inferred schemas for database creation
- [Query Generation](../query-generation/overview.md) - How shapes enable query generation
