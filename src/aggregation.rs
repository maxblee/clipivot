//! The `aggregation` module is the part of `csvpivot` that works directly with command-line arguments.
//!
//! The structure is conceptually simple. First, we validate the command-line arguments, using
//! the `CliConfig` struct. Then, for each row in the dataset, we parse (or deserialize) the row
//! using the `ParsingHelper` struct before creating or updating
//! records based on methods implemented by the `AggregationMethod` trait. Finally, we go through our
//! aggregated records set, cell by cell, and use the `to_output` method from the `AggregationMethod` trait
//! to convert the aggregation record into a string and write the strings into standard output.
//!
//! To figure out how it works, imagine we are running a pivot table using the `sum` function:
//!
//! 1. First, the `run` method will tell us to create a `CliConfig<Sum>` object.
//!
//! 2. Then, the `CliConfig` `run_config` method, which is called by the `run` method, will
//! validate all of the arguments we entered in the command line, before passing the work to the `Aggregator`
//! struct, which is responsible for creating the aggregations row by row.
//!
//! 3. For each new row that comes in, the `Aggregator` will convert the value in our values column
//! into a `Decimal` type using the `ParsingHelper` struct.
//!
//! 4. Then, the `Aggregator` struct will update the value inside its `aggregations` attribute using
//! the `Sum.new(&parsed_val)` and `Sum.update(&parsed_val)` functions.
//!
//! 5. When we are ready to output the values, the `Aggregator` struct will call the `Sum.to_output()`
//! method, which will convert each cell into a String.
//!
//! 6. Finally, the `Aggregator` struct will write the results to standard output.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

use crate::aggfunc::*;
use crate::csv_settings::CsvSettings;
use crate::errors::CsvCliResult;
use crate::parsing::{ParsingHelper, ParsingType};
use clap::ArgMatches;

const FIELD_SEPARATOR: &str = "_<sep>_";

