use std::process::Command;
use std::process::ExitStatus;

use crate::error::GourdError;

pub fn run_locally(tasks: Vec<Command>) -> Result<Vec<ExitStatus>, GourdError> {
    todo!()
}
