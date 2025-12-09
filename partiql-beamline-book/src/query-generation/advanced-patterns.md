# Advanced Query Patterns

This section covers advanced query generation using all four query strategies: `rand-sfw` (projections), `rand-select-all-efw` (exclusions), and `rand-sefw` (projections + exclusions). These strategies generate more sophisticated PartiQL queries that test complex language features.

## Query Strategy Overview

### Available Strategies

| Strategy | Query Pattern | Features |
|----------|---------------|----------|
| `rand-select-all-fw` | `SELECT * FROM WHERE` | Basic queries with WHERE clauses |
| `rand-sfw` | `SELECT fields FROM WHERE` | Custom projections + WHERE |
| `rand-select-all-efw` | `SELECT * EXCLUDE FROM WHERE` | Exclusions + WHERE |
| `rand-sefw` | `SELECT EXCLUDE FROM WHERE` | Projections + Exclusions + WHERE |

## SELECT with Projections (rand-sfw)

### Basic Projection Queries

Generate queries with specific field selections:

```bash
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 5 \
        --project-path-depth-min 1 \
        --project-path-depth-max 1 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all
```

**Example Output from README:**
```sql
SELECT test_data.completed, test_data.completed FROM test_data AS test_data
WHERE (NOT (test_data.completed) OR NOT ((test_data.created_at IS MISSING)))

SELECT test_data.completed, test_data.marketplace_id, test_data.created_at
FROM test_data AS test_data WHERE (NOT ((test_data.transaction_id IS NULL)) OR
  (((test_data.transaction_id IN [
            'Iam in.',
            'Se.',
            'Sine amicitia firmam.',
            'Notae sunt.'
          ] OR (test_data.transaction_id IS NULL)) OR
      NOT ((test_data.description IS NULL))) OR
    (test_data.marketplace_id >= 28)))

SELECT test_data, test_data.description FROM test_data AS test_data
WHERE (test_data.completed IN [ false, false ] AND
  (((test_data.price <= 5.761136291521325) AND
      NOT ((test_data.transaction_id IS MISSING))) AND
    (NOT ((test_data.created_at IS MISSING)) AND
      (test_data.created_at IS NULL))))
```

### Understanding Projection Parameters

- `--project-rand-min 2 --project-rand-max 5`: Select 2-5 fields in SELECT clause
- `--project-path-depth-max 1`: Use simple field names (no deep nesting)
- `--project-pathstep-final-all`: Allow all path types (`.field`, `[*]`, `.*`)
- `--project-type-final-all`: Project all types (scalars, structs, sequences)

## SELECT * EXCLUDE (rand-select-all-efw)

### Basic Exclusion Queries

Generate `SELECT *` queries that exclude specific fields:

```bash
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-efw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-lt \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --exclude-path-depth-min 1 \
        --exclude-path-depth-max 1 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-all \
        --exclude-type-final-all
```

**Example Output from README:**
```sql
SELECT * EXCLUDE test_data.marketplace_id, test_data.*, test_data.completed
FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)

SELECT * EXCLUDE test_data.completed FROM test_data AS test_data
WHERE (test_data.price < 18.418581624952935)

SELECT * EXCLUDE test_data.marketplace_id, test_data.completed,
  test_data.marketplace_id
FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```

### Understanding Exclusion Parameters

- `--exclude-rand-min 1 --exclude-rand-max 3`: Exclude 1-3 fields
- `--exclude-path-depth-max 1`: Use simple field exclusions
- `--exclude-pathstep-final-all`: Allow all exclusion path types
- `--exclude-type-final-all`: Exclude all types (scalars, structs, arrays)

## SELECT EXCLUDE FROM WHERE (rand-sefw)

### Complete Query Generation

The most sophisticated strategy combines projections, exclusions, and WHERE clauses:

```bash
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 5 \
        --project-path-depth-min 1 \
        --project-path-depth-max 1 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --exclude-path-depth-min 1 \
        --exclude-path-depth-max 1 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-all \
        --exclude-type-final-all
```

