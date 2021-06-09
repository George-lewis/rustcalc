use super::model::{constants::Constant, errors::Error, operators::Operator, tokens::Token};

/// Evaluate a list of tokens
/// * `tokens` - The tokens
///
/// Returns the result as a 64-bit float or an `Error`
pub fn eval(tokens: &[Token]) -> Result<f64, Error> {
    // We need a mutable copy of the tokens
    let mut stack: Vec<Token> = tokens.iter().rev().copied().collect();
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

    #![allow(clippy::shadow_unrelated)]

    use super::{eval, Token};
    use crate::{model::{operators::OperatorType, tokens::ParenType}, rpn::rpn};

    #[test]
    fn test_eval_ok() {
        let tokens = [Token::Number { value: 4.67 }];
        let result = eval(&tokens).unwrap();
        same!(result, 4.67);

        // sin(5)^2 + cos(5)^2 => 1
        let tokens = [
            Token::Operator {
                kind: OperatorType::Sin
            },
            Token::Paren {
                kind: ParenType::Left,
            },
            Token::Number {
                value: 5.0
            },
            Token::Paren {
                kind: ParenType::Right,
            },
            Token::Operator {
                kind: OperatorType::Pow
            },
            Token::Number {
                value: 2.0
            },
            Token::Operator {
                kind: OperatorType::Add
            },
            Token::Operator {
                kind: OperatorType::Cos
            },
            Token::Paren {
                kind: ParenType::Left,
            },
            Token::Number {
                value: 5.0
            },
            Token::Paren {
                kind: ParenType::Right,
            },
            Token::Operator {
                kind: OperatorType::Pow
            },
            Token::Number {
                value: 2.0
            }
        ];
        let tokens = rpn(&tokens).unwrap();
        dbg!(&tokens);
        let result = eval(&tokens).unwrap();
        same!(result, 1.0);
    }
}
