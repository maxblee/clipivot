//! This module is designed to determine the types of different fields
//! (e.g. date, number, etc.) in an unfamiliar CSV file.

/// An enum for describing the different kinds of data this program can currently parse.
/// Right now, it just deals with (or doesn't deal with, as the case may be) String
/// data. But eventually, I want it to handle:
/// * numbers (as decimal types)
/// * dates (using regex)
/// * boolean values
#[derive(Debug, PartialEq)]
pub enum ParsingType {
    /// String data. Defaults to this if the parser can't detect a consistent use
    /// of another type of data
    StringType,
}

/// The struct that I use to actually parse through the data
/// and the part of this module that interfaces with the main
/// `Aggregator` struct.
#[derive(Debug, PartialEq)]
pub struct ParsingHelper {
    values_type: ParsingType,
    possible_values: Vec<ParsingType>
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::StringType,
            possible_values: vec![ParsingType::StringType]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}