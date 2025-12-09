# Datasets and Collections

Datasets in PartiQL Beamline represent collections of related data records that share the same structure. Understanding how to design, organize, and work with multiple datasets is essential for creating realistic data generation scenarios.

## What are Datasets?

A **dataset** is a named collection of records that share a common schema. In Ion scripts, datasets are defined as top-level keys within the `rand_processes` structure:

```ion
rand_processes::{
    users: rand_process::{ /* ... */ },        // "users" dataset
    orders: rand_process::{ /* ... */ },       // "orders" dataset  
    products: static_data::{ /* ... */ }       // "products" dataset
}
```

Each dataset becomes a separate data collection in the output, whether in text format, Ion format, or database generation.

## Single Dataset Scripts

### Basic Single Dataset

```ion
rand_processes::{
    sensors: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },
        $data: {
            sensor_id: UUID,
            temperature: NormalF64::{ mean: 22.0, std_dev: 3.0 },
            humidity: UniformF64::{ low: 30.0, high: 80.0 },
            timestamp: Instant
        }
    }
}
```

**Output characteristics:**
- Single dataset named "sensors"
- All records have the same structure
- Records generated according to arrival process

## Multiple Dataset Scripts

### Independent Datasets

Create multiple unrelated datasets in the same script:

```ion
rand_processes::{
    // User activity dataset
    user_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
        $data: {
            user_id: UUID,
            event_type: Uniform::{ choices: ["login", "logout", "click", "purchase"] },
            timestamp: Instant
        }
    },

    // System metrics dataset  
    system_metrics: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::60 },
        $data: {
            metric_name: Uniform::{ choices: ["cpu", "memory", "disk", "network"] },
            value: UniformF64::{ low: 0.0, high: 100.0 },
            timestamp: Instant
        }
    },

    // Configuration dataset (static)
    app_config: static_data::{
        $data: {
            config_key: Uniform::{ choices: ["max_users", "timeout", "retry_count"] },
            config_value: UniformAnyOf::{ types: [UniformI32::{ low: 1, high: 1000 }, Bool] }
        }
    }
}
```

### Related Datasets with Shared Variables

Create datasets that share common identifiers or generators:

```ion
rand_processes::{
    // Shared generators
    $user_id: UUID,
    $session_id: UUID,

    // User profiles (static)
    users: static_data::{
        $data: {
            user_id: $user_id,
            username: Format::{ pattern: "user_{UUID}" },
            created_at: Date
        }
    },

    // User sessions (dynamic)
    sessions: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::2 },
        $data: {
            session_id: $session_id,
            user_id: $user_id,  // Links to users dataset
            start_time: Instant,
            duration_minutes: UniformU16::{ low: 5, high: 180 }
        }
    },

    // Session events (dynamic) 
    session_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::3 },
        $data: {
            event_id: UUID,
            session_id: $session_id,  // Links to sessions dataset
            event_type: Uniform::{ choices: ["page_view", "click", "scroll", "exit"] },
            timestamp: Instant
        }
    }
}
```

## Complex Dataset Relationships

### Dynamic Dataset Creation with Loops

From the real `client-service.ion` test script:

```ion
rand_processes::{
    // Generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },

    // Shared ID generators
    $id_gen: UUID,
    $rid_gen: UUID,

    requests: $n::[
        // Each iteration creates datasets for customer $@n
        {
            // Unique ID per customer
            $id: $id_gen::(),
            $rate: UniformF64::{ low: 0.995, high: 1.0 },
            $success: Bool::{ p: $rate },

            // Service dataset - shared by all customers
            service: rand_process::{
                $r: UniformU8::{ low: 20, high: 150 },
                $arrival: HomogeneousPoisson::{ interarrival: milliseconds::$r },
                $data: {
                    Request: $rid_gen,
                    StartTime: Instant,
                    Program: "FancyService", 
                    Operation: "GetMyData",
                    Account: $id,
                    client: Format::{ pattern: "customer #{$@n}" },
                    success: $success
                }
            },

            // Individual client dataset - one per customer
            'client_{$@n}': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: milliseconds::$r },
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

This creates:
- **1 service dataset**: Shared across all customers
- **N client datasets**: `client_0`, `client_1`, `client_2`, etc.
- **Shared variables**: Same request IDs, customer IDs, success rates

### Output Example

```bash
$ partiql-beamline-cli gen data \
    --seed 100 \
    --start-auto \
    --script-path client-service.ion \
    --sample-count 20 \
    --output-format text

