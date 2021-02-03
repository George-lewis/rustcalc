use std::io::Write;
use std::collections::HashMap;

mod operators;

static NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
static OPERATOR_CHARACTERS: [char; 5] = ['+', '-', '*', '/', '^'];
static PAREN_CHARACTERS: [char; 2] = ['(', ')'];



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

    }

}

fn next_num(string: &str) -> String {
    string.chars().take_while(|c| NUMBER_CHARACTERS.contains(c)).collect::<String>()
}

fn tokenize(string: &str) -> Vec<Token> {
    // let mut out: Vec<Token> = Vec::new();
    // for (mut idx, c) in string.chars().enumerate() {
        if NUMBER_CHARACTERS.contains(&c) {
            let num = next_num(&string[idx..]);
            idx += num.len();
            out.push(Token { value: num, kind: TokenType::NUMBER });
        } else if OPERATOR_CHARACTERS.contains(&c) {
            out.push(Token { value: c.to_string(), kind: TokenType::OPERATOR});
        } else if PAREN_CHARACTERS.contains(&c) {
            out.push(Token { value: c.to_string(), kind: TokenType::PAREN });
        } else {
            panic!("UNEXPECTED TOKEN");
        }
    }
    // out
}

fn solve(string: String) -> f64 {

    let mut input = tokenize(&string);

    let mut operator_stack: Vec<String> = Vec::new();
    let mut output: Vec<f64> = Vec::new();

    while input.len() > 0 {
        let token = input.pop().unwrap();
        match token.kind {
            TokenType::NUMBER => {
                output.push(token);
            },
            TokenType::OPERATOR => {

            },
            TokenType::PAREN => {
                match token.value.as_ref() {
                    "(" => operator_stack.push(token),
                    ")" => {

                    }
                    _ => panic!("???")
                }
            }
        }
    }

    0.0

}