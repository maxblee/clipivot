use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum CsvPivotError {
    InvalidField,
}

impl fmt::Display for CsvPivotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CsvPivotError::InvalidField => write!(f, "Tried to access a column that does not exist"),
        }
    }
}

impl Error for CsvPivotError {
    fn description(&self) -> &str {
        match *self {
            CsvPivotError::InvalidField => "field not found"
        }
    }
}