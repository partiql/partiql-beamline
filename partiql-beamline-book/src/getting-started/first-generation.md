# Your First Data Generation

Now that you have PartiQL Beamline installed, let's generate your first dataset! This hands-on tutorial will walk you 
through creating a simple sensor data generator and understanding the basic concepts.

## Quick Start: Using an Example Script

PartiQL Beamline comes with several example scripts. Let's start with the sensors example to see data generation in action.

### Step 1: Generate Your First Dataset

Run the following command to generate 2 sensor readings:

```bash
beamline gen data \
    --seed-auto \
    --start-auto \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion
```

You should see output similar to:

```
Seed: 5372343081885320050
Start: 2022-01-08T18:38:38.000000000Z
[2022-01-08 18:38:57.155 +00:00:00] : DataSetName("sensors") { 'tick': 19155, 'i8': 57, 'f': 30.103028021670184, 'w': 3.2669, 'd': 2, 'a': 'ed6b2d0c-dd09-4d7e-b1d3-fc16e3547eb5', 'ar1': [1.2, 1.4, 0.8], 'ar2': ['8fe9ee2c-a9e0-462a-8a44-a9abc51e759b', '0411eace-53be-4647-b351-3fa2de9b8e5f'], 'ar3': [3.2669, NULL, 3.0777], 'ar4': [10, 4, 8, 2], 'ar5': ['ed6b2d0c-dd09-4d7e-b1d3-fc16e3547eb5'] }
```

Congratulations! You've just generated your first synthetic dataset with PartiQL Beamline.

### Understanding the Output

Let's break down what happened:

- **Seed**: `5372343081885320050` — This random seed ensures reproducibility
- **Start**: `2024-01-20T20:05:41.000000000Z` — The simulation start time
- **Data Records**: Two sensor readings with timestamps, each containing:
  - `f`: A floating-point sensor value
  - `i8`: An 8-bit integer value
  - `tick`: A simulation tick counter

### Step 2: Reproduce the Same Data

Let's generate the exact same data using the seed from the previous run:

```bash
beamline gen data \
    --seed 5372343081885320050 \
    --start-auto \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion
```

Notice that the data values are identical, but the timestamps might be different because we used `--start-auto`. To get exactly the same output, use the same start time:

```bash
beamline gen data \
    --seed 5372343081885320050 \
    --start-iso "2022-01-08T18:38:38.000000000Z" \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion
```

Now you'll get exactly the same output as the first run!

## Understanding the Script

Let's examine the script that generated this data. Look at the contents of `partiql-beamline-sim/tests/scripts/sensors.ion`:

```ion
rand_processes::{
  $n:UniformU8::{
    low:2,
    high:10
  },
  sensors:$n::[
    rand_process::{
      $r:Uniform::{
        choices:[
          5,
          10
        ]
      },
      $arrival:HomogeneousPoisson::{
        interarrival:minutes::$r
      },
      $weight:UniformDecimal::{
        nullable:0.75,
        low:1.995,
        high:4.9999,
        optional:true
      },
      $anyof:UniformAnyOf::{
        types:[
          Tick,
          UniformF64,
          UUID,
          UniformDecimal::{
            low:1.995,
            high:4.9999,
            nullable:false
          }
        ]
      },
      $array:UniformArray::{
        min_size:3,
        max_size:3,
        element_type:UniformDecimal::{
          low:0.5,
          high:1.5
        }
      },
      $data:{
        tick:Tick,
        i8:UniformI8,
        f:UniformF64,
        w:$weight,
        d:UniformDecimal::{
          low:0.,
          high:42.,
          nullable:false
        },
        a:$anyof,
        ar1:$array,
        ar2:UniformArray::{
          min_size:2,
          max_size:4,
          element_type:UUID
        },
        ar3:UniformArray::{
          min_size:2,
          max_size:4,
          element_type:$weight
        },
        ar4:UniformArray::{
          min_size:2,
          max_size:4,
          element_type:UniformI8::{
            low:2,
            high:10
          }
        },
        ar5:UniformArray::{
          min_size:1,
          max_size:1,
          element_type:$anyof
        }
      }
    }
  ]
}
```

### Script Breakdown

1. **`rand_processes::`**: This annotation tells PartiQL Beamline that this structure defines random processes

2. **`$n: UniformU8::{ low: 1, high: 3 }`**: Creates a variable `n` that generates a random number between 1 and 3

3. **`sensors: $n::[...]`**: Creates a dataset called "sensors" with `n` random processes (1-3 processes)

4. **`rand_process::`**: Defines a single random process within the sensors dataset

5. **`$r: Uniform::[5,10]`**: Creates a variable `r` that randomly selects between 5 and 10

