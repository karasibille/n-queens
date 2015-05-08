extern crate rand;
extern crate num;

use self::rand::Rng;
use std::collections::BTreeSet;

pub struct CheckerBoard {
    n: usize,
    c: usize,
    /// This will hold the queens positions.
    /// Index is the queen number, value is the queen column.
    queen: Vec<usize>,

    /// This will hold the queens that are in conflicts.
    conflict_queens: Vec<usize>,

    /// Diags are Array, but siez isn't known so we use vectors.
    prim_diag: Vec<u8>,
    sec_diag: Vec<u8>,

    /// Sets that will keep the diagonals that have more than 1 queen un it.
    /// This will represents the number of confilcts in our checkerboard.
    conflict_prim_diags: BTreeSet<usize>,
    conflict_sec_diags: BTreeSet<usize>,

    /// free_cols is a vector that will keep the columns that have no queens in it.
    /// It's created with a capacity of n.
    /// But the size can be reduced during initialisation and fall down to 0.
    free_cols: Vec<usize>,
}

impl CheckerBoard {
    /// Create a new checkerboard,
    /// and make all possibe alocations and emset to 0.
    pub fn new(n: usize) -> CheckerBoard {
        let mut cb = CheckerBoard {
            n: n,
            c: 0,
            queen: Vec::with_capacity(n),
            conflict_queens: Vec::new(),
            prim_diag: Vec::with_capacity(2 * n),
            sec_diag: Vec::with_capacity(2 * n),
            conflict_prim_diags: BTreeSet::new(),
            conflict_sec_diags: BTreeSet::new(),
            free_cols: Vec::with_capacity(n),
        };

        for _ in 0..cb.queen.capacity() {
            cb.queen.push(0);
        }


        // Set the number of queen that will be kept in conflict during init().
        cb.c = match cb.n {
            r @ 4...10 => r,
            r @ 10...100 => r / 2,
            100...1000 => 30,
            1000...100000 => 50,
            100000...1000000 => 80,
            _ => 100,
        };

        // Reserve some space for the queens that will be kept in confilct during initialisation.
        cb.conflict_queens.reserve(2 * cb.c);

        // At first all diagonals contains 0 queens.
        for _ in 0..cb.prim_diag.capacity() {
            cb.prim_diag.push(0);
            cb.sec_diag.push(0);
        }

        // At first all columns are free.
        for i in 0..cb.free_cols.capacity() {
            cb.free_cols.push(i);
        }

        cb
    }

    pub fn solve(&mut self) {
        self.init();
        self.local_search();
    }

    /// This methods will try de place a maximum number of queens without confilct,
    /// and then leave the placement of the queens tha are in conflict to the algorithme.
    fn init(&mut self) {
        let mut nb_rand: usize = 0;

        let rfc: usize = self.get_rand_free_col();
        self.place_queen(0, rfc);
        self.pop_last_free_col();

        for i in 1..self.queen.capacity() {
            let mut conflict: bool = true;

            while conflict {
                let rfc: usize = self.get_rand_free_col();
                nb_rand += 1;

                conflict = self.check_queen(i, rfc);

                if i < self.n - self.c {
                    if !conflict {
                        self.place_queen(i, rfc);
                        self.pop_last_free_col();
                    }
                } else {
                    self.place_queen(i, rfc);
                    self.pop_last_free_col();
                    self.push_in_conflict_queens(i);
                    conflict = false;
                }
            }
        }

        println!("Average number of random : {}.",
                 nb_rand as f64 / self.queen.capacity() as f64);
        println!("Number of conflict after init : {}.", self.get_conflicts());
    }

    fn local_search(&mut self) -> bool {

        let mut nb_loop_with_same_conflict: usize = 0;
        let max_loop_with_same_conflict: usize = 4;
        let mut conflicts_prev_loop: usize = self.get_conflicts();
        let mut nb_swap: usize = 1;
        let mut conflicts: usize;


        // While we are not trapped (nb_swap != 0, i.e we have made some swap in the loop)
        // or the number of conflicts has not fall down to 0.
        while nb_swap != 0 && conflicts_prev_loop != 0 &&
              nb_loop_with_same_conflict < max_loop_with_same_conflict {

            println!("There is : {} conflicts.", conflicts_prev_loop);

            for i in 0..self.conflict_queens.len() {
                for j in (i + 1)..self.conflict_queens.len() {

                    let qi: usize = self.conflict_queens[i];
                    let qj: usize = self.conflict_queens[j];
                    if qi != qj && (self.queen_is_attacked(qi) || self.queen_is_attacked(qj)) {

                        let prev_conflicts: usize = self.get_conflicts();
                        self.swap_queens_columns(qi, qj);

                        conflicts = self.get_conflicts();
                        if prev_conflicts < conflicts {
                            self.swap_queens_columns(qi, qj);
                        } else {
                            nb_swap += 1;
                        }
                    }
                }
            }

            conflicts = self.get_conflicts();
            if conflicts_prev_loop == conflicts {
                nb_loop_with_same_conflict += 1;
            }
            conflicts_prev_loop = conflicts;
        }

        println!("Nb swap made : {}.", nb_swap);

        if self.get_conflicts() == 0 {
            true
        } else {
            false
        }
    }

    /// Method that get a random free column,
    /// by putting it at the end of the vector in order to
    /// have a O(1) random pop (with pop_last_free_col method).
    fn get_rand_free_col(&mut self) -> usize {
        let len = self.free_cols.len();
        let random = rand::thread_rng().gen_range(0, len);

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
    /// the algorithme used is as fast as possible and checking for
    /// already pushed queens will take to much time.
    fn push_in_conflict_queens(&mut self, i: usize) {
        let mut conflict: bool = false;

        for j in 0usize..i {
            if num::abs(i as isize - j as isize) ==
               num::abs(self.queen[i] as isize - self.queen[j] as isize) {
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
    /// The algorithme end when this number fall down to 0.
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

    /// Remove the queen i from is column.
    fn remove_queen(&mut self, i: usize) {
        let j: usize = self.queen[i];
        let x: usize = (i as isize - j as isize + self.n as isize) as usize;

        self.prim_diag[i + j] -= 1;
        self.sec_diag[x] -= 1;

        if self.prim_diag[i + j] == 1 {
            self.conflict_prim_diags.remove(&(i + j));
        }

        if self.sec_diag[x] == 1 {
            self.conflict_sec_diags.remove(&(x));
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

    pub fn is_correct(&mut self) -> bool {
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if num::abs(i as isize - j as isize) ==
                   num::abs(self.queen[i] as isize - self.queen[j] as isize) {
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
        println!("");

        for i in 0..self.n {
            print!("|");
            for j in 0..self.n {
                if self.queen[i] == j {
                    print!("o|");
                } else {
                    print!(" |");
                }
            }
            println!("");

            print!("+");
            for _ in 0..self.n {
                print!("-+");
            }
            println!("");
        }
    }

    pub fn print_checkerboard_as_list(&self) {
        for i in 0..self.n {
            println!("Queen[{}] => {}", i + 1, self.queen[i] + 1);
        }
    }
}
