use std::env;
use std::error::Error;
use std::io::{self, BufRead};
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
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help message")
        .optflag("V", "version", "Print version information")
        .optopt("l", "limit", "Maximum number of results", "MAX")
        .optopt("E", "exclude", "Exclude the specified entry", "NAME")
        .optopt("r", "reference", "A reference to match against", "NAME");

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

    let limit: Option<usize> = matches.opt_str("limit").and_then(|max| max.parse().ok());
    search(
        term,
        limit,
        matches.opt_str("exclude"),
        matches.opt_str("reference"),
    );
    Ok(())
}

fn search(term: &str, limit: Option<usize>, exclude: Option<String>, reference: Option<String>) {
    let mut scorer = Scorer::new(term, reference.map(|a| Anchor::new(&a)));

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

    let top = match limit {
        Some(max) if max < matches.len() => &matches[..max],
        _ => &matches,
    };

    for (text, _) in top {
        println!("{}", text);
    }
}

fn usage(opts: &Options) {
    let brief = "String similarity scoring\n\nUsage:\n    scores term [options]";
    println!("{}", opts.usage(brief));
}
