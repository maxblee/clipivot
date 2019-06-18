use std::collections::{HashSet, HashMap};
use std::io;
use std::fs;

use clap::ArgMatches;

mod errors;
use crate::aggregation::errors::CsvPivotError;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub enum ParsingType {
    Text(Option<String>)
}

#[derive(Debug, PartialEq)]
pub struct ParsingHelper {
    values_type: ParsingType,
    possible_values: Vec<ParsingType>
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::Text(None),
            possible_values: vec![ParsingType::Text(None)]
        }
    }
}

impl ParsingHelper {
    // TODO: Convert to Result Type
    fn parse_val(&self, new_val: &str) -> ParsingType {
        match self.values_type {
            ParsingType::Text(_) => ParsingType::Text(Some(new_val.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AggTypes {
    Count,
}

pub trait AggregationMethod {
    type Aggfunc;

    /// Returns the Aggregation method (e.g. AggTypes::Count)
    fn get_aggtype(&self) -> AggTypes;
    /// Instantiates a new Aggregation method
    fn new(parsed_val: &ParsingType) -> Self;
    /// Updates an existing method
    fn update(&mut self, parsed_val: &ParsingType);
}

struct Count {
    val: usize,
}

impl AggregationMethod for Count {
    type Aggfunc = Count;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Count }
    fn new(parsed_val: &ParsingType) -> Self {
        Count { val: 1 }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        self.val += 1;
    }
}

#[derive(Debug, PartialEq)]
pub struct Aggregator<T>
    where
        T: AggregationMethod,
{
    aggregations: HashMap<(String, String), T>,
    indexes: HashSet<String>,
    columns: HashSet<String>,
    parser: ParsingHelper,
    index_cols: Vec<usize>,
    column_cols: Vec<usize>,
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
    fn parse_writing(&self, row: &String, col: &String) -> String {
        let aggval = self.aggregations
            .get(&(row.to_string(), col.to_string()));
        match aggval {
            Some(agg) => "1".to_string(),
            None => "".to_string()
        }
    }

    fn add_record(&mut self, record: csv::StringRecord) -> Result<(), CsvPivotError> {
        // merges all of the index columns into a single column, separated by '$.'
        let indexnames = self.get_colname(&self.index_cols, &record)?;
        let columnnames = self.get_colname(&self.column_cols, &record)?;
        let str_val = record.get(self.values_col).ok_or(CsvPivotError::InvalidField)?;
        // This isn't memory efficient, but it should be OK for now
        // (i.e. I should eventually get self.indexes and self.columns
        // be tied to self.aggregations, rather than cloned)
        self.indexes.insert(indexnames.clone());
        self.columns.insert(columnnames.clone());
        let parsed_val = self.parser.parse_val(str_val);
        // this determines how to add the data as it's being read
        self.update_aggregations(indexnames, columnnames, &parsed_val);
        Ok(())
    }

    fn get_colname(&self, columns: &Vec<usize>, record: &csv::StringRecord) -> Result<String, CsvPivotError> {
        let mut colnames : Vec<&str> = Vec::new();
        for idx in columns {
            let idx_column = record.get(*idx).ok_or(CsvPivotError::InvalidField)?;
            colnames.push(idx_column);
        }
        Ok(colnames.join("$."))
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
}

impl <U: AggregationMethod> CliConfig<U> {
    /// Takes argument matches from main and tries to convert them into CliConfig
    pub fn from_arg_matches(arg_matches: ArgMatches) -> Result<CliConfig<U>, CsvPivotError> {
        // This method of error handling from
        // https://medium.com/@fredrikanderzon/custom-error-types-in-rust-and-the-operator-b499d0fb2925
        let values: usize = arg_matches.value_of("value").unwrap().parse().or(Err(CsvPivotError::InvalidField))?;
        // Eventually should replace unwrap() from rows and columns with unwrap_or
        // so I can aggregate solely by rows or solely by columns
        let rows = parse_column(arg_matches
            .values_of("rows").unwrap().collect())?;
        let columns = parse_column(arg_matches
            .values_of("columns").unwrap().collect())?;
        let filename = arg_matches.value_of("filename").map(String::from);
        let aggregator : Aggregator<U> = Aggregator::new()
            .set_value_column(values)
            .set_columns(columns)
            .set_indexes(rows);

        let cfg = CliConfig {
            filename,
            aggregator,
        };
        Ok(cfg)
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

/// This function is the part that directly interacts with `main`.
/// It shouldn't change, even as I add features and fix bugs.
pub fn run(arg_matches : ArgMatches) -> Result<(), CsvPivotError> {
    let aggfunc = arg_matches.value_of("aggfunc").unwrap();
    if aggfunc == "count" {
        let mut config : CliConfig<Count> = CliConfig::from_arg_matches(arg_matches)?;
        config.run_config()?;
    }
    Ok(())
}