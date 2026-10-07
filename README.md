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

1. Each queen gets its own row. For each one, a random position is drawn
   in the list of free columns, and the columns are tried from there, one
   after the other, until one has no diagonal conflict. The last few
   queens are placed directly, conflicts included.
2. The queens left attacked are then swapped two by two, as long as a
   swap doesn't increase the number of conflicts. When a pass makes no
   progress, every attacked queen joins the set, and each one tries a
   few random partners anywhere on the board.
3. If the search is still stuck after 16 passes without progress,
   everything restarts from a new random placement.

There is no solution for 2 and 3 queens, so these sizes are rejected.

## Performance

Wall-clock time without `-v` nor `-c`:

| n          | `d746b19` (2015) | `3ec545e`  | Now         |
|------------|------------------|------------|-------------|
| 1 000 000  | 0.354 s          | 0.19 s     | **0.16 s**  |
| 10 000 000 | 6.29 s           | 3.38 s     | **2.41 s**  |

Measured on 2026-10-07 on an Intel Core i7-6700HQ (2.6 GHz, 6 MiB L3),
15 GiB RAM, Linux 6.18, rustc 1.99.0, `cargo build --release`. The last
two columns are the median of 7 runs of both binaries, interleaved, in
the same session: this laptop varies by up to 25 % from one run to the
next, so figures from different sessions cannot be compared closely.
The `d746b19` column comes from an earlier session, median of 5 runs. The
time includes the restarts needed to reach a correct solution; `d746b19`
made a single attempt and often returned an incorrect one.

Almost all the time is spent in the initial placement, and for
n = 10 000 000 the arrays take about 200 MB, far more than the CPU
cache. The list of free columns alone is 80 MB: reading it sequentially
from one random start per queen, instead of one random entry per try,
is what brought 3.38 s down to 2.41 s. Tried without gain: a faster
random generator, drawing four candidates at once to overlap the cache
misses, huge pages, and deferring the diagonal counters to a pass after
the placement. A bitset of the occupied diagonals, small enough to stay
in cache, gave another 6 %, not worth its complexity for now.
