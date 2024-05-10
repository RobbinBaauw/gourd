#![cfg(not(tarpaulin_include))]

use std::fmt::Display;

use crate::constants::ERROR_STYLE;
use crate::constants::HELP_STYLE;

/// The error context structure, provides an explanation and help.
#[derive(Debug)]
pub struct Ctx<A, B>(pub A, pub B)
where
    A: Display,
    B: Display;

impl<A: Display, B: Display> Display for Ctx<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}caused by:{:#} {}", ERROR_STYLE, ERROR_STYLE, self.0)?;

        if !format!("{}", self.1).is_empty() {
            writeln!(f, "\n{}help:{:#} {}", HELP_STYLE, HELP_STYLE, self.1)?;
        }

        Ok(())
    }
}

/// This is a shorthand for returning the context of a error.
macro_rules! ctx {
    {$cause: expr,  $($arg_cause: expr)*; $help: expr, $($arg_help: tt)*} => {
      || Ctx(format!($cause, $($arg_cause)*), format!($help, $($arg_help)*))
    };
}

pub(crate) use ctx;
