use crate::model::{
    functions::{Function, Functions},
    tokens::StringToken,
    EvaluationContext,
};

use std::{borrow::Cow::{Borrowed, Owned}, mem};

use super::{
    model::{
        constants::Constant,
        errors::ParserError as Error,
        operators::{Operator, OperatorType},
        tokens::ParenType,
        tokens::Token,
        variables::Variable,
    },
    utils::{self, Pos},
};

#[derive(Clone, Debug, PartialEq)]
enum TokenType {
    Number,
    Operator,
    Function,
    Paren,
    Constant,
    Variable,
    Comma,
}

fn _type(s: &str) -> Option<TokenType> {
    Some(if Token::is_next_number(s) {
        TokenType::Number
    } else if Operator::is(s) {
        TokenType::Operator
    } else if Token::is_next_paren(s) {
        TokenType::Paren
    } else if Constant::is(s) {
        TokenType::Constant
    } else if Variable::is(s) {
        TokenType::Variable
    } else if Function::is(s) {
        TokenType::Function
    } else if Token::is_next_comma(s) {
        TokenType::Comma
    } else {
        return None;
    })
}

/// Tokenize an input string
/// * `string` - A string containing a mathematical expression
/// * `vars` - The available `Variable`s
///
/// Returns a list of tokens or an error
#[allow(
    clippy::unnecessary_unwrap,
    clippy::too_many_lines,
    clippy::missing_errors_doc
)]
pub fn tokenize<'var, 'func>(
    string: &str,
    context: &EvaluationContext<'var, 'func>,
) -> Vec<StringToken<'var, 'func>> {
    let mut tokens: Vec<StringToken> = Vec::new();

    // Indicates that the current operator would be unary
    let mut unary = true;

    let mut spaces: usize = 0;
    let mut partial_token = String::new();
    let mut current_error: Option<Error> = None;

    let mut idx = 0;
    let end = string.chars().count();
    while idx < end {
        let slice = utils::slice(string, idx, &Pos::End);

        // Current character
        let c = slice.chars().next().unwrap();

        // Ignore whitespace and commas
        if c.is_whitespace() {
            idx += 1;
            spaces += 1;
            continue;
        }

        let kind: TokenType = match _type(&slice) {
            Some(kind) => kind,
            None => {
                partial_token.push(c);
                idx += 1;
                continue
            },
        };

        if !partial_token.is_empty() {
            tokens.push(StringToken {
                inner: Err(Error::UnknownToken),
                repr: partial_token.clone(),
                idx: idx - partial_token.len(),
            });
            partial_token.clear();
        }

        let result  = match kind {
            TokenType::Operator => {
                let unar = Operator::unary(&slice);

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (kind, len) = unar.unwrap();

                    // Support for consecutive unary ops
                    Ok((Token::operator(*kind), Borrowed(len), true))
                } else {
                    let (operator, len) = Operator::by_repr(&slice).unwrap();
                    let token = Token::Operator {
                        inner: Functions::Builtin(operator),
                    };

                    // The next token cannot be unary if this operator is factorial
                    // ATM this is the only postfix operator we support
                    Ok((token, Borrowed(len), operator.kind != OperatorType::Factorial))
                }
            }
            TokenType::Function => {
                match Function::next_function(&slice[1..], context.funcs) {
                    Some((func, len)) => {
                        let token = Token::Operator {
                            inner: Functions::User(func),
                        };
                        Ok((token, Borrowed(len), func.arity() > 0))
                    },
                    None => Err(Error::UnknownFunction),
                }
            }
            TokenType::Paren => {
                let (token, kind) = Token::paren(c).unwrap();
                let (unary_, s) = match kind {
                    ParenType::Left => (true, "("),
                    ParenType::Right => (false, ")"),
                };
                Ok((token, Borrowed(s), unary_))
            }
            TokenType::Number => {
                match Token::number(&slice) {
                    Some((token, len)) => Ok((token, Owned(len), false)),
                    None => Err(Error::UnknownToken),
                }
            }
            TokenType::Constant => {
                let (constant, len) = Constant::by_repr(&slice).unwrap();
                let token = Token::Constant { inner: constant };
                Ok((token, Borrowed(len), false))
            }
            TokenType::Variable => {
                // [1..] to ignore the $ prefix
                match Variable::next_variable(&slice[1..], context.vars) {
                    Some((variable, len)) => Ok((Token::Variable { inner: variable }, Borrowed(len), false)),
                    None => Err(Error::UnknownVariable),
                }
            }
            TokenType::Comma => Ok((Token::Comma, Borrowed(","), true)),
        };

        match result {
            Ok((token, cow, unary_)) => {
                let len = cow.len();
                tokens.push(StringToken {
                    inner: Ok(token),
                    repr: cow.into_owned(),
                    idx,
                });
                idx += len;
                unary = unary_;
            },
            Err(e) => {
                partial_token.push(c);
                idx += 1;
                // partial_token.push(c);
                // if current_error.as_ref().map(|er| mem::discriminant(er) != mem::discriminant(&e)).unwrap_or(false) {
                //     tokens.push(StringToken {
                //         inner: Err(e),
                //         repr: partial_token.clone(),
                //         idx: idx - partial_token.len(),
                //     })
                // }
                // partial_token.clear();
            },
        }

        // idx += len;
        // tokens.push(token);
        // unary = unary_;
        // spaces = 0;
    }
    if !partial_token.is_empty() {
        tokens.push(StringToken {
            inner: Err(Error::UnknownToken),
            repr: partial_token.clone(),
            idx: idx - partial_token.len(),
        })
    }
    tokens
}

