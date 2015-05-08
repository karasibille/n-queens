extern crate getopts;
extern crate term;
mod checker_board;

use std::io::prelude::*;
use checker_board::CheckerBoard;
use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

/// Main Function.
fn main() {
    let mut t = term::stdout().unwrap();
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "size", "set the number of queens, must be greater or equal to 1.", "NUMBER");
    opts.optflag("v", "verbose", "print more informations during execution");
    opts.optflag("c", "check", "check if the solution found is correct or not. Warning : This is a very slow operation.");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            println!("{}\n", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let verbose: bool = matches.opt_present("v");
    let check: bool = matches.opt_present("c");

    let output: String = match matches.opt_str("n") {
        Some(x) => x,
        None => unreachable!(),
    };

    let n: usize = match output.trim().parse() {
        Ok(num) => {
            if num == 0 {
                print_usage(&program, opts);
                return;
            }
            num
        },
        Err(_) => {
            print_usage(&program, opts);
            return;
        },
    };

    let mut cb = CheckerBoard::new(n);
    cb.solve();

    if verbose {
        if n <= 50 {
            cb.print_checkerboard();
        } else {
            cb.print_checkerboard_as_list();
        }
    }

    if check {
        if cb.is_correct() {
            t.fg(term::color::GREEN).unwrap();
            (writeln!(t, "The solution is CORRECT !")).unwrap();
        } else {
            t.fg(term::color::RED).unwrap();
            (writeln!(t, "The solution is not correct !")).unwrap();
        }
    }
}
