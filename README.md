# N-Queens-Problem

Rust implementation of the N Queens Problem: place n queens on an n×n
checkerboard so that no two queens attack each other.

## Usage

```
cargo build --release
./target/release/n_queens_problem -n 1000000 -c
```

```
Options:
  -n, --size <NUMBER>  Set the number of queens, must be 1 or greater or equal to 4
  -v, --verbose        Print more informations during execution
  -c, --check          Check if the solution found is correct or not
  -s, --seed <SEED>    Seed of the random generator, to reproduce a run (random by default)
  -h, --help           Print help
  -V, --version        Print version
```

With `-v`, the seed is printed so that a run can be reproduced with `-s`.

## Algorithm

A min-conflicts local search, in the spirit of Sosic & Gu:

1. Each queen gets its own row, and columns are drawn at random among the
   free ones. Every queen is placed without any diagonal conflict if
   possible, except for the last few ones, which are placed directly.
2. The queens left attacked are then swapped two by two, as long as a
   swap doesn't increase the number of conflicts.
3. If the search gets trapped, everything restarts from a new random
   placement.

There is no solution for 2 and 3 queens, so these sizes are rejected.

## Performance

Wall-clock time, median of 5 runs, without `-v` nor `-c`:

| n          | Before (`d746b19`) | Now        |
|------------|--------------------|------------|
| 1 000 000  | 0.354 s            | **0.150 s** |
| 10 000 000 | 6.29 s             | **3.46 s**  |

Measured on 2026-10-07 on the same machine for both versions: Intel Core
i7-6700HQ (2.6 GHz, 6 MiB L3), 15 GiB RAM, Linux 6.18, rustc 1.99.0,
`cargo build --release`. The time includes the restarts needed to reach a
correct solution; the version before it made a single attempt and often
returned an incorrect one.

The earlier figures in this README (0.704 s for n = 1 000 000 and 9.291 s
for n = 10 000 000) were measured on another machine and toolchain, so
they cannot be compared directly.

Most of the time is spent drawing random free columns during the initial
placement: with n = 10 000 000 the arrays take about 200 MB, so these
random accesses mostly miss the CPU cache.
