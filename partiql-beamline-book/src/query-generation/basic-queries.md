# Basic Query Generation

This section covers fundamental query generation patterns using PartiQL Beamline's `rand-select-all-fw` strategy, which generates simple `SELECT *` queries with WHERE clauses. This is the best starting point for understanding how query generation works.

## Getting Started with Basic Queries

### Simple Transaction Data

Let's use the `simple_transactions.ion` script from the test suite as our data source:

```ion
rand_processes::{
    test_data: rand_process::{
        $r: Uniform::{ choices: [5,10] },
        $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

        $data: {
            transaction_id: UUID::{ nullable: false },
            marketplace_id: UniformU8::{ nullable: false },
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            created_at: Instant,
            completed: Bool,
            description: LoremIpsum::{ min_words:10, max_words:200 },
            price: UniformDecimal::{ low: 2.99, high: 99999.99, optional: true }
        }
    }
}
```

This script creates transaction data with various field types that the query generator can reference.

### Basic Query Generation Command

```bash
partiql-beamline-cli query basic \
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

**Generated Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)

SELECT * FROM test_data AS test_data WHERE (test_data.price < 18.418581624952935)

SELECT * FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```

### Understanding the Parameters

- `--tbl-flt-rand-min 1 --tbl-flt-rand-max 1`: Generate exactly 1 predicate per query
- `--tbl-flt-path-depth-max 1`: Use only top-level fields (no nested paths)
- `--tbl-flt-pathstep-final-project`: Final path step is field projection (`.field`)
- `--tbl-flt-type-final-scalar`: Only reference scalar values (numbers, strings, booleans)
- `--pred-lt`: Use only less-than (`<`) predicates

## Different Predicate Types

### Comparison Predicates

```bash
# Less than predicates
partiql-beamline-cli query basic \
    --seed 100 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-lt
```

**Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.price < 123.45)
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < 42)
```

### Equality Predicates

```bash
# Equality predicates
partiql-beamline-cli query basic \
    --seed 200 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-eq
```

**Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.completed = true)
SELECT * FROM test_data AS test_data WHERE (test_data.country_code = 'US')
```

### All Predicate Types

```bash
# Use all available predicates for comprehensive testing
partiql-beamline-cli query basic \
    --seed 300 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all
```

**Output (from README examples):**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.country_code IN [
      'Graecos quidem legendos.',
      'Possit et sine.'
    ] OR (NOT ((test_data.description IS MISSING)) OR
    (test_data.description IS MISSING)))

SELECT * FROM test_data AS test_data WHERE (((test_data.transaction_id IS NULL)
    AND (test_data.created_at IS NULL)) OR (((test_data.completed IN [
            false,
            false
          ] OR NOT ((test_data.completed IS NULL))) AND
      ((NOT ((test_data.price IS NULL)) OR
          (test_data.transaction_id LIKE 'Vidisse.' AND
            (test_data.country_code IS NULL))) AND
        NOT ((test_data.description IS MISSING)))) OR
    (test_data.description <> 'Nec vero.')))
```

## Multiple Predicates

### Combining Predicates

Increase predicate count to create more complex WHERE clauses:

```bash
# Generate queries with 2-5 predicates
partiql-beamline-cli query basic \
    --seed 400 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --pred-all
```

**Example Output:**
```sql
SELECT * FROM test_data AS test_data
WHERE (((((test_data.country_code <> 'Qua maxime ceterorum.') AND
        (NOT (test_data.completed IN [ false, true ]) OR
          (test_data.description = 'Non faciant.'))) AND
      (NOT ((test_data.price IS MISSING)) AND (test_data.price IS MISSING))) OR
    test_data.price IN [
        -47.936734585045905,
        -0.8509689800217544,
        24.263479438050297
      ]) OR ((test_data.created_at = UTCNOW()) OR
    (NOT ((test_data.country_code IS MISSING)) AND
      (test_data.description IS MISSING))))
