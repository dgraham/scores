use std::env;
use std::error::Error;
use std::io::{self, BufRead, ErrorKind};
use std::process::ExitCode;

use getopts::Options;
use scores::{Anchor, Scorer};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = args()?;
    let opts = opts();
    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("help") {
        usage(&opts);
        return Ok(());
    }

    if matches.opt_present("version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let term = match matches.free.first() {
        Some(free) => free,
        None => return Ok(()),
    };

    let limit = matches
        .opt_str("limit")
        .map(|value| {
            value.parse::<usize>().map_err(|_| {
                io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid --limit value: {value}"),
                )
            })
        })
        .transpose()?;

    search(
        term,
        limit,
        matches.opt_str("exclude"),
        matches.opt_str("reference"),
    );
    Ok(())
}

fn opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help message");
    opts.optflag("V", "version", "Print version information");
    opts.optopt("l", "limit", "Maximum number of results", "MAX");
    opts.optopt("E", "exclude", "Exclude the specified entry", "NAME");
    opts.optopt("r", "reference", "A reference to match against", "NAME");
    opts
}

fn args() -> Result<Vec<String>, io::Error> {
    env::args_os()
        .map(|arg| {
            arg.into_string().map_err(|arg| {
                io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid UTF-8 argument: {}", arg.to_string_lossy()),
                )
            })
        })
        .collect()
}

fn search(term: &str, limit: Option<usize>, exclude: Option<String>, reference: Option<String>) {
    let mut scorer = Scorer::new(term, reference.map(|a| Anchor::new(&a)), limit);

    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter(|line| match exclude {
            Some(ref exclude) => line != exclude,
            None => true,
        });

    let matches = scorer.rank(lines);

    for score in matches {
        println!("{}", score.line);
    }
}

fn usage(opts: &Options) {
    let brief = "String similarity scoring\n\nUsage:\n    scores term [options]";
    println!("{}", opts.usage(brief));
}
