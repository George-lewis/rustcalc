use crate::model::{
    functions::{Function, Functions},
    EvaluationContext,
};

use super::{
    model::{
        constants::Constant,
        errors::Error,
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
pub fn tokenize<'a>(
    string: &str,
    context: &EvaluationContext<'a>,
) -> Result<Vec<Token<'a>>, Error> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut explicit_paren = 0;

    // Indicates that the current operator would be unary
    let mut unary = true;

    let mut idx = 0;
    let end = string.chars().count();
    while idx < end {
        let slice = utils::slice(string, idx, &Pos::End);

        // Current character
        let c = slice.chars().next().unwrap();

        // Ignore whitespace and commas
        if c.is_whitespace() {
            idx += 1;
            continue;
        }

        let kind: TokenType = _type(&slice).ok_or(Error::Parsing(idx))?;

        let (token, len, unary_) = match kind {
            TokenType::Operator => {
                let unar = Operator::unary(&slice);

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (kind, len) = unar.unwrap();

                    // Support for consecutive unary ops
                    (Token::operator(*kind), len, true)
                } else {
                    let (operator, len) = Operator::by_repr(&slice).unwrap();
                    let token = Token::Operator {
                        inner: Functions::Builtin(operator),
                    };

                    // The next token cannot be unary if this operator is factorial
                    // ATM this is the only postfix operator we support
                    (token, len, operator.kind != OperatorType::Factorial)
                }
            }
            TokenType::Function => {
                let (func, len) = Function::next_function(&slice[1..], context.funcs)
                    .ok_or(Error::UnknownFunction(idx))?;
                let token = Token::Operator {
                    inner: Functions::User(func),
                };
                (token, len + 1, func.arity() > 0)
            }
            TokenType::Paren => {
                let (token, kind) = Token::paren(c).unwrap();
                let (paren_mod, unary_) = match kind {
                    ParenType::Left => (1, true),
                    ParenType::Right => (-1, unary),
                };
                explicit_paren += paren_mod;
                (token, 1, unary_)
            }
            TokenType::Number => {
                let (token, len) = Token::number(&slice).ok_or(Error::Parsing(idx))?;
                (token, len, false)
            }
            TokenType::Constant => {
                let (constant, len) = Constant::by_repr(&slice).unwrap();
                let token = Token::Constant { inner: constant };
                (token, len, false)
            }
            TokenType::Variable => {
                // [1..] to ignore the $ prefix
                let (variable, len) = Variable::next_variable(&slice[1..], context.vars)
                    .ok_or(Error::UnknownVariable(idx))?;
                let token = Token::Variable { inner: variable };
                // len + 1 to account for '$'
                (token, len + 1, false)
            }
            TokenType::Comma => (Token::Comma, 1, true),
        };

        idx += len;
        tokens.push(token);
        unary = unary_;
    }
    if explicit_paren == 0 {
        Ok(tokens)
    } else {
        Err(Error::MismatchingParens)
    }
}

#[cfg(test)]
mod tests {

    use crate::model::errors::ErrorContext;

    use super::OperatorType::{Add, Factorial};
    use super::{tokenize, Error, EvaluationContext, OperatorType, ParenType, Token, Variable};

    #[test]
    fn test_tokenize_simple_ok() {
        let tokens = tokenize("1 + 1", &EvaluationContext::default());
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::operator(Add),
                Token::Number { value: 1.0 }
            ]
        );

        let tokens = tokenize("(1 + 1)", &EvaluationContext::default());
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

        let tokens = tokenize("1! + 1", &EvaluationContext::default());
        assert_eq!(
            tokens.unwrap(),
            [
                Token::Number { value: 1.0 },
                Token::operator(Factorial),
                Token::operator(Add),
                Token::Number { value: 1.0 },
            ]
        );

        let tokens = tokenize("sin 5 exp 2 + cos 5^2", &EvaluationContext::default());
        assert_eq!(
            tokens.unwrap(),
            [
                Token::operator(OperatorType::Sin),
                Token::Number { value: 5.0 },
                Token::operator(OperatorType::Pow),
                Token::Number { value: 2.0 },
                Token::operator(OperatorType::Add),
                Token::operator(OperatorType::Cos),
                Token::Number { value: 5.0 },
                Token::operator(OperatorType::Pow),
                Token::Number { value: 2.0 }
            ]
        );
    }

    #[test]
    fn test_tokenize_unary() {
        let context = EvaluationContext::default();

        let tokens = tokenize("1 + -1", &context).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
        let tokens = tokenize("1 + +1", &context).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
        let tokens = tokenize("1 + +-", &context).unwrap();
        assert_eq!(tokens[2], Token::operator(OperatorType::Positive));
        assert_eq!(tokens[3], Token::operator(OperatorType::Negative));
        let tokens = tokenize("(+-1)", &context).unwrap();
        assert_eq!(tokens[1], Token::operator(OperatorType::Positive));
        assert_eq!(tokens[2], Token::operator(OperatorType::Negative));
        let tokens = tokenize("-(1)", &context).unwrap();
        assert_eq!(tokens[0], Token::operator(OperatorType::Negative));
    }

    #[test]
    fn test_tokenize_mismatched_parens() {
        let context = EvaluationContext::default();

        let result = tokenize("((1)) + (1))", &context);
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }

        let result = tokenize("(()", &context);
        match result {
            Err(Error::MismatchingParens) => {}
            _ => panic!("Expected mismatched parens"),
        }
    }

    #[test]
    fn test_tokenize_parse_error() {
        let context = EvaluationContext::default();

        let result = tokenize("1 + 2 + h", &context);
        assert!(matches!(result, Err(Error::Parsing(8))));
        let result = tokenize("1 + 2eq + 6", &context);
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
            context: ErrorContext::Main,
        };
        let result = tokenize("$x", &context);
        assert!(matches!(result, Err(Error::UnknownVariable(0))));
        let result = tokenize("1 * $x", &context);
        assert!(matches!(result, Err(Error::UnknownVariable(4))));
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
            context: ErrorContext::Main,
        };
        let tokens = tokenize("1 + $x", &context);
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

        let tokens = tokenize("sin $xx pow 5 + cos(6.54)", &context);
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
