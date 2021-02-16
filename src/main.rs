use core::panic;
use std::{
    cmp::min,
    fmt::{Debug, Display},
    io::Write,
    process::exit,
};

use tokens::*;

// #[cfg(feature = "color")]
use colored::*;

mod tokens;
mod utils;

static NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
static PAREN_CHARACTERS: [char; 2] = ['(', ')'];

#[derive(Debug)]
enum RMEError {
    ParseError(usize),
    OperandError(OperatorType),
    EmptyStack,
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

        fn clean(s: &mut String) {
            if s.ends_with('\n') {
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
            }
        }

        clean(&mut input);

        if input.is_empty() {
            continue;
        }

        if input.to_lowercase() == "quit" {
            exit(0);
        }

        // print!("Tokenized: {:?}", tokenize(&input));

        // if input.contains("=") {
        //     // println!("splitting");
        //     // let (left, right) = input.split_at(input.find("=").unwrap());
        //     let mut split = input.split("=");
        //     let left =  split.next().unwrap().trim();
        //     let right = split.next().unwrap().trim();
        //     // println!("[{}] - [{}]", left, right);
        //     let lr = eval(&left.to_string());
        //     let rr = eval(&right.to_string());
        //     println!("[{}] = [{}] => [{:.5}] = [{:.5}]; equal: {}", left, right.to_string(), lr, rr, (lr - rr).abs() < 0.01);
        // } else {

        let (x, repr) = match doeval(&input) {
            Ok((a, b)) => (a, b),
            Err(e) => {
                match e {
                    RMEError::ParseError(idx) => {
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
                            "Couldn't evaluate. Operator [{}] requires an operand",
                            format!("{:?}", kind).green()
                        );
                    }
                    RMEError::EmptyStack => {
                        // println!("Unknown problem causing the s")
                    }
                }

                continue;
            }
        };

        let formatted = stringify_color(&repr, color_cli);

        println!("[ {} ] => {}", formatted, format!("{:.3}", x).blue());
    }
}

fn next_num(string: &str) -> String {
    string
        .chars()
        .take_while(|c| NUMBER_CHARACTERS.contains(c))
        .collect::<String>()
}

fn tokenize(string: &str) -> Result<Vec<Token>, RMEError> {
    let mut vec: Vec<Token> = Vec::new();
    let mut idx = 0;
    let mut coeff = false;
    let mut implicit_paren = 0;
    while idx < string.chars().count() {
        let mut just = false;
        let c = string.chars().nth(idx).unwrap();
        if c.is_whitespace() || c == ',' {
            idx += 1;
            coeff = coeff && c != ',';
            continue;
        }
        let slice = utils::slice(string, idx, -0);
        if coeff {
            if c != ')' {
                let opt = Operator::by_repr(&slice);
                if opt.map_or(true, |(op, _)| {
                    op.associativity != Associativity::Left && op.kind != OperatorType::Pow
                }) {
                    vec.push(Token::Operator {
                        kind: OperatorType::Mul,
                    });
                }
            }
            coeff = false;
        }
        // println!("[{}] is A [{:?}]", &slice, 0);
        let kind = match _type(&slice) {
            Ok(k) => k,
            Err(_) => {
                return Err(RMEError::ParseError(idx));
            }
        };
        if c == '(' {
            implicit_paren = min(0, implicit_paren - 1);
        } else if implicit_paren > 0 {
            vec.push(Token::Paren {
                kind: ParenType::Left,
            });
        }
        match kind {
            TokenType::OPERATOR => {
                let (op, s) = Operator::by_repr(&slice).expect("Not an operator");

                if op.associativity == Associativity::Right && op.kind != OperatorType::Pow {
                    implicit_paren += 1;
                    just = true;
                }

                idx += s.chars().count();
                vec.push(Token::Operator { kind: op.kind });
            }
            TokenType::PAREN => {
                let (t, kind) = Token::paren(c);
                coeff = kind == ParenType::Right;
                // if kind == ParenType::Left {
                //     implicit_paren -= 1;
                // }
                vec.push(t);
                idx += 1;
            }
            TokenType::NUMBER => {
                // println!("SLICE FROM {} to {}", idx, string.chars().count() - idx);
                let num = next_num(&utils::slice(string, idx, -0));
                idx += num.chars().count();
                vec.push(Token::Number {
                    value: num.parse().expect("NOT PARSABLE AS A FLOAT"),
                });
                coeff = true;
            }
            TokenType::CONSTANT => {
                // println!("SLICED; {}", utils::slice(string,idx, string.chars().count() - idx));
                let (constant, s) = Constant::by_repr(&slice).unwrap();
                idx += s.chars().count();
                vec.push(Token::Constant {
                    kind: constant.kind,
                });
                coeff = true;
            }
        }
        if !just {
            for _ in 0..implicit_paren {
                vec.push(Token::Paren {
                    kind: ParenType::Right,
                });
            }
            implicit_paren = 0;
        }
    }
    // println!("tokens {:?}", vec.iter().map(|t| &t.value).cloned().collect::<Vec<String>>());
    Ok(vec)
}

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

