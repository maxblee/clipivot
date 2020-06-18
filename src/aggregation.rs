//! The module that actually aggregates the records.
//!
//! It has three main methods: `new`, which initializes the data; `aggregate`, which takes
//! a csv file and creates an `IndexMap` of `Accumulator`s; and `write_results` which
//! outputs the aggregated values to standard output.
use crate::aggfunc::Accumulate;
use crate::errors::{CsvCliError, CsvCliResult};
use crate::parsing::INPUT_DATE_FORMAT;
use indexmap::set::IndexSet;
use once_cell::sync::Lazy;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io;
use std::marker::PhantomData;

const FIELD_SEPARATOR: &str = "_<sep>_";
static EMPTY_VALUES: Lazy<HashSet<&str>> = Lazy::new(|| {
    let mut vals = HashSet::new();
    for val in &["", "null", "nan", "none", "na", "n/a"] {
        vals.insert(*val);
    }
    vals
});

/// How the rows or columns are going to be sorted
#[derive(Debug, PartialEq)]
pub enum OutputOrder {
    /// Results appear in index order
    IndexOrder,
    /// The results appear sorted in ascending order
    Ascending,
    /// The results appear sorted in descending order
    Descending,
}

/// The general type of data being used. I've used this to implement better error handling.
/// See [the GitHub](https://github.com/maxblee/clipivot#functions) page for more details on the
/// meaning of these functions.
#[derive(Debug, PartialEq)]
pub enum ParsingStrategy {
    /// For accumulators that hold and manipulate text (string) data.
    Text,
    /// For accumulators that manipulate numeric data (Decimal and floating point types)
    Numeric,
    /// For accumulators that manipulate dates
    Date,
    // design from https://docs.rs/csv/1.1.3/src/csv/error.rs.html#61-108
    #[doc(hidden)]
    __Nonexhaustive,
}

/// The object that computes the aggregations and writes to standard output.
#[derive(Debug, PartialEq)]
pub struct Aggregator<T, I, O>
where
    T: Accumulate<I, O>,
    I: std::str::FromStr,
    O: std::fmt::Display,
{
    aggregations: HashMap<(String, String), T>,
    indexes: IndexSet<String>,
    columns: IndexSet<String>,
    index_cols: Vec<usize>,
    column_cols: Vec<usize>,
    values_col: usize,
    skip_null: bool,
    row_order: OutputOrder,
    column_order: OutputOrder,
    parsing_strategy: ParsingStrategy,
    input_type: PhantomData<I>,
    output_type: PhantomData<O>,
}

