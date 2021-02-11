use core::panic;
use std::{io::Write};

// use operators::{Associativity, Operator};

use tokens::*;

// mod operators;
mod utils;
mod tokens;

// use operators::*;

static NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
static PAREN_CHARACTERS: [char; 2] = ['(', ')'];

#[derive(Clone, Debug, PartialEq)]
enum TokenType {
    NUMBER,
    OPERATOR,
    PAREN,
    CONSTANT
}

// #[derive(Clone, Debug)]
// struct Token {
//     value: String,
//     kind: TokenType
// }

// impl Display for Token {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "({})", self.value)
//     }
// }

fn main() {

    // let q = TToken::Paren { kind: ParenType::Left };

    loop {

        print!("Please input a statement: ");
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

        let (x, repr) = doeval(input);

        let formatted = stringify(&repr);

        println!("[{}] => {:.3}", formatted, x);

        // }

    }

}

fn next_num(string: &str) -> String {
    string.chars().take_while(|c| NUMBER_CHARACTERS.contains(c)).collect::<String>()
}

fn tokenize(string: &str) -> Vec<Token> {
    let mut vec: Vec<Token> = Vec::new();
    let mut idx = 0;

    let mut coeff = false;
    while idx < string.chars().count() {
        let c = string.chars().nth(idx).unwrap();
        if c.is_whitespace() || c == ',' {
            idx += 1;
            coeff = coeff && c != ',';
            continue;
        }
        let slice = utils::slice(string,idx, -0);
        if coeff {
            // match _type(&c) {
            //     TokenType::NUMBER | TokenType::CONSTANT | TokenType::PAREN => {
                    if c != ')' {//&& Operator::which_operator(c.to_string().as_str()).cloned().map_or(0, |op| op.arity) < 2 {\
                        let opt = Operator::by_repr(&slice);
                        if opt.map_or(0, |(op, _)| op.arity) < 2 {
                            vec.push(Token::Operator { kind: OperatorType::Mul });
                        }
                    }
                // },
                // _ => {}
            
            coeff = false;
        }
        // println!("[{}] is A [{:?}]", &slice, 0);
        match _type(&slice) {
            TokenType::OPERATOR => {
                let (op, s) = Operator::by_repr(&slice).expect("Not an operator");

                idx += s.chars().count();
                vec.push(Token::Operator { kind: op.kind });
            },
            TokenType::PAREN => {
                vec.push(Token::paren(c));
                idx += 1;
            },
            TokenType::NUMBER => {
                // println!("SLICE FROM {} to {}", idx, string.chars().count() - idx);
                let num = next_num(&utils::slice(string,idx, -0));
                idx += num.chars().count();
                vec.push(Token::Number { value: num.parse().expect("NOT PARSABLE AS A FLOAT") });
                coeff = true;
            },
            TokenType::CONSTANT => {
                // println!("SLICED; {}", utils::slice(string,idx, string.chars().count() - idx));
                let (constant, s) = Constant::by_repr(&slice).unwrap();
                idx += s.chars().count();
                vec.push(Token::Constant { kind: constant.kind });
                coeff = true;
            }
        }
    }
    // println!("tokens {:?}", vec.iter().map(|t| &t.value).cloned().collect::<Vec<String>>());
    vec
}

fn _type(s: &str) -> TokenType {
    let c = &s.chars().nth(0).unwrap();
    if NUMBER_CHARACTERS.contains(c) {
        TokenType::NUMBER
    } else if Operator::is(s) {
        TokenType::OPERATOR
    } else if PAREN_CHARACTERS.contains(c) {
        TokenType::PAREN
    } else if Constant::is(s) {
        TokenType::CONSTANT
    } else {
        panic!("NOT A VALID TOKEN");
    }
}

