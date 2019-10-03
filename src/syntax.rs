use std::fmt;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone)]
pub enum Expr {
    Num(i64),
    Var(Name),
    Op(Opcode, Box<Expr>, Box<Expr>),
    App(Name, Vec<Expr>),
    Let(Name, Box<Expr>, Box<Expr>),
    Lam(Vec<Name>, Box<Expr>),
}

#[derive(Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(dead_code)]
mod debruijn {
    use std::collections::HashMap;

    pub struct Indexer {
        indices: HashMap<String, usize>,
        next_index: usize,
    }

    impl Indexer {
        fn new() -> Self {
            Self {
                indices: HashMap::new(),
                next_index: 1,
            }
        }

        fn intro<T>(&mut self, x: &str, f: impl FnOnce(&mut Self) -> T) -> T {
            let old_index = self.indices.insert(x.to_string(), self.next_index);
            self.next_index += 1;
            let res = f(self);
            self.next_index -= 1;
            if let Some(old_index) = old_index {
                self.indices.insert(x.to_string(), old_index);
            } else {
                self.indices.remove(x);
            }
            res
        }

        fn get(&self, x: &str) -> Option<usize> {
            self.indices.get(x).map(|i| self.next_index - i)
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn test_indexer() {
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
    }
}
