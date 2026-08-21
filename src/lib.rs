use std::cmp::Ordering;

use regex::{Regex, RegexBuilder, escape};

mod anchor;
mod bigrams;
mod topk;

pub use anchor::Anchor;
use topk::TopK;

pub struct Scorer<'a> {
    re: Option<Regex>,
    term: &'a str,
    anchor: Option<Anchor>,
    limit: Option<usize>,
}

#[derive(Debug)]
pub struct Score {
    index: usize,
    pub line: String,
    pub regex: f32,
    pub similarity: f32,
}

impl Score {
    pub fn value(&self) -> f32 {
        self.regex * 0.9 + self.similarity * 0.1
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value()
            .total_cmp(&other.value())
            .then_with(|| other.index.cmp(&self.index))
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Score {}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str, anchor: Option<Anchor>, limit: Option<usize>) -> Self {
        Scorer {
            re: pattern(term),
            term,
            anchor,
            limit,
        }
    }

    fn score(&mut self, index: usize, line: impl Into<String>) -> Option<Score> {
        let line = line.into();

        if self.term.len() > line.len() {
            return None;
        }

        let range = self.re.as_ref()?.find(&line).map(|m| m.end() - m.start())?;
        let penalty = ((line.len() as f32) + 1.0) / 100.0;
        let ideal = (self.term.len() as f32) + ((self.term.len() as f32) + 1.0) / 100.0;
        let regex = ideal / ((range as f32) + penalty);

        let similarity = self.anchor.as_mut().map_or(0.0, |a| a.score(&line));

        Some(Score {
            index,
            line,
            regex,
            similarity,
        })
    }

    pub fn rank<I, T>(&mut self, lines: I) -> Vec<Score>
    where
        I: Iterator<Item = T>,
        T: Into<String>,
    {
        match self.limit {
            None => {
                let mut matches: Vec<_> = lines
                    .enumerate()
                    .filter_map(|(index, line)| self.score(index, line))
                    .collect();
                matches.sort_by(|a, b| b.cmp(a));
                matches
            }
            Some(limit) => {
                let mut matches: TopK<Score> = TopK::new(limit);
                for (index, line) in lines.enumerate() {
                    if let Some(score) = self.score(index, line) {
                        matches.push(score);
                    }
                }
                matches.into_sorted_vec()
            }
        }
    }
}

fn pattern(term: &str) -> Option<Regex> {
    let mut term = String::from(term);
    term.pop().and_then(|last| {
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
        let mut scorer = Scorer::new("", None, None);
        assert!(scorer.score(0, "hello").is_none())
    }

    #[test]
    fn it_awards_no_points_for_empty_input() {
        let mut scorer = Scorer::new("hello", None, None);
        assert!(scorer.score(0, "").is_none());
    }

    #[test]
    fn it_awards_no_points_for_term_longer_than_input() {
        let mut scorer = Scorer::new("hello", None, None);
        assert!(scorer.score(0, "hi").is_none());
    }

    #[test]
    fn it_awards_more_points_for_exact_match_than_close_match() {
        let mut scorer = Scorer::new("hello", None, None);
        let a = scorer.score(0, "hello").unwrap();
        let b = scorer.score(1, "hellolo").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_awards_full_regex_score_for_exact_match() {
        let mut scorer = Scorer::new("hello", None, None);
        let score = scorer.score(0, "hello").unwrap();

        assert_eq!(score.regex, 1.0);
    }

    #[test]
    fn it_awards_more_points_for_shorter_match_range() {
        let mut scorer = Scorer::new("hello", None, None);
        let a = scorer.score(0, "-hel-lo").unwrap();
        let b = scorer.score(1, "hel--lo").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_awards_more_points_for_shorter_input() {
        let mut scorer = Scorer::new("hello", None, None);
        let a = scorer.score(0, "-hello").unwrap();
        let b = scorer.score(1, "--hello").unwrap();
        assert!(a.regex > 0.0);
        assert!(b.regex > 0.0);
        assert!(a.regex > b.regex);
    }

    #[test]
    fn it_matches_case_insensitively() {
        let mut scorer = Scorer::new("hello", None, None);
        let a = scorer.score(0, "Hello").unwrap();
        let b = scorer.score(1, "hello").unwrap();
        assert!(a.regex > 0.0);
        assert_eq!(a.regex, b.regex);
    }

    #[test]
    fn it_matches_case_sensitively() {
        let mut scorer = Scorer::new("Hello", None, None);
        let a = scorer.score(0, "Hello");
        let b = scorer.score(1, "hello");
        assert!(a.is_some());
        assert!(b.is_none());
        assert!(a.unwrap().regex > 0.0);
    }

    #[test]
    fn it_ranks_borrowed_lines() {
        let lines = ["hellolo", "hello"];
        let mut scorer = Scorer::new("hello", None, None);
        let matches = scorer.rank(lines.into_iter());

        assert_eq!(matches[0].line, "hello");
        assert_eq!(matches[1].line, "hellolo");
    }

    #[test]
    fn it_limits_ranked_lines() {
        let lines = ["hellolo", "hello", "-hello"];
        let mut scorer = Scorer::new("hello", None, Some(2));
        let matches = scorer.rank(lines.into_iter());

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, "hello");
        assert_eq!(matches[1].line, "-hello");
    }
}