```

## Field Types and Query Generation

### Numeric Fields

The query generator creates appropriate predicates for numeric types:

```ion
rand_processes::{
    numeric_test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            count: UniformI32::{ low: 1, high: 1000 },
            price: UniformDecimal::{ low: 9.99, high: 999.99 },
            score: NormalF64::{ mean: 75.0, std_dev: 15.0 }
        }
    }
}
```

**Generated Queries:**
```sql
SELECT * FROM numeric_test WHERE (numeric_test.count > 500)
SELECT * FROM numeric_test WHERE (numeric_test.price BETWEEN 50.0 AND 200.0)
SELECT * FROM numeric_test WHERE (numeric_test.score <= 85.5)
```

### String Fields

String generators produce string-appropriate predicates:

```ion
rand_processes::{
    string_test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            name: LoremIpsumTitle,
            email: Format::{ pattern: "user{UUID}@example.com" },
            country: Regex::{ pattern: "[A-Z]{2}" },
            description: LoremIpsum::{ min_words: 5, max_words: 50 }
        }
    }
}
```

**Generated Queries:**
```sql
SELECT * FROM string_test WHERE (string_test.country = 'US')
SELECT * FROM string_test WHERE (string_test.name LIKE '%Test%')
SELECT * FROM string_test WHERE (string_test.email IN ['user1@example.com', 'user2@example.com'])
```

### Boolean Fields

Boolean fields generate boolean predicates:

```ion
rand_processes::{
    boolean_test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            active: Bool,
            verified: Bool::{ p: 0.8 },
            premium: Bool::{ p: 0.1 }
        }
    }
}
```

**Generated Queries:**
```sql
SELECT * FROM boolean_test WHERE (boolean_test.active = true)
SELECT * FROM boolean_test WHERE (boolean_test.verified AND boolean_test.premium)
SELECT * FROM boolean_test WHERE (NOT boolean_test.active)
```

## Null and Missing Value Queries

### Testing Null Handling

When your data includes nullable fields, queries will test null handling:

```ion
rand_processes::{
    nullable_test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            required_field: UUID::{ nullable: false },
            nullable_field: UniformI32::{ nullable: 0.3, low: 1, high: 100 },
            optional_field: UniformDecimal::{ optional: 0.2, low: 0.0, high: 1000.0 }
        }
    }
}
```

**Generated Queries:**
```sql
SELECT * FROM nullable_test WHERE (nullable_test.nullable_field IS NOT NULL)
SELECT * FROM nullable_test WHERE (nullable_test.optional_field IS MISSING)
SELECT * FROM nullable_test WHERE (nullable_test.required_field IS NOT NULL AND nullable_test.nullable_field > 50)
```

### Focusing on Null/Missing Tests

```bash
# Generate queries focused on null/missing testing
partiql-beamline-cli query basic \
    --seed 500 \
    --start-auto \
    --script-path nullable_data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --pred-absent  # Only IS NULL, IS NOT NULL, IS MISSING, IS NOT MISSING
```

## Progressive Query Complexity

### Start Simple

```bash
# Begin with single predicates
partiql-beamline-cli query basic \
    --seed 1 \
    --start-auto \
    --script-path data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-eq
```

### Add More Predicates

```bash
# Increase to 2-3 predicates
partiql-beamline-cli query basic \
    --seed 1 \
    --start-auto \
    --script-path data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 3 \
        --pred-comparison
```

### Enable All Predicates

```bash
# Use all available predicates for full complexity
partiql-beamline-cli query basic \
    --seed 1 \
    --start-auto \
    --script-path data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 5 \
        --pred-all
```

## Real Examples from README

### Single Predicate Queries

From the README example:

```bash
partiql-beamline-cli query basic \
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

