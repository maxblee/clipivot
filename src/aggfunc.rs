//! This is the module for creating new aggregation functions.
//!
//! This functionality centers around the `Aggregation` trait,
//! which implements a number of methods aimed at making it easy
//! to create new aggregation methods without rewriting much code
//! in the main `aggregation` module.
//!
//! The API for the main `AggregationMethod` should provide more information
//! on how to create your own new method.
use std::collections::{BTreeMap, HashSet};
use rust_decimal::Decimal;
use crate::parsing::ParsingType;


/// An enum designed to list all of the possible types of aggregation functions.
///
/// Each aggregation method should have an associated enum value. For instance,
/// the `Count` struct, which implements AggregationMethod, has an associated
/// `Count` value in `AggTypes`.
#[derive(Debug, PartialEq)]
pub enum AggTypes {
    /// for counting records
    Count,
    /// Counts the number of unique records.
    CountUnique,
    /// Computes a mean of the records
    Mean,
    /// Computes the median of the records
    Median,
    /// Sums the records
    Sum,
}

/// All aggregation methods implement the `AggregationMethod` trait.
///
/// If your method does not, the method will not compile without rewriting
/// `main.rs`, the `run` method in `aggregation.rs`, and the `Aggregation`
/// struct that forms the backbone of a lot of this program. In other words, it is
/// imperative that you implement `AggregationMethod` if you want to implement your
/// own new method.
///
/// This trait has four required functions, in addition to a required type parameter.
/// You must implement a `new` method. The main `Aggregation` structure implements
/// this method when it is trying to access a cell in the aggregated pivot table
/// that does not yet exist. For example, say you are implementing the `Count` structure
/// with a csv file that looks like this:
/// ```csv
/// field1,field2,field3
/// a,b,c
/// ```
/// If you decided to take a pivot table using `field1` and `field2` as rows and
/// columns, the `Aggregation` structure would implement `new` on the first row
/// because it does not have any records matching an index of `a` and
/// a column of `b`. Then, `Count` creates a new `Count` type with a value of 1
/// because there is now 1 record that has an index of `a` and a column of `b`.
///
/// The `update` method is implemented when the `Aggregation` structure already
/// has a record of a row-column match existing. So if you had a second row
/// ```csv
/// a,b,d
/// ```
/// in your CSV file, the `Aggregator` struct would implement `update` because it
/// has a record of `(a,b)` existing, and it would change its value to 2.
///
/// Finally, the trait has a `to_output` method. This converts your instance into
/// a String that `Aggregator` can write to standard output.
///
/// Then, there are two last things you need to do in order to create a new method.
/// You need to add a line into the [`run`](../aggregation/fn.run.html) function
/// specifying when the method should be implemented, and you need to add a line into the
/// `cli.yml` file in the `src` directory under `aggfunc` telling the command-line parser
/// that your new function is permissible. The source code under both should make doing that clear,
/// but let me know if you have questions.
pub trait AggregationMethod {
    /// The name of the function (e.g. Count for `Count`).
    type Aggfunc;

    /// Returns the Aggregation method (e.g. AggTypes::Count)
    fn get_aggtype(&self) -> AggTypes;
    /// Instantiates a new AggregationMethod record
    fn new(parsed_val: &ParsingType) -> Self;
    /// Updates an existing record
    fn update(&mut self, parsed_val: &ParsingType);
    /// Converts to a `String` output so the value can be written to standard output
    fn to_output(&self) -> String;
}

/// The aggregation method for counting records.
#[derive(Debug, PartialEq)]
pub struct Count {
    /// Represents the number of matching records
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
    fn to_output(&self) -> String {
        self.val.to_string()
    }
}

pub struct CountUnique {
    vals: HashSet<String>,
}

impl AggregationMethod for CountUnique {
    type Aggfunc = CountUnique;

    fn get_aggtype(&self) -> AggTypes { AggTypes::CountUnique }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Text(Some(str_val)) => {
                let mut vals = HashSet::new();
                vals.insert(str_val.to_string());
                CountUnique {vals}
            },
            _ => {
                let mut vals = HashSet::new();
                vals.insert("".to_string());
                CountUnique {vals}
            }
        }
    }

    fn update(&mut self, parsed_val: &ParsingType) {
        let val = match parsed_val {
            ParsingType::Text(Some(new_val)) => new_val.to_string(),
            _ => "".to_string()
        };
        self.vals.insert(val);
    }

    fn to_output(&self) -> String { self.vals.len().to_string() }
}

pub struct Sum {
    cur_total: Decimal,
}

impl AggregationMethod for Sum {
    type Aggfunc = Sum;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Sum }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => Sum { cur_total: *num },
            // Note: I really need to make this more robust
            _ => Sum {cur_total: Decimal::new(0, 0)}
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        let total = self.cur_total.checked_add(match parsed_val {
            ParsingType::Numeric(Some(num)) => *num,
            _ => Decimal::new(0, 0)
        });
        // Again, need to make this more robust
        self.cur_total = total.unwrap();
    }
    fn to_output(&self) -> String { self.cur_total.to_string() }
}

