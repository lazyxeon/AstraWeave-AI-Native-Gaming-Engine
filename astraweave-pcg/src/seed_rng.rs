//! Deterministic RNG with explicit seeds per layer

use rand::distr::uniform::{SampleRange, SampleUniform};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Deterministic random number generator with layer tracking
pub struct SeedRng {
    inner: StdRng,
    layer: String,
}

impl SeedRng {
    /// Create a new seeded RNG for a specific layer
    pub fn new(seed: u64, layer: &str) -> Self {
        Self {
            inner: StdRng::seed_from_u64(seed),
            layer: layer.to_string(),
        }
    }

    /// Fork this RNG into a child with a new sublayer
    /// The child has a deterministic seed derived from this RNG
    pub fn fork(&mut self, sublayer: &str) -> Self {
        let subseed = self.inner.random::<u64>();
        Self::new(subseed, &format!("{}::{}", self.layer, sublayer))
    }

    /// Get the layer name (useful for debugging)
    pub fn layer(&self) -> &str {
        &self.layer
    }

    /// Generate a random value in the given range
    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.inner.random_range(range)
    }

    /// Generate a random boolean
    pub fn gen_bool(&mut self) -> bool {
        self.inner.random()
    }

    /// Generate a random boolean with given probability
    pub fn gen_bool_with_prob(&mut self, probability: f64) -> bool {
        self.inner.random_bool(probability)
    }

    /// Choose a random element from a slice
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            Some(&slice[self.gen_range(0..slice.len())])
        }
    }

    /// Choose a random element from a slice and return a mutable reference
    pub fn choose_mut<'a, T>(&mut self, slice: &'a mut [T]) -> Option<&'a mut T> {
        if slice.is_empty() {
            None
        } else {
            let idx = self.gen_range(0..slice.len());
            Some(&mut slice[idx])
        }
    }

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.inner);
    }

    /// Generate a random f32 in [0, 1)
    pub fn gen_f32(&mut self) -> f32 {
        self.inner.random()
    }

    /// Generate a random f64 in [0, 1)
    pub fn gen_f64(&mut self) -> f64 {
        self.inner.random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_seed_same_sequence() {
        let mut rng1 = SeedRng::new(42, "test");
        let mut rng2 = SeedRng::new(42, "test");

        for _ in 0..100 {
            assert_eq!(rng1.gen_range(0..100), rng2.gen_range(0..100));
        }
    }

    #[test]
    fn test_different_seed_different_sequence() {
        let mut rng1 = SeedRng::new(42, "test");
        let mut rng2 = SeedRng::new(43, "test");

        let vals1: Vec<_> = (0..100).map(|_| rng1.gen_range(0..100)).collect();
        let vals2: Vec<_> = (0..100).map(|_| rng2.gen_range(0..100)).collect();

        // Extremely unlikely to be all equal
        assert_ne!(vals1, vals2);
    }

    #[test]
    fn test_fork_deterministic() {
        let mut rng1 = SeedRng::new(42, "parent");
        let mut rng2 = SeedRng::new(42, "parent");

        let mut child1 = rng1.fork("child");
        let mut child2 = rng2.fork("child");

        for _ in 0..100 {
            assert_eq!(child1.gen_range(0..100), child2.gen_range(0..100));
        }
    }

    #[test]
    fn test_fork_independent() {
        let mut parent = SeedRng::new(42, "parent");
        let mut child = parent.fork("child");

        // Parent and child should have different sequences
        let parent_vals: Vec<_> = (0..100).map(|_| parent.gen_range(0..100)).collect();
        let child_vals: Vec<_> = (0..100).map(|_| child.gen_range(0..100)).collect();

        assert_ne!(parent_vals, child_vals);
    }

    #[test]
    fn test_choose() {
        let mut rng = SeedRng::new(42, "test");
        let items = vec![1, 2, 3, 4, 5];

        let chosen = rng.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));
    }

    #[test]
    fn test_choose_empty() {
        let mut rng = SeedRng::new(42, "test");
        let items: Vec<i32> = vec![];

        let chosen = rng.choose(&items);
        assert!(chosen.is_none());
    }

    #[test]
    fn test_shuffle_deterministic() {
        let mut rng1 = SeedRng::new(42, "test");
        let mut rng2 = SeedRng::new(42, "test");

        let mut items1 = vec![1, 2, 3, 4, 5];
        let mut items2 = vec![1, 2, 3, 4, 5];

        rng1.shuffle(&mut items1);
        rng2.shuffle(&mut items2);

        assert_eq!(items1, items2);
    }

    #[test]
    fn test_layer_tracking() {
        let parent = SeedRng::new(42, "parent");
        assert_eq!(parent.layer(), "parent");

        let mut parent_mut = parent;
        let child = parent_mut.fork("child");
        assert_eq!(child.layer(), "parent::child");
    }
}