Seed: 100
Start: 2024-01-01T00:00:00Z
[2024-01-01 00:00:10.123] : "service" { 'Request': 'req-001', 'Account': 'customer-abc', 'client': 'customer #0' }
[2024-01-01 00:00:10.124] : "client_0" { 'id': 'customer-abc', 'request_id': 'req-001' }
[2024-01-01 00:00:15.456] : "service" { 'Request': 'req-002', 'Account': 'customer-def', 'client': 'customer #1' }
[2024-01-01 00:00:15.457] : "client_1" { 'id': 'customer-def', 'request_id': 'req-002' }
```

## Dataset Filtering

### CLI Dataset Selection

Generate data for specific datasets only:

```bash
# Generate all datasets
partiql-beamline-cli gen data \
  --seed 42 \
  --start-auto \
  --script-path multi_dataset.ion \
  --sample-count 100

# Generate only specific datasets
partiql-beamline-cli gen data \
  --seed 42 \
  --start-auto \
  --script-path multi_dataset.ion \
  --sample-count 100 \
  --dataset users \
  --dataset orders

# Generate only one dataset
partiql-beamline-cli gen data \
  --seed 42 \
  --start-auto \
  --script-path multi_dataset.ion \
  --sample-count 100 \
  --dataset system_metrics
```

### Use Cases for Dataset Filtering

- **Focused testing**: Test specific components in isolation
- **Performance optimization**: Generate only needed data
- **Development**: Work with subset of complex systems
- **Incremental development**: Build datasets one at a time

## Dataset Design Patterns

### Master-Detail Pattern

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 10, high: 50 },
    $customer_id: UUID,
    $order_id: UUID,

    customers: $n_customers::[
        {
            $id: $customer_id::(),

            // Master dataset - customer information
            customer_master: static_data::{
                $data: {
                    customer_id: $id,
                    name: LoremIpsumTitle,
                    email: Format::{ pattern: "customer{$@n}@example.com" },
                    registration_date: Date
                }
            },

            // Detail dataset - customer orders
            'customer_{$@n}_orders': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: days::UniformU8::{ low: 1, high: 30 } },
                $data: {
                    order_id: $order_id,
                    customer_id: $id,  // Foreign key relationship
                    order_date: Instant,
                    total_amount: UniformDecimal::{ low: 10.00, high: 500.00 }
                }
            }
        }
    ]
}
```

### Event Sourcing Pattern

```ion
rand_processes::{
    $entity_id: UUID,

    // Entity snapshots (static)
    entity_snapshots: static_data::{
        $data: {
            entity_id: $entity_id,
            entity_type: Uniform::{ choices: ["user", "order", "product"] },
            created_at: Date,
            initial_state: LoremIpsumTitle
        }
    },

    // Entity events (dynamic)
    entity_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::UniformU8::{ low: 5, high: 60 } },
        $data: {
            event_id: UUID,
            entity_id: $entity_id,  // Links to snapshots
            event_type: Uniform::{ choices: ["created", "updated", "deleted", "restored"] },
            timestamp: Instant,
            event_data: LoremIpsum::{ min_words: 5, max_words: 20 }
        }
    }
}
```

### Multi-Tenant Pattern

```ion
rand_processes::{
    $n_tenants: UniformU8::{ low: 3, high: 10 },
    $tenant_id: UUID,

    tenants: $n_tenants::[
        {
            $id: $tenant_id::(),

            // Tenant configuration (static)
            'tenant_{$@n}_config': static_data::{
                $data: {
                    tenant_id: $id,
                    tenant_name: Format::{ pattern: "Tenant {$@n}" },
                    plan: Uniform::{ choices: ["basic", "premium", "enterprise"] },
                    max_users: UniformU16::{ low: 10, high: 1000 }
                }
            },

            // Tenant activity (dynamic)
            'tenant_{$@n}_activity': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: minutes::UniformU8::{ low: 1, high: 30 } },
                $data: {
                    activity_id: UUID,
                    tenant_id: $id,
                    activity_type: Uniform::{ choices: ["login", "api_call", "data_export", "config_change"] },
                    timestamp: Instant,
                    user_count: UniformU16::{ low: 1, high: 100 }
                }
            }
        }
    ]
}
```

