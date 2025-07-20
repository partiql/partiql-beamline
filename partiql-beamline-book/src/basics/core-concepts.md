# Core Concepts

Before diving deeper into PartiQL Beamline's advanced features, it's essential to understand the fundamental concepts that power its data generation capabilities. This chapter will introduce you to the mathematical and computational foundations that make PartiQL Beamline both powerful and reliable.

## Stochastic Processes

At the heart of PartiQL Beamline lies the concept of **stochastic processes** - mathematical models that describe systems appearing to vary randomly over time.

### What is a Stochastic Process?

A stochastic process is a collection of random variables indexed by time or space. In simpler terms, it's a way to model how things change randomly over time while still following certain patterns or rules.

**Real-world examples:**
- Stock prices over time
- Sensor readings from IoT devices
- User activity on a website
- Network traffic patterns
- Temperature measurements

### Why Stochastic Processes Matter

Traditional random data generators often produce data that looks random but lacks the realistic patterns found in real-world data. Stochastic processes allow PartiQL Beamline to:

1. **Model Temporal Relationships**: Data points aren't just random - they follow realistic time-based patterns
2. **Create Correlations**: Different data elements can be related in meaningful ways
3. **Simulate Real Patterns**: Generate data that behaves like real-world systems
4. **Maintain Consistency**: Ensure generated data follows logical rules and constraints

### Example: Sensor Data

Consider a temperature sensor:
- **Simple Random**: Each reading is completely independent
- **Stochastic Process**: Readings follow realistic patterns (gradual changes, daily cycles, seasonal trends)

```ion
// Simple random (unrealistic)
temperature: UniformF64::{ low: -10.0, high: 40.0 }

// Stochastic process (realistic)
temperature: NormalF64::{ mean: 22.0, std_dev: 5.0 }
```

## Random Processes in PartiQL Beamline

PartiQL Beamline implements stochastic processes through **random processes** defined in Ion scripts.

### Anatomy of a Random Process

```ion
rand_process::{
    $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
    $data: {
        // Data structure definition
    }
}
```

Every random process has two key components:

1. **Arrival Process** (`$arrival`): Defines *when* data is generated
2. **Data Structure** (`$data`): Defines *what* data is generated

### Arrival Processes

Arrival processes control the timing of data generation. PartiQL Beamline supports several types:

#### Homogeneous Poisson Process

The most common arrival process, modeling events that occur at a constant average rate:

```ion
$arrival: HomogeneousPoisson:: { interarrival: minutes::5 }
```

**Characteristics:**
- Events occur independently
- Average rate is constant over time
- Time between events follows an exponential distribution
- Models many real-world phenomena (customer arrivals, system events, etc.)

**Use cases:**
- Web server requests
- Sensor readings
- User logins
- System alerts

#### Time Units

PartiQL Beamline supports various time units for arrival processes:

```ion
// Different time units
seconds::30      // 30 seconds
minutes::5       // 5 minutes  
hours::2         // 2 hours
days::1          // 1 day
milliseconds::100 // 100 milliseconds
```

## Data Generators

Data generators define the structure and content of generated data. They use probability distributions to create realistic values.

### Probability Distributions

PartiQL Beamline supports many probability distributions, each suited for different types of data:

#### Uniform Distributions

Generate values where each value in a range is equally likely:

```ion
// Discrete uniform (integers)
age: UniformU8::{ low: 18, high: 65 }

// Continuous uniform (floats)
temperature: UniformF64::{ low: 20.0, high: 30.0 }

// Uniform choice from literals
status: Uniform::{ choices: ["active", "inactive", "pending"] }
```

**Use cases:**
- IDs, categories, discrete choices
- Baseline random values
- Testing edge cases

#### Normal (Gaussian) Distributions

Generate values that cluster around a mean with a bell-curve distribution:

```ion
height: NormalF64::{ mean: 170.0, std_dev: 10.0 }
```

**Characteristics:**
- Most values near the mean
- Symmetric distribution
- Models many natural phenomena

**Use cases:**
- Physical measurements (height, weight)
- Performance metrics
- Error values

#### Other Distributions

```ion
// Exponential (for modeling wait times)
response_time: ExpF64::{ rate: 0.1 }

// Log-normal (for modeling sizes, prices)
file_size: LogNormalF64::{ location: 10.0, scale: 1.0 }

// Weibull (for modeling lifetimes, reliability)
device_lifetime: WeibullF64::{ shape: 2.0, scale: 1000.0 }
```

### Data Types

PartiQL Beamline supports all PartiQL data types:

#### Scalar Types

