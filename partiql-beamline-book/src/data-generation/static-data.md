# Static Data Generation

Static data in PartiQL Beamline refers to data that is generated once at the beginning of the simulation, before any temporal events occur. This is useful for creating reference tables, lookup data, or any information that doesn't change over the course of your simulation.

## What is Static Data?

**Static data** is generated using `static_data` blocks instead of `rand_process` blocks. Key differences:

- **Generated once**: All static data is created at simulation time 0
- **No arrival process**: No `$arrival` configuration needed
- **Reference data**: Often used for lookup tables, master data, configuration
- **Shared across processes**: Can be referenced by multiple dynamic processes

## Basic Syntax

```ion
dataset_name: static_data::{
    $data: {
        // Generator configuration (same as rand_process)
        field1: GeneratorType,
        field2: GeneratorType::{ configuration }
    }
}
```

## Static vs Dynamic Data

### Dynamic Data (rand_process)

```ion
orders: rand_process::{
    $arrival: HomogeneousPoisson::{ interarrival: days::5 },
    $data: {
        order_id: UUID,
        timestamp: Instant,
        amount: UniformDecimal::{ low: 10.00, high: 500.00 }
    }
}
```

**Characteristics:**
- Generated over simulation time
- Each record has different timestamps
- Follows arrival process (Poisson, uniform, etc.)

### Static Data (static_data)

```ion
product_catalog: static_data::{
    $data: {
        product_id: UUID,
        name: LoremIpsumTitle,
        base_price: UniformDecimal::{ low: 5.00, high: 200.00 }
    }
}
```

**Characteristics:**
- Generated all at once at time 0
- All records have the same timestamp (simulation start time)
- No arrival process needed

## Real Example: Customer and Orders

From the `orders.ion` test script, here's how static and dynamic data work together:

```ion
rand_processes::{
    // Generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },

    // Shared generators
    $id_gen: UUID,
    $oid_gen: UUID,

    customers: $n::[
        {
            // Each customer gets a unique ID 
            $id: $id_gen::(),

            // Static customer data - generated once per customer
            customer_table: static_data::{
                $data: {
                    id: $id,
                    address: Format::{ pattern: "{$@n} Foo Bar Ave" }
                }
            },

            // Dynamic order data - generated over time
            orders: rand_process::{
                $r: UniformU8::{ low: 1, high: 150 },
                $arrival: HomogeneousPoisson::{ interarrival: days::$r },
                $data: {
                    Order: $oid_gen,
                    Time: Instant,
                    Customer: $id  // References the same ID
                }
            }
        }
    ]
}
```

When executed, this generates:

**Static Data (all at simulation start):**
```
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'address': '0 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': '179e600a-c1c5-8ac2-05b6-15b20f8fe740', 'address': '1 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'address': '2 Foo Bar Ave', 'id': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0' }
```

**Dynamic Data (spread over time):**
```
[2019-08-01 7:26:21.964 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '4c579e42-8c70-93f4-b99b-cc45c50197ed' }
[2019-08-10 5:46:15.24 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '38900593-e9cc-994a-98d9-0becf77d9144' }
[2019-08-11 7:27:49.565 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'b2aa0efc-dac3-b391-f4c2-3c298e0c99f4' }
```

Notice how:
- All `customer_table` records have the same timestamp (simulation start)
- The `orders` records are distributed over time with different timestamps
- Both datasets share the same customer IDs, creating referential relationships

## Use Cases for Static Data

### Reference Tables

Create lookup tables that don't change during simulation:

```ion
rand_processes::{
    // Static product catalog
    products: static_data::{
        $data: {
            product_id: UUID,
            name: LoremIpsumTitle,
            category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home"] },
            base_price: UniformDecimal::{ low: 5.00, high: 500.00 }
        }
    },

    // Dynamic orders referencing products
    orders: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::30 },
        $data: {
            order_id: UUID,
            // Note: In real usage, you'd want to reference actual product IDs
            product_category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home"] },
            timestamp: Instant
        }
    }
}
```

### Configuration Data

Generate system configuration that remains constant:

```ion
rand_processes::{
    // System configuration - static
    config: static_data::{
        $data: {
            system_id: UUID,
            version: Uniform::{ choices: ["1.0", "1.1", "2.0"] },
            max_connections: UniformU16::{ low: 100, high: 1000 },
            timeout_seconds: UniformU8::{ low: 30, high: 300 }
        }
    },

    // Application events - dynamic
    events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::10 },
        $data: {
            event_id: UUID,
            event_type: Uniform::{ choices: ["login", "logout", "action", "error"] },
            timestamp: Instant
        }
    }
}
```

### User Profiles and Activity

Create user profiles once, then generate their activities over time:

