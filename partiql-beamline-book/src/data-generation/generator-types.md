# Data Generator Types

PartiQL Beamline provides a comprehensive set of data generators that can create values following various statistical distributions and patterns. Each generator is designed to produce realistic data for specific use cases and data types.

## Generator Categories

### Basic System Generators

These generators provide fundamental values based on simulation state:

| Generator | PartiQL Type | Description | Configuration |
|-----------|--------------|-------------|---------------|
| `Bool` | BOOL | Boolean values using Bernoulli distribution | `p: f64` (probability of true, default: 0.5) |
| `Date` | DATETIME | Current simulation date | No configuration |
| `Instant` | DATETIME | Current simulation timestamp with timezone | No configuration |
| `Tick` | Int64 | Current simulation tick counter | No configuration |
| `UUID` | STRING | Version 4 UUID identifiers | No configuration |

#### Examples

```ion
rand_processes::{
    basic_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // System generators
            created_at: Instant,           // Current simulation time
            event_tick: Tick,              // Current tick counter  
            user_id: UUID,                 // Random UUID
            active: Bool,                  // 50% true by default
            premium: Bool::{ p: 0.1 },     // 10% true, 90% false
            event_date: Date               // Current simulation date
        }
    }
}
```

### Uniform Integer Generators

Generate integers using discrete uniform distribution:

#### Unsigned Integers

| Generator | Range | Default Range | Configuration |
|-----------|-------|---------------|---------------|
| `UniformU8` | 0 to 255 | `low: 0, high: 255` | `low: u8, high: u8` |
| `UniformU16` | 0 to 65,535 | `low: 0, high: 65535` | `low: u16, high: u16` |
| `UniformU32` | 0 to 4,294,967,295 | `low: 0, high: 4294967295` | `low: u32, high: u32` |
| `UniformU64` | 0 to 9,223,372,036,854,775,807 | `low: 0, high: 9223372036854775807` | `low: u64, high: u64` |

#### Signed Integers

| Generator | Range | Default Range | Configuration |
|-----------|-------|---------------|---------------|
| `UniformI8` | -128 to 127 | `low: -127, high: 127` | `low: i8, high: i8` |
| `UniformI16` | -32,768 to 32,767 | `low: -32767, high: 32767` | `low: i16, high: i16` |
| `UniformI32` | -2,147,483,648 to 2,147,483,647 | `low: -2147483647, high: 2147483647` | `low: i32, high: i32` |
| `UniformI64` | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 | `low: -9223372036854775807, high: 9223372036854775807` | `low: i64, high: i64` |

#### Examples

```ion
rand_processes::{
    numeric_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // Default ranges
            age_category: UniformU8,                    // 0-255
            small_count: UniformI8,                     // -127 to 127
            
            // Custom ranges
            human_age: UniformU8::{ low: 0, high: 120 },
            temperature_c: UniformI8::{ low: -40, high: 50 },
            user_score: UniformU16::{ low: 0, high: 1000 },
            large_id: UniformU64::{ low: 1000000, high: 9999999 }
        }
    }
}
```

### Floating Point Generators

#### Uniform Float

```ion
UniformF64::{ low: -127.0, high: 127.0 }  // Default range
UniformF64::{ low: 0.0, high: 1.0 }       // Unit interval
```

#### Uniform Decimal (Exact Arithmetic)

```ion
UniformDecimal::{ low: 0.995, high: 499.9999 }  // Default range
UniformDecimal::{ low: 9.99, high: 99.99 }      // Price range
```

#### Examples

```ion
rand_processes::{
    measurements: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
        $data: {
            // Floating point measurements
            temperature: UniformF64::{ low: -10.0, high: 40.0 },
            pressure: UniformF64::{ low: 980.0, high: 1050.0 },
            
            // Exact decimal values for money
            price: UniformDecimal::{ low: 9.99, high: 999.99 },
            tax_rate: UniformDecimal::{ low: 0.05, high: 0.12 }
        }
    }
}
```

