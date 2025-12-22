# Nullability and Optionality

PartiQL Beamline provides fine-grained control over `NULL` and `MISSING` values in generated data. Understanding the distinction between these concepts and how to configure them is crucial for creating realistic datasets that match real-world data patterns.

## NULL vs MISSING Values

PartiQL distinguishes between two types of absent values:

### NULL Values
- **Meaning**: The field exists but has no value
- **JSON equivalent**: `"field": null`
- **SQL equivalent**: `NULL`
- **Ion format**: `field: null`

### MISSING Values  
- **Meaning**: The field doesn't exist at all
- **JSON equivalent**: Field is not present in the object
- **SQL equivalent**: Column not included in row
- **Ion format**: Field is omitted entirely

## Configuration Syntax

Every generator supports both nullability and optionality configuration:

### Basic Syntax

```ion
generator_name::{ nullable: <config>, optional: <config> }
```

### Configuration Values

**Boolean Configuration:**
- `nullable: true` - Field can be NULL (but 0% chance by default)
- `nullable: false` - Field cannot be NULL (default)
- `optional: true` - Field can be MISSING (but 0% chance by default)  
- `optional: false` - Field cannot be MISSING (default)

**Probability Configuration:**
- `nullable: 0.0` - 0% chance of NULL (same as `false`)
- `nullable: 0.25` - 25% chance of NULL
- `nullable: 1.0` - 100% chance of NULL (always NULL)
- `optional: 0.1` - 10% chance of MISSING
- `optional: 0.5` - 50% chance of MISSING

## Examples from Real Test Scripts

### Basic Nullability

From the `sensors.ion` test script:

```ion
rand_processes::{
    sensors: rand_process::{
        $weight: UniformDecimal::{ 
            nullable: 0.75,     // 75% chance of NULL
            low: 1.995, 
            high: 4.9999, 
            optional: true      // Can be MISSING (0% chance by default)
        },
        $data: {
            weight: $weight,
            price: UniformDecimal::{ 
                low: 2.99, 
                high: 99999.99, 
                optional: true  // Field might not appear at all
            }
        }
    }
}
```

### Advanced Configuration

From the `numbers.ion` test script showing all combinations:

```ion
rand_processes::{
    test_data: rand_process::{
        $data: {
            // Default behavior - not nullable, not optional
            basic_int: UniformI32::{ low: 1, high: 100 },
            
            // Only nullable
            nullable_only: UniformI32::{ 
                nullable: 0.2,    // 20% NULL
                low: 1, 
                high: 100 
            },
            
            // Only optional  
            optional_only: UniformI32::{ 
                optional: 0.1,    // 10% MISSING
                low: 1, 
                high: 100 
            },
            
            // Both nullable and optional
            both_configured: UniformI32::{ 
                nullable: 0.2,    // 20% NULL
                optional: 0.1,    // 10% MISSING
                low: 1,           // 70% present values
                high: 100 
            }
        }
    }
}
```

## Output Examples

### Text Format Output

```bash
$ beamline gen data \
    --seed 1000 \
    --start-auto \
    --script-path nullability_test.ion \
    --sample-count 10

# Sample outputs showing NULL and MISSING behavior
[2024-01-01 00:00:01.123] : "test_data" { 'basic_int': 42, 'nullable_only': null, 'both_configured': 15 }
[2024-01-01 00:00:02.456] : "test_data" { 'basic_int': 78, 'nullable_only': 23, 'both_configured': null }
[2024-01-01 00:00:03.789] : "test_data" { 'basic_int': 91, 'nullable_only': 67 }  // optional_only is MISSING
[2024-01-01 00:00:04.012] : "test_data" { 'basic_int': 33, 'nullable_only': null, 'optional_only': 88, 'both_configured': 54 }
```

Notice how:
- `basic_int` always appears (not nullable, not optional)
- `nullable_only` can be `null` but always present  
- `optional_only` might not appear at all (MISSING)
- `both_configured` can be `null` or MISSING

### Ion Pretty Format Output

