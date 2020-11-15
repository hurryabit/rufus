#[macro_use]
extern crate lalrpop_util;

pub mod check;
pub mod diagnostic;
pub mod location;
pub mod parser;
pub mod syntax;

lalrpop_mod!(
    #[allow(clippy::all)]
    pub grammar
);

#[cfg(test)]
mod tests {
    mod check;
    mod parser;
}