### Statistical Distribution Generators

PartiQL Beamline supports several important probability distributions:

#### Normal Distribution

Models natural phenomena that cluster around a mean value:

```ion
NormalF64::{ mean: 100.0, std_dev: 15.0 }
```

**Use Cases:**
- Human measurements (height, weight, IQ scores)
- Measurement errors
- Natural phenomena
- AI model features

**Example:**
```ion
// Human height in centimeters (approximately normal)
height: NormalF64::{ mean: 170.0, std_dev: 10.0 }

// Test scores
test_score: NormalF64::{ mean: 75.0, std_dev: 12.0 }
```

#### Log-Normal Distribution

Models positive values that are log-normally distributed (multiplicative effects):

```ion
LogNormalF64::{ location: 0.0, scale: 1.0 }
```

**Use Cases:**
- Income distributions
- Stock prices
- File sizes
- Response times

**Example:**
```ion
// Income distribution (log-normal is realistic)
annual_income: LogNormalF64::{ location: 10.5, scale: 0.5 }

// File sizes
file_size_bytes: LogNormalF64::{ location: 10.0, scale: 2.0 }
```

#### Exponential Distribution

Models time between events or lifetimes:

```ion
ExpF64::{ rate: 1.0 }
```

**Use Cases:**
- Time between events
- Equipment lifetimes
- Queue waiting times
- Radioactive decay

**Example:**
```ion
// Time between customer arrivals (exponential inter-arrival times)
wait_time_minutes: ExpF64::{ rate: 0.1 }  // Average 10 minutes

// Equipment lifetime
lifetime_hours: ExpF64::{ rate: 0.001 }   // Average 1000 hours
```

#### Weibull Distribution

Models reliability, survival analysis, and extreme values:

```ion
WeibullF64::{ shape: 2.0, scale: 1000.0 }
```

**Use Cases:**
- Equipment failure times
- Material strength
- Wind speeds
- Survival analysis

**Example:**
```ion
// Equipment failure time
failure_time_hours: WeibullF64::{ shape: 2.0, scale: 8760.0 }  // ~1 year scale

// Material strength
breaking_force: WeibullF64::{ shape: 3.0, scale: 500.0 }
```

### String Generators

#### Lorem Ipsum Text

Generate placeholder text:

```ion
LoremIpsum::{ min_words: 10, max_words: 200 }
LoremIpsumTitle  // 3-8 words, title case
```

**Examples:**
```ion
description: LoremIpsum::{ min_words: 5, max_words: 20 }
title: LoremIpsumTitle
```

**Sample Output:**
```
description: "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor"
title: "Importari Putant Quae Autem Tanta"
```

#### Regular Expression Generator

Generate strings matching regex patterns:

```ion
Regex::{ pattern: "[A-Z]{2}[0-9]{4}" }
```

**Examples:**
```ion
rand_processes::{
    test_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // Country codes
            country: Regex::{ pattern: "[A-Z]{2}" },           // "US", "GB", "FR"
            
            // License plates  
            license: Regex::{ pattern: "[A-Z]{3}[0-9]{3}" },   // "ABC123"
            
            // Phone numbers
            phone: Regex::{ pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}" }, // "555-123-4567"
            
            // IPv4 addresses
            ip: Regex::{ pattern: "([0-9]{1,3}\\.){3}[0-9]{1,3}" }, // "192.168.1.1"
        }
    }
}
```

**Important Notes:**
- Use double backslashes for escape sequences: `\\d` not `\d`
- Character classes are Unicode-aware: `\\d` matches all Unicode digits
- Complex patterns supported: quantifiers, alternatives, character classes

#### Format String Generator

Generate formatted strings with variable substitution:

```ion
Format::{ pattern: "User #{$@n}" }
Format::{ pattern: "Order {$order_id} for customer {$customer_id}" }
```