```ion
rand_processes::{
    $n: UniformU8::{ low: 10, high: 50 },  // 10-50 users
    $id_gen: UUID,

    users: $n::[
        {
            $user_id: $id_gen::(),  // One ID per user

            // Static user profile
            user_profiles: static_data::{
                $data: {
                    user_id: $user_id,
                    username: Format::{ pattern: "user_{$@n}" },
                    email: Format::{ pattern: "user{$@n}@example.com" },
                    registration_date: Date,
                    plan_type: Uniform::{ choices: ["free", "premium", "enterprise"] }
                }
            },

            // Dynamic user activity
            user_activity: rand_process::{
                $r: UniformU8::{ low: 30, high: 180 },  // 30-180 minutes between actions
                $arrival: HomogeneousPoisson::{ interarrival: minutes::$r },
                $data: {
                    user_id: $user_id,
                    action_type: Uniform::{ choices: ["view", "click", "purchase", "search"] },
                    timestamp: Instant,
                    session_id: UUID
                }
            }
        }
    ]
}
```

## Time-Related Generators in Static Data

When using time-related generators in static data, they use the simulation start time:

### Instant and Date in Static Data

```ion
rand_processes::{
    // System startup data
    system_info: static_data::{
        $data: {
            system_id: UUID,
            startup_time: Instant,      // Will be simulation start time
            startup_date: Date,         // Will be simulation start date
            boot_tick: Tick,            // Will be 0 (initial tick)
            version: "1.0.0"
        }
    }
}
```

**Output Example:**
```
[2024-01-01 00:00:00.000 +00:00] : "system_info" { 
    'system_id': '123e4567-e89b-12d3-a456-426614174000',
    'startup_time': 2024-01-01T00:00:00.000000000+00:00,
    'startup_date': 2024-01-01T00:00:00.000000000+00:00,
    'boot_tick': 0,
    'version': '1.0.0'
}
```

## Multiple Static Datasets

You can create multiple static datasets in the same script:

```ion
rand_processes::{
    // Company information
    companies: static_data::{
        $data: {
            company_id: UUID,
            name: Format::{ pattern: "Company {UUID}" },
            industry: Uniform::{ choices: ["Tech", "Finance", "Retail", "Healthcare"] }
        }
    },

    // Department information  
    departments: static_data::{
        $data: {
            dept_id: UUID,
            name: Uniform::{ choices: ["Engineering", "Sales", "Marketing", "HR"] },
            budget: UniformDecimal::{ low: 50000.00, high: 2000000.00 }
        }
    },

    // Employee events - references both static datasets
    employee_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::8 },
        $data: {
            employee_id: UUID,
            event_type: Uniform::{ choices: ["hire", "promotion", "transfer", "resignation"] },
            timestamp: Instant,
            // Note: In real usage, you'd reference actual company/dept IDs
            company_type: Uniform::{ choices: ["Tech", "Finance", "Retail", "Healthcare"] },
            department: Uniform::{ choices: ["Engineering", "Sales", "Marketing", "HR"] }
        }
    }
}
```

## Static Data with Variables and Loops

Create multiple static datasets using loops:

```ion
rand_processes::{
    $n: UniformU8::{ low: 3, high: 8 },  // 3-8 regions
    $region_id: UUID,

    regions: $n::[
        {
            $id: $region_id::(),  // Unique ID per region

            // Static region data
            'region_{$@n}': static_data::{
                $data: {
                    region_id: $id,
                    region_name: Format::{ pattern: "Region {$@n}" },
                    timezone: Uniform::{ choices: ["UTC-8", "UTC-5", "UTC", "UTC+1"] },
                    population: UniformU32::{ low: 100000, high: 10000000 }
                }
            }
        }
    ]
}
```

This creates multiple static datasets like `region_0`, `region_1`, `region_2`, etc.

## Complex Static Data Structures

Static data supports all the same generators as dynamic data:

```ion
rand_processes::{
    // Complex static configuration
    system_config: static_data::{
        $data: {
            config_id: UUID,
            created_at: Instant,
            
            // Nested configuration
            database: {
                host: Regex::{ pattern: "db[0-9]{2}\\.example\\.com" },
                port: UniformU16::{ low: 5432, high: 5439 },
                ssl_enabled: Bool::{ p: 0.9 }
            },
            
            // Array of server configurations
            servers: UniformArray::{
                min_size: 3,
                max_size: 10,
                element_type: {
                    server_id: UUID,
                    hostname: Regex::{ pattern: "server[0-9]{3}\\.example\\.com" },
                    cpu_cores: Uniform::{ choices: [4, 8, 16, 32] },
                    memory_gb: Uniform::{ choices: [16, 32, 64, 128] }
                }
            },
            
            // Mixed type configuration
            features: UniformAnyOf::{
                types: [
                    Bool,
                    UniformI32::{ low: 1, high: 100 },
                    LoremIpsumTitle
                ]
            }
        }
    }
}
```