pub struct Mean {
    num: usize,
    cur_total: Decimal,
}

impl AggregationMethod for Mean {
    type Aggfunc = Mean;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Mean }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => Mean { cur_total: *num, num: 1 },
            // This will never be implemented, but it's needed bc compiler can't tell that
            _ => Mean {cur_total: Decimal::new(0, 0), num: 0 }
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        let total = self.cur_total.checked_add(match parsed_val {
            ParsingType::Numeric(Some(num)) => *num,
            _ => Decimal::new(0, 0)
        });
        // Unwrap is OK because ParsingType should always be Some when `update` is implemented
        self.cur_total = total.unwrap();
        self.num += 1;
    }
    fn to_output(&self) -> String {
        // Note: unwrap is OK here because self.num can never be 0
        // so this should theoretically never panic
        let mean = self.cur_total
            .checked_div(Decimal::new(self.num as i64, 0)).unwrap();
        mean.to_string()
    }
}

pub struct Median {
    // Note: the median implementation I'm using relies on a B-Tree
    // instead of the heap structure described here
    // https://www.geeksforgeeks.org/median-of-stream-of-integers-running-integers/
    // in order to preserve memory.
    // This slows down on CPU performance, but the loss shouldn't be that significant
    // because, typically, the number of bins in a median is going to be far smaller
    // than the number of records. For instance, in a ~1 GB file of yellow taxi cab records
    // from NYC (https://s3.amazonaws.com/nyc-tlc/trip+data/yellow_tripdata_2018-03.csv)
    // the trips have 4,528 separate distance values, out of the 9.5 million records
    values: BTreeMap<Decimal, usize>,
    num: usize,
}

impl AggregationMethod for Median {
    type Aggfunc = Median;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Median }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => {
                let mut b = BTreeMap::new();
                b.insert(*num, 1);
                Median { values: b, num: 1}
            },
            _ => Median { values: BTreeMap::new(), num: 0 }
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        self.values.entry(match  parsed_val {
            ParsingType::Numeric(Some(num)) => *num,
            _ => Decimal::new(0, 0)
        }).and_modify(|val| *val += 1)
            .or_insert(1);
        self.num += 1;
    }
    fn to_output(&self) -> String {
        // we set up a running count to track where our index would be were this a sorted vec
        // instead of a sorted histogram
        let mut cur_count = 0;
        let mut cur_val  = Decimal::new(0, 0);
        // creating an iter bc we're stopping at N/2
        let mut iter = self.values.iter();
        while (cur_count as f64) < (self.num as f64 / 2.) {
            // iter.next() leaves an Option but we're guaranteed to break
            // before iter.next().is_none()
            let (result, count) = iter.next().unwrap();
            // we increase the count to our current index
            // Also, there's got to be a better way to deal with this than by
            // using all this ugly casting
            // theoretically, involving changing count to f64 by default for this reason
            cur_count += count;
            cur_val = *result;
        }
        // now our current value is either at the median or,
        // if the median is a mean of two values, is the lower
        // of the two values at the median
        // It can only be at the lower of the two values if
        // a) we have an even number of records AND b) we didn't pass through
        // the median (where the median would *technically* be the mean of cur_val and cur_val
        if (self.num % 2) == 0 && (cur_count as f64) == (self.num as f64 / 2.) {
            // iter.next() will always be Some(_) because this is always initialized with
            // a single value
            // checked_add I should maybe find a robust alternative to unwrap for
            // checked_div will only panic if checked_add panics or if other == Decimal::new(0, 0)
            // which it does not
            let mean = cur_val.checked_add(*iter.next().unwrap().0).unwrap()
                .checked_div(Decimal::new(2, 0)).unwrap();
            return mean.to_string();
        } else { return cur_val.to_string(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_count_creates_single_count() {
        let c = Count::new(&ParsingType::Text(None));
        assert_eq!(c.val, 1);
    }
    #[test]
    fn adding_multiple_counts_creates_multiple_counts() {
        let mut c = Count::new(&ParsingType::Text(None));
        c.update(&ParsingType::Text(None));
        assert_eq!(c.val, 2);
    }

    #[test]
    fn adding_unique_count_creates_single_count() {
        let uncount = CountUnique::new(&ParsingType::Text(Some("record".to_string())));
        assert_eq!(uncount.vals.len(), 1);
    }

    #[test]
    fn multiple_identical_records_read_as_one() {
        let myrecord = &ParsingType::Text(Some("record".to_string()));
        let mut uncount = CountUnique::new(myrecord);
        uncount.update(myrecord);
        assert_eq!(uncount.vals.len(), 1);
    }

    #[test]
    fn different_records_read_as_different() {
        let record1 = &ParsingType::Text(Some("record".to_string()));
        let record2 = &ParsingType::Text(Some("new record".to_string()));
        let mut uncount = CountUnique::new(record1);
        uncount.update(record2);
        assert_eq!(uncount.vals.len(), 2);
    }
}