use std::collections::BTreeSet;

pub struct CheckerBoard {
    n: usize,

    /// Number of queens that will be kept in conflict during init().
    nb_init_conflicts: usize,

    /// This will hold the queens positions.
    /// Index is the queen number, value is the queen column.
    queen: Vec<usize>,

    /// This will hold the queens that are in conflicts.
    conflict_queens: Vec<usize>,

    /// Diags are arrays, but their size isn't known at compile time so we use vectors.
    prim_diag: Vec<u8>,
    sec_diag: Vec<u8>,

    /// Sets that will keep the diagonals that have more than 1 queen in it.
    /// This will represent the number of conflicts in our checkerboard.
    conflict_prim_diags: BTreeSet<usize>,
    conflict_sec_diags: BTreeSet<usize>,

    /// free_cols is a vector that will keep the columns that have no queens in it.
    /// It's created with all the n columns,
    /// but the size is reduced during initialisation and falls down to 0.
    free_cols: Vec<usize>,
}

impl CheckerBoard {
    /// Return false for the sizes where no queens placement is possible.
    pub fn has_solution(n: usize) -> bool {
        !matches!(n, 0 | 2 | 3)
    }

    /// Create a new checkerboard,
    /// and make all possible allocations and set them to 0.
    ///
    /// # Panics
    ///
    /// Panics if there is no solution for n queens (see has_solution),
    /// since solve() would never end.
    pub fn new(n: usize) -> CheckerBoard {
        assert!(Self::has_solution(n), "there is no solution for {n} queens");

        let nb_init_conflicts = match n {
            4..=10 => n,
            11..=100 => n / 2,
            101..=1_000 => 30,
            1_001..=100_000 => 50,
            100_001..=1_000_000 => 80,
            _ => 100,
        };

        CheckerBoard {
            n,
            nb_init_conflicts,
            queen: vec![0; n],
            // Reserve some space for the queens that will be kept in conflict during initialisation.
            conflict_queens: Vec::with_capacity(2 * nb_init_conflicts),
            // At first all diagonals contain 0 queens.
            prim_diag: vec![0; 2 * n],
            sec_diag: vec![0; 2 * n],
            conflict_prim_diags: BTreeSet::new(),
            conflict_sec_diags: BTreeSet::new(),
            // At first all columns are free.
            free_cols: (0..n).collect(),
        }
    }

    /// Solve the problem, restarting from a new random placement
    /// each time the local search gets trapped.
    pub fn solve(&mut self) {
        loop {
            self.init();
            if self.local_search() {
                return;
            }
            println!("No solution found, restarting.");
            self.reset();
        }
    }

    /// Put the checkerboard back in the state it had after new().
    fn reset(&mut self) {
        self.queen.fill(0);
        self.conflict_queens.clear();
        self.prim_diag.fill(0);
        self.sec_diag.fill(0);
        self.conflict_prim_diags.clear();
        self.conflict_sec_diags.clear();
        self.free_cols.clear();
        self.free_cols.extend(0..self.n);
    }

    /// This method will try to place a maximum number of queens without conflict,
    /// and then leave the placement of the queens that are in conflict to the algorithm.
    fn init(&mut self) {
        // Number of random columns tried for a queen before accepting a conflict,
        // so that we never loop forever when all the free columns are attacked.
        const MAX_TRIES: usize = 100;

        let mut nb_rand: usize = 0;

        let rfc: usize = self.get_rand_free_col();
        self.place_queen(0, rfc);
        self.pop_last_free_col();

        let first_kept_in_conflict = self.n.saturating_sub(self.nb_init_conflicts);

        for i in 1..self.n {
            for tries in 1..=MAX_TRIES {
                let rfc: usize = self.get_rand_free_col();
                nb_rand += 1;

                let conflict = self.check_queen(i, rfc);

                if !conflict || i >= first_kept_in_conflict || tries == MAX_TRIES {
                    self.place_queen(i, rfc);
                    self.pop_last_free_col();
                    if conflict {
                        self.push_in_conflict_queens(i);
                    }
                    break;
                }
            }
        }

        println!(
            "Average number of random : {}.",
            nb_rand as f64 / self.n as f64
        );
        println!("Number of conflict after init : {}.", self.get_conflicts());
    }

