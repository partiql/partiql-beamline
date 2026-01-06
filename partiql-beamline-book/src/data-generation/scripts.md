# Working with Scripts

Ion scripts are the core of Beamline's data generation system. This section covers advanced scripting techniques, best practices, and patterns for creating sophisticated data generation scenarios.

## Ion Script Fundamentals

### Basic Script Structure

Every Beamline script follows this structure:

```ion
rand_processes::{
    // 1. Variable definitions (optional)
    $variable_name: GeneratorType::{ configuration },
    
    // 2. Dataset definitions (required)
    dataset_name: dataset_type::{
        // Configuration specific to dataset type
    }
}
```

### Script Validation

Before generating large datasets, validate your script:

```bash
# Quick validation with minimal generation
beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --sample-count 1

# Check inferred schema
beamline infer-shape \
  --seed 1 \
  --start-auto \
  --script-path new_script.ion \
  --output-format basic-ddl
```

## Variable Management

### Variable Definition Best Practices

```ion
rand_processes::{
    // Group related variables together with comments
    // === ID Generators ===
    $user_id: UUID,
    $session_id: UUID, 
    $transaction_id: UUID,
    
    // === Shared Distributions ===
    $age_distribution: NormalF64::{ mean: 35.0, std_dev: 12.0 },
    $price_range: UniformDecimal::{ low: 9.99, high: 999.99 },
    
    // === Configuration Values ===
    $max_users: UniformU8::{ low: 10, high: 50 },
    $success_rate: UniformF64::{ low: 0.95, high: 0.99 },
    
    // === Categorical Choices ===
    $status_options: Uniform::{ choices: ["active", "inactive", "pending", "suspended"] },
    $priority_levels: Uniform::{ choices: [1, 2, 3, 4, 5] },
    
    // Dataset definitions follow...
}
```

### Variable Scoping Rules

Variables have different scoping behaviors:

```ion
rand_processes::{
    // Global variable - accessible everywhere
    $global_id: UUID,
    
    dataset: $n::[
        {
            // Loop-scoped variable - unique per iteration
            $local_id: UUID::(),  // Forces evaluation per loop iteration
            
            'data_{$@n}': rand_process::{
                $data: {
                    global: $global_id,      // Same value across all loops
                    local: $local_id,        // Different per loop iteration  
                    index: '$@n'             // Current loop index
                }
            }
        }
    ]
}
```

### Advanced Variable Techniques

#### Computed Variables

```ion
rand_processes::{
    // Base measurements
    $base_temp: NormalF64::{ mean: 20.0, std_dev: 3.0 },
    $temp_variance: UniformF64::{ low: 0.5, high: 2.0 },
    
    // Computed distributions based on other variables
    $adjusted_temp: NormalF64::{ 
        mean: 22.0,  // Slightly higher than base
        std_dev: 4.0 // More variation
    },
    
    sensors: rand_process::{
        $data: {
            base_temperature: $base_temp,
            adjusted_temperature: $adjusted_temp,
            temperature_diff: UniformF64::{ low: -5.0, high: 5.0 }
        }
    }
}
```

#### Conditional Variable Usage

```ion
rand_processes::{
    // Define multiple generators for different scenarios
    $high_value_price: UniformDecimal::{ low: 100.00, high: 1000.00 },
    $low_value_price: UniformDecimal::{ low: 1.00, high: 50.00 },
    $medium_value_price: UniformDecimal::{ low: 25.00, high: 200.00 },
    
    products: rand_process::{
        $data: {
            product_id: UUID,
            category: Uniform::{ choices: ["electronics", "books", "clothing"] },
            
            // Use different price generators for different scenarios
            price: UniformAnyOf::{
                types: [
                    $high_value_price,    // Electronics
                    $low_value_price,     // Books  
                    $medium_value_price   // Clothing
                ]
            }
        }
    }
}
```

## Advanced Script Patterns

### Multi-Level Hierarchies

