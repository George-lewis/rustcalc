use core::slice;
use std::cmp::{max, min};

use crate::utils;

#[derive(Clone, PartialEq)]
pub enum Associativity {
    LEFT,
    RIGHT
}

#[derive(Clone)]
pub struct Operator {
    pub repr: &'static str,
    pub precedence: i8,
    pub associativity: Associativity,
    pub arity: i8,
    pub doit: fn(&Vec<f64>) -> f64
}

static OPERATORS: [Operator; 11] = [
    Operator { repr: "+", precedence: 1, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] + arr[1] },
    Operator { repr: "-", precedence: 1, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] - arr[1] },
    Operator { repr: "*", precedence: 2, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] * arr[1] },
    Operator { repr: "/", precedence: 2, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] / arr[1] },
    Operator { repr: "^", precedence: 3, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].powf(arr[1]) },
    Operator { repr: "%", precedence: 3, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] % arr[1] },
    Operator { repr: "sin", precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].sin() },
    Operator { repr: "cos", precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].cos() },
    Operator { repr: "tan", precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].tan() },
    Operator { repr: "max", precedence: 4, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].max(arr[1]) },
    Operator { repr: "min", precedence: 4, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].max(arr[1]) },
];

impl Operator {
    pub fn from_repr(s: &str) -> &Operator {
        OPERATORS.iter().find(|op| op.repr == s).unwrap()
    }
    pub fn which_operator(s: &str) -> Option<&Operator> {
        OPERATORS.iter().find(|op| op.repr.starts_with(&utils::slice(s,0,min(s.len(), op.repr.len()))))
    }
    pub fn is_operator(s: &str) -> bool {
        Operator::which_operator(s).is_some()
    }
}