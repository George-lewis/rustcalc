use self::{functions::Function, variables::Variable};

pub mod constants;
pub mod errors;
pub mod functions;
pub mod operators;
pub mod tokens;
pub mod variables;

mod representable;

#[derive(Default, Clone, Copy)]
pub struct EvaluationContext<'a> {
    pub vars: &'a [Variable],
    pub funcs: &'a [Function],
    pub depth: u8
}