**Actual Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)
SELECT * FROM test_data AS test_data WHERE (test_data.price < 18.418581624952935)
SELECT * FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```

### Multiple Predicate Queries

From the README example with more complex predicates:

```bash
partiql-beamline-cli query basic \
    --seed 1234 \
    --start-auto \
    --script-path simple_transactions.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 3 \
        --tbl-flt-rand-max 10 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-all \
        --pred-all
```

**Actual Output:**
```sql
SELECT * FROM test_data AS test_data WHERE (test_data.country_code IN [
      'Graecos quidem legendos.',
      'Possit et sine.'
    ] OR (NOT ((test_data.description IS MISSING)) OR
    (test_data.description IS MISSING)))

SELECT * FROM test_data AS test_data WHERE (((test_data.transaction_id IS NULL)
    AND (test_data.created_at IS NULL)) OR (((test_data.completed IN [
            false,
            false
          ] OR NOT ((test_data.completed IS NULL))) AND
      ((NOT ((test_data.price IS NULL)) OR
          (test_data.transaction_id LIKE 'Vidisse.' AND
            (test_data.country_code IS NULL))) AND
        NOT ((test_data.description IS MISSING)))) OR
    (test_data.description <> 'Nec vero.')))

SELECT * FROM test_data AS test_data
WHERE (((((test_data.country_code <> 'Qua maxime ceterorum.') AND
        (NOT (test_data.completed IN [ false, true, true ]) OR
          (test_data.description = 'Non faciant.'))) AND
      (NOT ((test_data.price IS MISSING)) AND (test_data.price IS MISSING))) OR
    test_data.price IN [
        -47.936734585045905,
        -0.8509689800217544,
        24.263479438050297,
        -48.953369038690255
      ]) OR ((test_data.created_at = UTCNOW()) OR
    (NOT ((test_data.country_code IS MISSING)) AND
      (test_data.description IS MISSING))))
```

## Specific Predicate Types

### Comparison Predicates

```bash
# Only numeric comparisons
partiql-beamline-cli query basic \
    --seed 100 \
    --start-auto \
    --script-path numeric_data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-comparison  # <, <=, >, >=, =, <>
```

**Example Results:**
```sql
SELECT * FROM test_data WHERE (test_data.price >= 50.0)
SELECT * FROM test_data WHERE (test_data.count <= 500)
SELECT * FROM test_data WHERE (test_data.score <> 75.5)
```

### String Pattern Matching

```bash
# Focus on LIKE predicates
partiql-beamline-cli query basic \
    --seed 200 \
    --start-auto \
    --script-path text_data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-like
```

**Example Results:**
```sql
SELECT * FROM test_data WHERE (test_data.description LIKE '%lorem%')
SELECT * FROM test_data WHERE (test_data.country_code LIKE 'U_')
```

### Set Membership

```bash
# Use IN and NOT IN predicates
partiql-beamline-cli query basic \
    --seed 300 \
    --start-auto \
    --script-path categorical_data.ion \
    --sample-count 5 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-in
```

**Example Results:**
```sql
SELECT * FROM test_data WHERE (test_data.status IN ['active', 'pending'])
SELECT * FROM test_data WHERE (test_data.category IN ['electronics', 'books', 'clothing'])
```

## Reproducible Query Testing

### Test Suite Generation

Create consistent test suites:

```bash
#!/bin/bash
# Generate reproducible query test suite

SCRIPT="test_schema.ion"
BASE_SEED=12345

# Simple queries for basic functionality
partiql-beamline-cli query basic \
    --seed $BASE_SEED \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-eq > basic_equality.sql

