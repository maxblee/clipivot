//! This module serves as the backbone of `csvpivot`.
//!
//! It contains three submodules, one for handling errors,
//! one for parsing text, and one for handling the aggregation functions.
//! It contains two structures, the `Aggregator` struct and
//! the `CliConfig` struct, which converts command-line arguments
//! into the `Aggregator` struct.
//!
//! The `run` function, meanwhile, is the part of the code that interacts
//! with the `main` binary function. I don't expect either it or `main`
//! to change at all in future versions.
//!
//! If you want to make changes to this project, I'd suggest you look around
//! at those existing structures and enums for a sense of how the code's organized
//! and how it works.

use std::collections::{HashSet, HashMap};
use std::fs;
use std::io;
use clap::ArgMatches;

mod errors;
mod parsing;
mod aggfunc;
use crate::aggregation::errors::CsvPivotError;
use crate::aggregation::parsing::ParsingHelper;
use crate::aggregation::aggfunc::{AggType, AggregationMethod};

/// The struct used to aggregate records
#[derive(Debug, PartialEq)]
pub struct Aggregator {
    /// This is used for parsing individual records. For simple aggregation
    /// methods like `count` and `unique`, this doesn't do anything.
    /// But for methods like `minimum`, it's important to handle the method differently
    /// based on the type of data (e.g. date, integer, etc.) being represented.
    parser: ParsingHelper,
    /// Used to store the names of the index column so they can be output in the final
    /// pivot table
    index_cols: Vec<usize>,
    /// Used to store the names of the columns in the final pivot table
    column_cols: Vec<usize>,
    /// The index of the column you're calculating the aggregation from.
    /// If you use a `minimum` method, for instance, the final calculation is performed on
    /// this column
    values_col: usize,
    /// The column(s) used to form the final rows in the pivot table.
    /// If empty, it computes a total.
    indexes: HashSet<String>,
    /// The column(s) used to form the final columns in the pivot table.
    /// If empty, it computes a total.
    columns: HashSet<String>,
    /// The attribute I use to calculate the final cells from
    // TODO: Adjust for aggfunc.rs
    // probably can stay the same, just with eliminating the above AggregateType
    aggregations: HashMap<(String, String), Box<AggregationMethod>>,
    /// The attribute that determines how to calculate the final values
    /// in the pivot table
    // TODO: Adjust for aggfunc.rs
    // probably can stay the same, just with eliminating above AggregateType?
    aggregation_type: AggType,
}

impl Default for Aggregator {
    fn default() -> Aggregator {
        Aggregator {
            parser: ParsingHelper::default(),
            index_cols: Vec::new(),
            column_cols: Vec::new(),
            values_col: 0,
            indexes: HashSet::new(),
            columns: HashSet::new(),
            aggregations: HashMap::new(),
            aggregation_type: AggType::Count,
        }
    }
}

impl Aggregator {
    /// Creates a new Aggregator; not designed to be used without method chaining
    pub fn new() -> Aggregator {
        Aggregator::default()
    }

    // the following approach to method chaining comes from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    /// Adds the list of index columns to the default aggregator.
    /// (This approach to method chaining comes from
    /// http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html).
    pub fn set_indexes(mut self, new_indexes: Vec<usize>) -> Self {
        self.index_cols = new_indexes;
        self
    }

    /// Adds the list of columns to the aggregator
    pub fn set_columns(mut self, new_cols: Vec<usize>) -> Self {
        self.column_cols = new_cols;
        self
    }

    /// Adds the column where the aggregation type is applied.
    /// For instance, if you decided to `sum` a bunch of salaries
    /// based on two columns, you would use this function to
    /// set the value column to the 'salaries' column.
    /// I've purposefully allowed users to only use a single value
    /// column. This contrasts with Excel, which allows for multiple values columns.
    /// As a tool designed for data exploration, I feel that users should limit themselves
    /// to a single aggregation method. Users can take a different approach
    /// by joining the data from one pivot table output to the data from another pivot table output.
    pub fn set_value_column(mut self, value_col: usize) -> Self {
        self.values_col = value_col;
        self
    }

    /// Sets the method of aggregation
    pub fn set_aggregation_type(mut self, agg_type: AggType) -> Self {
        // Sets the aggregation type, which is used when adding rows / writing to stdout
        self.aggregation_type = agg_type;
        self
    }

