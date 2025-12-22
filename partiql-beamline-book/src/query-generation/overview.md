# Query Generator Overview

PartiQL Beamline's query generator creates reproducible PartiQL queries that match the shapes and types of data defined in Ion scripts. This allows you to generate realistic test queries for PartiQL implementations, ensuring your queries are both syntactically valid and semantically meaningful for your data structures.

## What is Query Generation?

The query generator analyzes the data shapes from your Ion scripts and creates PartiQL queries that:

- **Match your data structure**: Queries reference actual fields and types from your data
- **Are syntactically correct**: All generated queries parse and execute properly
- **Have realistic complexity**: Configurable query patterns from simple to complex
- **Are reproducible**: Same seed produces identical query sequences
- **Test diverse patterns**: Cover different PartiQL constructs and edge cases

## How Query Generation Works

### Process Flow

1. **Script Analysis**: Parse Ion script to understand data shapes
2. **Shape Inference**: Determine field types, structures, and relationships  
3. **Query Strategy**: Apply configured query generation strategy
4. **Query Construction**: Build queries matching data structure
5. **Output Generation**: Produce formatted PartiQL queries

### Shape-Aware Generation

The query generator understands your data structure:

```ion
rand_processes::{
    test_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: milliseconds::100 },
        $data: {
            transaction_id: UUID::{ nullable: false },
            marketplace_id: UniformU8::{ nullable: false },
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            created_at: Instant,
            completed: Bool,
            price: UniformDecimal::{ low: 2.99, high: 99999.99, optional: true }
        }
    }
}
```

Generated queries will reference actual fields like `transaction_id`, `marketplace_id`, `country_code`, etc.

## Query Generation Strategies

PartiQL Beamline supports four main query generation strategies:

### 1. `rand-select-all-fw` - SELECT * FROM WHERE

Generates `SELECT *` queries with WHERE clauses:

```bash
beamline query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-lt
```

**Example Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)
SELECT * FROM test_data AS test_data WHERE (test_data.price < 18.418581624952935)  
SELECT * FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```

### 2. `rand-sfw` - SELECT fields FROM WHERE

Generates queries with specific field projections and WHERE clauses:

```bash
beamline query basic \
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

**Example Output:**
```sql
SELECT test_data.completed, test_data.completed FROM test_data AS test_data
WHERE (NOT (test_data.completed) OR NOT ((test_data.created_at IS MISSING)))

SELECT test_data.completed, test_data.marketplace_id, test_data.created_at
FROM test_data AS test_data WHERE (NOT ((test_data.transaction_id IS NULL)) OR
  (((test_data.transaction_id IN ['Iam in.', 'Se.']) OR 
      NOT ((test_data.description IS NULL))) OR
    (test_data.marketplace_id >= 28)))
```

### 3. `rand-select-all-efw` - SELECT * EXCLUDE FROM WHERE

Generates `SELECT * EXCLUDE` queries:

```bash
beamline query basic \
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

**Example Output:**
```sql
SELECT * EXCLUDE test_data.marketplace_id, test_data.*, test_data.completed
FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)

SELECT * EXCLUDE test_data.completed FROM test_data AS test_data
WHERE (test_data.price < 18.418581624952935)
```

### 4. `rand-sefw` - SELECT EXCLUDE FROM WHERE  

Generates queries with projections, exclusions, and WHERE clauses:

```bash
beamline query basic \
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

## Path Generation

Query generation creates paths that navigate your data structure:

### Path Components

- **Projection paths**: `field_name`, `object.field`, `nested.object.field`
- **Index paths**: `array[0]`, `array[5]`
- **Wildcard paths**: `array[*]`, `object.*`
- **Deep paths**: `nested.object.array[*].field`

### Path Depth Control

```bash
# Simple paths (depth 1)
--tbl-flt-path-depth-max 1
# Results: test_data.price, test_data.completed

# Complex paths (depth 3+) 
--tbl-flt-path-depth-max 5
# Results: test_data.nested.object.array[*].field
```

### Real Examples from Complex Data

From the README's transactions.ion example with nested structures:

```bash
beamline query basic \
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
        --pred-all
```

**Generated Deep Paths:**
```sql
SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct
EXCLUDE test_data.*.*.*.*, test_data.price.*
FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))
```

## Predicate Generation

### Available Predicate Types

Based on the README's comprehensive predicate options:

- **Comparison**: `<`, `<=`, `>`, `>=`, `=`, `<>`
- **Range**: `BETWEEN value1 AND value2`
- **String**: `LIKE pattern`, `NOT LIKE pattern`
- **Set membership**: `IN (value1, value2)`, `NOT IN (value1, value2)`
- **Null testing**: `IS NULL`, `IS NOT NULL`
- **Missing testing**: `IS MISSING`, `IS NOT MISSING`
- **Logical**: `AND`, `OR`, `NOT`

