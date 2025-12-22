# Output Formats

PartiQL Beamline supports multiple output formats for generated data, each optimized for different use cases. Understanding these formats helps you choose the right one for your workflow.

## Available Formats

The CLI supports four main output formats via `--output-format`:

| Format | Description | Use Case | Performance |
|--------|-------------|----------|-------------|
| `text` | Human-readable timestamped format | Debugging, inspection | Moderate |
| `ion` | Compact Ion text format | Data processing | Fast |
| `ion-pretty` | Pretty-printed Ion with metadata | Configuration, documentation | Slower |
| `ion-binary` | Binary Ion format | High-performance storage | Fastest |

## Text Format (Default)

### Characteristics

- **Human-readable**: Easy to read and debug
- **Timestamped**: Each record includes generation timestamp and dataset name
- **Streaming**: Records appear as they're generated
- **Metadata**: Shows seed and start time

### Output Structure

```
Seed: <seed_value>
Start: <start_timestamp>
[<timestamp>] : "<dataset_name>" { <ion_data> }
[<timestamp>] : "<dataset_name>" { <ion_data> }
...
```

### Example Output

```bash
$ beamline gen data \
    --seed 1234 \
    --start-auto \
    --script-path sensors.ion \
    --sample-count 5 \
    --output-format text

Seed: 1234
Start: 2024-05-10T04:04:53.000000000Z
[2024-05-10 4:06:07.274 +00:00:00] : DataSetName("sensors") { 'tick': 74274, 'i8': -86, 'f': 48.07286740416876, 'w': NULL, 'd': 23, 'a': 3.1640, 'ar1': [0.8, 1.1, 1.1], 'ar2': ['e8b12a6c-7cf1-45b6-a8a4-89cd6a418660', 'ba408184-3b94-41e7-860f-6042708bb4be'], 'ar3': [NULL, NULL], 'ar4': [6, 4], 'ar5': [3.1640] }
[2024-05-10 4:08:15.65 +00:00:00] : DataSetName("sensors") { 'tick': 202650, 'i8': 6, 'f': 45.56429323253781, 'w': NULL, 'd': 26, 'a': '613de2a3-195c-410f-8dac-56237f53aa99', 'ar1': [1.1, 0.9, 0.7], 'ar2': ['e0c6700e-f429-429a-a461-c018820fbafe', '9fce83a7-45ef-4210-affe-b87b45e3ac73'], 'ar3': [NULL, 2.4409], 'ar4': [4, 8], 'ar5': ['613de2a3-195c-410f-8dac-56237f53aa99'] }
```

### Use Cases

- **Development and debugging**: Easy to read individual records
- **Log file analysis**: Timestamped records for event correlation  
- **Quick inspection**: Rapid visual validation of generated data
- **Educational**: Learning how data generation works

## Ion Format

### Characteristics

- **Compact**: No pretty-printing or extra whitespace
- **Fast**: Minimal formatting overhead
- **Ion text**: Preserves all Ion type information
- **Processable**: Easy to parse with Ion libraries

### Output Structure

```ion
{seed:<seed>,start:"<timestamp>",data:{<dataset_name>:[{<record>},{<record>}...]}}
```

### Example Output

```bash
$ beamline gen data \
    --seed 42 \
    --start-auto \
    --script-path simple.ion \
    --sample-count 3 \
    --output-format ion

{seed:42,start:"2024-01-01T00:00:00Z",data:{sensors:[{f:-2.543639,i8:4,tick:125532},{f:-63.493088,i8:4,tick:218756},{f:12.345679,i8:-12,tick:253123}]}}
```

### Use Cases

- **Data processing pipelines**: Efficient parsing and processing
- **API responses**: Compact data transmission
- **Intermediate storage**: Balance between readability and efficiency
- **Configuration files**: Structured data that's still readable

## Ion Pretty Format

### Characteristics

- **Human-readable**: Well-formatted with indentation
- **Complete metadata**: Includes seed, start time, and full data structure
- **Ion text format**: Preserves all type information
- **Structured**: Clear hierarchical organization

### Output Structure

```ion
{
  seed: <seed>,
  start: "<timestamp>", 
  data: {
    <dataset_name>: [
      {
        <field>: <value>,
        <field>: <value>
      },
      {
        <field>: <value>,
        <field>: <value>
      }
    ]
  }
}
```

### Example Output

```bash
$ beamline gen data \
    --seed 123 \
    --start-auto \
    --script-path sensors.ion \
    --sample-count 2 \
    --output-format ion-pretty

{
  seed: 123,
  start: "2024-01-20T10:30:00.000000000Z",
  data: {
    sensors: [
      {
        f: -2.5436390152455175e0,
        i8: 4,
        tick: 125532
      },
      {
        f: -63.49308817145054e0,
        i8: 4,
        tick: 218756
      }
    ]
  }
}
```

