# Sensor Data Tutorial

This tutorial will walk you through creating a comprehensive sensor data generation system using PartiQL Beamline. We'll start with simple sensor readings and gradually build up to a complex IoT monitoring system with multiple sensor types, realistic patterns, and related datasets.

## Tutorial Overview

By the end of this tutorial, you'll have created:
- Basic sensor data generators
- Multiple sensor types with different characteristics
- Realistic temporal patterns
- Static reference data
- Related datasets for a complete IoT system
- Queries to analyze the generated data

## Part 1: Simple Sensor Data

Let's start with a basic temperature sensor that takes readings every few minutes.

### Basic Temperature Sensor

Create a file called `simple-sensor.ion`:

```ion
rand_processes::{
    temperature_readings: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            sensor_id: "TEMP_001",
            timestamp: Instant,
            temperature: NormalF64::{ mean: 22.0, std_dev: 3.0 },
            unit: "celsius"
        }
    }
}
```

**Test it:**
```bash
partiql-beamline-cli gen data \
    --seed 42 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 10 \
    --script-path simple-sensor.ion \
    --output-format ion-pretty
```

**Expected Output:**
```ion
{
  seed: 42,
  start: "2024-01-01T00:00:00Z",
  data: {
    temperature_readings: [
      {
        sensor_id: "TEMP_001",
        timestamp: 2024-01-01T00:03:45.123000000+00:00,
        temperature: 24.5,
        unit: "celsius"
      },
      // ... more readings
    ]
  }
}
```

### Understanding the Script

- **`HomogeneousPoisson`**: Models realistic sensor reading intervals
- **`NormalF64`**: Creates realistic temperature variations around 22°C
- **`Instant`**: Captures the exact time of each reading
- **Fixed values**: `sensor_id` and `unit` remain constant

## Part 2: Multiple Sensor Types

Now let's create a more realistic system with different types of sensors.

### Multi-Sensor System

Create `multi-sensor.ion`:

```ion
rand_processes::{
    // Temperature sensor - indoor environment
    temperature: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            sensor_id: "TEMP_001",
            sensor_type: "temperature",
            timestamp: Instant,
            value: NormalF64::{ mean: 22.0, std_dev: 2.0 },
            unit: "celsius",
            location: "office_main"
        }
    },
    
    // Humidity sensor - correlated with temperature
    humidity: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::7 },
        $data: {
            sensor_id: "HUM_001",
            sensor_type: "humidity",
            timestamp: Instant,
            value: NormalF64::{ mean: 45.0, std_dev: 8.0 },
            unit: "percent",
            location: "office_main"
        }
    },
    
    // Motion sensor - binary events
    motion: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::15 },
        $data: {
            sensor_id: "MOT_001",
            sensor_type: "motion",
            timestamp: Instant,
            value: Bool::{ p: 0.3 },  // 30% chance of motion detected
            unit: "boolean",
            location: "office_main"
        }
    },
    
    // Light sensor - varies throughout the day
    light: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::10 },
        $data: {
            sensor_id: "LUX_001",
            sensor_type: "light",
            timestamp: Instant,
            value: LogNormalF64::{ location: 6.0, scale: 0.5 },  // Realistic light distribution
            unit: "lux",
            location: "office_main"
        }
    }
}
```

**Test with specific sensors:**
```bash
partiql-beamline-cli gen data \
    --seed 42 \
    --start-auto \
    --sample-count 20 \
    --script-path multi-sensor.ion \
    --dataset temperature --dataset motion \
    --output-format ion-pretty
```

## Part 3: Realistic Sensor Networks

Let's create a more sophisticated system with multiple locations and sensor relationships.

### Sensor Network with Variables

Create `sensor-network.ion`:

