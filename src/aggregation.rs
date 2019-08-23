//! The `aggregation` module is part of `csvpivot` that works directly with command-line arguments.
//!
//! Once you pass command-line arguments to the program, the `run` method in this module creates a
//! `CliConfig` object. The `CliConfig` object then creates an `Aggregator` object. That `Aggregator` object
//! then holds the data you're aggregating, passing off parsing and computational tasks
//! to the `aggfunc` and `parsing` modules. Then, the `run` method calls `CliConfig`'s `run_config` method,
//! which runs a final validation of your command-line arguments before telling the aggregator to aggregate the
//! CSV file, line by line. Upon completion, the `Aggregator` object also writes your results to standard output.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

use crate::aggfunc::*;
use crate::errors::{CsvCliResult, CsvCliError};
use crate::parsing::{ParsingHelper, ParsingType};
use clap::ArgMatches;

const FIELD_SEPARATOR: &str = "_<sep>_";

/// The main struct for aggregating CSV files
#[derive(Debug, PartialEq)]
pub struct Aggregator<T>
where
    T: AggregationMethod,
{
    /// Holds the aggregations, mapping (row, column) matches to an object implementing the
    /// `AggregationMethod` trait, like `Count`.
    aggregations: HashMap<(String, String), T>,
    /// Holds all of the unique row names
    indexes: HashSet<String>,
    /// Holds the unique column names
    columns: HashSet<String>,
    /// Determines how new records are aggregated. See [this](../parsing/index.html)
    /// for details.
    parser: ParsingHelper,
    /// A vector of columns the user is using for determining the row names of the pivot table
    index_cols: Vec<usize>,
    /// A vector of columns for determining the columns of the pivot table
    column_cols: Vec<usize>,
    /// The column that determines the values of each cell in the pivot table
    values_col: usize,
}

impl<T: AggregationMethod> Aggregator<T> {
    pub fn new() -> Aggregator<T> {
        Aggregator {
            aggregations: HashMap::new(),
            indexes: HashSet::new(),
            columns: HashSet::new(),
            parser: ParsingHelper::default(),
            index_cols: Vec::new(),
            column_cols: Vec::new(),
            values_col: 0,
        }
    }

    /// Creates a new `Aggregator` object from a `ParsingHelper` object.
    /// This is used to initialize the `Aggregator` within `CliConfig`
    pub fn from_parser(parser: ParsingHelper) -> Aggregator<T> {
        let mut agg = Aggregator::new();
        agg.parser = parser;
        agg
    }

    /// Sets the indexes of the `Aggregator`
    pub fn set_indexes(&mut self, new_indexes: Vec<usize>) -> &mut Aggregator<T> {
        self.index_cols = new_indexes;
        self
    }

    /// Adds the list of columns to the aggregator
    pub fn set_columns(&mut self, new_cols: Vec<usize>) -> &mut Aggregator<T> {
        self.column_cols = new_cols;
        self
    }

    /// Adds the column where the aggregation type is applied.
    ///
    /// For instance, if you decided to `sum` a bunch of salaries
    /// based on two columns, you would use this function to
    /// set the value column to the 'salaries' column.
    ///
    /// I've purposefully allowed users to only use a single value
    /// column. This contrasts with Excel, which allows for multiple values columns.
    ///
    /// As a tool designed for data exploration, I feel that users should limit themselves
    /// to a single aggregation method. Users can take a different approach
    /// by joining the data from one pivot table output to the data from another pivot table output.
    pub fn set_value_column(&mut self, value_col: usize) -> &mut Aggregator<T> {
        self.values_col = value_col;
        self
    }

    /// Takes a CSV reader object from a file path and adds records, row by row.
    /// Returns an error if it can't read any of the records.
    ///
    /// This can either happen because of a problem in how the CSV
    /// was formatted or because the values/columns/indexes
    /// handed to the aggregator from the command line refer to
    /// fields that do not exist.
    ///
    /// Additionally, the aggregator currently only supports valid UTF-8
    /// data, so it won't work on all CSV files. I'd eventually like to support
    /// all ASCII data.
    pub fn aggregate_from_file(
        &mut self,
        mut rdr: csv::Reader<fs::File>,
    ) -> CsvCliResult<()> {
        let mut iter = rdr.into_records();
        for (line_num, result) in iter.enumerate() {
            let record = result?;
            self.add_record(record, line_num)?;
        }
        Ok(())
    }

