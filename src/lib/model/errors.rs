use super::{functions::Function, operators::OperatorType};

#[derive(Debug, PartialEq)]
pub enum InnerFunction {
    Builtin(OperatorType),
    User(Function),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorContext {
    Main,
    Scoped(Function),
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Arises when an expression failed to pase at a particular index
    Parsing(usize),

    /// Arises when an `Operator` failed to compute a value. e.g. when there are insufficient arguments.
    Operand(InnerFunction),

    /// Arises when the stack is empty for some reason
    EmptyStack,

    /// Arises when parentheses mismatch
    MismatchingParens,

    /// Arises when an unknown `Variable` is found at a particular index
    UnknownVariable(usize),

    UnknownFunction(usize),

    RecursionLimit,
}

impl Error {
    pub fn with_context(self, context: ErrorContext) -> ContextError {
        ContextError {
            context,
            error: self,
        }
    }
}

// impl From<Error> for ContextError<'_> {
//     fn from(error: Error) -> Self {
//         Self {
//             context: ErrorContext::Main,
//             error
//         }
//     }
// }

pub struct ContextError {
    pub context: ErrorContext,
    pub error: Error,
}
