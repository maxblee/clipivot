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
use std::str::FromStr;
use rust_decimal::Decimal;
use crate::errors::CsvPivotError;

/// The types of data `csvpivot` converts `&str` records into.
/// `csvpivot` only does these conversions on the values column.
/// (Note: I may eventually change this.)
#[derive(Debug, PartialEq)]
pub enum ParsingType {
    /// Representing String data
    Text(Option<String>),
    /// This is used for all of the numeric operations with the exception of standard deviation
    Numeric(Option<Decimal>),
    /// This is used for numeric operations involving minimum and maximum, as well as standard deviation
    FloatingPoint(Option<f64>),
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
    possible_values: Vec<ParsingType>,
    parse_empty: bool,
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Text(None),
            possible_values: vec![],
            parse_empty: true,
        }
    }
}

impl ParsingHelper {
    pub fn from_parsing_type(parsing: ParsingType) -> ParsingHelper {
        ParsingHelper {
            values_type: parsing,
            possible_values: vec![],
            parse_empty: true,
        }
    }
    /// Converts a new `&str` value read in from a CSV file into a format determined
    /// by the `values_type` attribute of `ParsingHelper`. Raises an error
    /// if the value read in is not compatible with the parsing type.
    pub fn set_numeric() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Numeric(None),
            possible_values: vec![],
            parse_empty: true,
        }
    }

    pub fn set_floating() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::FloatingPoint(None),
            possible_values: vec![],
            parse_empty: true,
        }
    }

    pub fn parse_empty_vals(mut self, yes: bool) -> Self {
        self.parse_empty = yes;
        self
    }

    pub fn parse_val(&self, new_val: &str) -> Result<Option<ParsingType>, CsvPivotError> {
        // list of empty values heavily borrowed from `agate` in Python
        // https://agate.readthedocs.io/en/1.6.1/api/data_types.html
        let empty_vals = vec!["", "na", "n/a", "none", "null", "nan"];
        if empty_vals.contains(&new_val.to_ascii_lowercase().as_str()) && !self.parse_empty {
            return Ok(None);
        }
        let parsed_val = match self.values_type {
            ParsingType::Text(_) => Ok(ParsingType::Text(Some(new_val.to_string()))),
            ParsingType::Numeric(_) => ParsingHelper::parse_numeric(new_val),
            ParsingType::FloatingPoint(_) => ParsingHelper::parse_floating(new_val),
        }?;
        Ok(Some(parsed_val))
    }

    fn parse_numeric(new_val: &str) -> Result<ParsingType, CsvPivotError> {
        let dec = Decimal::from_str(new_val).or(Err(CsvPivotError::ParsingError))?;
        Ok(ParsingType::Numeric(Some(dec)))
    }

    fn parse_floating(new_val: &str) -> Result<ParsingType, CsvPivotError> {
        let num : f64 = new_val.parse().or(Err(CsvPivotError::ParsingError))?;
        Ok(ParsingType::FloatingPoint(Some(num)))
    }
}