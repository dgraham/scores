extern crate getopts;
extern crate scores;

use std::env;
use std::io::{self, BufRead};
use std::process::exit;

use getopts::Options;
use scores::{Anchor, Scorer};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Display this message")
        .optopt("l", "limit", "Maximum number of results", "MAX")
        .optopt("E", "exclude", "Exclude the specified entry", "NAME")
        .optopt("r", "reference", "A reference to match against", "NAME");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            usage(&opts);
            println!("{}", e);
            exit(1);
        }
    };

    if matches.opt_present("h") {
        usage(&opts);
        exit(0);
    }

    let term = match matches.free.first() {
        Some(free) => free,
        None => exit(0),
    };

    let limit: Option<usize> = matches.opt_str("l").and_then(|max| max.parse().ok());
    search(&term, limit, matches.opt_str("E"), matches.opt_str("r"));
}

fn search(term: &str, limit: Option<usize>, exclude: Option<String>, reference: Option<String>) {
    let mut scorer = Scorer::new(term, reference.map(|a| Anchor::new(&a)));

    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| match exclude {
            Some(ref exclude) => line != exclude,
            None => true,
        });

    let matches = scorer.rank(lines);

    let top = match limit {
        Some(max) if max < matches.len() => &matches[..max],
        _ => &matches,
    };

    for &(ref text, _) in top {
        println!("{}", text);
    }
}

fn usage(opts: &Options) {
    let brief = "String similarity scoring\n\nUsage:\n    scores term [options]";
    println!("{}", opts.usage(brief));
}
