use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::Solution;
use crate::has_solution;

/// Statistics about a call to solve().
/// Except for restarts, they describe the attempt that found the solution.
#[derive(Debug)]
pub struct SolveStats {
    /// Number of times the search restarted from a new random placement.
    pub restarts: usize,

    /// Average number of free columns tried per queen during init().
    pub avg_tries: f64,

    /// Number of conflicts left by init().
    pub conflicts_after_init: usize,

    /// Number of swaps made by the local search.
    pub swaps: usize,
}

/// The search state: the queens being placed and everything needed to
/// move them cheaply. solve() consumes it and returns the Solution.
pub struct Solver {
    n: usize,

    /// Number of queens that will be kept in conflict during init().
    nb_init_conflicts: usize,

    /// This will hold the queens positions.
    /// Index is the queen number, value is the queen column.
    queen: Vec<usize>,

    /// The queens attacked at the end of init(), the only ones moved by the local search.
    conflict_queens: Vec<usize>,

    /// Diags are arrays, but their size isn't known at compile time so we use vectors.
    prim_diag: Vec<u8>,
    sec_diag: Vec<u8>,

    /// Number of diagonals that have more than 1 queen in it.
    /// This represents the number of conflicts in our checkerboard.
    conflict_diags: usize,

    /// The columns that have no queen yet. It starts with all the n columns,
    /// and init() empties it.
    free_cols: Vec<usize>,

    /// Random generator, seeded so that a run can be reproduced.
    rng: SmallRng,
}

