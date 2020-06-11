use std::collections::HashMap;

pub struct Indexer {
    indices: HashMap<String, usize>,
    next_index: usize,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
            next_index: 1,
        }
    }

    pub fn intro<T>(&mut self, x: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        self.intro_many(&[x], f)
    }

    pub fn intro_many<T>(&mut self, xs: &[&str], f: impl FnOnce(&mut Self) -> T) -> T {
        let old_indices: Vec<(&str, Option<usize>)> = xs
            .iter()
            .map(|&x| {
                let old_index = self.indices.insert(x.to_string(), self.next_index);
                self.next_index += 1;
                (x, old_index)
            })
            .collect();
        let res = f(self);
        self.next_index -= xs.len();
        for (x, old_index) in old_indices.into_iter().rev() {
            if let Some(old_index) = old_index {
                self.indices.insert(x.to_string(), old_index);
            } else {
                self.indices.remove(x);
            }
        }
        res
    }

    pub fn get(&self, x: &str) -> Option<usize> {
        self.indices.get(x).map(|i| self.next_index - i)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_xy() {
        let mut idx = super::Indexer::new();

        assert_eq!(idx.get("x"), None);
        assert_eq!(idx.get("y"), None);

        idx.intro("x", |idx| {
            assert_eq!(idx.get("x"), Some(1));
            assert_eq!(idx.get("y"), None);

            idx.intro("y", |idx| {
                assert_eq!(idx.get("x"), Some(2));
                assert_eq!(idx.get("y"), Some(1));

                idx.intro("x", |idx| {
                    assert_eq!(idx.get("x"), Some(1));
                    assert_eq!(idx.get("y"), Some(2));
                });

                assert_eq!(idx.get("x"), Some(2));
                assert_eq!(idx.get("y"), Some(1));

                idx.intro("y", |idx| {
                    assert_eq!(idx.get("x"), Some(3));
                    assert_eq!(idx.get("y"), Some(1));
                });

                assert_eq!(idx.get("x"), Some(2));
                assert_eq!(idx.get("y"), Some(1));
            });

            assert_eq!(idx.get("x"), Some(1));
            assert_eq!(idx.get("y"), None);
        });

        assert_eq!(idx.get("x"), None);
        assert_eq!(idx.get("y"), None);
    }

    #[test]
    fn test_many() {
        let mut idx = super::Indexer::new();
        idx.intro_many(&["x", "y"], |idx| {
            assert_eq!(idx.get("x"), Some(2));
            assert_eq!(idx.get("y"), Some(1));
        });

        assert_eq!(idx.get("x"), None);
        assert_eq!(idx.get("y"), None);
    }

    #[test]
    fn test_many_shadowing() {
        let mut idx = super::Indexer::new();
        idx.intro_many(&["x", "x"], |idx| {
            assert_eq!(idx.get("x"), Some(1));
        });
        assert_eq!(idx.get("x"), None);
    }
}