fn rpn(tokens: Vec<Token>) -> Vec<Token> {
    // println!("Tokens: {:?}", tokens);

    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::new();

    for token in &tokens {
        match token {
            Token::Number { .. } | Token::Constant { .. } => output.push(*token),
            Token::Operator { kind } => {
                let op1 = Operator::by_type(*kind);
                while operator_stack.len() > 0 {
                    let last = operator_stack.last().expect("Empty op stack?");
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
                        if operator_stack.len() > 0 {
                            let op = operator_stack.pop().unwrap();
                            if let Token::Paren { kind } = op {
                                if kind == ParenType::Left {
                                    break;
                                }
                            }
                            output.push(op);
                        } else {
                            panic!("Mismatched parens!");
                        }
                    }
                    if matches!(operator_stack.last(), Some(Token::Operator { .. })) {
                        output.push(operator_stack.pop().unwrap());
                    }
                }
            },
        }
    }

    while operator_stack.len() > 0 {
        output.push(operator_stack.pop().unwrap());
    }

    // println!("{:?}", output);

    output
}

// fn process_unary(tokens: &Vec<Token>) -> Vec<Token> {
//     let mut out: Vec<Token> = tokens.clone();
//     for (idx, token) in out.iter_mut().enumerate() {
//         let op1 = Operator::which_operator(&token.value);
//         let tok = tokens.iter().nth(idx + 1);
//         if op1.is_some() && tok.is_some() {
//             if let Some(op2) = tok.map(|t| Operator::which_operator(&t.value)) {
//                 let repr1 = op1.unwrap().0.repr[0];
//                 let repr2 = op2.unwrap().0.repr[0];
//                 if repr1 == repr2 == "+" {

//                 } else if repr1 == repr2 == "-" {

//                 }
//             }
//         }
//         if token.kind == TokenType::CONSTANT {
//             let (constant, _) = get_constant(token.value.as_str()).expect("umm.. not a valid constant");
//             *token = Token { value: constant.value.to_string(), kind: TokenType::NUMBER };
//         }
//     }
//     out
// }

fn eval(_k: Vec<Token>) -> Result<f64, RMEError> {
    let mut k: Vec<Token> = _k.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new();
    // println!("tokens : {:?}", k);
    while k.len() > 0 {
        let token = k.pop().unwrap();

        // println!("Cur: {:?}; args; {:?}; k; {:?}", token, args, k);

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
                let mut argg: Vec<f64> = Vec::new();
                for _ in 0..op.arity {
                    match args.pop() {
                        Some(e) => argg.push(e),
                        None => return Err(RMEError::OperandError(op.kind)),
                    };
                }
                let result = (op.doit)(&argg.iter().rev().cloned().collect());
                k.push(Token::Number { value: result });
            }
            Token::Paren { .. } => {}
        }
    }
    // println!("args; {:?}; k; {:?}", args, k);
    if k.len() == 0 {
        return Ok(args[0]);
    } else {
        return Ok(std::f64::NAN);
    }
}

