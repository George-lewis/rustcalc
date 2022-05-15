use itertools::Itertools;

use crate::model::{
    functions::{Function, Functions},
    tokens::{PartialToken, StringToken},
    EvaluationContext,
};

use std::rc::Rc;

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
pub fn tokenize<'vars, 'funcs>(
    string: &'funcs str,
    context: &EvaluationContext<'vars, 'funcs>,
) -> Result<Vec<StringToken<'funcs, 'funcs>>, Vec<PartialToken<'funcs, 'funcs>>> {
    let mut tokens: Vec<PartialToken> = Vec::new();

    // Indicates that the current operator would be unary
    let mut unary = true;
    let mut partial_token: Option<usize> = None;
    let mut idx = 0;
    let end = string.chars().count();

    while idx < end {
        let slice = utils::slice(string, idx, &Pos::End);

        // Current character
        let c = slice.chars().next().unwrap();

        let whitespace = c.is_whitespace();
        let kind = _type(slice);

        match (whitespace, &kind) {
            // don't finish partial tokens on a possible constant
            (false, Some(ty)) if *ty == TokenType::Constant => {
                // This is here to block the next arm; do nothing
            }
            // If this is whitespace or a good token (not a constant)
            // we complete our partial unknown token
            (true, None) | (false, Some(_)) => {
                if let Some(idx_) = partial_token {
                    tokens.push(PartialToken {
                        inner: Err(Error::UnknownToken),
                        repr: utils::slice(string, idx_, &Pos::Idx(idx)),
                        idx: idx_,
                    });
                    partial_token = None;
                }
            },
            // Some garbage
            (false, None) => {
                // Start a new partial unknown token if there isn't one already
                if partial_token.is_none() {
                    partial_token = Some(idx);
                }

                idx += 1;
                continue;
            },
            (true, Some(_)) => unreachable!("A character cannot be whitespace and a valid kind at the same time"),
        }

        if whitespace {
            idx += 1;
            continue;
        }

        // Safety: If this is none, then either whitespace is true
        // and thus we have continued or we started a new partial token and continued
        let kind = kind.unwrap();

        let result = match kind {
            TokenType::Operator => {
                let unar = Operator::unary(slice);

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (kind, len) = unar.unwrap();

                    // Support for consecutive unary ops
                    Ok((Token::operator(*kind), len, true))
                } else {
                    let (operator, len) = Operator::by_repr(slice).unwrap();
                    let token = Token::Operator {
                        inner: Functions::Builtin(operator),
                    };

                    // The next token cannot be unary if this operator is factorial
                    // ATM this is the only postfix operator we support
                    Ok((token, len, operator.kind != OperatorType::Factorial))
                }
            }
            TokenType::Function => match Function::next_function(&slice[1..], context.funcs) {
                Some((func, len)) => {
                    let token = Token::Operator {
                        inner: Functions::User(func),
                    };
                    // For the prefix
                    idx += 1;
                    Ok((token, len, func.arity() > 0))
                }
                None => Err(Error::UnknownFunction),
            },
            TokenType::Paren => {
                let (token, kind) = Token::paren(c).unwrap();
                let (unary_, s) = match kind {
                    ParenType::Left => (true, "("),
                    ParenType::Right => (false, ")"),
                };
                Ok((token, s, unary_))
            }
            TokenType::Number => match Token::number(slice) {
                Some((token, len)) => Ok((token, len, false)),
                None => Err(Error::UnknownToken),
            },
            TokenType::Constant => {
                // If partial_token is empty, then we're not in an unknown blob
                // or there is a space preceeding, so we're ok to find constants
                if partial_token == None {
                    let (constant, len) = Constant::by_repr(slice).unwrap();
                    let token = Token::Constant { inner: constant };
                    Ok((token, len, false))
                } else {
                    Err(Error::UnknownToken)
                }
            }
            TokenType::Variable => {
                // Err(Error::UnknownVariable)
                // [1..] to ignore the $ prefix
                match Variable::next_variable(&slice[1..], context.vars) {
                    Some((variable, len)) => {
                        // For the prefix
                        idx += 1;
                        Ok((
                            Token::Variable {
                                inner: Rc::clone(variable),
                            },
                            len,
                            false,
                        ))
                    }
                    None => Err(Error::UnknownVariable),
                }
            }
            TokenType::Comma => Ok((Token::Comma, ",", true)),
        };

        match result {
            Ok((token, cow, unary_)) => {
                let len = cow.len();
                tokens.push(PartialToken {
                    inner: Ok(token),
                    repr: cow,
                    idx,
                });
                idx += len;
                unary = unary_;
            }
            Err(e) => match e {
                // If the token is unknown, we check
                // if we have to start a new partial and advance the read
                Error::UnknownToken => {
                    if partial_token.is_none() {
                        partial_token = Some(idx);
                    }
                    idx += 1;
                }
                Error::UnknownVariable | Error::UnknownFunction => {
                    let until = slice
                        .char_indices()
                        .find_map(|(i, c)| {
                            if [' ', '('].contains(&c) {
                                Some(i)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(slice.len());
                    tokens.push(PartialToken {
                        inner: Err(e),
                        repr: &slice[..until],
                        idx,
                    });
                    idx += until;
                }
            },
        }
    }

    // If the line ends with bad input we need to push the final partial token
    if let Some(idx) = partial_token {
        tokens.push(PartialToken {
            inner: Err(Error::UnknownToken),
            repr: utils::slice(string, idx, &Pos::End),
            idx,
        });
    }

    if tokens.iter().any(|pt: &PartialToken| pt.inner.is_err()) {
        Err(tokens)
    } else {
        let vec = tokens
            .into_iter()
            .map(|pt: PartialToken| StringToken {
                inner: pt.inner.unwrap(),
                repr: pt.repr,
                idx: pt.idx,
            })
            .collect_vec();
        Ok(vec)
    }
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
