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
    pub const fn with_context(self, context: ErrorContext) -> ContextualError {
        ContextualError {
            context,
            error: self,
        }
    }
}

#[derive(Debug)]
pub struct ContextualError {
    pub context: ErrorContext,
    pub error: Error,
}

impl ContextualError {
    // Why:
    // > destructors cannot be evaluated at compile-time
    // > constant functions cannot evaluate destructors
    // > rustc(E0493)
    // It may be possible to fix this later
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_context(self, context: ErrorContext) -> Self {
        Self {
            context,
            error: self.error,
        }
    }
}
