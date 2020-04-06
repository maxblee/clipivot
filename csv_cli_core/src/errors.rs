//! The module for describing recoverable errors in my CSV command-lline tools.
//!
//! > *Note:* All of the error handling in this module is structured from
//! > [this error handling guide](https://blog.burntsushi.net/rust-error-handling)
//! > and from the source code of the [csv crate](https://github.com/BurntSushi/rust-csv)
//! > in Rust. If you're hoping to implement you're own library or binary in Rust,
//! > I highly recommend looking at both (and, especiialy, the guide).
//!
//! You can characterize all four error types in two general categories:
//! errors configuring the CSV reader and errors parsing individual lines.
//! For errors relating to configuration, my goal is simply to be as specific
//! and clear as possible about the nature of a given error. For errors relating to
//! parsing, however, I also think it's important to display record numbers to help
//! users debug errors they run into. Currently, this refers to the 0-indexed number in
//! which a record appears in a CSV document. So record 5 of a CSV would be the seventh line
//! of a CSV with a header row and the sixth line of a CSV without a header row.
//!
//! This indexing plan is meant to interact nicely with the `xsv slice` subcommand in the
//! [`xsv`](https://github.com/BurntSushi/xsv) toolkit. So if you run into an error, you can type:
//! ```shell
//! $ xsv slice YOUR_FILENAME -i <RECORD_NUMBER>
//! ```
//! to see the full line that caused you to run into an error.
//!

extern crate csv;

use std::error::Error;
use std::fmt;
use std::io;
use std::result;

/// An alias for CsvCliError
// from https://github.com/BurntSushi/rust-csv/blob/master/src/error.rs
pub type CsvCliResult<T> = result::Result<T, CsvCliError>;

/// The type of CSV error
#[derive(Debug)]
pub enum CsvCliError {
    /// Errors from reading a CSV file.
    ///
    /// This should be limited to inconsistencies in the number of lines appearing in a given row
    /// or errors parsing data as UTF-8.
    CsvError(csv::Error),
    /// Errors in the initial configuration from command-line arguments.
    ///
    /// This error likely occurs most frequently because of problems in how fields are named
    /// but can also occur because of errors parsing delimiters as single UTF-8 characters.
    InvalidConfiguration(String),
    /// A standard IO error. Typically from trying to read a file that does not exist
    Io(io::Error),
    /// Errors trying to parse a new value.

    /// The way in which `clipivot` parses values depends on the aggregation function
    /// and command-line flags, but all errors in converting the string records in the values
    /// column into a particular data type result in a `ParsingError`.
    ParsingError {
        /// The current line number. This conforms with the way that `xsv slice` operates
        /// so you can easily find the row that failed by running `xsv slice`.
        line_num: usize,
        /// The string that failed to parse. This allows you to avoid having to run operations
        /// like `xsv slice` in most cases
        str_to_parse: String,
        /// The general error message. This is specific to the type of error, so failures to parse
        /// data as datetimes will tell you they failed to parse datetimes, etc.
        err: String,
    },
}

impl fmt::Display for CsvCliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvCliError::CsvError(ref err) => err.fmt(f),
            CsvCliError::InvalidConfiguration(ref err) => {
                write!(f, "Could not properly configure the aggregator: {}", err)
            }
            CsvCliError::Io(ref err) => err.fmt(f),
            // adapted from https://github.com/BurntSushi/rust-csv/blob/master/src/error.rs
            CsvCliError::ParsingError {
                ref line_num,
                ref str_to_parse,
                ref err,
            } => write!(
                f,
                "Could not parse record `{}` with index {}: {}",
                str_to_parse, line_num, err
            ),
        }
    }
}

impl Error for CsvCliError {
    fn description(&self) -> &str {
        match *self {
            CsvCliError::CsvError(ref err) => err.description(),
            CsvCliError::Io(ref err) => err.description(),
            CsvCliError::InvalidConfiguration(ref _err) => "could not configure the aggregator",
            CsvCliError::ParsingError {
                line_num: ref _num,
                str_to_parse: ref _str,
                err: ref _err,
            } => "failed to parse values column",
        }
    }
}

impl From<io::Error> for CsvCliError {
    fn from(err: io::Error) -> CsvCliError {
        CsvCliError::Io(err)
    }
}

impl From<csv::Error> for CsvCliError {
    fn from(err: csv::Error) -> CsvCliError {
        CsvCliError::CsvError(err)
    }
}
