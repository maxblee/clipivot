use std::collections::{HashSet, HashMap};
use clap::ArgMatches;

mod errors;
mod parsing;
use crate::aggregation::errors::CsvPivotError;
use crate::aggregation::parsing::ParsingHelper;

#[derive(Debug, PartialEq)]
pub enum AggregateType {
    /// An enum that I use to determine how to collect the records for aggregation
    /// Corresponds to the aggfunc parameter in the CLI
    /// Count aggregates the values column by Count;
    /// Mean, by mean; Median, by median; Stdev, by standard deviation;
    /// Min, by minimum; Max, by maximum; Sum, by sum;
    /// Unique, by the number of distinct values; Mode, by mode; and
    /// CountExists, by the number of non-NULL values.
    /// Whether or not you can use a given type depends on the structure of the values
    /// column. Count, Unique, CountExists, Mode, Min, and Max support all types.
    /// Median, Sum, Mean, and Standard Deviation require numeric types
    Count(usize),
}

#[derive(Debug, PartialEq)]
pub struct Aggregator {
    /// The struct used to actually create the aggregations.
    /// It's designed to be initially configured using CliConfig
    parser: ParsingHelper,
    index_cols: Vec<usize>,
    column_cols: Vec<usize>,
    values_col: usize,
    indexes: HashSet<String>,
    columns: HashSet<String>,
    aggregations: HashMap<(String, String), AggregateType>,
    aggregation_type: AggregateType,
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
            aggregation_type: AggregateType::Count(0),
        }
    }
}

impl Aggregator {
    pub fn new() -> Aggregator {
        /// Creates a new Aggregator; not designed to be used without method chaining below
        Aggregator::default()
    }

    // the following approach to method chaining comes from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    pub fn set_indexes(mut self, new_indexes: Vec<usize>) -> Self {
        // sets the index columns (i.e. the first column of the pivot table) for the aggregator
        self.index_cols = new_indexes;
        self
    }

    pub fn set_columns(mut self, new_cols: Vec<usize>) -> Self {
        // sets the columns to aggregate on
        self.column_cols = new_cols;
        self
    }

    pub fn set_value_column(mut self, value_col: usize) -> Self {
//        Sets the column that forms the cell aggregations
//        (e.g. sets a 'salary' column for a sum aggregation,
//        so the resulting cells determine the SUM of salary
//        where index columns AND column columns are a given value)
        self.values_col = value_col;
        self
    }

    pub fn set_aggregation_type(mut self, agg_type: AggregateType) -> Self {
        // Sets the aggregation type, which is used when adding rows / writing to stdout
        self.aggregation_type = agg_type;
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct CliConfig {
    /// The struct for converting from Clap's ArgMatches into the Aggregator struct
    filename: Option<String>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    aggfunc: String,
    values: Option<usize>,
}

impl CliConfig {
    pub fn from_arg_matches(arg_matches: ArgMatches) -> Result<CliConfig, CsvPivotError> {
        /// Takes argument matches from main and tries to convert them into CliConfig
        let values: usize = arg_matches.value_of("value").unwrap().parse()?;
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

    pub fn to_aggregator(&self) -> Result<Aggregator, CsvPivotError> {
        /// converts from CliConfig into an Aggregator
        // take a reference of aggfunc -> Convert from &Option to &String ->
        // take a reference of &String (so it becomes &str) (**I think?)
        let agg_type = match self.aggfunc.as_ref() {
            "count" => Ok(AggregateType::Count(0)),
            _ => Err(CsvPivotError::InvalidAggregator)
        }?;
        let agg = Aggregator::new()
            .set_indexes(self.rows.clone().unwrap_or(vec![]))
            .set_columns(self.columns.clone().unwrap_or(vec![]))
            .set_value_column(self.values.clone().unwrap_or(0))
            .set_aggregation_type(agg_type);
        Ok(agg)
    }

    fn parse_column(column: Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
        let mut indexes = Vec::new();
        for idx in column {
            let index_val = idx.parse()?;
            indexes.push(index_val);
        }
        Ok(indexes)
    }

}

pub fn run(arg_matches : ArgMatches) -> Result<(), CsvPivotError> {
    let config = CliConfig::from_arg_matches(arg_matches)?;
    let mut agg = config.to_aggregator()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_yield_proper_config() {
        /// Makes sure the CliConfig::from_arg_matches impl works properly
        // Note: I eventually want this to come from a setup func, but have to deal with
        // lifetimes for that :(
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec!["csvpivot", "count", "tmp/layoffs.csv", "--rows=3", "--cols=1", "--val=0"]);
        let expected_config = CliConfig {
            filename: Some("tmp/layoffs.csv".to_string()),
            rows: Some(vec![3]),
            columns: Some(vec![1]),
            values: Some(0),
            aggfunc: "count".to_string(),
        };
        assert_eq!(CliConfig::from_arg_matches(matches).unwrap(), expected_config);
    }

    #[test]
    fn test_config_creates_proper_aggregator() {
        let config = CliConfig {
            filename: Some("tmp/layoffs.csv".to_string()),
            rows: Some(vec![3]),
            columns: Some(vec![1]),
            values: Some(0),
            aggfunc: "count".to_string()
        };
        let expected = Aggregator {
            parser: ParsingHelper::default(),
            index_cols: vec![3],
            column_cols: vec![1],
            values_col: 0,
            indexes: HashSet::new(),
            columns: HashSet::new(),
            aggregations: HashMap::new(),
            aggregation_type: AggregateType::Count(0),
        };
        assert_eq!(config.to_aggregator().unwrap(), expected);
    }
}