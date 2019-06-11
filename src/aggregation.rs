extern crate csv;

use std::collections::HashMap;
use self::csv::StringRecord;

#[derive(Debug, PartialEq)]
enum CsvTypes {
    Text(String),
}

#[derive(Debug)]
pub struct Aggregator {
    index_cols: Vec<usize>,
    col_cols: Vec<usize>,
    values_col: usize,
    aggregations: HashMap<(String, String), Vec<Option<String>>>,   // TODO
    has_nulls: bool,
}

impl Aggregator {
    fn new() -> Aggregator {
        Aggregator {
            index_cols: vec![],
            col_cols: vec![],
            values_col: 0,
            aggregations: HashMap::new(),
            has_nulls: false,
        }
    }
    fn set_indexes(self, indexes: Vec<usize>) -> Aggregator {
        Aggregator {
            index_cols: indexes,
            col_cols: self.col_cols,
            values_col: self.values_col,
            aggregations: self.aggregations,
            has_nulls: false,
        }
    }
    fn set_columns(self, colnames: Vec<usize>) -> Aggregator {
        Aggregator {
            index_cols: self.index_cols,
            col_cols: colnames,
            values_col: self.values_col,
            aggregations: self.aggregations,
            has_nulls: false,
        }
    }

    fn set_value_column(self, col: usize) -> Aggregator {
        Aggregator {
            index_cols: self.index_cols,
            col_cols: self.col_cols,
            values_col: col,
            aggregations: self.aggregations,
            has_nulls: false,
        }
    }

    fn add_record(&mut self, record: csv::StringRecord) {
        let indexname = Aggregator::get_colname(&self.index_cols, &record);
        let colname= Aggregator::get_colname(&self.col_cols, &record);
        let val = record.get(self.values_col).map(|v| v.to_string());
        self.aggregations.entry((indexname, colname)).or_insert(Vec::new()).push(val);
    }

    fn get_colname(indexes: &Vec<usize>, record: &csv::StringRecord) -> String {
        let mut colnames : Vec<&str> = Vec::new();
        for idx in indexes {
            colnames.push(record.get(*idx).unwrap_or(""));
        }
        colnames.join(".")
    }

    fn get_contents(&self) -> &HashMap<(String, String), Vec<Option<String>>> {
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
        assert!(agg.get_contents().contains_key(&("Columbus.OH".to_string(), "Blue Jackets.Hockey".to_string())));
    }
}