    /// Takes records from standard input and aggregates them row by row. The code here is identical to
    /// the code in the `aggregate_from_file` function, because the CSV reader
    /// from handling files is different from the reader for handling standard input.
    ///
    /// In the spirit of DRY, I'm open to suggestions for refactoring this code. But
    /// it's not really pressing, since we're talking about 5-ish lines of code.
    pub fn aggregate_from_stdin(
        &mut self,
        mut rdr: csv::Reader<io::Stdin>,
    ) -> CsvCliResult<()> {
        let mut iter = rdr.into_records();
        // we pass line_num here as a way of making for better error handling
        for (line_num, result) in iter.enumerate() {
            let record = result?;
            self.add_record(record, line_num)?;
        }
        Ok(())
    }

    /// Once I've added all of the records to the dataset, I use this method to
    /// write them to standard output. The function adds a header based on all of the unique
    /// strings appearing in the columns column. Then, it parses the data, cell by cell
    /// and writes the data, row by row, to standard output.
    pub fn write_results(&self) -> CsvCliResult<()> {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        let mut header = vec![""];
        for col in &self.columns {
            header.push(col);
        }
        wtr.write_record(header)?;
        for row in &self.indexes {
            let mut record = vec![row.to_string()];
            for col in &self.columns {
                let output = self.parse_writing(row, col);
                record.push(output);
            }
            wtr.write_record(record)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// This method parses a given cell, outputting it as a string so the CSV
    /// writer can write the data to standard output.
    fn parse_writing(&self, row: &str, col: &str) -> String {
        let aggval = self.aggregations.get(&(row.to_string(), col.to_string()));
        match aggval {
            Some(agg) => agg.to_output(),
            None => "".to_string(),
        }
    }

    /// Adds a new record (row) to the aggregator.
    fn add_record(&mut self, record: csv::StringRecord, line_num: usize) -> CsvCliResult<()> {
        // merges all of the index columns into a single column, separated by FIELD_SEPARATOR
        let indexnames = self.get_colname(&self.index_cols, &record);
        let columnnames = self.get_colname(&self.column_cols, &record);
        // CliConfig + csv crate do error handling that should prevent get from being None
        let str_val = record
            .get(self.values_col).unwrap();
        // This isn't memory efficient, but it should be OK for now
        // (i.e. I should eventually get self.indexes and self.columns
        // be tied to self.aggregations, rather than cloned)
        // My thought here is that the contains check is probably faster than the close but not sure***
        if !(self.columns.contains(&columnnames)) {
            self.columns.insert(columnnames.clone());
        }
        if !(self.indexes.contains(&indexnames)) {
            self.indexes.insert(indexnames.clone());
        }

        let parsed_val = self.parser.parse_val(str_val, line_num)?;
        // this determines how to add the data as it's being read
        if parsed_val.is_some() {
            self.update_aggregations(indexnames, columnnames, &parsed_val.unwrap());
        }
        Ok(())
    }

    /// Converts a vector of column indexes into a String. Used as a way to eliminate code duplication
    /// for the conversion of cells into values for the rows and columns of the final pivot table.
    fn get_colname(
        &self,
        columns: &[usize],
        record: &csv::StringRecord,
    ) -> String {
        let mut colnames: Vec<&str> = Vec::new();
        if columns.is_empty() {
            return "total".to_string();
        }
        for idx in columns {
            // unwrap should be safe bc CliConfig + csv crate error handling should prevent
            // record.get(idx) == None
            let idx_column = record.get(*idx).unwrap();
            colnames.push(idx_column);
        }
        colnames.join(FIELD_SEPARATOR)
    }

    /// This function only runs when `ParsingHelper` has returned a value that is neither
    /// an error nor a `None` (empty) value. In this case, this actually adds records into the aggregator.
    fn update_aggregations(
        &mut self,
        indexname: String,
        columnname: String,
        parsed_val: &ParsingType,
    ) {
        // modified from
        // https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations
            .entry((indexname, columnname))
            .and_modify(|val| val.update(parsed_val))
            .or_insert_with(|| T::new(parsed_val));
    }
}

/// `parse_delimiter` converts `ArgMatches` from the command-line into a delimiter that
/// it intends to use to parse CSV values with. For instance, tsv files have a delimiter of `'\t'`.
///
/// Taking from the excellent `xsv` command-line CSV toolkit, this function automatically
/// assumes that `.tsv` and `.tab` files are tab-delimited, saving you the trouble of
/// adding a `-t` or `-d` flag. It will return an error if you try to pass a multi-character
/// string. 
/// 
/// **Note**, though, that what counts as a "character" for this function is really a single
/// byte, so single characters like 'त' will return errors here.
fn parse_delimiter(filename: &Option<&str>, arg_matches: &ArgMatches) -> CsvCliResult<u8> {
    let default_delim = match filename {
        _ if arg_matches.is_present("tab") => vec![b'\t'],
        _ if arg_matches.is_present("delim") => {
            let delim = arg_matches.value_of("delim").unwrap();
            if let r"\t" = delim {
                vec![b'\t']
            } else {
                delim.as_bytes().to_vec()
            }
        }
        // altered from https://github.com/BurntSushi/xsv/blob/master/src/config.rs
        Some(fname) if fname.ends_with(".tsv") || fname.ends_with(".tab") => vec![b'\t'],
        _ => vec![b','],
    };
    if default_delim.len() != 1 {
        let msg = format!(
            "Could not convert `{}` delimiter to a single ASCII character",
            String::from_utf8(default_delim).unwrap()
        );
        return Err(CsvCliError::InvalidConfiguration(msg));
    }
    Ok(default_delim[0])
}

/// This struct is intended for converting from Clap's `ArgMatches` to the `Aggregator` struct
#[derive(Debug, PartialEq)]
pub struct CliConfig<U>
where
    U: AggregationMethod,
{
    /// The relative (or actual) path to a CSV file. Is None when reading from standard input.
    // set as an option so I can handle standard input
    filename: Option<String>,
    /// `CliConfig` creates an `Aggregator` object to run the aggregations and hold onto the aggregated data.
    aggregator: Aggregator<U>,
    /// Whether or not you want to read the file with headers. Defaults to true.
    has_header: bool,
    /// The delimiter separating fields in your CSV file. Defaults to '\t' for `.tsv` or `.tab` files, ',' otherwise.
    delimiter: u8,
    /// The name of the column you're running the aggregation function on.
    values_col: String,
    /// The name of the column(s) (or indexes) forming the columns of the final pivot table. vec![] if no columns.
    column_cols: Vec<String>,
    /// The name of the column(s) (or indexes) forming the indexes of the final pivot table. 
    indexes: Vec<String>,
}

impl<U: AggregationMethod> CliConfig<U> {
    /// Creates a new, basic CliConfig
    pub fn new() -> CliConfig<U> {
        CliConfig {
            filename: None,
            aggregator: Aggregator::new(),
            has_header: true,
            delimiter: b',',
            values_col: "".to_string(),
            column_cols: vec![],
            indexes: vec![],
        }
    }
    /// Takes argument matches from main and tries to convert them into CliConfig
    pub fn from_arg_matches(arg_matches: ArgMatches) -> CsvCliResult<CliConfig<U>> {
        let base_config: CliConfig<U> = CliConfig::new();
        let values_col = arg_matches.value_of("value").unwrap().to_string(); // unwrap safe because required arg
        let column_cols = arg_matches
            .values_of("columns")
            .map_or(vec![], |it| it.map(|val| val.to_string()).collect());
        let indexes = arg_matches
            .values_of("rows")
            .map_or(vec![], |it| it.map(|val| val.to_string()).collect());
        let filename = arg_matches.value_of("filename");
        // TODO This is hacky
        let parser = base_config.get_parser(&arg_matches);
        let aggregator: Aggregator<U> = Aggregator::from_parser(parser);

        let delimiter = parse_delimiter(&filename, &arg_matches)?;

        let cfg = CliConfig {
            filename: filename.map(String::from),
            aggregator,
            has_header: !arg_matches.is_present("noheader"),
            delimiter,
            values_col,
            indexes,
            column_cols,
        };
        Ok(cfg)
    }
    /// Determines how to parse data, depending on the type of function you're running and command-line flags.
    fn get_parsing_approach(&self, parse_numeric: bool, parse_date: bool) -> ParsingType {
        match U::get_aggtype() {
            AggTypes::Count => ParsingType::Text(None),
            AggTypes::CountUnique => ParsingType::Text(None),
            AggTypes::Mode => ParsingType::Text(None),
            AggTypes::Mean => ParsingType::Numeric(None),
            AggTypes::Median => ParsingType::Numeric(None),
            AggTypes::Sum => ParsingType::Numeric(None),
            AggTypes::StdDev => ParsingType::FloatingPoint(None),
            AggTypes::Minimum if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Maximum if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Range if parse_numeric => ParsingType::Numeric(None),
            AggTypes::Maximum if parse_date => ParsingType::DateTypes(None),
            AggTypes::Minimum if parse_date => ParsingType::DateTypes(None),
            AggTypes::Range => ParsingType::DateTypes(None),
            AggTypes::Minimum => ParsingType::Text(None),
            AggTypes::Maximum => ParsingType::Text(None),
        }
    }

    /// Gets the `ParsingHelper` object for `Aggregator` based on command-line arguments.
    fn get_parser(&self, arg_matches: &ArgMatches) -> ParsingHelper {
        let parse_numeric = arg_matches.is_present("numeric");
        let infer_date = arg_matches.is_present("infer");
        let dayfirst = arg_matches.is_present("dayfirst");
        let yearfirst = arg_matches.is_present("yearfirst");
        let parse_type = self.get_parsing_approach(parse_numeric, infer_date);
        ParsingHelper::from_parsing_type(parse_type, dayfirst, yearfirst)
            .parse_empty_vals(!arg_matches.is_present("empty"))
    }
    /// Converts from a file path to either a CSV reader or a CSV error.
    pub fn get_reader_from_path(&self) -> Result<csv::Reader<fs::File>, csv::Error> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            // this function is only run if self.filename.is_some() so unwrap() is fine
            .from_path(self.filename.as_ref().unwrap())
    }

    /// Converts from standard input to a CSV reader.
    pub fn get_reader_from_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_reader(io::stdin())
    }

