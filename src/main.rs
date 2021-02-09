use core::panic;
use std::{f64, fmt::Display, io::Write};

use operators::{Associativity, Operator};

mod operators;
mod utils;

use operators::*;

static NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
static PAREN_CHARACTERS: [char; 2] = ['(', ')'];

#[derive(Clone, Debug, PartialEq)]
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

        if input.contains("=") {
            // println!("splitting");
            // let (left, right) = input.split_at(input.find("=").unwrap());
            let mut split = input.split("=");
            let left =  split.next().unwrap().trim();
            let right = split.next().unwrap().trim();
            // println!("[{}] - [{}]", left, right);
            let lr = eval(&left.to_string());
            let rr = eval(&right.to_string());
            println!("[{}] = [{}] => [{:.5}] = [{:.5}]; equal: {}", left, right.to_string(), lr, rr, (lr - rr).abs() < 0.01);
        } else {

        let x = eval(&input);

        println!("[{}] => {:.3}", input, x);

        }

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
        let slice = utils::slice(string,idx, string.chars().count());
        if coeff {
            // match _type(&c) {
            //     TokenType::NUMBER | TokenType::CONSTANT | TokenType::PAREN => {
                    if c != ')' {//&& Operator::which_operator(c.to_string().as_str()).cloned().map_or(0, |op| op.arity) < 2 {\
                        let opt = Operator::which_operator(&slice);
                        if opt.map_or(0, |(op, s)| op.arity) < 2 {
                            vec.push(Token { value: "*".to_string(), kind: TokenType::OPERATOR });
                        }
                    }
                // },
                // _ => {}
            
            coeff = false;
        }
        // println!("[{}] is A [{:?}]", &slice, 0);
        match _type(&slice) {
            TokenType::OPERATOR => {
                let (_, s) = Operator::which_operator(&slice).expect("Not an operator");

                idx += s.chars().count();
                vec.push(Token { value: s.to_string(), kind: TokenType::OPERATOR });
            },
            TokenType::PAREN => {
                vec.push(Token { value: c.to_string(), kind: TokenType::PAREN });
                idx += 1;
            },
            TokenType::NUMBER => {
                // println!("SLICE FROM {} to {}", idx, string.chars().count() - idx);
                let num = next_num(&utils::slice(string,idx, string.chars().count()));
                idx += num.chars().count();
                vec.push(Token { value: num, kind: TokenType::NUMBER });
                coeff = true;
            },
            TokenType::CONSTANT => {
                // println!("SLICED; {}", utils::slice(string,idx, string.chars().count() - idx));
                let (constant, s) = get_constant(&slice).unwrap();
                idx += s.chars().count();
                vec.push(Token { value: constant.value.to_string(), kind: TokenType::CONSTANT });
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
    } else if Operator::is_operator(s) {
        TokenType::OPERATOR
    } else if PAREN_CHARACTERS.contains(c) {
        TokenType::PAREN
    } else if get_constant(s).is_some() {
        println!("it's a constant");
        TokenType::CONSTANT
    } else {
        panic!("NOT A VALID TOKEN");
    }
}

fn solve(string: &String) -> Vec<Token> {

    let tokens = tokenize(&string);

    // println!("Tokens: {:?}", tokens);

    let mut operator_stack: Vec<Token> = Vec::new();
    let mut output: Vec<Token> = Vec::new();

    for token in tokens {
        
        match token.kind {
            TokenType::NUMBER | TokenType::CONSTANT => {
                output.push(token);
            },
            TokenType::OPERATOR => {
                let (op, s) = Operator::which_operator(&token.value).expect("Not an operator");
                while operator_stack.len() > 0 && operator_stack.last().unwrap().value != "(" {
                    let (op_, ss) = Operator::which_operator(&operator_stack.last().unwrap().value).unwrap();
                    if op_.precedence > op.precedence || (op_.precedence == op.precedence && op.associativity == Associativity::LEFT) {
                        output.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                operator_stack.push(Token { value: s.to_string(), kind: TokenType::OPERATOR });
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
                         } else if operator_stack.last().map_or(false, |t| t.kind == TokenType::OPERATOR) {
                            output.push(operator_stack.pop().unwrap());
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

fn eval(string: &String) -> f64 {
    let mut k: Vec<Token> = solve(string).iter().rev().cloned().collect();
    println!("? {:?}", k);
    if k.len() == 0 {
        return std::f64::NAN;
    }
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
                let (op, _) = Operator::which_operator(token.value.as_str()).unwrap();
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