/// The main tool for creating aggregations.
///
/// At a base level, this reads csv files, row by row, using the helper struct `ParsingHelper`
/// and a struct implementing the`AggregationMethod` trait to build the aggregations.
/// Once it has built these aggregations, it goes row by row using `AggregationMethod` to write the
/// computed records to standard output.
#[derive(Debug, Default, PartialEq)]
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
    /// This is used to initialize the `Aggregator` within `CliConfig`.
    pub fn from_parser(parser: ParsingHelper) -> Aggregator<T> {
        let mut agg = Aggregator::new();
        agg.parser = parser;
        agg
    }

    /// Sets the indexes of the `Aggregator`.
    pub fn set_indexes(&mut self, new_indexes: Vec<usize>) -> &mut Aggregator<T> {
        self.index_cols = new_indexes;
        self
    }

    /// Adds the list of columns to the aggregator.
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
    pub fn aggregate_from_file(&mut self, mut rdr: csv::Reader<fs::File>) -> CsvCliResult<()> {
        let mut line_num = 0;
        let mut record = csv::StringRecord::new();
        while rdr.read_record(&mut record)? {
            self.add_record(&record, line_num)?;
            line_num += 1;
        }
        Ok(())
    }

    /// Takes records from standard input and aggregates them row by row. The code here is identical to
    /// the code in the `aggregate_from_file` function, because the CSV reader
    /// from handling files is different from the reader for handling standard input.
    ///
    /// In the spirit of DRY, I'm open to suggestions for refactoring this code. But
    /// it's not really pressing, since we're talking about 5-ish lines of code.
    pub fn aggregate_from_stdin(&mut self, mut rdr: csv::Reader<io::Stdin>) -> CsvCliResult<()> {
        let mut line_num = 0;
        let mut record = csv::StringRecord::new();
        while rdr.read_record(&mut record)? {
            self.add_record(&record, line_num)?;
            line_num += 1;
        }
        Ok(())
    }

    /// This method goes cell by cell and serializes the data inside the `Aggregator.aggregations`
    /// attribute and outputs them into standard output.
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
    fn add_record(&mut self, record: &csv::StringRecord, line_num: usize) -> CsvCliResult<()> {
        // merges all of the index columns into a single column, separated by FIELD_SEPARATOR
        let indexnames = self.get_colname(&self.index_cols, &record);
        let columnnames = self.get_colname(&self.column_cols, &record);
        // CliConfig + csv crate do error handling that should prevent get from being None
        let str_val = record.get(self.values_col).unwrap();
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
    fn get_colname(&self, columns: &[usize], record: &csv::StringRecord) -> String {
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

/// Validates command-line arguments, before moving control of the program to the `Aggregator` struct.
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
    /// The name of the column you're running the aggregation function on.
    values_col: String,
    /// The name of the column(s) (or indexes) forming the columns of the final pivot table. vec![] if no columns.
    column_cols: Vec<String>,
    /// The name of the column(s) (or indexes) forming the indexes of the final pivot table.
    indexes: Vec<String>,
    settings: CsvSettings,
}

impl<U: AggregationMethod> Default for CliConfig<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: AggregationMethod> CliConfig<U> {
    /// Creates a new, basic CliConfig
    pub fn new() -> CliConfig<U> {
        CliConfig {
            filename: None,
            aggregator: Aggregator::new(),
            values_col: "".to_string(),
            column_cols: vec![],
            indexes: vec![],
            settings: CsvSettings::default(),
        }
    }
    /// Takes command-line arguments and tries to convert them into a `CliConfig` object, returning an error on failure.
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

        let delim_vals = if let true = arg_matches.is_present("tab") {
            Some(r"\t")
        } else {
            arg_matches.value_of("delim")
        };
        let settings =
            CsvSettings::parse_new(&filename, delim_vals, !arg_matches.is_present("noheader"))?;

        let cfg = CliConfig {
            filename: filename.map(String::from),
            aggregator,
            values_col,
            indexes,
            column_cols,
            settings,
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
        let infer_date = arg_matches.is_present("infer") || arg_matches.is_present("format");
        let dayfirst = arg_matches.is_present("dayfirst");
        let yearfirst = arg_matches.is_present("yearfirst");
        let parse_type = self.get_parsing_approach(parse_numeric, infer_date);
        let date_format = arg_matches.value_of("format");
        ParsingHelper::from_parsing_type(parse_type, dayfirst, yearfirst, date_format)
            .parse_empty_vals(!arg_matches.is_present("empty"))
    }

    /// Validates the columns you enter with the `-c`/`cols`, `-v`/`--val`, and `-r`/`--rows`,
    /// and updates the `Aggregator` object so we can run aggregations.
    fn validate_columns(&mut self, headers: &Vec<&str>) -> CsvCliResult<()> {
        // validates the aggregation columns and then updates the aggregator
        let index_vec = self.settings.get_indexes_from_header_descriptions(
            &self.indexes.iter().map(|v| v.as_ref()).collect(),
            headers,
        )?;
        let column_vec = self.settings.get_indexes_from_header_descriptions(
            &self.column_cols.iter().map(|v| v.as_ref()).collect(),
            headers,
        )?;
        let values_vec = self
            .settings
            .get_column_from_header_descriptions(&self.values_col, headers)?;

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
            // unwrap safe because of `is_some` call
            let mut rdr = self
                .settings
                .get_reader_from_path(&self.filename.clone().unwrap())?;
            let headers = rdr.headers()?;
            self.validate_columns(&headers.iter().collect())?;
            self.aggregator.aggregate_from_file(rdr)?;
        } else {
            let mut rdr = self.settings.get_reader_from_stdin();
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
        agg.add_record(&setup_simple_record(), 0).unwrap();
        agg
    }

    fn setup_multiple_counts() -> Aggregator<Count> {
        let mut agg = setup_simple_count();
        let second_vec = vec!["Nashville", "TN", "Predators", "Hockey", "Playoffs"];
        let second_record = csv::StringRecord::from(second_vec);
        agg.add_record(&second_record, 0).unwrap();
        let third_vec = vec!["Nashville", "TN", "Titans", "Football", "Bad"];
        let third_record = csv::StringRecord::from(third_vec);
        agg.add_record(&third_record, 0).unwrap();
        let fourth_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Bad"];
        let fourth_record = csv::StringRecord::from(fourth_vec);
        agg.add_record(&fourth_record, 0).unwrap();
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
            values_col: "0".to_string(),
            column_cols: vec!["1".to_string()],
            indexes: vec!["2".to_string()],
            settings: CsvSettings::parse_new(&Some("test_csvs/one_liner"), Some(","), true)
                .unwrap(),
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
            values_col: "0".to_string(),
            column_cols: vec!["1".to_string()],
            indexes: vec!["3".to_string()],
            settings: CsvSettings::parse_new(&Some("test_csvs/layoffs.csv"), Some(","), true)
                .unwrap(),
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
        let mut rdr = config
            .settings
            .get_reader_from_path(&config.filename.unwrap())
            .unwrap();
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
