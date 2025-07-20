# Data Generator Reference

This comprehensive reference covers all data generators available in PartiQL Beamline, their configurations, and usage examples. Use this as a quick lookup when building your data generation scripts.

## Scalar Generators

### Boolean Generators

#### Bool

Generates boolean values with configurable probability.

**Configuration:**
- `p: f64` - Probability of generating `true` (default: 0.5)

**PartiQL Type:** `BOOL`

**Examples:**
```ion
// 50% chance of true (default)
active: Bool

// 80% chance of true
premium: Bool::{ p: 0.8 }

// 10% chance of true
error_occurred: Bool::{ p: 0.1 }
```

### Numeric Generators

#### Integer Generators

##### UniformU8
Generates unsigned 8-bit integers (0-255).

**Configuration:**
- `low: u8` - Minimum value (default: 0)
- `high: u8` - Maximum value (default: 255)

**PartiQL Type:** `INT64`

**Examples:**
```ion
// Full range (0-255)
byte_value: UniformU8

// Custom range
age: UniformU8::{ low: 18, high: 65 }
percentage: UniformU8::{ low: 0, high: 100 }
```

##### UniformU16
Generates unsigned 16-bit integers (0-65,535).

**Configuration:**
- `low: u16` - Minimum value (default: 0)
- `high: u16` - Maximum value (default: 65,535)

**PartiQL Type:** `INT64`

##### UniformU32
Generates unsigned 32-bit integers (0-4,294,967,295).

**Configuration:**
- `low: u32` - Minimum value (default: 0)
- `high: u32` - Maximum value (default: 4,294,967,295)

**PartiQL Type:** `INT64`

##### UniformU64
Generates unsigned 64-bit integers (0-9,223,372,036,854,775,807).

**Configuration:**
- `low: u64` - Minimum value (default: 0)
- `high: u64` - Maximum value (default: 9,223,372,036,854,775,807)

**PartiQL Type:** `INT64`

##### UniformI8
Generates signed 8-bit integers (-127 to 127).

**Configuration:**
- `low: i8` - Minimum value (default: -127)
- `high: i8` - Maximum value (default: 127)

**PartiQL Type:** `INT64`

**Examples:**
```ion
// Temperature in Celsius
temperature: UniformI8::{ low: -40, high: 50 }

// Small signed values
offset: UniformI8::{ low: -10, high: 10 }
```

##### UniformI16
Generates signed 16-bit integers (-32,767 to 32,767).

**Configuration:**
- `low: i16` - Minimum value (default: -32,767)
- `high: i16` - Maximum value (default: 32,767)

**PartiQL Type:** `INT64`

##### UniformI32
Generates signed 32-bit integers (-2,147,483,647 to 2,147,483,647).

**Configuration:**
- `low: i32` - Minimum value (default: -2,147,483,647)
- `high: i32` - Maximum value (default: 2,147,483,647)

**PartiQL Type:** `INT64`

##### UniformI64
Generates signed 64-bit integers.

**Configuration:**
- `low: i64` - Minimum value (default: -9,223,372,036,854,775,807)
- `high: i64` - Maximum value (default: 9,223,372,036,854,775,807)

**PartiQL Type:** `INT64`

#### Floating-Point Generators

##### UniformF64
Generates 64-bit floating-point numbers with uniform distribution.

**Configuration:**
- `low: f64` - Minimum value (default: -127.0)
- `high: f64` - Maximum value (default: 127.0)

**PartiQL Type:** `DOUBLE`

**Examples:**
```ion
// Default range
random_value: UniformF64

// Custom range
price: UniformF64::{ low: 0.99, high: 999.99 }
probability: UniformF64::{ low: 0.0, high: 1.0 }
```

##### NormalF64
Generates normally distributed 64-bit floating-point numbers.

**Configuration:**
- `mean: f64` - Mean of the distribution (required)
- `std_dev: f64` - Standard deviation (required)

**PartiQL Type:** `DOUBLE`

