// The entire structure of this file / these error messages comes from
// https://blog.burntsushi.net/rust-error-handling/
// Which, such a good guide

//! The module for describing recoverable errors in `csvpivot`.
//!
//! Finally, I'd be remiss if I failed to mention the inspiration for this particular
//! format of error handling. I *heavily* based this module on
//! [the error handling guide](https://blog.burntsushi.net/rust-error-handling/)
//! from Andrew Gallant, which is a terrific research for handling errors
//! and understanding combinators in Rust.

extern crate csv;

use std::error::Error;
use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum CsvPivotError {
    /// Errors caused from reading a CSV file, either because of problems in the
    /// formatting of the file or because of problems accessing a given field
    CsvError(csv::Error),
    /// This error is thrown if your initial configuration is not valid.
    ///
    /// For instance, you will receive this error if you set a delimiter as a multi-character string.
    InvalidConfiguration(String),
    /// A standard IO error. Typically from trying to read a file that does not exist
    Io(io::Error),
    /// Errors trying to parse a new value
    ParsingError {
        line_num: usize,
        err: String,
    }
}

impl fmt::Display for CsvPivotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvPivotError::CsvError(ref err) => err.fmt(f),
            CsvPivotError::InvalidConfiguration(ref err) => {
                write!(f, "Could not properly configure the aggregator: {}", err)
            }
            CsvPivotError::Io(ref err) => err.fmt(f),
            // adapted from https://github.com/BurntSushi/rust-csv/blob/master/src/error.rs
            CsvPivotError::ParsingError { line_num: ref line_num, err: ref err } => {
                write!(
                    f,
                    "Could not parse record {}: {}",
                    line_num + 1,
                    err
                )
            },
        }
    }
}

impl Error for CsvPivotError {
    fn description(&self) -> &str {
        match *self {
            CsvPivotError::CsvError(ref err) => err.description(),
            CsvPivotError::Io(ref err) => err.description(),
            CsvPivotError::InvalidConfiguration(ref _err) => "could not configure the aggregator",
            CsvPivotError::ParsingError {line_num: ref _num, err: ref _err } => "failed to parse values column",
        }
    }
}

impl From<io::Error> for CsvPivotError {
    fn from(err: io::Error) -> CsvPivotError {
        CsvPivotError::Io(err)
    }
}

impl From<csv::Error> for CsvPivotError {
    fn from(err: csv::Error) -> CsvPivotError {
        CsvPivotError::CsvError(err)
    }
}