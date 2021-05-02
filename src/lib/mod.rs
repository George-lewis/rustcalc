mod eval;
mod rpn;
mod tokenize;

pub mod utils;

pub mod model;

use eval::eval;
use rpn::rpn;
use tokenize::tokenize;

use self::model::{errors::Error, tokens::Token, variables::Variable};

pub fn doeval<'a>(string: &str, vars: &'a [Variable]) -> Result<(f64, Vec<Token<'a>>), Error> {
    let tokens = tokenize(string, vars)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}
