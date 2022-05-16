use crate::model::{
    functions::Functions,
    tokens::{ParenType, Token, Tokens},
};

fn check_arg_count(tokens: &[Tokens], idx: usize, needed: usize) -> bool {
    let iter = tokens.iter().skip(idx);

    let mut count = 0;
    let mut bracket_level = 0;

    for token in iter {
        match token.token() {
            // Blocks the next operator match from being a non-function
            Token::Operator { inner } if !inner.is_function() => {}
            Token::Number { .. }
            | Token::Constant { .. }
            | Token::Variable { .. }
            | Token::Operator { .. } => {
                if bracket_level == 1 {
                    count += 1;
                }
            }
            Token::Paren {
                kind: ParenType::Left,
            } => {
                bracket_level += 1;
            }
            Token::Paren {
                kind: ParenType::Right,
            } => {
                bracket_level -= 1;
            }
            _ => {}
        }
        if bracket_level == 0 {
            return count == needed;
        }
    }

    count == needed
}

#[allow(clippy::module_name_repetitions)]
pub fn verify_fn_args<'vars, 'funcs>(
    tokens: &[Tokens<'vars, 'funcs>],
) -> Option<(Tokens<'vars, 'funcs>, Functions<'funcs>)> {
    for (idx, tok) in tokens.iter().enumerate() {
        if let Token::Operator { inner } = tok.token() {
            if !inner.is_function() {
                continue;
            }

            if !check_arg_count(tokens, idx + 1, inner.arity()) {
                return Some((tok.clone(), *inner));
            }
        }
    }
    None
}
