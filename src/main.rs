#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::wildcard_imports)]

use std::{
    fmt::{Debug, Display},
    io::Write,
    process::exit,
};

use colored::*;
use tokens::*;

mod tokens;
mod utils;

#[derive(Debug)]
enum RMEError {
    ParsingError(usize),
    OperandError(OperatorType),
    EmptyStack,
    MismatchingParens,
}
#[derive(Clone, Debug, PartialEq)]
enum TokenType {
    NUMBER,
    OPERATOR,
    PAREN,
    CONSTANT,
}

fn main() {
    loop {
        print!("> ");
        std::io::stdout().flush().ok().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Trailing newlines
        input = input.trim_end().to_string();

        if input.is_empty() {
            continue;
        }

        if input.to_lowercase() == "quit" {
            println!("{}{}", "Bye".blue(), "!".red());
            exit(0);
        }

        let (x, repr) = match doeval(&input) {
            Ok((a, b)) => (a, b),
            Err(e) => {
                match e {
                    RMEError::ParsingError(idx) => {
                        let first = if idx > 0 {
                            utils::slice(&input, 0, (idx) as i64)
                        } else {
                            "".to_string()
                        };
                        println!(
                            "Couldn't parse the token at index [{}]\n{}{}{}\n{}{}",
                            idx.to_string().red(),
                            first,
                            input.chars().nth(idx).unwrap().to_string().on_red().white(),
                            utils::slice(&input, idx + 1, -0),
                            "~".repeat(idx).red().bold(),
                            "^".red()
                        );
                    }
                    RMEError::OperandError(kind) => {
                        println!(
                            "Couldn't evaluate. Operator [{}] requires an operand.",
                            format!("{:?}", kind).green()
                        );
                    }
                    RMEError::EmptyStack => {
                        println!("Couldn't evalutate. Stack was empty?");
                    }
                    RMEError::MismatchingParens => {
                        println!("Couldn't evaluate. Mismatched parens.");
                    }
                }

                continue;
            }
        };

        let formatted = stringify(&repr, color_cli);

        println!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());
    }
}

fn next_num(string: &str) -> String {
    string
        .chars()
        .take_while(|c| NUMBER_CHARACTERS.contains(c))
        .collect::<String>()
}

