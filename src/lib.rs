extern crate regex;
use regex::{escape, Regex, RegexBuilder};

mod bigrams;
use bigrams::Bigrams;

pub struct Scorer<'a> {
    re: Option<Regex>,
    term: &'a str,
    term_bi: Bigrams,
    line_bi: Bigrams,
}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str) -> Self {
        Scorer {
            re: pattern(term),
            term,
            term_bi: Bigrams::build(term),
            line_bi: Bigrams::new(),
        }
    }

    pub fn score(&self, line: &str) -> u32 {
        if self.term.len() > line.len() {
            return 0;
        }

        self.re.as_ref().map_or(0, |re| {
            re.find(line).map_or(0, |m| {
                let range = m.end() - m.start() + 1;
                let penalty = ((line.len() as f32) + 1.0) / 100.0;
                let score = 1.0 / ((range as f32) + penalty);
                (score * 100_000.0) as u32
            })
        })
    }
}

fn pattern(term: &str) -> Option<Regex> {
    let mut term = String::from(term);
    term.pop().map_or(None, |last| {
        let mut insensitive = last.is_lowercase();

        let mut pattern = String::new();
        for ch in term.chars() {
            let esc = escape(&ch.to_string());
            pattern.push_str(&esc);
            pattern.push_str("[^");
            pattern.push_str(&esc);
            pattern.push_str("]*");

            if ch.is_uppercase() {
                insensitive = false;
            }
        }
        pattern.push(last);

        RegexBuilder::new(&pattern)
            .case_insensitive(insensitive)
            .build()
            .ok()
    })
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
    fn it_awards_more_points_for_shorter_match_range() {
        let mut scorer = Scorer::new("hello");
        let a = scorer.score("-hel-lo");
        let b = scorer.score("hel--lo");
        assert!(a > 0);
        assert!(b > 0);
        assert!(a > b);
    }

    #[test]
    fn it_awards_more_points_for_shorter_input() {
        let mut scorer = Scorer::new("hello");
        let a = scorer.score("-hello");
        let b = scorer.score("--hello");
        assert!(a > 0);
        assert!(b > 0);
        assert!(a > b);
    }

    #[test]
    fn it_matches_case_insensitively() {
        let mut scorer = Scorer::new("hello");
        let a = scorer.score("Hello");
        let b = scorer.score("hello");
        assert!(a > 0);
        assert_eq!(a, b);
    }

    #[test]
    fn it_matches_case_sensitively() {
        let mut scorer = Scorer::new("Hello");
        let a = scorer.score("Hello");
        let b = scorer.score("hello");
        assert!(a > 0);
        assert_eq!(0, b);
    }
}