// #[cfg(test)]
// mod tests {

//     use crate::model::errors::ErrorContext;

//     use super::OperatorType::{Add, Factorial};
//     use super::{tokenize, Error, EvaluationContext, OperatorType, ParenType, Token, Variable};

//     #[test]
//     fn test_tokenize_simple_ok() {
//         let tokens = tokenize("1 + 1", &EvaluationContext::default());
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::Number { value: 1.0 },
//                 Token::operator(Add),
//                 Token::Number { value: 1.0 }
//             ]
//         );

//         let tokens = tokenize("(1 + 1)", &EvaluationContext::default());
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::Paren {
//                     kind: ParenType::Left
//                 },
//                 Token::Number { value: 1.0 },
//                 Token::operator(Add),
//                 Token::Number { value: 1.0 },
//                 Token::Paren {
//                     kind: ParenType::Right
//                 },
//             ]
//         );

//         let tokens = tokenize("1! + 1", &EvaluationContext::default());
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::Number { value: 1.0 },
//                 Token::operator(Factorial),
//                 Token::operator(Add),
//                 Token::Number { value: 1.0 },
//             ]
//         );

//         let tokens = tokenize("sin 5 exp 2 + cos 5^2", &EvaluationContext::default());
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::operator(OperatorType::Sin),
//                 Token::Number { value: 5.0 },
//                 Token::operator(OperatorType::Pow),
//                 Token::Number { value: 2.0 },
//                 Token::operator(OperatorType::Add),
//                 Token::operator(OperatorType::Cos),
//                 Token::Number { value: 5.0 },
//                 Token::operator(OperatorType::Pow),
//                 Token::Number { value: 2.0 }
//             ]
//         );
//     }

//     #[test]
//     fn test_tokenize_unary() {
//         let context = EvaluationContext::default();

//         let tokens = tokenize("1 + -1", &context).unwrap();
//         assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
//         let tokens = tokenize("1 + +1", &context).unwrap();
//         assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
//         let tokens = tokenize("1 + +-", &context).unwrap();
//         assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
//         assert_eq!(tokens[3], Token::operator(OperatorType::Negative));
//         let tokens = tokenize("(+-1)", &context).unwrap();
//         assert_eq!(tokens[1], Token::operator(OperatorType::Positive));
//         assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
//         let tokens = tokenize("-(1)", &context).unwrap();
//         assert_eq!(tokens[0], Token::operator(OperatorType::Negative));
//     }

//     #[test]
//     fn test_tokenize_mismatched_parens() {
//         let context = EvaluationContext::default();

//         let result = tokenize("((1)) + (1))", &context);
//         match result {
//             Err(Error::MismatchingParens) => {}
//             _ => panic!("Expected mismatched parens"),
//         }

//         let result = tokenize("(()", &context);
//         match result {
//             Err(Error::MismatchingParens) => {}
//             _ => panic!("Expected mismatched parens"),
//         }
//     }

//     #[test]
//     fn test_tokenize_parse_error() {
//         let context = EvaluationContext::default();

//         let result = tokenize("1 + 2 + h", &context);
//         assert!(matches!(result, Err(Error::Parsing(8))));
//         let result = tokenize("1 + 2eq + 6", &context);
//         assert!(matches!(result, Err(Error::Parsing(6))));
//     }

//     #[test]
//     fn test_tokenize_unknown_variable() {
//         let vars = [Variable {
//             repr: "q".to_string(),
//             value: 1.0,
//         }];
//         let context = EvaluationContext {
//             vars: &vars,
//             funcs: &[],
//             depth: 0,
//             context: ErrorContext::Main,
//         };
//         let result = tokenize("$x", &context);
//         assert!(matches!(result, Err(Error::UnknownVariable(0))));
//         let result = tokenize("1 * $x", &context);
//         assert!(matches!(result, Err(Error::UnknownVariable(4))));
//     }

//     #[test]
//     fn test_tokenize_variables_ok() {
//         // It's important that these variables are sorted by length in descending order
//         let vars = [
//             Variable {
//                 repr: "xx".to_string(),
//                 value: 10.0,
//             },
//             Variable {
//                 repr: "x".to_string(),
//                 value: 3.0,
//             },
//         ];
//         let context = EvaluationContext {
//             vars: &vars,
//             funcs: &[],
//             depth: 0,
//             context: ErrorContext::Main,
//         };
//         let tokens = tokenize("1 + $x", &context);
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::Number { value: 1.0 },
//                 Token::operator(OperatorType::Add),
//                 Token::Variable {
//                     inner: &context.vars[1]
//                 }
//             ]
//         );

//         let tokens = tokenize("sin $xx pow 5 + cos(6.54)", &context);
//         assert_eq!(
//             tokens.unwrap(),
//             [
//                 Token::operator(OperatorType::Sin),
//                 Token::Variable {
//                     inner: &context.vars[0]
//                 },
//                 Token::operator(OperatorType::Pow),
//                 Token::Number { value: 5.0 },
//                 Token::operator(OperatorType::Add),
//                 Token::operator(OperatorType::Cos),
//                 Token::Paren {
//                     kind: ParenType::Left
//                 },
//                 Token::Number { value: 6.54 },
//                 Token::Paren {
//                     kind: ParenType::Right
//                 }
//             ]
//         );
//     }
// }
