mod bigrams;
use bigrams::Bigrams;

pub struct Scorer<'a> {
    term: &'a str,
    term_bi: Bigrams,
    line_bi: Bigrams,
}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str) -> Self {
        let mut term_bi = Bigrams::new();
        term_bi.insert(term);

        Scorer {
            term,
            term_bi,
            line_bi: Bigrams::new(),
        }
    }

    pub fn score(&mut self, line: &str) -> u32 {
        let term = self.term;
        if term.len() > line.len() {
            return 0;
        }

        let total = if term == line {
            1.0 * 2.0
        } else {
            self.line_bi.clear();
            self.line_bi.insert(line);
            let similarity = self.term_bi.similarity(&self.line_bi);
            if similarity == 0.0 {
                0.0
            } else if similarity == 1.0 {
                1.0 * 2.0
            } else {
                similarity + bonus(prefix(line, term, 5)) + bonus(suffix(line, term, 5))
            }
        };

        (total * 10000.0) as u32
    }
}

fn prefix(a: &str, b: &str, limit: usize) -> usize {
    let stop = a.chars()
        .zip(b.chars())
        .enumerate()
        .take(limit)
        .find(|&(_, (x, y))| x != y);
    match stop {
        Some((ix, _)) => ix,
        None => limit,
    }
}

fn suffix(a: &str, b: &str, limit: usize) -> usize {
    let stop = a.chars()
        .rev()
        .zip(b.chars().rev())
        .enumerate()
        .take(limit)
        .find(|&(_, (x, y))| x != y);
    match stop {
        Some((ix, _)) => ix,
        None => limit,
    }
}

fn bonus(shared: usize) -> f32 {
    match shared {
        0 => 0.0,
        len if len >= 2 => 0.2,
        _ => 0.05,
    }
}
