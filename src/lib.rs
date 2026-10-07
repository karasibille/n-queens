//! A min-conflicts solver of the n queens problem: place n queens on an
//! n×n board so that no two queens attack each other.
//!
//! ```
//! use n_queens_problem::Solver;
//!
//! let (solution, stats) = Solver::new(8, 42).solve();
//! assert!(solution.is_correct());
//! assert_eq!(solution.size(), 8);
//! println!("{} restarts", stats.restarts);
//! ```

mod solution;
mod solver;

pub use solution::Solution;
pub use solver::{SolveStats, Solver};

/// Return false for the sizes where no queens placement is possible.
pub fn has_solution(n: usize) -> bool {
    !matches!(n, 0 | 2 | 3)
}
