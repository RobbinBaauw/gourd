#![warn(missing_docs)]
#![allow(clippy::redundant_static_lifetimes)]

//! Gourd allows

use std::fs;
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::wrapper::wrap;
use crate::wrapper::Run;
use crate::wrapper_binary::Measurement;

#[cfg(test)]
/// The tests validating the behaviour of `gourd`.
mod tests;

pub(crate) mod error;
pub(crate) mod wrapper;
pub(crate) mod wrapper_binary;

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
fn main() {
    println!("Hello, world!");

    let path = "./bin".parse::<PathBuf>().unwrap();

    let _: Vec<ExitStatus> = wrap(
        vec![Run {
            binary: path,
            arguments: vec![],
        }],
        vec!["./test1".parse().unwrap()],
        62,
    )
    .unwrap()
    .iter_mut()
    .map(|x| x.spawn().unwrap().wait().unwrap())
    .collect();

    let results: Measurement =
        postcard::from_bytes(&fs::read("/tmp/gourd/run0_0_result").unwrap()).unwrap();

    println!("{:?}", results);
}