# Comparison queries for numeric testing  
partiql-beamline-cli query basic \
    --seed $((BASE_SEED + 1)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-comparison > numeric_comparisons.sql

# Complex queries for comprehensive testing
partiql-beamline-cli query basic \
    --seed $((BASE_SEED + 2)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 15 \
    rand-select-all-fw \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --pred-all > complex_queries.sql

echo "Generated test suite:"
echo "- basic_equality.sql: $(wc -l < basic_equality.sql) queries"
echo "- numeric_comparisons.sql: $(wc -l < numeric_comparisons.sql) queries"  
echo "- complex_queries.sql: $(wc -l < complex_queries.sql) queries"
```

### Regression Testing

```bash
# Generate baseline queries
partiql-beamline-cli query basic \
    --seed 999 \
    --start-auto \
    --script-path stable_schema.ion \
    --sample-count 20 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all > baseline_queries.sql

# Later: regenerate with same seed to verify no regressions
partiql-beamline-cli query basic \
    --seed 999 \
    --start-auto \
    --script-path stable_schema.ion \
    --sample-count 20 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-all > regression_test_queries.sql

# Verify identical output
diff baseline_queries.sql regression_test_queries.sql
```

## Common Query Patterns

### Data Validation Queries

Generate queries that validate data constraints:

```bash
# Focus on range and constraint validation
partiql-beamline-cli query basic \
    --seed 600 \
    --start-auto \
    --script-path validation_schema.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --pred-comparison --pred-between
```

**Example Validation Queries:**
```sql
SELECT * FROM test_data WHERE (test_data.age BETWEEN 0 AND 120)
SELECT * FROM test_data WHERE (test_data.price > 0)
SELECT * FROM test_data WHERE (test_data.email IS NOT NULL)
```

### Performance Baseline Queries

Create simple queries for performance baseline:

```bash
# Simple queries for baseline performance measurement
partiql-beamline-cli query basic \
    --seed 700 \
    --start-auto \
    --script-path performance_data.ion \
    --sample-count 25 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-eq --pred-lt --pred-gt
```

## Integration with Data Generation

### Complete Workflow

```bash
#!/bin/bash
# Complete data + query generation workflow

SCRIPT="customer_transactions.ion"
SEED=12345

echo "Generating test data..."
partiql-beamline-cli gen data \
    --seed $SEED \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 1000 \
    --output-format ion-pretty > test_data.ion

echo "Generating basic queries..."
partiql-beamline-cli query basic \
    --seed $((SEED + 1)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 15 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --pred-all > basic_test_queries.sql

echo "Generated $(wc -l < test_data.ion) data records and $(wc -l < basic_test_queries.sql) test queries"

# Test first few queries (example)
head -5 basic_test_queries.sql | while IFS= read -r query; do
    echo "Query: $query"
    # your-partiql-engine --query "$query" --data test_data.ion
done
```

## Best Practices

### 1. Start with Simple Configurations

```bash
# Begin testing with minimal complexity
partiql-beamline-cli query basic \
    --seed 1 \
    --start-auto \
    --script-path new_schema.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --pred-eq
```

### 2. Match Predicates to Data Types

```bash
# For numeric-heavy data
--pred-comparison --pred-between

# For string-heavy data  
--pred-like --pred-eq --pred-in

# For boolean data
--pred-eq --pred-logical-not
```

### 3. Test Query Coverage

```bash
# Generate enough queries to cover different data patterns
partiql-beamline-cli query basic \
    --seed 100 \
    --start-auto \
    --script-path comprehensive_data.ion \
    --sample-count 50 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 4 \
        --pred-all
```

### 4. Validate Against Real Data

```bash
# Always test generated queries work with generated data
partiql-beamline-cli gen data --seed 1 --start-auto --script-path schema.ion --sample-count 100 > data.ion
partiql-beamline-cli query basic --seed 2 --start-auto --script-path schema.ion --sample-count 5 rand-select-all-fw --pred-all > queries.sql

# Validate each query
# for query in $(cat queries.sql); do
#     your-partiql-engine --validate "$query"
# done
```

## Next Steps

Now that you understand basic query generation:

- [Advanced Patterns](./advanced-patterns.md) - Complex queries with projections and exclusions
- [Parameterization](./parameterization.md) - Complete guide to all query generation options
- [CLI Query Commands](../cli/query-commands.md) - Detailed CLI reference with all parameters