```ion
{
  seed: 1000,
  start: "2024-01-01T00:00:00Z",
  data: {
    test_data: [
      {
        basic_int: 42,
        nullable_only: null,        // NULL value present
        both_configured: 15
        // optional_only is MISSING (field not present)
      },
      {
        basic_int: 78,
        nullable_only: 23,
        optional_only: 67,
        both_configured: null       // NULL value present
      },
      {
        basic_int: 91,
        nullable_only: 67,
        optional_only: 45
        // both_configured is MISSING (field not present)
      }
    ]
  }
}
```

## Global Defaults via CLI

You can set global nullability and optionality defaults via CLI options:

### CLI Default Configuration

```bash
# Make all types nullable with 10% NULL values
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --pct-null 0.1

# Make all types optional with 5% MISSING values  
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --pct-optional 0.05

# Combine both
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --pct-null 0.1 \
  --pct-optional 0.05

# Disable both globally
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path data.ion \
  --default-nullable false \
  --default-optional false
```

### Script vs CLI Override Behavior

- **Script configuration takes precedence** over CLI defaults
- **CLI defaults apply** to generators without explicit nullable/optional configuration
- **Explicit `false`** in script overrides CLI defaults

**Example:**
```bash
# CLI sets 20% NULL globally
beamline gen data \
  --pct-null 0.2 \
  --script-path mixed_config.ion
```

```ion
rand_processes::{
    test_data: rand_process::{
        $data: {
            // Uses CLI default: 20% NULL
            field1: UniformI32::{ low: 1, high: 100 },
            
            // Overrides CLI default: never NULL
            field2: UniformI32::{ nullable: false, low: 1, high: 100 },
            
            // Overrides CLI default: 50% NULL
            field3: UniformI32::{ nullable: 0.5, low: 1, high: 100 }
        }
    }
}
```

## Realistic Data Patterns

### Database-like Nullability

```ion
rand_processes::{
    users: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::1 },
        $data: {
            // Required fields - never NULL or MISSING
            user_id: UUID::{ nullable: false, optional: false },
            created_at: Instant::{ nullable: false, optional: false },
            
            // Often present, sometimes NULL
            email: Regex::{ 
                pattern: "[a-z]+@[a-z]+\\.[a-z]{2,3}",
                nullable: 0.05  // 5% NULL emails
            },
            
            // Optional profile fields
            full_name: LoremIpsumTitle::{ optional: 0.3 },  // 30% don't provide name
            phone: Regex::{ 
                pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}",
                optional: 0.4,   // 40% don't provide phone
                nullable: 0.1    // 10% provide NULL phone
            },
            
            // Rarely provided optional fields
            bio: LoremIpsum::{ 
                min_words: 10, 
                max_words: 50,
                optional: 0.8    // 80% don't provide bio
            }
        }
    }
}
```

### Sensor Data with Missing Readings

```ion
rand_processes::{
    sensor_readings: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
        $data: {
            sensor_id: UUID::{ nullable: false },
            timestamp: Instant::{ nullable: false },
            
            // Primary measurement - rarely fails
            temperature: UniformF64::{ 
                low: -10.0, 
                high: 50.0,
                nullable: 0.02   // 2% sensor failures (NULL)
            },
            
            // Secondary measurement - more failures
            humidity: UniformF64::{ 
                low: 0.0, 
                high: 100.0,
                nullable: 0.05   // 5% sensor failures
            },
            
            // Optional calibration data
            calibration_offset: UniformF64::{ 
                low: -1.0, 
                high: 1.0,
                optional: 0.7    // 70% don't have calibration data
            },
            
            // Rarely available GPS coordinates
            latitude: UniformF64::{ 
                low: -90.0, 
                high: 90.0,
                optional: 0.9    // 90% don't have GPS
            },
            longitude: UniformF64::{ 
                low: -180.0, 
                high: 180.0,
                optional: 0.9    // 90% don't have GPS  
            }
        }
    }
}
```

## Statistical Distribution Implications

### Stable Value Generation

An important feature of Beamline's nullability/optionality system is **stable value generation**:

> The value that would have been generated if not absent is still generated, it is just discarded. This ensures that value generation is stable even across runs with different densities of NULL and/or MISSING data.

**Example:**
```ion
test_field: UniformI32::{ low: 1, high: 100, nullable: 0.5 }
```

