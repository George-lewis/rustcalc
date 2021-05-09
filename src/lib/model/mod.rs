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