    /// Given a string (passed through the command line), this function returns an index for that field
    /// within the header of the CSV file. If the CSV file doesn't have a header, every String argument
    /// must be a string number.
    fn get_header_idx(&self, colname: &str, headers: &[&str]) -> CsvCliResult<usize> {
        // Without making an explicit comparison to empty, `csvpivot` will panic on an empty string
        if colname == "" {
            return headers.iter().position(|&x| x == "" ).ok_or(CsvCliError::InvalidConfiguration("Could not parse the fieldname \"\"".to_string()));
        }
        let mut in_quotes = false;
        let mut order_specification = false; // True if we've passed a '['
                                             // fieldname occurrence is the order in which a field occurs. Defaults to 0.
        let mut fieldname_occurrence: String = "".to_string();
        let mut occurrence_start = 0;
        let mut occurrence_end = 0;
        let header_length = headers.len();
        let mut all_numeric = true; // default to reading the field as a 0-indexed number
        let chars = colname.chars();
        if self.has_header {
            let mut count = 0;
            for (i, c) in chars.enumerate() {
                if !(c.is_ascii_digit()) {
                    all_numeric = false;
                }
                if (c == '\'' || c == '\"') && !(in_quotes) {
                    in_quotes = true;
                    continue;
                }
                if (c == '\'' || c == '\"') && in_quotes {
                    in_quotes = false;
                    continue;
                }
                if in_quotes {
                    continue;
                }
                if c == '[' && !(in_quotes) {
                    order_specification = true;
                    if i + 1 < colname.len() {
                        occurrence_start = i + 1;
                    }
                    continue;
                }
                if c == ']' {
                    occurrence_end = i;
                    continue;
                }
                if order_specification {
                    if !(c.is_ascii_digit()) {
                        let msg = format!(
                            "Could not parse the fieldname {}. You may need to encapsulate the field in quotes",
                            colname
                        );
                        return Err(CsvCliError::InvalidConfiguration(msg));
                    }
                    fieldname_occurrence.push_str(&c.to_string());
                }
            }
        }
        if all_numeric {
            // Unwrap should be fine because we know that every character is an ASCII digit
            // and .parse() automatically removes leading zeros
            // AND (this is key) clap should prevent colname from ever being an empty string
            let parsed_val: usize = colname.parse().unwrap();
            if parsed_val >= header_length {
                let msg = format!(
                    "Column selection must be between
                0 <= selection < {}",
                    header_length
                );
                Err(CsvCliError::InvalidConfiguration(msg))
            } else {
                Ok(parsed_val)
            }
        } else if order_specification {
            let orig_end = match occurrence_start {
                i if i >= 1 => Ok(i - 1),
                _i => Err(CsvCliError::InvalidConfiguration(
                    "Couldn't parse field.".to_string(),
                )),
            }?;
            let orig_name = &colname[..orig_end];
            let parsed_val = colname[occurrence_start..occurrence_end].parse::<usize>()
                .or(Err(CsvCliError::InvalidConfiguration(
                    "Fieldnames with brackets must be in quotes or have at least one ASCII digit within the brackets (e.g. FIELDNAME[0]".to_string()
                    )))?;
            let mut count = 0;
            for (i, field) in headers.iter().enumerate() {
                if field == &orig_name {
                    count += 1;
                    if count == parsed_val + 1 {
                        {
                            return Ok(i);
                        }
                    }
                }
            }
            let msg = format!(
                "There are only {} occurrences of the fieldname {}",
                count, orig_name
            );
            Err(CsvCliError::InvalidConfiguration(msg))
        } else {
            // csv crate automatically trims quotes, so headers will be trimmed
            match headers.iter().position(|&i| i == colname.replace("\"", "")) {
                Some(position) => {
                    Ok(position)
                }
                None => {
                    let msg = format!("Could not find the fieldname `{}` in the header", colname);
                    Err(CsvCliError::InvalidConfiguration(msg))
                }
            }
        }
    }

