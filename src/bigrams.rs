pub struct Bigrams {
    grams: Vec<u64>,
}

impl Bigrams {
    pub fn new() -> Self {
        Bigrams { grams: Vec::new() }
    }

    pub fn build(text: &str) -> Self {
        let mut set = Bigrams::new();
        set.insert(text);
        set
    }

    pub fn clear(&mut self) {
        self.grams.clear();
    }

    pub fn insert(&mut self, text: &str) {
        bigrams(text, &mut self.grams);
        self.grams.sort_unstable();
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

        let card1 = self.len();
        let card2 = other.len();

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

#[cfg(test)]
mod tests {
    use super::Bigrams;

    #[test]
    fn it_awards_no_score_for_short_text() {
        let a = Bigrams::build("h");
        let b = Bigrams::build("hello");
        assert_eq!(0.0, a.similarity(&b));
        assert_eq!(0.0, b.similarity(&a));
    }

    #[test]
    fn it_awards_full_score_for_identical_text() {
        let a = Bigrams::build("hello");
        let b = Bigrams::build("hello");
        assert_eq!(1.0, a.similarity(&b));
    }

    #[test]
    fn it_awards_more_points_for_closer_matches() {
        let a = Bigrams::build("he");
        let b = Bigrams::build("hello");
        let c = Bigrams::build("helo");
        assert!(a.similarity(&b) > 0.0);
        assert!(c.similarity(&b) > 0.0);
        assert!(c.similarity(&b) > a.similarity(&b));
    }
}
