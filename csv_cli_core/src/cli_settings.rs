use crate::errors::{CsvCliError, CsvCliResult};
use std::fs;
use std::io;
/// Defines some basic settings surrounding a CSV file.
///
/// This is designed to make it simple for me to create new command-line
/// interfaces for parsing CSVs. I built this specifically for the
/// command-line tool [I built](https://github.com/maxblee/clipivot) for
/// creating pivot tables. So far, I have not adapted the code at all,
/// except to change the documentation and change the names on some of the functions.
///
/// The basic idea is that you can use this tool to convert `ArgMatches`
/// from `Clap` (a command-line argument parser in Rust) into things that
/// are more convenient from a back-end logic perspective.
///
/// For instance, say you have a `Clap` app that looks like this:
/// ```ignore
/// extern crate clap;
/// use clap:::{Arg, App};
/// let app = App::new("CSV Program")
///                     .version("0.1.0")
///                     .author("Max Lee<maxbmhlee@gmail.com")
///                     .about("Selects fields from a CSV file")
///                     .arg(Arg::with_name("filename")
///                         .takes_value(true))
///                     .arg(Arg::with_name("delimiter")
///                         .takes_value(true))
///                     .arg(Arg::with_name("noheader"))
///                     .arg(Arg::with_name("fieldselect")
///                         .multiple(true));
/// let matches = app.get_matches();
/// ```
/// where "filename" refers to the name of your file (or None if you're reading from standard input),
/// "delimiter" refers to the single byte UTF-8 delimiter separating fields in each row,
/// "noheader" refers to whether or not your file has a header row,
/// and "fieldselect" refers to a list of fields from the CSV file that the user wants to do
/// something based off (more on that in a bit).
/// 
/// In order to create a `CsvSettings` object *and* validate your delimiter,
/// simply type
/// ```ignore
/// let filename = matches.value_of("filename");
/// let settings = CsvSettings::parse_new(
///          &filename,
///         matches.value_of("delimiter"),
///         !matches.is_present("noheader")
///     ).expect("Couldn't properly parse the delimiter");
/// ```
///
/// From there, you can easily create a `csv::Reader` object:
/// ```ignore
/// if filename.is_some() {
///     let mut rdr = settings.get_reader_from_path(&filename).expect("Couldn't read file");
/// } else {
///     let mut rdr = settings.get_reader_from_stdin();
/// }
/// ```
/// Finally, let's say you want to allow a user to select a list of fields from a CSV
/// file and do something based on the values of those fields. If the header row looks like this:
/// ```csv
/// col1,col2,col1,col3
/// ```
/// The user can enter one of the following things to select the first column:
/// - col1
/// - 0
/// - col1[0]
///
/// Or, to grab the third column, the user can type
/// - 2
/// - col1[1]
///
/// And finally, assuming the user is allowed to select multiple columns,
/// the user can type one of the following things to grab the first two columns in our file:
/// - --option col1,col2
/// - --option col1 col2
/// - --option col1 --option col2
/// 
/// In order to convert the user's selection into a list of unsigned integers
/// so your program can more easily retrieve the value of a given row at a column the user selected,
/// simply take one of the `csv::Reader` objects you created and type
/// ```ignore
/// let headers = rdr.headers().expect("Couldn't parse header");
/// let string_fields = matches.values_of("fieldselect")
///     .unwrap_or(vec![]);
/// let index_vec = settings.get_field_indexes(
///     &string_fileds, &headers.iter().collect()
/// ).expect("Couldn't parse one or more of the fields");
/// ```
/// Or, to select one column and return its index after parsing, type:
/// ```ignore
/// let colname = matches.value_of("selection").unwrap();
/// let col_idx = settings.get_field_index(colname, &headers.iter().collect());
/// ```
/// 
/// One final thing: When it comes to field selection, `CsvSettings` keeps in mind
/// whether or not you have a header row. If you don't, it will require that
/// all of your fields be unsigned integers between 0 and the total number of fields
/// in the first row of your CSV file. And because the `rdr.headers()` method
/// returns the first row of your file regardless of whether or not the file has a header row,
/// you don't need to change a line of code to get it to work.

