use super::{
    model::{
        constants::Constant,
        errors::Error,
        operators::{Associativity, Operator, OperatorType},
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
    Paren,
    Constant,
    Variable,
}

fn _type(s: &str) -> Result<TokenType, ()> {
    Ok(if Token::is_next_number(s) {
        TokenType::Number
    } else if Operator::is(s) {
        TokenType::Operator
    } else if Token::is_next_paren(s) {
        TokenType::Paren
    } else if Constant::is(s) {
        TokenType::Constant
    } else if Variable::is(s) {
        TokenType::Variable
    } else {
        return Err(());
    })
}

/// Tokenize an input string
/// * `string` - A string containing a mathematical expression
/// * `vars` - The available `Variable`s
///
/// Returns a list of tokens or an error
#[allow(clippy::unnecessary_unwrap, clippy::too_many_lines, clippy::missing_errors_doc, clippy::module_name_repetitions)]
pub fn tokenize1<'a, T, F: Fn(Token<'a>, Option<String>) -> T>(string: &str, vars: &'a [Variable], f: F) -> Result<Vec<T>, (Vec<T>, Error)> {
    let mut vec: Vec<T> = Vec::new();
    let mut explicit_paren = 0;

    // Indicates the possibility of an implicit coefficient
    let mut coeff = false;

    // Indicates that the current operator would be unary
    let mut unary = true;

    let char_count = string.chars().count();
    let mut idx = 0;

    while idx < char_count {
        // Current character
        let c = string.chars().nth(idx).unwrap();

        // Ignore whitespace and commas
        if c.is_whitespace() || c == ',' {
            idx += 1;
            coeff = coeff && c != ',';
            continue;
        }

        let slice = utils::slice(string, idx, &Pos::End);

        if coeff {
            // No coefficient if the current character is an r-paren
            let is_r_paren = Token::paren_type(c) == Some(ParenType::Right);
            if !is_r_paren {
                let opt = Operator::by_repr(&slice);
                let is_left_assoc_or_pow = opt.map_or(false, |(op, _)| {
                    op.associativity == Associativity::Left || op.kind == OperatorType::Pow
                });

                // Only a coefficient if the next (current) token is not
                // A left-associative function or pow
                if !is_left_assoc_or_pow {
                    vec.push(f(Token::Operator {
                        kind: OperatorType::Mul,
                    }, None));
                }
            }
            coeff = false;
        }

        let kind: TokenType = match _type(&slice) {
            Ok(k) => k,
            Err(()) => {
                return Err((vec, Error::Parsing(idx)));
            }
        };

        match kind {
            TokenType::Operator => {
                let unar = Operator::unary(&slice);

                unary = if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (a, b) = unar.unwrap();
                    idx += b.chars().count();
                    vec.push(f(Token::Operator { kind: *a }, Some(b.to_string())));

                    // Support for consecutive unary ops
                    true
                } else {
                    let (operator, s) = Operator::by_repr(&slice).unwrap();

                    idx += s.chars().count();
                    vec.push(f(Token::Operator {
                        kind: operator.kind,
                    }, Some(s.to_owned())));

                    // The next token cannot be unary if this operator is factorial
                    // ATM this is the only postfix operator we support
                    operator.kind != OperatorType::Factorial
                };
            }
            TokenType::Paren => {
                let (t, kind) = Token::paren(c).unwrap();
                match kind {
                    ParenType::Left => {
                        // Covers cases like `sin(-x)`
                        unary = true;
                        explicit_paren += 1;
                    }
                    ParenType::Right => {
                        // Covers cases like `sin(x) y => sin(x) * y`
                        coeff = true;
                        explicit_paren -= 1;
                    }
                }

                vec.push(f(t, Some(c.to_string())));
                idx += 1;
            }
            TokenType::Number => {
                let (t, n) = match Token::number(&slice) {
                    Some((t, s)) => (f(t, Some(s.to_string())), s.chars().count()),
                    None => return Err((vec, Error::Parsing(idx))),
                };
                idx += n;
                vec.push(t);
                coeff = true;
                unary = false;
            }
            TokenType::Constant => {
                let (constant, s) = Constant::by_repr(&slice).unwrap();
                idx += s.chars().count();
                vec.push(f(Token::Constant {
                    kind: constant.kind,
                }, Some(s.to_string())));
                coeff = true;
                unary = false;
            }
            TokenType::Variable => {
                let (variable, n) = match Variable::next_variable(&slice[1..], vars) {
                    // [1..] to ignore the $ prefix
                    Some(a) => a,
                    None => return Err((vec, Error::UnknownVariable(idx))),
                };
                idx += n.chars().count() + 1; // +1 to account for '$'
                vec.push(f(Token::Variable { inner: variable }, Some(n.to_string())));
                coeff = true;
                unary = false;
            }
        }
    }
    if explicit_paren == 0 {
        Ok(vec)
    } else {
        Err((vec, Error::MismatchingParens))
    }
}

pub fn tokenize<'a>(string: &str, vars: &'a [Variable]) -> Result<Vec<Token<'a>>, Error> {
    tokenize1(string, vars, |a, b| a).map_err(|(_, a)| a)
}