impl Solver {
    /// Create a solver for n queens,
    /// and make all possible allocations and set them to 0.
    /// The same seed always gives the same solution.
    ///
    /// # Panics
    ///
    /// Panics if there is no solution for n queens (see has_solution),
    /// since solve() would never end.
    pub fn new(n: usize, seed: u64) -> Solver {
        assert!(has_solution(n), "there is no solution for {n} queens");

        let nb_init_conflicts = match n {
            4..=10 => n,
            11..=100 => n / 2,
            101..=1_000 => 30,
            1_001..=100_000 => 50,
            100_001..=1_000_000 => 80,
            _ => 100,
        };

        Solver {
            n,
            nb_init_conflicts,
            queen: vec![0; n],
            conflict_queens: Vec::new(),
            // At first all diagonals contain 0 queens.
            prim_diag: vec![0; 2 * n],
            sec_diag: vec![0; 2 * n],
            conflict_diags: 0,
            // At first all columns are free.
            free_cols: (0..n).collect(),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Solve the problem, restarting from a new random placement
    /// each time the local search gets trapped.
    /// The solver is consumed: its queens become the solution.
    pub fn solve(mut self) -> (Solution, SolveStats) {
        let mut restarts = 0;

        loop {
            let tries = self.init();
            let conflicts_after_init = self.get_conflicts();
            let (solved, swaps) = self.local_search();

            if solved {
                let stats = SolveStats {
                    restarts,
                    avg_tries: tries as f64 / self.n as f64,
                    conflicts_after_init,
                    swaps,
                };

                return (Solution::new(self.queen), stats);
            }

            restarts += 1;
            self.reset();
        }
    }

    /// Put the solver back in the state it had after new().
    fn reset(&mut self) {
        self.queen.fill(0);
        self.conflict_queens.clear();
        self.prim_diag.fill(0);
        self.sec_diag.fill(0);
        self.conflict_diags = 0;
        self.free_cols.clear();
        self.free_cols.extend(0..self.n);
    }

    /// This method will try to place a maximum number of queens without conflict,
    /// and then leave the placement of the queens that are in conflict to the algorithm.
    /// Return the number of free columns tried.
    ///
    /// For each queen, one random position is drawn in free_cols, and the
    /// columns are then tried one after the other from there: reading
    /// consecutive entries is much cheaper than one random entry per try,
    /// since free_cols is far larger than the CPU cache for the big sizes.
    /// The entries of free_cols are in a random order (see take_free_col),
    /// so consecutive entries are still random columns.
    fn init(&mut self) -> usize {
        // Number of free columns tried for a queen before accepting a conflict,
        // so that we never loop forever when all the free columns are attacked.
        const MAX_TRIES: usize = 100;

        let mut nb_tries: usize = 0;

        let first_kept_in_conflict = self.n.saturating_sub(self.nb_init_conflicts);

        for i in 0..self.n {
            let len = self.free_cols.len();
            let start = self.rng.random_range(0..len);

            for tries in 1..=MAX_TRIES {
                let r = (start + tries - 1) % len;
                let col = self.free_cols[r];
                nb_tries += 1;

                if !self.check_queen(i, col) || i >= first_kept_in_conflict || tries == MAX_TRIES {
                    self.take_free_col(r);
                    self.place_queen(i, col);
                    break;
                }
            }
        }

        self.conflict_queens = (0..self.n).filter(|&i| self.queen_is_attacked(i)).collect();

        nb_tries
    }

    /// Swap the columns of queens in conflict as long as it doesn't increase
    /// the number of conflicts.
    /// Return whether a solution was found (false if we are trapped),
    /// and the number of swaps made.
    fn local_search(&mut self) -> (bool, usize) {
        const MAX_PASSES_WITHOUT_PROGRESS: usize = 16;

        let mut passes_without_progress: usize = 0;
        let mut conflicts: usize = self.get_conflicts();
        let mut nb_swap: usize = 0;

        while conflicts != 0 && passes_without_progress < MAX_PASSES_WITHOUT_PROGRESS {
            let mut swaps_in_pass = self.swap_pass();

            // No swap, or swaps that only moved the conflicts around:
            // the queens of the set are not enough to solve them.
            if self.get_conflicts() == conflicts {
                swaps_in_pass += self.escape();
            }

            // Still no swap possible at all: we are trapped.
            if swaps_in_pass == 0 {
                break;
            }
            nb_swap += swaps_in_pass;

            let new_conflicts = self.get_conflicts();
            if new_conflicts == conflicts {
                passes_without_progress += 1;
            } else {
                passes_without_progress = 0;
            }
            conflicts = new_conflicts;
        }

        (conflicts == 0, nb_swap)
    }

    /// Try to swap every pair of queens of conflict_queens where at least
    /// one is attacked, keeping the swaps that don't increase the conflicts.
    /// Return the number of swaps kept.
    fn swap_pass(&mut self) -> usize {
        let mut swaps: usize = 0;

        for i in 0..self.conflict_queens.len() {
            for j in (i + 1)..self.conflict_queens.len() {
                let qi: usize = self.conflict_queens[i];
                let qj: usize = self.conflict_queens[j];

                if (self.queen_is_attacked(qi) || self.queen_is_attacked(qj))
                    && self.try_swap(qi, qj)
                {
                    swaps += 1;
                }
            }
        }

        swaps
    }

    /// Called when no swap inside conflict_queens helps any more.
    /// The swaps made so far may have attacked queens outside the set, so
    /// every attacked queen joins it. Then each attacked queen tries a few
    /// random partners anywhere on the board, which also join the set when
    /// the swap is kept. Return the number of swaps kept.
    fn escape(&mut self) -> usize {
        const MAX_PARTNER_TRIES: usize = 32;

        for q in 0..self.n {
            if self.queen_is_attacked(q) && !self.conflict_queens.contains(&q) {
                self.conflict_queens.push(q);
            }
        }

        let mut swaps: usize = 0;

        for k in 0..self.conflict_queens.len() {
            let qi = self.conflict_queens[k];

            if !self.queen_is_attacked(qi) {
                continue;
            }

            for _ in 0..MAX_PARTNER_TRIES {
                let qj = self.rng.random_range(0..self.n);

                if qj != qi && self.try_swap(qi, qj) {
                    swaps += 1;
                    if !self.conflict_queens.contains(&qj) {
                        self.conflict_queens.push(qj);
                    }
                    break;
                }
            }
        }

        swaps
    }

    /// Swap the columns of two queens, and undo the swap if it increases
    /// the number of conflicts. Return whether the swap was kept.
    fn try_swap(&mut self, qi: usize, qj: usize) -> bool {
        let prev_conflicts: usize = self.get_conflicts();
        self.swap_queens_columns(qi, qj);

        if prev_conflicts < self.get_conflicts() {
            self.swap_queens_columns(qi, qj);
            false
        } else {
            true
        }
    }

    /// Remove the entry r of free_cols in O(1), by moving the last entry
    /// in its place. This is what shuffles free_cols during init().
    fn take_free_col(&mut self, r: usize) {
        self.free_cols.swap_remove(r);
    }

    /// @return true if there is a queen in one of the diags.
    fn check_queen(&self, i: usize, j: usize) -> bool {
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] > 0 || self.sec_diag[x] > 0
    }

    /// Get the number of conflicts.
    /// The algorithm ends when this number fall down to 0.
    fn get_conflicts(&self) -> usize {
        self.conflict_diags
    }

    /// Place queen i in column j.
    fn place_queen(&mut self, i: usize, j: usize) {
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.queen[i] = j;
        self.prim_diag[i + j] += 1;
        self.sec_diag[x] += 1;

        if self.prim_diag[i + j] == 2 {
            self.conflict_diags += 1;
        }

        if self.sec_diag[x] == 2 {
            self.conflict_diags += 1;
        }
    }

    /// Remove the queen i from its column.
    fn remove_queen(&mut self, i: usize) {
        let j: usize = self.queen[i];
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] -= 1;
        self.sec_diag[x] -= 1;

        if self.prim_diag[i + j] == 1 {
            self.conflict_diags -= 1;
        }

        if self.sec_diag[x] == 1 {
            self.conflict_diags -= 1;
        }
    }

