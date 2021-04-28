pub mod constants;
pub mod errors;
mod eval;
pub mod operators;
mod representable;
mod rpn;
mod tokenize;
pub mod tokens;
pub mod utils;
pub(crate) mod variables;

use errors::Error;
use eval::eval;
use rpn::rpn;
use tokenize::tokenize;
use tokens::Token;
use variables::Variable;

pub fn doeval<'a, 'b>(
    string: &'a str,
    vars: &'b [Variable],
) -> Result<(f64, Vec<Token<'b>>), Error> {
    let tokens = tokenize(string, vars)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}
