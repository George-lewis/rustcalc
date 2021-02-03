#[derive(Clone)]
pub enum Associativity {
    LEFT,
    RIGHT
}

#[derive(Clone)]
pub struct Operator {
    pub repr: &'static str,
    pub precedence: i8,
    pub associativity: Associativity,
    doit: fn(&[f64]) -> f64
}

static OPERATORS: [Operator; 9] = [
    Operator { repr: "+", precedence: 1, associativity: Associativity::LEFT, doit: |arr| arr[0] + arr[1] },
    Operator { repr: "-", precedence: 1, associativity: Associativity::LEFT, doit: |arr| arr[0] - arr[1] },
    Operator { repr: "*", precedence: 2, associativity: Associativity::LEFT, doit: |arr| arr[0] * arr[1] },
    Operator { repr: "/", precedence: 2, associativity: Associativity::LEFT, doit: |arr| arr[0] / arr[1] },
    Operator { repr: "^", precedence: 3, associativity: Associativity::RIGHT, doit: |arr| arr[0].powf(arr[1]) },
    Operator { repr: "%", precedence: 3, associativity: Associativity::LEFT, doit: |arr| arr[0] % arr[1] },
    Operator { repr: "sin", precedence: 4, associativity: Associativity::RIGHT, doit: |arr| arr[0].sin() },
    Operator { repr: "cos", precedence: 4, associativity: Associativity::RIGHT, doit: |arr| arr[0].cos() },
    Operator { repr: "tan", precedence: 4, associativity: Associativity::RIGHT, doit: |arr| arr[0].tan() },
];

impl Operator {
    pub fn from_repr(s: &str) -> &Operator {
        OPERATORS.iter().find(|op| op.repr == s).unwrap()
    }
}