```ion
rand_processes::{
    // Generate between 3 & 5 sensor locations  
    $n: UniformU8::{ low: 3, high: 5 },

    // Multiple sensor locations
    locations: $n::[
        // each iteration of the loop will assign an index from 0..=$n to the variable $@n
        {
            // Sensor interval varies per location
            $interval: Uniform::{ choices: [3, 5, 8] },
            $arrival: HomogeneousPoisson:: { interarrival: minutes::$interval },

            // Temperature sensor for this location  
            temperature: rand_process::{
                $data: {
                    sensor_id: Format::{ pattern: "TEMP_{ $@n }" },
                    location: Format::{ pattern: "ROOM_{ $@n }" },
                    timestamp: Instant,
                    value: NormalF64::{ mean: 20.0, std_dev: 3.0 },
                    unit: "celsius",
                    battery_level: UniformF64::{ low: 0.8, high: 1.0 },
                    signal_strength: UniformI8::{ low: -80, high: -20 }
                }
            },
            
            // Humidity sensor for this location
            'humidity_{ $@n }': rand_process::{
                $arrival: HomogeneousPoisson:: { interarrival: minutes::6 },
                $data: {
                    sensor_id: Format::{ pattern: "HUM_{ $@n }" },
                    location: Format::{ pattern: "ROOM_{ $@n }" },
                    timestamp: Instant,
                    value: NormalF64::{ mean: 40.0, std_dev: 10.0 },
                    unit: "percent",
                    battery_level: UniformF64::{ low: 0.7, high: 1.0 },
                    signal_strength: UniformI8::{ low: -85, high: -25 }
                }
            }
        }
    ]
}
```

**Generate network data:**
```bash
partiql-beamline-cli gen data \
    --seed 123 \
    --start-auto \
    --sample-count 50 \
    --script-path sensor-network.ion \
    --output-format text
```

## Part 4: Adding Static Reference Data

Real IoT systems need configuration and reference data. Let's add static sensor metadata.

### Complete IoT System

Create `iot-system.ion`:

```ion
rand_processes::{
    // Static sensor configuration data
    sensor_registry: static_data::{
        $data: {
            sensor_id: "TEMP_001",
            model: "TempSense Pro",
            manufacturer: "IoT Corp",
            firmware_version: "2.1.4",
            installation_date: "2024-01-01",
            calibration_date: "2024-01-01",
            location: "office_main",
            coordinates: { lat: 37.7749e0, lon: -122.4194e0 },
            specifications: {
                range_min: -40.0e0,
                range_max: 85.0e0,
                accuracy: 0.5e0,
                resolution: 0.1e0
            }
        }
    },
    
    // Dynamic sensor readings
    temperature_readings: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            sensor_id: "TEMP_001",
            timestamp: Instant,
            sequence: Tick,
            reading: {
                value: NormalF64::{ mean: 22.0, std_dev: 2.0 },
                unit: "celsius",
                quality: Uniform::{ choices: ["good", "fair", "poor"] },
                confidence: UniformF64::{ low: 0.85, high: 1.0 }
            },
            metadata: {
                battery_voltage: UniformF64::{ low: 3.0, high: 4.2 },
                signal_rssi: UniformI8::{ low: -90, high: -30 },
                uptime_seconds: Tick,  // tick count since start
                memory_usage: UniformF64::{ low: 0.2, high: 0.8 }
            },
            status: {
                operational: Bool::{ p: 0.95 },
                error_code: Uniform::{ choices: ["NONE", "LOW_BATTERY", "COMM_ERROR", "CALIBRATION"] },
                last_maintenance: "2024-01-01T00:00:00Z"
            }
        }
    },
    
    // Sensor alerts/events
    sensor_alerts: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: hours::2 },
        $data: {
            alert_id: UUID,
            sensor_id: "TEMP_001",
            timestamp: Instant,
            alert_type: Uniform::{ choices: [
                "temperature_high", 
                "temperature_low", 
                "battery_low", 
                "communication_lost",
                "calibration_needed"
            ]},
            severity: Uniform::{ choices: ["info", "warning", "critical"] },
            message: LoremIpsum::{ min_words: 5, max_words: 15 },
            acknowledged: Bool::{ p: 0.7 },
            resolved: Bool::{ p: 0.8 }
        }
    }
}
```

