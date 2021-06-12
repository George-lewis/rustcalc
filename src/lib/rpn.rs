use super::model::{
    errors::Error,
    operators::Associativity,
    tokens::{ParenType, Token},
};

/// Convert a list of tokens into Reverse-Polish-Notation
/// * `tokens` - The tokens
///
/// Returns a `Vec` of token in RPN or an `Error::MismatchingParens`. This function will catch
/// some instances of parentheses-mismatch, but not all.
pub fn rpn<'a>(tokens: &'a [Token]) -> Result<Vec<Token<'a>>, Error> {
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token {
            Token::Comma => {}
            Token::Number { .. } | Token::Constant { .. } | Token::Variable { .. } => {
                output.push(*token)
            }
            Token::Operator { inner: op1 } => {
                // let op1 = Operator::by_type(*kind);
                while !operator_stack.is_empty() {
                    let last = operator_stack.last().unwrap();
                    if let Token::Paren { kind } = last {
                        if *kind == ParenType::Left {
                            break;
                        }
                    }
                    if let Token::Operator { inner: op2 } = last {
                        // let op2 = Operator::by_type(*kind);
                        if !(op2.precedence() > op1.precedence()
                            || (op2.precedence() == op1.precedence()
                                && op1.associativity() == Associativity::Left))
                        {
                            break;
                        }
                    }
                    output.push(operator_stack.pop().unwrap());
                }
                operator_stack.push(*token);
            }
            Token::Paren { kind } => match kind {
                ParenType::Left => operator_stack.push(*token),
                ParenType::Right => {
                    loop {
                        if operator_stack.is_empty() {
                            return Err(Error::MismatchingParens);
                        }
                        let op = operator_stack.pop().unwrap();
                        if let Token::Paren { kind } = op {
                            if kind == ParenType::Left {
                                break;
                            }
                        }
                        output.push(op);
                    }
                    if matches!(operator_stack.last(), Some(Token::Operator { .. })) {
                        output.push(operator_stack.pop().unwrap());
                    }
                }
            },
        }
    }

    // Pop all of `operator_stack` onto `output`
    output.extend(operator_stack.iter().rev());

    Ok(output)
}

#[cfg(test)]
mod tests {

    use super::{rpn, Error, ParenType, Token};
    use crate::model::operators::OperatorType;

    #[test]
    fn test_rpn() {
        let tokens = [
            Token::Number { value: 1.0 },
            Token::operator(OperatorType::Add),
            Token::Number { value: 3.0 },
        ];
        let tokens = rpn(&tokens).unwrap();
        assert_eq!(
            tokens,
            [
                Token::Number { value: 1.0 },
                Token::Number { value: 3.0 },
                Token::operator(OperatorType::Add)
            ]
        );
    }

    #[test]
    fn test_rpn_mismatched_parens() {
        let tokens = [
            Token::Paren {
                kind: ParenType::Left,
            },
            Token::Paren {
                kind: ParenType::Right,
            },
            Token::Paren {
                kind: ParenType::Right,
            },
        ];
        let result = rpn(&tokens);
        assert!(matches!(result, Err(Error::MismatchingParens)));
    }
}