6. **`$arrival: HomogeneousPoisson:: { interarrival: minutes::$r }`**: Defines how often data arrives (every `r` minutes using a Poisson process)

7. **`$data:`**: Defines the structure of each generated data record:
   - `tick: Tick` - Current simulation tick
   - `id: '$@n'` - Process identifier
   - `i8: UniformI8` - Random 8-bit integer
   - `f: UniformF64` - Random 64-bit float

## Exploring Different Output Formats

PartiQL Beamline supports multiple output formats. Let's try generating the same data in different formats:

### Ion Pretty Format

```bash
beamline gen data \
    --seed 5372343081885320050 \
    --start-auto \
    --sample-count 3 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion \
    --output-format ion-pretty
```

This produces nicely formatted Ion output:

```ion
{
  seed: 12328924104731257599,
  start: "2024-01-20T20:05:41.000000000Z",
  data: {
    sensors: [
      {
        i8: -21,
        tick: 9421,
        f: 2.803799956162891e0,
        id: 1
      },
      {
        i8: -70,
        tick: 12294,
        f: 1.7229362418585936e1,
        id: 1
      },
      {
        i8: 84,
        tick: 32697,
        f: -2.4809825455060093e1,
        id: 0
      }
    ]
  }
}
```

### Text Format (Default)

The default text format is human-readable and great for quick inspection:

```bash
beamline gen data \
    --seed 5372343081885320050 \
    --start-auto \
    --sample-count 3 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion \
    --output-format text
```

## Creating Your Own Simple Script

Now let's create your own script from scratch. Create a new file called `my-first-script.ion`:

```ion
rand_processes::{
    simple_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            timestamp: Instant,
            temperature: UniformF64::{ low: 20.0, high: 35.0 },
            humidity: UniformF64::{ low: 30.0, high: 80.0 },
            sensor_id: UUID,
            active: Bool::{ p: 0.9 }
        }
    }
}
```

This script creates a simple weather sensor that generates:
- `timestamp`: Current simulation time
- `temperature`: Random temperature between 20-35°C
- `humidity`: Random humidity between 30-80%
- `sensor_id`: A unique UUID for each reading
- `active`: Boolean with 90% chance of being true

### Test Your Script

```bash
beamline gen data \
    --seed 42 \
    --start-auto \
    --sample-count 5 \
    --script-path my-first-script.ion \
    --output-format ion-pretty
```

## Understanding Key Concepts

### Seeds and Reproducibility

The `--seed` parameter controls randomness:
- `--seed-auto`: Generate a random seed (different data each time)
- `--seed 42`: Use a specific seed (same data each time)

### Start Times

The `--start` parameter controls simulation time:
- `--start-auto`: Use current time
- `--start-iso "2024-01-01T00:00:00Z"`: Use specific time
- `--start-epoch-ms 1704067200000`: Use epoch milliseconds

### Sample Count

The `--sample-count` parameter controls how many data points to generate. This is particularly useful for:
- Testing with small datasets
- Generating large datasets for performance testing
- Controlling output size

## Common Patterns

### Multiple Datasets

You can generate data for specific datasets using the `--dataset` flag:

```bash
beamline gen data \
    --seed 42 \
    --start-auto \
    --sample-count 10 \
    --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
    --dataset service --dataset client_1 \
    --output-format ion-pretty
```

### Controlling Nullability

You can control how often NULL values appear:

```bash
beamline gen data \
    --seed 42 \
    --start-auto \
    --sample-count 5 \
    --script-path my-first-script.ion \
    --default-nullable true \
    --pct-null 0.1  # 10% chance of NULL values
```

## Next Steps

Now that you've successfully generated your first datasets, you're ready to dive deeper into PartiQL Beamline's 
capabilities. In the next section, we'll explore the core concepts that power PartiQL Beamline's data generation, including:

- Random processes and stochastic modeling
- Data generators and their configurations
- Temporal modeling and arrival patterns
- Relationships between data elements

## Quick Reference

Here are the commands you've learned in this chapter:

```bash
# Basic data generation
beamline gen data --seed-auto --start-auto --sample-count N --script-path SCRIPT

# Reproducible generation
beamline gen data --seed SEED --start-iso "TIMESTAMP" --sample-count N --script-path SCRIPT

# Different output formats
beamline gen data ... --output-format [text|ion|ion-pretty]

# Specific datasets
beamline gen data ... --dataset DATASET_NAME

# Control nullability
beamline gen data ... --default-nullable true --pct-null 0.1
```

Congratulations on completing your first data generation with PartiQL Beamline! You're now ready to explore more advanced features and create more sophisticated synthetic datasets.
