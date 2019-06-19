// The entire structure of this file / these error messages comes from
// https://blog.burntsushi.net/rust-error-handling/
// Which, such a good guide

//! I've used this errors module to describe all of the recoverable errors
//! that (I know) can occur from creating a pivot table with `csvpivot.`
//! If you want to improve the error handling in this program,
//! I'd appreciate help making the InvalidField error more specific and useful.
//!
//! Finally, I'd be remiss if I failed to mention the inspiration for this particular
//! format of error handling. I *heavily* based this module on
//! [the error handling guide](https://blog.burntsushi.net/rust-error-handling/)
//! from Andrew Gallant, which is a terrific research for handling errors
//! and understanding combinators in Rust.

extern crate csv;

use std::fmt;
use std::error::Error;
use std::io;
use std::num;

/// Covers all errors in `csvpivot`. Most of these are outside errors.
/// However, I use the InvalidField error to note when the user
/// made an error in declaring which fields to aggregate on.
/// This means that if the user used a query like
/// --col Applejuice, they will receive a more helpful error than a ParseIntError
#[derive(Debug)]
pub enum CsvPivotError {
    /// Errors caused from reading a CSV file, either because of problems in the
    /// formatting of the file or because of problems accessing a given field
    CsvError(csv::Error),
    /// This error type is thrown if you enter an aggregation method that does not exist.
    /// For instance, if you type
    /// ```bash
    /// csvpivot badcount -c 0 -i 1 -v 2
    /// ```
    /// you could theoretically receive this error. However, because of how this binary
    /// interacts with `Clap`, you should never see this. If you do, please contact me
    /// at maxbmhlee@gmail.com or send a pull request.
    InvalidAggregator,
    /// Errors caused by trying to access a field that doesn't exist. Either appears
    /// when trying to search by column name (instead of by index) or when trying
    /// to access, say, the 5th field of a CSV file that has 4 fields.
    /// I eventually want to fix this to make it clearer. I may also fiddle with replacing
    /// this with CsvError in the latter of these two cases.
    InvalidField,
    /// A standard IO error. Typically from trying to read a file that does not exist
    Io(io::Error),
    /// An error occurring when the program tries to convert a string into an integer but is
    /// unable to
    ParseInt(num::ParseIntError),
}

impl fmt::Display for CsvPivotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvPivotError::CsvError(ref err) => err.fmt(f),
            // TODO: I need to work on making this message more helpful
            CsvPivotError::InvalidField => write!(f, "Invalid field error: You tried to access a \
             field that does not exist."),
            // This error only appears if you enter an aggregation function that isn't supported
            // But Clap should prevent those messages from ever getting passed through
            // to CliConfig, so it shouldn't be a problem
            // tldr: the error exists bc I needed a comprehensive match statement in aggregation.rs
            CsvPivotError::InvalidAggregator => write!(f, "Invalid aggregation error: \
             You should never get this error. If you see it,\
             please send a bug report to maxbmhlee@gmail.com"),
            CsvPivotError::Io(ref err) => err.fmt(f),
            CsvPivotError::ParseInt(ref err) => err.fmt(f),
        }
    }
}

impl Error for CsvPivotError {
    fn description(&self) -> &str {
        match *self {
            CsvPivotError::CsvError(ref err) => err.description(),
            CsvPivotError::Io(ref err) => err.description(),
            CsvPivotError::InvalidAggregator => "aggregation failed",
            CsvPivotError::InvalidField => "field not found",
            CsvPivotError::ParseInt(ref err) => err.description(),
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

impl From<num::ParseIntError> for CsvPivotError {
    fn from(err: num::ParseIntError) -> CsvPivotError {
        CsvPivotError::ParseInt(err)
    }
}