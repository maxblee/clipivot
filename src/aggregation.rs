//! The module that interacts most directly with `main`.
//!
//! This module is centered around two structs. `Aggregator` serves
//! as a generic struct for each `AggregationMethod` type and each
//! `ParsingHelper` type for reading, aggregating, and writing pivot tables.
//!
//! `CliConfig`, meanwhile, is designed as a simple interface between `Clap`'s
//! `ArgMatches` and the `Aggregator` struct.
//!
//! There are a few major changes I eventually want to see with these structs.
//! I eventually want to support delimiters other than commas, and I eventually
//! want to support non-UTF-8 text. Any additional flags or options I add
//! (or you add) to `csvpivot` also will have to result in changes to `CliConfig`.
use std::collections::{HashSet, HashMap};
use std::io;
use std::fs;

use clap::ArgMatches;
use crate::aggfunc::*;
use crate::errors::CsvPivotError;
use crate::parsing::{ParsingHelper, ParsingType};


const FIELD_SEPARATOR: &'static str = "_<sep>_";

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

impl <T: AggregationMethod> Aggregator<T> {

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

    // the following approach to method chaining comes from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    /// Adds the list of index columns to the default aggregator.
    /// (This approach to method chaining comes from
    /// [http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html](http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html).)
    pub fn set_indexes(mut self, new_indexes: Vec<usize>) -> Self {
        self.index_cols = new_indexes;
        self
    }

    /// Adds the list of columns to the aggregator
    pub fn set_columns(mut self, new_cols: Vec<usize>) -> Self {
        self.column_cols = new_cols;
        self
    }