### Complex Type Generators

#### Array Generator

Generate arrays with variable length and typed elements:

```ion
UniformArray::{
    min_size: 1,
    max_size: 10,
    element_type: UniformI32::{ low: 1, high: 100 }
}
```

**Configuration:**
- `min_size`: Minimum array length
- `max_size`: Maximum array length  
- `element_type`: Generator for array elements

**Examples:**
```ion
rand_processes::{
    array_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // Array of integers
            scores: UniformArray::{
                min_size: 3,
                max_size: 10,
                element_type: UniformU8::{ low: 0, high: 100 }
            },
            
            // Array of UUIDs
            related_ids: UniformArray::{
                min_size: 1,
                max_size: 5,
                element_type: UUID
            },
            
            // Array using variable generator
            weights: UniformArray::{
                min_size: 2,
                max_size: 4,
                element_type: $weight_generator
            }
        }
    }
}
```

#### Union Type Generator (Any Of)

Generate values that can be one of several types:

```ion
UniformAnyOf::{
    types: [
        UUID,
        UniformI32::{ low: 1, high: 1000 },
        LoremIpsumTitle,
        Bool
    ]
}
```

**Use Cases:**
- Heterogeneous data
- Schema evolution simulation  
- Polymorphic fields
- Variant types

**Example:**
```ion
rand_processes::{
    flexible_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // Field that can be different types
            metadata_value: UniformAnyOf::{
                types: [
                    UUID,                                    // Could be an ID
                    UniformI32::{ low: 1, high: 10000 },    // Could be a count
                    LoremIpsumTitle,                         // Could be a title
                    UniformDecimal::{ low: 0.0, high: 100.0 } // Could be a percentage
                ]
            }
        }
    }
}
```

#### Choice from Literals

Select from a predefined list of values:

```ion
Uniform::{ choices: [1, 2, 5, 10, 20] }
Uniform::{ choices: ["pending", "processing", "shipped", "delivered"] }
```

**Examples:**
```ion
rand_processes::{
    categorical_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            // Status choices
            status: Uniform::{ choices: ["active", "inactive", "pending"] },
            
            // Priority levels
            priority: Uniform::{ choices: [1, 2, 3, 4, 5] },
            
            // Mixed type choices
            config_value: Uniform::{ choices: [true, false, "auto", 0] }
        }
    }
}
```

### Timestamp Generators

#### Timestamp with Configuration

Generate timestamps with precision and timezone control:

```ion
Timestamp::{
    timezone: true,        // Include timezone (default: implementation dependent)
    precision: "microsecond" // Precision level
}
```

**Precision Options:**
- `"microsecond"` - Microsecond precision
- `"millisecond"` - Millisecond precision
- `"second"` - Second precision
- `"minute"` - Minute precision
- `"hour"` - Hour precision
- `"day"` - Day precision

**Example:**
```ion
rand_processes::{
    temporal_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::1 },
        $data: {
            // Different timestamp precisions
            precise_time: Timestamp::{ timezone: true, precision: "microsecond" },
            log_time: Timestamp::{ timezone: false, precision: "second" },
            daily_snapshot: Timestamp::{ timezone: true, precision: "day" }
        }
    }
}
```

## Generator Configuration Options

### Nullability and Optionality

All generators support NULL and MISSING value configuration:

```ion
// 20% NULL values
generator::{ nullable: 0.2 }

// 10% MISSING values (field won't appear)
generator::{ optional: 0.1 }

// Combined: 15% NULL, 5% MISSING, 80% present
generator::{ nullable: 0.15, optional: 0.05 }

// Disable NULL/MISSING
generator::{ nullable: false, optional: false }
```

### Range-Based Generators

Most numeric generators support range configuration:

