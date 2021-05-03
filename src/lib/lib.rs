#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

mod eval;
mod rpn;
mod tokenize;

pub mod utils;

pub mod model;

use eval::eval;
use rpn::rpn;
use tokenize::tokenize;

use self::model::{errors::Error, tokens::Token, variables::Variable};

/// Evaluate a string containing a mathematical expression
///
/// * `string` - The string
/// * `vars` - The available `Variable`s
///
/// ## Returns
/// The result of the computation as an a 64-bit float plus the result of the tokenization
///
/// ## Errors
/// Returns an error if the expression couldn't be computed
pub fn doeval<'a>(string: &str, vars: &'a [Variable]) -> Result<(f64, Vec<Token<'a>>), Error> {
    let tokens = tokenize(string, vars)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}
