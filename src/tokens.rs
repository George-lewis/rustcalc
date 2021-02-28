#![warn(clippy::pedantic)]
#![allow(clippy::non_ascii_literal, clippy::bind_instead_of_map)]

use core::panic;
use rand::Rng;

pub const NUMBER_CHARACTERS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.'];
pub const PAREN_CHARACTERS: [char; 2] = ['(', ')'];

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OperatorType {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Sin,
    Cos,
    Tan,
    Max,
    Min,
    Sqrt,
    Negative,
    Positive,
    Factorial,
    RandomInt,
    RandomFloat,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ParenType {
    Left,
    Right,
}
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ConstantType {
    PI,
    E,
    Tau,
}
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Token {
    Number { value: f64 },
    Operator { kind: OperatorType },
    Paren { kind: ParenType },
    Constant { kind: ConstantType },
}

impl Token {
    pub fn paren(c: char) -> (Token, ParenType) {
        match c {
            '(' => (
                Token::Paren {
                    kind: ParenType::Left,
                },
                ParenType::Left,
            ),
            ')' => (
                Token::Paren {
                    kind: ParenType::Right,
                },
                ParenType::Right,
            ),
            _ => panic!("[{}] IS NOT A PAREN", c),
        }
    }
    pub fn ideal_repr(&self) -> String {
        match self {
            Token::Number { value } => value.to_string(),
            Token::Operator { kind } => Operator::by_type(*kind).repr[0].to_string(),
            Token::Paren { kind } => match kind {
                ParenType::Left => "(".to_string(),
                ParenType::Right => ")".to_string(),
            },
            Token::Constant { kind } => Constant::by_type(*kind).repr[0].to_string(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

pub struct Operator {
    pub kind: OperatorType,
    pub repr: &'static [&'static str],
    pub precedence: i8,
    pub associativity: Associativity,
    pub arity: i8,
    pub doit: fn(&Vec<f64>) -> f64,
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn _factorial(x: f64) -> f64 {
    let mut out: f64 = 1.0;
    for i in 1..=(x as i64) {
        out *= i as f64
    }
    out
}

fn factorial(x: f64) -> f64 {
    _factorial(x.floor())
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
static OPERATORS: &[Operator] = &[
    Operator {
        kind: OperatorType::Add,
        repr: &["+", "add", "plus"],
        precedence: 1,
        associativity: Associativity::Left,
        arity: 2,
        doit: |arr| arr[0] + arr[1],
    },
    Operator {
        kind: OperatorType::Sub,
        repr: &["-", "subtract", "sub", "minus"],
        precedence: 1,
        associativity: Associativity::Left,
        arity: 2,
        doit: |arr| arr[0] - arr[1],
    },
    Operator {
        kind: OperatorType::Mul,
        repr: &["×", "*", "times", "⋅", "mul"],
        precedence: 2,
        associativity: Associativity::Left,
        arity: 2,
        doit: |arr| arr[0] * arr[1],
    },
    Operator {
        kind: OperatorType::Div,
        repr: &["÷", "/", "over", "divide", "div"],
        precedence: 2,
        associativity: Associativity::Left,
        arity: 2,
        doit: |arr| arr[0] / arr[1],
    },
    Operator {
        kind: OperatorType::Pow,
        repr: &["^", "exp", "pow"],
        precedence: 3,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| arr[0].powf(arr[1]),
    },
    Operator {
        kind: OperatorType::Mod,
        repr: &["%", "mod"],
        precedence: 3,
        associativity: Associativity::Left,
        arity: 2,
        doit: |arr| arr[0] % arr[1],
    },
    Operator {
        kind: OperatorType::Sin,
        repr: &["sin"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| arr[0].sin(),
    },
    Operator {
        kind: OperatorType::Cos,
        repr: &["cos"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| arr[0].cos(),
    },
    Operator {
        kind: OperatorType::Tan,
        repr: &["tan"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| arr[0].tan(),
    },
    Operator {
        kind: OperatorType::Max,
        repr: &["max"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| arr[0].max(arr[1]),
    },
    Operator {
        kind: OperatorType::Min,
        repr: &["min"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| arr[0].min(arr[1]),
    },
    Operator {
        kind: OperatorType::Sqrt,
        repr: &["√", "sqrt", "root"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| arr[0].sqrt(),
    },
    Operator {
        kind: OperatorType::Factorial,
        repr: &["!", "factorial", "fact"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| factorial(arr[0]),
    },
    Operator {
        kind: OperatorType::RandomFloat,
        repr: &["randf"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| rand::thread_rng().gen_range(arr[0]..arr[1]),
    },
    Operator {
        kind: OperatorType::RandomInt,
        repr: &["randint"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| rand::thread_rng().gen_range((arr[0] as i64)..(arr[1] as i64)) as f64,
    },
    Operator {
        kind: OperatorType::Negative,
        repr: &["-"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| -arr[0],
    },
    Operator {
        kind: OperatorType::Positive,
        repr: &["+"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 1,
        doit: |arr| arr[0],
    },
];

impl Operator {
    pub fn by_type(kind: OperatorType) -> &'static Operator {
        OPERATORS.iter().find(|op| op.kind == kind).unwrap()
    }
    pub fn by_repr(repr: &str) -> Option<(&'static Operator, &'static &'static str)> {
        OPERATORS.iter().find_map(|op| {
            op.repr
                .iter()
                .find(|str| repr.to_lowercase().starts_with(&str.to_lowercase()))
                .and_then(|sstr| Option::Some((op, sstr)))
        })
    }
    pub fn is(repr: &str) -> bool {
        Operator::by_repr(repr).is_some()
    }
    pub fn unary(s: &str) -> Option<(&OperatorType, &&str)> {
        [OperatorType::Positive, OperatorType::Negative]
            .iter()
            .map(|kind| (kind, Operator::by_type(*kind)))
            .find_map(|(kind, op)| {
                op.repr
                    .iter()
                    .find(|str| s.to_lowercase().starts_with(&str.to_lowercase()))
                    .and_then(|sstr| Option::Some((kind, sstr)))
            })
    }
    pub fn implicit_paren(&self) -> bool {
        ![
            OperatorType::Positive,
            OperatorType::Negative,
            OperatorType::Pow,
        ]
        .contains(&self.kind)
    }
}

pub struct Constant {
    pub kind: ConstantType,
    pub repr: &'static [&'static str],
    pub value: f64,
}

static CONSTANTS: &[Constant] = &[
    Constant {
        kind: ConstantType::PI,
        repr: &["π", "pi"],
        value: std::f64::consts::PI,
    },
    Constant {
        kind: ConstantType::Tau,
        repr: &["τ", "tau"],
        value: std::f64::consts::TAU,
    },
    Constant {
        kind: ConstantType::E,
        repr: &["e"],
        value: std::f64::consts::E,
    },
];

impl Constant {
    pub fn by_type(kind: ConstantType) -> &'static Constant {
        CONSTANTS.iter().find(|c| c.kind == kind).unwrap()
    }
    pub fn by_repr(repr: &str) -> Option<(&'static Constant, &'static &'static str)> {
        CONSTANTS.iter().find_map(|c| {
            c.repr
                .iter()
                .find(|str| repr.to_lowercase().starts_with(&str.to_lowercase()))
                .and_then(|sstr| Option::Some((c, sstr)))
        })
    }
    pub fn is(repr: &str) -> bool {
        Constant::by_repr(repr).is_some()
    }
}
