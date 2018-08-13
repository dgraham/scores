extern crate regex;
use regex::{escape, Regex, RegexBuilder};

mod anchor;
mod bigrams;

pub use anchor::Anchor;

pub struct Scorer<'a> {
    re: Option<Regex>,
    term: &'a str,
    anchor: Option<Anchor>,
}

#[derive(Debug)]
pub struct Score {
    pub regex: f32,
    pub similarity: f32,
}

impl Score {
    pub fn value(&self, min: f32, max: f32) -> u32 {
        let regex = (self.regex - min) / (max - min);
        let weighted = regex * 0.9 + self.similarity * 0.1;
        (weighted * 100_000.0) as u32
    }
}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str, anchor: Option<Anchor>) -> Self {
        Scorer {
            re: pattern(term),
            term,
            anchor,
        }
    }

    fn score(&mut self, line: &str) -> Option<Score> {
        if self.term.len() > line.len() {
            return None;
        }

        let anchor = &mut self.anchor;
        self.re.as_ref().and_then(|re| {
            re.find(line).and_then(|m| {
                let range = m.end() - m.start() + 1;
                let penalty = ((line.len() as f32) + 1.0) / 100.0;
                let score = 1.0 / ((range as f32) + penalty);
                Some(Score {
                    regex: score,
                    similarity: anchor.as_mut().map_or(0.0, |a| a.score(line)),
                })
            })
        })
    }

    pub fn rank<I>(&mut self, lines: I) -> Vec<(String, Score)>
    where
        I: Iterator<Item = String>,
    {
        let mut min = 0.0;
        let mut max = 0.0;

        let mut matches: Vec<_> = lines
            .filter_map(|line| match self.score(&line) {
                None => None,
                Some(val) => {
                    if val.regex < min {
                        min = val.regex;
                    }
                    if val.regex > max {
                        max = val.regex;
                    }
                    Some((line, val))
                }
            })
            .collect();

        matches.sort_by(|a, b| a.1.value(min, max).cmp(&b.1.value(min, max)).reverse());
        matches
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
        let mut scorer = Scorer::new("", None);
        assert!(scorer.score("hello").is_none())
    }

    #[test]
    fn it_awards_no_points_for_empty_input() {
        let mut scorer = Scorer::new("hello", None);
        assert!(scorer.score("").is_none());
    }

    #[test]
    fn it_awards_no_points_for_term_longer_than_input() {
        let mut scorer = Scorer::new("hello", None);
        assert!(scorer.score("hi").is_none());
    }

    #[test]
    fn it_awards_more_points_for_exact_match_than_close_match() {
        let mut scorer = Scorer::new("hello", None);
        let a = scorer.score("hello").unwrap();
        let b = scorer.score("hellolo").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_awards_more_points_for_shorter_match_range() {
        let mut scorer = Scorer::new("hello", None);
        let a = scorer.score("-hel-lo").unwrap();
        let b = scorer.score("hel--lo").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_awards_more_points_for_shorter_input() {
        let mut scorer = Scorer::new("hello", None);
        let a = scorer.score("-hello").unwrap();
        let b = scorer.score("--hello").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_matches_case_insensitively() {
        let mut scorer = Scorer::new("hello", None);
        let a = scorer.score("Hello").unwrap();
        let b = scorer.score("hello").unwrap();
        assert!(a.regex > 0.0);
        assert_eq!(a.regex, b.regex);
    }

    #[test]
    fn it_matches_case_sensitively() {
        let mut scorer = Scorer::new("Hello", None);
        let a = scorer.score("Hello");
        let b = scorer.score("hello");
        assert!(a.is_some());
        assert!(b.is_none());
        assert!(a.unwrap().regex > 0.0);
    }
}