**Generate complete system data:**
```bash
partiql-beamline-cli gen data \
    --seed 456 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 100 \
    --script-path iot-system.ion \
    --output-format ion-pretty
```

## Part 5: Advanced Patterns

### Time-Based Variations

Let's create sensors that show realistic daily patterns:

Create `daily-patterns.ion`:

```ion
rand_processes::{
    // Office temperature with daily cycle
    office_temperature: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::10 },
        $data: {
            sensor_id: "TEMP_OFFICE",
            timestamp: Instant,
            // Simulate daily temperature cycle (warmer during work hours)
            base_temp: NormalF64::{ mean: 21.0, std_dev: 1.0 },
            // Add some randomness for HVAC cycles
            hvac_variation: NormalF64::{ mean: 0.0, std_dev: 0.5 },
            temperature_celsius: NormalF64::{ mean: 21.0, std_dev: 1.5 },
            occupancy_detected: Bool::{ p: 0.6 },  // 60% chance during work hours
            unit: "celsius"
        }
    },
    
    // Outdoor weather station
    weather_station: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::15 },
        $data: {
            station_id: "WEATHER_001",
            timestamp: Instant,
            measurements: {
                temperature: NormalF64::{ mean: 15.0, std_dev: 8.0 },
                humidity: NormalF64::{ mean: 65.0, std_dev: 15.0 },
                pressure: NormalF64::{ mean: 1013.25, std_dev: 20.0 },
                wind_speed: WeibullF64::{ shape: 2.0, scale: 5.0 },
                wind_direction: UniformU16::{ low: 0, high: 359 },
                precipitation: ExpF64::{ rate: 10.0 }  // Most readings are 0, occasional rain
            },
            conditions: Uniform::{ choices: [
                "clear", "partly_cloudy", "cloudy", "overcast", 
                "light_rain", "rain", "heavy_rain", "snow"
            ]},
            visibility_km: LogNormalF64::{ location: 2.0, scale: 0.5 }
        }
    }
}
```

### Correlated Sensors

Create sensors where readings are related to each other:

```ion
rand_processes::{
    // Shared variables for correlation
    $room_base_temp: NormalF64::{ mean: 22.0, std_dev: 1.0 },
    $is_occupied: Bool::{ p: 0.4 },
    $hvac_active: Bool::{ p: 0.3 },
    
    // Temperature affected by occupancy and HVAC
    temperature: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::5 },
        $data: {
            sensor_id: "TEMP_CORR",
            timestamp: Instant,
            base_temperature: $room_base_temp,
            occupancy_detected: $is_occupied,
            hvac_running: $hvac_active,
            temperature_celsius: NormalF64::{ mean: 22.0, std_dev: 2.0 },
            unit: "celsius"
        }
    },
    
    // CO2 levels affected by occupancy
    co2_levels: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::8 },
        $data: {
            sensor_id: "CO2_CORR",
            timestamp: Instant,
            base_co2: NormalF64::{ mean: 400.0, std_dev: 50.0 },
            occupancy_detected: $is_occupied,
            co2_ppm: NormalF64::{ mean: 450.0, std_dev: 75.0 },
            unit: "ppm"
        }
    },
    
    // Occupancy sensor (source of truth)
    occupancy: rand_process::{
        $arrival: HomogeneousPoisson:: { interarrival: minutes::3 },
        $data: {
            sensor_id: "OCC_CORR",
            timestamp: Instant,
            occupied: $is_occupied,
            person_count: UniformU8::{ low: 0, high: 8 },
            confidence: UniformF64::{ low: 0.8, high: 1.0 }
        }
    }
}
```

## Part 6: Generating Queries

Now let's generate some queries to analyze our sensor data:

```bash
# Generate queries for sensor data analysis
partiql-beamline-cli query basic \
    --seed 789 \
    --start-auto \
    --script-path simple-sensor.ion \
    --sample-count 3 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 1 \
        --tbl-flt-path-depth-max 2 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all
```

**Example generated queries:**
```sql
SELECT * FROM temperature_readings AS temperature_readings
WHERE NOT ((temperature_readings.unit IS NULL))

SELECT * FROM temperature_readings AS temperature_readings
WHERE temperature_readings.timestamp IN [
    UTCNOW(),
    UTCNOW(),
    UTCNOW(),
    UTCNOW(),
    UTCNOW()
  ]

SELECT * FROM temperature_readings AS temperature_readings
WHERE (temperature_readings.temperature < -30.634268316887464)
```

## Part 7: Shape Inference

Let's examine the data shapes our sensors generate:

```bash
# Infer shapes from our IoT system
partiql-beamline-cli infer-shape \
    --seed 456 \
    --start-auto \
    --script-path iot-system.ion \
    --output-format basic-ddl
```

**Expected DDL output:**
```sql
-- Dataset: sensor_registry
"coordinates" STRUCT<"lat": DOUBLE,"lon": DOUBLE>,
"firmware_version" VARCHAR,
"installation_date" VARCHAR,
"location" VARCHAR,
"manufacturer" VARCHAR,
"model" VARCHAR,
"sensor_id" VARCHAR,
"specifications" STRUCT<"accuracy": DOUBLE,"range_max": DOUBLE,"range_min": DOUBLE,"resolution": DOUBLE>

-- Dataset: temperature_readings  
"metadata" STRUCT<"battery_voltage": DOUBLE,"memory_usage": DOUBLE,"signal_rssi": TINYINT,"uptime_seconds": INT8>,
"reading" STRUCT<"confidence": DOUBLE,"quality": VARCHAR,"unit": VARCHAR,"value": DOUBLE>,
"sensor_id" VARCHAR,
"sequence" INT8,
"status" STRUCT<"error_code": VARCHAR,"last_maintenance": VARCHAR,"operational": BOOL>,
"timestamp" TIMESTAMP
```

## Part 8: Database Generation

Finally, let's create a complete database with our IoT system:

```bash
# Generate a complete IoT database
partiql-beamline-cli gen db beamline-lite \
    --seed 456 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path iot-system.ion \
    --catalog_name iot-monitoring \
    --catalog_path ./iot-database
```

This creates a complete database with:
- Data files for each dataset
- Shape files describing the schema
- Manifest with generation metadata
- Script backup for reproducibility

## Key Concepts Demonstrated

Through this tutorial, you've learned:

1. **Basic Sensor Modeling**: Simple periodic readings with realistic distributions
2. **Multi-Sensor Systems**: Different sensor types with appropriate characteristics
3. **Variable Usage**: Sharing configuration and creating relationships
4. **Static vs Dynamic Data**: Reference data vs. time-series readings
5. **Complex Data Structures**: Nested objects for rich sensor metadata
6. **Realistic Patterns**: Using appropriate probability distributions
7. **Correlated Data**: Making sensor readings influence each other
8. **Query Generation**: Creating queries that match your data patterns
9. **Shape Inference**: Understanding the schema of generated data
10. **Database Creation**: Building complete, queryable databases

## Next Steps

Try these extensions to deepen your understanding:

1. **Add More Sensor Types**: Create pressure, vibration, or chemical sensors
2. **Implement Sensor Failures**: Model realistic failure patterns and recovery
3. **Create Sensor Networks**: Build hierarchical sensor deployments
4. **Add Data Quality Issues**: Simulate missing readings, outliers, and errors
5. **Model Seasonal Patterns**: Create long-term variations in sensor readings
6. **Build Alert Systems**: Generate complex alert and notification patterns

This tutorial provides a solid foundation for modeling any IoT or sensor-based system with PartiQL Beamline. The patterns you've learned can be adapted to industrial monitoring, smart buildings, environmental sensing, and many other domains.