With seed 42:
- **50% NULL case**: Generates `17`, discards it, outputs `null`  
- **0% NULL case**: Generates `17`, outputs `17`
- **Same underlying sequence**: Both cases generate the same random number sequence

This stability is crucial for:
- **A/B testing**: Compare models with different missing data rates
- **Robustness testing**: Test algorithms with varying data completeness
- **Reproducible experiments**: Same seed produces same value patterns

### AI Model Training Applications

```ion
rand_processes::{
    training_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: milliseconds::1 },
        $data: {
            // Always present features
            sample_id: UUID::{ nullable: false, optional: false },
            
            // Core features with realistic missingness
            age: NormalF64::{ 
                mean: 35.0, 
                std_dev: 12.0,
                nullable: 0.02    // 2% missing age data
            },
            
            income: LogNormalF64::{ 
                location: 10.5, 
                scale: 0.5,
                nullable: 0.15,   // 15% NULL income (sensitive data)
                optional: 0.05    // 5% refuse to provide income
            },
            
            // Optional survey responses
            satisfaction_score: UniformF64::{ 
                low: 1.0, 
                high: 10.0,
                optional: 0.4     // 40% don't respond to survey
            },
            
            // Rarely collected features
            location: Regex::{ 
                pattern: "[A-Z]{2}",
                optional: 0.8     // 80% don't provide location
            },
            
            // Target variable - usually complete
            target: Bool::{ 
                p: 0.3,
                nullable: 0.01    // 1% labeling errors
            }
        }
    }
}
```

## Complex Nullability Patterns

### Conditional Nullability

Use variables to create related nullability patterns:

```ion
rand_processes::{
    $has_premium: Bool::{ p: 0.3 },  // 30% premium users
    
    users: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::2 },
        $data: {
            user_id: UUID,
            plan_type: Uniform::{ choices: ["free", "premium"] },
            
            // Premium features - NULL for non-premium users
            premium_start_date: Instant::{ 
                nullable: 0.7  // 70% NULL (approx. non-premium rate)
            },
            premium_features: UniformArray::{
                min_size: 1,
                max_size: 5,
                element_type: LoremIpsumTitle,
                nullable: 0.7,  // 70% NULL for non-premium
                optional: 0.1   // 10% don't specify even if premium
            }
        }
    }
}
```

### Correlated Missing Data

```ion
rand_processes::{
    user_profiles: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::5 },
        $data: {
            user_id: UUID::{ nullable: false },
            
            // Contact information - often missing together
            email: Regex::{ 
                pattern: "[a-z]+@[a-z]+\\.[a-z]{2,3}",
                optional: 0.2  // 20% don't provide email
            },
            phone: Regex::{ 
                pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}",
                optional: 0.25,  // 25% don't provide phone
                nullable: 0.05   // 5% provide NULL phone
            },
            
            // Address components - missing together or not at all
            street_address: LoremIpsumTitle::{ optional: 0.3 },
            city: LoremIpsumTitle::{ optional: 0.3 },
            postal_code: Regex::{ 
                pattern: "[0-9]{5}",
                optional: 0.3 
            },
            
            // Optional demographic info
            age: UniformU8::{ 
                low: 18, 
                high: 80,
                optional: 0.4,   // 40% don't provide age
                nullable: 0.02   // 2% provide invalid age (NULL)
            }
        }
    }
}
```

## Schema Impact

Nullability and optionality affect inferred schemas:

### Schema Inference Output

```bash
$ beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path nullability_test.ion \
    --output-format basic-ddl

-- Dataset: test_data  
"basic_field" INT NOT NULL,                    -- nullable: false
"nullable_field" INT,                          -- nullable: 0.2
"optional_field" OPTIONAL INT NOT NULL,        -- optional: 0.1
"both_field" OPTIONAL INT                      -- nullable: 0.2, optional: 0.1
```

### CLI Default Impact on Schema

```bash
$ beamline infer-shape \
    --seed 1 \
    --start-auto \
    --script-path simple_data.ion \
    --default-nullable true \
    --default-optional true \
    --output-format basic-ddl

-- All fields become nullable and optional by default
"field1" OPTIONAL INT,
"field2" OPTIONAL VARCHAR,
"field3" OPTIONAL BOOL
```