### Predicate Configuration

```bash
# Only less-than predicates
--pred-lt

# All comparison predicates
--pred-comparison

# All predicates including logical operators
--pred-all

# Only null/missing testing
--pred-absent
```

### Real Predicate Examples

From the README examples:

```sql
-- Simple predicates
WHERE (test_data.marketplace_id < -5)
WHERE (test_data.price BETWEEN 10.0 AND 100.0)

-- Complex logical combinations
WHERE (((test_data.country_code <> 'Qua maxime ceterorum.') AND
    (NOT (test_data.completed IN [false, true]) OR
      (test_data.description = 'Non faciant.'))) AND
  (NOT ((test_data.price IS MISSING)) AND (test_data.price IS MISSING)))

-- Null and missing testing
WHERE (test_data.email IS NOT NULL AND test_data.optional_field IS MISSING)
```

## Configuration Parameters

### Table Filter Parameters

Control WHERE clause generation:

| Parameter | Description | Values |
|-----------|-------------|--------|
| `--tbl-flt-rand-min` | Minimum predicates | 1-255 |
| `--tbl-flt-rand-max` | Maximum predicates | 1-255 |
| `--tbl-flt-path-depth-max` | Maximum path depth | 1-255 |

### Path Step Configuration

Control how paths navigate through data:

| Parameter | Description |
|-----------|-------------|
| `--tbl-flt-pathstep-internal-all` | Enable all internal path steps |
| `--tbl-flt-pathstep-internal-project` | Enable projection steps (`.field`) |
| `--tbl-flt-pathstep-internal-index` | Enable index steps (`[1]`) |
| `--tbl-flt-pathstep-internal-foreach` | Enable wildcard steps (`[*]`) |
| `--tbl-flt-pathstep-internal-unpivot` | Enable unpivot steps (`.*`) |

### Type Constraints

Control what types can appear in query paths:

| Parameter | Description |
|-----------|-------------|
| `--tbl-flt-type-final-all` | Allow all final types |
| `--tbl-flt-type-final-scalar` | Only scalar types (`9`, `'text'`, `true`) |
| `--tbl-flt-type-final-sequence` | Only sequence types (`[1,2,3]`) |
| `--tbl-flt-type-final-struct` | Only struct types (`{'a': 1}`) |

## Real Query Examples

### Simple Transaction Queries

Based on simple_transactions.ion test script:

```bash
beamline query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-lt
```

**Results:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)
SELECT * FROM test_data AS test_data WHERE (test_data.price < 18.418581624952935)
SELECT * FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```

### Complex Nested Structure Queries

With more complex path generation:

```bash
beamline query basic \
    --seed 1234 \
    --start-auto \
    --script-path transactions.ion \
    --sample-count 2 \
    rand-sefw \
        --project-rand-min 2 \
        --project-rand-max 3 \
        --project-path-depth-max 6 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 3 \
        --tbl-flt-path-depth-max 6 \
        --pred-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 2 \
        --exclude-path-depth-max 4
```

**Results with Deep Paths:**
```sql
SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct.*.*
EXCLUDE test_data.test_nest_struct.*.*, test_data.price.*
FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.*.*.nested_struct.nested_struct.*.nested_struct.test_int >= -9))
```

## Reproducible Query Generation

### Consistent Query Generation

Use specific seeds for reproducible query sets:

```bash
# Generate same queries each time
beamline query basic \
    --seed 12345 \
    --start-auto \
    --script-path data.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all
```

### Query Complexity Control

#### Simple Queries

```bash
# Generate simple queries for basic testing
beamline query basic \
    --seed 100 \
    --start-auto \
    --script-path data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --tbl-flt-path-depth-max 1 \
        --pred-eq
```

#### Complex Queries

```bash
# Generate complex queries for comprehensive testing
beamline query basic \
    --seed 200 \
    --start-auto \
    --script-path nested_data.ion \
    --sample-count 5 \
    rand-sefw \
        --project-rand-min 3 \
        --project-rand-max 8 \
        --project-path-depth-max 5 \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 6 \
        --tbl-flt-path-depth-max 5 \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --pred-all
```

## Integration Patterns

### Testing Workflow Integration

```bash
#!/bin/bash
# Generate test data and matching queries

SCRIPT="test_data.ion"
SEED=12345

# Generate test dataset
beamline gen data \
    --seed $SEED \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 1000 \
    --output-format ion-pretty > test_data.ion