    /// Adds a `ParsingHelper` (if you're not using the default of Text(Option<String>))
    pub fn set_parser(mut self, new_parser: ParsingHelper) -> Self {
        self.parser = new_parser;
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
    pub fn set_value_column(mut self, value_col: usize) -> Self {
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
    pub fn aggregate_from_file(&mut self, mut rdr: csv::Reader<fs::File>) -> Result<(), CsvPivotError> {
        for result in rdr.records() {
            let record = result?;
            self.add_record(record)?;
        }
        Ok(())
    }

    /// Takes records from standard input and aggregates them row by row. The code here is identical to
    /// the code in the `aggregate_from_file` function, because the CSV reader
    /// from handling files is different from the reader for handling standard input.
    ///
    /// In the spirit of DRY, I'm open to suggestions for refactoring this code. But
    /// it's not really pressing, since we're talking about 5-ish lines of code.
    pub fn aggregate_from_stdin(&mut self, mut rdr: csv::Reader<io::Stdin>) -> Result<(), CsvPivotError> {
        for result in rdr.records() {
            let record = result?;
            self.add_record(record)?;
        }
        Ok(())
    }

    /// Once I've added all of the records to the dataset, I use this method to
    /// write them to standard output. The function adds a header based on all of the unique
    /// strings appearing in the columns column. Then, it parses the data, cell by cell
    /// and writes the data, row by row, to standard output.
    pub fn write_results(&self) -> Result<(), CsvPivotError> {
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
    /// writer can write the data to standard output
    fn parse_writing(&self, row: &String, col: &String) -> String {
        let aggval = self.aggregations
            .get(&(row.to_string(), col.to_string()));
        match aggval {
            Some(agg) => agg.to_output(),
            None => "".to_string()
        }
    }

    fn add_record(&mut self, record: csv::StringRecord) -> Result<(), CsvPivotError> {
        // merges all of the index columns into a single column, separated by FIELD_SEPARATOR
        let indexnames = self.get_colname(&self.index_cols, &record)?;
        let columnnames = self.get_colname(&self.column_cols, &record)?;
        let str_val = record.get(self.values_col).ok_or(CsvPivotError::InvalidField)?;
        // This isn't memory efficient, but it should be OK for now
        // (i.e. I should eventually get self.indexes and self.columns
        // be tied to self.aggregations, rather than cloned)
        self.indexes.insert(indexnames.clone());
        self.columns.insert(columnnames.clone());
        let parsed_val = self.parser.parse_val(str_val)?;
        // this determines how to add the data as it's being read
        if parsed_val.is_some() {
            self.update_aggregations(indexnames, columnnames, &parsed_val.unwrap());
        }
        Ok(())
    }

    fn get_colname(&self, columns: &Vec<usize>, record: &csv::StringRecord) -> Result<String, CsvPivotError> {
        let mut colnames : Vec<&str> = Vec::new();
        if columns.is_empty() {
            return Ok("total".to_string())
        }
        for idx in columns {
            let idx_column = record.get(*idx).ok_or(CsvPivotError::InvalidField)?;
            colnames.push(idx_column);
        }
        Ok(colnames.join(FIELD_SEPARATOR))
    }

    fn update_aggregations(&mut self, indexname: String, columnname: String, parsed_val: &ParsingType) {
        // modified from
        // https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| val.update(parsed_val))
            .or_insert(T::new(parsed_val));
    }
}

/// This struct is intended for converting from Clap's `ArgMatches` to the `Aggregator` struct
#[derive(Debug, PartialEq)]
pub struct CliConfig<U>
    where U:
        AggregationMethod,
{
    // set as an option so I can handle standard input
    filename: Option<String>,
    aggregator: Aggregator<U>,
    has_header: bool,
}

impl <U: AggregationMethod> CliConfig<U> {
    /// Creates a new, basic CliConfig
    pub fn new() -> CliConfig<U> {
        CliConfig {filename: None, aggregator: Aggregator::new(), has_header: true }
    }
    /// Takes argument matches from main and tries to convert them into CliConfig
    pub fn from_arg_matches(arg_matches: ArgMatches) -> Result<CliConfig<U>, CsvPivotError> {
        let mut base_config : CliConfig<U> = CliConfig::new();
        // This method of error handling from
        // https://medium.com/@fredrikanderzon/custom-error-types-in-rust-and-the-operator-b499d0fb2925
        let values: usize = arg_matches.value_of("value").unwrap()
            .parse().or(Err(CsvPivotError::InvalidField))?;
        // This makes it so set_indexes and set_columns can set an empty vector (for totals)
        let rowvec = match arg_matches.values_of("rows") {
            Some(vals) => vals.collect(),
            None => Vec::new()
        };
        let colvec = match arg_matches.values_of("columns") {
            Some(vals) => vals.collect(),
            None => Vec::new()
        };
        let rows = parse_column(rowvec)?;
        let columns = parse_column(colvec)?;
        let filename = arg_matches.value_of("filename").map(String::from);
        let parser = base_config.get_parser(&arg_matches);
        let aggregator : Aggregator<U> = Aggregator::new()
            .set_value_column(values)
            .set_columns(columns)
            .set_indexes(rows)
            .set_parser(parser);

        let cfg = CliConfig {
            filename,
            aggregator,
            has_header: !arg_matches.is_present("noheader"),
        };
        Ok(cfg)
    }
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
            AggTypes::Maximum => ParsingType::DateTypes(None),
            AggTypes::Minimum => ParsingType::DateTypes(None),
            AggTypes::Range => ParsingType::DateTypes(None),
        }
    }

    fn get_parser(&self, arg_matches: &ArgMatches) -> ParsingHelper {
        let parse_numeric = arg_matches.is_present("numeric");
        let infer_date = arg_matches.is_present("infer");
        let parse_type = self.get_parsing_approach(parse_numeric, infer_date);
        ParsingHelper::from_parsing_type(parse_type)
            .parse_empty_vals(!arg_matches.is_present("empty"))
    }
    /// Converts from a file path to either a CSV reader or a CSV error.
    ///
    /// In the spirit of DRY, it would be nice to avoid replicating code from this and
    /// `get_reader_from_stdin`.
    ///
    /// This should be able to be done simply by creating a function
    /// that returns a `csv::ReaderBuilder` and then applying that to both functions.
    /// That will become especially important when I eventually get around to adding
    /// additional features, like allowing users to select a delimeter other than ','.
    // TODO: Refactor this code
    pub fn get_reader_from_path(&self) -> Result<csv::Reader<fs::File>, csv::Error> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            // this function is only run if self.filename.is_some() so unwrap() is fine
            .from_path(self.filename.as_ref().unwrap())
    }

    /// Converts from standard input to a CSV reader.
    pub fn get_reader_from_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .has_headers(self.has_header)
            .from_reader(io::stdin())
    }

    /// Returns `true` if the user entered a filename. Used to determine
    /// whether the program should read from standard input or from a file
    pub fn is_from_path(&self) -> bool {
        self.filename.is_some()
    }

    /// Runs the `Aggregator` for the given type.
    pub fn run_config(&mut self) -> Result<(), CsvPivotError> {
        if self.filename.is_some() {
            let rdr = self.get_reader_from_path()?;
            self.aggregator.aggregate_from_file(rdr)?;
        } else {
            let rdr = self.get_reader_from_stdin();
            self.aggregator.aggregate_from_stdin(rdr)?;
        }
        self.aggregator.write_results()?;
        Ok(())
    }
}

/// Tries to convert the --columns and --rows flags from the CLI into
    /// a vector of (positive) integers. If it cannot do so, it returns an
    /// `InvalidField` error.
pub fn parse_column(column: Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
    let mut indexes = Vec::new();
    for idx in column {
        let index_val = idx.parse().or(Err(CsvPivotError::InvalidField))?;
        indexes.push(index_val);
    }
    Ok(indexes)
}

