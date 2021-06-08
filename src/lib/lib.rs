#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

#[macro_use]
pub mod utils;

mod eval;
mod rpn;
mod tokenize;

pub mod model;

use eval::eval;
use model::EvaluationContext;
use rpn::rpn;
pub use tokenize::tokenize;

use self::model::{
    errors::{ContextualError, Error},
    tokens::Token,
};

pub const RECURSION_LIMIT: u8 = std::u8::MAX as _;

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
pub fn doeval<'a>(
    string: &str,
    context: EvaluationContext<'a>,
) -> Result<(f64, Vec<Token<'a>>), ContextualError> {
    if context.depth == RECURSION_LIMIT {
        return Err(Error::RecursionLimit.with_context(context.context));
    }

    let tokens = tokenize(string, &context).map_err(|e| e.with_context(context.context.clone()))?;
    let rpn = rpn(&tokens); //.map_err(|e| e.with_context(context.context))?;

    let rpn = match rpn {
        Ok(e) => e,
        Err(e) => return Err(e.with_context(context.context)),
    };
    let result = eval(&rpn, context)?;
    Ok((result, tokens))
}

// #[cfg(test)]
// mod tests {

//     #![allow(clippy::shadow_unrelated)]

//     use super::doeval;

//     use crate::{
//         model::{
//             constants::Constant, constants::ConstantType, operators::OperatorType,
//             tokens::ParenType, variables::Variable, EvaluationContext,
//         },
//         Error, Token,
//     };

//     #[test]
//     #[allow(clippy::too_many_lines)]
//     fn test_doeval_ok() {
//         let vars = [Variable {
//             repr: "x".to_string(),
//             value: 5.5,
//         }];
//         let context = EvaluationContext {
//             vars: &vars,
//             funcs: &[],
//             depth: 0,
//         };

//         // Relatively simple case with a variable
//         let (result, tokens) = doeval("1.1 + 2.2 + $x", context).unwrap();
//         assert_same!(result, 8.8);
//         assert_eq!(
//             tokens,
//             [
//                 Token::Number { value: 1.1 },
//                 Token::operator(OperatorType::Add),
//                 Token::Number { value: 2.2 },
//                 Token::operator(OperatorType::Add),
//                 Token::Variable {
//                     inner: &context.vars[0]
//                 }
//             ]
//         );

//         // Functions w/o parens
//         let (result, tokens) = doeval("sin pi", context).unwrap();
//         assert_same!(result, std::f64::consts::PI.sin());
//         assert_eq!(
//             tokens,
//             [
//                 Token::operator(OperatorType::Sin,),
//                 Token::Constant {
//                     inner: Constant::by_type(ConstantType::PI),
//                 },
//             ]
//         );

//         let (result, tokens) = doeval("1 plus 7 sub 2 times 3", context).unwrap();
//         assert_same!(result, 2.0);
//         assert_eq!(
//             tokens,
//             [
//                 Token::Number { value: 1.0 },
//                 Token::operator(OperatorType::Add,),
//                 Token::Number { value: 7.0 },
//                 Token::operator(OperatorType::Sub,),
//                 Token::Number { value: 2.0 },
//                 Token::operator(OperatorType::Mul,),
//                 Token::Number { value: 3.0 },
//             ]
//         );

//         let (result, tokens) = doeval("sin(1 + 2 + 3)", context).unwrap();
//         assert_same!(result, ((1.0 + 2.0 + 3.0) as f64).sin());
//         assert_eq!(
//             tokens,
//             [
//                 Token::operator(OperatorType::Sin,),
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::operator(OperatorType::Add,),
//                 Token::Number { value: 2.0 },
//                 Token::operator(OperatorType::Add,),
//                 Token::Number { value: 3.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ]
//         );

//         let (result, tokens) = doeval("(1)", context).unwrap();
//         assert_same!(result, 1.0);
//         assert_eq!(
//             tokens,
//             [
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ]
//         );

//         let (result, tokens) = doeval("((1))", context).unwrap();
//         assert_same!(result, 1.0);
//         assert_eq!(
//             tokens,
//             [
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ]
//         );

//         let (result, tokens) = doeval("-1", context).unwrap();
//         assert_same!(result, -1.0);
//         assert_eq!(
//             tokens,
//             [
//                 Token::operator(OperatorType::Negative,),
//                 Token::Number { value: 1.0 },
//             ]
//         );

