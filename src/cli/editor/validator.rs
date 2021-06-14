use rustyline::validate::{ValidationResult, Validator};

use super::MyHelper;

impl Validator for MyHelper<'_> {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
