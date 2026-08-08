Beamline is a tool for **fast data generation**.
It generates **reproducible [pseudo-random data](https://en.wikipedia.org/wiki/Pseudorandomness)** using a **[stochastic approach](https://en.wikipedia.org/wiki/Stochastic_process)** and
**[probability distributions](https://en.wikipedia.org/wiki/Probability_distribution)**,
meaning you can create realistic datasets that follow specific mathematical patterns. This makes the data both random
enough to be useful for  AI/ML model training, simulation, and testing purposing, while remaining deterministic enough
to be reproducible for debugging and validation.

The tool's ability to generate data based on statistical distributions makes it particularly valuable for AI model
training scenarios where you need synthetic data that resembles specific population distributions or statistical characteristics.

Beamline can generate both random data and SQL-like (PartiQL) queries. For the random data, it gives you the capability to generate (or infer)
schemas for the generated data. In addition, it allows local (file-system) catalog generation for the generated data along with its schema.

Read more in the [Beamline mdbook](https://partiql.org/partiql-beamline/).

# Beamline 101

## Data Generation

### Example 1
In the following example we generate a data-set with two records based on the `sensors.ion` script (we will cover scripts in the next section):

```
$ cargo run gen data \
    --seed-auto \
    --start-auto \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 12328924104731257599
Start: 2024-01-20T20:05:41.000000000Z
[2024-01-20 20:07:46.532 +00:00:00] : "sensors" { 'f': -2.5436390152455175, 'i8': 4, 'tick': 125532 }
[2024-01-20 20:09:19.756 +00:00:00] : "sensors" { 'f': -63.49308817145054, 'i8': 4, 'tick': 218756 }
```

`Example 1` shows, our data-sets has three attributes `Tick`, `f`, and `i8`. It also shows that the random seed that the
tool has created using `--seed-auto` command is `45121008347100595`; using this seed and the same script, we can re-generate the same data.

```
$ cargo run gen data \
    --seed 12328924104731257599 \
    --start-auto \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 12328924104731257599
Start: 2024-01-20T20:51:02.000000000Z
[2024-01-20 20:53:07.532 +00:00:00] : "sensors" { 'tick': 125532, 'i8': 4, 'f': -2.5436390152455175 }
[2024-01-20 20:54:40.756 +00:00:00] : "sensors" { 'tick': 218756, 'i8': 4, 'f': -63.49308817145054 }
```

In case you want to generate the data with the same `seed` and `start` use `--start-iso` as shown below:

```
$ cargo run gen data \
    --seed 12328924104731257599 \
    --start-iso "2024-01-20T20:51:02.000000000Z" \
    --sample-count 2 \
    --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 12328924104731257599
Start: 2024-01-20T20:51:02.000000000Z
[2024-01-20 20:53:07.532 +00:00:00] : "sensors" { 'tick': 125532, 'i8': 4, 'f': -2.5436390152455175 }
[2024-01-20 20:54:40.756 +00:00:00] : "sensors" { 'tick': 218756, 'i8': 4, 'f': -63.49308817145054 }
```

### Example 2 — scripts
Data Generator uses scripts as recipes for data generation. Let's first create some data using  `sensors-nested.ion` script:

```
$ cargo run gen data \
    --seed-auto --start-auto \
    --sample-count 3 \
    --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion \
    --output-format ion-pretty

{
  seed: 8555667609863993831,
  start: "2023-02-18T11:47:36.000000000Z",
  data: {
    sensors: [
      {
        i8: -21,
        tick: 9421,
        f: 2.803799956162891e0,
        sub: {
          f: -3.4540829609160596e1,
          o: -15
        },
        id: 1
      },
      {
        i8: -70,
        tick: 12294,
        f: 1.7229362418585936e1,
        sub: {
          f: -8.237685427198443e1,
          o: -118
        },
        id: 1
      },
      {
        sub: {
          o: -40,
          f: 8.906143160040727e0
        },
        i8: 84,
        id: 0,
        tick: 32697,
        f: -2.4809825455060093e1
      }
    ]
  }
}
```

Notice the `--outputformat ion-pretty` argument; it generates data in [Amazon Ion](https://amazon-ion.github.io/ion-docs/) data format.

As you can see, data for `sensors` in `data`, all share the same shape for the data; e.g., they all have `sub` and `tick`; this shape 
along with other attributes are defined by the `sensors-nested.ion` script.

Here is the contents of `sensors-nested.ion`; as the file extension suggests, the script is written in [Amazon Ion](https://amazon-ion.github.io/ion-docs/) data format:

```
rand_processes::{
    $n: UniformU8::{ low: 1, high: 3 },

    sensors: $n::[
        rand_process::{
            $r: Uniform::[5,10],
            $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
            $data: {
                tick: Tick,
                id: '$@n',
                i8: UniformI8,
                f: UniformF64,
                sub: {
                    o:UniformI8,
                    f:UniformF64,
                }
            }
        }
    ],
}
```

Let's break-down the above script in more details:

#### Data Generator and random processes
Scripts define random processes. A Random Process (or Stochastic Process) is a mathematical models of systems and 
phenomena that appear to vary in a random manner—Wikipedia: https://en.wikipedia.org/wiki/Stochastic_process.

As shown below, this is what the outer `struct`'s annotation in the script says:

```
rand_processes::{
  // Attributes are elided
}
```

Moving on to the attributes, the first attribute `$n: UniformU8::{ low: 2, high: 10 },` defines variable `n` with the type
`UniformU8::{ low: 1, high: 3 }` which is a type that its values are 8-bit unsigned integers and are randomly generated 
between `1` lower and `3` upper bounds (inclusive) using [Discrete Uniform Distribution](https://en.wikipedia.org/wiki/Discrete_uniform_distribution).

The next attribute `sensor` defines a list with `n` (defined previously) elements:

```
sensors: $n::[
  // Attributes are elided
]
```

And as for the list elements, `rand_process` defines a random process as shows below:

```
rand_process::{
    $r: Uniform::[5,10],
    $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
    $data: {
        tick: Tick,
        id: '$@n',
        i8: UniformI8,
        f: UniformF64,
        sub: {
            o:UniformI8,
            f:UniformF64,
        }
    }
}
```

In the above we are defining a random process that has three variable `r`, `arrival`, and `data`. Variable `data` defines
a `struct` with attributes such as `f` and `sub`. Other than `tick`, and `id` other attributes are self-explanatory
(see variable `n`) in the previous paragraphs. `tick` is of type `Tick`, which is a global state attribute that increments
as events occur and samples are retrieved; consider it as [vector clock](https://en.wikipedia.org/wiki/Vector_clock) for
the random process. `id` is a string that its values comes from variable `n`.

##### Arrival

We are defining variable `arrival` with reference to variable `r`. Arrival defines the data sampling model; 
in other words, in defines how random samples arrive for collection. In this case, we are using 
Homogeneous [Poisson process](https://en.wikipedia.org/wiki/Poisson_point_process):

> For the homogeneous Poisson point process, the derivative of the intensity measure is simply a constant λ > 0 
> which can be referred to as the rate, usually when the underlying space is the real line,
> or the intensity. It is also called the mean rate or the mean density or rate.
> For λ = 1, the corresponding process is sometimes referred to as the standard Poisson (point) process.

In other words the homogeneous Poisson process assumes that the rate of occurrence is constant over time or space.

With the above, variable `arrival` is a homogeneous Poisson process with `r` minutes inter-arrival which means for this
process, time elapsed between two consecutive processes will be constant `r` minutes.


#### Example 2 Summary 
Putting all the pieces together the scripts results in generating random data such as below:

```
{
  seed: 7958511458449874628,
  start: "2020-01-30T12:22:54.000000000Z",
  values: [
    {
      datetime: "2020-01-30T12:23:15.958000000Z",
      value: {
        sub: {
          o: -73,
          f: -6.612087476014153e0
        },
        tick: 21958,
        i8: -16,
        id: 0,
        f: 1.1290698764718218e2
      }
    },
  ]
}
```

### Example 3 — Datasets

In the following example we show what datasets are and how one can create data for one or more datasets. We will also introduce some new variable
types such as `Instant` and `UUID` but first the command and its result:

```
$ cargo run gen data \
    --seed 45121008347100595 \
    --start-iso '2020-06-16T14:41:51.000000000Z' \
    --script-path partiql-beamline-sim/tests/scripts/client-service.ion \
    --sample-count 10 \
    --dataset service --dataset client_1 \
    --output-format ion-pretty

{
  seed: 45121008347100595,
  start: "2020-06-16T14:41:51.000000000Z",
  data: {
    service: [
      {
        StartTime: 2020-06-16T14:41:51.011000000+00:00,
        Operation: "GetMyData",
        Account: "5724d45f-d346-6a14-c1c7-654f62b58514",
        client: "customer #3",
        success: true,
        Request: "acd04972-7ed6-2d31-0784-2aa6580dbe5e",
        Program: "FancyService"
      },
      // output-data is elided
      {
        Request: "b8d27ab0-6187-c960-cc3b-606da777c5f8",
        Account: "d8da158c-5262-0be2-9d7d-34ce3eb8d8f1",
        success: true,
        Program: "FancyService",
        Operation: "GetMyData",
        StartTime: 2020-06-16T14:41:51.055000000+00:00,
        client: "customer #7"
      }
    ],
    client_1: [
      {
        id: "d40b50d0-fccf-6773-3a83-06f2957eb91e",
        request_id: "acd04972-7ed6-2d31-0784-2aa6580dbe5e",
        request_time: 2020-06-16T14:41:51.098000000+00:00,
        success: true
      },
      {
      // output-data is elided
      {
        id: "d40b50d0-fccf-6773-3a83-06f2957eb91e",
        request_id: "09aecd26-ec93-95be-b553-d864d2e8f1a9",
        request_time: 2020-06-16T14:41:52.236000000+00:00,
        success: true
      }
    ]
  }
}
```

Notice the `--dataset service --dataset client_1` arguments. This means that we are only interested in getting data for
`service` and `client_1` datasets. If no dataset argument is passed (or no `--dataset`), data for all datasets will be shown.

As the name suggests, datasets represents a collection of data the have a specific shape.

Let's look at the `client-service.ion` file:

```
$ cat partiql-beamline-sim/tests/scripts/client-service.ion

rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 1, high: 3 },

    // A generator for client ids
    $id_gen: UUID,

    // A generator for request ids
    $rid_gen: UUID,

    requests: $n::[
        // each iteration of the loop will assign an index from 1..=$n to the variable $@n
        {
            // customer $@n has a UUID
            // `::()` ensures that `$id_gen` gets evaluated at read time (once) as opposed to generation type, hence it yields a single value for each customer.
            $id: $id_gen::(), 

            // customer $@n will arrive every $r milliseconds
            $r: UniformU8::{low:20, high:150},
            $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

            // customer $@n will have a success rate between 99.5% and 100%
            $rate: UniformF64::{ low:0.995e0, high:1.0e0 },
            $success: Bool::{ p: $rate },

            service: rand_process::{
                $data: {
                    Request: $rid_gen,
                    StartTime: Instant,
                    Program: "FancyService",
                    Operation: "GetMyData",

                    Account: $id,
                    client: Format::{pattern: "customer #{ $@n }"},

                    success: $success,
                }
            },
            'client_{ $@n }': rand_process::{
                $data: {
                    id: $id,
                    request_time: Instant,
                    request_id: $rid_gen,
                    success: $success,
                }
            }
        }
    ]
}
```

`client-service.ion` defines a `service` dataset and `n` `client` datasets, depending on the random number selected
between `5` and `20` (E.g., `8`). In the above you also see some new types, let's go over them:

- `UUID`—yields a UUID as a unique identifier.
- `Instant`—yields the simulation's current 'Time' when a value is generated.

The above example also shows that one can reference variables across datasets. For example `$rid_gen` has been defined
under `rand_processes` and is referenced in `service` and `client_ {$@n }` datasets.

Another point to clarify is `$id: $id_gen::()`. As you can see `$id_gen` is `UUID`. Here, `::()` means that `beamline` creates
a UUID when reading the scripts for each customer, hence having the same `id` across all the generated data for `client_2`
that are different from the `id`s for `client_3`:

```
   client_2: [
      {
        request_time: 2022-09-24T11:51:11.074000000+00:00,
        id: "fc7f9cc8-4c11-4f08-36db-ea036df29385",
        request_id: "0f4a7219-55e6-f6d5-a204-0b3f28700538",
        success: true
      },
      {
        request_time: 2022-09-24T11:51:11.214000000+00:00,
        id: "fc7f9cc8-4c11-4f08-36db-ea036df29385",
        request_id: "e6d21825-9dee-3328-9f49-ef058fd8d4b4",
        success: true
      },
      {
        request_time: 2022-09-24T11:51:11.351000000+00:00,
        id: "fc7f9cc8-4c11-4f08-36db-ea036df29385",
        request_id: "a0fcfd2b-26b6-d86e-18bd-a2d1d3074cff",
        success: true
      }
    ],
    client_3: [
      {
        success: true,
        request_time: 2022-09-24T11:51:11.117000000+00:00,
        id: "7e6d2342-e551-1a73-091f-1fe6f67017fc",
        request_id: "0f4a7219-55e6-f6d5-a204-0b3f28700538"
      },
      {
        success: true,
        request_time: 2022-09-24T11:51:11.125000000+00:00,
        id: "7e6d2342-e551-1a73-091f-1fe6f67017fc",
        request_id: "e6d21825-9dee-3328-9f49-ef058fd8d4b4"
      },
      {
        success: true,
        request_time: 2022-09-24T11:51:11.361000000+00:00,
        id: "7e6d2342-e551-1a73-091f-1fe6f67017fc",
        request_id: "a0fcfd2b-26b6-d86e-18bd-a2d1d3074cff"
      }
    ],


```

### Example 4 — Shape (Schema) Inference
CLI allows you to get the shape of your generated data (a.k.a. `Schema`); see the following example:

```
$ cat sensors.ion

rand_processes::{
    $n: UniformU8::{ low: 2, high: 4 },

    sensors: $n::[
        rand_process::{
            $r: Uniform::[5,10],
            $arrival: HomogeneousPoisson:: { interarrival: minutes::$r },
            $weight: UniformDecimal::{ low: 1.995, high: 4.9999 },
            $data: {
                tick: Tick,
                id: '$@n',
                i8: UniformI8,
                f: UniformF64,
                w: $weight,
                d: UniformDecimal::{ low: 0d0, high: 4.2d1 },
                sub: {
                    o:UniformI8,
                    f:UniformF64,
                }
            }
        }
    ],
}

$ cargo run --release --all-features infer-shape  \
    --seed-auto --start-auto \
    --script-path ./partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 17685918364143248531
Start: 2022-12-12T19:52:29.000000000Z
{
    "sensors": PartiqlType(
        Bag(
            BagType {
                element_type: PartiqlType(
                    Struct(
                        StructType {
                            constraints: {
                                Fields(
                                    {
                                        StructField {
                                            name: "d",
                                            ty: PartiqlType(
                                                DecimalP(
                                                    2,
                                                    0,
                                                ),
                                            ),
                                        },
                                        StructField {
                                            name: "f",
                                            ty: PartiqlType(
                                                Float64,
                                            ),
                                        },
                                        StructField {
                                            name: "i8",
                                            ty: PartiqlType(
                                                Int64,
                                            ),
                                        },
                                        StructField {
                                            name: "tick",
                                            ty: PartiqlType(
                                                Int64,
                                            ),
                                        },
                                        StructField {
                                            name: "w",
                                            ty: PartiqlType(
                                                DecimalP(
                                                    5,
                                                    4,
                                                ),
                                            ),
                                        },
                                    },
                                ),
                            },
                        },
                    ),
                ),
            },
        ),
    ),
}
```

As you can see from the example, using the `shape` command, you can infer the shape of the data as `PartiQLType`.
Beamline also provides different encodings for the output shape; for example you can get the output shape in Beamline JSON
format (a testing suite for PartiQL) or SQL-like DDL; for getting the output in a specific encoding, you can use `--output-format` as the following examples show:

```
$ cargo run infer-shape \       
      --seed 7844265201457918498 \
      --start-auto \
      --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion \
      --output-format basic-ddl

-- Seed: 7844265201457918498
-- Start: 2024-01-01T06:53:06.000000000Z
-- Syntax: partiql_datatype_syntax.0.1
-- Dataset: sensors
"f" DOUBLE,
"i8" INT8,
"id" INT,
"sub" STRUCT<"f": DOUBLE,"o": INT8>,
"tick" INT8

$ cargo run --release --all-features infer-shape  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/sensors.ion \
   --output-format beamline-json
   
{
  seed: -3711181901898679775,
  start: 2022-05-22T13:49:57.000000000+00:00,
  shapes: {
    sensors: partiql::shape::v0::{
      type: "bag",
      items: {
        type: "struct",
        constraints: [
          ordered,
          closed
        ],
        fields: [
          {
            name: "d",
            type: "decimal(2, 0)"
          },
          {
            name: "f",
            type: "double"
          },
          {
            name: "i8",
            type: "int8"
          },
          {
            name: "tick",
            type: "int8"
          },
          {
            name: "w",
            type: "decimal(5, 4)"
          }
        ]
      }
    }
  }
}
```

### Example 5 — Database Generation
Beamline supports creating databases that include both shapes and data. It currently supports BeamlineLite Database
generation on the file system as follows in an example:

```
$ cargo run --release --all-features gen db beamline-lite  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion

writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!
```

The above command creates the database under the `./beamline-catalog` directly. You can customize the catalog name and 
path using `--catalog-name` and `--catalog-path` arguments. See the following for more details on the files created under
the catalog directory:

```
$ cat beamline-catalog/.beamline-manifest
{"seed": "949665520117506306", "start": "2023-02-06T12:52:29.000000000Z" }, "ddl_syntax.version": "partiql_datatype_syntax.0.1" }%

$ cat ./beamline-catalog/.beamline-script                                  

rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },

    // A generator for client ids
    $id_gen: UUID,

    // A generator for request ids
    $rid_gen: UUID,

    requests: $n::[
        // each iteration of the loop will assign an index from 1..=$n to the variable $@n
        {
            // customer $@n has a UUID
            $id: $id_gen::(), // here we force the evaluation of the generator at read time with `::()` to get a single UUID

            // customer $@n will arrive every $r milliseconds
            $r: UniformU8::{low:20, high:150},
            $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

            // customer $@n will have a success rate between 99.5% and 100%
            $rate: UniformF64::{ low:0.995e0, high:1.0e0 },

            $weight: UniformDecimal::{ low: 1.995, high: 4.9999 },

            $success: Bool::{ p: $rate },

            service: rand_process::{
                $data: {
                    Request: $rid_gen,
                    StartTime: Instant,
                    Program: "FancyService",
                    Operation: "GetMyData",
                    Weight: $weight,
                    Distance: UniformDecimal::{ low: 0d0, high: 4.2d1 },
                    Account: $id,
                    client: Format::{pattern: "customer #{ $@n }"},
                    success: $success,
                }
            },
            'client_{ $@n }': rand_process::{
                $data: {
                    id: $id,
                    request_time: Instant,
                    request_id: $rid_gen,
                    success: $success,
                }
            }
        }
    ]
}%

$ tree ./beamline-catalog
./beamline-catalog
├── client_0.ion
├── client_0.shape.ion
├── client_1.ion
├── client_1.shape.ion
├── client_10.ion
├── client_10.shape.ion
├── client_11.ion
├── client_11.shape.ion
├── client_12.ion
├── client_12.shape.ion
├── client_13.ion
├── client_13.shape.ion
├── client_14.ion
├── client_14.shape.ion
├── client_15.ion
├── client_15.shape.ion
├── client_16.ion
├── client_16.shape.ion
├── client_17.ion
├── client_17.shape.ion
├── client_18.ion
├── client_18.shape.ion
├── client_19.ion
├── client_19.shape.ion
├── client_2.ion
├── client_2.shape.ion
├── client_3.ion
├── client_3.shape.ion
├── client_4.ion
├── client_4.shape.ion
├── client_5.ion
├── client_5.shape.ion
├── client_6.ion
├── client_6.shape.ion
├── client_7.ion
├── client_7.shape.ion
├── client_8.ion
├── client_8.shape.ion
├── client_9.ion
├── client_9.shape.ion
├── service.ion
└── service.shape.ion

$ cat ./beamline-catalog/client_0.ion ./beamline-catalog/client_0.shape.ion
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "0de35d1e-a87c-e540-734d-6f2a4fa410c3", request_time: 2021-01-05T03:55:01.035000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "3539cdf0-6f7e-6bdc-c25a-4e0b7d8f8bac", request_time: 2021-01-05T03:55:01.182000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "c6d8ad08-ee24-33d2-50cb-e743e2b9490d", request_time: 2021-01-05T03:55:01.187000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "7b3e0cc7-ee18-148a-d64e-208de07c4bd3", request_time: 2021-01-05T03:55:01.194000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "45e9a44a-67cb-fe8e-0097-abcef70799da", request_time: 2021-01-05T03:55:01.215000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "e9b4fecc-3104-6b44-6bd5-61da0eabc26a", request_time: 2021-01-05T03:55:01.310000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "e26c5803-96ba-ceb6-5069-86f18ed87951", request_time: 2021-01-05T03:55:01.310000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "4311f491-fc4c-8f17-68c6-57ce2f35bcf0", request_time: 2021-01-05T03:55:01.324000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "1ed18755-89ce-d2b4-cc9a-01ea49939510", request_time: 2021-01-05T03:55:01.339000000+00:00}
{success: true, id: "7dbd12cf-b506-22ad-2d81-b0a1cd259697", request_id: "8df88397-585e-1c5d-dd5c-f3bea7990da1", request_time: 2021-01-05T03:55:01.376000000+00:00}
{
  type: "bag",
  items: {
    type: "struct",
    constraints: [
      ordered,
      closed
    ],
    fields: [
      {
        name: "id",
        type: "string"
      },
      {
        name: "request_id",
        type: "string"
      },
      {
        name: "request_time",
        type: "datetime"
      },
      {
        name: "success",
        type: "bool"
      }
    ]
  }
}%

$ cat beamline-catalog/service.shape.sql

"Account" VARCHAR,
"Distance" DECIMAL(2, 0),
"Operation" VARCHAR,
"Program" VARCHAR,
"Request" VARCHAR,
"StartTime" TIMESTAMP,
"Weight" DECIMAL(5, 4),
"anyof" UNION<INT8,DECIMAL(5, 4)>,
"array" ARRAY<INT8>,
"client" VARCHAR,
"success" BOOL
```

The database generation is a safe operation; running the same command won't result in overwriting the created catalog:
```
$ cargo run --release --all-features gen db beamline-lite  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion

creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17
```

If you need to overwrite to the same catalog, you can use `--force` argument. With this command, if the directory exists
Beamline will backup the existing catalog and overwrite the catalog afterward:

```
$ cargo run --release --all-features gen db beamline-lite  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion --force

command is using --force ...
Beamline catalog ./beamline-catalog/ exists, backing it up to "beamline-catalog.2024-05-10T22:15:54.019316000Z.bkp"...
back up completed
writing manifest file ./beamline-catalog/.beamline-manifest ...[COMPLETED]
writing script file ./beamline-catalog/.beamline-script ...[COMPLETED]
writing shape file(s)...[COMPLETED]
writing data file(s)...[COMPLETED]
done!
```


### Example 6 — static data

In many cases, it is useful to have some _static_ data: data that is generated 'before' the first arrival time.


#### 'Static' data generation
To generate static data, we can write a script as we have been doing, but use `static_data` where we would have used 
`rand_process`. Sampling of `static_data` will occur only once at the very beginning of data generation, thus no 
`arrival` is specified. The `data` section of `static_data` is specified the exact same way as `rand_process`, but note
that any time- or tick- related generators will take place at _time 0_.

```
static_data::{  
  $data: {
    // Attributes are elided
  }
  // No $arrival is specified
}
```

Here is the contents of `orders.ion`; In addition to the now-familiar `rand_process` specification, it also contains a
`static_data` generator.

```
rand_processes::{
    // generate between 5 & 20 customers
    $n: UniformU8::{ low: 5, high: 20 },

    // generate between 20 & 100 items
    $item: UniformU8::{ low: 20, high: 100 },

    // A generator for customer ids
    $id_gen: UUID,

    // A generator for order ids
    $oid_gen: UUID,

    customers: $n::[
        // each iteration of the loop will assign an index from 1..=$n to the variable $@n
        {
            // customer $@n has a UUID
            $id: $id_gen::(), // here we force the evaluation of the generator at read time with `::()` to get a single UUID

            // some 'static' data (i.e., generated before simulation starts, thus with no arrivals during simulation)
            // the table has $n 'data row's (1 per $@n)
            customer_table: static_data::{
                $data: {
                    id: $id,
                    address: Format::{pattern: "{ $@n } Foo Bar Ave"},
                }
            },

            // customer $@n will order every $r days
            $r: UniformU8::{low:1, high:150},
            $arrival: HomogeneousPoisson:: { interarrival: days::$r },

            orders: rand_process::{
                $data: {
                    Order: $oid_gen,
                    Customer: $id,
                }
            },
        }
    ],
}
```

As with many of the scripts we've seen in previous examples, here we generate `n` customers and create generators for 
each `@n`. The new bit here is the `customer_table` dataset using the `static_data` specification.

We can execute the `orders.ion` script and request 30 samples:
```
$ cargo run gen data \
    --seed 1234 \
    --start-iso "2019-08-01T00:00:01-07:00" \
    --script-path ./partiql-beamline-sim/tests/scripts/orders.ion \
    --sample-count 30 \
    --output-format text
```

Notice that the output generates 5 customers (and thus 5 entries in the `customer_table`), and then the requested 30 
samples of the `orders` generator. As in previous examples the `id` of each customer is shaed across both the `orders`
generator and the `customer_table` generator.

```
Seed: 1234
Start: 2019-08-01T00:00:01.000000000-07:00
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'address': '0 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'id': '179e600a-c1c5-8ac2-05b6-15b20f8fe740', 'address': '1 Foo Bar Ave' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'address': '2 Foo Bar Ave', 'id': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'address': '3 Foo Bar Ave', 'id': '0730b612-ec93-a2b1-b079-125d57321028' }
[2019-08-01 0:00:01.0 -07:00:00] : "customer_table" { 'address': '4 Foo Bar Ave', 'id': '117ca090-b1c3-21e0-f2ca-a11c15fb812b' }
[2019-08-01 7:26:21.964 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '4c579e42-8c70-93f4-b99b-cc45c50197ed' }
[2019-08-10 5:46:15.24 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '38900593-e9cc-994a-98d9-0becf77d9144' }
[2019-08-11 7:27:49.565 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'b2aa0efc-dac3-b391-f4c2-3c298e0c99f4' }
[2019-08-13 0:23:44.083 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': '4c579e42-8c70-93f4-b99b-cc45c50197ed' }
[2019-08-13 5:22:32.466 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': '38900593-e9cc-994a-98d9-0becf77d9144' }
[2019-08-17 7:59:26.777 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'cf601354-032f-9f74-7547-e4ad25e23ee1' }
[2019-08-20 21:37:07.454 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '736b1863-12d3-0c04-e895-2d3062225171' }
[2019-08-30 9:47:02.759 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '47e4fd27-11e8-ea4c-ac3a-4254922dbdd1' }
[2019-09-05 11:57:24.427 -07:00:00] : "orders" { 'Customer': '0730b612-ec93-a2b1-b079-125d57321028', 'Order': '4c579e42-8c70-93f4-b99b-cc45c50197ed' }
[2019-09-05 20:40:28.682 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': 'b2aa0efc-dac3-b391-f4c2-3c298e0c99f4' }
[2019-09-08 12:34:18.015 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '4c77e699-c643-ef60-0a15-e9a6e0bc8bad' }
[2019-09-09 10:01:08.932 -07:00:00] : "orders" { 'Customer': '0730b612-ec93-a2b1-b079-125d57321028', 'Order': '38900593-e9cc-994a-98d9-0becf77d9144' }
[2019-09-23 23:04:21.425 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': 'cf601354-032f-9f74-7547-e4ad25e23ee1' }
[2019-09-28 9:00:52.046 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'c7bc7140-c38c-15d5-f08b-00dade39da6e' }
[2019-09-28 20:39:05.331 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '4e5424f6-d436-8de8-d43a-8c31777c3161' }
[2019-10-02 14:36:02.158 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'bad7dda0-4bfb-52af-d805-e7fedc53b1af' }
[2019-10-06 18:47:40.54 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': '736b1863-12d3-0c04-e895-2d3062225171' }
[2019-10-10 5:47:40.428 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '32c687bb-01d6-2b44-415a-4a0ffb34a34f' }
[2019-10-12 22:31:48.082 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': '47e4fd27-11e8-ea4c-ac3a-4254922dbdd1' }
[2019-10-13 3:54:28.68 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': '4c77e699-c643-ef60-0a15-e9a6e0bc8bad' }
[2019-10-14 9:52:46.512 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'f1fee644-1c5c-f2eb-a9ab-86306950c9ee' }
[2019-10-17 11:57:39.337 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '50962d64-0b88-fc26-8cb6-7ac160630908' }
[2019-10-20 15:51:21.192 -07:00:00] : "orders" { 'Customer': '0730b612-ec93-a2b1-b079-125d57321028', 'Order': 'b2aa0efc-dac3-b391-f4c2-3c298e0c99f4' }
[2019-10-23 13:57:15.716 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '1ebf70bd-e4fc-a382-14af-6593e83aeb77' }
[2019-10-26 18:24:47.649 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'd864f1ec-a454-8479-3960-a7be57f13aae' }
[2019-10-28 3:51:28.407 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'ac56efa6-b1ae-b1a2-d742-775fceddd0ea' }
[2019-11-02 16:12:24.104 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'bfcd53c3-3f9f-f4a1-9b16-04ecd3393c56' }
[2019-11-04 6:43:42.527 -07:00:00] : "orders" { 'Customer': 'd858b1e7-7327-7c40-1698-0e0e4fe89ecc', 'Order': 'c7bc7140-c38c-15d5-f08b-00dade39da6e' }
[2019-11-06 15:21:28.125 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': '996e847a-39c1-8a88-12e4-c66576067b30' }
[2019-11-07 15:33:31.942 -07:00:00] : "orders" { 'Customer': '5e39c6eb-0bc1-7040-cf52-6e69cdf386e0', 'Order': 'c20ecc3b-f3dd-5977-0cec-ed542ccb7ff7' }
```

## Query Generation
Query Generator creates reproducible PartiQL queries that match the shapes and types (and soon some of the value
aspects) of data defined for the data generator. Let's unpack this with an example:

### Example 1 -- `SELECT * FROM ... WHERE ...`
Given a script like:
```
$ cat partiql-beamline-sim/tests/scripts/simple_transactions.ion
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

We can generate queries to match the shape of the data as specified by the script.
```
$ cargo run query  \
    basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-select-all-fw \
              --tbl-flt-rand-min 1 --tbl-flt-rand-max 1 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all \
                  --tbl-flt-pathstep-final-project \
                  --tbl-flt-type-final-scalar \
                  --pred-lt   
```

And the above invocation results in:
```
SELECT * FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)


SELECT * FROM test_data AS test_data
WHERE (test_data.price < 18.418581624952935)


SELECT * FROM test_data AS test_data
WHERE (test_data.price < 15.495327785402296)
```

Let's revisit the query generation command.
1. We first specify the `query` command and its sub-command `basic`.
   1. `basic` takes `seed`, `start`, `script`, and `sample-count` parameters just like data generation
   2. `basic` has a sub-command `rand-select-all-fw` 
      1. `rand-select-all-fw` generates queries like `SELECT * FROM <from> WHERE <where>` 
      2. the `<from>` is based on  data sets defined in the generator script (corresponding to e.g., table names) 
      3. the `<where>` predicates are randomly generated based on the shape of the dataset and various `tbl-flt-*` command line parameters
      4. the parameters in detail:
         1. `--tbl-flt-rand-min 1 --tbl-flt-rand-max 1` says to generate a random number of predicates between `1` and `1` (i.e., always a single predicate)
         2. `--tbl-flt-path-depth-max 1` says to generate paths with at most one level of depth (e.g. `foo.bar`, but not `foo.bar.baz`)
         3. `--tbl-flt-pathstep-internal-all` says that all types of path steps are valid at internal positions (e.g,. all 
            the following are valid: `foo.bar.baz`, `foo.*.baz`, `foo[4].baz`, `foo[*].baz`)
         3. `--tbl-flt-pathstep-final-project` says that only projection type path steps are valid for the final path component 
            (e.g,. `foo.bar` is valid but the following are invalid: `foo.*`, `foo[4]`, `foo[*]`)
         4. `--tbl-flt-type-final-scalar` says that the type of the value at the final step must be a scalar (not a struct
            or sequence) (e.g., `9`, `'foo'`, `true`, etc.) 
         5. `--pred-lt` says to generate only less-than (`<`) predicates
```
$ cargo run query  \
    basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-select-all-fw \
              --tbl-flt-rand-min 1 --tbl-flt-rand-max 1 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all \
                  --tbl-flt-pathstep-final-project \
                  --tbl-flt-type-final-scalar \
                  --pred-lt   
```

Running the command again with the same seed should yield the same query output.

We can tweak `--tbl-flt-rand-min`, `--tbl-flt-rand-max`, and change `--pred-lt` to `--pred-all` to generate more 'interesting' queries
```
$ cargo run query  \
    basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-select-all-fw \
              --tbl-flt-rand-min 3 --tbl-flt-rand-max 10 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-all \
                  --pred-all  
```

Resulting in:
```
SELECT * FROM test_data AS test_data WHERE (test_data.country_code IN [
      'Graecos quidem legendos.',
      'Possit et sine.'
    ] OR (NOT ((test_data.description IS MISSING)) OR
    (test_data.description IS MISSING)))


SELECT * FROM test_data AS test_data WHERE (((test_data.transaction_id IS NULL)
    AND (test_data.created_at IS NULL)) OR (((test_data.completed IN [
            false,
            false
          ] OR NOT ((test_data.completed IS NULL))) AND
      ((NOT ((test_data.price IS NULL)) OR
          (test_data.transaction_id LIKE 'Vidisse.' AND
            (test_data.country_code IS NULL))) AND
        NOT ((test_data.description IS MISSING)))) OR
    (test_data.description <> 'Nec vero.')))


SELECT * FROM test_data AS test_data
WHERE (((((test_data.country_code <> 'Qua maxime ceterorum.') AND
        (NOT (test_data.completed IN [ false, true, true ]) OR
          (test_data.description = 'Non faciant.'))) AND
      (NOT ((test_data.price IS MISSING)) AND (test_data.price IS MISSING))) OR
    test_data.price IN [
        -47.936734585045905,
        -0.8509689800217544,
        24.263479438050297,
        -48.953369038690255
      ]) OR ((test_data.created_at = UTCNOW()) OR
    (NOT ((test_data.country_code IS MISSING)) AND
      (test_data.description IS MISSING))))
```


### Example 2 -- `SELECT ... FROM ... WHERE ...`

Using similar parameterization as we use for table filters (i.e., `tbl-flt-*`), we can parameterize the generation of
projections in the select clause using the `rand-sfw` instead of the `rand-select-all-fw` sub-command.

```
$ cargo run query \
    basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-sfw \
              --project-rand-min 2 --project-rand-max 5 \
                  --project-path-depth-min 1 --project-path-depth-max 1 \
                  --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all \
              --tbl-flt-rand-min 2 --tbl-flt-rand-max 5 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar \
                  --pred-all      
```

Results in:

```
SELECT test_data.completed, test_data.completed FROM test_data AS test_data
WHERE (NOT (test_data.completed) OR NOT ((test_data.created_at IS MISSING)))


SELECT test_data.completed, test_data.marketplace_id, test_data.created_at
FROM test_data AS test_data WHERE (NOT ((test_data.transaction_id IS NULL)) OR
  (((test_data.transaction_id IN [
            'Iam in.',
            'Se.',
            'Sine amicitia firmam.',
            'Notae sunt.'
          ] OR (test_data.transaction_id IS NULL)) OR
      NOT ((test_data.description IS NULL))) OR
    (test_data.marketplace_id >= 28)))


SELECT test_data, test_data.description FROM test_data AS test_data
WHERE (test_data.completed IN [ false, false ] AND
  (((test_data.price <= 5.761136291521325) AND
      NOT ((test_data.transaction_id IS MISSING))) AND
    (NOT ((test_data.created_at IS MISSING)) AND
      (test_data.created_at IS NULL))))
```

### Example 3 -- `SELECT ... EXCLUDE ... FROM ... WHERE ...`

Using similar parameterization as we use for table filters and projections, we can parameterize the generation of
exclude items in the exclude clause using the `rand-sefw` instead of the `rand-sefw` sub-command.

```
$ cargo run query \
    basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-sefw \
              --project-rand-min 2 --project-rand-max 5 \
                  --project-path-depth-min 1 --project-path-depth-max 1 \
                  --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all \
              --tbl-flt-rand-min 2 --tbl-flt-rand-max 5 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar \
                  --pred-all \
              --exclude-rand-min 1 --exclude-rand-max 3 \
                  --exclude-path-depth-min 1 --exclude-path-depth-max 1 \
                  --exclude-pathstep-internal-all --exclude-pathstep-final-all --exclude-type-final-all      
```

Results in:

```
SELECT test_data.completed, test_data.completed
EXCLUDE test_data.marketplace_id, test_data.*, test_data.completed
FROM test_data AS test_data 
WHERE (NOT (test_data.completed) OR
  NOT ((test_data.created_at IS MISSING)))


SELECT test_data.completed, test_data.marketplace_id, test_data.created_at
EXCLUDE test_data.completed 
FROM test_data AS test_data
WHERE (NOT ((test_data.transaction_id IS NULL)) OR
  (((test_data.transaction_id IN [
            'Iam in.',
            'Se.',
            'Sine amicitia firmam.',
            'Notae sunt.'
          ] OR (test_data.transaction_id IS NULL)) OR
      NOT ((test_data.description IS NULL))) OR
    (test_data.marketplace_id >= 28)))


SELECT test_data, test_data.description 
EXCLUDE test_data.marketplace_id, test_data.completed, test_data.marketplace_id
FROM test_data AS test_data 
WHERE (test_data.completed IN [ false, false ] AND
  (((test_data.price <= 5.761136291521325) AND
      NOT ((test_data.transaction_id IS MISSING))) AND
    (NOT ((test_data.created_at IS MISSING)) AND
      (test_data.created_at IS NULL))))
```

### Example 4 -- `SELECT * EXCLUDE ... FROM ... WHERE ...`

Similar to `rand-select-all-fw`, there is a `rand-select-all-efw` command that will generate `SELECT *` queries with
`EXCLUDE` clauses.

```
$ cargo run query  \
    basic  --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/simple_transactions.ion \
           --sample-count 3 \
    rand-select-all-efw \
              --tbl-flt-rand-min 1 --tbl-flt-rand-max 1 \
                  --tbl-flt-path-depth-max 1 \
                  --tbl-flt-pathstep-internal-all \
                  --tbl-flt-pathstep-final-project \
                  --tbl-flt-type-final-scalar \
                  --pred-lt \
              --exclude-rand-min 1 --exclude-rand-max 3 \
                  --exclude-path-depth-min 1 --exclude-path-depth-max 1 \
                  --exclude-pathstep-internal-all --exclude-pathstep-final-all --exclude-type-final-all  
```

Results in:

```
SELECT * EXCLUDE test_data.marketplace_id, test_data.*, test_data.completed
FROM test_data AS test_data WHERE (test_data.marketplace_id < -5)


SELECT * EXCLUDE test_data.completed FROM test_data AS test_data
WHERE (test_data.price < 18.418581624952935)


SELECT * EXCLUDE test_data.marketplace_id, test_data.completed,
  test_data.marketplace_id
FROM test_data AS test_data WHERE (test_data.price < 15.495327785402296)
```


### Example 5 -- More deeply nested data

Given a script file with more complicated and more deeply nested data, we can see how path depth parameters affect query generation.

```
cat ./partiql-beamline-sim/tests/scripts/transactions.ion
rand_processes::{

    test_data: rand_process::{
        $r: Uniform::{ choices: [5,10] },
        $arrival: HomogeneousPoisson:: { interarrival: milliseconds::$r },

        $currency_price: {
            value: UniformDecimal::{  low: 0.995, high: 499.9999, },
            currency: Regex::{ pattern: "(USD)|(GBP)|(EUR)"},
        },

        $data: {
            transaction_id: UUID,
            marketplace_id: UniformU8,
            country_code: Regex::{ pattern: "[A-Z]{2}" },
            created_at: Instant,
            test_address: Regex::{ pattern: "[1-9][[:digit:]]{1,4} (?:(?:[A-Z][a-z]{2,8})(?:[ -](?:[A-Z][a-z]{2,8})){0,3}) (?:Ave|St|Pl|Way)(?: (?:N|S|E|W|NE|NW|SE|SW))?"},
            completed: Bool,
            description: LoremIpsum::{ min_words:10, max_words:200 },
            price: UniformAnyOf::{ types: [
                $currency_price,
                UniformDecimal::{ low: 2.99, high: 9.99 },
                UniformDecimal::{ low: 5.99, high: 25 },
                UniformDecimal::{ low: 9.99, high: 54.99 },
                UniformDecimal::{ low: 34.99, high: 999.99 },
            ] },
            test_struct: {
                test_decimal: UniformDecimal,
                test_string: LoremIpsumTitle,
            },
            test_nest_struct: {
                nested_struct: {
                    nested_struct: {
                        nested_struct: {
                            nested_struct: {
                                nested_struct: {
                                    test_int: UniformI8::{ low: 0, high: 5 }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

}
```

Compare a query generation command with `--project-path-depth-min 1 --project-path-depth-max 10`
```
$ cargo run query \
    basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/transactions.ion \
           --sample-count 3 \
    rand-sefw \
              --project-rand-min 2 --project-rand-max 5 \
                  --project-path-depth-min 1 --project-path-depth-max 10 \
                  --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all \
              --tbl-flt-rand-min 2 --tbl-flt-rand-max 5 \
                  --tbl-flt-path-depth-max 10 \
                  --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar \
                  --pred-all \
              --exclude-rand-min 1 --exclude-rand-max 2 \
                  --exclude-path-depth-min 3 --exclude-path-depth-max 4 \
                  --exclude-pathstep-internal-all --exclude-pathstep-final-unpivot --exclude-type-final-all
```

Notice the length of paths generated for the select clause.

```
SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct
EXCLUDE test_data.*.*.*.*, test_data.price.* FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))


SELECT test_data.test_nest_struct.*.nested_struct.*.*.nested_struct.*,
  test_data.test_nest_struct.*.*.nested_struct.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.nested_struct.*,
  test_data.test_nest_struct.*.nested_struct.nested_struct.nested_struct.*
EXCLUDE test_data.test_nest_struct.*.*, test_data.test_nest_struct.*.*.*
FROM test_data AS test_data
WHERE ((test_data.*.*.nested_struct.*.*.*.test_int < 40) OR
  (test_data.*.*.nested_struct.nested_struct.*.nested_struct.test_int >= -9))


SELECT test_data.*.nested_struct.nested_struct.nested_struct.nested_struct.*,
  test_data.*.nested_struct.nested_struct.nested_struct.*.*.test_int
EXCLUDE test_data.*.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.*
FROM test_data AS test_data
WHERE ((((test_data.price.value <= 6.206304713037888) OR
      (test_data.*.nested_struct.nested_struct.*.nested_struct.*.test_int <> -29))
    AND
    (test_data.test_nest_struct.*.nested_struct.*.nested_struct.nested_struct.test_int < 6))
  AND ((test_data.price > -44.666855950508584) OR
    (test_data.*.*.*.nested_struct.*.*.test_int > -42)))
```

VS. a query generation command with `--project-path-depth-min 1 --project-path-depth-max 3`
```
$ cargo run query \
    basic --seed 1234 --start-auto --script-path ./partiql-beamline-sim/tests/scripts/transactions.ion \
           --sample-count 3 \
    rand-sefw \
              --project-rand-min 2 --project-rand-max 5 \
                  --project-path-depth-min 1 --project-path-depth-max 3 \
                  --project-pathstep-internal-all --project-pathstep-final-all --project-type-final-all \
              --tbl-flt-rand-min 2 --tbl-flt-rand-max 5 \
                  --tbl-flt-path-depth-max 10 \
                  --tbl-flt-pathstep-internal-all --tbl-flt-pathstep-final-project --tbl-flt-type-final-scalar \
                  --pred-all \
              --exclude-rand-min 1 --exclude-rand-max 2 \
                  --exclude-path-depth-min 3 --exclude-path-depth-max 4 \
                  --exclude-pathstep-internal-all --exclude-pathstep-final-unpivot --exclude-type-final-all
```

Notice the length of paths generated for the select clause.

```
SELECT test_data.price, test_data.*.*.nested_struct EXCLUDE test_data.*.*.*.*,
  test_data.price.*
FROM test_data AS test_data
WHERE ((test_data.test_nest_struct.*.*.*.nested_struct.*.test_int <> 19) OR
  (test_data.test_nest_struct.*.*.nested_struct.*.*.test_int > 35))


SELECT test_data.price, test_data.*.*.nested_struct, test_data.test_struct,
  test_data.*.*.*
EXCLUDE test_data.test_nest_struct.*.*, test_data.test_nest_struct.*.*.*
FROM test_data AS test_data
WHERE ((test_data.*.*.nested_struct.*.*.*.test_int < 40) OR
  (test_data.*.*.nested_struct.nested_struct.*.nested_struct.test_int >= -9))


SELECT test_data.transaction_id, test_data.*.nested_struct
EXCLUDE test_data.*.nested_struct.*.*,
  test_data.test_nest_struct.nested_struct.*.*
FROM test_data AS test_data
WHERE ((((test_data.price.value <= 6.206304713037888) OR
      (test_data.*.nested_struct.nested_struct.*.nested_struct.*.test_int <> -29))
    AND
    (test_data.test_nest_struct.*.nested_struct.*.nested_struct.nested_struct.test_int < 6))
  AND ((test_data.price > -44.666855950508584) OR
    (test_data.*.*.*.nested_struct.*.*.test_int > -42)))
```

## CLI
`partiql-beamline-cli` is a CLI tool that enables interaction with the Beamline through command-line.

### CLI Build

Run the following for building the library which also generates the CLI binary:

```bash
# Using Makefile (recommended)
make build

# Or using cargo directly
cargo build
```

For a release build with optimizations:

```bash
# Using Makefile
make build-release

# Or using cargo directly
cargo build --release
```

Once ran successfully the CLI binary will be under `./target/debug/partiql-beamline-cli`.

### Using the Makefile

The project includes a comprehensive Makefile with common development tasks:

```bash
# Show all available commands
make help

# Build the project
make build

# Run tests
make test

# Format code and run linter
make dev

# Run examples
make examples

# Install the CLI
make install
```

### CLI options

Here is the snapshot of the current command-line options:
```
$ target/debug/partiql-beamline-cli --help    
PartiQL Beamline CLI

Usage: partiql-beamline-cli <COMMAND>

Commands:
  gen          Run the generator
  infer-shape  Run the script shape inference
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

```
$ target/debug/partiql-beamline-cli gen --help
Run the generator

Usage: partiql-beamline-cli gen <COMMAND>

Commands:
  data  Run the data generator
  db    Run the Db generator with both data and schema(s)
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
$ target/debug/partiql-beamline-cli gen data --help
Run the data generator

Usage: partiql-beamline-cli gen data [OPTIONS] <--seed-auto|--seed <SEED>> <--start-auto|--start-epoch-ms <EPOCH_MS>|--start-iso <ISO_8601>> <--script-path <PATH/TO/SCRIPT>|--script <SCRIPT_DATA>|--ddl-path <PATH/TO/DDL>|--ddl <DDL_DATA>>

Options:
      --seed-auto                            Use the local machine's entropy to generate a 'random' seed
      --seed <SEED>                          (Re)play from a specified seed
      --start-auto                           Use the local machine's entropy to generate a 'random' start time
      --start-epoch-ms <EPOCH_MS>            (Re)play from a specified start time (specified in ms since the unix epoch)
      --start-iso <ISO_8601>                 (Re)play from a specified start time (specified in ms since the unix epoch)
      --script-path <PATH/TO/SCRIPT>
      --script <SCRIPT_DATA>                 (Re)play from a specified seed
      --ddl-path <PATH/TO/DDL>              Path to a DDL file (column definitions format)
      --ddl <DDL_DATA>                      DDL column definitions inline
      --dataset-name <DATASET_NAME>         Dataset name for DDL-based generation [default: data]
      --default-nullable <DEFAULT_NULLABLE>  If true, value types will be nullable by default; Else if false, not-nullable by default [possible values: true, false]
      --pct-null <PCT_NULL>                  If specified, value types are nullable by default and will generate `NULL` at the given percentage
      --default-optional <DEFAULT_OPTIONAL>  If true, value types will be optional by default; Else if false, not-optional by default [possible values: true, false]
      --pct-optional <PCT_OPTIONAL>          If specified, value types are optional by default and will generate `MISSING` at the given percentage
      --sample-count <SAMPLE_COUNT>          Value for the number of samples [default: 10]
  -f, --output-format <OUTPUT_FORMAT>        [default: text] [possible values: ion, ion-pretty, ion-binary, text, parquet]
  -o, --output-path <OUTPUT_PATH>           Output directory for file-based formats (e.g., parquet)
  -d, --dataset <DATASETS>
  -h, --help                                 Print help
```

### Parquet Output

Beamline supports outputting generated data in [Apache Parquet](https://parquet.apache.org/) format - a columnar
storage format widely used in data analytics and ML pipelines.

```
$ cargo run gen data \
    --seed 1234 \
    --start-iso "2024-01-01T00:00:00.000000000Z" \
    --sample-count 100 \
    --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion \
    --output-format parquet \
    --output-path ./my-output

wrote 100 row(s) to ./my-output/sensors.parquet
```

Each dataset produces its own `.parquet` file. The Parquet schema is derived from the script's type
information:
- Scalars (bool, int, float, string, decimal, timestamp) map to native Parquet types
- Nested structs are preserved as Parquet struct columns
- Arrays with scalar elements produce native Parquet List columns
- `Value::Null` and `Value::Missing` both map to Parquet's optional (absent) representation

Note: Scripts containing `AnyOf` (union) types will produce an error when using Parquet output,
since Parquet requires a single concrete type per column.

### DDL-Based Data Generation

In addition to Ion scripts, Beamline can generate data directly from SQL-like DDL column definitions:

```
$ cargo run gen data \
    --seed 42 \
    --start-auto \
    --ddl '"sensor_id" VARCHAR, "temperature" DOUBLE, "is_active" BOOL, "reading_time" TIMESTAMP' \
    --dataset-name sensors \
    --sample-count 5 \
    --output-format ion-pretty
```

Or from a DDL file:

```
$ cat schema.ddl
"id" VARCHAR,
"name" VARCHAR,
"price" DECIMAL(10,2),
"created_at" TIMESTAMP,
"metadata" STRUCT<"key": VARCHAR, "value": VARCHAR>

$ cargo run gen data \
    --seed 100 \
    --start-auto \
    --ddl-path schema.ddl \
    --dataset-name products \
    --sample-count 10 \
    --output-format parquet \
    --output-path ./output
```

Supported DDL types: `VARCHAR`, `TINYINT`/`INT8`, `SMALLINT`/`INT16`, `INT`/`INT32`/`INTEGER`,
`BIGINT`/`INT64`, `DOUBLE`/`FLOAT`/`FLOAT64`, `DECIMAL(p,s)`, `BOOL`/`BOOLEAN`,
`TIMESTAMP`/`DATETIME`, `STRUCT<...>`, `ARRAY<T>`, `UNION<T1, T2, ...>`.
