#[derive(Clone, PartialEq)]
pub enum Associativity {
    LEFT,
    RIGHT
}

#[derive(Clone)]
pub struct Operator {
    pub repr: &'static [&'static str],
    pub precedence: i8,
    pub associativity: Associativity,
    pub arity: i8,
    pub doit: fn(&Vec<f64>) -> f64
}

fn factorial(x: f64) -> f64 {
    if  x > 1.0 {
        return x * factorial(x - 1.0);
    }
    x
}

static OPERATORS: &[Operator] = &[
    Operator { repr: &["+", "add", "plus"], precedence: 1, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] + arr[1] },
    Operator { repr: &["-", "sub", "minus"], precedence: 1, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] - arr[1] },
    Operator { repr: &["×", "*", "times", "⋅", "mul"], precedence: 2, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] * arr[1] },
    Operator { repr: &["÷", "/", "over", "divide", "div"], precedence: 2, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] / arr[1] },
    Operator { repr: &["^", "exp", "pow"], precedence: 3, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].powf(arr[1]) },
    Operator { repr: &["%", "mod"], precedence: 3, associativity: Associativity::LEFT, arity: 2, doit: |arr| arr[0] % arr[1] },
    Operator { repr: &["sin"], precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].sin() },
    Operator { repr: &["cos"], precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].cos() },
    Operator { repr: &["tan"], precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].tan() },
    Operator { repr: &["max"], precedence: 4, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].max(arr[1]) },
    Operator { repr: &["min"], precedence: 4, associativity: Associativity::RIGHT, arity: 2, doit: |arr| arr[0].min(arr[1]) },
    Operator { repr: &["√", "sqrt", "root"], precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| arr[0].sqrt() },
    Operator { repr: &["!"], precedence: 4, associativity: Associativity::RIGHT, arity: 1, doit: |arr| factorial(arr[0]) },
];

impl Operator {
    pub fn which_operator(s: &str) -> Option<(&'static Operator, &&'static str)> {
        OPERATORS.iter().find_map(|c|
            c.repr.iter().find(|r| {
                s.to_lowercase().starts_with(&r.to_lowercase().to_string())
            }).and_then(|ss| Option::Some((c, ss)))
        )
    }
    pub fn is_operator(s: &str) -> bool {
        Operator::which_operator(s).is_some()
    }
}
pub struct Constant {
    pub repr: &'static [&'static str],
    pub value: f64
}

static CONSTANTS: &[Constant] = &[
    Constant { repr: &["π", "pi"], value: std::f64::consts::PI },
    Constant { repr: &["τ", "tau"], value: std::f64::consts::TAU },
    Constant { repr: &["e"], value: std::f64::consts::E }
];

pub fn get_constant(s: &str) -> Option<(&'static Constant, &&'static str)> {
    CONSTANTS.iter().find_map(|c|
        c.repr.iter().find(|r| {
            s.to_lowercase().starts_with(&r.to_lowercase().to_string())
        }).and_then(|s| Option::Some((c, s)))
    )
}
