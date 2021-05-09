use self::{errors::ErrorContext, functions::Function, variables::Variable};

pub mod constants;
pub mod errors;
pub mod functions;
pub mod operators;
pub mod tokens;
pub mod variables;

mod representable;

#[derive(Default, Clone, Copy)]
pub struct EvaluationContext<'var, 'func> {
    pub vars: &'var [Variable],
    pub funcs: &'func [Function],
    pub context: ErrorContext<'func>,
    pub depth: u8,
}