#[derive(Debug, PartialEq)]
pub struct CsvSettings {
    /// The column separator (e.g. '\t' for TSV files, ',' for CSV, etc.)
    delimiter: u8,
    /// Whether or not the CSV file has a field separator
    has_header: bool,
}

impl Default for CsvSettings {
    fn default() -> CsvSettings {
        CsvSettings {
            delimiter: b',',
            has_header: true,
        }
    }
}

impl CsvSettings {
    /// Tries to create a new CSVSettings struct. Returns an error if it fails to parse the delimiter.
    /// (If this happens, it is likely because the delimiter **must be a single UTF-8 byte.**)
    pub fn parse_new(
        fname: &Option<&str>,
        delim: Option<&str>,
        has_header: bool,
    ) -> CsvCliResult<CsvSettings> {
        let delimiter = CsvSettings::parse_delimiter(&fname, delim)?;
        let settings = CsvSettings {
            delimiter,
            has_header,
        };
        Ok(settings)
    }

    /// Returns a `csv::Reader` object from a filepath, returning an error if the file doesn't exist.
    pub fn get_reader_from_path(&self, filename: &str) -> csv::Result<csv::Reader<fs::File>> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_path(filename)
    }

    /// Returns a `csv::Reader` object from standard input.
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
            None => None,
        };
        let expected_delim = match *fname {
            _ if explicit_delim.is_some() => explicit_delim.unwrap(),
            // altered from https://github.com/BurntSushi/xsv/blob/master/src/config.rs
            Some(fname) if fname.ends_with(".tsv") || fname.ends_with(".tab") => vec![b'\t'],
            _ => vec![b','],
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

    /// Returns a single index where a single string appears. Allows you to validate a single column, rather
    /// than multiple columns.
    pub fn get_field_index(
        &self,
        colname: &str,
        headers: &Vec<&str>,
    ) -> CsvCliResult<usize> {
        let infered_num = match self.get_numeric_index(&colname) {
            Some(num) if num < headers.len() => Ok(Some(num)),
            Some(_num) => Err(CsvCliError::InvalidConfiguration(format!(
                "Could not properly configure. Column selection needs to be between 0 and `{}`",
                headers.len()
            ))),
            None if !self.has_header => Err(CsvCliError::InvalidConfiguration(
                "Columns must be numeric if you don't have a header".to_string(),
            )),
            None => Ok(None),
        }?;
        // TODO There's probably a way to handle this with combinators
        if infered_num.is_some() {
            return Ok(infered_num.unwrap());
        }
        let str_idx = self.get_string_index(&colname, headers)?;
        Ok(str_idx)
    }

    /// Given a vector of column descriptions, returns indexes where they appear
    /// You can see a more complete description on [GitHub](https://www.github.com/maxblee/clipivot),
    /// but at a basic level, the idea of this function is to allow users to
    /// describe columns either by their names or by their indexes.
    pub fn get_field_indexes(
        &self,
        user_defs: &Vec<&str>,
        headers: &Vec<&str>,
    ) -> CsvCliResult<Vec<usize>> {
        let mut output_vec = Vec::new();
        for user_input in user_defs {
            let all_cols = self.split_arg_string(user_input);
            for colname in all_cols {
                let idx = self.get_field_index(&colname, headers)?;
                output_vec.push(idx);
            }
        }
        Ok(output_vec)
    }

    fn split_arg_string(&self, combined_cols: &str) -> Vec<String> {
        let mut split_strings = Vec::new();
        // quote_char represents whether or not we're inside quotes
        // None => not inside quotes; Some('\'') => inside single quote,
        // Some('\"') => inside double quotes
        let mut quote_char = None;
        let mut current_splice = String::new();
        for c in combined_cols.chars() {
            if quote_char.is_none() {
                if c == '\'' || c == '\"' {
                    quote_char = Some(c);
                } else if c == ',' {
                    split_strings.push(current_splice);
                    current_splice = String::new();
                    continue;
                }
            } else if (c == '\'' || c == '\"') && (Some(c) == quote_char) {
                quote_char = None;
            }
            current_splice.push(c);
        }
        if !(current_splice.is_empty()) {
            split_strings.push(current_splice);
        }
        split_strings
    }

    fn get_numeric_index(&self, colname: &str) -> Option<usize> {
        // ignore leading whitespace
        let parsed_str = colname.trim();
        // because of `unwrap` at the end here, we need to check for empty string
        if parsed_str == "" {
            return None;
        }
        for char in parsed_str.chars() {
            if !(char.is_ascii_digit()) {
                return None;
            }
        }
        Some(parsed_str.parse().unwrap())
    }


    fn get_string_index(&self, colname: &str, headers: &Vec<&str>) -> CsvCliResult<usize> {
        // same implementation here as in `split_arg_string`
        let mut quote_char = None;
        let mut in_brackets = false;
        // the name we expect the field to be based on this function
        let mut expected_header = String::new();
        // If there are multiple fields with the same name, the order we expect this one appears in
        let mut expected_order = String::new();
        // Trim string because of CSV reader settings
        let trimmed_str = colname.trim();
        for c in trimmed_str.chars() {
            if quote_char.is_none() {
                if in_brackets {
                    if c != ']' {
                        // append every character, even if we've passed the closing bracket
                        expected_order.push(c);
                    }
                } else if c == '\'' || c == '\"' {
                    quote_char = Some(c);
                } else if c != '[' {
                    expected_header.push(c);
                } else {
                    in_brackets = true;
                }
            } else {
                if (c == '\'' || c == '\"') && Some(c) == quote_char {
                    quote_char = None;
                    continue;
                }
                expected_header.push(c);
            }
        }
        // TODO Figure out the best way to handle this; deserializing and reserializing isn't great
        if expected_order.is_empty() {
            expected_order = "0".to_string();
        }
        let order = expected_order
            .parse::<usize>()
            .or_else(|_| Err(CsvCliError::InvalidConfiguration(format!(
                "Could not convert column name `{}`. Hint: consider enclosing the column in quotes",
                colname
            ))))?;
        self.find_index_from_expected(&expected_header, order, headers)
    }

    fn find_index_from_expected(
        &self,
        expected_header: &str,
        expected_order: usize,
        headers: &Vec<&str>,
    ) -> CsvCliResult<usize> {
        let mut count = 0;
        for (i, field) in headers.iter().enumerate() {
            if &expected_header == field {
                if count == expected_order {
                    return Ok(i);
                }
                count += 1;
            }
        }
        Err(CsvCliError::InvalidConfiguration(format!(
            "Could not find `{}` in header row",
            expected_header
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::panic;

    // adapted from https://altsysrq.github.io/proptest-book/proptest/getting-started.html
    proptest! {
        #[test]
        fn delimiter_never_panics(s in "\\PC*") {
            let result = panic::catch_unwind(|| {
                let _settings = CsvSettings::parse_new(&None, Some(&s), true);
            });
            assert!(result.is_ok());
        }

        #[test]
        fn split_header_never_panics(s in "\\PC*") {
            let settings = CsvSettings::default();
            let result = panic::catch_unwind(|| {
                let _valid = settings.split_arg_string(&s);
            });
            assert!(result.is_ok());
        }

        #[test]
        fn numeric_index_never_panics(s in "\\PC*") {
            let settings = CsvSettings::default();
            let result = panic::catch_unwind(|| {
                let _valid = settings.get_numeric_index(&s);
            });
            assert!(result.is_ok());
        }

        #[test]
        fn leading_whitespace_parses(s in " [0-9]\t") {
            let settings = CsvSettings::default();
            assert!(settings.get_numeric_index(&s).is_some());
        }

        #[test]
        fn nums_correctly_parse(n: usize) {
            let settings = CsvSettings::default();
            assert_eq!(settings.get_numeric_index(&n.to_string()), Some(n));
        }

        #[test]
        fn string_index_never_panics(s in "\\PC*") {
            let settings = CsvSettings::default();
            let sample_header = vec!["hello"];
            let result = panic::catch_unwind(|| {
                let _valid = settings.get_string_index(&s, &sample_header);
            });
            assert!(result.is_ok());
        }

        // Using [A-Z] to avoid brackets etc
        fn matching_strings_get_idx(s in "[A-Za-z]") {
            let header = vec![s.as_ref()];
            let settings = CsvSettings::default();
            assert_eq!(settings.get_string_index(&s, &header).unwrap(), 0);
            let header = vec![s.as_ref(), s.as_ref()];
            let new_str = format!("{}{}", s, "[1]".to_string());
            assert_eq!(settings.get_string_index(&new_str, &header).unwrap(), 1);
        }
    }

    #[test]
    fn test_split_single_str() {
        let settings = CsvSettings::default();
        assert_eq!(
            settings.split_arg_string("FIELDNAME"),
            vec!["FIELDNAME".to_string()]
        );
        assert_eq!(
            settings.split_arg_string("\'FIELDNAME\'"),
            vec!["\'FIELDNAME\'".to_string()]
        );
        assert_eq!(
            settings.split_arg_string("\'FIELDNAME,a\'"),
            vec!["\'FIELDNAME,a\'".to_string()]
        );
        assert_eq!(
            settings.split_arg_string("\'FIELDNAME,a\',a"),
            vec!["\'FIELDNAME,a\'".to_string(), "a".to_string()]
        );
        assert_eq!(
            settings.split_arg_string("a,b"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            settings.split_arg_string("\"FIELDNAME\',a"),
            vec!["\"FIELDNAME\',a".to_string()]
        );
    }

    #[test]
    fn test_str_indexes() {
        let header = vec![
            "FIELDNAME1",
            "FIELDNAME2",
            "FIELDNAME1",
            "FIELDNAME2[0]",
            "FIELDNAME2[0]",
            "",
        ];
        let settings = CsvSettings::default();
        assert_eq!(settings.get_string_index("FIELDNAME1", &header).unwrap(), 0);
        assert!(settings.get_string_index("BLABLABLA", &header).is_err());
        assert_eq!(
            settings.get_string_index("FIELDNAME1[1]", &header).unwrap(),
            2
        );
        assert_eq!(
            settings
                .get_string_index("'FIELDNAME2[0]'", &header)
                .unwrap(),
            3
        );
        assert_eq!(
            settings
                .get_string_index("'FIELDNAME2[0]'[1]", &header)
                .unwrap(),
            4
        );
        assert!(settings
            .get_string_index("'FIELDNAME2[0]'[2]", &header)
            .is_err());
        assert!(settings
            .get_string_index("FIELDNAME2[0][0]", &header)
            .is_err());
    }

    #[test]
    // makes sure that 'Nd' and other numerical unicode characters that are not [0-9] fail to parse
    fn test_non_ascii_unicode_digits_fail_numeric_parsing() {
        // sampled from http://www.fileformat.info/info/unicode/category
        let invalid_n_chars = vec!["۳", "᠐", "ᛯ", "Ⅿ", "¼", "౸"];
        let settings = CsvSettings::default();
        for inv_char in invalid_n_chars {
            assert!(settings.get_numeric_index(inv_char).is_none());
        }
    }

    #[test]
    fn test_empty_numeric_index_doesnt_parse() {
        let settings = CsvSettings::default();
        assert!(settings.get_numeric_index("").is_none());
    }

    #[test]
    fn test_no_header_doesnt_parse() {
        let no_header_set = CsvSettings::parse_new(&None, None, false).unwrap();
        let header_row = vec!["a", "b"];
        assert!(no_header_set
            .get_field_indexes(&vec!["a"], &header_row)
            .is_err());
    }
}
