use crate::model::{
    self,
    functions::{Function, Functions},
    EvaluationContext,
};

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
    Function,
    Paren,
    Constant,
    Variable,
}

#[allow(clippy::clippy::iter_nth_zero)]
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
    } else if Function::is(s) {
        TokenType::Function
    } else {
        return Err(());
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
pub fn tokenize<'a>(string: &str, context: EvaluationContext<'a>) -> Result<Vec<Token<'a>>, Error> {
    let mut vec: Vec<Token> = Vec::new();
    let mut explicit_paren = 0;
    let mut idx = 0;
    let mut coeff = false;
    let mut unary = true;
    while idx < string.chars().count() {
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
                    vec.push(Token::operator(OperatorType::Mul));
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

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (a, b) = unar.unwrap();
                    idx += b;
                    vec.push(Token::operator(*a));
                } else {
                    let (operator, n) = Operator::by_repr(&slice).unwrap();

                    idx += n;
                    vec.push(Token::Operator {
                        inner: model::functions::Functions::Builtin(operator),
                    });
                }
                unary = true;
            }
            TokenType::Function => {
                let search = Function::next_function(&slice[1..], context.funcs);
                let (func, n) = match search {
                    Some(x) => x,
                    None => return Err(Error::UnknownFunction(idx)),
                };
                vec.push(Token::Operator {
                    inner: Functions::User(func),
                });
                idx += n + 1;
                if func.arity() == 0 {
                    unary = false;
                    coeff = true;
                } else {
                    unary = true;
                }
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
                vec.push(Token::Constant { inner: constant });
                coeff = true;
                unary = false;
            }
            TokenType::Variable => {
                let (variable, n) = match Variable::next_variable(&slice[1..], context.vars) {
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

    use super::OperatorType::Add;
    use super::{tokenize, Error, EvaluationContext, OperatorType, ParenType, Token, Variable};

    #[test]
    fn test_tokenize_simple_ok() {
        let tokens = tokenize("1 + 1", EvaluationContext::default());
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::operator(Add),
                Token::Number { value: 1.0 }
            ]
        );

        let tokens = tokenize("(1 + 1)", EvaluationContext::default());
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Paren {
                    kind: ParenType::Left
                },
                Token::Number { value: 1.0 },
                Token::operator(Add),
                Token::Number { value: 1.0 },
                Token::Paren {
                    kind: ParenType::Right
                },
            ]
        );
    }

    #[test]
    fn test_tokenize_unary() {
        let tokens = tokenize("1 + -1", EvaluationContext::default()).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
        let tokens = tokenize("1 + +1", EvaluationContext::default()).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
        let tokens = tokenize("1 + +-", EvaluationContext::default()).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
        assert_eq!(tokens[3], Token::operator(OperatorType::Negative));
        let tokens = tokenize("(+-1)", EvaluationContext::default()).unwrap();
        assert_eq!(tokens[1], Token::operator(OperatorType::Positive));
        assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
        let tokens = tokenize("-(1)", EvaluationContext::default()).unwrap();
        assert_eq!(tokens[0], Token::operator(OperatorType::Negative));
    }

    #[test]
    fn test_tokenize_mismatched_parens() {
        let result = tokenize("((1)) + (1))", EvaluationContext::default());
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }

        let result = tokenize("(()", EvaluationContext::default());
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }
    }

    #[test]
    fn test_tokenize_parse_error() {
        let result = tokenize("1 + 2 + h", EvaluationContext::default());
        assert!(matches!(result, Err(Error::Parsing(8))));
        let result = tokenize("1 + 2eq + 6", EvaluationContext::default());
        assert!(matches!(result, Err(Error::Parsing(6))));
    }

    #[test]
    fn test_tokenize_unknown_variable() {
        let vars = [Variable {
            repr: "q".to_string(),
            value: 1.0,
        }];
        let context = EvaluationContext {
            vars: &vars,
            funcs: &[],
            depth: 0,
        };
        let result = tokenize("$x", context);
        assert!(matches!(result, Err(Error::UnknownVariable(0))));
        let result = tokenize("1 * $x", context);
        assert!(matches!(result, Err(Error::UnknownVariable(4))));
    }

    #[test]
    fn test_tokenize_implicit_coeff() {
        let vars = [Variable {
            repr: "q".to_string(),
            value: 1.0,
        }];
        let context = EvaluationContext {
            vars: &vars,
            funcs: &[],
            depth: 0,
        };

        let mul = Token::operator(OperatorType::Mul);

        let tokens = tokenize("1 2 3", context).unwrap();
        assert_eq!(tokens[1], mul);
        assert_eq!(tokens[3], mul);

        let tokens = tokenize("1 $q sin(pi) e", context).unwrap();
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
        let context = EvaluationContext {
            vars: &vars,
            funcs: &[],
            depth: 0,
        };
        let tokens = tokenize("1 + $x", context);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::operator(OperatorType::Add),
                Token::Variable {
                    inner: &context.vars[1]
                }
            ]
        );

        let tokens = tokenize("sin $xx pow 5 + cos(6.54)", context);
        assert_eq!(
            tokens.unwrap(),
            [
                Token::operator(OperatorType::Sin),
                Token::Variable {
                    inner: &context.vars[0]
                },
                Token::operator(OperatorType::Pow),
                Token::Number { value: 5.0 },
                Token::operator(OperatorType::Add),
                Token::operator(OperatorType::Cos),
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
