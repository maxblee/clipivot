extern crate csv;

use std::fmt;
use std::error::Error;
use std::io;
use std::num;

#[derive(Debug)]
pub enum CsvPivotError {
    CsvError(csv::Error),
    InvalidAggregator,
    InvalidField,
    Io(io::Error),
    ParseInt(num::ParseIntError),
}

impl fmt::Display for CsvPivotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvPivotError::CsvError(ref err) => err.fmt(f),
            CsvPivotError::InvalidField => write!(f, "Tried to access a column that does not exist"),
            CsvPivotError::InvalidAggregator => write!(f, "Could not properly configure for aggregating"),
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