# Generate queries for the dataset
beamline query basic \
    --seed $((SEED + 1)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 20 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 4 \
        --pred-all > test_queries.sql

echo "Generated test data and matching queries"
```

### Query Validation Testing

```bash
# Generate queries to test PartiQL implementation
beamline query basic \
    --seed 300 \
    --start-auto \
    --script-path complex_schema.ion \
    --sample-count 50 \
    rand-sfw \
        --project-rand-min 1 \
        --project-rand-max 5 \
        --project-path-depth-max 3 \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all > validation_queries.sql

# Test each query against your PartiQL implementation
while IFS= read -r query; do
    echo "Testing query: $query"
    # Run query against your PartiQL engine
    # your-partiql-engine --query "$query" --data test_data.ion
done < validation_queries.sql
```

## Advanced Query Patterns

### Null and Missing Value Testing

Generate queries that test NULL and MISSING value handling:

```bash
beamline query basic \
    --seed 400 \
    --start-auto \
    --script-path nullable_data.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --pred-absent  # Focus on IS NULL, IS NOT NULL, IS MISSING, IS NOT MISSING
```

**Example Queries:**
```sql
SELECT * FROM test_data WHERE (test_data.optional_field IS MISSING)
SELECT * FROM test_data WHERE (test_data.nullable_field IS NOT NULL)  
SELECT * FROM test_data WHERE (test_data.price IS NULL OR test_data.completed = true)
```

### Performance Testing Queries

Generate queries for performance benchmarking:

```bash
# Generate queries with different complexity levels
for complexity in 1 2 5 10; do
    beamline query basic \
        --seed 500 \
        --start-auto \
        --script-path large_dataset.ion \
        --sample-count 10 \
        rand-select-all-fw \
            --tbl-flt-rand-min $complexity \
            --tbl-flt-rand-max $complexity \
            --pred-all > "queries_complexity_$complexity.sql"
done
```

## Query Generation Best Practices

### 1. Match Query Complexity to Data Structure

```bash
# Simple flat data - use simple paths
beamline query basic --script-path flat_data.ion --project-path-depth-max 2

# Complex nested data - use deeper paths  
beamline query basic --script-path nested_data.ion --project-path-depth-max 8
```

### 2. Use Appropriate Predicate Sets

```bash
# For numeric data testing
--pred-comparison --pred-between

# For string data testing  
--pred-like --pred-in

# For comprehensive testing
--pred-all
```

### 3. Generate Query Suites

```bash
# Generate different query types for comprehensive testing
beamline query basic --script-path data.ion --sample-count 10 rand-select-all-fw --pred-all > select_star.sql
beamline query basic --script-path data.ion --sample-count 10 rand-sfw --pred-all > projections.sql  
beamline query basic --script-path data.ion --sample-count 10 rand-select-all-efw --pred-all > excludes.sql
```

### 4. Validate Generated Queries

Test generated queries against your data:

```bash
# Generate data and queries with same script
beamline gen data --seed 1 --start-auto --script-path test.ion --sample-count 100 > data.ion
beamline query basic --seed 2 --start-auto --script-path test.ion --sample-count 5 rand-select-all-fw --pred-all > queries.sql

# Validate queries parse correctly
# your-partiql-parser --validate queries.sql
```

## Use Cases

### PartiQL Implementation Testing

Generate comprehensive query test suites:

```bash
# Generate queries covering all PartiQL features
beamline query basic \
    --seed 600 \
    --start-auto \
    --script-path comprehensive_schema.ion \
    --sample-count 100 \
    rand-sefw \
        --project-rand-min 1 \
        --project-rand-max 10 \
        --project-path-depth-max 5 \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 5 \
        --exclude-rand-min 1 \
        --exclude-rand-max 3 \
        --pred-all
```

### Performance Benchmarking

Create query workloads for performance testing:

```bash
# Generate queries with increasing complexity
beamline query basic \
    --seed 700 \
    --start-auto \
    --script-path performance_schema.ion \
    --sample-count 50 \
    rand-sfw \
        --project-rand-min 1 \
        --project-rand-max 20 \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 10 \
        --pred-all > performance_queries.sql
```

### Edge Case Testing

Generate queries that test edge cases:

```bash
# Focus on complex path expressions
beamline query basic \
    --seed 800 \
    --start-auto \
    --script-path edge_case_data.ion \
    --sample-count 25 \
    rand-sefw \
        --project-path-depth-min 3 \
        --project-path-depth-max 8 \
        --project-pathstep-internal-foreach \
        --project-pathstep-final-unpivot \
        --tbl-flt-path-depth-max 6 \
        --exclude-path-depth-min 2 \
        --exclude-path-depth-max 4 \
        --pred-all
```

## Next Steps

Now that you understand query generation fundamentals, explore specific aspects:

- [Basic Queries](./basic-queries.md) - Simple query patterns and common use cases
- [Advanced Patterns](./advanced-patterns.md) - Complex nested queries and deep paths
- [Parameterization](./parameterization.md) - Complete guide to all configuration options
- [CLI Query Commands](../cli/query-commands.md) - Detailed CLI reference
