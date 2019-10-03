use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone, Debug)]
pub enum Expr {
    Num(i64),
    Var(Name, Option<usize>),
    Op(Opcode, Box<Expr>, Box<Expr>),
    App(Box<Expr>, Vec<Expr>),
    Let(Name, Box<Expr>, Box<Expr>),
    Lam(Vec<Name>, Box<Expr>),
}

#[derive(Clone, Debug)]
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

impl Expr {
    pub fn index(self) -> Self {
        use debruijn::Indexer;

        self.index_aux(&mut Indexer::new())
    }

    fn index_aux(self, indexer: &mut debruijn::Indexer) -> Self {
        use std::borrow::Borrow;
        match self {
            Self::Num(n) => Self::Num(n),
            Self::Var(x, None) => {
                let i = indexer.get(&x.0);
                Self::Var(x, i)
            }
            Self::Var(_, Some(_)) => panic!("indexer running on indexed expression"),
            Self::Op(op, e1, e2) => {
                let e1 = Box::new(e1.index_aux(indexer));
                let e2 = Box::new(e2.index_aux(indexer));
                Self::Op(op, e1, e2)
            }
            Self::App(f, es) => {
                let f = Box::new(f.index_aux(indexer));
                let es = es.into_iter().map(|e| e.index_aux(indexer)).collect();
                Self::App(f, es)
            }
            Self::Let(x, e1, e2) => {
                let e1 = Box::new(e1.index_aux(indexer));
                let e2 = indexer.intro(&x.0, |indexer| Box::new(e2.index_aux(indexer)));
                Self::Let(x, e1, e2)
            }
            Self::Lam(xs, e) => {
                // TODO(MH): Make this more efficient by using iterators.
                let e = indexer.intro_many(
                    &xs.iter().map(|x| x.0.borrow()).collect::<Vec<&str>>(),
                    |indexer| Box::new(e.index_aux(indexer)),
                );
                Self::Lam(xs, e)
            }
        }
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
}