## Performance Considerations

### Value Generation Stability

NULL and MISSING generation maintains the same computational cost:

```ion
// Same performance regardless of nullability rate
fast_field: UniformI32::{ low: 1, high: 1000, nullable: 0.0 }   // 0% NULL
slow_field: UniformI32::{ low: 1, high: 1000, nullable: 0.9 }   // 90% NULL
```

**Why**: The underlying value is always generated, then conditionally discarded.

### Memory Usage

- **NULL values**: Stored in output (takes memory)
- **MISSING values**: Not stored (saves memory)  
- **High optionality**: Can reduce output size significantly

```ion
// Large optional fields save memory when MISSING
large_description: LoremIpsum::{ 
    min_words: 100, 
    max_words: 1000,
    optional: 0.8  // 80% MISSING saves significant memory
}
```

## Testing Data Quality

### Missing Data Robustness Testing

Create datasets with increasing levels of missingness:

```ion
// Test script for robustness testing
rand_processes::{
    // Dataset with low missingness
    clean_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            feature1: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.01 },
            feature2: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.01 },
            target: Bool::{ p: 0.5, nullable: false }
        }
    },
    
    // Dataset with moderate missingness
    noisy_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            feature1: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.1 },
            feature2: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.15 },
            target: Bool::{ p: 0.5, nullable: 0.02 }
        }
    },
    
    // Dataset with high missingness
    sparse_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::1 },
        $data: {
            feature1: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.3 },
            feature2: NormalF64::{ mean: 0.0, std_dev: 1.0, nullable: 0.4 },
            target: Bool::{ p: 0.5, nullable: 0.05 }
        }
    }
}
```

### Real-World Data Simulation

```ion
rand_processes::{
    customer_survey: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::1 },
        $data: {
            // Always collected
            survey_id: UUID::{ nullable: false, optional: false },
            timestamp: Instant::{ nullable: false, optional: false },
            
            // Demographics - some prefer not to answer
            age: UniformU8::{ low: 18, high: 80, optional: 0.15 },
            gender: Uniform::{ 
                choices: ["M", "F", "Other"], 
                optional: 0.2  // 20% prefer not to say
            },
            
            // Income - sensitive, often skipped or invalid
            income_range: Uniform::{ 
                choices: ["<30K", "30-60K", "60-100K", "100K+"],
                optional: 0.3,   // 30% skip question
                nullable: 0.1    // 10% provide invalid answer
            },
            
            // Rating questions - sometimes skipped
            overall_rating: UniformU8::{ 
                low: 1, 
                high: 10,
                optional: 0.1    // 10% skip rating
            },
            
            // Open-ended responses - frequently skipped  
            comments: LoremIpsum::{ 
                min_words: 5, 
                max_words: 100,
                optional: 0.6    // 60% don't provide comments
            }
        }
    }
}
```

## Best Practices

### 1. Realistic Nullability Rates

```ion
// Good - realistic rates based on domain
email: Regex::{ pattern: "...", nullable: 0.05 }    // 5% invalid emails
age: UniformU8::{ low: 18, high: 80, optional: 0.1 }  // 10% don't provide age

// Avoid - extreme rates without justification  
field: UniformI32::{ low: 1, high: 100, nullable: 0.99 }  // 99% NULL - rarely useful
```

### 2. Use Appropriate Absence Types

```ion
// NULL for invalid/unknown values
sensor_reading: UniformF64::{ low: 0.0, high: 100.0, nullable: 0.02 }  // Sensor malfunction

// MISSING for optional fields
optional_comment: LoremIpsum::{ min_words: 5, max_words: 50, optional: 0.4 }  // User choice
```

### 3. Document Nullability Decisions

```ion
rand_processes::{
    // Document nullability reasoning
    user_data: rand_process::{
        $data: {
            // Required business key - never absent
            customer_id: UUID::{ nullable: false, optional: false },
            
            // Email required for notifications - rare NULLs for bad data
            email: Regex::{ pattern: "...", nullable: 0.01 },
            
            // Phone optional - users may not provide
            phone: Regex::{ pattern: "...", optional: 0.3 },
            
            // Marketing consent - some users skip this question
            marketing_consent: Bool::{ optional: 0.15, nullable: 0.05 }
        }
    }
}
```