## Dataset Analysis and Inspection

### Examining Generated Datasets

```bash
# Generate multi-dataset output
partiql-beamline-cli gen data \
  --seed 123 \
  --start-auto \
  --script-path complex_system.ion \
  --sample-count 1000 \
  --output-format ion-pretty > output.ion

# Extract dataset names and record counts
jq -r '.data | keys[]' output.ion  # List all dataset names
jq '.data.users | length' output.ion  # Count records in users dataset
jq '.data | to_entries[] | "\(.key): \(.value | length) records"' output.ion  # All counts
```

### Database Catalog Analysis

```bash
# Generate database
partiql-beamline-cli gen db beamline-lite \
  --seed 456 \
  --start-auto \
  --script-path multi_dataset.ion \
  --sample-count 5000

# Analyze generated datasets
ls -la beamline-catalog/*.ion | grep -v shape  # List data files
for f in beamline-catalog/*.ion; do
  if [[ "$f" != *".shape.ion" ]]; then
    echo "$(basename "$f" .ion): $(wc -l < "$f") records"
  fi
done
```

### Schema Comparison Across Datasets

```bash
# Compare schemas of related datasets
diff beamline-catalog/client_0.shape.sql beamline-catalog/client_1.shape.sql
# Should be identical for datasets created from same template

# Compare different dataset schemas
diff beamline-catalog/users.shape.sql beamline-catalog/orders.shape.sql
# Should be different - different structures
```

## Advanced Dataset Patterns

### Hierarchical Data Modeling

```ion
rand_processes::{
    $n_orgs: UniformU8::{ low: 2, high: 5 },
    $n_depts_per_org: UniformU8::{ low: 3, high: 8 },
    $n_users_per_dept: UniformU8::{ low: 5, high: 20 },

    organizations: $n_orgs::[
        {
            $org_id: UUID::(),

            // Organization master data
            'org_{$@n}': static_data::{
                $data: {
                    org_id: $org_id,
                    org_name: Format::{ pattern: "Organization {$@n}" },
                    industry: Uniform::{ choices: ["Tech", "Finance", "Healthcare", "Retail"] }
                }
            },

            // Departments within organization
            departments: $n_depts_per_org::[
                {
                    $dept_id: UUID::(),

                    'org_{$@n}_dept_{$@n}': static_data::{
                        $data: {
                            dept_id: $dept_id,
                            org_id: $org_id,
                            dept_name: Uniform::{ choices: ["Engineering", "Sales", "Marketing", "HR"] }
                        }
                    },

                    // Users within department
                    'org_{$@n}_dept_{$@n}_users': $n_users_per_dept::[
                        rand_process::{
                            $arrival: HomogeneousPoisson::{ interarrival: hours::UniformU8::{ low: 8, high: 24 } },
                            $data: {
                                user_id: UUID,
                                dept_id: $dept_id,
                                org_id: $org_id,
                                activity_type: Uniform::{ choices: ["work", "meeting", "break", "training"] },
                                timestamp: Instant
                            }
                        }
                    ]
                }
            ]
        }
    ]
}
```

### Time-Series Dataset Families

