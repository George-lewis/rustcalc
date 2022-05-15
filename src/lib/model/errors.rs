use super::{functions::Function, operators::OperatorType, tokens::StringToken};

#[derive(Debug, PartialEq, Clone)]
pub enum InnerFunction<'a> {
    Builtin(OperatorType),
    User(&'a Function),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorContext<'funcs> {
    Main,
    Scoped(&'funcs Function),
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
    UnknownFunction,
}

#[derive(Debug, Clone, Copy)]
pub enum RpnError {
    MismatchingParens,
}

#[derive(Debug, Clone)]
pub enum EvalError<'str, 'funcs> {
    EmptyStack,
    Operand {
        op: InnerFunction<'funcs>,
        tok: StringToken<'str, 'funcs>,
    },
}