**Example Output from README:**
```sql
SELECT test_data.completed, test_data.completed
EXCLUDE test_data.marketplace_id, test_data.*, test_data.completed
FROM test_data AS test_data 
WHERE (NOT (test_data.completed) OR
  NOT ((test_data.created_at IS MISSING)))

SELECT test_data.completed, test_data.marketplace_id, test_data.created_at
EXCLUDE test_data.completed 
FROM test_data AS test_data
WHERE (NOT ((test_data.transaction_id IS NULL)) OR
  (((test_data.transaction_id IN [
            'Iam in.',
            'Se.',
            'Sine amicitia firmam.',
            'Notae sunt.'
          ] OR (test_data.transaction_id IS NULL)) OR
      NOT ((test_data.description IS NULL))) OR
    (test_data.marketplace_id >= 28)))

SELECT test_data, test_data.description 
EXCLUDE test_data.marketplace_id, test_data.completed, test_data.marketplace_id
FROM test_data AS test_data 
WHERE (test_data.completed IN [ false, false ] AND
  (((test_data.price <= 5.761136291521325) AND
      NOT ((test_data.transaction_id IS MISSING))) AND
    (NOT ((test_data.created_at IS MISSING)) AND
      (test_data.created_at IS NULL))))
```

## Deep Path Generation

### Complex Nested Structures

For deeply nested data structures, use higher path depth limits:

```bash
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path transactions.ion \
    --sample-count 3 \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 5 \
        --project-path-depth-min 1 \
        --project-path-depth-max 10 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --tbl-flt-path-depth-max 10 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 2 \
        --exclude-path-depth-min 3 \
        --exclude-path-depth-max 4 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-unpivot \
        --exclude-type-final-all
```

**Generated Deep Nested Queries from README:**
```sql
SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct
EXCLUDE test_data.*.*.*.*, test_data.price.* FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))

SELECT test_data.test_nest_struct.*.nested_struct.*.*.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.nested_struct.*,
  test_data.test_nest_struct.*.nested_struct.nested_struct.nested_struct.*
EXCLUDE test_data.test_nest_struct.*.*, test_data.test_nest_struct.*.*.*
FROM test_data AS test_data
WHERE ((test_data.*.*.nested_struct.*.*.*.test_int < 40) OR
  (test_data.*.*.nested_struct.nested_struct.*.nested_struct.test_int >= -9))

SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.*.nested_struct.nested_struct.nested_struct.*.*.test_int
EXCLUDE test_data.*.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.*
FROM test_data AS test_data
WHERE ((((test_data.price.value <= 6.206304713037888) OR
      (test_data.*.nested_struct.nested_struct.*.nested_struct.*.test_int <> -29))
    AND
    (test_data.test_nest_struct.*.nested_struct.*.nested_struct.nested_struct.test_int < 6))
  AND ((test_data.price > -44.666855950508584) OR
    (test_data.*.*.*.nested_struct.*.*.test_int > -42)))
```

### Controlling Path Depth

```bash
# Moderate depth for readability
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path transactions.ion \
    --sample-count 3 \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 5 \
        --project-path-depth-min 1 \
        --project-path-depth-max 3 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --tbl-flt-path-depth-max 10 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 2 \
        --exclude-path-depth-min 3 \
        --exclude-path-depth-max 4 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-unpivot \
        --exclude-type-final-all
```

**More Manageable Output:**
```sql
SELECT test_data.price, test_data.*.*.nested_struct EXCLUDE test_data.*.*.*.*,
  test_data.price.*
FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))

SELECT test_data.price, test_data.*.*.nested_struct, test_data.test_struct,
  test_data.*.*.*
EXCLUDE test_data.test_nest_struct.*.*, test_data.test_nest_struct.*.*.*
FROM test_data AS test_data
WHERE ((test_data.*.*.nested_struct.*.*.*.test_int < 40) OR
  (test_data.*.*.nested_struct.nested_struct.*.nested_struct.test_int >= -9))

SELECT test_data.transaction_id, test_data.*.nested_struct
EXCLUDE test_data.*.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.*
FROM test_data AS test_data
WHERE ((((test_data.price.value <= 6.206304713037888) OR
      (test_data.*.nested_struct.nested_struct.*.nested_struct.*.test_int <> -29))
    AND
    (test_data.test_nest_struct.*.nested_struct.*.nested_struct.nested_struct.test_int < 6))
  AND ((test_data.price > -44.666855950508584) OR
    (test_data.*.*.*.nested_struct.*.*.test_int > -42)))
```

## Path Expression Types

### Path Step Types

PartiQL Beamline can generate different path step types:

#### Projection Steps (`.field`)
```sql
SELECT test_data.transaction_id, test_data.customer.name
FROM test_data AS test_data
```

#### Index Steps (`[N]`)
```sql  
SELECT test_data.items[0], test_data.scores[5]
FROM test_data AS test_data
```

