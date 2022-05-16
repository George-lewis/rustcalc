#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

#[macro_use]
pub mod utils;

mod eval;
mod rpn;
mod tokenize;
mod transform;
mod verify;

pub mod model;

use eval::eval;
use itertools::Itertools;
use model::{
    errors::{ErrorContext, EvalError, RpnError},
    tokens::{PartialToken, Tokens},
    EvaluationContext,
};
use rpn::rpn;
pub use tokenize::tokenize;

pub const RECURSION_LIMIT: u8 = 25;

#[derive(Debug, Clone)]
pub enum DoEvalResult<'str, 'funcs> {
    RecursionLimit {
        context: ErrorContext<'funcs>,
    },
    ParsingError {
        context: ErrorContext<'funcs>,
        partial_tokens: Vec<PartialToken<'str, 'funcs>>,
    },
    RpnError {
        context: ErrorContext<'funcs>,
        error: RpnError,
    },
    EvalError {
        context: ErrorContext<'funcs>,
        tokens: Vec<Tokens<'str, 'funcs>>,
        error: EvalError<'str, 'funcs>,
    },
    Ok {
        tokens: Vec<Tokens<'str, 'funcs>>,
        result: f64,
    },
}

/// Evaluate a string containing a mathematical expression
///
/// * `string` - The string
/// * `vars` - The available `Variable`s.
/// These must be sorted such that no variable's representation is a subset of one that comes after it
///
/// ## Returns
/// The result of the computation as an a 64-bit float plus the result of the tokenization
///
/// ## Errors
/// Returns an error if the expression couldn't be computed
pub fn doeval<'funcs, 'var>(
    string: &'funcs str,
    context: EvaluationContext<'var, 'funcs>,
) -> DoEvalResult<'funcs, 'funcs> {
    // Recursion limit check
    if context.depth == RECURSION_LIMIT {
        return DoEvalResult::RecursionLimit {
            context: context.context,
        };
    }

    // Tokenize input
    let string_tokens = match tokenize(string, &context) {
        Ok(string_tokens) => string_tokens,
        Err(partial_tokens) => {
            return DoEvalResult::ParsingError {
                context: context.context,
                partial_tokens,
            };
        }
    };

    // We're about to apply transformations, so we need to
    // Change to [Tokens]
    let mut tokens = string_tokens.into_iter().map(Tokens::String).collect_vec();

    // Apply transformations
    transform::implicit_parens(&mut tokens);
    transform::implicit_coeffs(&mut tokens);

    if let Some((tok, func)) = verify::verify_fn_args(&tokens) {
        let st = match tok {
            Tokens::String(st) => st,
            Tokens::Synthetic(_) => unreachable!(),
        };
        return DoEvalResult::EvalError {
            context: context.context,
            tokens,
            error: EvalError::Operand {
                op: func,
                tok: st,
            },
        }
    }

    // Convert in to reverse polish notation to prepare for eval
    let rpn = match rpn(&tokens) {
        Ok(rpn) => rpn,
        Err(error) => {
            return DoEvalResult::RpnError {
                context: context.context,
                error,
            }
        }
    };

    // Perform evaluation
    eval(rpn, tokens, context)
}

// #[cfg(test)]
// mod tests {

//     #![allow(clippy::shadow_unrelated, clippy::needless_for_each)]

//     use super::doeval;

//     use crate::{
//         model::{
//             constants::Constant, constants::ConstantType, errors::ErrorContext,
//             operators::OperatorType, tokens::ParenType, variables::Variable, EvaluationContext,
//         },
//         Token,
//     };

//     macro_rules! context {
//         ($vars:ident) => {
//             EvaluationContext {
//                 vars: &$vars,
//                 funcs: &[],
//                 depth: 0,
//                 context: ErrorContext::Main,
//             };
//         };
//     }

// #[test]
// #[allow(clippy::too_many_lines)]
// fn test_doeval_ok() {
//     let vars = [Variable {
//         repr: "x".to_string(),
//         value: 5.5,
//     }];

//     // Relatively simple case with a variable
//     let (result, tokens) = doeval("1.1 + 2.2 + $x", context!(vars)).unwrap();
//     assert_same!(result, 8.8);
//     assert_eq!(
//         tokens,
//         [
//             Token::Number { value: 1.1 },
//             Token::operator(OperatorType::Add),
//             Token::Number { value: 2.2 },
//             Token::operator(OperatorType::Add),
//             Token::Variable { inner: &vars[0] }
//         ]
//     );

//     // Functions w/o parens
//     let (result, tokens) = doeval("sin pi", context!(vars)).unwrap();
//     assert_same!(result, std::f64::consts::PI.sin());
//     assert_eq!(
//         tokens,
//         [
//             Token::operator(OperatorType::Sin,),
//             Token::Paren {
//                 kind: ParenType::Left
//             },
//             Token::Constant {
//                 inner: Constant::by_type(ConstantType::PI),
//             },
//             Token::Paren {
//                 kind: ParenType::Right
//             },
//         ]
//     );