```ion
// Integer ranges
UniformI32::{ low: 1, high: 1000 }
UniformU8::{ low: 18, high: 65 }  // Age range

// Float ranges  
UniformF64::{ low: -10.0, high: 50.0 }  // Temperature range

// Decimal ranges (exact arithmetic)
UniformDecimal::{ low: 9.99, high: 999.99 }  // Price range
```

### Statistical Distribution Parameters

#### Normal Distribution

```ion
NormalF64::{
    mean: 100.0,      // Mean (μ)
    std_dev: 15.0     // Standard deviation (σ)
}
```

**Example Applications:**
```ion
// Human height (cm) - approximately normal
height: NormalF64::{ mean: 170.0, std_dev: 10.0 }

// IQ scores - designed to be normal
iq_score: NormalF64::{ mean: 100.0, std_dev: 15.0 }

// Measurement errors
measurement_error: NormalF64::{ mean: 0.0, std_dev: 0.1 }
```

#### Log-Normal Distribution

```ion
LogNormalF64::{
    location: 0.0,    // Location parameter (μ)
    scale: 1.0        // Scale parameter (σ)
}
```

**Example Applications:**
```ion
// Income - typically log-normal
income: LogNormalF64::{ location: 10.5, scale: 0.5 }  // ~$36K median

// File sizes
file_size: LogNormalF64::{ location: 8.0, scale: 2.0 }  // Bytes

// Response times
response_ms: LogNormalF64::{ location: 3.0, scale: 0.5 }  // Milliseconds
```

#### Exponential Distribution

```ion
ExpF64::{
    rate: 1.0         // Rate parameter (λ)
}
```

**Example Applications:**
```ion
// Time between events
inter_arrival_time: ExpF64::{ rate: 0.1 }  // Average 10 time units

// Equipment lifetime  
lifetime_hours: ExpF64::{ rate: 0.0001 }  // Average 10,000 hours

// Queue waiting time
wait_time_sec: ExpF64::{ rate: 0.05 }  // Average 20 seconds
```

#### Weibull Distribution

```ion
WeibullF64::{
    shape: 2.0,       // Shape parameter (k)
    scale: 100.0      // Scale parameter (λ)
}
```

**Example Applications:**
```ion
// Equipment reliability
failure_time: WeibullF64::{ shape: 2.0, scale: 1000.0 }

// Wind speed modeling
wind_speed: WeibullF64::{ shape: 2.0, scale: 15.0 }

// Material strength
breaking_stress: WeibullF64::{ shape: 3.0, scale: 500.0 }
```

## Advanced Generator Usage

### Nested Structures

Create complex nested objects:

```ion
rand_processes::{
    complex_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::5 },
        $data: {
            user: {
                id: UUID,
                profile: {
                    name: LoremIpsumTitle,
                    age: UniformU8::{ low: 18, high: 80 },
                    preferences: {
                        notifications: Bool::{ p: 0.8 },
                        theme: Uniform::{ choices: ["light", "dark", "auto"] }
                    }
                },
                stats: {
                    login_count: UniformU32::{ low: 0, high: 10000 },
                    last_login: Instant,
                    score: NormalF64::{ mean: 85.0, std_dev: 12.0 }
                }
            }
        }
    }
}
```

### Arrays of Complex Objects

```ion
rand_processes::{
    order_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::2 },
        $data: {
            order_id: UUID,
            items: UniformArray::{
                min_size: 1,
                max_size: 10,
                element_type: {
                    product_id: UUID,
                    quantity: UniformU8::{ low: 1, high: 5 },
                    unit_price: UniformDecimal::{ low: 5.00, high: 200.00 }
                }
            }
        }
    }
}
```

### Variable References in Complex Generators

