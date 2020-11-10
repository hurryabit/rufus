#[macro_use]
extern crate lalrpop_util;

pub mod syntax;
lalrpop_mod!(
    #[allow(clippy::all)]
    pub parser
);

pub mod check;

pub mod util;

#[cfg(test)]
mod tests {
    mod check;
    mod parser;
}