    /// Takes a string from the command line and combines it into a vector of strings
    /// for `validate_columns` to convert into `usize` indexes.
    fn parse_combined_col(&self, combined_name: &str) -> CsvCliResult<Vec<String>> {
        let mut output_vec = Vec::new();
        let mut in_quotes = false;
        let mut cur_string = String::new();
        // Option<char> where None == No previous quote, Some('\'' == previous single quote)
        let mut prev_quote = None;
        let mut last_parsed = None;
        for c in combined_name.chars() {
            last_parsed = Some(c);
            if !in_quotes {
                if (c == '\'' || c == '\"') && cur_string.is_empty() {
                    prev_quote = Some(c);
                    in_quotes = true;
                    cur_string.push(c);
                    continue;
                }
                if c == ',' {
                    output_vec.push(cur_string);
                    cur_string = String::new();
                } else {
                    cur_string.push(c);
                }
            } else if c == '\'' || c == '\"' {
                if prev_quote != Some(c) {
                    return Err(CsvCliError::InvalidConfiguration(
                        "Quotes inside fieldname were not properly closed".to_string()
                    ));
                }
                in_quotes = false;
                cur_string.push(c);
                continue;
            } else {
                cur_string.push(c);
            }
        }
        if !cur_string.is_empty() {
            output_vec.push(cur_string);
        }
        if in_quotes {
            return Err(CsvCliError::InvalidConfiguration(
                "Quotes inside fieldname were not properly closed".to_string(),
            ));
        }
        if last_parsed == Some(',') {
            return Err(CsvCliError::InvalidConfiguration(
                "One of the fieldnames ends with an unquoted comma".to_string(),
            ));
        }
        Ok(output_vec)
    }

