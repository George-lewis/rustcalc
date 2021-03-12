#![allow(clippy::non_ascii_literal)]

use super::tokens::get_by_repr;
use super::tokens::Representable;

use rand::Rng;

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

const UNARY_OPERATORS: &[OperatorType] = &[OperatorType::Positive, OperatorType::Negative];

impl Representable for OperatorType {
    fn repr(&self) -> &'static [&'static str] {
        Operator::by_type(*self).repr
    }
}

#[derive(Clone, PartialEq, Copy)]
pub enum Associativity {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub struct Operator {
    pub kind: OperatorType,
    pub repr: &'static [&'static str],
    pub precedence: u8,
    pub associativity: Associativity,
    pub arity: usize,
    pub doit: fn(&[f64]) -> f64,
}

impl Representable for Operator {
    fn repr(&self) -> &[&str] {
        self.repr
    }
}

impl Operator {
    pub fn by_type(kind: OperatorType) -> &'static Self {
        OPERATORS.iter().find(|op| op.kind == kind).unwrap()
    }
    pub fn by_repr(repr: &str) -> Option<(&'static Self, &'static &'static str)> {
        get_by_repr(repr, OPERATORS)
    }
    pub fn is(repr: &str) -> bool {
        Self::by_repr(repr).is_some()
    }
    pub fn unary(repr: &str) -> Option<(&OperatorType, &&str)> {
        get_by_repr(repr, UNARY_OPERATORS)
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

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn _factorial(x: f64) -> f64 {
    let mut out: f64 = 1.0;
    for i in 1..=(x as i64) {
        out *= i as f64
    }
    out
}

fn factorial(x: f64) -> f64 {
    if x >= 1000.0 {
        f64::INFINITY
    } else {
        _factorial(x.floor())
    }
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
