//! The module for parsing through text records.
//!
//! This is designed to allow for a way for `csvpivot` to convert
//! string records from a CSV file into meaningful records of some
//! other variety (like numbers and dates).
//!
//! Currently, the program only does this by converting from a
//! `&str` record to a record of the `ParsingType` enum. However,
//! I eventually want to extend the functionality of this so the program
//! can automatically determine the type of record appearing in the values column.

use crate::errors::CsvPivotError;

/// The types of data `csvpivot` converts `&str` records into.
/// `csvpivot` only does these conversions on the values column.
/// (Note: I may eventually change this.)
#[derive(Debug, PartialEq)]
pub enum ParsingType {
    /// Representing String data
    Text(Option<String>)
}

/// Stores information about the type of data appearing in the values column
/// of your pivot table.
#[derive(Debug, PartialEq)]
pub struct ParsingHelper {
    /// Represents the type of data `ParsingHelper` will convert `&str` records
    /// into while aggregating
    values_type: ParsingType,
    /// Not being used right now, but designed for use when I automatically determine
    /// the data type of a given values field.
    possible_values: Vec<ParsingType>
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Text(None),
            possible_values: vec![ParsingType::Text(None)]
        }
    }
}

impl ParsingHelper {
    /// Converts a new `&str` value read in from a CSV file into a format determined
    /// by the `values_type` attribute of `ParsingHelper`. Raises an error
    /// if the value read in is not compatible with the parsing type.
    pub fn parse_val(&self, new_val: &str) -> Result<ParsingType, CsvPivotError> {
        match self.values_type {
            ParsingType::Text(_) => Ok(ParsingType::Text(Some(new_val.to_string()))),
        }
    }
}