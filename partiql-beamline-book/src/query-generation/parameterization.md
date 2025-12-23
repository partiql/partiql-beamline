# Query Generation Parameterization

This section provides a complete reference for all query generation parameters. Beamline's query generator is highly configurable, allowing you to control every aspect of query generation from simple predicates to complex nested path expressions.

## Parameter Categories

Query generation parameters are organized into several categories:

1. **Table Filter Parameters** - Control WHERE clause generation
2. **Projection Parameters** - Control SELECT field selection
3. **Exclusion Parameters** - Control EXCLUDE clause generation  
4. **Path Parameters** - Control how paths navigate data structures
5. **Predicate Parameters** - Control predicate types and operators
6. **Type Parameters** - Control what data types can be referenced

## Table Filter Parameters

Control WHERE clause generation across all query strategies:

### Filter Count

| Parameter | Description | Valid Values |
|-----------|-------------|--------------|
| `--tbl-flt-rand-min` | Minimum number of predicates | 1-255 |
| `--tbl-flt-rand-max` | Maximum number of predicates | 1-255 |

**Example:**
```bash
# Generate 1-3 predicates per WHERE clause
--tbl-flt-rand-min 1 --tbl-flt-rand-max 3
```

### Filter Path Configuration

| Parameter | Description |
|-----------|-------------|
| `--tbl-flt-path-depth-min` | Minimum path depth (default: unbounded) |
| `--tbl-flt-path-depth-max` | Maximum path depth (default: unbounded) |

**Example:**
```bash
# Allow paths up to 3 levels deep
--tbl-flt-path-depth-max 3
# Results: field, object.field, object.nested.field
```

### Table Filter Path Steps

Control what types of path steps can appear in WHERE clauses:

#### Internal Path Steps

| Parameter | Description | Example Path |
|-----------|-------------|--------------|
| `--tbl-flt-pathstep-internal-all` | Enable all internal path step types | All below |
| `--tbl-flt-pathstep-internal-project` | Enable projection internal steps | `field.subfield` |
| `--tbl-flt-pathstep-internal-index` | Enable index internal steps | `array[1].field` |
| `--tbl-flt-pathstep-internal-foreach` | Enable wildcard internal steps | `array[*].field` |
| `--tbl-flt-pathstep-internal-unpivot` | Enable unpivot internal steps | `object.*.field` |

#### Final Path Steps

| Parameter | Description | Example Path |
|-----------|-------------|--------------|
| `--tbl-flt-pathstep-final-all` | Enable all final path step types | All below |
| `--tbl-flt-pathstep-final-project` | Enable projection final steps | `object.field` |
| `--tbl-flt-pathstep-final-index` | Enable index final steps | `array[1]` |
| `--tbl-flt-pathstep-final-foreach` | Enable wildcard final steps | `array[*]` |
| `--tbl-flt-pathstep-final-unpivot` | Enable unpivot final steps | `object.*` |

### Table Filter Type Constraints

Control what types of values can appear in WHERE clauses:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `--tbl-flt-type-final-all` | Allow all final types | Any type |
| `--tbl-flt-type-final-scalar` | Allow only scalar types | `9`, `'text'`, `true` |
| `--tbl-flt-type-final-sequence` | Allow only sequence types | `[1,2,3]`, `<1, 'a'>` |
| `--tbl-flt-type-final-struct` | Allow only struct types | `{'a': 1, 'b': 2}` |

## Projection Parameters

Control SELECT clause generation (applies to `rand-sfw` and `rand-sefw` strategies):

### Projection Count

| Parameter | Description | Valid Values |
|-----------|-------------|--------------|
| `--project-rand-min` | Minimum number of projections | 1-255 |
| `--project-rand-max` | Maximum number of projections | 1-255 |

**Example:**
```bash
# Generate 2-5 fields in SELECT clause
--project-rand-min 2 --project-rand-max 5
```

### Projection Path Configuration

| Parameter | Description |
|-----------|-------------|
| `--project-path-depth-min` | Minimum projection path depth |
| `--project-path-depth-max` | Maximum projection path depth |

### Projection Path Steps

Same options as table filter path steps, but for SELECT clause:

#### Internal Path Steps

| Parameter | Description |
|-----------|-------------|
| `--project-pathstep-internal-all` | Enable all internal path step types |
| `--project-pathstep-internal-project` | Enable projection internal steps |
| `--project-pathstep-internal-index` | Enable index internal steps |
| `--project-pathstep-internal-foreach` | Enable wildcard internal steps |
| `--project-pathstep-internal-unpivot` | Enable unpivot internal steps |

