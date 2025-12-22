# Query Commands

The `beamline query` command generates PartiQL queries that match the shapes and types of data defined in Ion scripts. This allows you to create realistic queries for testing PartiQL implementations.

## Command Syntax

```bash
beamline query basic [OPTIONS] <STRATEGY>
```

## Required Options

Query generation requires the same core configuration as data generation:

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

### Sample Count
```bash
--sample-count <COUNT>         # Number of queries to generate (default: 10)
```

## Query Strategies

PartiQL Beamline supports four different query generation strategies:

### 1. `rand-select-all-fw` - SELECT * with WHERE

Generates `SELECT *` queries with randomly generated WHERE clauses.

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

Generates queries with random projections and WHERE clauses.

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

### 3. `rand-select-all-efw` - SELECT * EXCLUDE WHERE

Generates `SELECT * EXCLUDE` queries with WHERE clauses.

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

### 4. `rand-sefw` - SELECT EXCLUDE FROM WHERE

Generates queries with projections, exclusions, and WHERE clauses.

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

## Parameter Reference

### Table Filter Parameters

Control WHERE clause generation:

```bash
--tbl-flt-rand-min <N>              # Minimum number of predicates (1-255)
--tbl-flt-rand-max <N>              # Maximum number of predicates (1-255)
--tbl-flt-path-depth-max <N>        # Maximum path depth (1-255)

# Path step types (internal positions)
--tbl-flt-pathstep-internal-all     # Enable all internal path step types
--tbl-flt-pathstep-internal-project # Enable projection steps (.field)
--tbl-flt-pathstep-internal-index   # Enable index steps ([1])
--tbl-flt-pathstep-internal-foreach # Enable for-each steps ([*])
--tbl-flt-pathstep-internal-unpivot # Enable unpivot steps (.*)

# Path step types (final positions)
--tbl-flt-pathstep-final-all        # Enable all final path step types  
--tbl-flt-pathstep-final-project    # Enable projection steps (.field)
--tbl-flt-pathstep-final-index      # Enable index steps ([1])
--tbl-flt-pathstep-final-foreach    # Enable for-each steps ([*])
--tbl-flt-pathstep-final-unpivot    # Enable unpivot steps (.*)

# Type constraints
--tbl-flt-type-final-all            # Allow all final types
--tbl-flt-type-final-scalar         # Allow scalar final types only
--tbl-flt-type-final-sequence       # Allow sequence final types
--tbl-flt-type-final-struct         # Allow struct final types
```

### Predicate Types

Control which predicates can be generated:

```bash
--pred-all                 # Enable all predicates
--pred-lt                  # Less than (<)
--pred-lte                 # Less than or equal (<=)
--pred-gt                  # Greater than (>)
--pred-gte                 # Greater than or equal (>=)
--pred-eq                  # Equal (=)
--pred-neq                 # Not equal (<>)
--pred-between             # BETWEEN predicate
--pred-like                # LIKE predicate
--pred-not-like            # NOT LIKE predicate
--pred-in                  # IN predicate
--pred-not-in              # NOT IN predicate
--pred-is-null             # IS NULL
--pred-is-not-null         # IS NOT NULL
--pred-is-missing          # IS MISSING
--pred-is-not-missing      # IS NOT MISSING
--pred-logical-and         # AND operator
--pred-logical-or          # OR operator
--pred-logical-not         # NOT operator
```

### Projection Parameters (for `rand-sfw` and `rand-sefw`)

Control SELECT clause generation:

```bash
--project-rand-min <N>              # Minimum projections (1-255)
--project-rand-max <N>              # Maximum projections (1-255)
--project-path-depth-min <N>        # Minimum path depth
--project-path-depth-max <N>        # Maximum path depth

# Same path step and type options as table filters
--project-pathstep-internal-all     # Enable all internal path steps
--project-pathstep-final-all        # Enable all final path steps
--project-type-final-all            # Allow all final types
```

### Exclusion Parameters (for `rand-select-all-efw` and `rand-sefw`)

Control EXCLUDE clause generation:

```bash
--exclude-rand-min <N>              # Minimum exclusions (1-255)
--exclude-rand-max <N>              # Maximum exclusions (1-255)
--exclude-path-depth-min <N>        # Minimum path depth
--exclude-path-depth-max <N>        # Maximum path depth

# Same path step and type options as table filters
--exclude-pathstep-internal-all     # Enable all internal path steps
--exclude-pathstep-final-all        # Enable all final path steps
--exclude-type-final-all            # Allow all final types
```

## Complex Examples

### Deep Path Generation

For nested data structures, control path depth:

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

This generates queries with deeply nested paths like:
```sql
SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct
EXCLUDE test_data.*.*.*.*, test_data.price.* 
FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))
```

### Reproducible Query Generation

Use specific seeds for consistent query generation:

```bash
# Generate same queries each time
beamline query basic \
  --seed 12345 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path data.ion \
  --sample-count 5 \
  rand-select-all-fw \
    --tbl-flt-rand-min 1 \
    --tbl-flt-rand-max 3 \
    --pred-all
```

## Best Practices

### 1. Start with Simple Queries

```bash
# Begin with basic queries
beamline query basic \
  --seed 1 \
  --start-auto \
  --script-path data.ion \
  --sample-count 5 \
  rand-select-all-fw \
    --tbl-flt-rand-min 1 \
    --tbl-flt-rand-max 1 \
    --pred-eq
```

### 2. Match Query Complexity to Data Structure

```bash
# Simple data = simple paths
--project-path-depth-max 2

# Complex nested data = deeper paths  
--project-path-depth-max 6
```

### 3. Use Appropriate Predicates for Testing

```bash
# For numeric testing
--pred-lt --pred-gt --pred-between

# For comprehensive testing
--pred-all
```

### 4. Validate Generated Queries

Test generated queries against your data to ensure they're valid and meaningful.

## Integration with Data Generation

Combine query and data generation for complete testing:

```bash
# Generate test data
beamline gen data \
  --seed 100 \
  --start-auto \
  --script-path test_data.ion \
  --sample-count 1000 \
  --output-format ion-pretty > test_data.ion

# Generate matching queries  
beamline query basic \
  --seed 101 \
  --start-auto \
  --script-path test_data.ion \
  --sample-count 20 \
  rand-select-all-fw \
    --tbl-flt-rand-min 1 \
    --tbl-flt-rand-max 3 \
    --pred-all > test_queries.sql
```

## Next Steps

- [Shape Commands](./shape-commands.md) - Infer schemas from your data
- [Database Commands](./database-commands.md) - Create complete test databases
- [Query Generation Guide](../query-generation/overview.md) - Learn about query generation theory