```ion
rand_processes::{
    $n_regions: UniformU8::{ low: 2, high: 4 },
    $n_stores_per_region: UniformU8::{ low: 3, high: 8 },
    $n_employees_per_store: UniformU8::{ low: 5, high: 20 },

    retail_hierarchy: $n_regions::[
        {
            $region_id: UUID::(),
            
            // Region data
            'region_{$@n}': static_data::{
                $data: {
                    region_id: $region_id,
                    region_name: Format::{ pattern: "Region {$@n}" },
                    timezone: Uniform::{ choices: ["PST", "MST", "CST", "EST"] }
                }
            },

            // Stores in region
            stores: $n_stores_per_region::[
                {
                    $store_id: UUID::(),
                    
                    'region_{$@n}_store_{$@n}': static_data::{
                        $data: {
                            store_id: $store_id,
                            region_id: $region_id,
                            store_name: Format::{ pattern: "Store {$@n}-{$@n}" },
                            address: Format::{ pattern: "{$@n} Commerce St" }
                        }
                    },

                    // Employees in store
                    'region_{$@n}_store_{$@n}_employees': $n_employees_per_store::[
                        rand_process::{
                            $arrival: HomogeneousPoisson::{ interarrival: hours::8 },
                            $data: {
                                employee_id: UUID,
                                store_id: $store_id,
                                region_id: $region_id,
                                clock_in_time: Instant,
                                activity: Uniform::{ choices: ["sales", "inventory", "cleaning", "break"] }
                            }
                        }
                    ]
                }
            ]
        }
    ]
}
```

### Time-Based Dataset Coordination

```ion
rand_processes::{
    // Shared timing variables
    $peak_hours_rate: HomogeneousPoisson::{ interarrival: minutes::2 },
    $off_hours_rate: HomogeneousPoisson::{ interarrival: minutes::15 },
    $maintenance_rate: HomogeneousPoisson::{ interarrival: hours::6 },
    
    // High-frequency events during peak hours
    peak_user_activity: rand_process::{
        $arrival: $peak_hours_rate,
        $data: {
            event_id: UUID,
            event_type: Uniform::{ choices: ["login", "search", "purchase"] },
            timestamp: Instant,
            load_factor: UniformF64::{ low: 0.7, high: 1.0 }  // High load
        }
    },
    
    // Lower frequency during off hours  
    off_hours_activity: rand_process::{
        $arrival: $off_hours_rate,
        $data: {
            event_id: UUID,
            event_type: Uniform::{ choices: ["backup", "cleanup", "monitoring"] },
            timestamp: Instant,
            load_factor: UniformF64::{ low: 0.1, high: 0.3 }  // Low load
        }
    },
    
    // Maintenance events
    maintenance_events: rand_process::{
        $arrival: $maintenance_rate,
        $data: {
            maintenance_id: UUID,
            maintenance_type: Uniform::{ choices: ["scheduled", "emergency", "upgrade"] },
            timestamp: Instant,
            duration_minutes: UniformU16::{ low: 30, high: 240 }
        }
    }
}
```

### Cross-Dataset Correlation

```ion
rand_processes::{
    // Shared correlation factors
    $system_load: UniformF64::{ low: 0.1, high: 0.9 },
    $error_probability: Bool::{ p: 0.05 },  // 5% base error rate
    
    // System metrics affected by load
    system_metrics: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
        $data: {
            metric_id: UUID,
            timestamp: Instant,
            cpu_usage: $system_load,
            memory_usage: UniformF64::{ low: 0.2, high: 0.8 },
            response_time_ms: LogNormalF64::{ location: 2.0, scale: 0.5 }
        }
    },
    
    // Application events affected by same factors
    application_events: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::10 },
        $data: {
            event_id: UUID,
            timestamp: Instant,
            event_type: Uniform::{ choices: ["request", "response", "error", "timeout"] },
            has_error: $error_probability,  // Correlated error rate
            load_factor: $system_load       // Same load factor
        }
    }
}
```

## Script Organization Strategies

### Modular Script Design

