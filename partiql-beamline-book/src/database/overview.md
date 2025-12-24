# Database Overview

Beamline provides local data and schema generation capability. This allows to create a local copy of the generated data
in a local catalog directory.

## What is BeamlineLite?

**BeamlineLite** is Beamline's local database generation capability that creates filesystem-based databases containing:

- **Generated data** in Ion format  
- **Inferred schemas** in both Ion and SQL DDL formats
- **Metadata** about generation parameters
- **Original scripts** for reproducibility

## Database vs Data Generation

### Data Generation (gen data)

```bash
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 1000 \
  --output-format ion-pretty
```

**Output:** Stream of data records to stdout or file

**Use cases:** Data processing pipelines, API testing, analysis

### Database Generation (gen db)

```bash
beamline gen db beamline-lite \
  --seed 42 \
  --start-auto \
  --script-path sensors.ion \
  --sample-count 1000
```

**Output:** Complete database directory with data + schemas

**Use cases:** Local development databases, testing environments, demos

## BeamlineLite Database Structure

### Catalog Directory Layout

A BeamlineLite database creates a **catalog directory** with this structure:

```
beamline-catalog/
├── .beamline-manifest          # Generation metadata (JSON)
├── .beamline-script           # Original Ion script
├── <dataset>.ion              # Data files (one per dataset)
├── <dataset>.shape.ion        # Ion format schemas
└── <dataset>.shape.sql        # SQL DDL schemas
```

### Real Example from client-service.ion

```bash
$ beamline gen db beamline-lite \
    --seed-auto \
    --start-auto \
    --script-path client-service.ion \
    --sample-count 1000

writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!

$ tree beamline-catalog/
beamline-catalog/
├── .beamline-manifest
├── .beamline-script
├── service.ion
├── service.shape.ion
├── service.shape.sql
├── client_0.ion
├── client_0.shape.ion
├── client_0.shape.sql
├── client_1.ion
├── client_1.shape.ion  
├── client_1.shape.sql
└── ... (more client datasets)
```

## Database Files Deep Dive

### Manifest File (.beamline-manifest)

Contains generation metadata in JSON format:

```bash
$ cat beamline-catalog/.beamline-manifest
{"seed": "949665520117506306", "start": "2023-02-06T12:52:29.000000000Z", "ddl_syntax.version": "partiql_datatype_syntax.0.1"}
```

**Contents:**
- **seed**: Random seed used for generation (for reproducibility)
- **start**: Simulation start timestamp
- **ddl_syntax.version**: SQL DDL syntax version used in .shape.sql files

### Script File (.beamline-script)

Preserved copy of the original Ion script:

```bash
$ cat beamline-catalog/.beamline-script
rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },
    
    // A generator for client ids
    $id_gen: UUID,
    
    // ... rest of original script
}
```

**Purpose:**
- **Reproducibility**: Regenerate identical database later
- **Documentation**: What script created this database
- **Version control**: Track script changes over time

### Data Files (dataset.ion)

Contains generated data in compact Ion format:

```bash
$ cat beamline-catalog/client_0.ion
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "0de35d1e-a87c-e540-734d-6f2a4fa410c3", request_time: 2021-01-05T03:55:01.035000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "3539cdf0-6f7e-6bdc-c25a-4e0b7d8f8bac", request_time: 2021-01-05T03:55:01.182000000+00:00}
```

**Characteristics:**
- **One record per line**: Newline-delimited Ion format
- **Complete type information**: All Ion types preserved
- **Temporal ordering**: Records ordered by generation time

### Schema Files (dataset.shape.ion)

Ion format schema definitions:

```bash
$ cat beamline-catalog/client_0.shape.ion
{
  type: "bag",
  items: {
    type: "struct",
    constraints: [ordered, closed],
    fields: [
      { name: "id", type: "string" },
      { name: "request_id", type: "string" },
      { name: "request_time", type: "datetime" },
      { name: "success", type: "bool" }
    ]
  }
}
```

**Use cases:**
- **PartiQL validation**: Validate queries against schema
- **Type checking**: Ensure data types match expectations
- **Tool integration**: Tools can use schema information

### Schema Files (dataset.shape.sql)

SQL DDL format schemas:

```bash
$ cat beamline-catalog/service.shape.sql
"Account" VARCHAR,
"Distance" DECIMAL(2, 0),
"Operation" VARCHAR,
"Program" VARCHAR,
"Request" VARCHAR,
"StartTime" TIMESTAMP,
"Weight" DECIMAL(5, 4),
"client" VARCHAR,
"success" BOOL
```

**Use cases:**
- **Database creation**: Create tables in SQL databases
- **Schema documentation**: Human-readable schema reference
- **Migration scripts**: Database schema evolution