    /// Takes a CSV reader object from a file path and adds records, row by row.
    /// Returns an error if it can't read any of the records.
    /// This can either happen because of a problem in how the CSV
    /// was formatted or because the values/columns/indexes
    /// handed to the aggregator from the command line refer to
    /// fields that do not exist.
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
//    fn parse_writing(&self, row: &String, col: &String) -> String {
//        let aggval = self.aggregations
//            .get(&(row.to_string(), col.to_string()));
//        match aggval {
//            Some(AggregateType::Count(num)) => num.to_string(),
//            Some(AggregateType::CountUnique(vals)) =>vals.len().to_string(),
//            None => "".to_string(),
//        }
//    }

    /// Adds a record to the final pivot table. How exactly it does this depends heavily
    /// on the aggregation type, but I've opted for methods that are
    /// memory-efficient and computationally efficient whenever possible. That means, I've
    /// opted for online algorithms when able and that I've opted for algorithms that do not
    /// require putting all of the data into memory whenever possible.
    /// On that note, if you have any ideas of how to improve the algorithms used here,
    /// let me know.
    fn add_record(&mut self, record: csv::StringRecord) -> Result<(), CsvPivotError> {
        // merges all of the index columns into a single column, separated by '$.'
        let indexnames = Aggregator::get_colname(&self.index_cols, &record)?;
        let columnnames = Aggregator::get_colname(&self.column_cols, &record)?;
        let str_val = record.get(self.values_col).ok_or(CsvPivotError::InvalidField)?;
        // This isn't memory efficient, but it should be OK for now
        // (i.e. I should eventually get self.indexes and self.columns
        // be tied to self.aggregations, rather than cloned)
        self.indexes.insert(indexnames.clone());
        self.columns.insert(columnnames.clone());
        // parses the individual record. Primarily intended for parsing
        // string dates into datetime objects so methods like `min` can be applied to them accurately
        // acts differently based on the aggregation type because `count` and `unique`
        // don't require any parsing
        // TODO: I should change this into a function that matches (self.parse_val(str_val))
        let parsed_val = match self.aggregation_type {
            _ => str_val,
        };
        // this determines how to add the data as it's being read
        // TODO: convert this to aggfunc
//        match self.aggregation_type {
//            AggregateType::Count(_) => self.add_count(indexnames, columnnames),
//        };
        Ok(())
    }


    /// This method determines the name of the indexes and columns in the final pivot table.
    /// If you are just aggregating on a single column and a single index, it will just
    /// return the value of a record at that specific column. If, on the other hand, you are
    /// aggregating on multiple columns, it will take the value at the first column and the
    /// value at the second column and join them using the "$." separator.
    /// The idea behind this is that "$." is unlikely to appear naturally in most files,
    /// so you can replace it easily if needed, and it preserves the separation between
    /// different columns in the original dataset.
    ///
    /// That said, it does admittedly look clunky, so I'd love to hear any alternatives
    /// to the method I've chosen.
    ///
    /// Additionally, I'd like to eventually replace empty column vectors with "total"
    /// so you can aggregate on a single row/column. (Note that `CliConfig::from_arg_matches`
    /// will need to be changed in order for this to work.)
    fn get_colname(columns: &Vec<usize>, record: &csv::StringRecord) -> Result<String, CsvPivotError> {
        let mut colnames : Vec<&str> = Vec::new();
        for idx in columns {
            let idx_column = record.get(*idx).ok_or(CsvPivotError::InvalidField)?;
            colnames.push(idx_column);
        }
        Ok(colnames.join("$."))
    }
}

/// This struct is intended for converting from Clap's `ArgMatches` to the `Aggregator` struct
#[derive(Debug, PartialEq)]
pub struct CliConfig {
    // set as an option so I can handle standard input
    filename: Option<String>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    aggfunc: String,
    values: Option<usize>,
}

