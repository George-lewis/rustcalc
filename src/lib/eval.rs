use crate::model::EvaluationContext;

use super::model::{errors::{Error, ContextError, ErrorContext}, tokens::Token, functions::Functions};

/// Evaluate a list of tokens
/// * `tokens` - The tokens
///
/// Returns the result as a 64-bit float or an `Error`
pub fn eval<'var: 'context, 'func: 'context, 'context>(tokens: &[Token], context: EvaluationContext<'var, 'func>) -> Result<f64, ContextError<'func>> {
    // We need a mutable copy of the tokens
    let mut stack: Vec<Token> = tokens.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new();

    while let Some(token) = stack.pop() {
        match token {
            Token::Number { value } => {
                args.push(value);
            }
            Token::Constant { inner } => {
                args.push(inner.value);
            }
            Token::Variable { inner } => args.push(inner.value),
            Token::Operator { inner: op } => {
                let start = match args.len().checked_sub(op.arity()) {
                    Some(x) => x,
                    None => panic!(), // None => return Err(Error::Operand(op.kind)),
                };

                // Takes the last `op.arity` number of values from `args`
                // `start = args.len() - op.arity`
                let args_: Vec<f64> = args.drain(start..).collect();

                let result = match op {
                    Functions::Builtin(b) => {
                        // op.apply(&args_, context)?
                        (b.doit)(&args_)
                    },
                    Functions::User(func) => {
                        let find = context.funcs.iter().find(|f| f.name == func.name).unwrap();
                        find.apply(&args_, context)?
                    }
                };
                
                // {
                //     Ok(result) => {
                //         // Push the result of the evaluation
                //         stack.push(Token::Number { value: result });
                //     }
                //     Err(error) => {
                //         if let Functions::User(func) = op {
                //             let find = context.funcs.iter().find(|f| f.name == func.name).unwrap();
                //             return Err(
                //                 ContextError {
                //                     context: ErrorContext::Scoped(find),
                //                     error: error.error
                //                 }
                //             )
                //         }
                //         return Err(error)
                //     }
                // }
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
    Err(Error::EmptyStack.with_context(context.context))
}

// #[cfg(test)]
// mod tests {

//     use super::{eval, EvaluationContext, Token};

//     #[test]
//     fn test_eval_ok() {
//         let tokens = [Token::Number { value: 4.67 }];
//         let result = eval(&tokens, EvaluationContext::default()).unwrap();
//         assert_same!(result, 4.67);
//     }
// }
