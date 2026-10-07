use std::io::{self, Write};

/// A placement of n queens, one per row, as returned by Solver::solve().
pub struct Solution {
    /// Index is the row, value is the column of the queen on that row.
    queens: Vec<usize>,
}

impl Solution {
    pub(crate) fn new(queens: Vec<usize>) -> Solution {
        Solution { queens }
    }

    /// Number of queens, which is also the size of the board.
    pub fn size(&self) -> usize {
        self.queens.len()
    }

    /// The column of each queen, by row.
    pub fn queens(&self) -> &[usize] {
        &self.queens
    }

    /// Check the solution from scratch, without trusting the counters
    /// updated during the search: one queen per column and per diagonal.
    pub fn is_correct(&self) -> bool {
        let n = self.size();
        let mut used_cols = vec![false; n];
        let mut used_prim_diags = vec![false; 2 * n];
        let mut used_sec_diags = vec![false; 2 * n];

        for (i, &j) in self.queens.iter().enumerate() {
            let x = i + n - j;

            if used_cols[j] || used_prim_diags[i + j] || used_sec_diags[x] {
                return false;
            }

            used_cols[j] = true;
            used_prim_diags[i + j] = true;
            used_sec_diags[x] = true;
        }

        true
    }

    /// Write the board as a drawing, one row per queen.
    pub fn write_board(&self, out: &mut impl Write) -> io::Result<()> {
        let n = self.size();
        let border = format!("+{}", "-+".repeat(n));

        writeln!(out, "{border}")?;

        for &col in &self.queens {
            write!(out, "|")?;
            for j in 0..n {
                write!(out, "{}|", if col == j { 'o' } else { ' ' })?;
            }
            writeln!(out)?;
            writeln!(out, "{border}")?;
        }

        Ok(())
    }

    /// Write the board as a list of columns, one line per queen.
    pub fn write_list(&self, out: &mut impl Write) -> io::Result<()> {
        for (i, &col) in self.queens.iter().enumerate() {
            writeln!(out, "Queen[{}] => {}", i + 1, col + 1)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_a_placement() {
        assert!(Solution::new(vec![1, 3, 0, 2]).is_correct());

        // Two queens on the same column.
        assert!(!Solution::new(vec![1, 1, 0, 2]).is_correct());

        // Two queens on the same diagonal.
        assert!(!Solution::new(vec![0, 1, 3, 2]).is_correct());
    }

    #[test]
    fn writes_the_board_with_one_queen_per_row() {
        let solution = Solution::new(vec![1, 3, 0, 2]);

        let mut out = Vec::new();
        solution.write_board(&mut out).unwrap();
        let text = String::from_utf8(out).unwrap();

        // A border line, then a row and a border line per queen.
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 9);
        assert_eq!(lines[0], "+-+-+-+-+");
        assert_eq!(lines[1], "| |o| | |");
        for (i, row) in lines.iter().skip(1).step_by(2).enumerate() {
            assert_eq!(row.matches('o').count(), 1, "row {i}: {row}");
        }

        let mut out = Vec::new();
        solution.write_list(&mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_eq!(
            text,
            "Queen[1] => 2\nQueen[2] => 4\nQueen[3] => 1\nQueen[4] => 3\n"
        );
    }
}