```ion
rand_processes::{
    // === CONFIGURATION SECTION ===
    // System-wide settings
    $system_version: "2.1.0",
    $max_concurrent_users: UniformU16::{ low: 100, high: 1000 },
    
    // === SHARED GENERATORS ===  
    // Reusable ID generators
    $user_id: UUID,
    $session_id: UUID,
    $request_id: UUID,
    
    // Reusable distributions
    $user_age_dist: NormalF64::{ mean: 34.5, std_dev: 12.8 },
    $response_time_dist: LogNormalF64::{ location: 3.0, scale: 0.4 },
    
    // === REFERENCE DATA ===
    // Static lookup tables
    user_types: static_data::{
        $data: {
            type_id: UniformU8::{ low: 1, high: 5 },
            type_name: Uniform::{ choices: ["free", "premium", "enterprise", "admin", "guest"] },
            max_sessions: Uniform::{ choices: [1, 5, 10, 100, 1] }
        }
    },
    
    // === OPERATIONAL DATA ===
    // Dynamic user activity
    user_sessions: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::UniformU8::{ low: 2, high: 30 } },
        $data: {
            user_id: $user_id,
            session_id: $session_id,
            start_time: Instant,
            user_age: $user_age_dist
        }
    },
    
    // === PERFORMANCE DATA ===
    // System performance metrics
    performance_metrics: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::15 },
        $data: {
            metric_timestamp: Instant,
            response_time: $response_time_dist,
            concurrent_users: UniformU16::{ low: 0, high: 1000 }
        }
    }
}
```

### Environment-Specific Scripts

Create scripts that can be configured for different environments:

```ion
rand_processes::{
    // === ENVIRONMENT CONFIGURATION ===
    // Development environment settings
    $dev_user_count: UniformU8::{ low: 5, high: 20 },
    $dev_load_factor: UniformF64::{ low: 0.1, high: 0.3 },
    $dev_error_rate: 0.1,  // 10% errors in dev
    
    // Production-like environment settings  
    $prod_user_count: UniformU16::{ low: 100, high: 1000 },
    $prod_load_factor: UniformF64::{ low: 0.6, high: 0.95 },
    $prod_error_rate: 0.01,  // 1% errors in prod
    
    // Use dev settings (change as needed)
    $current_user_count: $dev_user_count,
    $current_load_factor: $dev_load_factor,
    $current_error_rate: $dev_error_rate,
    
    // === DATASETS ===
    users: $current_user_count::[
        rand_process::{
            $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },
            $data: {
                user_id: UUID,
                load_impact: $current_load_factor,
                has_error: Bool::{ p: $current_error_rate },
                timestamp: Instant
            }
        }
    ]
}
```

## Complex Data Relationships

### Foreign Key Relationships

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 10, high: 50 },
    $n_products: UniformU8::{ low: 20, high: 100 },
    
    // Generate customer IDs we can reference
    $customer_ids: $n_customers::[UUID::()],  // Array of customer UUIDs
    $product_ids: $n_products::[UUID::()],    // Array of product UUIDs
    
    customers: static_data::{
        $data: {
            customer_id: Uniform::{ choices: $customer_ids },  // Reference predefined IDs
            name: LoremIpsumTitle,
            email: Format::{ pattern: "customer{UUID}@example.com" }
        }
    },
    
    products: static_data::{
        $data: {
            product_id: Uniform::{ choices: $product_ids },   // Reference predefined IDs
            name: LoremIpsumTitle,
            price: UniformDecimal::{ low: 5.00, high: 200.00 }
        }
    },
    
    orders: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::UniformU8::{ low: 5, high: 30 } },
        $data: {
            order_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },  // Valid customer reference
            product_id: Uniform::{ choices: $product_ids },    // Valid product reference  
            quantity: UniformU8::{ low: 1, high: 5 },
            timestamp: Instant
        }
    }
}
```

### Temporal Coordination

```ion
rand_processes::{
    // Shared timing patterns
    $business_hours: HomogeneousPoisson::{ interarrival: minutes::UniformU8::{ low: 2, high: 10 } },
    $after_hours: HomogeneousPoisson::{ interarrival: hours::UniformU8::{ low: 1, high: 4 } },
    
    // Customer activity during business hours
    customer_activity: rand_process::{
        $arrival: $business_hours,
        $data: {
            activity_id: UUID,
            activity_type: Uniform::{ choices: ["browse", "search", "purchase", "support"] },
            timestamp: Instant,
            response_time: LogNormalF64::{ location: 2.5, scale: 0.3 }  // Faster during business hours
        }
    },
    
    // System maintenance after hours
    system_maintenance: rand_process::{
        $arrival: $after_hours,
        $data: {
            maintenance_id: UUID,
            maintenance_type: Uniform::{ choices: ["backup", "update", "cleanup", "monitoring"] },
            timestamp: Instant,
            duration_minutes: UniformU16::{ low: 15, high: 120 }
        }
    }
}
```

## Script Testing and Development

### Iterative Development Process

```bash
# 1. Start with minimal script
echo 'rand_processes::{ test: rand_process::{ $arrival: HomogeneousPoisson::{ interarrival: seconds::1 }, $data: { id: UUID } } }' > minimal.ion

