PartiQL Beamline is a tool for fast data generation for PartiQL testing and experimentation purposes. It generates reproducible
[pseudo-random data](https://en.wikipedia.org/wiki/Pseudorandomness) using a [stochastic](https://en.wikipedia.org/wiki/Stochastic) approach.  

Currently, it includes the following components:
1. Data Generator
2. CLI

## Data Generator
Data Generator creates reproducible pseudo-random data. Let's unpack this with an example:

### Example 1 — re-using seed
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
[Homogeneous] [Poisson process](https://en.wikipedia.org/wiki/Poisson_point_process):

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
in under `rand_processes` and is referenced in `service` and `client_ {$@n }` datasets.

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
Beamline also provides different encodings for the output shape; for example you can get the output shape in PartiQL Kollider
format which is a testing suite for PartiQL; for getting the output in a specific encoding, you can use `--output-format` as the following example shows:

```
$ cargo run --release --all-features infer-shape  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/sensors.ion \
   --output-format partiql-kollider
   
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
Beamline supports creating databases that include both shapes and data. It currently supports PartiQL Kollider Database
generation on the file system as follows in an example:

```
$ cargo run --release --all-features gen db kollider  \
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
$ cat ./beamline-catalog/.beamline-manifest
{"seed": 3114525943991198161, "start": 2023-11-07T19:01:28.000000000Z }

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
```

The database generation is a safe operation; running the same command won't result in overwriting the created catalog:
```
$ cargo run --release --all-features gen db kollider  \
   --seed-auto --start-auto \
   --script-path ./partiql-beamline-sim/tests/scripts/client-service.ion

creating directory ./beamline-catalog/ failed with the following error:
File exists (os error 17
```

If you need to overwrite to the same catalog, you can use `--force` argument. With this command, if the directory exists
Beamline will backup the existing catalog and overwrite the catalog afterward:

```
$ cargo run --release --all-features gen db kollider  \
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

### Data Generator Types

| Type           | Description                              | Has Bounded Type | PartiQL Type | PartiQL Type (Bounded) |
|----------------|------------------------------------------|------------------|--------------|------------------------|
| Bool           | Boolean                                  | Y                | BOOL         | BOOL                   |
| String         | String                                   | N                | STRING       | [N/A]                  |
| Uniform        | Uniform distribution over literal values | N                | Union        | [N/A]                  |
| UniformAnyOf   | Uniform distribution over types          | N                | Union        | [N/A]                  |
| UniformU8      | Unsigned 8-bit integer                   | Y                | INT8         | INT8                   |
| UniformU16     | Unsigned 16-bit integer                  | Y                | INT8         | INT8                   |
| UniformU32     | Unsigned 32-bit integer                  | Y                | INT8         | INT8                   |
| UniformU64     | Unsigned 64-bit integer                  | Y                | INT8         | INT8                   |
| UniformI8      | Signed 8-bit integer                     | Y                | INT8         | INT8                   |
| UniformI16     | Signed 16-bit integer                    | Y                | INT8         | INT8                   |
| UniformI32     | Signed 32-bit integer                    | Y                | INT8         | INT8                   |
| UniformI64     | Signed 64-bit integer                    | Y                | INT8         | INT8                   |
| UniformF64     | 64-bit Float (Inexact)                   | Y                | DOUBLE       | DOUBLE                 |
| UniformDecimal | Decimal (Exact)                          | Y                | DECIMAL      | DECIMAL(p, s)          |
| UUID           | UUID                                     | N                | STRING       | [N/A]                  |

1. For types that also have a bounded counter-part, you can define their lower and upper bounds in scripts; for example for
bounded `UniformDecimal` you can specify `UniformDecimal::{ low: 1.995, high: 4.9999 }` which picks a random decimal 
number from the provided boundary.
2. The values for all the types prepended with `Uniform` will get generated using [Discrete Uniform Distribution](https://en.wikipedia.org/wiki/Discrete_uniform_distribution).

### Data Generator Reserved Variable

| Variable       | Description                                            | PartiQL Type |
|----------------|--------------------------------------------------------|--------------|
| Tick           | Simulation Tick                                        | INT8         |
| Instant        | Simulation's current 'Time' when a value is generated. | DATETIME     |

### Data Generator Output Data Formats

| Data Format | Description                                                                            |
|-------------|----------------------------------------------------------------------------------------|
| Text        | A human readable text format                                                           |
| Ion         | [Amazon Ion](https://amazon-ion.github.io/ion-docs/) data format                       |
| Ion Pretty  | [Amazon Ion](https://amazon-ion.github.io/ion-docs/) data format pretty-printed format |

### Data Generator Output Shape Formats

| Shape Format     | Description                                                                            |
|------------------|----------------------------------------------------------------------------------------|
| Text             | A human readable text format                                                           |
| PartiQL Kollider | ParitQL Kollider (a testing suite for PartiQL) shape Format                            |

### Pending Features For Data Generator
- Random PartiQL Query Generation Based on a Schema
- Random Schema generation

## CLI
`partiql-beamline-cli` is a CLI tool that enables interaction with the Beamline through command-line.

### CLI Build

Run the following for building the library which also generates the CLI binary: // TODO add a `MAKE` file or similar
```
$ cargo build
```

Once ran successfully the CLI binary will be under `./target/debug/partiql-beamline-cli`.

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

Usage: partiql-beamline-cli gen [OPTIONS] <--sample-count <SAMPLE_COUNT>> <--seed-auto|--seed <SEED>> <--start-auto|--start-epoch-ms <EPOCH_MS>|--start-iso <ISO_8601>> <--script-path <PATH/TO/SCRIPT>|--script <SCRIPT_DATA>>

Options:
      --sample-count <SAMPLE_COUNT>    Value for the number of samples
      --seed-auto                      Use the local machine's entropy to generate a 'random' seed
      --seed <SEED>                    (Re)play from a specified seed
      --start-auto                     Use the local machine's entropy to generate a 'random' start time
      --start-epoch-ms <EPOCH_MS>      (Re)play from a specified start time (specified in ms since the unix epoch)
      --start-iso <ISO_8601>           (Re)play from a specified start time (specified in ms since the unix epoch)
      --script-path <PATH/TO/SCRIPT>   
      --script <SCRIPT_DATA>           (Re)play from a specified seed
  -f, --output-format <OUTPUT_FORMAT>  [default: text] [possible values: ion, ion-pretty, text]
  -h, --help      
```
