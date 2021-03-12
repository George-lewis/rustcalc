use super::{tokens::Token, errors::Error, constants::Constant, operators::Operator};

pub fn eval(tokens: &[Token]) -> Result<f64, Error> {
    // We need a mutable copy of the tokens
    let mut stack: Vec<Token> = tokens.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new();

    while !stack.is_empty() {
        let token = stack.pop().unwrap();

        match token {
            Token::Number { value } => {
                args.push(value);
            }
            Token::Constant { kind } => {
                let constant = Constant::by_type(kind);
                args.push(constant.value);
            }
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
    if stack.is_empty() && args.len() == 1 {
        return Ok(args[0]);
    }
    Err(Error::EmptyStack)
}