**Examples:**
```ion
// Human height (cm)
height: NormalF64::{ mean: 170.0, std_dev: 10.0 }

// Test scores
score: NormalF64::{ mean: 75.0, std_dev: 15.0 }

// Response time (ms)
response_time: NormalF64::{ mean: 200.0, std_dev: 50.0 }
```

##### LogNormalF64
Generates log-normally distributed 64-bit floating-point numbers.

**Configuration:**
- `location: f64` - Location parameter (required)
- `scale: f64` - Scale parameter (required)

**PartiQL Type:** `DOUBLE`

**Examples:**
```ion
// File sizes (bytes)
file_size: LogNormalF64::{ location: 10.0, scale: 1.0 }

// Income distribution
income: LogNormalF64::{ location: 11.0, scale: 0.5 }
```

##### WeibullF64
Generates Weibull distributed 64-bit floating-point numbers.

**Configuration:**
- `shape: f64` - Shape parameter (required)
- `scale: f64` - Scale parameter (required)

**PartiQL Type:** `DOUBLE`

**Examples:**
```ion
// Device lifetime (hours)
lifetime: WeibullF64::{ shape: 2.0, scale: 1000.0 }

// Wind speed
wind_speed: WeibullF64::{ shape: 2.0, scale: 10.0 }
```

##### ExpF64
Generates exponentially distributed 64-bit floating-point numbers.

**Configuration:**
- `rate: f64` - Rate parameter (required)

**PartiQL Type:** `DOUBLE`

**Examples:**
```ion
// Wait time between events
wait_time: ExpF64::{ rate: 0.1 }

// Time to failure
failure_time: ExpF64::{ rate: 0.001 }
```

#### Decimal Generators

##### UniformDecimal
Generates decimal numbers with uniform distribution.

**Configuration:**
- `low: f64` - Minimum value (default: -127.0)
- `high: f64` - Maximum value (default: 127.0)

**PartiQL Type:** `DECIMAL(p,s)` (precision and scale inferred)

**Examples:**
```ion
// Currency values
price: UniformDecimal::{ low: 1.99, high: 999.99 }

// Precise measurements
weight: UniformDecimal::{ low: 0.001, high: 100.000 }
```

### String Generators

#### UUID
Generates Version 4 UUIDs.

**Configuration:** None

**PartiQL Type:** `STRING`

**Examples:**
```ion
// Simple UUID
user_id: UUID

// Shared UUID across records
$shared_id: UUID::()
record1: { id: $shared_id }
record2: { id: $shared_id }
```

#### LoremIpsum
Generates Lorem Ipsum text with configurable word count.

**Configuration:**
- `min_words: u32` - Minimum number of words (required)
- `max_words: u32` - Maximum number of words (required)

**PartiQL Type:** `STRING`

**Examples:**
```ion
// Short description
summary: LoremIpsum::{ min_words: 5, max_words: 15 }

// Long content
article: LoremIpsum::{ min_words: 100, max_words: 500 }

// Single sentence
title: LoremIpsum::{ min_words: 3, max_words: 8 }
```

#### LoremIpsumTitle
Generates title-cased Lorem Ipsum text (3-8 words).

**Configuration:** None

**PartiQL Type:** `STRING`

**Examples:**
```ion
// Article titles
title: LoremIpsumTitle

// Product names
product_name: LoremIpsumTitle
```

#### Regex
Generates strings matching a regular expression pattern.

**Configuration:**
- `pattern: String` - Regular expression pattern (required)

**PartiQL Type:** `STRING`

**Examples:**
```ion
// Phone numbers
phone: Regex::{ pattern: "\\+1-[0-9]{3}-[0-9]{3}-[0-9]{4}" }

// Product codes
sku: Regex::{ pattern: "[A-Z]{3}-[0-9]{4}" }

// Email addresses
email: Regex::{ pattern: "[a-z]{5,10}@[a-z]{3,8}\\.(com|org|net)" }

// Postal codes
zip: Regex::{ pattern: "[0-9]{5}(-[0-9]{4})?" }
```

**Note:** Use double backslashes (`\\`) for escape sequences in Ion strings.

