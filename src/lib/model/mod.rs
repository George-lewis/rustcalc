use self::{errors::ErrorContext, functions::Function, variables::Variable};

pub mod constants;
pub mod errors;
pub mod functions;
pub mod operators;
pub mod tokens;
pub mod variables;

mod representable;

#[derive(Default, Clone)]
pub struct EvaluationContext<'a> {
    pub vars: &'a [Variable],
    pub funcs: &'a [Function],
    pub context: ErrorContext,
    pub depth: u8,
}

#[cfg(test)]
mod tests {

    use super::{
        errors::{ContextualError, Error, ErrorContext},
        functions::{Function, Functions},
        operators::{Associativity, Operator, OperatorType},
        tokens::Token,
        variables::Variable,
        EvaluationContext
    };

    /// Outputs `size_of` information for a list of types
    macro_rules! sizeof {
        ($($type_:ty),+) => {
            $(
                println!("sizeof({}): {} bytes", stringify!($type_), std::mem::size_of::<$type_>());
            )+
        };
    }

    // Sizes of our data types
    // Mostly for curiosity
    #[test]
    fn size_of() {
        sizeof! {
            ContextualError,
            EvaluationContext,
            Function,
            ErrorContext,
            Variable,
            Token,
            Functions,
            &Functions,
            Error,
            Operator,
            OperatorType,
            Associativity
        }
    }
}
