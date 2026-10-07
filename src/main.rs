mod checker_board;

use std::num::NonZeroUsize;

use anstyle::{AnsiColor, Style};
use checker_board::CheckerBoard;
use clap::Parser;

/// Rust implementation of the N Queens Problem.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Set the number of queens, must be 1 or greater or equal to 4.
    #[arg(short = 'n', long = "size", value_name = "NUMBER", value_parser = parse_size)]
    size: NonZeroUsize,

    /// Print more informations during execution.
    #[arg(short, long)]
    verbose: bool,

    /// Check if the solution found is correct or not. Warning: this is a very slow operation.
    #[arg(short, long)]
    check: bool,
}

/// Parse the number of queens, rejecting the sizes that have no solution.
fn parse_size(arg: &str) -> Result<NonZeroUsize, String> {
    let n: NonZeroUsize = arg.parse().map_err(|e| format!("{e}"))?;

    if CheckerBoard::has_solution(n.get()) {
        Ok(n)
    } else {
        Err(format!("there is no solution for {n} queens"))
    }
}

/// Main Function.
fn main() {
    let args = Args::parse();
    let n = args.size.get();

    let mut cb = CheckerBoard::new(n);
    cb.solve();

    if args.verbose {
        if n <= 50 {
            cb.print_checkerboard();
        } else {
            cb.print_checkerboard_as_list();
        }
    }

    if args.check {
        if cb.is_correct() {
            let green = Style::new().fg_color(Some(AnsiColor::Green.into()));
            anstream::println!("{green}The solution is CORRECT !{green:#}");
        } else {
            let red = Style::new().fg_color(Some(AnsiColor::Red.into()));
            anstream::println!("{red}The solution is not correct !{red:#}");
        }
    }
}