    /// Swap the columns of queens in conflict as long as it doesn't increase
    /// the number of conflicts.
    /// Return true if a solution was found, false if we are trapped.
    fn local_search(&mut self) -> bool {
        let max_loop_with_same_conflict: usize = 4;
        let mut nb_loop_with_same_conflict: usize = 0;
        let mut conflicts: usize = self.get_conflicts();
        let mut nb_swap: usize = 0;

        while conflicts != 0 && nb_loop_with_same_conflict < max_loop_with_same_conflict {
            println!("There is : {} conflicts.", conflicts);

            let mut nb_swap_in_loop: usize = 0;

            for i in 0..self.conflict_queens.len() {
                for j in (i + 1)..self.conflict_queens.len() {
                    let qi: usize = self.conflict_queens[i];
                    let qj: usize = self.conflict_queens[j];
                    if qi != qj && (self.queen_is_attacked(qi) || self.queen_is_attacked(qj)) {
                        let prev_conflicts: usize = self.get_conflicts();
                        self.swap_queens_columns(qi, qj);

                        if prev_conflicts < self.get_conflicts() {
                            self.swap_queens_columns(qi, qj);
                        } else {
                            nb_swap_in_loop += 1;
                        }
                    }
                }
            }

            // No swap could be made: we are trapped.
            if nb_swap_in_loop == 0 {
                break;
            }
            nb_swap += nb_swap_in_loop;

            let new_conflicts = self.get_conflicts();
            if new_conflicts == conflicts {
                nb_loop_with_same_conflict += 1;
            } else {
                nb_loop_with_same_conflict = 0;
            }
            conflicts = new_conflicts;
        }

        println!("Nb swap made : {}.", nb_swap);

        conflicts == 0
    }

    /// Method that get a random free column,
    /// by putting it at the end of the vector in order to
    /// have a O(1) random pop (with pop_last_free_col method).
    fn get_rand_free_col(&mut self) -> usize {
        let len = self.free_cols.len();
        let random = rand::random_range(0..len);

        self.free_cols.swap(random, len - 1);

        self.free_cols[len - 1]
    }

    /// This method is meant to be used after get_rand_free_col.
    fn pop_last_free_col(&mut self) {
        self.free_cols.pop();
    }

    /// This method push a new queen in the conflict queens array.
    /// We also add all the queens that are in conflict with it.
    /// However we do not test if the queens are already in the vector,
    /// the algorithm used is as fast as possible and checking for
    /// already pushed queens would take too much time.
    fn push_in_conflict_queens(&mut self, i: usize) {
        let mut conflict: bool = false;

        for j in 0..i {
            if i.abs_diff(j) == self.queen[i].abs_diff(self.queen[j]) {
                conflict = true;

                self.conflict_queens.push(j);
            }
        }

        if conflict {
            self.conflict_queens.push(i);
        }
    }

    /// @return true if there is a queen in one of the diags.
    fn check_queen(&self, i: usize, j: usize) -> bool {
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] > 0 || self.sec_diag[x] > 0
    }

    /// Get the number of conflicts.
    /// The algorithm ends when this number fall down to 0.
    fn get_conflicts(&self) -> usize {
        self.conflict_prim_diags.len() + self.conflict_sec_diags.len()
    }

    /// Place queen i in column j.
    fn place_queen(&mut self, i: usize, j: usize) {
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.queen[i] = j;
        self.prim_diag[i + j] += 1;
        self.sec_diag[x] += 1;

        if self.prim_diag[i + j] == 2 {
            self.conflict_prim_diags.insert(i + j);
        }

        if self.sec_diag[x] == 2 {
            self.conflict_sec_diags.insert(x);
        }
    }

    /// Remove the queen i from its column.
    fn remove_queen(&mut self, i: usize) {
        let j: usize = self.queen[i];
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] -= 1;
        self.sec_diag[x] -= 1;

        if self.prim_diag[i + j] == 1 {
            self.conflict_prim_diags.remove(&(i + j));
        }

        if self.sec_diag[x] == 1 {
            self.conflict_sec_diags.remove(&x);
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

    pub fn is_correct(&self) -> bool {
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if i.abs_diff(j) == self.queen[i].abs_diff(self.queen[j]) {
                    return false;
                }
            }
        }

        true
    }

    /// Print the checkerboard as a checkerboard.
    pub fn print_checkerboard(&self) {
        print!("+");
        for _ in 0..self.n {
            print!("-+");
        }
        println!();

        for i in 0..self.n {
            print!("|");
            for j in 0..self.n {
                if self.queen[i] == j {
                    print!("o|");
                } else {
                    print!(" |");
                }
            }
            println!();

            print!("+");
            for _ in 0..self.n {
                print!("-+");
            }
            println!();
        }
    }

    pub fn print_checkerboard_as_list(&self) {
        for i in 0..self.n {
            println!("Queen[{}] => {}", i + 1, self.queen[i] + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solves(n: usize) -> bool {
        let mut cb = CheckerBoard::new(n);
        cb.solve();
        cb.is_correct()
    }

    #[test]
    fn solves_a_single_queen() {
        assert!(solves(1));
    }

    #[test]
    #[should_panic(expected = "no solution for 3 queens")]
    fn refuses_a_size_without_solution() {
        CheckerBoard::new(3);
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
    fn solves_large_sizes() {
        for n in [1_000, 10_000] {
            assert!(solves(n), "size {n} not solved");
        }
    }
}