```ion
// Numbers
integer_val: UniformI32::{ low: 1, high: 1000 }
float_val: UniformF64::{ low: 0.0, high: 1.0 }
decimal_val: UniformDecimal::{ low: 1.99, high: 999.99 }

// Text
name: LoremIpsumTitle
description: LoremIpsum::{ min_words: 10, max_words: 50 }
pattern_text: Regex::{ pattern: "[A-Z]{2}[0-9]{4}" }

// Boolean
active: Bool::{ p: 0.8 }  // 80% chance of true

// Temporal
created_at: Instant
birth_date: Date

// Identifiers
user_id: UUID
```

#### Complex Types

```ion
// Structures
user: {
    id: UUID,
    name: LoremIpsumTitle,
    age: UniformU8::{ low: 18, high: 65 },
    preferences: {
        theme: Uniform::{ choices: ["light", "dark"] },
        notifications: Bool::{ p: 0.7 }
    }
}

// Arrays
tags: UniformArray::{ 
    min_size: 1, 
    max_size: 5, 
    element_type: LoremIpsumTitle 
}

// Union types
value: UniformAnyOf::{ types: [
    UniformI32::{ low: 1, high: 100 },
    LoremIpsumTitle,
    Bool
]}
```

## Variables and References

PartiQL Beamline supports variables for creating relationships and reusing values:

### Variable Definition

```ion
rand_processes::{
    // Define variables at the top level
    $user_count: UniformU8::{ low: 5, high: 20 },
    $id_generator: UUID,
    
    // Use variables in processes
    users: $user_count::[
        rand_process::{
            $data: {
                id: $id_generator,
                name: LoremIpsumTitle
            }
        }
    ]
}
```

### Variable Types

#### Generator Variables

Store data generators for reuse:

```ion
$temperature_sensor: NormalF64::{ mean: 22.0, std_dev: 3.0 }
$id_gen: UUID
```

#### Value Variables

Store computed values:

```ion
$success_rate: UniformF64::{ low: 0.95, high: 1.0 }
$is_successful: Bool::{ p: $success_rate }
```

#### Evaluation Control

Control when variables are evaluated:

```ion
// Evaluate once at script read time
$user_id: $id_gen::()

// Evaluate each time it's used
$request_id: $id_gen
```

## Datasets and Collections

PartiQL Beamline organizes generated data into datasets, which represent collections of related data.

### Single Dataset

```ion
rand_processes::{
    sensors: rand_process::{
        $data: { /* sensor data */ }
    }
}
```

### Multiple Datasets

```ion
rand_processes::{
    users: rand_process::{
        $data: { /* user data */ }
    },
    
    orders: rand_process::{
        $data: { /* order data */ }
    }
}
```

### Dynamic Datasets

Create multiple related datasets:

```ion
rand_processes::{
    $n: UniformU8::{ low: 3, high: 8 },
    
    // Creates client_1, client_2, ..., client_n datasets
    clients: $n::[
        'client_{ $@n }': rand_process::{
            $data: {
                client_id: '$@n',
                // ... other fields
            }
        }
    ]
}
```

## Reproducibility and Determinism

One of PartiQL Beamline's key strengths is its ability to generate reproducible data.

### Seeds

Seeds control the random number generation:

```bash
# Same seed = same data
cargo run gen data --seed 42 --script my-script.ion
cargo run gen data --seed 42 --script my-script.ion  # Identical output
```

### Timestamps

Control the simulation start time:

```bash
# Same timestamp = same temporal patterns
cargo run gen data --seed 42 --start-iso "2024-01-01T00:00:00Z" --script my-script.ion
```

### Deterministic Behavior

PartiQL Beamline ensures that:
- Same inputs always produce same outputs
- Random sequences are predictable and reproducible
- Debugging is possible with consistent data
- Tests can be reliable and repeatable

## Static vs. Dynamic Data

PartiQL Beamline supports both static and dynamic data generation:

### Dynamic Data (Default)

Generated during simulation with temporal patterns:

```ion
rand_process::{
    $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
    $data: {
        timestamp: Instant,
        value: UniformF64
    }
}
```

### Static Data

Generated once at the beginning of simulation:

```ion
static_data::{
    $data: {
        id: UUID,
        created_at: Instant,  // Will be simulation start time
        config: LoremIpsum
    }
}
```

**Use cases for static data:**
- Reference tables
- Configuration data
- Lookup tables
- Master data

## Summary

Understanding these core concepts is crucial for effectively using PartiQL Beamline:

1. **Stochastic Processes**: Mathematical foundation for realistic data patterns
2. **Random Processes**: Implementation of stochastic processes in PartiQL Beamline
3. **Arrival Processes**: Control timing of data generation
4. **Data Generators**: Create realistic values using probability distributions
5. **Variables**: Enable relationships and reuse in data generation
6. **Datasets**: Organize generated data into meaningful collections
7. **Reproducibility**: Ensure consistent, debuggable data generation
8. **Static vs. Dynamic**: Choose appropriate data generation patterns

In the next chapter, we'll dive deeper into scripts and random processes, exploring how to create more sophisticated data generation patterns and relationships.
