pub struct Scorer<'a> {
    term: &'a str,
}

impl<'a> Scorer<'a> {
    pub fn new(term: &'a str) -> Self {
        Scorer { term }
    }

    pub fn score(&self, line: &str) -> u32 {
        let term = self.term;
        if term.len() > line.len() {
            return 0;
        }

        let similarity = dice(line, term);
        let total = if similarity == 0.0 {
            0.0
        } else if similarity == 1.0 {
            1.0 * 2.0
        } else {
            similarity + bonus(prefix(line, term, 5)) + bonus(suffix(line, term, 5))
        };

        (total * 10000.0) as u32
    }
}

// Returns a string similarity score between 0.0 and 1.0.
//
// https://en.wikipedia.org/wiki/Sørensen–Dice_coefficient
fn dice(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    if a.len() < 2 || b.len() < 2 {
        return 0.0;
    }

    let mut bigrams1 = bigrams(a);
    let mut bigrams2 = bigrams(b);

    let card1 = bigrams1.len();
    let card2 = bigrams2.len();

    bigrams1.sort();
    bigrams2.sort();

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

fn bigrams(text: &str) -> Vec<u64> {
    text.chars()
        .collect::<Vec<_>>()
        .windows(2)
        .map(|bi| ((bi[0] as u64) << 32) | bi[1] as u64)
        .collect()
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
    use super::dice;

    #[test]
    fn it_awards_no_score_for_short_text() {
        assert_eq!(0.0, dice("h", "hello"));
        assert_eq!(0.0, dice("hello", "h"));
    }

    #[test]
    fn it_awards_full_score_for_identical_text() {
        assert_eq!(1.0, dice("hello", "hello"));
    }

    #[test]
    fn it_awards_more_points_for_closer_matches() {
        let a = dice("he", "hello");
        let b = dice("helo", "hello");
        assert!(a > 0.0);
        assert!(b > 0.0);
        assert!(b > a);
    }
}