### Temporal Generators

#### Instant
Generates the current simulation timestamp.

**Configuration:** None

**PartiQL Type:** `DATETIME`

**Examples:**
```ion
// Current timestamp
created_at: Instant

// Multiple timestamps in same record
record: {
    created: Instant,
    updated: Instant  // Will be same as created within same record
}
```

#### Date
Generates the current simulation date.

**Configuration:** None

**PartiQL Type:** `DATETIME`

**Examples:**
```ion
// Current date
birth_date: Date
event_date: Date
```

#### Timestamp
Generates timestamps with configurable precision and timezone.

**Configuration:**
- `timezone: bool` - Include timezone information (optional)
- `precision: String` - Time precision: "microsecond", "millisecond", "second", "minute", "hour", "day" (optional)

**PartiQL Type:** `DATETIME`

**Examples:**
```ion
// High precision with timezone
precise_time: Timestamp::{ timezone: true, precision: "microsecond" }

// Date only
date_only: Timestamp::{ precision: "day" }

// Hour precision
hourly: Timestamp::{ precision: "hour" }
```

#### Tick
Generates the current simulation tick counter.

**Configuration:** None

**PartiQL Type:** `INT64`

**Examples:**
```ion
// Sequence number
sequence: Tick

// Event counter
event_id: Tick
```

### Choice Generators

#### Uniform
Generates values by uniformly choosing from a list of literals.

**Configuration:**
- `choices: [Literal]` - Array of Ion literals to choose from (required)

**PartiQL Type:** `Union` (type of chosen literals)

**Examples:**
```ion
// String choices
status: Uniform::{ choices: ["active", "inactive", "pending"] }

// Numeric choices
priority: Uniform::{ choices: [1, 2, 3, 4, 5] }

// Mixed types
value: Uniform::{ choices: [42, "unknown", true, null] }

// Complex values
config: Uniform::{ choices: [
    { mode: "fast", timeout: 100 },
    { mode: "slow", timeout: 1000 }
]}
```

## Complex Generators

### Array Generators

#### UniformArray
Generates arrays with uniform size distribution.

**Configuration:**
- `min_size: u64` - Minimum array size (required)
- `max_size: u64` - Maximum array size (required)
- `element_type: DataGenerator` - Generator for array elements (required)

**PartiQL Type:** `ARRAY<T>` where T is the element type

**Examples:**
```ion
// Array of strings
tags: UniformArray::{ 
    min_size: 1, 
    max_size: 5, 
    element_type: LoremIpsumTitle 
}

// Array of numbers
scores: UniformArray::{ 
    min_size: 3, 
    max_size: 10, 
    element_type: UniformF64::{ low: 0.0, high: 100.0 }
}

// Array of UUIDs
related_ids: UniformArray::{ 
    min_size: 0, 
    max_size: 3, 
    element_type: UUID 
}
```

### Union Generators

#### UniformAnyOf
Generates values by uniformly choosing from multiple data generators.

**Configuration:**
- `types: [DataGenerator]` - Array of data generators to choose from (required)

**PartiQL Type:** `Union` of all generator types

**Examples:**
```ion
// Mixed numeric types
value: UniformAnyOf::{ types: [
    UniformI32::{ low: 1, high: 100 },
    UniformF64::{ low: 0.0, high: 1.0 },
    UniformDecimal::{ low: 1.99, high: 99.99 }
]}

// Mixed data types
content: UniformAnyOf::{ types: [
    LoremIpsumTitle,
    UniformI32::{ low: 1, high: 1000 },
    Bool,
    UUID
]}

// Different object structures
event: UniformAnyOf::{ types: [
    { type: "click", x: UniformI32, y: UniformI32 },
    { type: "scroll", delta: UniformI32 },
    { type: "key", code: UniformU8 }
]}
```

### Format Generators

#### Format
Generates formatted strings using templates.

**Configuration:**
- `pattern: String` - Format pattern with placeholders (required)

**PartiQL Type:** `STRING`

