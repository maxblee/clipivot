//! This module serves as a way to convert string descriptions
//! of column names and delimiter names for CSV files into
//! settings `CliConfig` can use.
//!
//! The main reason -- the only reason, really -- this module is separated
//! from the main `aggregation` module and from CliConfig is that I expect
//! I'll wind up reusing this particular file in any future CSV CLIs I might build.

use errors::CsvPivotError;

/// A structure defining the basic settings of a CSV file. 
pub struct CsvSettings {
    /// The name of the CSV file
    filename: Option<String>,
    /// The column separator (e.g. '\t' for TSV files, ',' for CSV, etc.)
    delimiter: u8,
    /// Whether or not the CSV file has a field separator
    has_header: bool,
}

impl CsvSettings {
    /// Tries to create a new CSVSettings struct. Returns an error otherwise
    pub fn parse_new(filename: Option<&str>, delimiter: Option<&str>, has_header: bool) -> CsvCliResult<CsvSettings> {
        let delim = CsvSettings::parse_delimiter(&filename, delimiter)?;
    }

/// Parses the 1-byte value of a delimiter, for parsing as a CSV
/// Taking from the excellent `xsv` command-line CSV toolkit, this function automatically
/// assumes that `.tsv` and `.tab` files are tab-delimited, saving you the trouble of
/// adding a `-t` or `-d` flag. It will return an error if you try to pass a multi-character
/// string. 
/// 
/// **Note**, though, that what counts as a "character" for this function is really a single
/// byte, so single characters like 'त' will return errors here.
    fn parse_delimiter(filename: &Option<&str>, delimiter: Option<&str>) -> CsvCliResult<u8> {
        // Some(vec![u8]) if the user explicitly states a delimiter, None otherwise
        let explicit_delim = match delimiter {
            Some(r"\t") => vec![b'\t'],
            Some(val) => val.as_bytes().to_vec(),
            None => None
        };
        let expected_delim = match *filename {
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
        return Err(CsvPivotError::InvalidConfiguration(msg));
        }
        Ok(expected_delim[0])
    }
}