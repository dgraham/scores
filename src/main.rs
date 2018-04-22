extern crate getopts;

use std::env;
use std::io::{self, BufRead};
use std::process::exit;

use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this message").optopt(
        "l",
        "limit",
        "Maximum number of results",
        "MAX",
    );

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
        None => {
            usage(&opts);
            exit(1);
        }
    };

    let limit: Option<usize> = matches.opt_str("l").and_then(|max| max.parse().ok());
    search(&term, limit);
}

fn search(term: &str, limit: Option<usize>) {
    let stdin = io::stdin();
    let mut matches: Vec<_> = stdin
        .lock()
        .lines()
        .filter_map(|line| {
            line.ok().and_then(|full| match score(&full, term) {
                0 => None,
                val => Some((full, val)),
            })
        })
        .collect();

    matches.sort_by(|a, b| b.1.cmp(&a.1));

    let top = match limit {
        Some(max) if max < matches.len() => &matches[..max],
        _ => &matches,
    };

    for &(ref text, _) in top {
        println!("{}", text);
    }
}

fn score(line: &str, term: &str) -> u32 {
    match line.contains(term) {
        true => 1,
        false => 0,
    }
}

fn usage(opts: &Options) {
    let brief = "String similarity scoring\n\nUsage:\n    scores term [options]";
    println!("{}", opts.usage(brief));
}