```ion
rand_processes::{
    // Define reusable components
    $id_gen: UUID,
    $weight_dist: NormalF64::{ mean: 70.0, std_dev: 15.0 },
    $status_options: Uniform::{ choices: ["new", "active", "suspended", "closed"] },
    
    users: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::1 },
        $data: {
            user_id: $id_gen,
            weight_kg: $weight_dist,
            account_status: $status_options,
            
            // Arrays using variables
            measurement_history: UniformArray::{
                min_size: 5,
                max_size: 20,
                element_type: $weight_dist  // Same distribution for all measurements
            },
            
            // Union types with variables
            contact_method: UniformAnyOf::{
                types: [
                    $id_gen,  // UUID for anonymous contact
                    Regex::{ pattern: "[a-z]+@[a-z]+\\.[a-z]{2,3}" },  // Email
                    Regex::{ pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}" }    // Phone
                ]
            }
        }
    }
}
```

## AI Model Training Examples

### Classification Dataset

```ion
rand_processes::{
    classification_training: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::1 },
        $data: {
            // Features with realistic distributions
            feature_1: NormalF64::{ mean: 0.0, std_dev: 1.0 },
            feature_2: NormalF64::{ mean: 0.0, std_dev: 1.0 },
            feature_3: LogNormalF64::{ location: 0.0, scale: 0.5 },
            feature_4: ExpF64::{ rate: 1.0 },
            
            // Categorical features
            category: Uniform::{ choices: ["A", "B", "C"] },
            region: Uniform::{ choices: ["North", "South", "East", "West"] },
            
            // Binary classification target
            label: Bool::{ p: 0.3 }  // 30% positive class
        }
    }
}
```

### Regression Dataset

```ion
rand_processes::{
    regression_training: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::1 },
        $data: {
            // Independent variables
            x1: NormalF64::{ mean: 10.0, std_dev: 2.0 },
            x2: UniformF64::{ low: 0.0, high: 20.0 },
            x3: ExpF64::{ rate: 0.1 },
            
            // Dependent variable (could be computed based on x1, x2, x3)
            y: NormalF64::{ mean: 50.0, std_dev: 10.0 },
            
            // Noise term
            noise: NormalF64::{ mean: 0.0, std_dev: 1.0 }
        }
    }
}
```

### Time Series Dataset

```ion
rand_processes::{
    time_series: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::60 },  // Every minute
        $data: {
            timestamp: Instant,
            
            // Trending value with noise
            base_value: NormalF64::{ mean: 100.0, std_dev: 5.0 },
            seasonal_component: NormalF64::{ mean: 0.0, std_dev: 10.0 },
            noise: NormalF64::{ mean: 0.0, std_dev: 2.0 },
            
            // External factors
            temperature: NormalF64::{ mean: 22.0, std_dev: 5.0 },
            humidity: UniformF64::{ low: 30.0, high: 80.0 }
        }
    }
}
```

## Performance Considerations

### Generator Efficiency

1. **Simple generators** (`UUID`, `Bool`, `UniformI32`) are fastest
2. **Statistical distributions** (`NormalF64`, `ExpF64`) require more computation
3. **String generators** (`LoremIpsum`, `Regex`) can be slower for complex patterns
4. **Array generators** scale with array size and element complexity

### Memory Usage

- **Streaming generation**: Constant memory usage regardless of dataset size
- **Variable caching**: Variables are computed once and reused
- **Complex nesting**: Memory usage scales with structure depth

### Optimization Tips

```ion
// Efficient - simple generators
id: UUID,
count: UniformU32::{ low: 1, high: 1000 }

// Less efficient - complex regex
complex_pattern: Regex::{ pattern: "(very|extremely|quite)\\s+complex\\s+pattern\\s+with\\s+many\\s+alternatives" }

// Efficient - reuse variables
$common_decimal: UniformDecimal::{ low: 1.0, high: 100.0 },
field1: $common_decimal,
field2: $common_decimal,
field3: $common_decimal
```

## Next Steps

- [Static Data](./static-data.md) - Learn about static_data generation
- [Output Formats](./output-formats.md) - Understand different output formats
- [Nullability](./nullability.md) - Deep dive into NULL and MISSING values
- [Scripts](./scripts.md) - Advanced Ion scripting techniques