### 4. Test Multiple Missingness Levels

```bash
# Generate datasets with different missingness for testing
beamline gen data --seed 1 --start-auto --script data.ion --pct-null 0.0 --sample-count 1000 > clean.ion
beamline gen data --seed 1 --start-auto --script data.ion --pct-null 0.1 --sample-count 1000 > noisy.ion  
beamline gen data --seed 1 --start-auto --script data.ion --pct-null 0.3 --sample-count 1000 > sparse.ion
```

## Common Patterns

### Required vs Optional Fields

```ion
rand_processes::{
    e_commerce: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::2 },
        $data: {
            // Required for business logic
            order_id: UUID::{ nullable: false, optional: false },
            customer_id: UUID::{ nullable: false, optional: false },
            created_at: Instant::{ nullable: false, optional: false },
            
            // Required but can have data quality issues
            total_amount: UniformDecimal::{ 
                low: 5.00, 
                high: 500.00,
                nullable: 0.005  // 0.5% data corruption
            },
            
            // Optional customer-provided data
            shipping_instructions: LoremIpsum::{ 
                min_words: 3, 
                max_words: 20,
                optional: 0.7  // 70% don't provide instructions
            },
            
            // Optional promotional data
            promo_code: Regex::{ 
                pattern: "[A-Z]{4}[0-9]{2}",
                optional: 0.8  // 80% don't use promo codes
            }
        }
    }
}
```

### Legacy Data Migration Patterns

```ion
rand_processes::{
    migrated_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::1 },
        $data: {
            // Legacy ID - sometimes missing from old records
            legacy_id: UniformI32::{ 
                low: 1, 
                high: 999999,
                optional: 0.1  // 10% of old records missing legacy ID
            },
            
            // New ID - always present for new system
            new_id: UUID::{ nullable: false, optional: false },
            
            // Data quality issues from migration
            migrated_date: Instant::{ 
                nullable: 0.05  // 5% migration errors (NULL dates)
            },
            
            // Fields added after migration - missing from old records
            new_feature: LoremIpsumTitle::{ 
                optional: 0.6  // 60% old records don't have this field
            }
        }
    }
}
```

## Troubleshooting

### Issue: Unexpected NULL/MISSING Behavior

**Check configuration precedence:**
1. Script-level configuration overrides CLI defaults
2. Variable-level configuration applies to all uses
3. CLI defaults apply to unconfigured generators

### Issue: Too Many/Few NULL Values

**Verify probability values:**
- Values must be between 0.0 and 1.0
- 0.1 = 10%, 0.25 = 25%, 0.5 = 50%, etc.

### Issue: Schema Doesn't Match Expected Nullability

**Check CLI defaults:**
```bash
# Check if CLI is setting global defaults
beamline infer-shape --seed 1 --start-auto --script-path data.ion --default-nullable false
```

## Integration with Query Generation

NULL and MISSING values affect query generation:

```bash
# Generate data with nullability
beamline gen data \
  --seed 1 \
  --start-auto \
  --script-path nullable_data.ion \
  --sample-count 1000 \
  --output-format ion-pretty > test_data.ion

# Generate queries that handle NULL/MISSING
beamline query basic \
  --seed 2 \
  --start-auto \
  --script-path nullable_data.ion \
  --sample-count 10 \
  rand-select-all-fw \
    --pred-absent  # Include IS NULL, IS NOT NULL, IS MISSING predicates
```

This creates queries like:
```sql
SELECT * FROM test_data WHERE (test_data.email IS NOT NULL)
SELECT * FROM test_data WHERE (test_data.optional_field IS MISSING)  
SELECT * FROM test_data WHERE (test_data.phone IS NULL OR test_data.phone LIKE '%555%')
```

## Next Steps

- [Output Formats](./output-formats.md) - See how NULL/MISSING values appear in different formats
- [Scripts](./scripts.md) - Advanced techniques for managing nullability in complex scripts
- [Query Generation](../query-generation/overview.md) - Generate queries that handle absent values
