use crate::model::{
    functions::Functions,
    operators::{OperatorType, FUNCTIONAL_STYLE_OPERATORS},
    tokens::{ParenType, Token, Tokens},
};

/// Insert implicit parantheses into the tokens.
/// Implicit parentheses are inserted for arguments to functions
/// or function-like operators that accept 0 or 1 arguments
/// Ex: sin sin 2^5 + 9 => sin(sin(2^5)) + 9
pub fn implicit_parens(tokens: &mut Vec<Tokens>) {
    let mut implicit_paren: usize = 0;

    let mut idx = 0;
    while idx < tokens.len() {
        let (cur, next) = {
            let mut iter = tokens.iter();
            (
                iter.nth(idx).unwrap().token(),
                iter.next().map(Tokens::token),
            )
        };

        let preclude = matches!(
            next,
            Some(Token::Paren {
                kind: ParenType::Left,
            })
        );

        // We delay the r_parens when the next operator is pow
        // Because exponents have a higher precedence in BEDMAS
        // So, `sin 5^2` should become `sin(5^2)` NOT `sin(5)^2`
        let delay = match next {
            Some(Token::Operator {
                inner: Functions::Builtin(inner),
            }) => inner.kind == OperatorType::Pow,
            _ => false,
        };

        // Insert r_parens if the current token is a value type AND we're not delaying
        // The delay case should _never_ coincide with the `else if` condition on this block, so it's ok
        if matches!(
            cur,
            Token::Number { .. } | Token::Variable { .. } | Token::Constant { .. }
        ) && !delay
        {
            for offset in 0..implicit_paren {
                tokens.insert(
                    idx + 1 + offset as usize,
                    Tokens::Synthetic(Token::Paren {
                        kind: ParenType::Right,
                    }),
                );
            }
            idx += implicit_paren as usize;
            implicit_paren = 0;
        } else if !preclude {
            let wants_implicit_paren = match cur {
                Token::Operator { inner } => match inner {
                    // Functional-style builtin operators *that only take a single argument*
                    Functions::Builtin(op) => [
                        OperatorType::Sin,
                        OperatorType::Cos,
                        OperatorType::Tan,
                        OperatorType::Sqrt,
                    ]
                    .contains(&op.kind),
                    Functions::User(func) => {
                        if func.arity() == 1 {
                            true
                        } else {
                            tokens.insert(
                                idx + 1,
                                Tokens::Synthetic(Token::Paren {
                                    kind: ParenType::Left,
                                }),
                            );
                            tokens.insert(
                                idx + 2,
                                Tokens::Synthetic(Token::Paren {
                                    kind: ParenType::Right,
                                }),
                            );
                            idx += 2;
                            false
                        }
                    }
                },
                _ => false,
            };
            if wants_implicit_paren {
                tokens.insert(
                    idx + 1,
                    Tokens::Synthetic(Token::Paren {
                        kind: ParenType::Left,
                    }),
                );
                implicit_paren += 1;
                idx += 1;
            }
        }
        idx += 1;
    }
}

/// Insert implicit coefficients into the tokens.
/// It's important that parantheses and commas are present.
/// `tokens` should be run through `implicit_parantheses` before this function
#[allow(clippy::unnested_or_patterns)]
pub fn implicit_coeffs(tokens: &mut Vec<Tokens>) {
    let mut idx = 0;
    while idx < tokens.len() {
        let (cur, next) = {
            let mut iter = tokens.iter();
            (
                iter.nth(idx).unwrap().token(),
                iter.next().map(Tokens::token),
            )
        };

        // Certain tokens preclude coefficients
        // Cases, the next token:
        // - Is an operator, but not a functional-style one
        // - Is an r_paren
        // - Is a comma
        // - Does not exist (meaning `cur` is the last token)
        let precluded = match next {
            Some(Token::Operator {
                inner: Functions::Builtin(op),
            }) => !FUNCTIONAL_STYLE_OPERATORS.contains(&op.kind),
            Some(Token::Paren {
                kind: ParenType::Right,
            })
            | Some(Token::Comma)
            | None => true,
            _ => false,
        };

        if !precluded {
            let can_coeff = match cur {
                // Allows: 5! 5 => 5!*5
                Token::Operator {
                    inner: Functions::Builtin(op),
                } => op.kind == OperatorType::Factorial,
                Token::Number { .. }
                | Token::Constant { .. }
                | Token::Variable { .. }
                | Token::Paren {
                    kind: ParenType::Right,
                } => true,
                _ => false,
            };

            if can_coeff {
                tokens.insert(
                    idx + 1,
                    Tokens::Synthetic(Token::operator(OperatorType::Mul)),
                );
                idx += 1;
            }
        }
        idx += 1;
    }
}

// #[cfg(test)]
// mod tests {
//     #![allow(clippy::missing_const_for_fn)]

//     use super::{implicit_coeffs, implicit_parens};

//     use crate::{
//         model::{
//             errors::ErrorContext, functions::Function, operators::OperatorType, tokens::Token,
//             variables::Variable, EvaluationContext,
//         },
//         tokenize,
//     };

//     #[test]
//     fn test_coeff() {
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

//         let mul = Token::operator(OperatorType::Mul);

//         let mut tokens = tokenize("1 2 3", &context).unwrap();
//         implicit_coeffs(&mut tokens);
//         assert_eq!(tokens[1], mul);
//         assert_eq!(tokens[3], mul);

//         let mut tokens = tokenize("1 $q sin(pi) e", &context).unwrap();
//         implicit_coeffs(&mut tokens);

//         assert_eq!(tokens[1], mul);
//         assert_eq!(tokens[3], mul);
//         assert_eq!(tokens[8], mul);
//     }

//     #[test]
//     fn test_implicit_parens() {
//         let mut tokens = tokenize("sin 5 cos 5", &EvaluationContext::default()).unwrap();
//         implicit_parens(&mut tokens);

//         let funcs = [Function {
//             name: "ident".to_string(),
//             args: vec!["a".to_string()],
//             code: "$a".to_string(),
//         }];
//         let context = EvaluationContext {
//             funcs: &funcs,
//             ..EvaluationContext::default()
//         };
//         let mut tokens = tokenize("#ident 5 + #ident(7) + sin(88)", &context).unwrap();
//         implicit_parens(&mut tokens);
//     }
// }
