use super::operators::OperatorType;

#[derive(Debug, PartialEq)]
pub enum Error {
    Parsing(usize),
    Operand(OperatorType),
    EmptyStack,
    MismatchingParens,
    Assignment,
    AssignmentName,
    UnknownVariable(usize)
}
