#[macro_use]
extern crate lalrpop_util;

pub mod syntax;
lalrpop_mod!(
    #[allow(clippy::all)]
    pub parser
);

#[cfg(test)]
mod tests {
    mod parser;
}