    /// Takes a vector of column names and converts them into a vector of Strings.
    /// This is effectively designed to allow us to split Strings like `a,b` into
    /// an Vector `vec!["a","b"]` while maintaining the String `'a,b' with quotes in it
    /// as a single-item vector `vec!["\'a,b\'"].
    fn get_multiple_header_columns(
        &self,
        colnames: &[String],
    ) -> CsvCliResult<Vec<String>> {
        let mut expected_columns = Vec::new();
        for col in colnames {
            let parsed_col = self.parse_combined_col(&col)?;
            expected_columns.extend(parsed_col);
        }
        Ok(expected_columns)
    }

    /// Takes a vector of columns and expected header rows from get_multiple_header_columns
    /// and converts them into a vector of indexes (helping `Aggregator` determine
    /// which columns you should parse values from).
    fn get_idx_vec(
        &self,
        expected_cols: &[String],
        headers: &[&str],
    ) -> CsvCliResult<Vec<usize>> {
        let mut all_cols = Vec::new();
        for col in expected_cols {
            let col_idx = self.get_header_idx(&col, headers)?;
            all_cols.push(col_idx);
        }
        let mut parsed_cols = HashSet::new();
        let mut output_cols = Vec::new();
        for col in all_cols {
            if !parsed_cols.contains(&col) {
                output_cols.push(col);
            } else {
                parsed_cols.insert(col);
            }
        }
        Ok(output_cols)
    }

