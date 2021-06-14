use rustyline::validate::{ValidationResult, Validator};

use super::MyHelper;

impl Validator for MyHelper<'_> {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        // let line = ctx.input();
        // let context = EvaluationContext::default();

        // // dbg!(line);

        // let (valid_, result) = if tokenize(line, &context).is_err() {
        //     (false, ValidationResult::Incomplete)
        // } else {
        //     (true, ValidationResult::Valid(None))
        // };
        // *self.valid.borrow_mut() = valid_;
        // rustyline::Result::Ok(result)
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        // true
        false
    }
}
