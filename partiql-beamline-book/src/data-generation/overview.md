# Data Generation Overview

Beamline's data generation system creates synthetic data using 
**[stochastic processes](https://en.wikipedia.org/wiki/Stochastic_process)** and 
**[probability distributions](https://en.wikipedia.org/wiki/Probability_distribution)**. 
The system is built around three core concepts: **random processes**, **value generators**, and **temporal modeling**.

## Architecture Overview

Data generation in Beamline follows a layered architecture:

1. **Random Processes** — Mathematical models that describe how events occur over time
2. **Value Generators** — Components that create specific data types and values
3. **Arrival Times** — Models for when events occur in the simulation
4. **Simulation Context** — Manages state, timing, and reproducibility

## Core Concepts

### Random Processes

A **Random Process** (also called Stochastic Process) is a mathematical model of systems that appear to vary 
randomly over time. In Beamline, these processes control:

- **When** data arrives (temporal patterns)
- **What** data is generated (value types and structures)
- **How** data relates (cross-field dependencies)

### Value Generators

**Value Generators** are the building blocks that create actual data values. They can generate:

- **Scalar values**: numbers, strings, booleans, timestamps
- **Complex structures**: objects, arrays, nested data
- **Statistical distributions**: normal, exponential, Weibull, etc.
- **Specialized types**: UUIDs, formatted text, regex patterns

Each generator can be configured for:
- **Nullability**: Probability of generating `NULL` values
- **Optionality**: Probability of generating `MISSING` values
- **Value ranges**: Minimum and maximum bounds
- **Distribution parameters**: Mean, standard deviation, shape, scale

### Temporal Modeling

Beamline models data generation as events occurring over time using:

- **Arrival processes**: When events occur (e.g. [Poisson Point Process](https://en.wikipedia.org/wiki/Poisson_point_process))
- **Simulation time**: Virtual time that advances as events are generated
- **Tick counters**: Global state that increments with each event
- **Instant generators**: Current simulation time when values are created

## Ion Script Structure

All data generation is controlled through Amazon Ion scripts with this basic structure:

```ion
rand_processes::{
    // Variable definitions
    $variable_name: GeneratorType::{ configuration },
    
    // Dataset definitions
    dataset_name: dataset_configuration
}
```

### Variable Definitions

Variables allow you to define generators once and reuse them:

```ion
rand_processes::{
    // Define reusable generators
    $id_generator: UUID,
    $weight_generator: UniformDecimal::{ low: 1.0, high: 10.0 },
    $count_range: UniformU8::{ low: 5, high: 20 },
    
    // Use variables in dataset definitions
    products: $count_range::[
        // ... uses $id_generator and $weight_generator
    ]
}
```

### Dataset Configurations

Datasets can be configured in several ways:

#### 1. Single Random Process

```ion
dataset_name: rand_process::{
    $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },
    $data: {
        id: UUID,
        value: UniformF64
    }
}
```

#### 2. Static Data (Generated Once)

```ion
dataset_name: static_data::{
    $data: {
        id: UUID,
        name: LoremIpsumTitle
    }
}
```

#### 3. Multiple Instances with Loops

```ion
$n: UniformU8::{ low: 2, high: 5 },

dataset_name: $n::[
    rand_process::{
        $data: {
            instance_id: '$@n',  // Current loop index
            value: UniformF64
        }
    }
]
```

## Data Generator Types

### Basic Generators



| Generator | Description             | PartiQL Type | Configuration                                |
|-----------|-------------------------|--------------|----------------------------------------------|
| `Bool`    | Boolean values          | BOOL         | `p: f64` (probability of true, default: 0.5) |
| `UUID`    | UUID v4 identifiers     | STRING       | No configuration                             |
| `Tick`    | Current simulation tick | Int64        | No configuration                             |
| `Instant` | Current simulation time | DATETIME     | No configuration                             |
| `Date`    | Current simulation date | DATETIME     | No configuration                             |

### Numeric Generators

#### Uniform Integer Generators

```ion
// Unsigned integers
UniformU8::{ low: 0, high: 255 }           // 8-bit unsigned
UniformU16::{ low: 0, high: 65535 }        // 16-bit unsigned  
UniformU32::{ low: 0, high: 4294967295 }  // 32-bit unsigned
UniformU64::{ low: 0, high: 18446744073709551615 }  // 64-bit unsigned

// Signed integers
UniformI8::{ low: -128, high: 127 }        // 8-bit signed
UniformI16::{ low: -32768, high: 32767 }   // 16-bit signed
UniformI32::{ low: -2147483648, high: 2147483647 }  // 32-bit signed
UniformI64::{ low: -9223372036854775808, high: 9223372036854775807 }  // 64-bit signed
```

#### Floating Point Generators

```ion
// Uniform float
UniformF64::{ low: -127.0, high: 127.0 }

// Uniform decimal (exact arithmetic)
UniformDecimal::{ low: 0.995, high: 499.9999 }
```

#### Statistical Distribution Generators

```ion
// Normal distribution (bell curve)
NormalF64::{ mean: 100.0, std_dev: 15.0 }

// Log-normal distribution
LogNormalF64::{ location: 0.0, scale: 1.0 }

// Weibull distribution
WeibullF64::{ shape: 2.0, scale: 1.0 }

// Exponential distribution
ExpF64::{ rate: 1.0 }
```

### String Generators

```ion
// Lorem Ipsum text
LoremIpsum::{ min_words: 10, max_words: 200 }

// Lorem Ipsum titles (3-8 words, title case)
LoremIpsumTitle

// Regular expression patterns
Regex::{ pattern: "[A-Z]{2}[0-9]{3}" }

// Format strings with variable substitution
Format::{ pattern: "User #{$@n}" }
```

### Complex Type Generators

#### Arrays

```ion
UniformArray::{
    min_size: 1,
    max_size: 5,
    element_type: UniformI32::{ low: 1, high: 100 }
}
```

#### Union Types (Any Of)

```ion
UniformAnyOf::{
    types: [
        UUID,
        UniformI32::{ low: 1, high: 1000 },
        LoremIpsumTitle
    ]
}
```

#### Choice from Literals

```ion
Uniform::{ choices: [1, 2, 5, 10, 20] }
```

## Nullability and Optionality

Every generator supports NULL and MISSING value generation:

### Nullability (NULL values)

```ion
// 20% chance of NULL values
generator::{ nullable: 0.2 }

// Never NULL
generator::{ nullable: false }

// Always NULL (not useful, but possible)
generator::{ nullable: 1.0 }
```

### Optionality (MISSING values)

```ion
// 10% chance of MISSING values  
generator::{ optional: 0.1 }

// Never MISSING
generator::{ optional: false }

// Always MISSING (field won't appear)
generator::{ optional: 1.0 }
```

### Combined Configuration

```ion
// 20% NULL, 10% MISSING, 70% present values
price: UniformDecimal::{
    nullable: 0.2,
    optional: 0.1, 
    low: 9.99,
    high: 999.99
}
```

## Arrival Processes

Control when events occur in simulation time. Beamline is currently supporintg only
[Homogeneous Poisson Process](https://en.wikipedia.org/wiki/Poisson_point_process):

### Homogeneous Poisson Process

Statistically indepe events occur at a constant average rate with random intervals:

```ion
$arrival: HomogeneousPoisson::{ interarrival: minutes::5 }
```

**Time units supported:**
- `milliseconds::N` - N milliseconds between events
- `seconds::N` - N seconds between events  
- `minutes::N` - N minutes between events
- `hours::N` - N hours between events
- `days::N` - N days between events

## Variable References and Scope

### Variable Definition and Usage

```ion
rand_processes::{
    $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },

    // Define variables at top level
    $customer_id: UUID,
    $price_range: UniformDecimal::{ low: 9.99, high: 199.99 },
    
    orders: rand_process::{
        $data: {
            customer: $customer_id,     // Reference variable
            price: $price_range,        // Reference variable
            order_id: UUID              // Direct generator
        }
    }
}
```

### Forced Evaluation with `::()`

Force generator evaluation at script read time (not generation time):

```ion
rand_processes::{
    $id_gen: UUID,
    
    customers: 3::[
        {
            // Each customer gets the same ID across all their records
            $id: $id_gen::(),  // Evaluated once per customer
            
            customer_profile: static_data::{
                $data: {
                    id: $id,           // Same ID for this customer
                    name: LoremIpsumTitle
                }
            },
            
            transactions: rand_process::{
                $data: {
                    customer_id: $id,  // Same ID for this customer
                    transaction_id: UUID,  // New UUID per transaction
                    amount: UniformDecimal::{ low: 10.0, high: 500.0 }
                }
            }
        }
    ]
}
```

### Loop Index Variable `$@n`

Access the current loop index in array definitions:

```ion
$n: UniformU8::{ low: 3, high: 7 },

clients: $n::[
    {
        'client_$@n': rand_process::{  // Dynamic dataset name
            $data: {
                client_number: '$@n',   // Current index as value
                name: Format::{ pattern: "Client #{$@n}" }
            }
        }
    }
]
```

## Some Real Examples

### Simple Sensor Data

```ion
rand_processes::{
    $n: UniformU8::{ low: 2, high: 10 },

    sensors: $n::[
        rand_process::{
            $r: Uniform::{ choices: [5,10] },
            $arrival: HomogeneousPoisson::{ interarrival: minutes::$r },
            $data: {
                tick: Tick,
                i8: UniformI8,
                f: UniformF64
            }
        }
    ]
}
```

### Complex Statistical Data

```ion
rand_processes::{
    test_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::100 },
        $data: {
            // Statistical distributions
            normal_score: NormalF64::{ mean: 100.0, std_dev: 15.0 },
            exponential_wait: ExpF64::{ rate: 0.1 },
            weibull_lifetime: WeibullF64::{ shape: 2.0, scale: 1000.0 },
            
            // Arrays with statistical elements
            measurements: UniformArray::{
                min_size: 5,
                max_size: 10,
                element_type: NormalF64::{ mean: 50.0, std_dev: 5.0 }
            },
            
            // Union types
            mixed_value: UniformAnyOf::{
                types: [
                    NormalF64::{ mean: 0.0, std_dev: 1.0 },
                    UniformI32::{ low: 1, high: 100 },
                    UUID
                ]
            }
        }
    }
}
```

### Static and Dynamic Data Combination

```ion
rand_processes::{
    $n: UniformU8::{ low: 5, high: 20 },
    $id_gen: UUID,

    customers: $n::[
        {
            $id: $id_gen::(),  // One ID per customer
            
            // Static customer data (generated once)
            customer_table: static_data::{
                $data: {
                    id: $id,
                    address: Format::{ pattern: "{$@n} Main Street" }
                }
            },
            
            // Dynamic order data (generated over time)
            orders: rand_process::{
                $r: UniformU8::{ low: 1, high: 30 },
                $arrival: HomogeneousPoisson::{ interarrival: days::$r },
                $data: {
                    customer_id: $id,
                    order_id: UUID,
                    timestamp: Instant
                }
            }
        }
    ]
}
```

## Probability Distribution Support

Beamline provides support for data generation based on probability distributions, 
making it particularly valuable for AI model training and statistical simulation:

### Available Distributions

- **Normal Distribution**: `NormalF64::{ mean: μ, std_dev: σ }`
- **Log-Normal Distribution**: `LogNormalF64::{ location: μ, scale: σ }`
- **Exponential Distribution**: `ExpF64::{ rate: λ }`
- **Weibull Distribution**: `WeibullF64::{ shape: k, scale: λ }`
- **Uniform Distribution**: All `Uniform*` generators use uniform distribution

### AI Model Training Applications

```ion
rand_processes::{
    training_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::1 },
        $data: {
            // Features following realistic distributions
            age: NormalF64::{ mean: 35.0, std_dev: 12.0 },
            income: LogNormalF64::{ location: 10.5, scale: 0.5 },
            response_time: ExpF64::{ rate: 0.1 },
            
            // Categorical features
            category: Uniform::{ choices: ["A", "B", "C", "D"] },
            
            // Correlated features using shared variables
            experience_years: NormalF64::{ mean: 8.0, std_dev: 5.0 },
            
            // Target variable (could be based on features)
            target: Bool::{ p: 0.3 }
        }
    }
}
```

## Next Steps

Now that you understand the data generation overview, explore specific aspects:

- [Generator Types](./generator-types.md) - Detailed guide to all available generators
- [Static Data](./static-data.md) - Using static_data for reference tables
- [Output Formats](./output-formats.md) - Understanding different output formats
- [Nullability](./nullability.md) - Controlling NULL and MISSING values
- [Scripts](./scripts.md) - Advanced Ion script techniques
- [Datasets](./datasets.md) - Working with multiple datasets and relationships