    /// Return true if a queen has conflicts, and false otherwise.
    fn queen_is_attacked(&self, i: usize) -> bool {
        let j: usize = self.queen[i];
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] > 1 || self.sec_diag[x] > 1
    }

    /// Swap the columns of the two given queens.
    fn swap_queens_columns(&mut self, i: usize, j: usize) {
        let qic = self.queen[i];
        let qjc = self.queen[j];

        self.remove_queen(i);
        self.remove_queen(j);
        self.place_queen(i, qjc);
        self.place_queen(j, qic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u64 = 42;

    fn solves(n: usize) -> bool {
        let (solution, _) = Solver::new(n, SEED).solve();
        solution.is_correct()
    }

    #[test]
    fn solves_a_single_queen_without_any_work() {
        let (solution, stats) = Solver::new(1, SEED).solve();

        assert!(solution.is_correct());
        assert_eq!(stats.restarts, 0);
        assert_eq!(stats.conflicts_after_init, 0);
        assert_eq!(stats.swaps, 0);
    }

    #[test]
    #[should_panic(expected = "no solution for 3 queens")]
    fn refuses_a_size_without_solution() {
        Solver::new(3, SEED);
    }

    #[test]
    fn solves_every_size_from_4_to_100() {
        let failures: Vec<usize> = (4..=100).filter(|&n| !solves(n)).collect();

        assert!(
            failures.is_empty(),
            "{} sizes out of 97 not solved: {failures:?}",
            failures.len()
        );
    }

    #[test]
    fn same_seed_gives_same_solution() {
        let (a, _) = Solver::new(1_000, SEED).solve();
        let (b, _) = Solver::new(1_000, SEED).solve();

        assert_eq!(a.queens(), b.queens());
    }

    #[test]
    fn rarely_restarts_on_small_sizes() {
        let mut restarts = 0;

        for n in 8..=20 {
            for seed in 1..=10 {
                restarts += Solver::new(n, seed).solve().1.restarts;
            }
        }

        // 130 runs: less than one restart per run on average.
        assert!(restarts < 130, "{restarts} restarts in 130 runs");
    }

    #[test]
    fn solves_large_sizes() {
        for n in [1_000, 10_000] {
            assert!(solves(n), "size {n} not solved");
        }
    }
}
