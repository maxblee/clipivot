extern crate csv;

use std::io;
use std::collections::{HashSet, HashMap};
use parsing::ParsingHelper;
use errors::CsvPivotError;
use std::hash::Hash;

mod parsing;
mod errors;

#[derive(Debug, PartialEq)]
enum AggregateType {
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
    fn new() -> Aggregator {
        Aggregator::default()
    }
    // approach to method chaining from
    // http://www.ameyalokare.com/rust/2017/11/02/rust-builder-pattern.html
    fn set_indexes(mut self, new_indexes: Vec<usize>) -> Self {
        self.index_cols = new_indexes;
        self
    }
    fn set_columns(mut self, new_cols: Vec<usize>) -> Self {
        self.column_cols = new_cols;
        self
    }
    fn set_value_column(mut self, value_col: usize) -> Self {
        self.values_cols = value_col;
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
    fn get_contents(&self) -> &HashMap<(String, String), AggregateType> {
        &self.aggregations
    }
    fn get_indexes(&self) -> &HashSet<String> { &self.indexes }
    fn get_columns(&self) -> &HashSet<String> { &self.columns }
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