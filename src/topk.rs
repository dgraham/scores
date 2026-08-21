use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct TopK<T: Ord> {
    items: BinaryHeap<Reverse<T>>,
    k: usize,
}

impl<T: Ord> TopK<T> {
    pub fn new(k: usize) -> Self {
        let items = if k <= 25 {
            BinaryHeap::with_capacity(k)
        } else {
            BinaryHeap::new()
        };

        TopK { items, k }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() < self.k {
            self.items.push(Reverse(item));
        } else if self
            .items
            .peek()
            .is_some_and(|worst| item.cmp(&worst.0).is_gt())
        {
            self.items.pop();
            self.items.push(Reverse(item));
        }
    }

    pub fn into_sorted_vec(self) -> Vec<T> {
        self.items
            .into_sorted_vec()
            .into_iter()
            .map(|Reverse(item)| item)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TopK;

    #[test]
    fn it_retains_the_largest_items_in_descending_order() {
        let mut top_k = TopK::new(3);
        for item in [2, 5, 1, 4, 3] {
            top_k.push(item);
        }

        assert_eq!(top_k.into_sorted_vec(), [5, 4, 3]);
    }

    #[test]
    fn it_retains_no_items_when_its_limit_is_zero() {
        let mut top_k = TopK::new(0);
        top_k.push(1);

        assert!(top_k.into_sorted_vec().is_empty());
    }
}
