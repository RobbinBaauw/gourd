#![cfg(not(tarpaulin_include))]

use std::fmt::Display;
use std::path::PathBuf;

use elf::to_str::e_machine_to_human_str;
use elf::ParseError;
use tokio::task::JoinError;

/// This error type is used by all gourd functions.
#[allow(dead_code)]
#[derive(Debug)]
pub enum GourdError {
    /// The configuration file could not be read.
    ConfigLoadError(Option<std::io::Error>, String),

    /// The architecture does not match the one we want to run on.
    ArchitectureMismatch {
        /// The expected architecture in `e_machine` format.
        expected: u16,

        /// The architecture of the binary in `e_machine` format.
        binary: u16,
    },

    /// A filesystem error occured.
    FileError(PathBuf, std::io::Error),

    /// A file unrelated filesystem error occured.
    IoError(std::io::Error),

    /// This ELF file failed to parse
    ElfParseError(ParseError),

    /// Couldn't join the child in the runner
    ChildJoinError(JoinError),

    /// Couldn't spawn the child
    ChildSpawnError(std::io::Error),

    /// Couldn't access an online resource
    NetworkError(reqwest::Error),
}

impl Display for GourdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigLoadError(_err, reason) => {
                write!(f, "The configuration file could not be read: {}", reason)
            }
            Self::ArchitectureMismatch { expected, binary } => write!(
                f,
                "The {:?} architecture does not match {:?}, the runners architecture",
                e_machine_to_human_str(*binary),
                e_machine_to_human_str(*expected)
            ),
            Self::FileError(file, io_err) => {
                write!(f, "Could not access file {:?}: {}", file, io_err)
            }
            Self::IoError(io_err) => write!(f, "An IO error occurred: {}", io_err),
            Self::ElfParseError(err) => write!(f, "This is not a valid elf file: {}", err),
            Self::ChildJoinError(err) => write!(f, "Could not join child to main thread: {}", err),
            Self::ChildSpawnError(err) => write!(f, "Could not spawn child: {}", err),
            Self::NetworkError(err) => write!(f, "Couldn't access an online resource: {}", err),
        }
    }
}

impl From<std::io::Error> for GourdError {
    fn from(value: std::io::Error) -> Self {
        GourdError::IoError(value)
    }
}

impl From<ParseError> for GourdError {
    fn from(value: ParseError) -> Self {
        GourdError::ElfParseError(value)
    }
}
