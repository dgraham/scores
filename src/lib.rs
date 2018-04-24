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

pub struct Bigrams {
    grams: Vec<u64>,
}

impl Bigrams {
    pub fn new() -> Self {
        Bigrams { grams: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.grams.clear();
    }

    pub fn insert(&mut self, text: &str) {
        bigrams(text, &mut self.grams);
        self.grams.sort();
    }

    pub fn len(&self) -> usize {
        self.grams.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grams.is_empty()
    }

    // Returns a string similarity score between 0.0 and 1.0.
    //
    // https://en.wikipedia.org/wiki/Sørensen–Dice_coefficient
    pub fn similarity(&self, other: &Bigrams) -> f32 {
        if self.is_empty() || other.is_empty() {
            return 0.0;
        }

        let bigrams1 = &self.grams;
        let bigrams2 = &other.grams;

        let card1 = bigrams1.len();
        let card2 = bigrams2.len();

        let mut matches = 0;
        let mut i = 0;
        let mut j = 0;
        while i < card1 && j < card2 {
            if bigrams1[i] == bigrams2[j] {
                matches += 1;
                i += 1;
                j += 1;
            } else if bigrams1[i] < bigrams2[j] {
                i += 1;
            } else {
                j += 1;
            }
        }

        (matches as f32 * 2.0) / (card1 + card2) as f32
    }
}

fn bigrams(text: &str, container: &mut Vec<u64>) {
    let mut first = None;
    for ch in text.chars() {
        if let Some(hi) = first {
            let gram = ((hi as u64) << 32) | ch as u64;
            container.push(gram);
        }
        first = Some(ch);
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

#[cfg(test)]
mod tests {
    use super::Bigrams;

    #[test]
    fn it_awards_no_score_for_short_text() {
        let mut a = Bigrams::new();
        a.insert("h");

        let mut b = Bigrams::new();
        b.insert("hello");

        assert_eq!(0.0, a.similarity(&b));
        assert_eq!(0.0, b.similarity(&a));
    }

    #[test]
    fn it_awards_full_score_for_identical_text() {
        let mut a = Bigrams::new();
        a.insert("hello");

        let mut b = Bigrams::new();
        b.insert("hello");

        assert_eq!(1.0, a.similarity(&b));
    }

    #[test]
    fn it_awards_more_points_for_closer_matches() {
        let mut a = Bigrams::new();
        a.insert("he");

        let mut b = Bigrams::new();
        b.insert("hello");

        let mut c = Bigrams::new();
        c.insert("helo");

        assert!(a.similarity(&b) > 0.0);
        assert!(c.similarity(&b) > 0.0);
        assert!(c.similarity(&b) > a.similarity(&b));
    }
}
