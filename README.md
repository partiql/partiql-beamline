# PartiQL Beamline

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
cargo run gen --seed-auto --start-auto --sample-count 2 --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 45121008347100595
Start: 2020-06-16T14:41:19.000000000Z
[2020-06-16 14:42:05.13 +00:00:00] : { 'tick': 46130, 'f': -116.17177080507548, 'i8': 58 }
[2020-06-16 14:42:24.323 +00:00:00] : { 'tick': 65323, 'i8': 88, 'f': -71.33349733660519 }
```

`Example 1` shows, our data-sets has three attributes `Tick`, `f`, and `i8`. It also shows that the random seed that the
tool has created using `--seed-auto` command is `45121008347100595`; using this seed and the same script, we can re-generate the same data.

```
cargo run gen --seed 45121008347100595 --start-auto --sample-count 2 --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 45121008347100595
Start: 2020-06-16T14:41:51.000000000Z
[2020-06-16 14:42:37.13 +00:00:00] : { 'i8': 58, 'f': -116.17177080507548, 'tick': 46130 }
[2020-06-16 14:42:56.323 +00:00:00] : { 'tick': 65323, 'i8': 88, 'f': -71.33349733660519 }
```

In case you want to generate the data with the same `seed` and `start` use `--start-iso` as shown below:

```
cargo run gen --seed 45121008347100595 --start-iso "2020-06-16T14:41:51.000000000Z" --sample-count 2 --script-path partiql-beamline-sim/tests/scripts/sensors.ion

Seed: 45121008347100595
Start: 2020-06-16T14:41:51.000000000Z
[2020-06-16 14:42:37.13 +00:00:00] : { i8: 58, f: -116.17177080507548, tick: 46130 }
[2020-06-16 14:42:56.323 +00:00:00] : { tick: 65323, f: -71.33349733660519, i8: 88 }
```

### Example 2 — scripts
Data Generator uses scripts as recipes for data generation. Let's first create some data using  `sensors-nested.ion` script:

```
cargo run gen --seed-auto --start-auto --sample-count 3 --script-path partiql-beamline-sim/tests/scripts/sensors-nested.ion --output-format ion-pretty

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
    {
      datetime: "2020-01-30T12:26:16.214000000Z",
      value: {
        sub: {
          o: -30,
          f: -4.268479415322838e1
        },
        tick: 202214,
        i8: -85,
        id: 0,
        f: 5.534157211267953e1
      }
    },
    {
      datetime: "2020-01-30T12:28:42.068000000Z",
      value: {
        sub: {
          o: -56,
          f: 1.1572697617723406e2
        },
        id: 1,
        f: -7.40682211763895e1,
        tick: 348068,
        i8: 71
      }
    }
  ]
}
```

Notice the `--outputformat ion-pretty` argument; it generates data in [Amazon Ion](https://amazon-ion.github.io/ion-docs/) data format.

As you can see, data for `value` in `values`, all share the same shape for the data; e.g., they all have `sub` and `tick`; this shape 
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

### Example 2 Summary 
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

## CLI
`partiql-beamline-cli` is a CLI tool that enables interaction with the Beamline through command-line.

### CLI Build

Run the following for building the library which also generates the CLI binary: // TODO add a `MAKE` file or similar
```
cargo build
```

Once ran successfully the CLI binary will be under `./target/debug/partiql-beamline-cli`.

### CLI options

Here is the snapshot of the current command-line options:
```
partiql-beamline-cli --help                                                                                                                                      
PartiQL Beamline CLI

Usage: partiql-beamline-cli <COMMAND>

Commands:
  gen   Run the data generator
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
 target/debug/partiql-beamline-cli gen --help
Run the data generator

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