#### Final Path Steps

| Parameter | Description |
|-----------|-------------|
| `--project-pathstep-final-all` | Enable all final path step types |
| `--project-pathstep-final-project` | Enable projection final steps |
| `--project-pathstep-final-index` | Enable index final steps |
| `--project-pathstep-final-foreach` | Enable wildcard final steps |
| `--project-pathstep-final-unpivot` | Enable unpivot final steps |

### Projection Type Constraints

| Parameter | Description |
|-----------|-------------|
| `--project-type-final-all` | Allow all final types |
| `--project-type-final-scalar` | Allow only scalar types |
| `--project-type-final-sequence` | Allow only sequence types |
| `--project-type-final-struct` | Allow only struct types |

## Exclusion Parameters

Control EXCLUDE clause generation (applies to `rand-select-all-efw` and `rand-sefw` strategies):

### Exclusion Count

| Parameter | Description | Valid Values |
|-----------|-------------|--------------|
| `--exclude-rand-min` | Minimum number of exclusions | 1-255 |
| `--exclude-rand-max` | Maximum number of exclusions | 1-255 |

### Exclusion Path Configuration

| Parameter | Description |
|-----------|-------------|
| `--exclude-path-depth-min` | Minimum exclusion path depth |
| `--exclude-path-depth-max` | Maximum exclusion path depth |

### Exclusion Path Steps

Same structure as projection parameters:

| Parameter | Description |
|-----------|-------------|
| `--exclude-pathstep-internal-all` | Enable all internal path steps |
| `--exclude-pathstep-internal-project` | Enable projection internal steps |
| `--exclude-pathstep-internal-index` | Enable index internal steps |
| `--exclude-pathstep-internal-foreach` | Enable wildcard internal steps |
| `--exclude-pathstep-internal-unpivot` | Enable unpivot internal steps |
| `--exclude-pathstep-final-all` | Enable all final path steps |
| `--exclude-pathstep-final-project` | Enable projection final steps |
| `--exclude-pathstep-final-index` | Enable index final steps |
| `--exclude-pathstep-final-foreach` | Enable wildcard final steps |
| `--exclude-pathstep-final-unpivot` | Enable unpivot final steps |

### Exclusion Type Constraints

| Parameter | Description |
|-----------|-------------|
| `--exclude-type-final-all` | Allow all final types |
| `--exclude-type-final-scalar` | Allow only scalar types |
| `--exclude-type-final-sequence` | Allow only sequence types |
| `--exclude-type-final-struct` | Allow only struct types |

## Predicate Parameters

Control what types of predicates can be generated in WHERE clauses:

### All Predicate Types

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-all` | Enable all predicate types | All below |
| `--pred-none` | Disable all predicates | None |

### Null and Missing Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-absent` | Enable null/missing predicates | `IS NULL`, `IS NOT NULL`, `IS MISSING`, `IS NOT MISSING` |
| `--pred-nullable` | Enable null predicates | `IS NULL`, `IS NOT NULL` |
| `--pred-is-null` | Enable IS NULL | `IS NULL` |
| `--pred-is-not-null` | Enable IS NOT NULL | `IS NOT NULL` |
| `--pred-optional` | Enable missing predicates | `IS MISSING`, `IS NOT MISSING` |
| `--pred-is-missing` | Enable IS MISSING | `IS MISSING` |
| `--pred-is-not-missing` | Enable IS NOT MISSING | `IS NOT MISSING` |

### Equality Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-equality` | Enable equality predicates | `=`, `<>` |
| `--pred-eq` | Enable equals | `=` |
| `--pred-neq` | Enable not equals | `<>` |

### Comparison Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-comparison` | Enable all comparison predicates | `<`, `<=`, `>`, `>=`, `BETWEEN` |
| `--pred-lt` | Enable less than | `<` |
| `--pred-lte` | Enable less than or equal | `<=` |
| `--pred-gt` | Enable greater than | `>` |
| `--pred-gte` | Enable greater than or equal | `>=` |
| `--pred-between` | Enable between | `BETWEEN` |

### Numeric Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-numeric` | Enable all numeric predicates | `=`, `<>`, `<`, `<=`, `>`, `>=`, `BETWEEN` |

### String Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-like-all` | Enable all LIKE predicates | `LIKE`, `NOT LIKE` |
| `--pred-like` | Enable LIKE | `LIKE` |
| `--pred-not-like` | Enable NOT LIKE | `NOT LIKE` |

### Set Membership Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-in-all` | Enable all IN predicates | `IN`, `NOT IN` |
| `--pred-in` | Enable IN | `IN` |
| `--pred-not-in` | Enable NOT IN | `NOT IN` |