#[cfg(test)]
mod tests {

    use super::{tokenize, Error, OperatorType, ParenType, Token, Variable};

    #[test]
    fn test_tokenize_simple_ok() {
        let tokens = tokenize("1 + 1", &[]);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Number { value: 1.0 }
            ]
        );

        let tokens = tokenize("(1 + 1)", &[]);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Paren {
                    kind: ParenType::Left
                },
                Token::Number { value: 1.0 },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Number { value: 1.0 },
                Token::Paren {
                    kind: ParenType::Right
                },
            ]
        );

        let tokens = tokenize("1! + 1", &[]);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::Operator {
                    kind: OperatorType::Factorial
                },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Number { value: 1.0 },
            ]
        );

        let tokens = tokenize("sin 5 exp 2 + cos 5^2", &[]);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Operator {
                    kind: OperatorType::Sin
                },
                Token::Number { value: 5.0 },
                Token::Operator {
                    kind: OperatorType::Pow
                },
                Token::Number { value: 2.0 },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Operator {
                    kind: OperatorType::Cos
                },
                Token::Number { value: 5.0 },
                Token::Operator {
                    kind: OperatorType::Pow
                },
                Token::Number { value: 2.0 }
            ]
        );
    }

    #[test]
    fn test_tokenize_unary() {
        let tokens = tokenize("1 + -1", &[]).unwrap();
        assert_eq!(
            tokens[2],
            Token::Operator {
                kind: OperatorType::Negative
            }
        );
        let tokens = tokenize("1 + +1", &[]).unwrap();
        assert_eq!(
            tokens[2],
            Token::Operator {
                kind: OperatorType::Positive
            }
        );
        let tokens = tokenize("1 + +-", &[]).unwrap();
        assert_eq!(
            tokens[2],
            Token::Operator {
                kind: OperatorType::Positive
            }
        );
        assert_eq!(
            tokens[3],
            Token::Operator {
                kind: OperatorType::Negative
            }
        );
        let tokens = tokenize("(+-1)", &[]).unwrap();
        assert_eq!(
            tokens[1],
            Token::Operator {
                kind: OperatorType::Positive
            }
        );
        assert_eq!(
            tokens[2],
            Token::Operator {
                kind: OperatorType::Negative
            }
        );
        let tokens = tokenize("-(1)", &[]).unwrap();
        assert_eq!(
            tokens[0],
            Token::Operator {
                kind: OperatorType::Negative
            }
        );
    }

    #[test]
    fn test_tokenize_mismatched_parens() {
        let result = tokenize("((1)) + (1))", &[]);
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }

        let result = tokenize("(()", &[]);
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }
    }

    #[test]
    fn test_tokenize_parse_error() {
        let result = tokenize("1 + 2 + h", &[]);
        assert!(matches!(result, Err(Error::Parsing(8))));
        let result = tokenize("1 + 2eq + 6", &[]);
        assert!(matches!(result, Err(Error::Parsing(6))));
    }

    #[test]
    fn test_tokenize_unknown_variable() {
        let vars = [Variable {
            repr: "q".to_string(),
            value: 1.0,
        }];
        let result = tokenize("$x", &vars);
        assert!(matches!(result, Err(Error::UnknownVariable(0))));
        let result = tokenize("1 * $x", &vars);
        assert!(matches!(result, Err(Error::UnknownVariable(4))));
    }

    #[test]
    fn test_tokenize_implicit_coeff() {
        let vars = [Variable {
            repr: "q".to_string(),
            value: 1.0,
        }];

        let mul = Token::Operator {
            kind: OperatorType::Mul,
        };

        let tokens = tokenize("1 2 3", &vars).unwrap();
        assert_eq!(tokens[1], mul);
        assert_eq!(tokens[3], mul);

        let tokens = tokenize("1 $q sin(pi) e", &vars).unwrap();
        assert_eq!(tokens[1], mul);
        assert_eq!(tokens[3], mul);
        assert_eq!(tokens[8], mul);
    }

    #[test]
    fn test_tokenize_variables_ok() {
        // It's important that these variables are sorted by length in descending order
        let vars = [
            Variable {
                repr: "xx".to_string(),
                value: 10.0,
            },
            Variable {
                repr: "x".to_string(),
                value: 3.0,
            },
        ];
        let tokens = tokenize("1 + $x", &vars);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Variable { inner: &vars[1] }
            ]
        );

        let tokens = tokenize("sin $xx pow 5 + cos(6.54)", &vars);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Operator {
                    kind: OperatorType::Sin
                },
                Token::Variable { inner: &vars[0] },
                Token::Operator {
                    kind: OperatorType::Pow
                },
                Token::Number { value: 5.0 },
                Token::Operator {
                    kind: OperatorType::Add
                },
                Token::Operator {
                    kind: OperatorType::Cos
                },
                Token::Paren {
                    kind: ParenType::Left
                },
                Token::Number { value: 6.54 },
                Token::Paren {
                    kind: ParenType::Right
                }
            ]
        );
    }
}
