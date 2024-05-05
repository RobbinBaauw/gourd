#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![allow(clippy::redundant_static_lifetimes)]
// for tarpaulin cfg
#![allow(unexpected_cfgs)]

//! Gourd allows

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::config::Config;
use crate::constants::X86_64_E_MACHINE;
use crate::measurement::Measurement;
use crate::wrapper::wrap;
use crate::wrapper::Program;

/// The tests validating the behaviour of `gourd`.
#[cfg(test)]
pub mod tests;

/// The error type of `gourd`.
pub mod error;

/// A struct and related methods for global configuration,
/// declaratively specifying experiments.
pub mod config;

/// The binary wrapper around run programs.
pub mod wrapper;

/// Constant values.
pub mod constants;

/// The local runner module: `gourd run local`.
pub mod local;

/// Code shared between the wrapper and `gourd`.
pub mod measurement;
mod local;

/// Code for accessing and managing resouces
pub mod resources;

/// The main entrypoint.
///
/// This function is the main entrypoint of the program.
#[cfg(not(tarpaulin_include))]
fn main() {
    println!("Hello, world!");
    let config_path = String::from("gourd.toml");

    println!("Loading configuration file at '{}'", config_path);
    let config = Config::from_file(Path::new(&config_path)).unwrap();
    // Prints contents of the configuration file. Remove.
    println!("{:?}", config);

    let path = "./bin".parse::<PathBuf>().unwrap();

    let _: Vec<ExitStatus> = wrap(
        vec![Program {
            binary: path,
            arguments: vec![],
        }],
        vec!["./test1".parse().unwrap()],
        X86_64_E_MACHINE,
        &config,
    )
    .unwrap()
    .iter_mut()
    .map(|x| {
        println!("{:?}", x);
        x.spawn().unwrap().wait().unwrap()
    })
    .collect();

    let results: Measurement = toml::from_str(
        &String::from_utf8(fs::read("/tmp/gourd/algo_0/0_result").unwrap()).unwrap(),
    )
    .unwrap();

    println!("{:?}", results);
}