#### Wildcard Steps (`[*]`)
```sql
SELECT test_data.items[*].price, test_data.users[*].name
FROM test_data AS test_data
```

#### Unpivot Steps (`.*`)
```sql
SELECT test_data.metadata.*, test_data.settings.*
FROM test_data AS test_data
```

### Path Configuration Examples

#### Simple Projections Only

```bash
# Only use projection steps
partiql-beamline-cli query basic \
    --seed 100 \
    --start-auto \
    --script-path nested_data.ion \
    --sample-count 5 \
    rand-sfw \
        --project-rand-min 3 \
        --project-rand-max 5 \
        --project-path-depth-max 3 \
        --project-pathstep-internal-project \
        --project-pathstep-final-project \
        --pred-all
```

#### Include Wildcards

```bash
# Add wildcard and unpivot paths
partiql-beamline-cli query basic \
    --seed 200 \
    --start-auto \
    --script-path array_data.ion \
    --sample-count 5 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --project-path-depth-max 2 \
        --project-pathstep-internal-all \
        --project-pathstep-final-foreach \
        --project-pathstep-final-unpivot \
        --pred-all
```

## Complex Query Combinations

### Full Feature Queries

Use all query features together for comprehensive PartiQL testing:

```bash
partiql-beamline-cli query basic \
    --seed 2000 \
    --start-auto \
    --script-path comprehensive_schema.ion \
    --sample-count 5 \
    rand-sefw \
        --project-rand-min 3 \
        --project-rand-max 8 \
        --project-path-depth-min 1 \
        --project-path-depth-max 5 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 6 \
        --tbl-flt-path-depth-max 4 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-all \
        --tbl-flt-type-final-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 4 \
        --exclude-path-depth-min 1 \
        --exclude-path-depth-max 3 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-all \
        --exclude-type-final-all \
        --pred-all
```

This generates very complex queries testing the full range of PartiQL features.

## Type-Specific Query Generation

### Scalar Type Focus

Generate queries that only work with scalar values:

```bash
partiql-beamline-cli query basic \
    --seed 300 \
    --start-auto \
    --script-path mixed_types.ion \
    --sample-count 8 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --project-type-final-scalar \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --tbl-flt-type-final-scalar \
        --pred-comparison
```

**Results focus on scalar fields:**
```sql
SELECT test_data.price, test_data.completed, test_data.marketplace_id
FROM test_data AS test_data
WHERE (test_data.transaction_id = 'some-uuid' AND test_data.price > 100.0)
```

### Structure Type Queries

Generate queries that work with complex structures:

```bash
partiql-beamline-cli query basic \
    --seed 400 \
    --start-auto \
    --script-path nested_objects.ion \
    --sample-count 5 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 3 \
        --project-type-final-struct \
        --tbl-flt-type-final-struct \
        --pred-absent --pred-eq
```

## Advanced Testing Patterns

### Edge Case Query Generation

Test PartiQL edge cases and complex scenarios:

```bash
# Generate edge case queries
partiql-beamline-cli query basic \
    --seed 500 \
    --start-auto \
    --script-path edge_case_schema.ion \
    --sample-count 10 \
    rand-sefw \
        --project-path-depth-min 4 \
        --project-path-depth-max 8 \
        --project-pathstep-internal-foreach \
        --project-pathstep-final-unpivot \
        --tbl-flt-path-depth-max 6 \
        --tbl-flt-pathstep-internal-unpivot \
        --tbl-flt-pathstep-final-foreach \
        --exclude-path-depth-min 2 \
        --exclude-path-depth-max 5 \
        --exclude-pathstep-final-unpivot \
        --pred-all
```

### Performance Stress Testing

Generate computationally expensive queries:

```bash
# Create performance stress test queries
partiql-beamline-cli query basic \
    --seed 600 \
    --start-auto \
    --script-path large_schema.ion \
    --sample-count 20 \
    rand-sefw \
        --project-rand-min 5 \
        --project-rand-max 15 \
        --project-path-depth-max 6 \
        --tbl-flt-rand-min 3 \
        --tbl-flt-rand-max 10 \
        --tbl-flt-path-depth-max 5 \
        --exclude-rand-min 2 \
        --exclude-rand-max 8 \
        --pred-all > stress_test_queries.sql
```

## Integration Workflows

### Multi-Strategy Testing

Generate different query types for comprehensive testing:

```bash
#!/bin/bash
# Generate complete query test suite

SCRIPT="test_schema.ion"
SEED=12345
SAMPLE_COUNT=10

echo "Generating comprehensive query test suite..."

# Basic SELECT * queries
partiql-beamline-cli query basic \
    --seed $SEED \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count $SAMPLE_COUNT \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all > select_star.sql

# Projection queries
partiql-beamline-cli query basic \
    --seed $((SEED + 1)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count $SAMPLE_COUNT \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 5 \
        --project-path-depth-max 2 \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all > projections.sql

# Exclusion queries  
partiql-beamline-cli query basic \
    --seed $((SEED + 2)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count $SAMPLE_COUNT \
    rand-select-all-efw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --pred-all > exclusions.sql

# Complex combined queries
partiql-beamline-cli query basic \
    --seed $((SEED + 3)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count $SAMPLE_COUNT \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --exclude-rand-min 1 \
        --exclude-rand-max 2 \
        --pred-all > combined.sql

echo "Query test suite generated:"
echo "- select_star.sql: $(wc -l < select_star.sql) queries"
echo "- projections.sql: $(wc -l < projections.sql) queries"
echo "- exclusions.sql: $(wc -l < exclusions.sql) queries"  
echo "- combined.sql: $(wc -l < combined.sql) queries"
echo "Total: $(($(wc -l < select_star.sql) + $(wc -l < projections.sql) + $(wc -l < exclusions.sql) + $(wc -l < combined.sql))) queries"
```

### Query Complexity Progression

```bash
#!/bin/bash
# Generate queries with increasing complexity

SCRIPT="complex_data.ion"
BASE_SEED=1000

for complexity_level in 1 2 3 5; do
    echo "Generating complexity level $complexity_level queries..."
    
    partiql-beamline-cli query basic \
        --seed $((BASE_SEED + complexity_level)) \
        --start-auto \
        --script-path $SCRIPT \
        --sample-count 10 \
        rand-sefw \
            --project-rand-min $complexity_level \
            --project-rand-max $((complexity_level * 2)) \
            --project-path-depth-max $complexity_level \
            --tbl-flt-rand-min $complexity_level \
            --tbl-flt-rand-max $((complexity_level * 2)) \
            --tbl-flt-path-depth-max $complexity_level \
            --exclude-rand-min 1 \
            --exclude-rand-max $complexity_level \
            --pred-all > "complexity_${complexity_level}.sql"
            
    echo "  Generated: $(wc -l < complexity_${complexity_level}.sql) queries"
done

echo "Query complexity suite completed"
```

## Best Practices

### 1. Match Complexity to Use Case

```bash
# Simple testing - basic patterns
--project-path-depth-max 2 --tbl-flt-path-depth-max 2

# Comprehensive testing - complex patterns
--project-path-depth-max 5 --tbl-flt-path-depth-max 5

# Edge case testing - maximum complexity
--project-path-depth-max 10 --tbl-flt-path-depth-max 10
```

### 2. Balance Query Features

```bash
# Don't overload with too many features at once
--project-rand-min 2 --project-rand-max 4    # Moderate projections
--exclude-rand-min 1 --exclude-rand-max 2    # Few exclusions
--tbl-flt-rand-min 1 --tbl-flt-rand-max 3   # Simple WHERE clauses
```

### 3. Test Incrementally

```bash
# Start simple
partiql-beamline-cli query basic --script-path data.ion --sample-count 3 rand-select-all-fw --pred-eq

# Add projections
partiql-beamline-cli query basic --script-path data.ion --sample-count 3 rand-sfw --project-rand-min 2 --project-rand-max 3 --pred-eq

# Add exclusions
partiql-beamline-cli query basic --script-path data.ion --sample-count 3 rand-sefw --project-rand-min 2 --exclude-rand-min 1 --pred-eq

# Full complexity
partiql-beamline-cli query basic --script-path data.ion --sample-count 5 rand-sefw --project-rand-min 3 --exclude-rand-min 2 --tbl-flt-rand-min 2 --pred-all
```

### 4. Validate Complex Queries

```bash
# Generate and validate complex queries
partiql-beamline-cli query basic \
    --seed 700 \
    --start-auto \
    --script-path validation_schema.ion \
    --sample-count 15 \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 6 \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --pred-all > complex_validation.sql

# Check each query
# your-partiql-parser --check-syntax complex_validation.sql
```

## Next Steps

Now that you understand advanced query patterns:

- [Parameterization](./parameterization.md) - Complete reference for all query generation parameters
- [CLI Query Commands](../cli/query-commands.md) - Detailed CLI usage with all options
- [Examples](../examples/sensors.md) - See query generation in complete workflows
