use super::operators::OperatorType;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Arises when an expression failed to pase at a particular index
    Parsing(usize),

    /// Arises when an `Operator` failed to compute a value. e.g. when there are insufficient arguments.
    Operand(OperatorType),

    /// Arises when the stack is empty for some reason
    EmptyStack,

    /// Arises when parentheses mismatch
    MismatchingParens,

    /// Arises when an unknown `Variable` is found at a particular index
    UnknownVariable(usize),

    UnknownFunction(usize),
    RecursionLimit
}