impl CliConfig {
    /// Takes argument matches from main and tries to convert them into CliConfig
    pub fn from_arg_matches(arg_matches: ArgMatches) -> Result<CliConfig, CsvPivotError> {
        // This method of error handling from
        // https://medium.com/@fredrikanderzon/custom-error-types-in-rust-and-the-operator-b499d0fb2925
        let values: usize = arg_matches.value_of("value").unwrap().parse().or(Err(CsvPivotError::InvalidField))?;
        // Eventually should replace unwrap() from rows and columns with unwrap_or
        // so I can aggregate solely by rows or solely by columns
        let rows = CliConfig::parse_column(arg_matches
            .values_of("rows").unwrap().collect())?;
        let columns = CliConfig::parse_column(arg_matches
            .values_of("columns").unwrap().collect())?;
        let filename = arg_matches.value_of("filename").map(String::from);
        let aggfunc = arg_matches.value_of("aggfunc").unwrap().to_string();
        let cfg = CliConfig {
            filename,
            rows: Some(rows),
            columns: Some(columns),
            aggfunc,
            values: Some(values),
        };
        Ok(cfg)
    }

    /// Converts from CliConfig into an Aggregator
    pub fn to_aggregator(&self) -> Result<Aggregator, CsvPivotError> {
        // take a reference of aggfunc -> Convert from &Option to &String ->
        // take a reference of &String (so it becomes &str) (**I think?)
        // TODO: Adjust this for aggfunc.rs
        let agg_type = match self.aggfunc.as_ref() {
            "count" => Ok(AggType::Count),
            // Clap should make the below statement irrelevent
            // But match needs to be comprehensive so here we are
            _ => Err(CsvPivotError::InvalidAggregator)
        }?;
        let agg = Aggregator::new()
            .set_indexes(self.rows.clone().unwrap_or(vec![]))
            .set_columns(self.columns.clone().unwrap_or(vec![]))
            .set_value_column(self.values.clone().unwrap_or(0))
            .set_aggregation_type(agg_type);
        Ok(agg)
    }

    /// Converts from a file path to either a CSV reader or a CSV error.
    /// In the spirit of DRY, it would be nice to avoid replicating code from this and
    /// `get_reader_from_stdin`. This should be able to be done simply by creating a function
    /// that returns a `csv::ReaderBuilder` and then applying that to both functions.
    /// That will become especially important when I eventually get around to adding
    /// additional features, like allowing users to select a delimeter other than ','.
    pub fn get_reader_from_path(&self) -> Result<csv::Reader<fs::File>, csv::Error> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            // this function is only run if self.filename.is_some() so unwrap() is fine
            .from_path(self.filename.as_ref().unwrap())
    }

    /// Converts from standard input to a CSV reader.
    pub fn get_reader_from_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(io::stdin())
    }

    /// Returns `true` if the user entered a filename. Used to determine
    /// whether the program should read from standard input or from a file
    pub fn is_from_path(&self) -> bool {
        self.filename.is_some()
    }

    /// Tries to convert the --columns and --rows flags from the CLI into
    /// a vector of (positive) integers. If it cannot do so, it returns an
    /// `InvalidField` error.
    fn parse_column(column: Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
        let mut indexes = Vec::new();
        for idx in column {
            let index_val = idx.parse().or(Err(CsvPivotError::InvalidField))?;
            indexes.push(index_val);
        }
        Ok(indexes)
    }

}