    /// Validates the columns you enter with the `-c`/`cols`, `-v`/`--val`, and `-r`/`--rows`,
    /// and updates the `Aggregator` object so we can run aggregations.
    // Note: the below function won't compile without taking only &Vec<&str> because of Iter::collect
    #[allow(clippy::ptr_arg)]
    fn validate_columns(&mut self, headers: &Vec<&str>) -> CsvCliResult<()> {
        // validates the aggregation columns and then updates the aggregator
        let expected_indexes = self.get_multiple_header_columns(&self.indexes)?;
        let index_vec = self.get_idx_vec(&expected_indexes, headers)?;
        let expected_columns = self.get_multiple_header_columns(&self.column_cols)?;
        let column_vec = self.get_idx_vec(&expected_columns, headers)?;
        let values_vec = self.get_header_idx(&self.values_col, headers)?;
        // need self.aggregator = .. right now bc set_indexes etc return Self (rather than mutating)
        // TODO clean up method chaining to avoid this mess
        self.aggregator
            .set_indexes(index_vec)
            .set_columns(column_vec)
            .set_value_column(values_vec);
        Ok(())
    }

    /// Runs the `Aggregator` for the given type.
    pub fn run_config(&mut self) -> CsvCliResult<()> {
        if self.filename.is_some() {
            let mut rdr = self.get_reader_from_path()?;
            let headers = rdr.headers()?;
            self.validate_columns(&headers.iter().collect())?;
            self.aggregator.aggregate_from_file(rdr)?;
        } else {
            let mut rdr = self.get_reader_from_stdin();
            let headers = rdr.headers()?;
            self.validate_columns(&headers.iter().collect())?;
            self.aggregator.aggregate_from_stdin(rdr)?;
        }
        self.aggregator.write_results()?;
        Ok(())
    }
}

