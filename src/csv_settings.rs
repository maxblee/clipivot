//! This module serves as a way to convert string descriptions
//! of column names and delimiter names for CSV files into
//! settings `CliConfig` can use.
//!
//! The main reason -- the only reason, really -- this module is separated
//! from the main `aggregation` module and from CliConfig is that I expect
//! I'll wind up reusing this particular file in any future CSV CLIs I might build.

use crate::errors::{CsvCliResult, CsvCliError};
use std::fs;
use std::io;

/// A structure defining the basic settings of a CSV file. 
#[derive(Debug, PartialEq)]
pub struct CsvSettings {
    /// The column separator (e.g. '\t' for TSV files, ',' for CSV, etc.)
    delimiter: u8,
    /// Whether or not the CSV file has a field separator
    has_header: bool,
}

impl Default for CsvSettings {
    fn default() -> CsvSettings {
        CsvSettings { delimiter: b',', has_header: true }
    }
}

impl CsvSettings {
    /// Tries to create a new CSVSettings struct. Returns an error otherwise
    pub fn parse_new(fname: &Option<&str>, delim: Option<&str>, has_header: bool) -> CsvCliResult<CsvSettings> {
        let delimiter = CsvSettings::parse_delimiter(&fname, delim)?;
        let settings = CsvSettings { delimiter, has_header };
        Ok(settings)
    }

    /// Returns a `csv::Reader` 
    pub fn get_reader_from_path(&self, filename: &str) -> csv::Result<csv::Reader<fs::File>> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_path(filename)
    }

    pub fn get_reader_from_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_reader(io::stdin())
    }

/// Parses the 1-byte value of a delimiter, for parsing as a CSV
/// Taking from the excellent `xsv` command-line CSV toolkit, this function automatically
/// assumes that `.tsv` and `.tab` files are tab-delimited, saving you the trouble of
/// adding a `-t` or `-d` flag. It will return an error if you try to pass a multi-character
/// string. 
/// 
/// **Note**, though, that what counts as a "character" for this function is really a single
/// byte, so single characters like 'त' will return errors here.
    fn parse_delimiter(fname: &Option<&str>, delim: Option<&str>) -> CsvCliResult<u8> {
        // Some(vec![u8]) if the user explicitly states a delimiter, None otherwise
        let explicit_delim = match delim {
            Some(r"\t") => Some(vec![b'\t']),
            Some(val) => Some(val.as_bytes().to_vec()),
            None => None
        };
        let expected_delim = match *fname {
            _ if explicit_delim.is_some() => explicit_delim.unwrap(),
             // altered from https://github.com/BurntSushi/xsv/blob/master/src/config.rs
            Some(fname) if fname.ends_with(".tsv") || fname.ends_with(".tab") => vec![b'\t'],
            _ => vec![b',']
        };
        if expected_delim.len() != 1 {
            let msg = format!(
                "Could not convert `{}` delimiter to a single ASCII character",
                String::from_utf8(expected_delim).unwrap()
            );
            return Err(CsvCliError::InvalidConfiguration(msg));
        }
        Ok(expected_delim[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use proptest::prelude::*;
    use clap::ArgMatches;

    // adapted from https://altsysrq.github.io/proptest-book/proptest/getting-started.html
    proptest! {
        #[test]
        fn delimiter_never_panics(s in "\\PC*") {
            let result = panic::catch_unwind(|| {
                let settings = CsvSettings::parse_new(&None, Some(&s), true);
            });
            assert!(result.is_ok());
        }
    }
}