use crate::doeval;

use super::{
    errors::Error,
    operators::{Associativity, Operator},
    representable::{get_by_repr, Searchable},
    variables::Variable,
    EvaluationContext,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Functions<'a> {
    Builtin(&'a Operator),
    User(&'a Function),
}

impl Functions<'_> {
    /// Apply this function over a set of arguments and return the result.
    /// This never fails for `Functions::Builtin`.
    ///
    /// ## Errors
    /// `Functions::User` produce errors in the same way as [doeval] can, as these are,
    /// in actuality, nested evaluation contexts
    pub fn apply(&self, args: &[f64], context: EvaluationContext) -> Result<f64, Error> {
        match self {
            Functions::Builtin(op) => Ok((op.doit)(args)),
            Functions::User(func) => {
                let vars: Vec<_> = func
                    .args
                    .iter()
                    .zip(args)
                    .map(|(name, value)| Variable {
                        repr: name.clone(),
                        value: *value,
                    })
                    .collect();
                let context = EvaluationContext {
                    vars: &vars,
                    funcs: context.funcs,
                    depth: context.depth + 1,
                };
                let result = doeval(&func.code, context);
                result.map(|(a, _)| a)
            }
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            Functions::Builtin(op) => op.arity,
            Functions::User(func) => func.args.len(),
        }
    }
    pub const fn precedence(&self) -> u8 {
        match self {
            Functions::Builtin(op) => op.precedence,
            Functions::User(_) => 4,
        }
    }
    pub const fn associativity(&self) -> Associativity {
        match self {
            Functions::Builtin(op) => op.associativity,
            Functions::User(_) => Associativity::Right,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub code: String,
}

impl Searchable for Function {
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, usize)> {
        // Case sensitive
        if search.starts_with(&self.name) {
            Some((self, self.name.chars().count()))
        } else {
            None
        }
    }
}

impl Function {
    pub fn is(text: &str) -> bool {
        text.starts_with('#')
    }
    pub fn next_function<'a>(text: &str, funcs: &'a [Self]) -> Option<(&'a Self, usize)> {
        get_by_repr(text, funcs)
    }
}
