use crate::{DoEvalResult, model::EvaluationContext};

use super::model::{
    errors::{ContextualError, Error, InnerFunction},
    functions::Functions,
    tokens::Token,
};

/// Evaluate a list of tokens
/// * `tokens` - The tokens
///
/// Returns the result as a 64-bit float or an `Error`
pub fn eval<'v1, 'v2: 'v1, 'f1, 'f2: 'f1>(tokens: Vec<Token<'v1, 'f1>>, eval_context: EvaluationContext<'v1, 'f1>) -> Result<f64, DoEvalResult<'v2, 'f2>> 
{
    // We need a mutable copy of the tokens
    // let mut stack: Vec<Token> = tokens.into_iter().rev().collect();
    tokens.reverse();
    let mut stack: Vec<Token<'v1, 'f1>> = tokens;
    let mut args: Vec<f64> = Vec::new();

    // drop(tokens);

    while let Some(token) = stack.pop() {
        let token: Token = token;
        match token {
            Token::Number { value } => {
                args.push(value);
            }
            Token::Constant { inner } => {
                args.push(inner.value);
            }
            Token::Variable { inner } => args.push(inner.value),
            Token::Operator { inner: op } => {
                let start = if let Some(x) = args.len().checked_sub(op.arity()) {
                    x
                } else {
                    let inner: InnerFunction = match op {
                        Functions::Builtin(b) => InnerFunction::Builtin(b.kind),
                        Functions::User(func) => InnerFunction::User(func),
                    };
                    return Err(DoEvalResult::EvalError {
                        context: eval_context.context,
                        error: Error::Operand(inner)
                    });
                };

                // Takes the last `op.arity` number of values from `args`
                // `start = args.len() - op.arity`
                let args_: Vec<f64> = args.drain(start..).collect();

                let result = match op {
                    Functions::Builtin(b) => (b.doit)(&args_),
                    Functions::User(f) => match f.apply(&args_, &eval_context) {
                        DoEvalResult::Ok {
                            result,
                            ..
                        } => result,
                        other => return Err(other)
                    },
                };

                // Push the result of the evaluation
                stack.push(Token::Number { value: result });
            }
            Token::Paren { .. } | Token::Comma => {}
        }
    }

    // Result
    if args.len() == 1 {
        Ok(args[0])
    } else {
        Err(DoEvalResult::EvalError {
            context: eval_context.context,
            error: Error::EmptyStack,
        })
    }
}

// #[cfg(test)]
// mod tests {

//     #![allow(clippy::shadow_unrelated)]

//     use std::vec;

//     use crate::{
//         model::{
//             errors::ErrorContext,
//             functions::{Function, Functions},
//             operators::OperatorType,
//             tokens::ParenType,
//             variables::Variable,
//         },
//         rpn::rpn,
//     };

//     use super::{eval, EvaluationContext, Token};

//     #[test]
//     fn test_eval_ok() {
//         let tokens = [Token::Number { value: 4.67 }];

//         let result = eval(&tokens, EvaluationContext::default()).unwrap();
//         assert_same!(result, 4.67);

//         // sin(5)^2 + cos(5)^2 => 1
//         let tokens = [
//             Token::operator(OperatorType::Sin),
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 5.0 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//             Token::operator(OperatorType::Pow),
//             Token::Number { value: 2.0 },
//             Token::operator(OperatorType::Add),
//             Token::operator(OperatorType::Cos),
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 5.0 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//             Token::operator(OperatorType::Pow),
//             Token::Number { value: 2.0 },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         let result = eval(&tokens, EvaluationContext::default()).unwrap();
//         assert_same!(result, 1.0);
//     }

//     #[test]
//     fn test_eval_functions() {
//         let funcs = [Function {
//             name: "inv".to_string(),
//             args: vec!["x".to_string()],
//             code: "1/$x".to_string(),
//         }];
//         let vars = [Variable {
//             repr: "e".to_string(),
//             value: 5.0,
//         }];
//         let context = EvaluationContext {
//             vars: &vars,
//             funcs: &funcs,
//             context: ErrorContext::Main,
//             depth: 0,
//         };

//         let tokens = [
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Number { value: 1.0 },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         let result = eval(&tokens, context.clone()).unwrap();
//         assert_same!(result, 1.0);

//         let tokens = [
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Variable { inner: &vars[0] },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         let result = eval(&tokens, context.clone()).unwrap();
//         assert_same!(result, 1.0 / vars[0].value);

//         let tokens = [
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Number { value: 8.0 },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         let result = eval(&tokens, context.clone()).unwrap();
//         assert_same!(result, 1.0 / 8.0);

//         let funcs = [Function {
//             name: "ident".to_string(),
//             args: vec!["a".to_string()],
//             code: "$a".to_string(),
//         }];
//         let context = EvaluationContext {
//             vars: &[],
//             funcs: &funcs,
//             context: ErrorContext::Main,
//             depth: 0,
//         };

//         let tokens = [
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Number { value: 1.0 },
//             Token::operator(OperatorType::Add),
//             Token::Operator {
//                 inner: Functions::User(&funcs[0]),
//             },
//             Token::Number { value: -1.0 },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         let result = eval(&tokens, context).unwrap();
//         assert_same!(result, 0.0);
//     }
// }