### Use Cases

- **Configuration files**: Readable but structured data
- **Documentation**: Examples and samples in documentation
- **Data inspection**: Understanding complex nested structures
- **Archive storage**: Long-term storage with metadata

## Ion Binary Format

### Characteristics

- **Most compact**: Smallest file size
- **Fastest**: Highest performance for generation and parsing
- **Type preservation**: All Ion types preserved exactly
- **Not human-readable**: Requires Ion tools to read

### Example Usage

```bash
$ beamline gen data \
    --seed 999 \
    --start-auto \
    --script-path large_dataset.ion \
    --sample-count 1000000 \
    --output-format ion-binary > data.ion
```

### Use Cases

- **Large datasets**: Maximum efficiency for big data generation
- **High-performance applications**: Minimal parsing overhead
- **Storage optimization**: Smallest possible file sizes
- **Data transmission**: Efficient network transfer

## Format Comparison

### Size Comparison

For the same dataset with 1000 records:

```bash
# Generate in all formats for comparison
beamline gen data --seed 1 --start-auto --script-path data.ion --sample-count 1000 --output-format text > data.txt
beamline gen data --seed 1 --start-auto --script-path data.ion --sample-count 1000 --output-format ion > data.ion  
beamline gen data --seed 1 --start-auto --script-path data.ion --sample-count 1000 --output-format ion-pretty > data_pretty.ion
beamline gen data --seed 1 --start-auto --script-path data.ion --sample-count 1000 --output-format ion-binary > data.bin

# Compare sizes
ls -lh data.*
# Example results:
# -rw-r--r-- 1 user user 245K data.txt        (text - largest)
# -rw-r--r-- 1 user user 156K data.ion        (ion - medium)  
# -rw-r--r-- 1 user user 189K data_pretty.ion (pretty - larger due to formatting)
# -rw-r--r-- 1 user user 98K  data.bin        (binary - smallest)
```

### Performance Comparison

For generation of 100,000 records:

1. **ion-binary**: Fastest (baseline)
2. **ion**: ~10% slower than binary
3. **text**: ~25% slower than binary  
4. **ion-pretty**: ~40% slower than binary (due to formatting)

## Format-Specific Features

### Text Format Features

**Timestamp visibility**: See exactly when each event occurred in simulation time

```
[2024-01-01 08:15:23.456 +00:00] : "orders" { 'order_id': '123e4567', 'amount': 99.99 }
[2024-01-01 08:20:45.789 +00:00] : "orders" { 'order_id': '987fcdeb', 'amount': 149.50 }
```

**Dataset identification**: Clear dataset labels for multi-dataset scripts

### Ion Formats Features

**Type preservation**: All Ion types are preserved exactly

```ion
{
  decimal_field: 123.45,           // Exact decimal
  float_field: 123.45e0,           // Float with exponent notation
  timestamp: 2024-01-01T00:00:00Z, // Full timestamp precision
  uuid: "123e4567-e89b-12d3-a456-426614174000"
}
```

**Structured data**: Complex nested structures preserved

```ion
{
  user: {
    profile: {
      preferences: ["dark_mode", "notifications"]
    }
  }
}
```

### NULL and MISSING Representation

Different formats handle absent values differently:

#### Text Format
```
[timestamp] : "dataset" { 'present': 42, 'null_field': null }  // MISSING fields omitted
```

#### Ion Formats
```ion
{
  present: 42,
  null_field: null
  // missing_field is omitted entirely
}
```

## Multiple Dataset Output

### Text Format with Multiple Datasets

```bash
$ beamline gen data \
    --seed 100 \
    --start-auto \
    --script-path client_service.ion \
    --sample-count 10 \
    --output-format text

Seed: 100
Start: 2024-01-01T00:00:00Z
[2024-01-01 00:00:00.000 +00:00] : "customer_table" { 'id': 'abc-123', 'address': '0 Main St' }
[2024-01-01 00:00:00.000 +00:00] : "customer_table" { 'id': 'def-456', 'address': '1 Main St' }  
[2024-01-01 00:05:30.123 +00:00] : "service" { 'Request': 'req-001', 'Account': 'abc-123' }
[2024-01-01 00:05:30.124 +00:00] : "client_1" { 'id': 'abc-123', 'request_id': 'req-001' }
```

### Ion Pretty Format with Multiple Datasets

