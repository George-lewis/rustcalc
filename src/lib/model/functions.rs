use std::borrow::Cow;

use crate::{DoEvalResult, doeval};

use super::{
    errors::{ContextualError, ErrorContext},
    operators::{Associativity, Operator},
    representable::{get_by_repr, Searchable},
    variables::Variable,
    EvaluationContext,
};

pub const PREFIX: char = '#';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Functions<'a> {
    Builtin(&'a Operator),
    User(&'a Function),
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
    fn search<'a>(&'a self, search: &str) -> Option<(&'a Self, &'a str)> {
        // Case sensitive
        if search.starts_with(&self.name) {
            Some((self, &self.name))
        } else {
            None
        }
    }
}

impl Function {
    pub fn is(text: &str) -> bool {
        text.starts_with(PREFIX)
    }
    pub fn next_function<'a>(text: &str, funcs: &'a [Self]) -> Option<(&'a Self, &'a str)> {
        get_by_repr(text, funcs)
    }
    pub fn arity(&self) -> usize {
        self.args.len()
    }

    /// Create the variables required to evaluate this function, including both arguments and scoped variables.
    /// The list is created such that arguments always come before scoped variables. This is important for correct varible-name resolution.
    pub fn create_variables(&self, args: &[f64], vars: &[Variable]) -> Vec<Variable> {
        // Create the arguments for the function
        let args = self.args.iter().zip(args).map(|(name, value)| Variable {
            repr: name.clone(),
            value: *value,
        });

        // Create a cloned iteration of the scoped variables
        let scoped = vars.iter().cloned();

        // Merge the function arguments with scoped variables
        // It's important that the function variables are first
        // So that variable name resolution prioritizes args
        args.chain(scoped).collect()
    }

    /// Apply this function to a set of arguments
    ///
    /// # Errors
    /// This function calls into `lib::doeval` and bubbles up and errors occuring from within there.
    pub fn apply<'var, 'func>(
        &'func self,
        args: &[f64],
        context: &EvaluationContext<'var, 'func>,
    ) -> DoEvalResult<'_, 'func> {
        let vars = self.create_variables(args, context.vars);

        // Return
        //

        // vars: &'_
        // funcs: &'func
        // context: &'func self
        let context: EvaluationContext<'_, 'func> = EvaluationContext {
            vars: &vars,
            funcs: context.funcs,
            depth: context.depth + 1,
            context: ErrorContext::Scoped(self),
        };

        let res: DoEvalResult<'_, 'func> = doeval(&self.code, context);

        res

        // match doeval(&self.code, context) {
        //     x @ DoEvalResult::RecursionLimit {..} => todo!(),
        //     DoEvalResult::ParsingError { context, string_tokens } => todo!(),
        //     DoEvalResult::EvalError { context, error } => todo!(),
        //     a @ DoEvalResult::Ok {..} => a,
        // }
    }
}