//         let (result, tokens) = doeval("1 + -1", context).unwrap();
//         assert_same!(result, 0.0);
//         assert_eq!(
//             tokens,
//             [
//                 Token::Number { value: 1.0 },
//                 Token::operator(OperatorType::Add,),
//                 Token::operator(OperatorType::Negative,),
//                 Token::Number { value: 1.0 },
//             ]
//         );

//         let (result, tokens) = doeval("-   (  1.1 +  2.2)", context).unwrap();
//         assert_same!(result, -3.3);
//         assert_eq!(
//             tokens,
//             [
//                 Token::operator(OperatorType::Negative,),
//                 Token::Paren {
//                     kind: ParenType::Left,
//                 },
//                 Token::Number { value: 1.1 },
//                 Token::operator(OperatorType::Add,),
//                 Token::Number { value: 2.2 },
//                 Token::Paren {
//                     kind: ParenType::Right,
//                 },
//             ]
//         );

//         // let (result _) =
//     }

//     // #[test]
//     // fn test_doeval_errors() {
//     //     [
//     //         // ("1 +", Error::Operand(OperatorType::Add)),
//     //         ("1 + 2 + 3 + h", Error::Parsing(12)),
//     //         ("h", Error::Parsing(0)),
//     //         ("(1", Error::MismatchingParens),
//     //         ("3 + $a", Error::UnknownVariable(4)),
//     //     ]
//     //     .iter()
//     //     .for_each(|(a, b)| assert_eq!(doeval(a, EvaluationContext::default()).unwrap_err(), *b));
//     // }

//     // #[test]
//     // fn test_vars() {
//     //     let test_vars = vec![
//     //         Variable {
//     //             repr: String::from('v'),
//     //             value: 5.0,
//     //         },
//     //         Variable {
//     //             repr: String::from("pi"),
//     //             value: 7.0,
//     //         },
//     //     ];

//     //     [
//     //         (
//     //             "$v",
//     //             5.0,
//     //             vec![Token::Variable {
//     //                 inner: &test_vars[0],
//     //             }],
//     //         ),
//     //         (
//     //             "$v + 5",
//     //             10.0,
//     //             vec![
//     //                 Token::Variable {
//     //                     inner: &test_vars[0],
//     //                 },
//     //                 Token::operator(OperatorType::Add),
//     //                 Token::Number { value: 5.0 },
//     //             ],
//     //         ),
//     //         (
//     //             "  5 +    $v    ",
//     //             10.0,
//     //             vec![
//     //                 Token::Number { value: 5.0 },
//     //                 Token::operator(OperatorType::Add),
//     //                 Token::Variable {
//     //                     inner: &test_vars[0],
//     //                 },
//     //             ],
//     //         ),
//     //         (
//     //             "pi + $pi",
//     //             std::f64::consts::PI + 7.0,
//     //             vec![
//     //                 Token::Constant {
//     //                     inner: Constant::by_type(ConstantType::PI),
//     //                 },
//     //                 Token::operator(OperatorType::Add),
//     //                 Token::Variable {
//     //                     inner: &test_vars[1],
//     //                 },
//     //             ],
//     //         ),
//     //     ]
//     //     .iter()
//     //     .for_each(|(a, b, c)| {
//     //         let context = EvaluationContext {
//     //             vars: &test_vars,
//     //             funcs: &[],
//     //             depth: 0,
//     //         };
//     //         let (result, tokens) = match doeval(a, context) {
//     //             Ok((x, y)) => (x, y),
//     //             Err(e) => panic!("error! {:?}; {}", e, a),
//     //         };
//     //         assert_eq!(&tokens, c, "Checking tokenization of [{}]", a);
//     //         assert_same!(result, *b, "Checking evaluation of [{}]", a);
//     //     });
//     // }

//     // #[test]
//     // fn fail_vars() {
//     //     vec![("3 + $a", Error::UnknownVariable(4))]
//     //         .iter()
//     //         .for_each(|(a, b)| {
//     //             assert_eq!(doeval(a, EvaluationContext::default()).unwrap_err(), *b)
//     //         });
//     // }
// }