```ion
{
  seed: 100,
  start: "2024-01-01T00:00:00Z",
  data: {
    customer_table: [
      {
        id: "abc-123",
        address: "0 Main St"
      },
      {
        id: "def-456", 
        address: "1 Main St"
      }
    ],
    service: [
      {
        Request: "req-001",
        Account: "abc-123",
        StartTime: 2024-01-01T00:05:30.123Z
      }
    ],
    client_1: [
      {
        id: "abc-123",
        request_id: "req-001",
        request_time: 2024-01-01T00:05:30.124Z
      }
    ]
  }
}
```

## Choosing the Right Format

### Development and Testing

```bash
# Use text for quick debugging
beamline gen data --script-path debug.ion --sample-count 5 --output-format text

# Use ion-pretty for understanding structure  
beamline gen data --script-path complex.ion --sample-count 10 --output-format ion-pretty
```

### Production and Performance

```bash
# Use ion-binary for large datasets
beamline gen data --script-path production.ion --sample-count 1000000 --output-format ion-binary

# Use ion for balance of efficiency and readability
beamline gen data --script-path data.ion --sample-count 100000 --output-format ion
```

### Integration Workflows

```bash
# Generate for different consumers
beamline gen data --seed 42 --start-auto --script-path data.ion --sample-count 10000 --output-format ion-binary > high_perf.ion
beamline gen data --seed 42 --start-auto --script-path data.ion --sample-count 100 --output-format ion-pretty > documentation.ion
beamline gen data --seed 42 --start-auto --script-path data.ion --sample-count 1000 --output-format text > debug.txt
```

## Format-Specific Processing

### Processing Text Format

```bash
# Extract specific datasets
beamline gen data --script-path multi.ion --output-format text | \
  grep '"sensors"' | \
  head -10

# Analyze timestamps
beamline gen data --script-path temporal.ion --output-format text | \
  awk -F'\\[|\\]' '{print $2}' | \
  head -20
```

### Processing Ion Formats

```bash
# Use Ion tools for processing
beamline gen data --script-path data.ion --output-format ion-binary | \
  ion-cli query "SELECT * FROM data.sensors WHERE f > 0"

# Convert between formats
beamline gen data --script-path data.ion --output-format ion | \
  ion-cli pretty > formatted.ion
```

### Pipeline Integration

```bash
# Generate and immediately process
beamline gen data \
  --seed 123 \
  --start-auto \
  --script-path metrics.ion \
  --sample-count 10000 \
  --output-format ion-pretty | \
  jq '.data.metrics[] | select(.temperature > 25)' | \
  head -10
```

## Database Generation Formats

Database generation creates multiple file formats automatically:

```bash
$ beamline gen db beamline-lite \
    --seed 42 \
    --start-auto \
    --script-path data.ion \
    --sample-count 1000

$ ls -la beamline-catalog/
-rw-r--r-- 1 user user   145 .beamline-manifest    # JSON metadata
-rw-r--r-- 1 user user  2.1K .beamline-script      # Ion script
-rw-r--r-- 1 user user   89K sensors.ion           # Data in Ion format
-rw-r--r-- 1 user user   412 sensors.shape.ion     # Schema in Ion format  
-rw-r--r-- 1 user user   298 sensors.shape.sql     # Schema in SQL DDL format
```

### Data Files (Ion Format)

```bash
$ head -3 beamline-catalog/sensors.ion
{f: -2.5436390152455175e0, i8: 4, tick: 125532}
{f: -63.49308817145054e0, i8: 4, tick: 218756}
{f: 12.34567890123456e0, i8: -12, tick: 253123}
```

### Schema Files (Ion Format)

```bash
$ cat beamline-catalog/sensors.shape.ion
{
  type: "bag",
  items: {
    type: "struct",
    constraints: [ordered, closed],
    fields: [
      { name: "f", type: "double" },
      { name: "i8", type: "int8" },
      { name: "tick", type: "int8" }
    ]
  }
}
```

### Schema Files (SQL DDL Format)

```bash
$ cat beamline-catalog/sensors.shape.sql
"f" DOUBLE,
"i8" INT8,
"tick" INT8
```

## Format Selection Guidelines

### By Use Case

| Use Case | Recommended Format | Rationale |
|----------|-------------------|-----------|
| **Quick debugging** | `text` | Timestamps and human readability |
| **Data inspection** | `ion-pretty` | Structure visibility with metadata |
| **Large dataset generation** | `ion-binary` | Maximum performance and compression |
| **Data processing** | `ion` | Good balance of efficiency and readability |
| **Documentation** | `ion-pretty` | Clear structure for examples |
| **Long-term storage** | `ion-binary` | Most compact and preserves all types |

### By Dataset Size

| Dataset Size | Recommended Format | Alternative |
|-------------|-------------------|-------------|
| **< 100 records** | `text` or `ion-pretty` | For inspection |
| **100 - 10K records** | `ion` or `ion-pretty` | Based on use case |
| **10K - 100K records** | `ion` or `ion-binary` | For efficiency |
| **> 100K records** | `ion-binary` | Maximum performance |

