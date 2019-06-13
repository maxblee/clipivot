extern crate csv;

use std::io;
use std::fs;
use std::collections::{HashSet, HashMap};
use parsing::ParsingHelper;
use errors::CsvPivotError;
use clap::App;

mod parsing;
mod errors;

#[derive(Debug, PartialEq)]
pub enum AggregateType {
    Count(usize),
}

/// Takes a file and creates a pivot table aggregation from it
/// That is, given a set of columns as index columns, 'columns' columns,
/// and a values column, it creates a HashMap of Index, Columns pairs and
/// simultaneously computes a value based on the value of the records at a given
/// column.
/// Uses the `parsing::ParsingHelper` struct to determine how exactly to do this
#[derive(Debug)]
pub struct Aggregator {
    parser: ParsingHelper,
    index_cols: Vec<usize>,
    column_cols: Vec<usize>,
    values_cols: usize,
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
            values_cols: 0,
            indexes: HashSet::new(),
            columns: HashSet::new(),
            aggregations: HashMap::new(),
            aggregation_type: AggregateType::Count(0),
        }
    }
}

impl Aggregator {
    pub fn new() -> Aggregator {
        Aggregator::default()
    }
    // approach to method chaining from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    pub fn set_indexes(mut self, new_indexes: Vec<usize>) -> Self {
        self.index_cols = new_indexes;
        self
    }
    pub fn set_columns(mut self, new_cols: Vec<usize>) -> Self {
        self.column_cols = new_cols;
        self
    }
    pub fn set_value_column(mut self, value_col: usize) -> Self {
        self.values_cols = value_col;
        self
    }
    pub fn set_aggregation_type(mut self, agg_type: AggregateType) -> Self {
        self.aggregation_type = agg_type;
        self
    }
    fn add_record(&mut self, record: csv::StringRecord) -> Result<(), CsvPivotError> {
        let indexnames = Aggregator::get_colname(&self.index_cols, &record)?;
        let columnnames = Aggregator::get_colname(&self.column_cols, &record)?;
        let str_val = record.get(self.values_cols).ok_or(CsvPivotError::InvalidField)?;
        // Not memory efficient, but it'll do for now
        // As in, I should eventually have indexnames and columnnames takes &str
        // and use lifetimes to store values of (indexnames, columnnames)
        self.indexes.insert(indexnames.clone());
        self.columns.insert(columnnames.clone());

        let parsed_val = str_val;   // TODO: Parse Value
        match self.aggregation_type {
            AggregateType::Count(_) => self.add_count(indexnames, columnnames),
        };
        Ok(())
    }
    fn add_count(&mut self, indexname: String, columnname: String) {
        // From https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| {
                let AggregateType::Count(cur_count) = *val;
                *val = AggregateType::Count(cur_count + 1)
            })
            .or_insert(AggregateType::Count(1));
    }
    fn get_colname(indexes: &Vec<usize>, record: &csv::StringRecord) -> Result<String, CsvPivotError> {
        /// Returns the String concatenation of the index fields
        /// Used to get index and column names
        let mut colnames : Vec<&str> = Vec::new();
        for idx in indexes {
            let idx_column = record.get(*idx).ok_or(CsvPivotError::InvalidField)?;
            colnames.push(idx_column);
        }
        Ok(colnames.join("$."))
    }
    pub fn parse_writing(&self, row: &String, col: &String) -> String {
        let aggval = self.aggregations.get(&(*row, *col));
        match aggval {
            Some(AggregateType::Count(num)) => num.to_string(),
            None => "".to_string(),
        }
    }
    pub fn get_contents(&self) -> &HashMap<(String, String), AggregateType> {
        &self.aggregations
    }
    pub fn get_indexes(&self) -> &HashSet<String> { &self.indexes }
    pub fn get_columns(&self) -> &HashSet<String> { &self.columns }
}

pub struct CliConfig<'a> {
    filename: Option<&'a str>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    aggfunc: Option<&'a str>,
    values: Option<usize>
}

impl<'a> CliConfig<'a> {
    pub fn from_app(app: App<'a, '_>) -> Result<CliConfig<'a>, CsvPivotError> {
        let matches = app.get_matches();
        let vals : usize = matches.value_of("value").unwrap().parse()?;
        let rows = CliConfig::parse_column(matches.values_of("rows").unwrap().collect())?;
        let cols = CliConfig::parse_column(matches.values_of("columns").unwrap().collect())?;
        let config = CliConfig {
            filename: matches.value_of("filename"),
            values: Some(vals),
            rows: Some(rows),
            columns: Some(cols),
            aggfunc: matches.value_of("aggfunc")
        };
        Ok(config)
    }

    pub fn to_aggregator(&self) -> Option<Aggregator> {
        let agg_type = match self.aggfunc.unwrap() {
            "count" => Some(AggregateType::Count(0)),
            _ => None
        };
        match agg_type {
            Some(aggfunc) => {
                Some(Aggregator::new()
                    .set_indexes(self.rows.unwrap())
                    .set_columns(self.columns.unwrap())
                    .set_value_column(self.values.unwrap())
                    .set_aggregation_type(aggfunc))
            },
            None => None
        }
    }

    fn parse_column(column: Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
        let mut idx_column : Vec<usize> = Vec::new();
        for idx in column {
            let index_val = idx.parse()?;
            idx_column.push(index_val);
        }
        Ok(idx_column)
    }
    pub fn parse_stdin(&self) -> csv::Reader<io::Stdin> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(io::stdin())
    }
    pub fn parse_filepath(&self) -> Result<csv::Reader<fs::File>, csv::Error> {
        csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(self.filename.unwrap())
    }
    pub fn is_from_path(&self) -> bool {
        match self.filename {
            Some(_) => true,
            None => false
        }
    }