fn rpn(tokens: Vec<Token>) -> Vec<Token> {

    // println!("Tokens: {:?}", tokens);

    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::new();

    for token in &tokens {
        
        match token {
            Token::Number{..} | Token::Constant{..} => output.push(*token),
            Token::Operator{ kind } => {
                let op1 = Operator::by_type(*kind);
                let is_l_paren = matches!(tokens.last(), Some(Token::Paren { kind: ParenType::Left }));
                while operator_stack.len() > 0 && !is_l_paren {
                    let last = operator_stack.last().expect("Empty op stack?");
                    let opt = if let Token::Operator{ kind } = last {
                        Option::Some(Operator::by_type(*kind))
                    } else {
                        Option::None
                    };
                    let op2 = opt.unwrap();
                    if op2.precedence > op1.precedence || (op2.precedence == op1.precedence && op1.associativity == Associativity::Left) {
                        output.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operator_stack.push(*token);
            },
            Token::Paren{ kind } => {
                match kind {
                    ParenType::Left => operator_stack.push(*token),
                    ParenType::Right => {
                        loop {
                            if operator_stack.len() > 0 {
                                let op = operator_stack.pop().unwrap();
                                if let Token::Paren{ kind } = op {
                                    if kind == ParenType::Left {
                                        break;
                                    }
                                }
                                output.push(op);
                            } else {
                                panic!("Mismatched parens!");
                            }
                        }
                        if let Some(op) = operator_stack.last() {
                            if let Token::Paren{ kind } = op {
                                if kind == &ParenType::Left {
                                    operator_stack.pop();
                                }
                            } else if let Token::Operator{..} = op {
                                output.push(operator_stack.pop().unwrap());
                            }
                        };
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

// fn process_constants(tokens: &mut Vec<Token>) {
//     for token in tokens.iter_mut() {
//         if let Token::Constant{ kind } = token {
//             let constant = Constant::by_type(*kind);
//             *token = Token::Number { value: constant.value };
//         }
//     }
// }

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

fn eval(_k: Vec<Token>) -> f64 {
    let mut k: Vec<Token> = _k.iter().rev().cloned().collect();
    let mut args: Vec<f64> = Vec::new(); 
    println!("tokens : {:?}", k);
    while k.len() > 0 {
        let token = k.pop().unwrap();

        match token {
            Token::Number{ value } => {
                args.push(value);
            },
            Token::Constant{ kind } => {
                let constant = Constant::by_type(kind);
                args.push(constant.value);
            },
            Token::Operator{ kind } => {
                let op = Operator::by_type(kind);
                let mut argg: Vec<f64> = Vec::new();
                for _ in 0..op.arity {
                    argg.push(args.pop().unwrap());
                }
                let result = (op.doit)(&argg.iter().rev().cloned().collect());
                k.push(Token::Number { value: result });
            }
            Token::Paren{..} => {}
        }
    }
    if k.len() == 0 {
        return args[0];
    } else {
        return std::f64::NAN;
    }
}

fn doeval(string: String) -> (f64, Vec<Token>) {
    let tokens = tokenize(&string);
    // let mut constants = tokens.clone();
    // process_constants(&mut constants);
    let rpn = rpn(tokens.clone());
    let result = eval(rpn);
    (result, tokens)
}

fn stringify(tokens: &Vec<Token>) -> String {
    let mut out = String::new();
    let mut implicit_paren = 0;
    for (idx, token) in tokens.iter().enumerate() {
        let (append, just) = match *token {
            Token::Number{value} => {
                if implicit_paren > 0 {
                    (format!("{}{} ", value, ")".repeat(implicit_paren)), false)
                } else {
                    (format!("{} ", value), false)
                }
            },
            Token::Constant{ kind } => {
                let constant = Constant::by_type(kind);
                let repr = constant.repr.first().unwrap();
                if implicit_paren > 0 {
                    (format!("{}{} ", repr, ")".repeat(implicit_paren)), false)
                } else {
                    (format!("{} ", repr), false)
                }
            },
            Token::Operator{ kind } => {
                let op = Operator::by_type(kind);
                let repr = op.repr.first().unwrap().clone();
                match op.associativity {
                    Associativity::Left => (format!("{} ", repr), false),
                    Associativity::Right => {
                        let is_l_paren = matches!(tokens.get(idx + 1), Some(Token::Paren { kind: ParenType::Left }));
                        if kind != OperatorType::Pow && !is_l_paren {
                            implicit_paren += 1;
                            (format!("{}(", repr.to_owned()), true)
                        } else {
                            (format!("{}", repr.to_owned()), false)
                        }
                    }
                }
            }
            Token::Paren{ kind } => {
                (match kind {
                    ParenType::Left => "(".to_owned(),
                    ParenType::Right => ") ".to_owned()
                }, false)
            }
        };
        if !just {
            implicit_paren = 0;
        }
        let is_l_paren_or_pow = matches!(tokens.get(idx + 1), Some(Token::Paren { kind: ParenType::Right }) | Some(Token::Operator { kind: OperatorType::Pow }));
        if append.chars().last().unwrap() == ' ' && is_l_paren_or_pow {
            out.push_str(&utils::slice(&append, 0, -1));
        } else {
            out.push_str(&append);
        }
    }
    if out.chars().last().unwrap() == ' ' {
        return utils::slice(out.as_str(), 0,-1);
    }
    out
}