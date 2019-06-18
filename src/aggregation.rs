use std::collections::{HashSet, HashMap};

mod errors;
use crate::aggregation::errors::CsvPivotError;

pub enum ParsingType {
    Text(Option<String>)
}

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
    fn update_aggregations(&mut self, indexname: String, columnname: String, parsed_val: &ParsingType) {
        // modified from
        // https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| val.update(parsed_val))
            .or_insert(T::new(parsed_val));
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
}