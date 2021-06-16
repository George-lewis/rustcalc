use std::borrow::Cow;

use super::{EvaluationContext, functions::Function, operators::OperatorType};

#[derive(Debug, PartialEq, Clone)]
pub enum InnerFunction<'a> {
    Builtin(OperatorType),
    User(&'a Function),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorContext<'a> {
    Main,
    Scoped(&'a Function),
}

impl Default for ErrorContext<'_> {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParserError {
    UnknownToken,
    UnknownVariable,
    UnknownFunction
}

#[derive(Debug, Clone, Copy)]
pub enum EvalError {
    EmptyStack,
    MismatchingParens
}

#[derive(Debug, Clone, Copy)]
pub enum EError {
    RecursionLimit,
    Parsing(ParserError),
    Eval(EvalError)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error<'a> {
    /// Arises when an expression failed to pase at a particular index
    Parsing,

    /// Arises when an `Operator` failed to compute a value. e.g. when there are insufficient arguments.
    Operand(InnerFunction<'a>),

    /// Arises when the stack is empty for some reason
    EmptyStack,

    /// Arises when parentheses mismatch
    MismatchingParens,

    /// Arises when an unknown `Variable` is found at a particular index
    UnknownVariable,

    UnknownFunction,

    RecursionLimit,
}

impl<'a> Error<'a> {
    pub const fn with_context(self, context: ErrorContext<'a>) -> ContextualError {
        ContextualError {
            context,
            error: self,
        }
    }
}

#[derive(Debug)]
pub struct ContextualError<'a> {
    pub context: ErrorContext<'a>,
    pub error: Error<'a>,
}

impl<'a> ContextualError<'a> {
    // Why:
    // > destructors cannot be evaluated at compile-time
    // > constant functions cannot evaluate destructors
    // > rustc(E0493)
    // It may be possible to fix this later
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_context(self, context: ErrorContext<'a>) -> Self {
        Self {
            context,
            error: self.error,
        }
    }
}


// pub fn some_err_with_context<T>(error: Error, context: &EvaluationContext) -> Option<Result<T, ContextualError>> {
//     Some(Err(error.with_context(context.context)))
// }

// pub fn some_err<T, Error>(error: Error) -> Option<Result<T, Error>> {
//     Some(Err(error))
// }