## Static Data Best Practices

### 1. Use for Reference Data

```ion
// Good - static reference data
product_categories: static_data::{
    $data: {
        category_id: UUID,
        name: Uniform::{ choices: ["Electronics", "Books", "Clothing"] },
        tax_rate: UniformDecimal::{ low: 0.05, high: 0.10 }
    }
}

// Avoid - frequently changing data should be dynamic
```

### 2. Share IDs Between Static and Dynamic

```ion
rand_processes::{
    $customer_id: UUID,
    
    customers: 5::[
        {
            $id: $customer_id::(),  // Generate once per customer
            
            // Static profile
            customer_profiles: static_data::{
                $data: {
                    customer_id: $id,
                    name: LoremIpsumTitle,
                    email: Format::{ pattern: "customer{$@n}@example.com" }
                }
            },
            
            // Dynamic transactions
            transactions: rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: days::10 },
                $data: {
                    customer_id: $id,  // Same ID
                    transaction_id: UUID,
                    amount: UniformDecimal::{ low: 10.00, high: 1000.00 }
                }
            }
        }
    ]
}
```

### 3. Use Meaningful Static Data

```ion
// Good - realistic static data
countries: static_data::{
    $data: {
        country_code: Regex::{ pattern: "[A-Z]{2}" },
        country_name: LoremIpsumTitle,
        population: LogNormalF64::{ location: 15.0, scale: 2.0 },  // Realistic population distribution
        gdp_per_capita: LogNormalF64::{ location: 8.5, scale: 1.5 }
    }
}

// Avoid - unrealistic or meaningless static data
```

### 4. Consider Static Data Size

```ion
rand_processes::{
    // Small static dataset - appropriate
    currencies: static_data::{
        $data: {
            currency_code: Regex::{ pattern: "[A-Z]{3}" },
            exchange_rate: UniformF64::{ low: 0.1, high: 10.0 }
        }
    }
}
```

For large reference datasets, consider if the data really needs to be static or could be part of a slow-changing dynamic process.

## Output Characteristics

### CLI Output Format

When you run data generation, static data appears first with identical timestamps:

```bash
$ beamline gen data \
    --seed 1234 \
    --start-iso "2019-08-01T00:00:01-07:00" \
    --script-path partiql-beamline-sim/tests/scripts/orders.ion \
    --sample-count 10 \
    --output-format text

Seed: 1234
Start: 2019-08-01T00:00:01.000000000-07:00

# Static data first (all at start time)
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'address': '0 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': '179e600a-c1c5-8ac2-05b6-15b20f8fe740', 'address': '1 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'address': '2 Foo Bar Ave', 'id': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0' }

# Dynamic data follows (spread over time)
[2019-08-01 7:26:21.964 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '4c579e42-8c70-93f4-b99b-cc45c50197ed' }
[2019-08-10 5:46:15.24 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '38900593-e9cc-994a-98d9-0becf77d9144' }
```

### Ion Pretty Format

```bash
$ beamline gen data \
    --seed 1234 \
    --start-auto \
    --script-path with_static.ion \
    --sample-count 5 \
    --output-format ion-pretty

{
  seed: 1234,
  start: "2024-01-01T00:00:00Z",
  data: {
    // Static data grouped together
    config: [
      {
        system_id: "123e4567-e89b-12d3-a456-426614174000",
        version: "1.0",
        created_at: 2024-01-01T00:00:00Z
      }
    ],
    
    // Dynamic data grouped together
    events: [
      {
        event_id: "987fcdeb-51a2-43d1-9f4e-123456789abc",
        timestamp: 2024-01-01T00:05:23Z,
        type: "user_login"
      },
      {
        event_id: "456789ab-cdef-1234-5678-9abcdef01234", 
        timestamp: 2024-01-01T00:08:45Z,
        type: "user_action"
      }
    ]
  }
}
```

## Common Patterns

### Master Data Pattern

```ion
rand_processes::{
    // Static master data
    locations: static_data::{
        $data: {
            location_id: UUID,
            city: LoremIpsumTitle,
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            latitude: UniformF64::{ low: -90.0, high: 90.0 },
            longitude: UniformF64::{ low: -180.0, high: 180.0 }
        }
    },

    // Events at locations
    weather_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::6 },
        $data: {
            event_id: UUID,
            // In real usage, would reference actual location_id
            temperature: NormalF64::{ mean: 20.0, std_dev: 10.0 },
            humidity: UniformF64::{ low: 20.0, high: 90.0 },
            timestamp: Instant
        }
    }
}
```

### Hierarchical Data Pattern

