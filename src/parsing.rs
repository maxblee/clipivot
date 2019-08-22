//! The module for parsing through text records.
//!
//! This module interacts with the `Aggregator` class
//! from the `aggregation` module. Every time we add a new record into the aggregator,
//! we parse it first, serializing the record into a given value.
//! 
//! Right now, there are four different types we can parse records into,
//! represented by the `ParsingType` enum. The floating point and numeric types
//! both represent numeric data. As you might assume, the difference is in the types of data they hold.
//! `FloatingPoint` `ParsingType` records hold floating point values, as you probably guessed, while
//! `Numeric` `ParsingType` records hold decimal values. (Mean and sum both use the Decimal type
//! to avoid truncation errors.)
//!
//! In addition, we have a `ParsingType` enum for text, which basically just converts `&str` splices
//! into `String` objects, and a `DateType` `ParsingType` enum which converts string dates into
//! datetimes using `dtparse`, Rust's equivalent of the Python `dateutil` parser.
//!
//! Finally, there are the two structs that do the heavy lifting, `ParsingHelper` and `DateFormatter`.
//! The `Aggregator` struct passes each value in the values column field as a string to the `ParsingHelper`.
//! The `ParsingHelper` will then serialize the string based on its settings, before handing the final, serialized
//! object back to the `Aggregator`. (Then, the `Aggregator` passes those values to one of the structs
//! implementing the `AggregationMethod` trait.) And finally, the `DateFormatter` serves as a helper struct
//! for `ParsingHelper`, providing settings for parsing dates to the `ParsingHelper`.
//!
//! Because `csvpivot` comes with support for parsing text, numeric data, and datetimes, you probably don't
//! need to change anything in the `parsing` module in order to add a feature (assuming you want to add a feature).
//! Instead, all you'll need to do is set how you want data to be parse in your new feature in the `get_parsing_approach`
//! function from within the `aggregation` module (and specifically, from within the `CliConfig` struct). The
//! implementation for `Count` should give you a sense of how this is done:
//!
//! ```rust
//! match U::get_aggtype {
//!     AggTypes::Count => ParsingType::Text(None)
//! ...
//! }   
//! ```
//! But in case you want to add new parsing types or alter the implementation of parsing
//! in `csvpivot`, taking a closer look at `ParsingHelper`, `ParsingType`, and `DateFormatter` might be helpful.

use crate::errors::CsvPivotError;
use chrono::{Datelike, NaiveDateTime};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// The types of data `csvpivot` parses. **Note** that `csvpivot` only parses the value column
/// of your data set. (That is, the indexes and columns of your pivot table are purely parsed as strings.)
#[derive(Debug, PartialEq)]
pub enum ParsingType {
    /// Represents String data
    Text(Option<String>),
    /// This is used for most numeric calculations. The Decimal type prevents truncation errors from
    /// unnecessarily reducing the accuracy of your calculations.
    Numeric(Option<Decimal>),
    /// This is used for numeric operations involving minimum and maximum, as well as standard deviation
    FloatingPoint(Option<f64>),
    /// This is used for parsing date types. Its implementation is fairly slow. For this reason,
    /// calculating minimum or maximum on ISO-formatted dates (e.g. 2019-08-12) can use either
    /// dates or Strings. (And I recommend you use strings if you are dealing with ISO-formatted dates.)
    DateTypes(Option<NaiveDateTime>),
}

/// Forms the settings for parsing dates. The settings come directly from the command-line arguments.
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
    /// Converts string dates into datetimes or errors.
    pub fn new(dayfirst: bool, yearfirst: bool) -> DateFormatter {
        let mut base_formatter = DateFormatter::default();
        base_formatter.parsing_info.dayfirst = dayfirst;
        base_formatter.parsing_info.yearfirst = yearfirst;
        base_formatter
    }
    pub fn parse(&self, new_val: &str, line_num: usize) -> Result<NaiveDateTime, CsvPivotError> {
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
            .or(Err(CsvPivotError::ParsingError {
                line_num, str_to_parse: new_val.to_string(),
                err: "Failed to parse datetime".to_string()
    }))?;
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
    /// Determines whether or not to parse empty values. If you use the `-e` flag
    /// with any query, `parse_empty` will be true and the `ParsingHelper` will pass over the record any time
    /// it's empty
    parse_empty: bool,
    /// The date formatter. This object is `None` when you are dealing with data other than
    /// datetimes and is some `DateFormatter` otherwise.
    date_helper: Option<DateFormatter>,
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Text(None),
            parse_empty: true,
            date_helper: None,
        }
    }
}

impl ParsingHelper {
    /// This method is used by `CliConfig` to initialize the `ParsingHelper` the `Aggregator` uses.
    pub fn from_parsing_type(parsing: ParsingType, dayfirst: bool, yearfirst: bool) -> ParsingHelper {
        let date_helper = match parsing {
            ParsingType::DateTypes(_) => Some(DateFormatter::new(dayfirst, yearfirst)),
            _ => None,
        };
        ParsingHelper {
            values_type: parsing,
            parse_empty: true,
            date_helper,
        }
    }