# 2. Validate basic structure
beamline gen data --seed 1 --start-auto --script-path minimal.ion --sample-count 3

# 3. Add complexity incrementally
# ... edit script to add fields, variables, etc.

# 4. Test each addition
beamline gen data --seed 1 --start-auto --script-path enhanced.ion --sample-count 5

# 5. Validate schema
beamline infer-shape --seed 1 --start-auto --script-path enhanced.ion --output-format basic-ddl
```

### Script Debugging Techniques

#### Add Debug Fields

```ion
rand_processes::{
    test_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::5 },
        $data: {
            // Production fields
            user_id: UUID,
            action: Uniform::{ choices: ["login", "logout"] },
            
            // Debug fields (remove in production)
            debug_tick: Tick,
            debug_timestamp: Instant,
            debug_seed_info: Format::{ pattern: "Generated at tick {Tick}" }
        }
    }
}
```

#### Validate Variable Evaluation

```ion
rand_processes::{
    // Test variable evaluation
    $test_var: UniformI32::{ low: 1, high: 10 },
    $forced_eval: UniformI32::{ low: 100, high: 200 }::(),
    
    debug_variables: rand_process::{
        $data: {
            normal_var: $test_var,      // New value each time
            forced_var: $forced_eval,   // Same value each time
            comparison: Format::{ pattern: "normal: {$test_var}, forced: {$forced_eval}" }
        }
    }
}
```

#### Test Script Fragments

```bash
# Test individual components
echo 'rand_processes::{ test_generators: rand_process::{ $arrival: HomogeneousPoisson::{ interarrival: seconds::1 }, $data: { test_field: NormalF64::{ mean: 0.0, std_dev: 1.0 } } } }' | \
beamline gen data --seed 1 --start-auto --script - --sample-count 5
```

## Performance Optimization in Scripts

### Generator Efficiency

```ion
rand_processes::{
    // Efficient - simple generators
    efficient_data: rand_process::{
        $data: {
            id: UUID,                                    // Very fast
            count: UniformI32::{ low: 1, high: 1000 },  // Fast
            flag: Bool                                   // Very fast
        }
    },
    
    // Less efficient - complex generators
    complex_data: rand_process::{
        $data: {
            // Slower - statistical distributions
            normal_value: NormalF64::{ mean: 0.0, std_dev: 1.0 },
            
            // Slower - complex regex patterns
            complex_pattern: Regex::{ pattern: "([A-Z][a-z]{2,8}\\s){3}[A-Z][a-z]{2,8}" },
            
            // Slower - large arrays
            large_array: UniformArray::{
                min_size: 50,
                max_size: 100,
                element_type: NormalF64::{ mean: 0.0, std_dev: 1.0 }
            }
        }
    }
}
```

### Variable Reuse for Performance

```ion
rand_processes::{
    // Efficient - reuse expensive generators
    $expensive_distribution: WeibullF64::{ shape: 2.0, scale: 100.0 },
    $simple_choices: Uniform::{ choices: ["A", "B", "C", "D"] },
    
    optimized_data: rand_process::{
        $data: {
            // Reuse the same expensive distribution
            measurement1: $expensive_distribution,
            measurement2: $expensive_distribution, 
            measurement3: $expensive_distribution,
            
            // Reuse simple categorical generator
            category1: $simple_choices,
            category2: $simple_choices
        }
    }
}
```

### Memory-Conscious Patterns

```ion
rand_processes::{
    // Memory-efficient approach
    streaming_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::10 },
        $data: {
            // Simple fields - low memory
            id: UUID,
            timestamp: Instant,
            value: UniformF64::{ low: 0.0, high: 100.0 },
            
            // Avoid large embedded structures in high-frequency data
            // metadata: { /* avoid large nested objects */ }
        }
    },
    
    // Separate detailed data as less frequent dataset
    detailed_metadata: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },  // Much less frequent
        $data: {
            detail_id: UUID,
            large_description: LoremIpsum::{ min_words: 50, max_words: 200 },
            complex_structure: {
                nested_data: LoremIpsumTitle,
                more_nested: {
                    deep_field: UniformF64::{ low: 0.0, high: 1.0 }
                }
            }
        }
    }
}
```

## Error Handling in Scripts

### Common Script Errors

#### Invalid Ion Syntax

```ion
// Wrong - missing closing brace
rand_processes::{
    test: rand_process::{
        $data: {
            id: UUID
        }
    // Missing closing brace here
```

**Error:**
```
Error: Failed to parse Ion script: Expected closing brace '}' at line 8
```

#### Invalid Generator Configuration

```ion
// Wrong - min > max
rand_processes::{
    test: rand_process::{
        $data: {
            bad_range: UniformI32::{ low: 100, high: 50 }  // Invalid range
        }
    }
}
```

#### Missing Required Fields

```ion
// Wrong - missing arrival for rand_process
rand_processes::{
    test: rand_process::{
        $data: { id: UUID }  // Missing $arrival
    }
}
```

### Script Validation Patterns

```ion
rand_processes::{
    // Good - comprehensive configuration
    validated_data: rand_process::{
        // Required: arrival process
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        
        // Required: data definition
        $data: {
            // Validate ranges
            valid_range: UniformI32::{ low: 1, high: 100 },  // min <= max
            
            // Validate probabilities  
            valid_probability: Bool::{ p: 0.5 },  // 0.0 <= p <= 1.0
            
            // Validate nullable/optional
            valid_nullable: UniformF64::{ 
                low: 0.0, 
                high: 1.0,
                nullable: 0.1,    // 0.0 <= nullable <= 1.0
                optional: 0.05    // 0.0 <= optional <= 1.0
            }
        }
    }
}
```

## Script Documentation

### Inline Documentation Best Practices

```ion
rand_processes::{
    // =============================================================================
    // E-Commerce Simulation Script v2.1
    // 
    // Purpose: Generate realistic e-commerce data for performance testing
    // Author: Data Team
    // Created: 2024-01-01
    // Last Modified: 2024-01-15
    //
    // Datasets Generated:
    // - customers: Static customer profiles (10-50 customers)
    // - products: Static product catalog (50-200 products) 
    // - orders: Dynamic order events (variable frequency)
    // - reviews: Dynamic product reviews (low frequency)
    // =============================================================================
    
    // === CONFIGURATION VARIABLES ===
    
    // Customer population size
    $n_customers: UniformU8::{ low: 10, high: 50 },  // 10-50 customers for testing
    
    // Product catalog size  
    $n_products: UniformU8::{ low: 50, high: 200 },  // 50-200 products
    
    // Business parameters
    $avg_order_value: UniformDecimal::{ low: 25.00, high: 500.00 },  // Realistic order sizes
    $customer_satisfaction: UniformF64::{ low: 0.7, high: 0.95 },    // High satisfaction rate
    
    // === SHARED GENERATORS ===
    
    $customer_id: UUID,     // Unique customer identifiers
    $product_id: UUID,      // Unique product identifiers  
    $order_id: UUID,        // Unique order identifiers
    
    // === STATIC REFERENCE DATA ===
    
    // Customer master data - generated once at simulation start
    customers: static_data::{
        $data: {
            customer_id: $customer_id,
            name: LoremIpsumTitle,  // Realistic names
            email: Format::{ pattern: "customer{UUID}@example.com" },
            registration_date: Date,  // All register at simulation start
            loyalty_tier: Uniform::{ choices: ["bronze", "silver", "gold", "platinum"] }
        }
    },
    
    // Product catalog - static reference data
    products: static_data::{
        $data: {
            product_id: $product_id,
            name: LoremIpsumTitle,
            category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home"] },
            base_price: $avg_order_value,
            in_stock: Bool::{ p: 0.9 }  // 90% of products in stock
        }
    },
    
    // === DYNAMIC TRANSACTIONAL DATA ===
    
    // Order events - customers place orders over time
    orders: rand_process::{
        // Variable order frequency - some customers more active
        $r: UniformU8::{ low: 30, high: 180 },  // 30-180 minutes between orders
        $arrival: HomogeneousPoisson::{ interarrival: minutes::$r },
        
        $data: {
            order_id: $order_id,
            customer_id: $customer_id,  // Links to customers dataset
            product_id: $product_id,    // Links to products dataset
            quantity: UniformU8::{ low: 1, high: 5 },
            order_total: $avg_order_value,
            timestamp: Instant,
            
            // Order status progression  
            status: Uniform::{ 
                choices: ["pending", "processing", "shipped", "delivered"],
                // Weight towards later statuses for realistic distribution
            }
        }
    },
    
    // Product reviews - less frequent than orders
    reviews: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::UniformU8::{ low: 2, high: 48 } },
        $data: {
            review_id: UUID,
            product_id: $product_id,    // Links to products dataset
            customer_id: $customer_id,  // Links to customers dataset
            rating: UniformU8::{ low: 1, high: 5 },
            review_text: LoremIpsum::{ 
                min_words: 10, 
                max_words: 100,
                optional: 0.3  // 30% don't write review text
            },
            timestamp: Instant,
            verified_purchase: Bool::{ p: 0.8 }  // 80% are verified purchases
        }
    }
}
```

## Script Maintenance and Version Control

### Script Versioning

```ion
rand_processes::{
    // === SCRIPT METADATA ===
    script_info: static_data::{
        $data: {
            script_version: "3.2.1",
            created_date: "2024-01-01",
            last_modified: "2024-01-15", 
            author: "data-engineering-team",
            description: "Multi-tenant SaaS simulation with realistic usage patterns"
        }
    },
    
    // Script content follows...
}
```

### Migration Between Script Versions

```bash
# Test new script version against old version
beamline gen data --seed 1000 --start-auto --script-path data_v3.ion --sample-count 100 > new_output.ion
beamline gen data --seed 1000 --start-auto --script-path data_v2.ion --sample-count 100 > old_output.ion

