# Generating Data from DDL

In addition to Ion scripts, Beamline can generate data directly from **DDL column
definitions**. This is convenient when you already have a table schema (for
example, one produced by `infer-shape --output-format basic-ddl`) and want to
generate matching data without hand-writing a script.

## How It Works

When you pass `--ddl` or `--ddl-path`, Beamline parses the column definitions,
maps each DDL type to a `Uniform` distribution generator, and builds an Ion
script internally. Generation then proceeds exactly as it would for a script.

## DDL Format

The input is a comma-separated list of quoted column names followed by a type:

```text
"column_name" TYPE,
"another_column" TYPE
```

## Type Mapping

| DDL Type                                | Generator                            |
|-----------------------------------------|--------------------------------------|
| `VARCHAR` / `STRING` / `CHAR` / `TEXT`  | `UUID`                               |
| `TINYINT` / `INT8`                      | `UniformI8`                          |
| `SMALLINT` / `INT16`                    | `UniformI16`                         |
| `INT` / `INT32` / `INTEGER`             | `UniformI32`                         |
| `INT64` / `BIGINT`                      | `UniformI64`                         |
| `DOUBLE` / `FLOAT` / `FLOAT64` / `FLOAT8` / `REAL` | `UniformF64`              |
| `DECIMAL` / `DECIMAL(p,s)` / `NUMERIC`  | `UniformDecimal`                     |
| `BOOL` / `BOOLEAN`                      | `Bool`                               |
| `TIMESTAMP` / `DATETIME`                | `Instant`                            |
| `STRUCT<field: TYPE, ...>`              | nested struct (types mapped recursively) |
| `ARRAY<T>`                              | `UniformArray` of the element type   |
| `UNION<TYPE1, TYPE2, ...>`              | `UniformAnyOf` over the member types |

Type names are case-insensitive.

## Options

DDL generation reuses all the standard `gen data` options. Two are specific to
this mode:

```bash
--ddl-path <PATH>          # Path to a DDL file (column definitions format)
--ddl <DDL_DATA>           # Inline DDL column definitions
--dataset-name <NAME>      # Dataset name for the generated data (default: "data")
```

Exactly one of `--ddl-path`, `--ddl`, `--script-path`, or `--script` is required.

## Example: Inline DDL

```bash
$ beamline gen data \
    --seed 42 \
    --start-iso "2024-01-01T00:00:00Z" \
    --ddl '"id" VARCHAR, "age" INT, "score" DOUBLE, "active" BOOL' \
    --dataset-name users \
    --sample-count 2 \
    --output-format text

Seed: 42
Start: 2024-01-01T00:00:00.000000000Z
[2024-01-01 0:00:00.0 +00:00:00] : DataSetName("users") { 'id': 'ee9a694c-0c16-4712-ab7b-788887ad520b', 'age': -1171422941, 'score': -79.32350957762677, 'active': true }
[2024-01-01 0:00:00.001 +00:00:00] : DataSetName("users") { 'id': '6a2e17fd-1f70-4dfb-8280-bebb751e2393', 'age': -1454245423, 'score': -0.3236575911005275, 'active': false }
```

## Example: DDL from a File

Given a file `users.ddl`:

```text
"id" VARCHAR,
"age" INT,
"score" DOUBLE,
"active" BOOL
```

Generate data from it:

```bash
$ beamline gen data \
    --seed 42 \
    --start-iso "2024-01-01T00:00:00Z" \
    --ddl-path users.ddl \
    --dataset-name users \
    --sample-count 100 \
    --output-format json
```

## Round-Tripping with Shape Inference

DDL generation pairs naturally with shape inference: infer a schema from existing
data as DDL, then use that DDL to generate more data with the same shape.

```bash
# Infer a DDL schema from a script
beamline infer-shape --script-path data.ion --output-format basic-ddl > schema.ddl

# Generate fresh data from the inferred schema
beamline gen data --seed 1 --start-auto --ddl-path schema.ddl --sample-count 1000
```

## Next Steps

- [Output Formats](./output-formats.md) - Emit DDL-generated data as Ion, JSON, or Parquet
- [Scripts](./scripts.md) - Move beyond `Uniform` defaults with full script control
- [Shape Inference](../schema/shape-inference.md) - Produce the DDL that feeds this mode
