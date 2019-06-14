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

    pub fn add_record(&mut self, record: csv::StringRecord) -> Result<(), CsvPivotError> {
        let indexnames = Aggregator::get_colname(&self.index_cols, &record)?;
        let columnnames = Aggregator::get_colname(&self.column_cols, &record)?;
        let str_val = record.get(self.values_col).ok_or(CsvPivotError::InvalidField)?;
        // This isn't memory efficient, but it should be OK for now
        // (i.e. I should eventually get self.indexes and self.columns
        // be tied to self.aggregations, rather than cloned)
        self.indexes.insert(indexnames.clone());
        self.columns.insert(columnnames.clone());
        let parsed_val = str_val; // TODO: Figure out parsing
        match self.aggregation_type {
            AggregateType::Count(_) => self.add_count(indexnames, columnnames),
        };
        Ok(())
    }

    fn add_count(&mut self, indexname: String, columnname: String) {
        // from https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| {
                let AggregateType::Count(cur_count) = *val;
                *val = AggregateType::Count(cur_count + 1)
            })
            .or_insert(AggregateType::Count(1));
    }

    fn get_colname(columns: &Vec<usize>, record: &csv::StringRecord) -> Result<String, CsvPivotError> {
        let mut colnames : Vec<&str> = Vec::new();
        for idx in columns {
            let idx_column = record.get(*idx).ok_or(CsvPivotError::InvalidField)?;
            colnames.push(idx_column);
        }
        Ok(colnames.join("$."))
    }

    // The following methods are quick public ways of acquiring data from self
    pub fn get_contents(&self) -> &HashMap<(String, String), AggregateType> {
        &self.aggregations
    }

    pub fn get_indexes(&self) -> &HashSet<String> { &self.indexes }
    pub fn get_columns(&self) -> &HashSet<String> { &self.columns }
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
        assert!(agg.get_contents()
            .contains_key(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())));
    }

    #[test]
    fn test_adding_record_results_in_single_count() {
        let agg = setup_simple_count();
        assert_eq!(agg.get_contents()
            .get(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())),
        Some(&AggregateType::Count(1)));
    }

    #[test]
    fn test_adding_record_stores_agg_indexes() {
        let agg = setup_simple_count();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus$.OH".to_string());
        assert_eq!(agg.get_indexes(), &expected_indexes);
    }

    #[test]
    fn test_adding_record_stores_agg_columns() {
        let agg = setup_simple_count();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets$.Hockey".to_string());
        assert_eq!(agg.get_columns(), &expected_columns);
    }

    #[test]
    fn test_multiple_matches_yields_multiple_counts() {
        let agg = setup_multiple_counts();
        let actual_counts = agg.get_contents()
            .get(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string()));
        assert_eq!(actual_counts, Some(&AggregateType::Count(2)));
    }

    #[test]
    fn test_different_index_and_cols_yields_one_count() {
        let agg = setup_multiple_counts();
        let actual_counts = agg.get_contents()
            .get(&("Nashville$.TN".to_string(), "Predators$.Hockey".to_string()));
        assert_eq!(actual_counts, Some(&AggregateType::Count(1)));
    }

    #[test]
    fn test_multiple_indexes() {
        let agg = setup_multiple_counts();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus$.OH".to_string());
        expected_indexes.insert("Nashville$.TN".to_string());
        assert_eq!(agg.get_indexes(), &expected_indexes);
    }

    #[test]
    fn test_multiple_columns() {
        let agg = setup_multiple_counts();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets$.Hockey".to_string());
        expected_columns.insert("Predators$.Hockey".to_string());
        expected_columns.insert("Titans$.Football".to_string());
        assert_eq!(agg.get_columns(), &expected_columns);
    }
}