### Logical Predicates

| Parameter | Description | SQL Operators |
|-----------|-------------|---------------|
| `--pred-logical-all` | Enable all logical operators | `AND`, `OR`, `NOT` |
| `--pred-logical-and` | Enable AND | `AND` |
| `--pred-logical-or` | Enable OR | `OR` |
| `--pred-logical-not` | Enable NOT | `NOT` |

## Strategy Compatibility Matrix

Different parameters apply to different query strategies:

| Parameter Category | `rand-select-all-fw` | `rand-sfw` | `rand-select-all-efw` | `rand-sefw` |
|-------------------|:--------------------:|:----------:|:---------------------:|:-----------:|
| **Table Filter** | ✅ | ✅ | ✅ | ✅ |
| **Projection** | ❌ | ✅ | ❌ | ✅ |  
| **Exclusion** | ❌ | ❌ | ✅ | ✅ |
| **Predicates** | ✅ | ✅ | ✅ | ✅ |

## Configuration Examples

### Simple Configuration

For basic testing with readable queries:

```bash
beamline query basic \
    --seed 100 \
    --start-auto \
    --script-path data.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --tbl-flt-path-depth-max 2 \
        --tbl-flt-pathstep-internal-project \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-comparison
```

### Moderate Configuration  

For comprehensive testing with controlled complexity:

```bash
beamline query basic \
    --seed 200 \
    --start-auto \
    --script-path data.ion \
    --sample-count 15 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --project-path-depth-max 3 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --tbl-flt-path-depth-max 2 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all
```

### Complex Configuration

For edge case testing with maximum complexity:

```bash
beamline query basic \
    --seed 300 \
    --start-auto \
    --script-path nested_data.ion \
    --sample-count 20 \
    rand-sefw \
        --project-rand-min 3 \
        --project-rand-max 8 \
        --project-path-depth-min 2 \
        --project-path-depth-max 6 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-all \
        --tbl-flt-rand-min 2 \
        --tbl-flt-rand-max 5 \
        --tbl-flt-path-depth-max 6 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-all \
        --tbl-flt-type-final-all \
        --exclude-rand-min 1 \
        --exclude-rand-max 4 \
        --exclude-path-depth-min 2 \
        --exclude-path-depth-max 4 \
        --exclude-pathstep-internal-all \
        --exclude-pathstep-final-all \
        --exclude-type-final-all \
        --pred-all
```

## Parameter Combination Patterns

### Testing Specific PartiQL Features

#### Test Wildcard Paths

```bash
# Focus on [*] and .* path expressions
beamline query basic \
    --seed 400 \
    --start-auto \
    --script-path array_data.ion \
    --sample-count 10 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --project-pathstep-internal-foreach \
        --project-pathstep-internal-unpivot \
        --project-pathstep-final-foreach \
        --project-pathstep-final-unpivot \
        --pred-all
```

#### Test Deep Nesting

```bash
# Generate very deep path expressions
beamline query basic \
    --seed 500 \
    --start-auto \
    --script-path deeply_nested.ion \
    --sample-count 8 \
    rand-sfw \
        --project-path-depth-min 4 \
        --project-path-depth-max 8 \
        --tbl-flt-path-depth-min 3 \
        --tbl-flt-path-depth-max 6 \
        --exclude-path-depth-min 2 \
        --exclude-path-depth-max 5 \
        --pred-all
```

#### Test Null Handling

```bash  
# Focus on null and missing value predicates
beamline query basic \
    --seed 600 \
    --start-auto \
    --script-path nullable_schema.ion \
    --sample-count 12 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --pred-absent --pred-logical-and --pred-logical-or
```

### Performance Testing Configurations

#### Lightweight Queries

```bash
# Generate simple, fast-executing queries
beamline query basic \
    --seed 700 \
    --start-auto \
    --script-path performance_data.ion \
    --sample-count 25 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --tbl-flt-path-depth-max 1 \
        --tbl-flt-type-final-scalar \
        --pred-eq --pred-comparison
```

#### Heavy Queries

```bash
# Generate complex, resource-intensive queries
beamline query basic \
    --seed 800 \
    --start-auto \
    --script-path large_data.ion \
    --sample-count 15 \
    rand-sefw \
        --project-rand-min 8 \
        --project-rand-max 15 \
        --project-path-depth-max 6 \
        --tbl-flt-rand-min 5 \
        --tbl-flt-rand-max 10 \
        --tbl-flt-path-depth-max 5 \
        --exclude-rand-min 3 \
        --exclude-rand-max 8 \
        --pred-all
```