```ion
rand_processes::{
    $n_sensors: UniformU8::{ low: 5, high: 15 },
    $sensor_id: UUID,

    sensors: $n_sensors::[
        {
            $id: $sensor_id::(),
            $location: Format::{ pattern: "Location-{$@n}" },

            // Sensor metadata (static)
            'sensor_{$@n}_metadata': static_data::{
                $data: {
                    sensor_id: $id,
                    location: $location,
                    sensor_type: Uniform::{ choices: ["temperature", "humidity", "pressure"] },
                    calibration_date: Date
                }
            },

            // Regular sensor readings (dynamic)
            'sensor_{$@n}_readings': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
                $data: {
                    sensor_id: $id,
                    reading_time: Instant,
                    value: NormalF64::{ mean: 22.0, std_dev: 5.0 },
                    quality: Uniform::{ choices: ["good", "fair", "poor"] }
                }
            },

            // Sensor alerts (dynamic, infrequent)
            'sensor_{$@n}_alerts': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: hours::UniformU8::{ low: 6, high: 48 } },
                $data: {
                    alert_id: UUID,
                    sensor_id: $id,
                    alert_type: Uniform::{ choices: ["high_value", "low_value", "malfunction", "maintenance"] },
                    timestamp: Instant,
                    severity: Uniform::{ choices: [1, 2, 3, 4, 5] }
                }
            }
        }
    ]
}
```

## Dataset Output in Different Formats

### Text Format Multi-Dataset Output

```bash
$ partiql-beamline-cli gen data \
    --seed 999 \
    --start-auto \
    --script-path multi_dataset.ion \
    --sample-count 20 \
    --output-format text

# Datasets are interleaved by timestamp
[2024-01-01 00:00:00.000] : "config" { 'key': 'timeout', 'value': 30 }
[2024-01-01 00:00:00.000] : "config" { 'key': 'max_users', 'value': 1000 }
[2024-01-01 00:02:15.123] : "users" { 'user_id': 'abc-123', 'action': 'login' }
[2024-01-01 00:03:45.456] : "metrics" { 'metric': 'cpu', 'value': 45.6 }
[2024-01-01 00:04:30.789] : "users" { 'user_id': 'def-456', 'action': 'click' }
```

### Ion Pretty Multi-Dataset Output

```ion
{
  seed: 999,
  start: "2024-01-01T00:00:00Z",
  data: {
    config: [
      { key: "timeout", value: 30 },
      { key: "max_users", value: 1000 }
    ],
    users: [
      { user_id: "abc-123", action: "login", timestamp: 2024-01-01T00:02:15.123Z },
      { user_id: "def-456", action: "click", timestamp: 2024-01-01T00:04:30.789Z }
    ],
    metrics: [
      { metric: "cpu", value: 45.6, timestamp: 2024-01-01T00:03:45.456Z }
    ]
  }
}
```

### Database Generation Multi-Dataset Files

```bash
$ partiql-beamline-cli gen db beamline-lite \
    --seed 42 \
    --start-auto \
    --script-path client_service.ion \
    --sample-count 1000

$ ls beamline-catalog/
.beamline-manifest
.beamline-script
service.ion              # Service dataset data
service.shape.ion        # Service dataset schema  
service.shape.sql        # Service dataset SQL
client_0.ion            # Client 0 dataset data
client_0.shape.ion      # Client 0 dataset schema
client_0.shape.sql      # Client 0 dataset SQL
client_1.ion            # Client 1 dataset data
client_1.shape.ion      # Client 1 dataset schema
client_1.shape.sql      # Client 1 dataset SQL
...                     # More client datasets
```

## Dataset Naming Best Practices

### 1. Use Descriptive Names

```ion
// Good - descriptive dataset names
user_profiles: static_data::{ /* ... */ },
user_activity_events: rand_process::{ /* ... */ },
system_performance_metrics: rand_process::{ /* ... */ }

// Avoid - generic names
data1: static_data::{ /* ... */ },
stuff: rand_process::{ /* ... */ }
```

### 2. Follow Consistent Naming Conventions

```ion
// Consistent naming pattern
user_profiles: static_data::{ /* ... */ },
user_sessions: rand_process::{ /* ... */ },
user_events: rand_process::{ /* ... */ },

order_master: static_data::{ /* ... */ },
order_items: rand_process::{ /* ... */ },
order_payments: rand_process::{ /* ... */ }
```

### 3. Use Meaningful Prefixes for Related Datasets

