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

    /// Check if the solution found is correct or not.
    #[arg(short, long)]
    check: bool,

    /// Seed of the random generator, to reproduce a run (random by default).
    #[arg(short, long)]
    seed: Option<u64>,
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

    let seed = args.seed.unwrap_or_else(rand::random);

    let mut cb = CheckerBoard::new(n, seed);
    let stats = cb.solve();

    if args.verbose {
        println!("Seed: {seed}");
        println!("Restarts: {}", stats.restarts);
        println!(
            "Average number of random draws per queen: {:.2}",
            stats.avg_random_draws
        );
        println!("Conflicts after init: {}", stats.conflicts_after_init);
        println!("Swaps made: {}", stats.swaps);

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
