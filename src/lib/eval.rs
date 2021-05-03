use super::model::{constants::Constant, errors::Error, operators::Operator, tokens::Token};

/// Evaluate a list of tokens
/// * `tokens` - The tokens
///
/// Returns the result as a 64-bit float or an `Error`
pub fn eval(tokens: &[Token]) -> Result<f64, Error> {
    // We need a mutable copy of the tokens
    let mut stack: Vec<Token> = tokens.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new();

    while let Some(token) = stack.pop() {
        match token {
            Token::Number { value } => {
                args.push(value);
            }
            Token::Constant { kind } => {
                let constant = Constant::by_type(kind);
                args.push(constant.value);
            }
            Token::Variable { inner } => args.push(inner.value),
            Token::Operator { kind } => {
                let op = Operator::by_type(kind);
                let start = match args.len().checked_sub(op.arity) {
                    Some(x) => x,
                    None => return Err(Error::Operand(op.kind)),
                };

                // Takes the last `op.arity` number of values from `args`
                // `start = args.len() - op.arity`
                let args_: Vec<f64> = args.drain(start..).collect();
                let result = (op.doit)(&args_);

                // Push the result of the evaluation
                stack.push(Token::Number { value: result });
            }
            Token::Paren { .. } => {}
        }
    }

    // Result
    if args.len() == 1 {
        return Ok(args[0]);
    }
    Err(Error::EmptyStack)
}


#[cfg(test)]
mod tests {

    // These tests rely on `tokenize` working correctly

    use super::{eval, Token, Error, Constant, Operator};
    use crate::model::{constants::ConstantType, operators::OperatorType};
    use crate::tokenize;
    use crate::utils::test_utils::same;

    #[test]
    fn test_eval_ok() {
        // Empty
        let tokens = tokenize("1", &[]).unwrap();
        let result = eval(&tokens).unwrap();
        assert!(same(result, 1.0));
    }

}