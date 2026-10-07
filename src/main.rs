use std::io::{self, BufWriter, ErrorKind, Write};
use std::num::NonZeroUsize;
use std::process::ExitCode;

use anstyle::{AnsiColor, Style};
use clap::Parser;
use n_queens_problem::{Solver, has_solution};

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

    if has_solution(n.get()) {
        Ok(n)
    } else {
        Err(format!("there is no solution for {n} queens"))
    }
}

/// Main Function.
fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        // The reader closed the pipe early (e.g. `| head`): nothing is wrong.
        Err(e) if e.kind() == ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Solve and write the results to stdout, through a single buffered writer
/// so that the output is written in large chunks and I/O errors are reported.
fn run(args: &Args) -> io::Result<()> {
    let n = args.size.get();
    let seed = args.seed.unwrap_or_else(rand::random);

    let (solution, stats) = Solver::new(n, seed).solve();

    let stdout = anstream::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if args.verbose {
        writeln!(out, "Seed: {seed}")?;
        writeln!(out, "Restarts: {}", stats.restarts)?;
        writeln!(
            out,
            "Average number of columns tried per queen: {:.2}",
            stats.avg_tries
        )?;
        writeln!(out, "Conflicts after init: {}", stats.conflicts_after_init)?;
        writeln!(out, "Swaps made: {}", stats.swaps)?;

        if n <= 50 {
            solution.write_board(&mut out)?;
        } else {
            solution.write_list(&mut out)?;
        }
    }

    if args.check {
        if solution.is_correct() {
            let green = Style::new().fg_color(Some(AnsiColor::Green.into()));
            writeln!(out, "{green}The solution is CORRECT !{green:#}")?;
        } else {
            let red = Style::new().fg_color(Some(AnsiColor::Red.into()));
            writeln!(out, "{red}The solution is not correct !{red:#}")?;
        }
    }

    out.flush()
}