```ion
// Group related datasets with prefixes
$n: UniformU8::{ low: 5, high: 10 },

services: $n::[
    {
        'service_{$@n}_config': static_data::{ /* ... */ },
        'service_{$@n}_requests': rand_process::{ /* ... */ },
        'service_{$@n}_responses': rand_process::{ /* ... */ },
        'service_{$@n}_errors': rand_process::{ /* ... */ }
    }
]
```

## Performance Considerations

### Dataset Count Impact

- **Few datasets (1-5)**: Minimal overhead
- **Many datasets (10-50)**: Slight memory overhead for tracking
- **Dynamic datasets (100+)**: Significant memory for metadata

### Dataset Size Balance

```ion
// Balanced approach - mix of small and large datasets
rand_processes::{
    // Small reference dataset
    config: static_data::{ $data: { /* small config */ } },

    // Medium operational dataset  
    users: rand_process::{ /* moderate activity */ },

    // Large transaction dataset
    transactions: rand_process::{ /* high frequency */ }
}
```

### Memory Usage with Multiple Datasets

```bash
# Monitor memory usage with many datasets
time partiql-beamline-cli gen data \
  --seed 1 \
  --start-auto \
  --script-path many_datasets.ion \
  --sample-count 10000

# Use dataset filtering to reduce memory
partiql-beamline-cli gen data \
  --seed 1 \
  --start-auto \
  --script-path many_datasets.ion \
  --sample-count 10000 \
  --dataset important_dataset_only
```

## Integration Workflows

### Dataset-Specific Processing

```bash
#!/bin/bash
# process-datasets.sh

SCRIPT="multi_system.ion"
SEED=12345

# Generate full dataset
partiql-beamline-cli gen data \
  --seed $SEED \
  --start-auto \
  --script-path $SCRIPT \
  --sample-count 10000 \
  --output-format ion-pretty > full_data.ion

# Extract individual datasets for processing
jq '.data.users' full_data.ion > users_only.json
jq '.data.orders' full_data.ion > orders_only.json  
jq '.data.metrics' full_data.ion > metrics_only.json

echo "Datasets extracted for individual processing"
```

### Cross-Dataset Validation

```bash
# Generate related datasets
partiql-beamline-cli gen data \
  --seed 999 \
  --start-auto \
  --script-path related_data.ion \
  --sample-count 5000 \
  --output-format ion-pretty > related_data.ion

# Validate relationships
jq '.data.orders[].customer_id' related_data.ion | sort -u > order_customers.txt
jq '.data.users[].user_id' related_data.ion | sort -u > all_customers.txt

# Check referential integrity
comm -23 order_customers.txt all_customers.txt  # Orders with invalid customer IDs (should be empty)
```

## Troubleshooting Multi-Dataset Scripts

### Issue: Missing Datasets in Output

**Cause**: Dataset filtering or script errors

**Solution**: 
```bash
# Check all available datasets
partiql-beamline-cli infer-shape --seed 1 --start-auto --script-path script.ion --output-format text

# Generate without filtering
partiql-beamline-cli gen data --seed 1 --start-auto --script-path script.ion --sample-count 5
```

### Issue: Uneven Dataset Sizes

**Cause**: Different arrival rates or loop counts

**Solution**: 
```bash
# Check arrival rates in your script
# Adjust interarrival times to balance dataset sizes
$arrival1: HomogeneousPoisson::{ interarrival: seconds::1 },   # Frequent
$arrival2: HomogeneousPoisson::{ interarrival: minutes::1 },   # Less frequent
```

### Issue: Memory Issues with Many Datasets

**Solution**:
```bash
# Use dataset filtering
partiql-beamline-cli gen data --script-path many.ion --dataset important_one --dataset important_two

# Or generate datasets separately
partiql-beamline-cli gen data --script-path script.ion --dataset batch_1 --sample-count 10000
partiql-beamline-cli gen data --script-path script.ion --dataset batch_2 --sample-count 10000
```

## Next Steps

- [Scripts](./scripts.md) - Advanced Ion scripting techniques for complex datasets
- [Output Formats](./output-formats.md) - How datasets appear in different output formats
- [Examples](../examples/sensors.md) - See complete multi-dataset examples in action
- [Database Guide](../database/overview.md) - Working with dataset catalogs and databases