```ion
rand_processes::{
    $n_orgs: UniformU8::{ low: 2, high: 5 },
    $org_id: UUID,

    organizations: $n_orgs::[
        {
            $id: $org_id::(),

            // Static organization info
            'org_{$@n}': static_data::{
                $data: {
                    org_id: $id,
                    org_name: Format::{ pattern: "Organization {$@n}" },
                    industry: Uniform::{ choices: ["Tech", "Finance", "Healthcare"] },
                    founded_year: UniformU16::{ low: 1950, high: 2020 }
                }
            },

            // Dynamic organizational events
            'org_events_{$@n}': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: days::30 },
                $data: {
                    org_id: $id,
                    event_type: Uniform::{ choices: ["hire", "fire", "restructure", "acquisition"] },
                    timestamp: Instant,
                    impact_score: NormalF64::{ mean: 5.0, std_dev: 2.0 }
                }
            }
        }
    ]
}
```

## Database Generation with Static Data

When creating databases with `gen db beamline-lite`, static data creates separate dataset files:

```bash
$ beamline gen db beamline-lite \
    --seed 1000 \
    --start-auto \
    --script-path partiql-beamline-sim/tests/scripts/orders.ion \
    --sample-count 1000

$ tree beamline-catalog/
beamline-catalog/
├── .beamline-manifest
├── .beamline-script
├── customer_table.ion        # Static data
├── customer_table.shape.ion  # Static data schema
├── customer_table.shape.sql  # Static data SQL schema
├── orders.ion               # Dynamic data
├── orders.shape.ion         # Dynamic data schema
└── orders.shape.sql         # Dynamic data SQL schema
```

**Static data file (`customer_table.ion`):**
```ion
{id: "abc-123", address: "0 Main St"}
{id: "def-456", address: "1 Main St"}
{id: "ghi-789", address: "2 Main St"}
```

**Dynamic data file (`orders.ion`):**
```ion
{Customer: "abc-123", Order: "order-001", Time: 2024-01-01T00:15:30Z}
{Customer: "def-456", Order: "order-002", Time: 2024-01-01T01:22:15Z}
{Customer: "abc-123", Order: "order-003", Time: 2024-01-01T02:08:45Z}
```

## Performance Implications

### Memory Usage

- **Static data** is generated once and stored in memory during generation
- **Large static datasets** may increase memory usage
- **Consider data size** when designing static datasets

### Generation Speed

- **Static generation** happens once at startup
- **No temporal computation** needed for static data
- **Overall faster** than equivalent dynamic data

### Best Practices for Large Static Data

```ion
// If you need large reference data, consider dynamic with very slow arrival
// Instead of large static data:
large_reference: static_data::{ /* ... thousands of records ... */ }

// Consider slow dynamic process:
reference_data: rand_process::{
    $arrival: HomogeneousPoisson::{ interarrival: days::365 },  // Very infrequent
    $data: { /* ... */ }
}
```

## Troubleshooting Static Data

### Issue: Static Data Not Appearing

**Cause**: No sample count affects static data - it's always generated based on script configuration.

**Solution**: Check your script syntax and variable definitions.

### Issue: Unexpected Timestamps

**Cause**: All static data uses simulation start time.

**Solution**: This is expected behavior. Use dynamic processes for time-distributed data.

### Issue: Large Memory Usage

**Cause**: Large static datasets are loaded into memory.

**Solution**: Reduce static dataset size or convert to slow dynamic processes.

## Examples from Test Scripts

### Simple Static Configuration

```ion
// From a test script pattern
config: static_data::{
    $data: {
        app_version: "2.1.0",
        max_users: UniformU32::{ low: 1000, high: 10000 },
        feature_flags: UniformAnyOf::{
            types: [Bool, UniformI32::{ low: 0, high: 100 }]
        }
    }
}
```

### Multi-Dataset Static Pattern

```ion
rand_processes::{
    $n: UniformU8::{ low: 5, high: 15 },
    
    servers: $n::[
        {
            'server_config_{$@n}': static_data::{
                $data: {
                    server_id: Format::{ pattern: "server-{$@n}" },
                    hostname: Format::{ pattern: "srv{$@n}.example.com" },
                    ip_address: Regex::{ pattern: "192\\.168\\.[0-9]{1,3}\\.[0-9]{1,3}" },
                    capacity: Uniform::{ choices: [100, 200, 500, 1000] }
                }
            }
        }
    ]
}
```

## Next Steps

- [Datasets](./datasets.md) - Learn about working with multiple datasets and relationships
- [Output Formats](./output-formats.md) - Understand how static data appears in different formats
- [Scripts](./scripts.md) - Advanced Ion scripting techniques with static and dynamic data
- [Examples](../examples/sensors.md) - See static data in complete examples
