pub mod constants;
pub mod errors;
mod eval;
pub mod operators;
mod representable;
mod rpn;
mod tokenize;
pub mod tokens;
pub mod utils;

use errors::Error;
use eval::eval;
use rpn::rpn;
use tokenize::tokenize;
use tokens::Token;

pub fn doeval(string: &str) -> Result<(f64, Vec<Token>), Error> {
    let tokens = tokenize(string)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}
