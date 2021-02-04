use core::panic;
use std::{cmp::min, convert::TryInto, f64, fmt::Display, io::Write, option};

use operators::{Associativity, Operator};

mod operators;
mod utils;

static NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
static PAREN_CHARACTERS: [char; 2] = ['(', ')'];

struct Constant {
    repr: &'static [&'static str],
    value: f64
}

static CONSTANTS: [Constant; 1] = [
    Constant { repr: &["pi", "π"], value: std::f64::consts::PI }
];

fn get_constant(s: &str) -> Option<(&'static Constant, usize)> {
    CONSTANTS.iter().find_map(|c|
        c.repr.iter().find(|r| {
            println!("COMPARE: [{}] == [{}] => {}", r, utils::slice(s,0,min(r.len()., s.len())),r.starts_with(&utils::slice(s,0,min(r.len(), s.len()))));
            r.starts_with(&utils::slice(s,0,min(r.len(), s.len())))
        }).and_then(|s| Option::Some((c, s.len())))
    )
}

#[derive(Clone, Debug)]
enum TokenType {
    NUMBER,
    OPERATOR,
    PAREN,
    CONSTANT
}

#[derive(Clone, Debug)]
struct Token {
    value: String,
    kind: TokenType
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.value)
    }
}

fn main() {

    let x = operators::Operator::from_repr("+");

    println!("{}", x.precedence);

    loop {

        print!("Please input a statement: ");
        std::io::stdout().flush().ok().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.is_empty() {
            continue;
        }

        fn clean(s: &mut String) {
            if ['\n', '\r'].contains(&s.chars().last().unwrap()) {
                s.pop();
                clean(s);
            }
        }

        clean(&mut input);

        let x = eval(input);

        println!("Result: {}", x);

    }

}

fn next_num(string: &str) -> String {
    string.chars().take_while(|c| NUMBER_CHARACTERS.contains(c)).collect::<String>()
}

fn tokenize(string: &str) -> Vec<Token> {
    let mut vec: Vec<Token> = Vec::new();
    let mut idx = 0;
    while idx < string.len() {
        let c = string.chars().nth(idx).unwrap();
        if c.is_whitespace() || c == ',' {
            idx += 1;
            continue;
        }
        match _type(&c) {
            TokenType::OPERATOR => {
                let op = Operator::which_operator(&utils::slice(string,idx, string.len() - idx)).expect("Not an operator").repr.to_string();
                idx += op.len();
                vec.push(Token { value: op, kind: TokenType::OPERATOR });
            },
            TokenType::PAREN => {
                vec.push(Token { value: c.to_string(), kind: TokenType::PAREN });
                idx += 1;
            },
            TokenType::NUMBER => {
                let num = next_num(&utils::slice(string,idx, string.len() - idx));
                idx += num.len();
                vec.push(Token { value: num, kind: TokenType::NUMBER });
            },
            TokenType::CONSTANT => {
                println!("SLICED; {}", utils::slice(string,idx, string.len() - idx));
                let (constant, len) = get_constant(&utils::slice(string,idx, string.len() - idx)).unwrap();
                idx += len;
                vec.push(Token { value: constant.value.to_string(), kind: TokenType::CONSTANT });
            }
        }
    }
    // println!("tokens {:?}", vec.iter().map(|t| &t.value).cloned().collect::<Vec<String>>());
    vec
}

fn _type(c: &char) -> TokenType {
    if NUMBER_CHARACTERS.contains(c) {
        TokenType::NUMBER
    } else if Operator::is_operator(&c.to_string()) {
        TokenType::OPERATOR
    } else if PAREN_CHARACTERS.contains(c) {
        TokenType::PAREN
    } else if get_constant(c.to_string().as_str()).is_some() {
        TokenType::CONSTANT
    } else {
        panic!("NOT A VALID TOKEN");
    }
}

fn solve(string: String) -> Vec<Token> {

    let tokens = tokenize(&string);

    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::new();

    for token in tokens {
        match token.kind {
            TokenType::NUMBER| TokenType::CONSTANT => {
                output.push(token);
            },
            TokenType::OPERATOR => {
                let op = Operator::which_operator(&token.value).expect("Not an operator");
                while operator_stack.len() > 0 && operator_stack.last().unwrap().value != "(" {
                    let op_ = Operator::which_operator(&operator_stack.last().unwrap().value).unwrap();
                    if op_.precedence > op.precedence || (op_.precedence == op.precedence && op.associativity == Associativity::LEFT) {
                        output.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operator_stack.push(Token { value: op.repr.to_string(), kind: TokenType::OPERATOR });
            },
            TokenType::PAREN => {
                match token.value.as_str() {
                    "(" => operator_stack.push(token),
                    ")" => {
                        loop {
                            if operator_stack.len() > 0 {
                                let op = operator_stack.pop().unwrap();
                                if op.value == "(" {
                                    break;
                                } else {
                                    output.push(op);
                                }
                            } else {
                                panic!("Mismatched parens!");
                            }
                        }
                         if operator_stack.last().map_or(false, |t| t.value.as_str() == "(") {
                             operator_stack.pop();
                         }
                    },
                    _ => panic!()
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

fn eval(string: String) -> f64 {
    let mut k: Vec<Token> = solve(string).iter().rev().cloned().collect();
    // println!("? {:?}", k);
    if k.len() == 1 {
        return k[0].value.parse::<f64>().unwrap();
    }
    let mut args: Vec<f64> = Vec::new(); 
    loop {
        let token = k.pop().unwrap();
        // println!("ARGS: {:?}; STACK: {:?}", args, k);
        match token.kind {
            TokenType::NUMBER | TokenType::CONSTANT => {
                args.push(token.value.parse::<f64>().unwrap());
            }
            TokenType::OPERATOR => {
                let op = Operator::which_operator(token.value.as_str()).unwrap();
                let mut argg: Vec<f64> = Vec::new();
                for _ in 0..op.arity {
                    argg.push(args.pop().unwrap());
                }
                let result = (op.doit)(&argg.iter().rev().cloned().collect());
                if k.len() == 0 {
                    return result
                }
                k.push(Token { value: result.to_string(), kind: TokenType::NUMBER });
                // args.clear();
            }
            TokenType::PAREN => {}
        }
    }
}