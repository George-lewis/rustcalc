#![allow(clippy::non_ascii_literal)]

use std::fmt;

use rand::Rng;

use super::representable::{get_by_repr, Representable};

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

/// Unary operators
const UNARY_OPERATORS: &[OperatorType] = &[OperatorType::Positive, OperatorType::Negative];
pub const FUNCTIONAL_STYLE_OPERATORS: &[OperatorType] = &[
    OperatorType::Sin,
    OperatorType::Cos,
    OperatorType::Tan,
    OperatorType::Max,
    OperatorType::Min,
    OperatorType::RandomFloat,
    OperatorType::RandomInt,
];

impl Representable for OperatorType {
    fn repr(&self) -> &'static [&'static str] {
        Operator::by_type(*self).repr
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
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

// NOTE: This assumes that Operator-OperatorType pairs are unique
impl PartialEq for Operator {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}]", self.kind)
    }
}

impl Representable for Operator {
    fn repr(&self) -> &[&str] {
        self.repr
    }
}

impl Operator {
    /// Get an `Operator` by its `OperatorType`
    pub fn by_type(kind: OperatorType) -> &'static Self {
        OPERATORS.iter().find(|op| op.kind == kind).unwrap()
    }

    /// get an `Operator` by one of its string representations
    pub fn by_repr<'a>(repr: &'a str) -> Option<(&'static Self, &'a str)> {
        get_by_repr(repr, OPERATORS)
    }

    /// Determines if the next sequence is an `Operator`
    pub fn is(repr: &str) -> bool {
        Self::by_repr(repr).is_some()
    }

    pub fn is_next(string: &str) -> bool {
        OPERATORS
            .iter()
            .any(|op| op.repr.iter().any(|s| s.starts_with(string)))
    }

    // Determines if the next sequence is a unary `Operator`
    pub fn unary(repr: &str) -> Option<(&OperatorType, &str)> {
        get_by_repr(repr, UNARY_OPERATORS)
    }

    #[inline]
    pub fn apply(&self, args: &[f64]) -> f64 {
        (self.doit)(args)
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn _factorial(x: f64) -> f64 {
    let mut out: f64 = 1.0;
    for i in 1..=(x as i64) {
        out *= i as f64;
    }
    out
}

/// Compute `x!`
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
        precedence: 4,
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
        repr: &["!"],
        precedence: 5,
        associativity: Associativity::Left,
        arity: 1,
        doit: |arr| factorial(arr[0]),
    },
    Operator {
        kind: OperatorType::RandomFloat,
        repr: &["randf", "randfloat"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| rand::thread_rng().gen_range(arr[0]..=arr[1]),
    },
    Operator {
        kind: OperatorType::RandomInt,
        repr: &["randi", "randint"],
        precedence: 4,
        associativity: Associativity::Right,
        arity: 2,
        doit: |arr| rand::thread_rng().gen_range((arr[0] as i64)..=(arr[1] as i64)) as f64,
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

#[cfg(test)]
mod tests {

    use super::{factorial, Operator, OperatorType};

    #[test]
    fn test_factorial_normal() {
        for &(a, b) in &[
            (1.0, 1.0),
            (0.0, 1.0),
            (2.0, 2.0),
            (3.0, 6.0),
            (5.0, 120.0),
            (10.0, 3_628_800.0),
        ] {
            let result = factorial(a);
            assert_same!(b, result);
        }
    }

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn test_factorial_infinite() {
        let result = factorial(1000.0);
        let cmp = result.is_infinite();
        assert!(cmp);
        let result = factorial(5050.5);
        let cmp = result.is_infinite();
        assert!(cmp);
    }

    #[test]
    fn test_factorial_negative() {
        let result = factorial(-1.0);
        assert_same!(result, 1.0);
    }

    #[test]
    fn test_by_type() {
        let cons = Operator::by_type(OperatorType::Add);
        assert!(cons.repr.contains(&"+"));
    }

    #[test]
    fn test_by_repr() {
        let cons = Operator::by_repr("pow").unwrap();
        assert_eq!(cons.0.kind, OperatorType::Pow);
        assert_eq!(cons.1, "pow");
    }

    #[test]
    fn test_is() {
        assert!(Operator::is("sin"));
        assert!(!Operator::is("qqq"));
    }
}