//    fn find_name_in_header(name: &str, headers: &Vec<&str>) -> Option<usize> {
//        for i in 0..headers.len() {
//            if headers.get(i) == Some(&name) {
//                return Some(i);
//            }
//        }
//        return None
//    }
}



pub fn run(config: CliConfig) -> Result<(), errors::CsvPivotError> {
    let mut agg = config.to_aggregator().ok_or(CsvPivotError::InvalidAggregator)?;
    if config.is_from_path() {
        let mut rdr = config.parse_filepath()?;
        for result in rdr.records() {
            let record = result?;
            agg.add_record(record)?;
        }
    } else {
        let mut rdr = config.parse_stdin();
        for result in rdr.records() {
            let record = result?;
            agg.add_record(record)?;
        }
    }
    let mut wtr = csv::Writer::from_writer(vec![]);
    let records = agg.get_contents();
    let index = agg.get_indexes();
    let columns = agg.get_columns();
    let mut header = vec![""];
    for col in columns {
        header.push(col);
    }
    wtr.write_record(&header);
    for row in index {
        let mut record = vec![row];
        for col in columns {
            let output = &agg.parse_writing(row, col);
            record.push(&output);
        }
        wtr.write_record(&record);
    }
    wtr.flush()?;
    Ok(())
}





#[cfg(test)]
mod tests {
    use super::*;

    // The tests here are designed to test the counting aggregation
    fn setup_simple_count() -> Aggregator {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(4);
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        let record = csv::StringRecord::from(record_vec);
        agg.add_record(record);
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
    fn setup_record_for_error_checks() -> csv::StringRecord {
        /// Returns a StringRecord to pass through add_record for error checking
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        csv::StringRecord::from(record_vec)
    }
    // check errors. These should work regardless of type (they're only checking for
    // InvalidField errors
    #[test]
    fn test_invalid_indexes_raise_error() {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,5])
            .set_columns(vec![2,3])
            .set_value_column(4);
        let record = setup_record_for_error_checks();
        assert!(agg.add_record(record).is_err());
    }
    #[test]
    fn test_invalid_columns_raise_error() {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![5, 2])
            .set_value_column(4);
        let record = setup_record_for_error_checks();
        assert!(agg.add_record(record).is_err());
    }
    #[test]
    fn test_invalid_values_raise_error() {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(5);
        let record = setup_record_for_error_checks();
        assert!(agg.add_record(record).is_err());
    }

    #[test]
    fn test_aggregate_adds_new_member() {
        let agg = setup_simple_count();
        assert!(agg.get_contents().contains_key(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())));
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
        assert_eq!(agg.get_contents()
            .get(&("Columbus$.OH".to_string(), "Blue Jackets$.Hockey".to_string())),
            Some(&AggregateType::Count(2)));
    }
    #[test]
    fn test_different_index_and_cols_yields_one_count() {
        /// Makes sure that adding the same values in the index columns but
        /// different values in the column columns
        /// yields counts of 1
        let agg = setup_multiple_counts();
        assert_eq!(agg.get_contents()
            .get(&("Nashville$.TN".to_string(), "Predators$.Hockey".to_string())),
            Some(&AggregateType::Count(1)));
    }
    #[test]
    fn test_multiple_indexes() {
        /// Makes sure that multiple index matches result in multiple indexes
        let agg = setup_multiple_counts();
        let mut expected_indexes = HashSet::new();
        expected_indexes.insert("Columbus$.OH".to_string());
        expected_indexes.insert("Nashville$.TN".to_string());
        assert_eq!(agg.get_indexes(), &expected_indexes);
    }
    #[test]
    fn test_multiple_columns() {
        /// Same as above, but for columns
        let agg = setup_multiple_counts();
        let mut expected_columns = HashSet::new();
        expected_columns.insert("Blue Jackets$.Hockey".to_string());
        expected_columns.insert("Predators$.Hockey".to_string());
        expected_columns.insert("Titans$.Football".to_string());
        assert_eq!(agg.get_columns(), &expected_columns);
    }
}