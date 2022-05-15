use std::borrow::Cow;

use crate::model::{
    errors::{EvalError, ParserError, RpnError},
    tokens::{StringToken, Tokens},
};

use super::model::{
    operators::Associativity,
    tokens::{ParenType, Token},
};

/// Convert a list of tokens into Reverse-Polish-Notation
/// * `tokens` - The tokens
///
/// Returns a `Vec` of token in RPN or an `Error::MismatchingParens`. This function will catch
/// some instances of parentheses-mismatch, but not all.
pub fn rpn<'vars, 'funcs>(
    tokens: &[Tokens<'vars, 'funcs>],
) -> Result<Vec<Tokens<'vars, 'funcs>>, RpnError> {
    let mut operator_stack: Vec<Tokens> = Vec::new();
    let mut output: Vec<Tokens> = Vec::with_capacity(tokens.len());

    for token in tokens {
        let cloned = token.clone();
        match token.token() {
            Token::Comma => {}
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
                output.push(cloned);
            }
            Token::Operator { inner: op1 } => {
                while !operator_stack.is_empty() {
                    let last = &operator_stack.last().unwrap().token();
                    if let Token::Paren { kind } = last {
                        if *kind == ParenType::Left {
                            break;
                        }
                    }
                    if let Token::Operator { inner: op2 } = last {
                        if !(op2.precedence() > op1.precedence()
                            || (op2.precedence() == op1.precedence()
                                && op1.associativity() == Associativity::Left))
                        {
                            break;
                        }
                    }
                    output.push(operator_stack.pop().unwrap());
                }
                operator_stack.push(cloned);
            }
            Token::Paren { kind } => match kind {
                ParenType::Left => operator_stack.push(cloned),
                ParenType::Right => {
                    loop {
                        if operator_stack.is_empty() {
                            return Err(RpnError::MismatchingParens);
                        }
                        let tok = operator_stack.pop().unwrap();
                        if let Token::Paren { kind } = tok.token() {
                            if *kind == ParenType::Left {
                                break;
                            }
                        }
                        output.push(tok);
                    }
                    if matches!(
                        operator_stack.last().map(Tokens::token),
                        Some(Token::Operator { .. })
                    ) {
                        output.push(operator_stack.pop().unwrap());
                    }
                }
            },
        }
    }

    // Pop all of `operator_stack` onto `output`
    output.extend(operator_stack.into_iter().rev());

    Ok(output)
}

// #[cfg(test)]
// mod tests {

//     use super::{rpn, ParenType, Token};
//     use crate::model::{errors::EvalError, operators::OperatorType};

//     #[test]
//     fn test_rpn() {
//         let tokens = [
//             Token::Number { value: 1.0 },
//             Token::operator(OperatorType::Add),
//             Token::Number { value: 3.0 },
//         ];
//         let tokens = rpn(&tokens).unwrap();
//         assert_eq!(
//             tokens,
//             [
//                 Token::Number { value: 1.0 },
//                 Token::Number { value: 3.0 },
//                 Token::operator(OperatorType::Add)
//             ]
//         );
//     }

//     #[test]
//     fn test_rpn_mismatched_parens() {
//         let tokens = [
//             Token::Paren {
//                 kind: ParenType::Left,
//             },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//             Token::Paren {
//                 kind: ParenType::Right,
//             },
//         ];
//         let result = rpn(&tokens);
//         assert!(matches!(result, Err(EvalError::MismatchingParens)));
//     }
// }
