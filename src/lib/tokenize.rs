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
#[allow(clippy::unnecessary_unwrap, clippy::too_many_lines)]
pub fn tokenize<'a>(string: &str, vars: &'a [Variable]) -> Result<Vec<Token<'a>>, Error> {
    let mut vec: Vec<Token> = Vec::new();
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
                    vec.push(Token::Operator {
                        kind: OperatorType::Mul,
                    });
                }
            }
            coeff = false;
        }

        let kind: TokenType = match _type(&slice) {
            Ok(k) => k,
            Err(()) => {
                return Err(Error::Parsing(idx));
            }
        };

        match kind {
            TokenType::Operator => {
                let unar = Operator::unary(&slice);

                unary = if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (a, b) = unar.unwrap();
                    idx += b;
                    vec.push(Token::Operator { kind: *a });

                    // Support for consecutive unary ops
                    true
                } else {
                    let (operator, n) = Operator::by_repr(&slice).unwrap();

                    idx += n;
                    vec.push(Token::Operator {
                        kind: operator.kind,
                    });

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

                vec.push(t);
                idx += 1;
            }
            TokenType::Number => {
                let (t, n) = match Token::number(&slice) {
                    Some(a) => a,
                    None => return Err(Error::Parsing(idx)),
                };
                idx += n;
                vec.push(t);
                coeff = true;
                unary = false;
            }
            TokenType::Constant => {
                let (constant, n) = Constant::by_repr(&slice).unwrap();
                idx += n;
                vec.push(Token::Constant {
                    kind: constant.kind,
                });
                coeff = true;
                unary = false;
            }
            TokenType::Variable => {
                let (variable, n) = match Variable::next_variable(&slice[1..], vars) {
                    // [1..] to ignore the $ prefix
                    Some(a) => a,
                    None => return Err(Error::UnknownVariable(idx)),
                };
                idx += n + 1; // +1 to account for '$'
                vec.push(Token::Variable { inner: variable });
                coeff = true;
                unary = false;
            }
        }
    }
    if explicit_paren == 0 {
        Ok(vec)
    } else {
        Err(Error::MismatchingParens)
    }
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
