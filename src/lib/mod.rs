pub mod constants;
pub mod errors;
pub mod operators;
pub mod tokens;
pub mod utils;
mod tokenize;
mod rpn;
mod eval;

use tokens::Token;
use errors::Error;
use tokenize::tokenize;
use rpn::rpn;
use eval::eval;

pub fn doeval(string: &str) -> Result<(f64, Vec<Token>), Error> {
    let tokens = tokenize(string)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}