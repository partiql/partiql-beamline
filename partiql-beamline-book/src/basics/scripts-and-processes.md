# Scripts and Random Processes

PartiQL Beamline uses **Ion-based scripts** to define data generation configurations and **stochastic processes** to model how data arrives and evolves over time. This combination provides powerful, flexible control over synthetic data generation.

## Ion Scripts Overview

### What are Ion Scripts?

Ion scripts are configuration files written in [Amazon Ion](https://amazon-ion.github.io/ion-docs/) format that define:
- **What data to generate** (data types, structures, values)  
- **How data arrives** (temporal patterns, frequencies)
- **How data relates** (cross-field dependencies, correlations)
- **How much data** (counts, durations, stopping conditions)

### Basic Script Structure

Every PartiQL Beamline script follows this structure:

```ion
rand_processes::{
    // Variable definitions (optional)
    $variable_name: GeneratorType::{ configuration },
    
    // Dataset definitions (required)
    dataset_name: rand_process::{
        $arrival: ArrivalProcess::{ configuration },
        $data: {
            field_name: GeneratorType::{ configuration },
            // ... more fields
        }
    }
}
```

### Real Example from Test Suite

From `sensors.ion` test script:

```ion
rand_processes::{
    $n: UniformU8::{ low: 2, high: 10 },

    sensors: $n::[
        rand_process::{
            $r: Uniform::{ choices: [5,10] },
            $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
            $weight: UniformDecimal::{ nullable: 0.75, low: 1.995, high: 4.9999, optional: true },
            $data: {
                tick: Tick,
                i8: UniformI8,
                f: UniformF64,
                w: $weight,
                d: UniformDecimal::{ low: 0d0, high: 4.2d1, nullable: false }
            }
        }
    ]
}
```

### Ion Format Benefits

Ion provides several advantages for configuration:
- **Type Safety**: Native support for numbers, strings, booleans, timestamps
- **Comments**: Document your configuration inline with `//`
- **Annotations**: Add type annotations like `minutes::$r`
- **Nested Structures**: Define complex object hierarchies naturally
- **Variable References**: Use `$variable` for reusable components

## Stochastic Processes

### What are Stochastic Processes?

**Stochastic processes** are mathematical models that describe how events occur over time in a seemingly random but statistically predictable way. In Beamline, they're defined using the `$arrival` field in `rand_process` blocks.

### Arrival Process Types

#### 1. Homogeneous Poisson Process

Models events that occur at a constant average rate with random intervals:

```ion
rand_processes::{
    sensor_readings: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            sensor_id: UUID,
            reading: UniformF64::{ low: 0.0, high: 100.0 },
            timestamp: Instant
        }
    }
}
```

**Time Units:**
- `milliseconds::N` - N milliseconds between events
- `seconds::N` - N seconds between events
- `minutes::N` - N minutes between events  
- `hours::N` - N hours between events
- `days::N` - N days between events

**Use Cases:**
- User logins to a website
- Network packet arrivals
- Customer service calls
- Sensor readings

#### 2. Variable Arrival Rates

Use variables to create dynamic arrival patterns:

```ion
rand_processes::{
    user_events: rand_process::{
        $r: Uniform::{ choices: [2, 5, 10] },  // Variable rate
        $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
        $data: {
            event_type: Uniform::{ choices: ["login", "logout", "action"] },
            user_id: UUID
        }
    }
}
```

## Data Generators

### Basic Generator Types

From the actual implementation:

#### Numeric Generators

```ion
rand_processes::{
    numeric_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            // Integer generators
            small_int: UniformI8,                                    // -127 to 127
            medium_int: UniformI16::{ low: 100, high: 1000 },       // Custom range
            large_int: UniformU32::{ low: 1, high: 1000000 },       // Unsigned
            
            // Float generators  
            decimal_value: UniformDecimal::{ low: 1.99, high: 99.99 },  // Exact decimal
            float_value: UniformF64::{ low: 0.0, high: 1.0 },           // Float
            
            // Statistical distributions
            normal_score: NormalF64::{ mean: 100.0, std_dev: 15.0 },
            exponential_wait: ExpF64::{ rate: 0.1 },
            weibull_lifetime: WeibullF64::{ shape: 2.0, scale: 1000.0 }
        }
    }
}
```

#### String Generators

```ion
rand_processes::{
    text_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::2 },
        $data: {
            // UUID generator
            id: UUID,
            
            // Lorem Ipsum text
            description: LoremIpsum::{ min_words: 5, max_words: 20 },
            title: LoremIpsumTitle,  // 3-8 title-cased words
            
            // Regular expressions
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            phone: Regex::{ pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}" },
            
            // Format strings with variables
            formatted_name: Format::{ pattern: "User #{UUID}" }
        }
    }
}
```

#### System Generators

```ion
rand_processes::{
    system_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            // System state generators
            current_time: Instant,      // Current simulation time
            current_date: Date,         // Current simulation date
            event_tick: Tick,           // Current tick counter
            
            // Boolean generator
            active: Bool,               // 50% true by default
            premium: Bool::{ p: 0.1 }   // 10% true
        }
    }
}
```

#### Complex Type Generators

```ion
rand_processes::{
    complex_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::5 },
        $data: {
            // Array generator
            measurements: UniformArray::{
                min_size: 3,
                max_size: 8,
                element_type: UniformF64::{ low: 0.0, high: 100.0 }
            },
            
            // Union type generator (any of several types)
            mixed_value: UniformAnyOf::{
                types: [
                    UUID,
                    UniformI32::{ low: 1, high: 1000 },
                    LoremIpsumTitle
                ]
            },
            
            // Choice from literals
            status: Uniform::{ choices: ["active", "inactive", "pending"] }
        }
    }
}
```

## Advanced Script Features

### Variable Definitions and References

From the real client-service.ion script:

```ion
rand_processes::{
    // Define reusable generators
    $n: UniformU8::{ low: 5, high: 20 },
    $id_gen: UUID,
    $rid_gen: UUID,
    
    requests: $n::[
        {
            // Force evaluation at script read time
            $id: $id_gen::(),
            $rate: UniformF64::{ low: 0.995e0, high: 1.0e0 },
            $success: Bool::{ p: $rate },
            
            service: rand_process::{
                $r: UniformU8::{ low: 20, high: 150 },
                $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },
                $data: {
                    Request: $rid_gen,
                    Account: $id,
                    client: Format::{ pattern: "customer #{ $@n }" },
                    success: $success
                }
            }
        }
    ]
}
```

**Key concepts:**
- **Variables**: `$variable_name` for reusable generators
- **Forced evaluation**: `$id_gen::()` evaluates once at script read time
- **Loop arrays**: `$n::[...]` creates N instances
- **Loop index**: `$@n` accesses current iteration index

### Static Data

From the orders.ion test script:

```ion
rand_processes::{
    $n: UniformU8::{ low: 5, high: 20 },
    $id_gen: UUID,
    
    customers: $n::[
        {
            $id: $id_gen::(),
            
            // Static data - generated once at simulation start
            customer_table: static_data::{
                $data: {
                    id: $id,
                    address: Format::{ pattern: "{ $@n } Foo Bar Ave" }
                }
            },
            
            // Dynamic data - generated over time
            orders: rand_process::{
                $arrival: HomogeneousPoisson:: { interarrival: days::UniformU8::{ low: 1, high: 150 } },
                $data: {
                    Order: UUID,
                    Customer: $id,
                    Time: Instant
                }
            }
        }
    ]
}
```

### Nullability and Optionality

Real syntax from test scripts:

```ion
rand_processes::{
    nullable_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            // 75% chance of NULL, can also be MISSING
            weight: UniformDecimal::{ 
                nullable: 0.75, 
                optional: true,
                low: 1.995, 
                high: 4.9999 
            },
            
            // Never NULL
            id: UUID::{ nullable: false },
            
            // 10% chance of MISSING (field won't appear)
            optional_field: UniformI32::{ optional: 0.1, low: 1, high: 100 }
        }
    }
}
```

## Real Script Examples

### Simple Sensor Script

Based on the actual sensors.ion test:

```ion
rand_processes::{
    $n: UniformU8::{ low: 2, high: 10 },

    sensors: $n::[
        rand_process::{
            $r: Uniform::{ choices: [5,10] },
            $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
            $weight: UniformDecimal::{ nullable: 0.75, low: 1.995, high: 4.9999, optional: true },
            $data: {
                tick: Tick,
                i8: UniformI8,
                f: UniformF64,
                w: $weight,
                d: UniformDecimal::{ low: 0d0, high: 4.2d1, nullable: false }
            }
        }
    ]
}
```

**Test this script:**
```bash
target/release/beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path partiql-beamline-sim/tests/scripts/sensors.ion \
  --sample-count 10 \
  --output-format ion-pretty
```

### Client-Service System

Based on client-service.ion test:

```ion
rand_processes::{
    // Generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },

    // Shared generators
    $id_gen: UUID,
    $rid_gen: UUID,

    requests: $n::[
        {
            // Each customer gets unique ID
            $id: $id_gen::(),
            $rate: UniformF64::{ low: 0.995e0, high: 1.0e0 },
            $success: Bool::{ p: $rate },

            // Service dataset
            service: rand_process::{
                $r: UniformU8::{ low: 20, high: 150 },
                $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },
                $data: {
                    Request: $rid_gen,
                    StartTime: Instant,
                    Program: "FancyService",
                    Operation: "GetMyData",
                    Account: $id,
                    client: Format::{ pattern: "customer #{ $@n }" },
                    success: $success
                }
            },

            // Individual client datasets
            'client_{ $@n }': rand_process::{
                $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },
                $data: {
                    id: $id,
                    request_time: Instant,
                    request_id: $rid_gen,
                    success: $success
                }
            }
        }
    ]
}
```

### Transaction Data Script

Based on simple_transactions.ion test:

```ion
rand_processes::{
    test_data: rand_process::{
        $r: Uniform::{ choices: [5,10] },
        $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },
        
        $data: {
            transaction_id: UUID::{ nullable: false },
            marketplace_id: UniformU8::{ nullable: false },
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            created_at: Instant,
            completed: Bool,
            description: LoremIpsum::{ min_words:10, max_words:200 },
            price: UniformDecimal::{ low: 2.99, high: 99999.99, optional: true }
        }
    }
}
```

## Advanced Script Patterns

### Complex Statistical Distributions

From numbers.ion test script:

```ion
rand_processes::{
    test_data: rand_process::{
        $r: Uniform::{ choices: [5,10] },
        $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

        $data: {
            uniform: {
                // Uniform distributions
                uniform_u8: UniformU8::{ low: 13, high: 42 },
                uniform_f64: UniformF64::{ low: -13.0, high: 42.0 },
                uniform_decimal: UniformDecimal::{ low: 0.995, high: 499.9999 }
            },

            statistical: {
                // Statistical distributions
                normal: NormalF64::{ mean: 14.3, std_dev: 3.0 },
                lognormal: LogNormalF64::{ location: 14.3, scale: 3.0 },
                weibull: WeibullF64::{ shape: 14.3, scale: 3.0 },
                exponential: ExpF64::{ rate: 3.0 }
            },
            
            // With nullability and optionality
            nullable_field: UniformI32::{ 
                nullable: 0.2,    // 20% NULL
                optional: 0.1,    // 10% MISSING
                low: 1, 
                high: 100 
            }
        }
    }
}
```

### Multiple Datasets with Relationships

Real pattern from client-service.ion:

```ion
rand_processes::{
    $n: UniformU8::{ low: 5, high: 20 },
    $id_gen: UUID,

    requests: $n::[
        {
            $id: $id_gen::(),  // One ID per customer
            $rid_gen: UUID,    // Separate request ID generator per customer
            
            // Shared service dataset
            service: rand_process::{
                $arrival: HomogeneousPoisson:: { interarrival: milliseconds::50 },
                $data: {
                    Request: $rid_gen,
                    StartTime: Instant,
                    Account: $id,
                    client: Format::{ pattern: "customer #{ $@n }" }
                }
            },
            
            // Individual client dataset for this customer
            'client_{ $@n }': rand_process::{
                $arrival: HomogeneousPoisson:: { interarrival: milliseconds::50 },
                $data: {
                    id: $id,
                    request_time: Instant,
                    request_id: $rid_gen
                }
            }
        }
    ]
}
```

### Static Data with Dynamic References

From orders.ion test script:

```ion
rand_processes::{
    $n: UniformU8::{ low: 5, high: 20 },
    $id_gen: UUID,
    $oid_gen: UUID,

    customers: $n::[
        {
            $id: $id_gen::(),

            // Static customer data (generated once)
            customer_table: static_data::{
                $data: {
                    id: $id,
                    address: Format::{ pattern: "{ $@n } Foo Bar Ave" }
                }
            },

            // Dynamic orders (generated over time)  
            orders: rand_process::{
                $r: UniformU8::{ low: 1, high: 150 },
                $arrival: HomogeneousPoisson:: { interarrival: days::$r },
                $data: {
                    Order: $oid_gen,
                    Time: Instant,
                    Customer: $id  // Links to customer_table
                }
            }
        }
    ]
}
```

## Script Testing and Validation

### Testing Script Syntax

```bash
# Test script with minimal data generation
target/release/beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 1

# Check inferred schema
target/release/beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --output-format basic-ddl
```

### Testing with Small Samples

```bash
# Test each dataset individually
target/release/beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path complex_script.ion \
  --sample-count 5 \
  --dataset specific_dataset

# Test all datasets with small sample
target/release/beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path complex_script.ion \
  --sample-count 5 \
  --output-format text
```

## Best Practices

### 1. Use Real Test Script Patterns

```ion
// Good - follows actual PartiQL Beamline syntax
rand_processes::{
    $arrival_rate: Uniform::{ choices: [5, 10] },
    
    events: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::$arrival_rate },
        $data: {
            event_id: UUID,
            timestamp: Instant,
            value: UniformF64::{ low: 0.0, high: 100.0 }
        }
    }
}
```

### 2. Test Scripts Incrementally

```bash
# Start with basic structure
echo 'rand_processes::{ test: rand_process::{ $arrival: HomogeneousPoisson:: { interarrival: seconds::1 }, $data: { id: UUID } } }' > minimal.ion

# Test basic structure
target/release/beamline gen data --seed 1 --start-auto --script-path minimal.ion --sample-count 3
```

### 3. Use Meaningful Variable Names

```ion
rand_processes::{
    // Clear variable names
    $customer_count: UniformU8::{ low: 10, high: 50 },
    $order_frequency: Uniform::{ choices: [1, 3, 7] },  // Days
    $customer_id_generator: UUID,
    
    orders: $customer_count::[
        rand_process::{
            $arrival: HomogeneousPoisson:: { interarrival: days::$order_frequency },
            $data: {
                customer_id: $customer_id_generator,
                order_time: Instant
            }
        }
    ]
}
```

### 4. Document Complex Patterns

```ion
rand_processes::{
    // === Customer Simulation Configuration ===
    // Generate 10-50 customers, each placing orders every 1-30 days
    
    $customer_count: UniformU8::{ low: 10, high: 50 },
    $shared_customer_id: UUID,
    
    customer_orders: $customer_count::[
        {
            // Each customer gets unique ID for all their orders
            $id: $shared_customer_id::(),
            
            // Customer places orders with variable frequency
            orders: rand_process::{
                $days_between_orders: UniformU8::{ low: 1, high: 30 },
                $arrival: HomogeneousPoisson:: { interarrival: days::$days_between_orders },
                $data: {
                    customer_id: $id,
                    order_id: UUID,
                    order_time: Instant,
                    amount: UniformDecimal::{ low: 10.00, high: 500.00 }
                }
            }
        }
    ]
}
```

## Common Script Errors and Solutions

### Error: Invalid Ion Syntax

```ion
// Wrong - missing closing brace
rand_processes::{
    test: rand_process::{
        $data: { id: UUID }
    // Missing closing brace for rand_processes
```

### Error: Missing Required Fields

```ion
// Wrong - missing $arrival
rand_processes::{
    test: rand_process::{
        $data: { id: UUID }  // Missing $arrival definition
    }
}

// Correct
rand_processes::{
    test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: { id: UUID }
    }
}
```

### Error: Invalid Generator Configuration

```ion
// Wrong - low > high
rand_processes::{
    test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            bad_range: UniformI32::{ low: 100, high: 50 }  // Invalid
        }
    }
}

// Correct
rand_processes::{
    test: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            good_range: UniformI32::{ low: 50, high: 100 }
        }
    }
}
```

## Performance Optimization

### Efficient Generator Usage

```ion
rand_processes::{
    // Efficient - reuse expensive generators
    $expensive_distribution: NormalF64::{ mean: 100.0, std_dev: 15.0 },
    $simple_uuid: UUID,
    
    efficient_data: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: seconds::1 },
        $data: {
            // Reuse expensive distribution
            score1: $expensive_distribution,
            score2: $expensive_distribution,
            score3: $expensive_distribution,
            
            // Simple generators are fast
            id: $simple_uuid,
            active: Bool,
            count: UniformI32::{ low: 1, high: 1000 }
        }
    }
}
```

### Testing Commands

```bash
# Test with small samples first
target/release/beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 5 \
  --output-format text

# Scale up after validation
target/release/beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 10000 \
  --output-format ion-binary
```

## Next Steps

Now that you understand real Ion scripts and stochastic processes, you're ready to dive deeper into the [Data Generation](../data-generation/overview.md) section, where you'll learn about specific generator types, output formats, and advanced data modeling techniques using the actual PartiQL Beamline syntax.
