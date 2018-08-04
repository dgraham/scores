mod bigrams;
use bigrams::Bigrams;

pub struct Scorer<'a> {
    term: &'a str,
    term_bi: Bigrams,
    line_bi: Bigrams,
}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str) -> Self {
        Scorer {
            term,
            term_bi: Bigrams::build(term),
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
        } else if term.len() == 1 {
            if line.starts_with(term) {
                0.2
            } else if line.contains(term) {
                0.1
            } else {
                0.0
            }
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
    let stop = a
        .chars()
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
    let stop = a
        .chars()
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

#[cfg(test)]
mod tests {
    use super::Scorer;

    #[test]
    fn it_awards_no_points_for_empty_term() {
        let mut scorer = Scorer::new("");
        assert_eq!(0, scorer.score("hello"));
    }

    #[test]
    fn it_awards_no_points_for_empty_input() {
        let mut scorer = Scorer::new("hello");
        assert_eq!(0, scorer.score(""));
    }

    #[test]
    fn it_awards_no_points_for_term_longer_than_input() {
        let mut scorer = Scorer::new("hello");
        assert_eq!(0, scorer.score("hi"));
    }

    #[test]
    fn it_awards_more_points_for_exact_match_than_close_match() {
        let mut scorer = Scorer::new("hello");
        let a = scorer.score("hello");
        let b = scorer.score("hellolo");
        assert!(a > 0);
        assert!(b > 0);
        assert!(a > b);
    }

    #[test]
    fn it_awards_anchor_points() {
        let mut scorer = Scorer::new("hello");
        let a = scorer.score("hel--lo");
        let b = scorer.score("-hello-");
        assert!(a > 0);
        assert!(b > 0);
        assert!(a > b);
    }
}