### By Integration Target

| Target System | Recommended Format | Notes |
|---------------|-------------------|-------|
| **Ion-aware tools** | `ion-binary` | Native format |
| **JSON processors** | `ion` + conversion | Ion can be converted to JSON |
| **SQL databases** | Use `gen db` | Creates SQL schemas automatically |
| **Log analysis** | `text` | Timestamped format |
| **Documentation** | `ion-pretty` | Human-readable structure |

## Format Conversion Patterns

### Manual Conversion

```bash
# Generate in efficient format, convert for specific use
beamline gen data --script-path data.ion --sample-count 10000 --output-format ion-binary > efficient.ion

# Convert to pretty format for inspection
ion-cli pretty < efficient.ion > readable.ion

# Extract specific fields
ion-cli query "SELECT data.sensors[*].temperature FROM `efficient.ion`" > temperatures.ion
```

### Multi-Format Generation

```bash
#!/bin/bash
# generate-multi-format.sh

SCRIPT="$1"
SEED="$2"
COUNT="$3"

# Generate in multiple formats
beamline gen data --seed $SEED --start-auto --script-path $SCRIPT --sample-count $COUNT --output-format ion-binary > data.bin
beamline gen data --seed $SEED --start-auto --script-path $SCRIPT --sample-count 100 --output-format ion-pretty > sample.ion
beamline gen data --seed $SEED --start-auto --script-path $SCRIPT --sample-count 10 --output-format text > debug.txt

echo "Generated:"
echo "- data.bin (binary, $COUNT records)"  
echo "- sample.ion (pretty, 100 records)"
echo "- debug.txt (text, 10 records)"
```

## Integration Examples

### Web API Integration

```bash
# Generate data for API testing
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path api_test_data.ion \
  --sample-count 1000 \
  --output-format ion-pretty | \
  jq '.data' > api_test_payload.json
```

### Database Loading

```bash
# Generate data and schema for database
beamline gen db beamline-lite \
  --seed 100 \
  --start-auto \
  --script-path warehouse_data.ion \
  --sample-count 50000

# Use generated SQL schema
psql -d warehouse -f beamline-catalog/orders.shape.sql

# Convert data for loading (would need custom conversion)
# partiql-to-csv beamline-catalog/orders.ion > orders.csv
# COPY orders FROM 'orders.csv' WITH CSV HEADER;
```

### Analytics Pipeline

```bash
#!/bin/bash
# analytics-pipeline.sh

# Generate raw data efficiently  
beamline gen data \
  --seed 202401 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path analytics.ion \
  --sample-count 1000000 \
  --output-format ion-binary > raw_data.ion

# Generate sample for validation
beamline gen data \
  --seed 202401 \
  --start-iso "2024-01-01T00:00:00Z" \
  --script-path analytics.ion \
  --sample-count 100 \
  --output-format ion-pretty > sample_validation.ion

echo "Analytics data generated:"
echo "- Raw data: $(wc -l < raw_data.ion) records in binary format"
echo "- Validation sample: 100 records in pretty format"
```

## Best Practices

### 1. Match Format to Purpose

```bash
# Debugging - use text
beamline gen data --script-path new_script.ion --sample-count 5 --output-format text

# Production - use binary
beamline gen data --script-path prod_data.ion --sample-count 1000000 --output-format ion-binary

# Documentation - use pretty
beamline gen data --script-path examples.ion --sample-count 10 --output-format ion-pretty
```

### 2. Consider File Size for Large Datasets

```bash
# Check estimated size first
beamline gen data --script-path large.ion --sample-count 1000 --output-format ion-binary | wc -c
# If 1000 records = 50KB, then 1M records ≈ 50MB
```

### 3. Use Appropriate Format for Storage

```bash
# Long-term storage
beamline gen data --script-path archive.ion --sample-count 100000 --output-format ion-binary

# Working files  
beamline gen data --script-path working.ion --sample-count 1000 --output-format ion-pretty

# Quick inspection
beamline gen data --script-path inspect.ion --sample-count 20 --output-format text
```

### 4. Document Format Choices

```bash
# Document why you chose specific formats
echo "# Data Formats Used
- raw_data.bin: ion-binary for maximum efficiency (1M+ records)
- sample.ion: ion-pretty for human inspection (100 records)  
- debug.txt: text format for timestamp analysis (50 records)
" > FORMAT_NOTES.md
```

## Next Steps

- [Scripts](./scripts.md) - Advanced Ion scripting techniques
- [Datasets](./datasets.md) - Working with multiple datasets and relationships  
- [CLI Data Commands](../cli/data-commands.md) - Complete CLI format options reference