/// This function is the part of the program that directly interacts with `main`.
pub fn run(arg_matches : ArgMatches) -> Result<(), CsvPivotError> {
    let aggfunc = arg_matches.value_of("aggfunc").unwrap();
    if aggfunc == "count" {
        let mut config : CliConfig<Count> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "countunique" {
        let mut config : CliConfig<CountUnique> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "mode" {
        let mut config : CliConfig<Mode> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "mean" {
        let mut config : CliConfig<Mean> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "sum" {
        let mut config : CliConfig<Sum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "median" {
        let mut config : CliConfig<Median> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "stddev" {
        let mut config : CliConfig<StdDev> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "min" {
        let mut config : CliConfig<Minimum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "max" {
        let mut config : CliConfig<Maximum> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    } else if aggfunc == "range" {
        let mut config : CliConfig<Range> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple_record() -> csv::StringRecord {
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        csv::StringRecord::from(record_vec)
    }

    fn setup_simple_count() -> Aggregator<Count> {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(4);
        agg.add_record(setup_simple_record());
        agg
    }

    fn setup_multiple_counts() -> Aggregator<Count> {
        let mut agg = setup_simple_count();
        let second_vec = vec!["Nashville", "TN", "Predators", "Hockey", "Playoffs"];
        let second_record = csv::StringRecord::from(second_vec);
        agg.add_record(second_record);
        let third_vec = vec!["Nashville", "TN", "Titans", "Football", "Bad"];
        let third_record = csv::StringRecord::from(third_vec);
        agg.add_record(third_record);
        let fourth_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Bad"];
        let fourth_record = csv::StringRecord::from(fourth_vec);
        agg.add_record(fourth_record);
        agg
    }

    fn setup_one_liners() -> CliConfig<Count> {
        let agg : Aggregator<Count> = Aggregator::new()
            .set_indexes(vec![2])
            .set_columns(vec![1])
            .set_value_column(0);
        CliConfig {
            filename: Some("test_csvs/one_liner.csv".to_string()),
            aggregator: agg,
            has_header: true,
        }
    }

    fn setup_config() -> CliConfig<Count> {
        let agg : Aggregator<Count> = Aggregator::new()
            .set_indexes(vec![3])
            .set_columns(vec![1])
            .set_value_column(0);
        CliConfig {
            filename: Some("test_csvs/layoffs.csv".to_string()),
            aggregator: agg,
            has_header: true,
        }
    }

    #[test]
    fn test_matches_yield_proper_config() {
        // Compares CliConfig::from_arg_matches to directly creating an Aggregator
        // to make sure CliConfig is working properly
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec!["csvpivot", "count", "test_csvs/layoffs.csv", "--rows=3", "--cols=1", "--val=0"]);
        let expected_config = setup_config();
        let actual_config : CliConfig<Count> = CliConfig::from_arg_matches(matches).unwrap();
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
        config.run_config();
        assert!(config.aggregator.aggregations.is_empty());
    }

    #[test]
    fn test_no_headers_parses_first_row() {
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec!["csvpivot", "count", "test_csvs/one_liner.csv",
                                   "--rows=0", "--cols=1", "--val=2", "--no-header"]);
        let mut config : CliConfig<Count> = CliConfig::from_arg_matches(matches).unwrap();
        config.run_config();
        assert!(!config.aggregator.aggregations.is_empty());
        let correct_vals = config.aggregator.aggregations
            .get(&("a".to_string(), "b".to_string())).is_some();
        assert!(correct_vals);
    }

    #[test]
    fn test_aggregating_records_adds_records() {
        let mut config = setup_config();
        config.run_config();
        assert!(config.aggregator.aggregations.contains_key(&("sales".to_string(), "true".to_string())));
    }

    #[test]
    fn test_invalid_indexes_raise_error() {
        let mut agg : Aggregator<Count> = Aggregator::new()
            .set_indexes(vec![0,5])
            .set_columns(vec![2,3])
            .set_value_column(4);
        let record = setup_simple_record();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_invalid_columns_raise_error() {
        let mut agg : Aggregator<Count> = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![5,2])
            .set_value_column(4);
        let record = setup_simple_record();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_invalid_value_raises_error() {
        let mut agg : Aggregator<Count> = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(5);
        let record = setup_simple_record();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_aggregate_adds_new_member() {
        let agg = setup_simple_count();
        assert!(agg.aggregations
            .contains_key(&("Columbus_<sep>_OH".to_string(), "Blue Jackets_<sep>_Hockey".to_string())));
    }

    #[test]
    fn test_adding_record_creates_new_record() {
        let agg = setup_simple_count();
        let val = agg.aggregations
            .get(&("Columbus_<sep>_OH".to_string(), "Blue Jackets_<sep>_Hockey".to_string()));
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