/// This function is the part of the program that directly interacts with `main`.
pub fn run(arg_matches: ArgMatches) -> CsvCliResult<()> {
    let aggfunc = arg_matches.value_of("aggfunc").unwrap();
    if aggfunc == "count" {
        let mut config: CliConfig<Count> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "countunique" {
        let mut config: CliConfig<CountUnique> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "mode" {
        let mut config: CliConfig<Mode> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "mean" {
        let mut config: CliConfig<Mean> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "sum" {
        let mut config: CliConfig<Sum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "median" {
        let mut config: CliConfig<Median> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "stddev" {
        let mut config: CliConfig<StdDev> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "min" {
        let mut config: CliConfig<Minimum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "max" {
        let mut config: CliConfig<Maximum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "range" {
        let mut config: CliConfig<Range> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use proptest::prelude::*;

    fn setup_simple_record() -> csv::StringRecord {
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        csv::StringRecord::from(record_vec)
    }

    fn setup_simple_count() -> Aggregator<Count> {
        let mut agg: Aggregator<Count> = Aggregator {
            aggregations: HashMap::new(),
            indexes: HashSet::new(),
            columns: HashSet::new(),
            parser: ParsingHelper::default(),
            index_cols: vec![0, 1],
            column_cols: vec![2, 3],
            values_col: 4,
        };
        agg.add_record(setup_simple_record(), 0).unwrap();
        agg
    }

    fn setup_multiple_counts() -> Aggregator<Count> {
        let mut agg = setup_simple_count();
        let second_vec = vec!["Nashville", "TN", "Predators", "Hockey", "Playoffs"];
        let second_record = csv::StringRecord::from(second_vec);
        agg.add_record(second_record, 0).unwrap();
        let third_vec = vec!["Nashville", "TN", "Titans", "Football", "Bad"];
        let third_record = csv::StringRecord::from(third_vec);
        agg.add_record(third_record, 0).unwrap();
        let fourth_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Bad"];
        let fourth_record = csv::StringRecord::from(fourth_vec);
        agg.add_record(fourth_record, 0).unwrap();
        agg
    }

    fn setup_one_liners() -> CliConfig<Count> {
        let agg: Aggregator<Count> = Aggregator {
            aggregations: HashMap::new(),
            indexes: HashSet::new(),
            columns: HashSet::new(),
            parser: ParsingHelper::default(),
            index_cols: vec![2],
            column_cols: vec![1],
            values_col: 0,
        };
        CliConfig {
            filename: Some("test_csvs/one_liner.csv".to_string()),
            aggregator: agg,
            has_header: true,
            delimiter: b',',
            values_col: "0".to_string(),
            column_cols: vec!["1".to_string()],
            indexes: vec!["2".to_string()],
        }
    }

    fn setup_config() -> CliConfig<Count> {
        let agg: Aggregator<Count> = Aggregator {
            aggregations: HashMap::new(),
            indexes: HashSet::new(),
            columns: HashSet::new(),
            parser: ParsingHelper::default(),
            index_cols: vec![3],
            column_cols: vec![1],
            values_col: 0,
        };
        CliConfig {
            filename: Some("test_csvs/layoffs.csv".to_string()),
            aggregator: agg,
            has_header: true,
            delimiter: b',',
            values_col: "0".to_string(),
            column_cols: vec!["1".to_string()],
            indexes: vec!["3".to_string()],
        }
    }

    // adapted from https://altsysrq.github.io/proptest-book/proptest/getting-started.html
    proptest! {
        // #[test]
        // fn invalid_headers_never_panic(s in "\\PC*") {
        //     let mut config : CliConfig<Count> = CliConfig::new();
        //     let headers = vec!["col1", "col2", "col3"];
        //     let result = panic::catch_unwind(|| {
        //         let validation = config.get_header_idx(&s, &headers);
        //     });
        //     assert!(result.is_ok());
        // }
        #[test]
        fn delimiter_never_panics(s in "\\PC*") {
            let yaml = load_yaml!("cli.yml");
            let result = panic::catch_unwind(|| {
                let delim = format!("{}{}", "--delim=".to_string(), s);
                let matches = clap::App::from_yaml(yaml)
                    .version(crate_version!())
                    .author(crate_authors!())
                    .get_matches_from(vec![
                        "csvpivot",
                        "count",
                        &delim,
                        "--val=0"
                    ]);
                parse_delimiter(&None, &matches);
            });
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_header_index_parses_variety_of_vals() {
        // Makes sure that CliConfig can parse a variety of different headers
        let headers = vec![
            "FIELDNAME1",
            "FIELDNAME2",
            "FIELDNAME1",
            "'FIELDNAME2[0]'",
            "'FIELDNAME2[0]'",
        ];
        let config: CliConfig<Count> = CliConfig::new();
        assert_eq!(config.get_header_idx("FIELDNAME1", &headers).unwrap(), 0);
        assert_eq!(config.get_header_idx("FIELDNAME1[1]", &headers).unwrap(), 2);
        assert_eq!(
            config.get_header_idx("'FIELDNAME2[0]'", &headers).unwrap(),
            3
        );
        assert_eq!(
            config
                .get_header_idx("'FIELDNAME2[0]'[1]", &headers)
                .unwrap(),
            4
        );
        assert_eq!(config.get_header_idx("0", &headers).unwrap(), 0);
        assert_eq!(config.get_header_idx("4", &headers).unwrap(), 4);
        assert!(!(config.get_header_idx("5", &headers)).is_ok());
    }

    #[test]
    fn test_colnames_split_properly() {
        // Makes sure that fields like `1,3` split properly
        let config: CliConfig<Count> = CliConfig::new();
        let empty_vec: Vec<String> = vec![];
        assert_eq!(
            config.get_multiple_header_columns(&empty_vec).unwrap(),
            empty_vec
        );
        let simple_single_input = vec!["a".to_string()];
        assert_eq!(
            config
                .get_multiple_header_columns(&simple_single_input)
                .unwrap(),
            vec!["a".to_string()]
        );
        let comma_sep = vec!["a,b".to_string()];
        assert_eq!(
            config.get_multiple_header_columns(&comma_sep).unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
        let two_records = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            config.get_multiple_header_columns(&two_records).unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
        let two_to_three = vec!["a".to_string(), "b,c".to_string()];
        assert_eq!(
            config.get_multiple_header_columns(&two_to_three).unwrap(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        let quoted = vec!["'a,b,c',c".to_string()];
        assert_eq!(
            config.get_multiple_header_columns(&quoted).unwrap(),
            vec!["'a,b,c'".to_string(), "c".to_string()]
        );
        let empty_comma = vec![",".to_string()];
        assert!(config.get_multiple_header_columns(&empty_comma).is_err());
        let open_quote = vec!["'a,b".to_string()];
        assert!(config.get_multiple_header_columns(&open_quote).is_err());
        let ends_comma = vec!["a,".to_string()];
        assert!(config.get_multiple_header_columns(&ends_comma).is_err());
    }

    #[test]
    fn test_matches_yield_proper_config() {
        // Compares CliConfig::from_arg_matches to directly creating an Aggregator
        // to make sure CliConfig is working properly
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec![
                "csvpivot",
                "count",
                "test_csvs/layoffs.csv",
                "--rows=3",
                "--cols=1",
                "--val=0",
            ]);
        let mut expected_config = setup_config();
        expected_config.run_config().unwrap();
        let mut actual_config: CliConfig<Count> = CliConfig::from_arg_matches(matches).unwrap();
        actual_config.run_config().unwrap();
        assert_eq!(actual_config, expected_config);
    }

    #[test]
    fn test_config_can_return_csv_reader_from_filepath() {
        // Makes sure the Config struct properly returns a CSV Reader
        // given a filepath
        let config = setup_one_liners();
        let mut rdr = config.get_reader_from_path().unwrap();
        let mut iter = rdr.records();
        if let Some(result) = iter.next() {
            let record = result.unwrap();
            assert_eq!(record, vec!["a", "b", "c"]);
        }
    }

    #[test]
    fn test_aggregating_records_ignores_header() {
        let mut config = setup_one_liners();
        config.run_config().unwrap();
        assert!(config.aggregator.aggregations.is_empty());
    }

    #[test]
    fn test_no_headers_parses_first_row() {
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec![
                "csvpivot",
                "count",
                "test_csvs/one_liner.csv",
                "--rows=0",
                "--cols=1",
                "--val=2",
                "--no-header",
            ]);
        let mut config: CliConfig<Count> = CliConfig::from_arg_matches(matches).unwrap();
        config.run_config().unwrap();
        assert!(!config.aggregator.aggregations.is_empty());
        let correct_vals = config
            .aggregator
            .aggregations
            .get(&("a".to_string(), "b".to_string()))
            .is_some();
        assert!(correct_vals);
    }

    #[test]
    fn test_aggregating_records_adds_records() {
        let mut config = setup_config();
        config.run_config().unwrap();
        assert!(config
            .aggregator
            .aggregations
            .contains_key(&("sales".to_string(), "true".to_string())));
    }

    #[test]
    fn test_aggregate_adds_new_member() {
        let agg = setup_simple_count();
        assert!(agg.aggregations.contains_key(&(
            "Columbus_<sep>_OH".to_string(),
            "Blue Jackets_<sep>_Hockey".to_string()
        )));
    }

    #[test]
    fn test_adding_record_creates_new_record() {
        let agg = setup_simple_count();
        let val = agg.aggregations.get(&(
            "Columbus_<sep>_OH".to_string(),
            "Blue Jackets_<sep>_Hockey".to_string(),
        ));
        assert!(val.is_some());
    }

    #[test]
    fn test_adding_record_stores_agg_indexes() {
        let agg = setup_simple_count();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus_<sep>_OH".to_string());
        assert_eq!(agg.indexes, expected_indexes);
    }

    #[test]
    fn test_adding_record_stores_agg_columns() {
        let agg = setup_simple_count();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets_<sep>_Hockey".to_string());
        assert_eq!(agg.columns, expected_columns);
    }

    #[test]
    fn test_multiple_indexes() {
        let agg = setup_multiple_counts();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus_<sep>_OH".to_string());
        expected_indexes.insert("Nashville_<sep>_TN".to_string());
        assert_eq!(agg.indexes, expected_indexes);
    }

    #[test]
    fn test_multiple_columns() {
        let agg = setup_multiple_counts();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets_<sep>_Hockey".to_string());
        expected_columns.insert("Predators_<sep>_Hockey".to_string());
        expected_columns.insert("Titans_<sep>_Football".to_string());
        assert_eq!(agg.columns, expected_columns);
    }
}