/// This function is the part that directly interacts with `main`.
/// It shouldn't change, even as I add features and fix bugs.
pub fn run(arg_matches : ArgMatches) -> Result<(), CsvPivotError> {
    let config = CliConfig::from_arg_matches(arg_matches)?;
    let mut agg = config.to_aggregator()?;
    if config.is_from_path() {
        let rdr = config.get_reader_from_path()?;
        agg.aggregate_from_file(rdr)?;
    } else {
        let rdr = config.get_reader_from_stdin();
        agg.aggregate_from_stdin(rdr)?;
    }
//    agg.write_results()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple_record() -> csv::StringRecord {
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        csv::StringRecord::from(record_vec)
    }

    fn setup_simple_count() -> Aggregator {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(4);
        agg.add_record(setup_simple_record());
        agg
    }

    fn setup_multiple_counts() -> Aggregator {
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

    fn setup_one_liners() -> CliConfig {
        CliConfig {
            filename: Some("test_csvs/one_liner.csv".to_string()),
            rows: Some(vec![2]),
            columns: Some(vec![1]),
            values: Some(0),
            aggfunc: "count".to_string(),
        }
    }

    fn setup_config() -> CliConfig {
        CliConfig {
            filename: Some("test_csvs/layoffs.csv".to_string()),
            rows: Some(vec![3]),
            columns: Some(vec![1]),
            values: Some(0),
            aggfunc: "count".to_string(),
        }
    }

    #[test]
    fn test_matches_yield_proper_config() {
        /// Makes sure the CliConfig::from_arg_matches impl works properly
        // Note: I eventually want this to come from a setup func, but have to deal with
        // lifetimes for that :(
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec!["csvpivot", "count", "test_csvs/layoffs.csv", "--rows=3", "--cols=1", "--val=0"]);
        let expected_config = setup_config();
        assert_eq!(CliConfig::from_arg_matches(matches).unwrap(), expected_config);
    }

    #[test]
    fn test_config_creates_proper_aggregator() {
        let config = setup_config();
        let expected = Aggregator {
            parser: ParsingHelper::default(),
            index_cols: vec![3],
            column_cols: vec![1],
            values_col: 0,
            indexes: HashSet::new(),
            columns: HashSet::new(),
            aggregations: HashMap::new(),
            aggregation_type: AggType::Count,
        };
        assert_eq!(config.to_aggregator().unwrap(), expected);
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
    fn test_config_can_return_csv_reader_from_stdin() {
        // same as above but with stdin

    }

    #[test]
    fn test_aggregating_records_ignores_header() {
        let config = setup_one_liners();
        let mut agg = config.to_aggregator().unwrap();
        let mut rdr = config.get_reader_from_path().unwrap();
        agg.aggregate_from_file(rdr);
        assert!(agg.aggregations.is_empty());
    }

    #[test]
    fn test_aggregating_records_adds_records() {
        let config = setup_config();
        let mut agg = config.to_aggregator().unwrap();
        let mut rdr = config.get_reader_from_path().unwrap();
        agg.aggregate_from_file(rdr);
        assert!(agg.aggregations.contains_key(&("sales".to_string(), "true".to_string())));
    }

    #[test]
    fn test_invalid_indexes_raise_error() {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,5])
            .set_columns(vec![2,3])
            .set_value_column(4);
        let record = setup_simple_record();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_invalid_columns_raise_error() {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![5,2])
            .set_value_column(4);
        let record = setup_simple_record();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_invalid_value_raises_error() {
        let mut agg = Aggregator::new()
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
            .contains_key(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())));
    }

//    #[test]
//    fn test_adding_record_results_in_single_count() {
//        let agg = setup_simple_count();
//        assert_eq!(agg.aggregations
//            .get(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())),
//        Some(&AggregateType::Count(1)));
//    }

    #[test]
    fn test_adding_record_stores_agg_indexes() {
        let agg = setup_simple_count();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus$.OH".to_string());
        assert_eq!(agg.indexes, expected_indexes);
    }

    #[test]
    fn test_adding_record_stores_agg_columns() {
        let agg = setup_simple_count();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets$.Hockey".to_string());
        assert_eq!(agg.columns, expected_columns);
    }

//    #[test]
//    fn test_multiple_matches_yields_multiple_counts() {
//        let agg = setup_multiple_counts();
//        let actual_counts = agg.aggregations
//            .get(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string()));
//        assert_eq!(actual_counts, Some(&AggregateType::Count(2)));
//    }

//    #[test]
//    fn test_different_index_and_cols_yields_one_count() {
//        let agg = setup_multiple_counts();
//        let actual_counts = agg.aggregations
//            .get(&("Nashville$.TN".to_string(), "Predators$.Hockey".to_string()));
//        assert_eq!(actual_counts, Some(&AggregateType::Count(1)));
//    }

    #[test]
    fn test_multiple_indexes() {
        let agg = setup_multiple_counts();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus$.OH".to_string());
        expected_indexes.insert("Nashville$.TN".to_string());
        assert_eq!(agg.indexes, expected_indexes);
    }

    #[test]
    fn test_multiple_columns() {
        let agg = setup_multiple_counts();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets$.Hockey".to_string());
        expected_columns.insert("Predators$.Hockey".to_string());
        expected_columns.insert("Titans$.Football".to_string());
        assert_eq!(agg.columns, expected_columns);
    }
}