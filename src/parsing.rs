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
use chrono::{Datelike, NaiveDateTime};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

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
    DateTypes(Option<NaiveDateTime>),
}

pub struct DateFormatter {
    parser: dtparse::Parser,
    pub parsing_info: dtparse::ParserInfo,
    default_date: NaiveDateTime,
}

impl fmt::Debug for DateFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.parsing_info.fmt(f)
    }
}

impl PartialEq for DateFormatter {
    fn eq(&self, other: &Self) -> bool {
        self.parsing_info == other.parsing_info && self.default_date == other.default_date
    }
}

impl Default for DateFormatter {
    fn default() -> DateFormatter {
        let parsing_info = dtparse::ParserInfo::default();
        let parser = dtparse::Parser::default();
        let cur_year = chrono::Local::today().year();
        let default_date = chrono::NaiveDate::from_ymd(cur_year, 1, 1).and_hms(0, 0, 0);
        DateFormatter {
            parsing_info,
            parser,
            default_date,
        }
    }
}

impl DateFormatter {
    pub fn parse(&self, new_val: &str) -> Result<NaiveDateTime, CsvPivotError> {
        // ignore tokens (not using in impl)
        // TODO handle offsets/timezones
        // TODO Currently fails on "01042007" formatted dates because of underlying dtparser/Python dateutil issue
        // (See https://github.com/dateutil/dateutil/issues/796 )
        let (dt, _offset, _tokens) = self
            .parser
            .parse(
                new_val,
                Some(self.parsing_info.dayfirst),
                Some(self.parsing_info.yearfirst),
                false,
                false,
                Some(&self.default_date),
                false,
                &HashMap::new(),
            )
            .or(Err(CsvPivotError::ParsingError))?;
        Ok(dt)
    }
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
    date_helper: Option<DateFormatter>,
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Text(None),
            possible_values: vec![],
            parse_empty: true,
            date_helper: None,
        }
    }
}

impl ParsingHelper {
    pub fn from_parsing_type(parsing: ParsingType) -> ParsingHelper {
        let date_helper = match parsing {
            // TODO handle other date formats
            ParsingType::DateTypes(_) => Some(DateFormatter::default()),
            _ => None,
        };
        ParsingHelper {
            values_type: parsing,
            possible_values: vec![],
            parse_empty: true,
            date_helper,
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
            ParsingType::DateTypes(_) => self.parse_datetime(new_val),
        }?;
        Ok(Some(parsed_val))
    }

    fn parse_datetime(&self, new_val: &str) -> Result<ParsingType, CsvPivotError> {
        let parsed_dt = match &self.date_helper {
            Some(helper) => helper.parse(new_val),
            None => Err(CsvPivotError::ParsingError),
        }?;
        Ok(ParsingType::DateTypes(Some(parsed_dt)))
    }

    fn parse_numeric(new_val: &str) -> Result<ParsingType, CsvPivotError> {
        let dec = Decimal::from_str(new_val)
            .or(Decimal::from_scientific(&new_val.to_ascii_lowercase()))  // infer scientific notation on error
            .or(Err(CsvPivotError::ParsingError))?;
        Ok(ParsingType::Numeric(Some(dec)))
    }

    fn parse_floating(new_val: &str) -> Result<ParsingType, CsvPivotError> {
        let num: f64 = new_val.parse().or(Err(CsvPivotError::ParsingError))?;
        Ok(ParsingType::FloatingPoint(Some(num)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_scientific_notation_parsed() -> Result<(), CsvPivotError> {
        // Makes sure Decimal conversion parses numbers as scientific notation
        let scinot1 = ParsingHelper::parse_numeric("1e-4");
        assert!(scinot1.is_ok());
        let scinot1_extract = match scinot1 {
            Ok(ParsingType::Numeric(Some(val))) => Ok(val.to_string()),
            Ok(_) => Ok("".to_string()),
            Err(e) => Err(e)
        }?;
        assert_eq!(scinot1_extract, "0.0001".to_string());
        let scinot2 = ParsingHelper::parse_numeric("1.3E4");
        assert!(scinot2.is_ok());
        let scinot2_extract = match scinot2 {
            Ok(ParsingType::Numeric(Some(val))) => Ok(val.to_string()),
            Ok(_) => Ok("".to_string()),
            Err(e) => Err(e)
        }?;
        assert_eq!(scinot2_extract, "13000".to_string());
        Ok(())
    }

    #[test]
    fn test_automatic_date_conversion() -> Result<(), CsvPivotError> {
        // determines whether valid month, day, year formats get properly handled
        // Note that this should also handle ISO8601 formats
        let parsable_dates = vec![
            "2003-01-03",
            "1/3/03",
            "January 3, 2003",
            "Jan 3, 2003",
            "3 Jan 2003",
            "3 January 2003",
            "Jan 3, 2003 12:00 a.m.",
            "Jan 3, 2003 12:00:00 a.m.",
            "Jan 3, 2003 00:00",
            "Jan 3, 2003 00:00:00",
            "2003-01-03T00:00:00",
            "2003-01-03T00:00:00+00:00",
            "2003.01.03",
            "Jan. 3, 2003",
        ];
        let helper = ParsingHelper::from_parsing_type(ParsingType::DateTypes(None));
        for date in parsable_dates {
            let parsed_opt_date = helper.parse_val(date)?;
            let parsed_date = match parsed_opt_date {
                Some(ParsingType::DateTypes(Some(val))) => Ok(val),
                _ => Err(CsvPivotError::ParsingError),
            }?;
            let naive_date = NaiveDate::from_ymd(2003, 1, 3);
            let naive_time = NaiveTime::from_hms(0, 0, 0);
            let expected_utc = NaiveDateTime::new(naive_date, naive_time);
            assert_eq!(parsed_date, expected_utc);
        }
        Ok(())
    }
}
