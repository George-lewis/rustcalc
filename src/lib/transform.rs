use crate::model::{
    functions::Functions,
    operators::OperatorType,
    tokens::{ParenType, Token},
};

pub fn implicit_parens(tokens: &mut Vec<Token>) {
    let mut implicit_paren: u8 = 0;

    let mut idx = 0;
    while idx < tokens.len() {
        let (cur, next) = {
            let mut iter = tokens.iter();
            (iter.nth(idx).unwrap(), iter.next())
        };

        let wants_implicit_paren = match cur {
            Token::Operator { inner } => match inner {
                Functions::Builtin(op) => [
                    OperatorType::Sin,
                    OperatorType::Cos,
                    OperatorType::Tan,
                    OperatorType::Sqrt,
                ]
                .contains(&op.kind),
                Functions::User(func) => func.arity() <= 1,
            },
            _ => false,
        };

        let preclude = matches!(
            next,
            Some(Token::Paren {
                kind: ParenType::Left,
            })
        );

        if wants_implicit_paren && !preclude {
            tokens.insert(
                idx + 1,
                Token::Paren {
                    kind: ParenType::Left,
                },
            );
            implicit_paren += 1;
            idx += 1;
        } else if matches!(
            cur,
            Token::Number { .. } | Token::Variable { .. } | Token::Constant { .. }
        ) {
            for offset in 0..implicit_paren {
                tokens.insert(
                    idx + 1 + offset as usize,
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                );
            }
            idx += implicit_paren as usize;
            implicit_paren = 0;
        }
        idx += 1;
    }
}

#[allow(clippy::unnested_or_patterns)]
pub fn implicit_coeffs(tokens: &mut Vec<Token>) {
    let mut idx = 0;
    while idx < tokens.len() {
        let (cur, next) = {
            let mut iter = tokens.iter();
            (iter.nth(idx).unwrap(), iter.next())
        };

        // Certain tokens preclude coefficients
        // For example, we want to prevent: `1 * + 5`
        let precluded = matches!(
            next,
            Some(Token::Operator { .. })
                | Some(Token::Paren {
                    kind: ParenType::Right,
                })
                | Some(Token::Comma)
                | None
        );

        if !precluded {
            let can_coeff = matches!(
                cur,
                Token::Number { .. }
                    | Token::Constant { .. }
                    | Token::Variable { .. }
                    | Token::Paren {
                        kind: ParenType::Right
                    }
            );

            if can_coeff {
                tokens.insert(idx + 1, Token::operator(OperatorType::Mul));
                idx += 1;
            }
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{implicit_coeffs, implicit_parens};

    use crate::{
        model::{functions::Function, EvaluationContext},
        tokenize,
    };

    #[test]
    fn test_coeff() {
        let mut tokens = tokenize("1 (2) 3!", &EvaluationContext::default()).unwrap();
        implicit_coeffs(&mut tokens);

        let funcs = [Function {
            name: "fixed".to_string(),
            args: vec![],
            code: "5".to_string(),
        }];

        let context = EvaluationContext {
            funcs: &funcs,
            ..EvaluationContext::default()
        };

        let mut tokens = tokenize("#fixed()^2", &context).unwrap();
        tokens.remove(1);
        implicit_coeffs(&mut tokens);
    }

    #[test]
    fn test_implicit_parens() {
        let mut tokens = tokenize("sin 5 cos 5", &EvaluationContext::default()).unwrap();
        implicit_parens(&mut tokens);

        let funcs = [Function {
            name: "ident".to_string(),
            args: vec!["a".to_string()],
            code: "$a".to_string(),
        }];
        let context = EvaluationContext {
            funcs: &funcs,
            ..EvaluationContext::default()
        };
        let mut tokens = tokenize("#ident 5 + #ident(7) + sin(88)", &context).unwrap();
        implicit_parens(&mut tokens);
    }
}