//     let (result, tokens) = doeval("1 plus 7 sub 2 times 3", context!(vars)).unwrap();
//     assert_same!(result, 2.0);
//     assert_eq!(
//         tokens,
//         [
//             Token::Number { value: 1.0 },
//             Token::operator(OperatorType::Add,),
//             Token::Number { value: 7.0 },
//             Token::operator(OperatorType::Sub,),
//             Token::Number { value: 2.0 },
//             Token::operator(OperatorType::Mul,),
//             Token::Number { value: 3.0 },
//         ]
//     );

//     let (result, tokens) = doeval("sin(1 + 2 + 3)", context!(vars)).unwrap();
//     assert_same!(result, ((1.0 + 2.0 + 3.0) as f64).sin());
//     assert_eq!(
//         tokens,
//         [
//             Token::operator(OperatorType::Sin,),
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 1.0 },
//             Token::operator(OperatorType::Add,),
//             Token::Number { value: 2.0 },
//             Token::operator(OperatorType::Add,),
//             Token::Number { value: 3.0 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//         ]
//     );

//     let (result, tokens) = doeval("(1)", context!(vars)).unwrap();
//     assert_same!(result, 1.0);
//     assert_eq!(
//         tokens,
//         [
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 1.0 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//         ]
//     );

//     let (result, tokens) = doeval("((1))", context!(vars)).unwrap();
//     assert_same!(result, 1.0);
//     assert_eq!(
//         tokens,
//         [
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 1.0 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//         ]
//     );

//     let (result, tokens) = doeval("-1", context!(vars)).unwrap();
//     assert_same!(result, -1.0);
//     assert_eq!(
//         tokens,
//         [
//             Token::operator(OperatorType::Negative,),
//             Token::Number { value: 1.0 },
//         ]
//     );

//     let (result, tokens) = doeval("1 + -1", context!(vars)).unwrap();
//     assert_same!(result, 0.0);
//     assert_eq!(
//         tokens,
//         [
//             Token::Number { value: 1.0 },
//             Token::operator(OperatorType::Add,),
//             Token::operator(OperatorType::Negative,),
//             Token::Number { value: 1.0 },
//         ]
//     );

//     let (result, tokens) = doeval("-   (  1.1 +  2.2)", context!(vars)).unwrap();
//     assert_same!(result, -3.3);
//     assert_eq!(
//         tokens,
//         [
//             Token::operator(OperatorType::Negative,),
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Number { value: 1.1 },
//             Token::operator(OperatorType::Add,),
//             Token::Number { value: 2.2 },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//         ]
//     );
// }

// #[test]
// fn test_doeval_errors() {
//     [
//         ("1 + 2 + 3 + h", Error::Parsing(12)),
//         ("h", Error::Parsing(0)),
//         ("(1", Error::MismatchingParens),
//         ("3 + $a", Error::UnknownVariable(4)),
//     ]
//     .iter()
//     .for_each(|(a, b)| {
//         assert_eq!(
//             doeval(a, EvaluationContext::default()).unwrap_err().error,
//             *b
//         );
//     });
// }

// #[test]
// fn test_vars() {
//     let test_vars = vec![
//         Variable {
//             repr: String::from('v'),
//             value: 5.0,
//         },
//         Variable {
//             repr: String::from("pi"),
//             value: 7.0,
//         },
//     ];

//     [
//         (
//             "$v",
//             5.0,
//             vec![Token::Variable {
//                 inner: &test_vars[0],
//             }],
//         ),
//         (
//             "$v + 5",
//             10.0,
//             vec![
//                 Token::Variable {
//                     inner: &test_vars[0],
//                 },
//                 Token::operator(OperatorType::Add),
//                 Token::Number { value: 5.0 },
//             ],
//         ),
//         (
//             "  5 +    $v    ",
//             10.0,
//             vec![
//                 Token::Number { value: 5.0 },
//                 Token::operator(OperatorType::Add),
//                 Token::Variable {
//                     inner: &test_vars[0],
//                 },
//             ],
//         ),
//         (
//             "pi + $pi",
//             std::f64::consts::PI + 7.0,
//             vec![
//                 Token::Constant {
//                     inner: Constant::by_type(ConstantType::PI),
//                 },
//                 Token::operator(OperatorType::Add),
//                 Token::Variable {
//                     inner: &test_vars[1],
//                 },
//             ],
//         ),
//     ]
//     .iter()
//     .for_each(|(a, b, c)| {
//         let context = context!(test_vars);
//         let (result, tokens) = match doeval(a, context) {
//             Ok((x, y)) => (x, y),
//             Err(e) => panic!("error! {:?}; {}", e, a),
//         };
//         // assert_eq!(&tokens, c, "Checking tokenization of [{}]", a);
//         assert_same!(result, *b, "Checking evaluation of [{}]", a);
//     });
// }

// #[test]
// fn fail_vars() {
//     vec![("3 + $a", Error::UnknownVariable(4))]
//         .iter()
//         .for_each(|(a, b)| {
//             assert_eq!(
//                 doeval(a, EvaluationContext::default()).unwrap_err().error,
//                 *b
//             );
//         });
// }
// }
