# DDL-Based Data Generation

Beamline can generate synthetic data directly from SQL-like DDL (Data Definition Language) column definitions, without requiring you to write an Ion script manually. This is useful when you already have a schema definition and want to quickly generate test data that matches it.

## Overview

Instead of writing a Beamline Ion script with explicit random processes, you provide column definitions in DDL format. Beamline automatically maps each DDL type to an appropriate random generator.

```bash
beamline gen data \
  --seed 42 \
  --start-auto \
  --ddl '"sensor_id" VARCHAR, "temperature" DOUBLE, "is_active" BOOL' \
  --dataset-name sensors \
  --sample-count 10
```

## Input Format

The DDL input is a comma-separated list of column definitions:

```sql
"column_name" TYPE [NOT NULL] [OPTIONAL],
"another_column" TYPE
```

Column names can be quoted (`"name"`) or unquoted (`name`). Quoting is required for names with special characters.

## Supported Types

| DDL Type | Generator | Description |
|----------|-----------|-------------|
| `VARCHAR` / `STRING` | UUID | Generates UUID strings |
| `TINYINT` / `INT8` | UniformI8 | Random 8-bit integers |
| `SMALLINT` / `INT16` | UniformI16 | Random 16-bit integers |
| `INT` / `INT32` / `INTEGER` | UniformI32 | Random 32-bit integers |
| `BIGINT` / `INT64` | UniformI64 | Random 64-bit integers |
| `DOUBLE` / `FLOAT` / `FLOAT64` | UniformF64 | Random 64-bit floats |
| `DECIMAL(p,s)` | UniformDecimal | Random decimals within precision/scale bounds |
| `DECIMAL` | UniformDecimal | Unconstrained random decimal |
| `BOOL` / `BOOLEAN` | Bool | Random booleans |
| `TIMESTAMP` / `DATETIME` | Instant | Simulation-time timestamps |
| `STRUCT<...>` | Nested struct | Recursive field generation |
| `ARRAY<T>` | UniformArray | Arrays of 1-5 elements of type T |
| `UNION<T1, T2, ...>` | UniformAnyOf | Random selection from listed types |

## CLI Usage

### Inline DDL

```bash
beamline gen data \
  --seed 100 \
  --start-auto \
  --ddl '"id" VARCHAR, "price" DECIMAL(10,2), "created_at" TIMESTAMP' \
  --dataset-name orders \
  --sample-count 50 \
  --output-format ion-pretty
```

### DDL from a File

```bash
# schema.ddl
# "order_id" VARCHAR,
# "customer_id" VARCHAR,
# "amount" DECIMAL(8,2),
# "status" BOOL,
# "order_time" TIMESTAMP

beamline gen data \
  --seed 100 \
  --start-auto \
  --ddl-path schema.ddl \
  --dataset-name orders \
  --sample-count 1000 \
  --output-format parquet \
  --output-path ./output
```

### Combining with Parquet Output

DDL-based generation pairs well with Parquet output for analytics use cases:

```bash
beamline gen data \
  --seed 42 \
  --start-auto \
  --ddl '"user_id" VARCHAR, "event_type" VARCHAR, "timestamp" TIMESTAMP, "value" DOUBLE' \
  --dataset-name events \
  --sample-count 100000 \
  --output-format parquet \
  --output-path ./data-lake/events
```

## Nested Types

### Structs

```sql
"metadata" STRUCT<"key": VARCHAR, "value": VARCHAR, "count": INT>
```

### Arrays

```sql
"tags" ARRAY<VARCHAR>,
"scores" ARRAY<DOUBLE>
```

### Unions

```sql
"payload" UNION<VARCHAR, INT, DOUBLE>
```

## Programmatic API

The DDL data generation feature is also available as a Rust library API:

```rust
use partiql_beamline::ddl_generator::DdlDataGenerator;

let mut generator = DdlDataGenerator::builder()
    .ddl(r#""sensor_id" VARCHAR, "temperature" DOUBLE, "active" BOOL"#)
    .dataset_name("sensors")
    .seed(42)
    .build()
    .expect("build generator");

for sample in generator.take(10) {
    let sample = sample.expect("sample");
    println!("tick={:?}, value={:?}", sample.tick, sample.value);
}
```

### Builder Options

| Method | Description |
|--------|-------------|
| `.ddl(str)` | DDL column definitions (required) |
| `.dataset_name(str)` | Name for the generated dataset (default: `"data"`) |
| `.seed(u64)` | Random seed for reproducibility |
| `.t0(OffsetDateTime)` | Start time for the simulation |
| `.nullability(f64)` | Default NULL percentage (0.0 to 1.0) |
| `.optionality(f64)` | Default MISSING percentage (0.0 to 1.0) |

## How It Works

Under the hood, DDL-based generation:

1. Parses the DDL column definitions
2. Converts each column type to a corresponding Beamline generator expression
3. Wraps the generators in a `rand_process` with a Poisson arrival model
4. Produces an Ion script string that feeds into the standard simulation engine

This means DDL-generated data has the same reproducibility guarantees as script-based generation - same seed and start time always produce the same output.