**Examples:**
```ion
// Customer names with numbers
customer: Format::{ pattern: "customer #{ $@n }" }

// Formatted addresses
address: Format::{ pattern: "{ $street_num } { $street_name } { $street_type }" }

// Log messages
log_message: Format::{ pattern: "[{ $level }] { $timestamp }: { $message }" }
```

## Nullability and Optionality

All generators support nullability and optionality configuration:

### Nullability

Controls generation of `NULL` values:

```ion
// Default: nullable but 0% chance of NULL
name: LoremIpsumTitle::{ nullable: true }

// Not nullable
id: UUID::{ nullable: false }

// 10% chance of NULL
description: LoremIpsum::{ nullable: 0.1, min_words: 10, max_words: 50 }
```

### Optionality

Controls generation of `MISSING` values:

```ion
// Default: not optional
required_field: UUID::{ optional: false }

// Optional but 0% chance of MISSING
optional_field: LoremIpsumTitle::{ optional: true }

// 20% chance of MISSING
maybe_field: UniformI32::{ optional: 0.2, low: 1, high: 100 }
```

### Global Defaults

Control default nullability and optionality via CLI:

```bash
# Make all fields nullable by default with 5% NULL rate
cargo run gen data --default-nullable true --pct-null 0.05 --script my-script.ion

# Make all fields optional by default with 10% MISSING rate
cargo run gen data --default-optional true --pct-optional 0.1 --script my-script.ion
```

## Reserved Variables

PartiQL Beamline provides several reserved variables:

### Tick
Current simulation tick (increments with each generated sample).

```ion
sequence_number: Tick
```

### Instant
Current simulation timestamp.

```ion
timestamp: Instant
```

## Best Practices

### Choosing Distributions

1. **Uniform**: Use for IDs, categories, or when all values are equally likely
2. **Normal**: Use for natural measurements (height, weight, test scores)
3. **Exponential**: Use for wait times, time between events
4. **Log-Normal**: Use for sizes, prices, income distributions
5. **Weibull**: Use for reliability analysis, lifetime modeling

### Performance Considerations

1. **String Generators**: `UUID` is faster than `Regex` for identifiers
2. **Array Sizes**: Large arrays with complex elements can be slow
3. **Regex Complexity**: Simple patterns are much faster than complex ones
4. **Distribution Choice**: Uniform distributions are fastest

### Memory Usage

1. **Large Arrays**: Consider streaming for very large arrays
2. **String Length**: Long Lorem Ipsum text uses more memory
3. **Complex Structures**: Deeply nested structures increase memory usage

### Reproducibility

1. **Variable Evaluation**: Use `::()` for values that should be constant across records
2. **Shared Generators**: Define generators as variables for consistency
3. **Seed Management**: Use consistent seeds for reproducible datasets

## Common Patterns

### Related Data

```ion
rand_processes::{
    $user_id: UUID,
    $success_rate: UniformF64::{ low: 0.95, high: 1.0 },
    
    events: rand_process::{
        $data: {
            user_id: $user_id::(),  // Same for all records
            success: Bool::{ p: $success_rate },
            timestamp: Instant
        }
    }
}
```

### Realistic Distributions

```ion
// E-commerce order values (log-normal distribution)
order_value: LogNormalF64::{ location: 3.0, scale: 1.0 }

// Response times (exponential distribution)
response_time: ExpF64::{ rate: 0.01 }

// User ratings (normal distribution, clamped)
rating: NormalF64::{ mean: 4.2, std_dev: 0.8 }
```

### Conditional Generation

```ion
rand_processes::{
    $is_premium: Bool::{ p: 0.1 },
    
    users: rand_process::{
        $data: {
            premium: $is_premium,
            // Higher limits for premium users
            api_limit: UniformAnyOf::{ types: [
                UniformI32::{ low: 1000, high: 5000 },    // Premium
                UniformI32::{ low: 100, high: 500 }       // Regular
            ]}
        }
    }
}
```

This reference should serve as your go-to guide when building PartiQL Beamline scripts. For more complex examples and patterns, see the [Examples and Tutorials](../examples/) section.
