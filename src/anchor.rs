use bigrams::Bigrams;

pub struct Anchor {
    anchor: Bigrams,
    bigrams: Bigrams,
}

impl Anchor {
    pub fn new(text: &str) -> Self {
        Anchor {
            anchor: Bigrams::build(text),
            bigrams: Bigrams::new(),
        }
    }

    pub fn score(&mut self, text: &str) -> f32 {
        self.bigrams.clear();
        self.bigrams.insert(text);
        self.bigrams.similarity(&self.anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::Anchor;

    #[test]
    fn it_awards_more_points_for_similar_paths() {
        let mut anchor = Anchor::new("app/assets/modules/test.js");
        let a = anchor.score("app/assets/modules/hello.js");
        let b = anchor.score("test/assets/modules/hello.js");
        assert!(a > 0.0);
        assert!(b > 0.0);
        assert!(a > b);
    }
}
