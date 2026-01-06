# Reproducible Data Generation

One of Beamline's core strengths is its ability to generate **reproducible data** — the same input parameters will always produce exactly the same output data, no matter when or where you run the generation process.

## What is Reproducibility?

Reproducible data generation means that given the same:
- **Seed value** (random number generator seed)
- **Configuration parameters** (Ion script, generators, etc.)
- **Timestamp** (starting time for temporal data)
- **Environment** (same version of Beamline)

You will get **exactly the same data** every single time, down to the last byte.

## Why Reproducibility Matters

### Debugging and Testing
```bash
# First run - discovers a bug with specific data
beamline gen data --seed 12345 --start-auto --script-path my_script.ion

# Later run - reproduce exact same data to debug
beamline gen data --seed 12345 --start-auto --script-path my_script.ion
```

When you find a bug or unexpected behavior in your tests, reproducibility lets you generate the exact same problematic data to investigate and fix the issue.

### Consistent Benchmarking
```bash
# Performance test run 1
beamline gen data --seed 42 --start-auto --sample-count 1000000 --script-path perf_test.ion

# Performance test run 2 (weeks later)  
beamline gen data --seed 42 --start-auto --sample-count 1000000 --script-path perf_test.ion
```

For meaningful performance comparisons, you need identical datasets. Reproducibility ensures your benchmarks are comparing like with like.

### AI Model Training
```bash
# Training dataset generation
beamline gen data --seed 789 --start-auto --script-path training_data.ion --sample-count 50000

# Later: regenerate exact same training data for model comparison
beamline gen data --seed 789 --start-auto --script-path training_data.ion --sample-count 50000
```

When training machine learning models, being able to regenerate identical training data is crucial for comparing model performance and reproducing results.

### Regression Testing
```bash
# Original test data
beamline gen data --seed 2024 --start-auto --script-path integration_test.ion

# After code changes - same test data to verify no regressions
beamline gen data --seed 2024 --start-auto --script-path integration_test.ion
```

Regression testing requires the same test data to verify that code changes don't break existing functionality.

## How Seeds Work

### Pseudorandom Number Generation

Beamline uses **cryptographically secure pseudorandom number generators** (PRNGs) that are initialized with a seed value:

```bash
# Different seeds = different data
beamline gen data --seed 1 --start-auto --script-path test.ion    # Generates dataset A
beamline gen data --seed 2 --start-auto --script-path test.ion    # Generates dataset B

# Same seed = identical data  
beamline gen data --seed 1 --start-auto --script-path test.ion    # Generates dataset A (identical)
beamline gen data --seed 1 --start-auto --script-path test.ion    # Generates dataset A (identical)
```

### Seed Propagation

Seeds propagate through the entire generation process:
- **Data generators** use the seed for all random decisions
- **Stochastic processes** use the seed for temporal modeling
- **Nested structures** maintain seed consistency across all levels

### Default Seeds

If you don't specify a seed, Beamline uses a default seed derived from the configuration:

```bash
# These generate identical data (same default seed)
beamline gen data --seed-auto --start-auto --script-path my_script.ion
beamline gen data --seed-auto --start-auto --script-path my_script.ion

# This generates different data (explicit different seed)
beamline gen data --seed 999 --start-auto --script-path my_script.ion
```

## Reproducibility Scope

### What IS Reproduced

✅ **Data Values**: All generated numbers, strings, booleans, etc.
✅ **Data Structure**: Object nesting, array lengths, field presence
✅ **Temporal Patterns**: Event timestamps and intervals
✅ **Statistical Distributions**: Same distribution samples
✅ **Relationships**: Cross-field correlations and dependencies

### What Might VARY

❌ **Beamline Version**: Different versions may produce different output
❌ **System Architecture**: 32-bit vs 64-bit might have subtle differences  
❌ **Floating Point**: Different CPUs might have tiny precision differences
❌ **Ion Formatting**: Whitespace and formatting might vary slightly

## Best Practices

### 1. Always Specify Seeds for Important Use Cases

```bash
# Good - explicit seed for reproducible testing
beamline gen data --seed 12345 --start-auto --script-path test_suite.ion

# Avoid - relying on default seed might change
beamline gen data --seed-auto --start-auto --script-path test_suite.ion
```