#[allow(clippy::unnecessary_unwrap)]
fn tokenize(string: &str) -> Result<Vec<Token>, RMEError> {
    let mut vec: Vec<Token> = Vec::new();
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

        // Slice the input from the index until the end
        let slice = utils::slice(string, idx, -0);

        if coeff {
            // No coefficient if the current character is an r-paren
            if c != ')' {
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

        let kind = match _type(&slice) {
            Ok(k) => k,
            Err(..) => {
                return Err(RMEError::ParsingError(idx));
            }
        };

        match kind {
            TokenType::OPERATOR => {
                let unar = Operator::unary(&slice);

                if unary && unar.is_some() {
                    // Current token is a unary operator
                    let (a, b) = unar.unwrap();
                    idx += b.chars().count();
                    vec.push(Token::Operator { kind: *a });
                    unary = false;
                } else {
                    unary = true;

                    let (op, s) = Operator::by_repr(&slice).unwrap();

                    idx += s.chars().count();
                    vec.push(Token::Operator { kind: op.kind });
                }
            }
            TokenType::PAREN => {
                let (t, kind) = Token::paren(c);
                match kind {
                    ParenType::Left => {
                        // Covers cases like `sin(-x)`
                        unary = true;
                    }
                    ParenType::Right => {
                        // Covers cases like `sin(x) y => sin(x) * y`
                        coeff = true;
                    }
                }

                vec.push(t);
                idx += 1;
            }
            TokenType::NUMBER => {
                let num = next_num(&utils::slice(string, idx, -0));
                let value = match num.parse::<f64>() {
                    Ok(x) => x,
                    Err(..) => return Err(RMEError::ParsingError(idx)),
                };
                idx += num.chars().count();
                vec.push(Token::Number { value });
                coeff = true;
                unary = false;
            }
            TokenType::CONSTANT => {
                let (constant, s) = Constant::by_repr(&slice).unwrap();
                idx += s.chars().count();
                vec.push(Token::Constant {
                    kind: constant.kind,
                });
                coeff = true;
                unary = false;
            }
        }
    }
    Ok(vec)
}

#[allow(clippy::clippy::iter_nth_zero)]
fn _type(s: &str) -> Result<TokenType, ()> {
    let c = &s.chars().nth(0).unwrap();
    Ok(if NUMBER_CHARACTERS.contains(c) {
        TokenType::NUMBER
    } else if Operator::is(s) {
        TokenType::OPERATOR
    } else if PAREN_CHARACTERS.contains(c) {
        TokenType::PAREN
    } else if Constant::is(s) {
        TokenType::CONSTANT
    } else {
        return Err(());
    })
}

fn rpn(tokens: &[Token]) -> Result<Vec<Token>, RMEError> {
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::with_capacity(tokens.len());

    for token in tokens {
        match token {
            Token::Number { .. } | Token::Constant { .. } => output.push(*token),
            Token::Operator { kind } => {
                let op1 = Operator::by_type(*kind);
                while !operator_stack.is_empty() {
                    let last = operator_stack.last().unwrap();
                    if matches!(
                        last,
                        Token::Paren {
                            kind: ParenType::Left
                        }
                    ) {
                        break;
                    }
                    if let Token::Operator { kind } = last {
                        let op2 = Operator::by_type(*kind);
                        if !(op2.precedence > op1.precedence
                            || (op2.precedence == op1.precedence
                                && op1.associativity == Associativity::Left))
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
                            return Err(RMEError::MismatchingParens);
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

    while !operator_stack.is_empty() {
        output.push(operator_stack.pop().unwrap());
    }

    Ok(output)
}

fn eval(tokens: &[Token]) -> Result<f64, RMEError> {
    // We need a mutable copy of the tokens
    let mut stack: Vec<Token> = tokens.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new();

    while !stack.is_empty() {
        let token = stack.pop().unwrap();

        match token {
            Token::Number { value } => {
                args.push(value);
            }
            Token::Constant { kind } => {
                let constant = Constant::by_type(kind);
                args.push(constant.value);
            }
            Token::Operator { kind } => {
                let op = Operator::by_type(kind);
                let start = match args.len().checked_sub(op.arity) {
                    Some(x) => x,
                    None => return Err(RMEError::OperandError(op.kind)),
                };

                // Takes the last `op.arity` number of values from `args`
                // `start = args.len() - op.arity`
                let args_: Vec<f64> = args.drain(start..).collect();
                let result = (op.doit)(&args_);

                // Push the result of the evaluation
                stack.push(Token::Number { value: result });
            }
            Token::Paren { .. } => {}
        }
    }

    // Result
    if stack.is_empty() && args.len() == 1 {
        return Ok(args[0]);
    }
    Err(RMEError::EmptyStack)
}

fn doeval(string: &str) -> Result<(f64, Vec<Token>), RMEError> {
    let tokens = tokenize(string)?;
    let rpn = rpn(&tokens)?;
    let result = eval(&rpn)?;
    Ok((result, tokens))
}

fn color_cli(string: &str, token: &Token) -> ColoredString {
    match token {
        Token::Number { .. } => string.clear(),
        Token::Operator { kind } => {
            let op = Operator::by_type(*kind);
            if op.associativity == Associativity::Left {
                string.green().bold()
            } else {
                string.blue().bold()
            }
        }
        Token::Paren { .. } => string.magenta(),
        Token::Constant { .. } => string.yellow(),
    }
}

fn stringify<F, T: Display>(tokens: &[Token], colorize: F) -> String
where
    F: Fn(&str, &Token) -> T,
{
    let mut out = String::new();
    let mut implicit_paren: usize = 0;
    for (idx, token) in tokens.iter().enumerate() {
        let colored: T = colorize(&token.ideal_repr(), token);
        let append = match *token {
            Token::Number { .. } | Token::Constant { .. } => {
                let is_r_paren = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    })
                );

                let is_op = matches!(tokens.get(idx + 1), Some(Token::Operator { .. }));

                let is_pow = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Operator {
                        kind: OperatorType::Pow
                    })
                );

                let last = idx == tokens.len() - 1;

                let appendix = if implicit_paren > 0 {
                    let space = if last || is_pow { "" } else { " " };
                    let r_paren: T = colorize(&")".repeat(implicit_paren), &Token::Paren{kind:ParenType::Right});
                    format!("{}{}", r_paren, space)
                } else if last {
                    "".to_string()
                } else if !(is_r_paren || is_op) {
                    ", ".to_string()
                } else if is_op && !is_pow {
                    " ".to_string()
                } else {
                    "".to_string()
                };

                implicit_paren = 0;

                format!("{}{}", colored, appendix)
            }
            Token::Operator { kind } => {
                let op = Operator::by_type(kind);

                match op.associativity {
                    Associativity::Left => format!("{} ", colored),
                    Associativity::Right => {
                        let is_l_paren = matches!(
                            tokens.get(idx + 1),
                            Some(Token::Paren {
                                kind: ParenType::Left
                            })
                        );

                        if op.implicit_paren() && !is_l_paren {
                            implicit_paren += 1;
                            let l_paren: T = colorize("(", &Token::Paren{kind:ParenType::Left});
                            format!("{}{}", colored, l_paren)
                        } else {
                            format!("{}", colored)
                        }
                    }
                }
            }
            Token::Paren { kind } => {
                if kind == ParenType::Left {
                    // Subtracts one bottoming out at 0 because `implicit_paren` is a `usize`
                    implicit_paren = implicit_paren.saturating_sub(1);
                }
                format!("{}", colored)
            }
        };
        out.push_str(&append)
    }
    out
}

#[cfg(test)]
mod tests {

    #![allow(
        clippy::float_cmp,
        clippy::non_ascii_literal,
        clippy::clippy::too_many_lines
    )]

    use crate::{
        doeval, stringify, tokens::ConstantType, tokens::OperatorType, tokens::ParenType, Token,
    };

    #[test]
    fn test() {
        [
            (
                "1 + 1",
                "1 + 1",
                2.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
            (
                "sin pi",
                "sin(π)",
                std::f64::consts::PI.sin(),
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Constant {
                        kind: ConstantType::PI,
                    },
                ],
            ),
            (
                "1 plus 7 sub 2 times 3",
                "1 + 7 - 2 × 3",
                2.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 7.0 },
                    Token::Operator {
                        kind: OperatorType::Sub,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Mul,
                    },
                    Token::Number { value: 3.0 },
                ],
            ),
            (
                "sin(1 + 2 + 3)",
                "sin(1 + 2 + 3)",
                ((1.0 + 2.0 + 3.0) as f64).sin(),
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Number { value: 3.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "345.67",
                "345.67",
                345.67,
                vec![Token::Number { value: 345.67 }],
            ),
            (
                "sin 66 pow 2 plus cos(66)^2",
                "sin(66)^2 + cos(66)^2",
                1.0,
                vec![
                    Token::Operator {
                        kind: OperatorType::Sin,
                    },
                    Token::Number { value: 66.0 },
                    Token::Operator {
                        kind: OperatorType::Pow,
                    },
                    Token::Number { value: 2.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Operator {
                        kind: OperatorType::Cos,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 66.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                    Token::Operator {
                        kind: OperatorType::Pow,
                    },
                    Token::Number { value: 2.0 },
                ],
            ),
            (
                "(1)",
                "(1)",
                1.0,
                vec![
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "((1))",
                "((1))",
                1.0,
                vec![
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Number { value: 1.0 },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                    Token::Paren {
                        kind: ParenType::Right,
                    },
                ],
            ),
            (
                "-1",
                "-1",
                -1.0,
                vec![
                    Token::Operator {
                        kind: OperatorType::Negative,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
            (
                "1 + -1",
                "1 + -1",
                0.0,
                vec![
                    Token::Number { value: 1.0 },
                    Token::Operator {
                        kind: OperatorType::Add,
                    },
                    Token::Operator {
                        kind: OperatorType::Negative,
                    },
                    Token::Number { value: 1.0 },
                ],
            ),
        ]
        .iter()
        .for_each(|(a, b, c, d)| {
            let (result, tokens) = match doeval(a) {
                Ok((x, y)) => (x, y),
                Err(e) => panic!("FAILED! {:?}", e),
            };
            assert_eq!(tokens, *d, "Checking tokenization of [{}]", a);
            assert_eq!(result, *c, "Checking evaluation of [{}]", a);
            assert_eq!(stringify(&tokens, |a, _| a.to_string()), *b);
        });
    }
}
