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
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
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
    /// This is used for parsing date types
    DateTypes(Option<chrono::DateTime<Utc>>),
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

// TODO: Get rid of this
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
    // TODO: Get rid of this
    pub fn set_numeric() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Numeric(None),
            possible_values: vec![],
            parse_empty: true,
        }
    }

    // TODO: Get rid of this
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
            ParsingType::DateTypes(_) => ParsingHelper::parse_datetime(new_val),
        }?;
        Ok(Some(parsed_val))
    }

    fn parse_datetime(new_val: &str) -> Result<ParsingType, CsvPivotError> {
        Ok(ParsingType::DateTypes(None))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::ParsingType::DateTypes;

    #[test]
    fn test_automatic_date_conversion() -> Result<(), CsvPivotError> {
        // determines whether valid month, day, year formats get properly handled
        // Note that this should also handle ISO8601 formats
        let parsable_dates = vec!["2003-01-03", "1/3/03", "January 3, 2003",
        "Jan 3, 2003", "3 Jan 2003", "3 January 2003", "Jan 3, 2003 12:00 a.m.",
        "Jan 3, 2003 12:00:00 a.m.", "Jan 3, 2003 00:00", "Jan 3, 2003 00:00:00",
        "2003-01-03T00:00:00", "2003-01-03T00:00:00+00:00", "2003.01.03", "Jan. 3, 2003"];
        let helper = ParsingHelper::from_parsing_type(DateTypes(None));
        for date in parsable_dates {
            let parsed_opt_date = helper.parse_val(date)?;
            let parsed_date = match parsed_opt_date {
                Some(ParsingType::DateTypes(Some(val))) => Ok(val),
                _ => Err(CsvPivotError::ParsingError)
            }?;
            let expected_utc = DateTime::<Utc>::from_utc(chrono::Utc
                .ymd(2003, 1, 3)
                .and_hms(0,0,0)
                .naive_utc(), Utc);
            assert_eq!(parsed_date, expected_utc);
        }
        Ok(())
    }
}