### 2. Document Your Seeds

```bash
# Document seeds in your scripts or README
# Training data: seed 2024
# Test data: seed 2025  
# Performance benchmark: seed 3000
```

### 3. Use Meaningful Seed Values

```bash
# Use dates, version numbers, or meaningful identifiers
beamline gen data --seed 20241212 --start-auto --script-path data.ion  # Today's date
beamline gen data --seed 100 --start-auto --script-path v1.0.0.ion    # Version-based
```

### 4. Pin Beamline Version for Critical Use Cases

```toml
# In your Cargo.toml or requirements
partiql-beamline = "=1.2.3"  # Exact version for reproducibility
```

### 5. Store Configuration Alongside Data

```bash
# Save configuration for later reproduction
beamline gen data \
  --seed 42 \
  --start-auto \
  --script-path production_test.ion \
  --sample-count 1000 \
  --output-format ion-pretty > data.ion
  
# Document generation parameters separately
echo "Seed: 42, Script: production_test.ion, Count: 1000" > config.txt
```

## Examples

### Basic Reproducibility

```bash
# Generate same data multiple times
$ beamline gen data --seed 100 --start-auto --sample-count 3 --script-path simple.ion
[1, 2, 5]

$ beamline gen data --seed 100 --start-auto --sample-count 3 --script-path simple.ion  
[1, 2, 5]  # Identical output

$ beamline gen data --seed 101 --start-auto --sample-count 3 --script-path simple.ion
[7, 1, 9]  # Different seed = different data
```

### Complex Structure Reproducibility

```bash
# Complex nested structures are also reproducible
$ beamline gen data --seed 200 --start-auto --sample-count 1 --script-path complex.ion
{
  id: 42,
  name: "Alice Johnson", 
  scores: [85, 92, 78],
  metadata: {
    timestamp: 2024-01-15T10:30:00Z,
    active: true
  }
}

# Run again with same seed
$ beamline gen data --seed 200 --start-auto --sample-count 1 --script-path complex.ion
{
  id: 42,
  name: "Alice Johnson",     # Identical name  
  scores: [85, 92, 78],      # Identical scores
  metadata: {
    timestamp: 2024-01-15T10:30:00Z,  # Identical timestamp
    active: true                      # Identical boolean
  }
}
```

### Time-based Reproducibility

```bash
# Even temporal data is reproducible
$ beamline gen data --seed 300 --start-iso "2024-01-01T00:00:00Z" --script-path events.ion --sample-count 3
[
  { event: "login", time: "2024-01-01T00:12:34Z" },
  { event: "action", time: "2024-01-01T00:15:47Z" }, 
  { event: "logout", time: "2024-01-01T00:23:12Z" }
]

# Same seed + same start time = identical temporal patterns
$ beamline gen data --seed 300 --start-iso "2024-01-01T00:00:00Z" --script-path events.ion --sample-count 3
[
  { event: "login", time: "2024-01-01T00:12:34Z" },   # Same intervals
  { event: "action", time: "2024-01-01T00:15:47Z" },  # Same timestamps
  { event: "logout", time: "2024-01-01T00:23:12Z" }   # Exact reproduction
]
```

## Troubleshooting Reproducibility

### Issue: Getting Different Data with Same Seed

**Possible Causes:**
1. Different Beamline versions
2. Different script files
3. Different command-line parameters
4. Different system architectures

**Solution:**
```bash
# Check version
beamline --version

# Use exact same command-line parameters
beamline gen data --seed 123 --start-auto --sample-count 100 --script-path exact_same_script.ion

# Verify script file hasn't changed (use checksums)
sha256sum my_script.ion
```

### Issue: Need to Break Reproducibility

Sometimes you want different data each run:

```bash
# Use current timestamp as seed
beamline gen data --seed $(date +%s) --start-auto --script-path varied_data.ion

# Use random seed
beamline gen data --seed $RANDOM --start-auto --script-path varied_data.ion

# Let Beamline generate a random seed
beamline gen data --seed-auto --start-auto --script-path varied_data.ion
```

## Next Steps

Now that you understand reproducible data generation, you're ready to learn about [Scripts and Processes](./scripts-and-processes.md), which will show you how to configure and control the data generation process through Ion-based scripts.