        // the following approach to method chaining comes from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    /// Adds the list of index columns to the default aggregator.
    /// (This approach to method chaining comes from
    /// [http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html](http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html).)
    pub fn parse_empty_vals(mut self, yes: bool) -> Self {
        self.parse_empty = yes;
        self
    }

    /// Parses a string from the `Aggregator`. Returns `Ok(Some(ParsingType))` if it a)
    /// doesn't run into an error and b) doesn't parse as empty; `Ok(None)` if it a)
    /// doesn't run into an error but b) parses the string as empty; and 
    /// `Err(CsvPivotError)` if it can't parse the string.
    ///
    /// **Note** that it only determines that a cell is empty if you have set the program
    /// to skip past empty values (using the `-e` flag) and the cell has one of the following values:
    /// - ""
    /// - "na"
    /// - "n/a"
    /// - "none"
    /// - "null"
    /// - "nan"
    ///
    /// (Thanks to Python's [`agate` library](https://agate.readthedocs.io/en/1.6.1/api/data_types.html)
    /// for coming up with these null values so I didn't have to.)
    pub fn parse_val(&self, new_val: &str, line_num: usize) -> Result<Option<ParsingType>, CsvPivotError> {
        // list of empty values heavily borrowed from `agate` in Python
        // https://agate.readthedocs.io/en/1.6.1/api/data_types.html
        // Note: this should probably use a HashSet, but doesn't matter enough for me to figure out how to do that.
        let empty_vals = vec!["", "na", "n/a", "none", "null", "nan"];
        if empty_vals.contains(&new_val.to_ascii_lowercase().as_str()) && !self.parse_empty {
            return Ok(None);
        }
        let parsed_val = match self.values_type {
            ParsingType::Text(_) => Ok(ParsingType::Text(Some(new_val.to_string()))),
            ParsingType::Numeric(_) => ParsingHelper::parse_numeric(new_val, line_num),
            ParsingType::FloatingPoint(_) => ParsingHelper::parse_floating(new_val, line_num),
            ParsingType::DateTypes(_) => self.parse_datetime(new_val, line_num),
        }?;
        Ok(Some(parsed_val))
    }

    /// This parses strings as datetimes given the setting of `self.date_helper`.
    fn parse_datetime(&self, new_val: &str, line_num: usize) -> Result<ParsingType, CsvPivotError> {
        let parsed_dt = match &self.date_helper {
            Some(helper) => helper.parse(new_val, line_num),
            None => Err(CsvPivotError::ParsingError {
                line_num, str_to_parse: new_val.to_string(),
                err: "Failed to parse datetime".to_string()
            }),
        }?;
        Ok(ParsingType::DateTypes(Some(parsed_dt)))
    }

    /// Parses cells as numeric (Decimal) types
    fn parse_numeric(new_val: &str, line_num: usize) -> Result<ParsingType, CsvPivotError> {
        let dec = Decimal::from_str(new_val)
            .or_else(|_| Decimal::from_scientific(&new_val.to_ascii_lowercase())) // infer scientific notation on error
            .or(Err(CsvPivotError::ParsingError {
                line_num, str_to_parse: new_val.to_string(),
                err: "Failed to parse as numeric type".to_string()
            }))?;
        Ok(ParsingType::Numeric(Some(dec)))
    }

    /// Parses cells as floating point types.
    fn parse_floating(new_val: &str, line_num: usize) -> Result<ParsingType, CsvPivotError> {
        let num: f64 = new_val.parse().or(Err(CsvPivotError::ParsingError {
            line_num, str_to_parse: new_val.to_string(),
            err: "Failed to parse floating point number".to_string()
        }))?;
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
        let scinot1 = ParsingHelper::parse_numeric("1e-4", 0);
        assert!(scinot1.is_ok());
        let scinot1_extract = match scinot1 {
            Ok(ParsingType::Numeric(Some(val))) => Ok(val.to_string()),
            Ok(_) => Ok("".to_string()),
            Err(e) => Err(e),
        }?;
        assert_eq!(scinot1_extract, "0.0001".to_string());
        let scinot2 = ParsingHelper::parse_numeric("1.3E4", 0);
        assert!(scinot2.is_ok());
        let scinot2_extract = match scinot2 {
            Ok(ParsingType::Numeric(Some(val))) => Ok(val.to_string()),
            Ok(_) => Ok("".to_string()),
            Err(e) => Err(e),
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
        let helper = ParsingHelper::from_parsing_type(ParsingType::DateTypes(None), false, false);
        for date in parsable_dates {
            let parsed_opt_date = helper.parse_val(date, 0)?;
            let parsed_date = match parsed_opt_date {
                Some(ParsingType::DateTypes(Some(val))) => val,
                // Since we're parsing from ParsingType::DateTypes, this should never happen
                _ => panic!(),
            };
            let naive_date = NaiveDate::from_ymd(2003, 1, 3);
            let naive_time = NaiveTime::from_hms(0, 0, 0);
            let expected_utc = NaiveDateTime::new(naive_date, naive_time);
            assert_eq!(parsed_date, expected_utc);
        }
        Ok(())
    }
}