fn doeval(string: &str) -> Result<(f64, Vec<Token>), RMEError> {
    let tokens = tokenize(&string)?;
    // println!("Tokens: {:?}", tokens);
    // let mut constants = tokens.clone();
    // process_constants(&mut constants);
    let rpn = rpn(tokens.clone());
    let result = eval(rpn)?;
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

fn color_html(string: &str, token: &Token) -> String {
    let code = match token {
        Token::Number { .. } => "red",
        Token::Operator { .. } => "blue",
        Token::Paren { .. } => "green",
        Token::Constant { .. } => "orange",
    };
    format!("<span style=\"color: {}\">{}</span>", code, string)
}

fn stringify(tokens: &Vec<Token>) -> String {
    stringify_color(tokens, |a, _| a.to_string())
}

fn stringify_color<F, T: Display>(tokens: &Vec<Token>, f: F) -> String
where
    F: Fn(&str, &Token) -> T,
{
    _stringify(tokens)
        .iter()
        .map(|(s, t, space, comma)| (space, comma, f(s, t)))
        .fold("".to_string(), |acc, (space, comma, x)| {
            format!(
                "{}{}{}{}",
                acc,
                x,
                if *comma { "," } else { "" },
                if *space { " " } else { "" }
            )
        })
}

fn _stringify(tokens: &Vec<Token>) -> Vec<(String, &Token, bool, bool)> {
    let mut out: Vec<(String, &Token, bool, bool)> = Vec::new();
    for (idx, token) in tokens.iter().enumerate() {
        let mut append = match *token {
            Token::Number { value } => {
                let is_r_paren_or_op = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    }) | Some(Token::Operator { .. })
                );
                vec![(value.to_string(), token, true, !is_r_paren_or_op)]
            }
            Token::Constant { kind } => {
                let repr = Constant::by_type(kind).repr.first().unwrap();
                let is_r_paren_or_op = matches!(
                    tokens.get(idx + 1),
                    Some(Token::Paren {
                        kind: ParenType::Right
                    }) | Some(Token::Operator { .. })
                );
                vec![(repr.to_string(), token, true, !is_r_paren_or_op)]
            }
            Token::Operator { kind } => {
                let op = Operator::by_type(kind);
                let repr = op.repr.first().unwrap().clone();
                match op.associativity {
                    Associativity::Left => vec![(repr.to_string(), token, true, false)],
                    Associativity::Right => {
                        let is_l_paren = matches!(
                            tokens.get(idx + 1),
                            Some(Token::Paren {
                                kind: ParenType::Left
                            })
                        );

                        if kind != OperatorType::Pow && !is_l_paren {
                            vec![
                                (repr.to_string(), token, false, false),
                                (
                                    "(".to_string(),
                                    &Token::Paren {
                                        kind: ParenType::Left,
                                    },
                                    false,
                                    false,
                                ),
                            ]
                        } else {
                            vec![(repr.to_string(), token, false, false)]
                        }
                    }
                }
            }
            Token::Paren { kind } => match kind {
                ParenType::Left => {
                    vec![("(".to_string(), token, false, false)]
                }
                ParenType::Right => {
                    vec![(")".to_string(), token, true, false)]
                }
            },
        };
        let is_l_paren_or_pow = matches!(
            tokens.get(idx + 1),
            Some(Token::Paren {
                kind: ParenType::Right
            }) | Some(Token::Operator {
                kind: OperatorType::Pow
            })
        );
        let mut last = append.iter_mut().last().unwrap();
        if last.2 && is_l_paren_or_pow {
            last.2 = false;
        }
        if idx == tokens.len() - 1 {
            last.3 = false;
        }
        out.extend_from_slice(&append);
    }
    out.iter_mut().last().unwrap().2 = false;
    out
}

#[cfg(test)]
mod tests {

    use crate::{
        doeval, stringify, tokens::ConstantType, tokens::OperatorType, tokens::ParenType, Token,
    };

    #[test]
    fn test_tokenize() {
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
                    Token::Paren {
                        kind: ParenType::Left,
                    },
                    Token::Constant {
                        kind: ConstantType::PI,
                    },
                    Token::Paren {
                        kind: ParenType::Right,
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
        ]
        .iter()
        .for_each(|(a, b, c, d)| {
            let (result, tokens) = match doeval(a) {
                Ok((x, y)) => (x, y),
                Err(e) => panic!("FAILED! {:?}", e),
            };
            assert_eq!(result, *c, "Checking evaluation of [{}]", a);
            assert_eq!(tokens, *d, "Checking tokenization of [{}]", a);
            assert_eq!(stringify(&tokens), *b);
        });
    }
}
