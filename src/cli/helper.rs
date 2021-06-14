#![allow(clippy::module_name_repetitions)]

use std::{borrow::Cow, cell::RefCell};

use colored::Colorize;
use rustmatheval::{
    model::{functions::Function, variables::Variable, EvaluationContext},
    tokenize,
};

use rustyline::{
    completion::{Candidate, Completer},
    highlight::Highlighter,
    hint::{Hint, Hinter},
    validate::{ValidationResult, Validator},
    Helper,
};

use crate::utils::find_last;