# Compare schemas
beamline infer-shape --seed 1 --start-auto --script-path data_v3.ion --output-format basic-ddl > new_schema.sql
beamline infer-shape --seed 1 --start-auto --script-path data_v2.ion --output-format basic-ddl > old_schema.sql
diff old_schema.sql new_schema.sql
```

## Real-World Script Examples

### IoT Sensor Network

```ion
rand_processes::{
    // Network topology
    $n_locations: UniformU8::{ low: 3, high: 12 },
    $n_sensors_per_location: UniformU8::{ low: 5, high: 15 },
    
    // Environmental factors
    $base_temperature: NormalF64::{ mean: 22.0, std_dev: 3.0 },
    $seasonal_variation: UniformF64::{ low: -5.0, high: 5.0 },
    
    iot_network: $n_locations::[
        {
            $location_id: UUID::(),
            $location_temp_offset: UniformF64::{ low: -2.0, high: 2.0 }::(), // Per-location offset
            
            // Location metadata
            'location_{$@n}': static_data::{
                $data: {
                    location_id: $location_id,
                    location_name: Format::{ pattern: "Site-{$@n}" },
                    coordinates: {
                        latitude: UniformF64::{ low: 40.0, high: 45.0 },
                        longitude: UniformF64::{ low: -75.0, high: -70.0 }
                    },
                    installation_date: Date
                }
            },
            
            // Sensors at location
            sensors: $n_sensors_per_location::[
                {
                    $sensor_id: UUID::(),
                    
                    'location_{$@n}_sensor_{$@n}': rand_process::{
                        $arrival: HomogeneousPoisson::{ interarrival: seconds::UniformU8::{ low: 30,