## Common Parameter Patterns

### Development and Debugging

```bash
# Simple, readable queries for development
--project-rand-min 1 --project-rand-max 3
--project-path-depth-max 2
--project-pathstep-final-project
--tbl-flt-rand-min 1 --tbl-flt-rand-max 2
--tbl-flt-path-depth-max 2
--pred-eq --pred-comparison
```

### Integration Testing

```bash
# Moderate complexity for integration tests
--project-rand-min 2 --project-rand-max 5
--project-path-depth-max 3
--project-pathstep-internal-all --project-pathstep-final-all
--tbl-flt-rand-min 1 --tbl-flt-rand-max 4
--tbl-flt-path-depth-max 3
--exclude-rand-min 1 --exclude-rand-max 2
--pred-all
```

### Stress Testing

```bash
# Maximum complexity for stress testing
--project-rand-min 5 --project-rand-max 12
--project-path-depth-min 3 --project-path-depth-max 8
--project-pathstep-internal-all --project-pathstep-final-all
--project-type-final-all
--tbl-flt-rand-min 3 --tbl-flt-rand-max 8
--tbl-flt-path-depth-max 6
--exclude-rand-min 2 --exclude-rand-max 6
--exclude-path-depth-min 2 --exclude-path-depth-max 5
--pred-all
```

## Parameter Validation

### Check Parameter Combinations

Some parameter combinations don't make sense:

```bash
# Invalid - min > max
--tbl-flt-rand-min 5 --tbl-flt-rand-max 3  # Error

# Invalid - conflicting path steps  
--project-pathstep-final-project --project-pathstep-final-foreach  # May conflict

# Valid - consistent configuration
--tbl-flt-rand-min 1 --tbl-flt-rand-max 5
--project-rand-min 2 --project-rand-max 4
```

### Default Behaviors

When parameters are not specified:

- **Path depth**: Unbounded (can generate very deep paths)
- **Path steps**: All types enabled by default
- **Type constraints**: All types allowed
- **Predicate count**: Implementation-dependent defaults

**Recommendation**: Always specify explicit bounds for predictable results.

## Advanced Configuration Techniques

### Targeted Testing

#### Test Specific Path Types

```bash
# Test only wildcard expressions
beamline query basic \
    --script-path array_data.ion \
    --sample-count 10 \
    rand-sfw \
        --project-pathstep-final-foreach \
        --project-pathstep-final-unpivot \
        --tbl-flt-pathstep-final-foreach \
        --pred-all
```

#### Test Specific Predicates

```bash
# Test only string operations
beamline query basic \
    --script-path text_data.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --pred-like --pred-in --pred-eq
```

### Graduated Complexity Testing

```bash
#!/bin/bash
# Generate test suites with graduated complexity

SCRIPT="test_schema.ion"
BASE_SEED=1000

# Level 1: Simple queries
beamline query basic \
    --seed $BASE_SEED \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 --tbl-flt-rand-max 1 \
        --pred-eq > level1.sql

# Level 2: Add comparisons  
beamline query basic \
    --seed $((BASE_SEED + 1)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 --tbl-flt-rand-max 2 \
        --pred-comparison > level2.sql

# Level 3: Add projections
beamline query basic \
    --seed $((BASE_SEED + 2)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-sfw \
        --project-rand-min 2 --project-rand-max 4 \
        --tbl-flt-rand-min 1 --tbl-flt-rand-max 2 \
        --pred-all > level3.sql

# Level 4: Add exclusions
beamline query basic \
    --seed $((BASE_SEED + 3)) \
    --start-auto \
    --script-path $SCRIPT \
    --sample-count 10 \
    rand-sefw \
        --project-rand-min 2 --project-rand-max 3 \
        --tbl-flt-rand-min 1 --tbl-flt-rand-max 3 \
        --exclude-rand-min 1 --exclude-rand-max 2 \
        --pred-all > level4.sql

echo "Generated graduated complexity test suite"
```

## Performance Impact of Parameters

### High Performance Impact

These parameters significantly affect query generation performance:

- **High path depth** (`--path-depth-max 10+`): Exponential complexity growth
- **Many projections** (`--project-rand-max 20+`): Large SELECT clauses
- **Many predicates** (`--tbl-flt-rand-max 10+`): Complex WHERE clauses
- **All path steps enabled**: More path generation options to evaluate

### Low Performance Impact

These parameters have minimal impact:

- **Predicate type selection**: Doesn't affect generation complexity
- **Type constraints**: Reduces rather than increases complexity
- **Path step restrictions**: Reduces generation options

