use std::{cell::Cell, rc::Rc};

use crate::{doeval, DoEvalResult};

use super::{
    errors::ErrorContext,
    operators::{Associativity, Operator},
    representable::{get_by_repr, Searchable},
    variables::Variable,
    EvaluationContext,
};

pub const PREFIX: char = '#';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Functions<'func> {
    Builtin(&'static Operator),
    User(&'func Function),
}

impl<'inner> Functions<'inner> {
    /// Apply this function over a set of arguments and return the result.
    /// This never fails for `Functions::Builtin`.
    ///
    /// ## Errors
    /// `Functions::User` produce errors in the same way as [doeval] can, as these are,
    /// in actuality, nested evaluation contexts
    pub fn arity(&self) -> usize {
        match self {
            Functions::Builtin(op) => op.arity,
            Functions::User(func) => func.arity(),
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

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub code: String,
}

impl Searchable for Function {
    fn search<'a, 'b>(&'a self, search: &'b str) -> Option<(&'a Self, &'b str)> {
        // Case sensitive
        if search.starts_with(&self.name) {
            Some((self, &search[..self.name.chars().count()]))
        } else {
            None
        }
    }
}

impl Function {
    pub fn is(text: &str) -> bool {
        text.starts_with(PREFIX)
    }
    pub fn next_function<'a, 'b>(text: &'b str, funcs: &'a [Self]) -> Option<(&'a Self, &'b str)> {
        get_by_repr(text, funcs)
    }
    pub fn arity(&self) -> usize {
        self.args.len()
    }

    /// Creates the argument variables for calling this function
    pub fn create_arguments(&self, args: &[f64]) -> Vec<Rc<Variable>> {
        // Create the arguments for the function
        let args = self
            .args
            .iter()
            .zip(args)
            .map(|(name, value)| Variable {
                repr: name.to_string(),
                value: Cell::new(*value),
            })
            .map(Rc::new);

        args.collect()
    }

    /// Apply this function to a set of arguments
    ///
    /// # Errors
    /// This function calls into `lib::doeval` and bubbles up and errors occuring from within there.
    pub fn apply<'str, 'funcs>(
        &'funcs self,
        args: &[f64],
        context: &EvaluationContext<'str, 'funcs>,
    ) -> DoEvalResult<'funcs, 'funcs> {
        let mut vars = self.create_arguments(args);
        vars.extend(context.vars.iter().map(Rc::clone));

        let context = EvaluationContext {
            vars: &vars,
            funcs: context.funcs,
            depth: context.depth + 1,
            context: ErrorContext::Scoped(self),
        };

        doeval(&self.code, context)
    }
}