impl<T, I, O> Aggregator<T, I, O>
where
    T: Accumulate<I, O>,
    I: std::str::FromStr,
    O: std::fmt::Display,
{
    pub fn new(
        index_cols: Vec<usize>,
        column_cols: Vec<usize>,
        values_col: usize,
        skip_null: bool,
        row_order: OutputOrder,
        column_order: OutputOrder,
        parsing_strategy: ParsingStrategy,
    ) -> Aggregator<T, I, O> {
        let aggregations = HashMap::new();
        let indexes = IndexSet::new();
        let columns = IndexSet::new();
        Aggregator {
            aggregations,
            indexes,
            columns,
            index_cols,
            column_cols,
            values_col,
            skip_null,
            row_order,
            column_order,
            parsing_strategy,
            input_type: PhantomData,
            output_type: PhantomData,
        }
    }

    /// Takes a CSV (in standard input or in an actual file) and aggregates information
    /// based on the struct's settings. Does not actually write the data.
    pub fn aggregate<R: std::io::Read>(&mut self, mut rdr: csv::Reader<R>) -> CsvCliResult<()> {
        let mut line_num = 0;
        let mut record = csv::StringRecord::new();
        while rdr.read_record(&mut record)? {
            self.add_record(&record, line_num)?;
            line_num += 1;
        }
        Ok(())
    }

    /// Writes the aggregated information to standard output.
    pub fn write_results(&mut self) -> CsvCliResult<()> {
        if self.columns.is_empty() {
            return Err(CsvCliError::InvalidConfiguration(
                "Did not parse any lines before finishing".to_string(),
            ));
        }
        self.sort_results();
        let mut writer = csv::Writer::from_writer(io::stdout());
        let mut header = vec![""];
        for col in &self.columns {
            header.push(col);
        }
        writer.write_record(header)?;
        for row in &self.indexes {
            let mut record = vec![row.to_string()];
            for col in &self.columns {
                let cell = self
                    .aggregations
                    .get(&(row.to_string(), col.to_string()))
                    .map_or(String::new(), |v| {
                        v.compute()
                            .map(|v| v.to_string())
                            .unwrap_or_else(String::new)
                    });
                record.push(cell);
            }
            writer.write_record(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: &csv::StringRecord, line_num: usize) -> CsvCliResult<()> {
        let value_string = record.get(self.values_col).unwrap();
        if !(self.skip_null && EMPTY_VALUES.contains(value_string.to_ascii_lowercase().as_str())) {
            let index_vals = self.get_column_string(&self.index_cols, record);
            self.indexes.insert(index_vals.clone());
            let column_vals = self.get_column_string(&self.column_cols, record);
            self.columns.insert(column_vals.clone());
            self.update_aggregations(index_vals, column_vals, value_string, line_num)?;
        }
        Ok(())
    }

    fn get_column_string(&self, columns: &[usize], record: &csv::StringRecord) -> String {
        if columns.is_empty() {
            return "total".to_string();
        }
        let mut column_records = Vec::new();
        for column in columns {
            let string_val = record.get(*column).unwrap();
            column_records.push(string_val.to_string());
        }
        column_records.join(FIELD_SEPARATOR)
    }

    fn describe_err(&self) -> String {
        match self.parsing_strategy {
            ParsingStrategy::Text => "Failed to parse as text".to_string(),
            ParsingStrategy::Numeric => "Failed to parse as numeric".to_string(),
            ParsingStrategy::Date => format!(
                "Could not parse as date with {} format",
                INPUT_DATE_FORMAT.get().unwrap()
            ),
            _ => "Generic parsing error".to_string(),
        }
    }

    fn update_aggregations(
        &mut self,
        indexname: String,
        columnname: String,
        input_str: &str,
        line_num: usize,
    ) -> CsvCliResult<()> {
        let parsed_val = input_str.parse().or_else(|_| {
            Err(CsvCliError::ParsingError {
                line_num,
                str_to_parse: input_str.to_string(),
                err: self.describe_err(),
            })
        })?;

        match self.aggregations.entry((indexname, columnname)) {
            Entry::Occupied(entry) => {
                entry.into_mut().update(parsed_val);
            }
            Entry::Vacant(entry) => {
                entry.insert(T::new(parsed_val));
            }
        };

        Ok(())
    }

    fn sort_results(&mut self) {
        match self.column_order {
            OutputOrder::Ascending => self.columns.sort(),
            OutputOrder::Descending => self.columns.sort_by(|a, b| b.cmp(a)),
            OutputOrder::IndexOrder => {}
        };
        match self.row_order {
            OutputOrder::Ascending => self.indexes.sort(),
            OutputOrder::Descending => self.indexes.sort_by(|a, b| b.cmp(a)),
            OutputOrder::IndexOrder => {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggfunc::Count;
    use csv::StringRecord;
    use indexmap::IndexSet;

    fn setup_simple() -> Aggregator<Count<String>, String, usize> {
        Aggregator::new(
            vec![0, 2],
            vec![3, 4],
            1,
            false,
            OutputOrder::IndexOrder,
            OutputOrder::Ascending,
            ParsingStrategy::Text,
        )
    }

    #[allow(unused_must_use)]
    #[test]
    fn test_add_record() {
        let mut agg = setup_simple();
        let record_vec = vec!["Columbus", "Playoffs", "OH", "Blue Jackets", "Hockey"];
        let csv_record = StringRecord::from(record_vec);
        agg.add_record(&csv_record, 0).unwrap();
        let expected_record = (
            "Columbus_<sep>_OH".to_string(),
            "Blue Jackets_<sep>_Hockey".to_string(),
        );
        assert!(agg.aggregations.contains_key(&expected_record));
        assert_eq!(
            agg.aggregations.get(&expected_record).unwrap().compute(),
            Some(1)
        );
        let mut expected_indexes = IndexSet::new();
        expected_indexes.insert("Columbus_<sep>_OH".to_string());
        assert_eq!(agg.indexes, expected_indexes);
        let mut expected_columns = IndexSet::new();
        expected_columns.insert("Blue Jackets_<sep>_Hockey".to_string());
        assert_eq!(agg.columns, expected_columns);
    }

    #[allow(unused_must_use)]
    #[test]
    fn test_no_columnnames() {
        let mut agg: Aggregator<Count<String>, String, usize> = Aggregator::new(
            vec![],
            vec![],
            0,
            false,
            OutputOrder::IndexOrder,
            OutputOrder::Ascending,
            ParsingStrategy::Text,
        );
        let record_vec = StringRecord::from(vec!["hello"]);
        agg.add_record(&record_vec, 0);
        let new_record = StringRecord::from(vec!["goodbye"]);
        agg.add_record(&new_record, 1);
        let mut expected_indexes = IndexSet::new();
        expected_indexes.insert("total".to_string());
        assert_eq!(expected_indexes, agg.indexes);
        let mut expected_columns = IndexSet::new();
        expected_columns.insert("total".to_string());
        assert_eq!(expected_columns, agg.columns);
        let count = agg
            .aggregations
            .get(&("total".to_string(), "total".to_string()));
        assert_eq!(count.unwrap().compute(), Some(2));
    }

    #[test]
    fn test_no_vals_is_error() {
        let mut agg = setup_simple();
        assert!(agg.write_results().is_err());
    }
}