### Performance Optimization

```bash
# Optimized for speed
beamline query basic \
    --script-path data.ion \
    --sample-count 100 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 --tbl-flt-rand-max 3 \
        --tbl-flt-path-depth-max 2 \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-comparison

# Comprehensive but slower
beamline query basic \
    --script-path data.ion \
    --sample-count 25 \
    rand-sefw \
        --project-rand-min 5 --project-rand-max 10 \
        --project-path-depth-max 5 \
        --tbl-flt-rand-min 3 --tbl-flt-rand-max 6 \
        --exclude-rand-min 2 --exclude-rand-max 4 \
        --pred-all
```

## Troubleshooting Parameter Issues

### Common Parameter Errors

#### Invalid Range Parameters

```bash
# Error: min > max
--tbl-flt-rand-min 5 --tbl-flt-rand-max 3

# Fix: min <= max
--tbl-flt-rand-min 3 --tbl-flt-rand-max 5
```

#### Conflicting Type Constraints

```bash
# May produce unexpected results
--project-type-final-scalar --project-pathstep-final-unpivot  # Scalar constraint conflicts with unpivot

# Better: consistent constraints
--project-type-final-all --project-pathstep-final-unpivot
```

### Parameter Testing

```bash
# Test parameter combinations before large generation
beamline query basic \
    --seed 1 \
    --start-auto \
    --script-path test.ion \
    --sample-count 3 \
    rand-sefw \
        --project-rand-min 2 --project-rand-max 2 \
        --exclude-rand-min 1 --exclude-rand-max 1 \
        --pred-eq

# If results look good, scale up
# ... run with --sample-count 50
```

## Best Practices

### 1. Start Conservative

```bash
# Begin with simple parameter values
--tbl-flt-rand-min 1 --tbl-flt-rand-max 2
--project-rand-min 1 --project-rand-max 3
--exclude-rand-min 1 --exclude-rand-max 1
```

### 2. Match Parameters to Data Structure

```bash
# Simple flat data
--project-path-depth-max 1 --tbl-flt-path-depth-max 1

# Nested object data  
--project-path-depth-max 3 --tbl-flt-path-depth-max 3

# Deeply nested data
--project-path-depth-max 6 --tbl-flt-path-depth-max 6
```

### 3. Use Consistent Parameter Ranges

```bash
# Good - balanced complexity
--project-rand-min 2 --project-rand-max 4
--tbl-flt-rand-min 1 --tbl-flt-rand-max 3
--exclude-rand-min 1 --exclude-rand-max 2

# Avoid - unbalanced (too many exclusions vs projections)
--project-rand-min 1 --project-rand-max 2
--exclude-rand-min 5 --exclude-rand-max 10
```

### 4. Document Your Configurations

```bash
# Create reusable parameter sets
SIMPLE_CONFIG="--tbl-flt-rand-min 1 --tbl-flt-rand-max 2 --pred-comparison"
MODERATE_CONFIG="--project-rand-min 2 --project-rand-max 4 --tbl-flt-rand-min 1 --tbl-flt-rand-max 3 --pred-all"

beamline query basic --script-path data.ion --sample-count 10 rand-select-all-fw $SIMPLE_CONFIG
beamline query basic --script-path data.ion --sample-count 10 rand-sfw $MODERATE_CONFIG
```

## Reference Quick Guide

### Most Common Configurations

**Basic Testing:**
```bash
rand-select-all-fw --tbl-flt-rand-min 1 --tbl-flt-rand-max 2 --pred-comparison
```

**Projection Testing:**
```bash
rand-sfw --project-rand-min 2 --project-rand-max 4 --tbl-flt-rand-min 1 --tbl-flt-rand-max 2 --pred-all
```

**Exclusion Testing:**
```bash
rand-select-all-efw --tbl-flt-rand-min 1 --tbl-flt-rand-max 2 --exclude-rand-min 1 --exclude-rand-max 3 --pred-all
```

**Comprehensive Testing:**
```bash
rand-sefw --project-rand-min 2 --project-rand-max 4 --exclude-rand-min 1 --exclude-rand-max 2 --tbl-flt-rand-min 1 --tbl-flt-rand-max 3 --pred-all
```

## Next Steps

Now that you understand all parameterization options:

- [CLI Query Commands](../cli/query-commands.md) - Complete CLI reference with parameter examples
- [Basic Queries](./basic-queries.md) - Apply parameters to simple query generation
- [Advanced Patterns](./advanced-patterns.md) - Use parameters for complex query patterns
- [Examples](../examples/sensors.md) - See parameterization in real-world scenarios
