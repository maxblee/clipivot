extern crate csv;

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum CsvTypes {
    Text(String),
}

#[derive(Debug)]
struct AggregateValue(String, Option<CsvTypes>);

#[derive(Debug)]
pub struct Aggregator {
    index_cols: Vec<i32>,
    col_cols: Vec<i32>,
    values_col: i32,
    aggregations: HashMap<String, Vec<AggregateValue>>
}

impl Aggregator {
    fn new() -> Aggregator {
        Aggregator {
            index_cols: vec![],
            col_cols: vec![],
            values_col: -1,
            aggregations: HashMap::new(),
        }
    }
    fn set_indexes(self, indexes: Vec<i32>) -> Aggregator {
        Aggregator {
            index_cols: indexes,
            col_cols: self.col_cols,
            values_col: self.values_col,
            aggregations: self.aggregations,
        }
    }
    fn set_columns(self, colnames: Vec<i32>) -> Aggregator {
        Aggregator {
            index_cols: self.index_cols,
            col_cols: colnames,
            values_col: self.values_col,
            aggregations: self.aggregations,
        }
    }
    fn set_value_column(self, col: i32) -> Aggregator {
        Aggregator {
            index_cols: self.index_cols,
            col_cols: self.col_cols,
            values_col: col,
            aggregations: self.aggregations,
        }
    }
    fn add_record(&mut self, record: csv::StringRecord) {
        let hi = 1;
    }
    fn get_contents(&self) -> &HashMap<String, Vec<AggregateValue>> {
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
        let record = csv::StringRecord::from(vec!["Columbus", "OH", "Blue Jackets", "Hockey", "Playoffs"]);
        agg.add_record(record);
        agg
    }
    #[test]
    fn test_aggregate_adds_new_member() {
        let agg = setup_simple();
        assert!(agg.get_contents().contains_key("Columbus.OH"));
    }
}