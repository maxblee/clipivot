extern crate csv;

use std::io;
use std::collections::{HashSet, HashMap};
use parsing::ParsingHelper;
use errors::CsvPivotError;

mod parsing;
mod errors;

#[derive(Debug)]
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
        Ok(())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_simple() -> Aggregator {
        let mut agg = Aggregator::new()
            .set_indexes(vec![0,1])
            .set_columns(vec![2,3])
            .set_value_column(4);
        let record_vec = vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"];
        let record = csv::StringRecord::from(record_vec);
        agg.add_record(record);
        agg
    }
    #[test]
    fn test_aggregate_adds_new_member() {
        let agg = setup_simple();
        assert!(agg.get_contents().contains_key(&("Columbus.OH".to_string(), "Blue Jackets.Hockey".to_string())));
    }
}