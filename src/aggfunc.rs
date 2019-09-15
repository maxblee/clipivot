//! The `aggfunc` module is the part of the code base for `clipivot` that
//! applies a function to the values column of the data.
//!
//! All of the functions rely on the `AggregationMethod` trait in order to operate.
//! The basic concept of that trait is simple: Each aggregation function must have
//! a way of defining what type of function it is, using the `AggTypes` enum and declaring
//! its type using the `get_aggtypes` method. They must also have a way of creating a new object
//! and a way of updating when new data comes in. And they must have
//! a way of converting from a `struct` into a final string output, using `to_output`.
//!
//! You can find more details in both the `AggTypes` enum and the `AggregationMethod` API. But as you might
//! imagine from that description, the impetus behind this design is to allow you to form an
//! aggregation using as little memory as possible. The design is also intended to allow you to create
//! aggregations from streaming records in a single pass, when possible, while also allowing for some amount
//! of flexibility in cases where a single-pass algorithm doesn't make sense because of memory constraints
//! or because it's impossible to implement. And of all these functions, the only one that does not
//! operate in a single pass (or in linear time) is the median, which forms a BTreeMap and then loops over
//! the sorted values until it finds the midpoint.
//!
//! With that in mind, if you want to build a new function, you need to do the following things:
//! 1. Add an enum to AggTypes.
//! 2. Create a new struct that implements the `AggregationMethod` trait.
//! 3. Update `cli.yml` so people can enter the name of your new function from
//! the command-line without getting an error message.
//! 4. Update the `run` method in `aggregation.rs` so your program will actually do something when you
//! write the function name in the command line. This should be as simple as adding
//! ```rust
//! else if aggfunc == "mynewfunction" {
//!     let mut config : CliConfig<MyNewFunction> = CliConfig::from_arg_matches(arg_matches)?;
//!     config.run_config()?;
//! }
//! ```
//! to the bottom of `run`.
//!
//! 5. Update the `get_parsing_approach` method within `CliConfig` so that the parsing
//! struct attached to `clipivot` knows how to interpret new records. (In order to make sense
//! of this struct, you'll probably need to look at the `AggregationMethod` API and possibly
//! the documentation for the `parsing` module.)

use crate::parsing::ParsingType;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};

const DATEFORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// An enum designed to list all of the possible types of aggregation functions.
///
/// Each aggregation method should have an associated enum value. For instance,
/// the `Count` struct, which implements AggregationMethod, has an associated
/// `Count` value in `AggTypes`.
#[derive(Debug, PartialEq)]
pub enum AggTypes {
    /// Counts the number of records it encounters.
    Count,
    /// Counts the number of unique records.
    CountUnique,
    /// Computes the maximum value of the records, or the most recent date.
    Maximum,
    /// Computes a mean of the records.
    Mean,
    /// Computes the median of the records.
    Median,
    /// Computes the minimum value of the records.
    Minimum,
    /// Computes the mode, in insertion order.
    Mode,
    /// Finds the difference between the minimum and maximum value.
    Range,
    /// Sums the records.
    Sum,
    /// Computes the sample standard deviation of the matching records.
    ///
    /// If there are fewer than two matching records (i.e. 0 or 1 matching records),
    /// returns an empty string.
    StdDev,
}

/// All aggregation methods must implement the `AggregationMethod` trait. (See the
/// description at the top of the `aggfunc` module page for instructions on how to
/// implement a new aggregation method.)
///
/// The trait has four required functions, in addition to a required type parameter.
/// You must implement a `new` method. The main `Aggregation` structure implements
/// this method when it is trying to access a cell in the aggregated pivot table
/// that does not yet exist.  **Note that the new method creates the first
/// record, in addition to initializing the object.**
///
/// For example, say you are implementing the `Count` structure
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

pub trait AggregationMethod {
    /// The name of the function (e.g. Count for `Count`).
    type Aggfunc;

    /// Returns the Aggregation method (e.g. AggTypes::Count)
    fn get_aggtype() -> AggTypes;
    /// Instantiates a new AggregationMethod object. In addition, this method
    /// adds the first record into this object
    fn new(parsed_val: &ParsingType) -> Self;
    /// Updates an existing record with a new record.
    fn update(&mut self, parsed_val: &ParsingType);
    /// Converts to a `String` output so the value can be written to standard output.
    ///
    /// In some cases, this just means implementing `self.data.to_string()`, but in others,
    /// it can be more complicated. For instance, standard deviation has to compute a final standard
    /// devation, returning an empty string if there aren't enough records. And median
    /// has to iterate through the sorted records it accumulated.
    fn to_output(&self) -> String;
}

/// The AggregationMethod for computing range.
///
/// This method returns the difference between the minimum and maximum values
/// accumulated. In the case of numeric data, the meaning of this should be obvious.
/// In the case of datetimes, the method returns the total number of days between the minimum
/// and maximum values, as a floating point number (allowing you to estimate the total number of
/// seconds/minutes/hours).
pub struct Range {
    /// The minimum value, or the earliest appearing date
    min_val: ParsingType,
    /// The maximum value, or the most recently occurring date.
    max_val: ParsingType,
}

impl AggregationMethod for Range {
    type Aggfunc = Range;

    fn get_aggtype() -> AggTypes {
        AggTypes::Range
    }

    fn new(parsed_val: &ParsingType) -> Self {
        let (min_val, max_val) = match parsed_val {
            ParsingType::Numeric(Some(new_val)) => (
                ParsingType::Numeric(Some(*new_val)),
                ParsingType::Numeric(Some(*new_val)),
            ),
            ParsingType::DateTypes(Some(new_val)) => (
                ParsingType::DateTypes(Some(*new_val)),
                ParsingType::DateTypes(Some(*new_val)),
            ),
            _ => (ParsingType::Numeric(None), ParsingType::Numeric(None)),
        };
        Range { min_val, max_val }
    }

    fn update(&mut self, parsed_val: &ParsingType) {
        match (&self.min_val, &self.max_val, parsed_val) {
            (
                ParsingType::Numeric(Some(min)),
                ParsingType::Numeric(Some(max)),
                ParsingType::Numeric(Some(new_val)),
            ) => {
                if new_val > max {
                    self.max_val = ParsingType::Numeric(Some(*new_val));
                } else if new_val < min {
                    self.min_val = ParsingType::Numeric(Some(*new_val));
                }
            }
            (
                ParsingType::DateTypes(Some(min)),
                ParsingType::DateTypes(Some(max)),
                ParsingType::DateTypes(Some(new_val)),
            ) => {
                if new_val > max {
                    self.max_val = ParsingType::DateTypes(Some(*new_val));
                } else if new_val < min {
                    self.min_val = ParsingType::DateTypes(Some(*new_val));
                }
            }
            _ => {}
        }
    }

    fn to_output(&self) -> String {
        match (&self.min_val, &self.max_val) {
            (ParsingType::Numeric(Some(min)), ParsingType::Numeric(Some(max))) => {
                let range = max.checked_sub(*min).unwrap();
                range.to_string()
            }
            (ParsingType::DateTypes(Some(min)), ParsingType::DateTypes(Some(max))) => {
                let duration = max.signed_duration_since(*min);
                let days = Decimal::new(duration.num_seconds(), 1)
                    .checked_div(Decimal::new(86400, 1))
                    .unwrap();
                // let days = duration.num_seconds() as f64 / 86400.;
                // format!("{}", days)
                days.to_string()
            }
            _ => "".to_string(),
        }
    }
}

/// This AggregationMethod computes the maximum number it sees, or the most recently occurring date.
pub struct Maximum {
    /// The maximum value, among values it has seen so far.
    max_val: ParsingType,
}

impl AggregationMethod for Maximum {
    type Aggfunc = Maximum;

    fn get_aggtype() -> AggTypes {
        AggTypes::Maximum
    }

    fn new(parsed_val: &ParsingType) -> Self {
        let max_val = match parsed_val {
            ParsingType::Numeric(Some(val)) => ParsingType::Numeric(Some(*val)),
            ParsingType::DateTypes(Some(dt)) => ParsingType::DateTypes(Some(*dt)),
            ParsingType::Text(Some(string_date)) => {
                ParsingType::Text(Some(string_date.to_string()))
            }
            _ => ParsingType::Numeric(None),
        };

        Maximum { max_val }
    }

    fn update(&mut self, parsed_val: &ParsingType) {
        match (&self.max_val, parsed_val) {
            (ParsingType::Numeric(Some(max)), ParsingType::Numeric(Some(cur))) => {
                if cur > max {
                    self.max_val = ParsingType::Numeric(Some(*cur));
                }
            }
            (ParsingType::DateTypes(Some(max)), ParsingType::DateTypes(Some(cur))) => {
                if cur > max {
                    self.max_val = ParsingType::DateTypes(Some(*cur));
                }
            }
            (ParsingType::Text(Some(max)), ParsingType::Text(Some(cur))) => {
                if cur > max {
                    self.max_val = ParsingType::Text(Some(cur.to_string()));
                }
            }
            _ => {}
        }
    }

    fn to_output(&self) -> String {
        match &self.max_val {
            ParsingType::Numeric(Some(val)) => val.to_string(),
            ParsingType::DateTypes(Some(dt)) => format!("{}", dt.format(DATEFORMAT)),
            ParsingType::Text(Some(string_date)) => string_date.to_string(),
            _ => "".to_string(),
        }
    }
}

/// This computes the minimum value, or the oldest date.
pub struct Minimum {
    /// The current minimum
    min_val: ParsingType,
}

impl AggregationMethod for Minimum {
    type Aggfunc = Minimum;

    fn get_aggtype() -> AggTypes {
        AggTypes::Minimum
    }

    fn new(parsed_val: &ParsingType) -> Self {
        // Minimum needs to be more inclusive in its matching than other methods
        // because multiple ParsingTypes work with it
        let min_val = match parsed_val {
            ParsingType::Numeric(Some(val)) => ParsingType::Numeric(Some(*val)),
            ParsingType::DateTypes(Some(dt)) => ParsingType::DateTypes(Some(*dt)),
            ParsingType::Text(Some(str_val)) => ParsingType::Text(Some(str_val.to_string())),
            _ => ParsingType::Numeric(None),
        };

        Minimum { min_val }
    }

    fn update(&mut self, parsed_val: &ParsingType) {
        match (&self.min_val, parsed_val) {
            (ParsingType::Numeric(Some(min)), ParsingType::Numeric(Some(cur))) => {
                if cur < min {
                    self.min_val = ParsingType::Numeric(Some(*cur));
                }
            }
            (ParsingType::DateTypes(Some(min)), ParsingType::DateTypes(Some(cur))) => {
                if cur < min {
                    self.min_val = ParsingType::DateTypes(Some(*cur));
                }
            }
            (ParsingType::Text(Some(min)), ParsingType::Text(Some(cur))) => {
                if cur < min {
                    self.min_val = ParsingType::Text(Some(cur.to_string()));
                }
            }
            _ => {}
        }
    }

    fn to_output(&self) -> String {
        match &self.min_val {
            ParsingType::Numeric(Some(val)) => val.to_string(),
            ParsingType::DateTypes(Some(dt)) => format!("{}", dt.format(DATEFORMAT)),
            ParsingType::Text(Some(val)) => val.to_string(),
            _ => "".to_string(),
        }
    }
}

/// This counts the total number of records.
#[derive(Debug, PartialEq)]
pub struct Count {
    /// Represents the number of matching records
    val: usize,
}

impl AggregationMethod for Count {
    type Aggfunc = Count;

    fn get_aggtype() -> AggTypes {
        AggTypes::Count
    }
    fn new(_parsed_val: &ParsingType) -> Self {
        Count { val: 1 }
    }
    fn update(&mut self, _parsed_val: &ParsingType) {
        self.val += 1;
    }
    fn to_output(&self) -> String {
        self.val.to_string()
    }
}

/// This counts the total number of unique records aggregated.
pub struct CountUnique {
    /// A HashSet containing all of the unique values encountered so far
    vals: HashSet<String>,
}

impl AggregationMethod for CountUnique {
    type Aggfunc = CountUnique;

    fn get_aggtype() -> AggTypes {
        AggTypes::CountUnique
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Text(Some(str_val)) => {
                let mut vals = HashSet::new();
                vals.insert(str_val.to_string());
                CountUnique { vals }
            }
            _ => {
                let mut vals = HashSet::new();
                vals.insert("".to_string());
                CountUnique { vals }
            }
        }
    }

    fn update(&mut self, parsed_val: &ParsingType) {
        let val = match parsed_val {
            ParsingType::Text(Some(new_val)) => new_val.to_string(),
            _ => "".to_string(),
        };
        self.vals.insert(val);
    }

    fn to_output(&self) -> String {
        self.vals.len().to_string()
    }
}

/// Sums up all of the values among the aggregated records.
pub struct Sum {
    /// The current total. Uses a Decimal type to avoid floating point truncation errors.
    cur_total: Decimal,
}

impl AggregationMethod for Sum {
    type Aggfunc = Sum;

    fn get_aggtype() -> AggTypes {
        AggTypes::Sum
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => Sum { cur_total: *num },
            // Note: I really need to make this more robust
            _ => Sum {
                cur_total: Decimal::new(0, 0),
            },
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        let total = self.cur_total.checked_add(match parsed_val {
            ParsingType::Numeric(Some(num)) => *num,
            _ => Decimal::new(0, 0),
        });
        // Again, need to make this more robust. (Maybe-- really just make better panic)
        self.cur_total = total.unwrap();
    }
    fn to_output(&self) -> String {
        self.cur_total.to_string()
    }
}

/// Computes the *sample* standard deviation in a single pass, using
/// [Welford's algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm).

/// The attributes in this method refer to the same ones described in
/// *Accuracy and Stability of Numerical Algorithms* by Higham (2nd Edition, page 11).
pub struct StdDev {
    // solution from Nicholas Higham: Accuracy and Stability of Numerical Algorithms
    // Second Edition, 2002, p. 11
    q: f64,
    m: f64,
    /// The number of records parsed so far
    num_records: f64,
}

impl AggregationMethod for StdDev {
    type Aggfunc = StdDev;

    fn get_aggtype() -> AggTypes {
        AggTypes::StdDev
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::FloatingPoint(Some(num)) => StdDev {
                q: 0.,
                m: *num,
                num_records: 1.,
            },
            _ => StdDev {
                q: 0.,
                m: 0.,
                num_records: 0.,
            },
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        if let ParsingType::FloatingPoint(Some(num)) = parsed_val {
            self.num_records += 1.;
            let squared_diff = (num - self.m).powi(2);
            self.q += ((self.num_records - 1.) * squared_diff) / self.num_records;
            self.m += (num - self.m) / self.num_records;
        }
    }

    fn to_output(&self) -> String {
        let stdev = self.compute();
        stdev.map_or("".to_string(), |v| v.to_string())
    }
}

impl StdDev {
    /// Computes the final standard deviation after finishing updating. A private function for `to_output`.
    fn compute(&self) -> Option<f64> {
        // we do the if statement and return Option to avoid divide by 0 error
        if self.num_records <= 1. {
            None
        } else {
            let variance = self.q / (self.num_records - 1.);
            let stdev = variance.sqrt();
            if stdev.is_nan() {
                None
            } else {
                Some(stdev)
            }
        }
    }
}

/// Computes the mean from a stream. Effectively, this algorithm is the same as `Sum` except it
/// keeps count of the total number of records parsed for use in the final computation.
pub struct Mean {
    num: usize,
    cur_total: Decimal,
}

impl AggregationMethod for Mean {
    type Aggfunc = Mean;

    fn get_aggtype() -> AggTypes {
        AggTypes::Mean
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => Mean {
                cur_total: *num,
                num: 1,
            },
            // This will never be implemented, but it's needed bc compiler can't tell that
            _ => Mean {
                cur_total: Decimal::new(0, 0),
                num: 0,
            },
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        let total = self.cur_total.checked_add(match parsed_val {
            ParsingType::Numeric(Some(num)) => *num,
            _ => Decimal::new(0, 0),
        });
        // Unwrap is OK because ParsingType should always be Some when `update` is implemented
        self.cur_total = total.unwrap();
        self.num += 1;
    }
    fn to_output(&self) -> String {
        // Note: unwrap is OK here because self.num can never be 0
        // so this should theoretically never panic
        self.compute().to_string()
    }
}

impl Mean {
    /// A helper function for computing the mean at the end of the program.
    fn compute(&self) -> Decimal {
        self.cur_total
            .checked_div(Decimal::new(self.num as i64, 0))
            .unwrap()
    }
}

/// Computes the mode.
pub struct Mode {
    /// All of the values parsed so far. Uses a HashMap (effectively a histogram) to conserve on memory
    values: HashMap<String, usize>,
    /// The maximum number of repeated records
    max_count: usize,
    /// The current mode
    max_val: String,
}

impl AggregationMethod for Mode {
    type Aggfunc = Mode;

    fn get_aggtype() -> AggTypes {
        AggTypes::Mode
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Text(Some(val)) => {
                let mut init_val = HashMap::new();
                init_val.insert(val.to_string(), 1);
                Mode {
                    values: init_val,
                    max_count: 1,
                    max_val: val.to_string(),
                }
            }
            _ => Mode {
                values: HashMap::new(),
                max_count: 0,
                max_val: "".to_string(),
            },
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        let entry = match parsed_val {
            ParsingType::Text(Some(val)) => val.to_string(),
            _ => "".to_string(),
        };
        // barely adapted from https://docs.rs/indexmap/1.0.2/indexmap/map/struct.IndexMap.html
        let new_count = *self.values.get(&entry).unwrap_or(&0) + 1;
        if new_count > self.max_count {
            self.max_count = new_count;
            self.max_val = entry.clone();
        }
        *self.values.entry(entry).or_insert(0) += 1;
    }

    fn to_output(&self) -> String {
        self.max_val.clone()
    }
}

/// Computes the median by building a sorted B-Tree map and iterating over the records.
///
/// In best case and average case, this should be lighter on memory than holding all of the records
/// in two separate heaps, although it is worse than the heap methods in worst case. Additionally, this is
/// the only algorithm that currently is not computed in a single pass, and it's the only algorithm that is
/// not computed in linear time. That said, it theoretically allows for creating aggregations from data that
/// exceeds RAM.
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
    /// A map of all of the values parsed so far to the number of times they've appeared.
    /// The B-Tree creates a sorted map that speeds up on iteration time in the second pass
    values: BTreeMap<Decimal, usize>,
    /// The total number of records parsed. Equivalent to a sum of all of the counts in `self.values`.
    num: usize,
}

impl AggregationMethod for Median {
    type Aggfunc = Median;

    fn get_aggtype() -> AggTypes {
        AggTypes::Median
    }
    fn new(parsed_val: &ParsingType) -> Self {
        match parsed_val {
            ParsingType::Numeric(Some(num)) => {
                let mut b = BTreeMap::new();
                b.insert(*num, 1);
                Median { values: b, num: 1 }
            }
            _ => Median {
                values: BTreeMap::new(),
                num: 0,
            },
        }
    }
    fn update(&mut self, parsed_val: &ParsingType) {
        self.values
            .entry(match parsed_val {
                ParsingType::Numeric(Some(num)) => *num,
                _ => Decimal::new(0, 0),
            })
            .and_modify(|val| *val += 1)
            .or_insert(1);
        self.num += 1;
    }
    fn to_output(&self) -> String {
        self.compute().to_string()
    }
}

impl Median {
    fn compute(&self) -> Decimal {
        // we set up a running count to track where our index would be were this a sorted vec
        // instead of a sorted histogram
        let mut cur_count = 0;
        let mut cur_val = Decimal::new(0, 0);
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
        if (self.num % 2) == 0
            && ((cur_count as f64) - (self.num as f64 / 2.)).abs() < std::f64::EPSILON
        {
            // iter.next() will always be Some(_) because this is always initialized with
            // a single value
            // checked_add I should maybe find a robust alternative to unwrap for
            // checked_div will only panic if checked_add panics or if other == Decimal::new(0, 0)
            // which it does not
            cur_val
                .checked_add(*iter.next().unwrap().0)
                .unwrap()
                .checked_div(Decimal::new(2, 0))
                .unwrap()
        } else {
            cur_val
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use chrono::NaiveDate;
    use rand::prelude::*;
    use std::str::FromStr;

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
        assert_eq!(uncount.to_output(), "1".to_string());
    }

    #[test]
    fn multiple_identical_records_read_as_one() {
        let myrecord = &ParsingType::Text(Some("record".to_string()));
        let mut uncount = CountUnique::new(myrecord);
        uncount.update(myrecord);
        assert_eq!(uncount.vals.len(), 1);
        assert_eq!(uncount.to_output(), "1".to_string());
    }

    #[test]
    fn different_records_read_as_different() {
        let record1 = &ParsingType::Text(Some("record".to_string()));
        let record2 = &ParsingType::Text(Some("new record".to_string()));
        let mut uncount = CountUnique::new(record1);
        uncount.update(record2);
        assert_eq!(uncount.vals.len(), 2);
        assert_eq!(uncount.to_output(), "2".to_string());
    }

    // testing standard deviation performance
    #[test]
    fn stdev_computation_is_stable() {
        // numerical stability can be a problem in some computations of standard deviation,
        // causing catastrophic cancellation errors.
        // (See https://www.johndcook.com/blog/2008/09/28/theoretical-explanation-for-numerical-results/.)
        // But with the algorithm we're using we should see minimal losses from cancellation.
        // This tests that; this edge case comes from
        // https://www.johndcook.com/blog/2008/09/26/comparing-three-methods-of-computing-standard-deviation/
        let large_num = 1e9;
        let randnum: f64 = random();
        // taking a standard deviation of 10^6 random (0,1) values
        // shouldn't suffer catastrophic cancellation, so we'll use as baseline
        let small_rand = ParsingType::FloatingPoint(Some(randnum));
        let mut decent_stddev = StdDev::new(&small_rand);
        // adding 10^9 to each value (0,1) could cause catastrophic cancellation in bad
        // standard deviation implementations
        let init_parsing = ParsingType::FloatingPoint(Some(randnum + large_num));
        let mut error_prone_stddev = StdDev::new(&init_parsing);
        for _ in 1..=1000000 {
            let randnum: f64 = random();
            let new_large = ParsingType::FloatingPoint(Some(randnum + large_num));
            let new_small = ParsingType::FloatingPoint(Some(randnum));
            decent_stddev.update(&new_small);
            error_prone_stddev.update(&new_large);
        }
        // checks that the two standard deviations are equal to within 7 significant digits
        // From what I've seen, it'll typically pass within 8 sig digits, but occasionally will fail there
        // EXAMPLE: left = 0.28864050983648876, right = 0.28864049865571434
        assert_abs_diff_eq!(
            decent_stddev.compute().unwrap(),
            error_prone_stddev.compute().unwrap(),
            epsilon = 1e-7
        );
    }

    #[test]
    fn test_one_value_stddev_is_empty() {
        let dev = StdDev::new(&ParsingType::FloatingPoint(Some(1.0)));
        assert_eq!(dev.to_output(), "".to_string());
    }

    // test median
    #[test]
    fn test_single_median_returns_value() {
        // tests the accuracy of the median on a single value
        let init_val = ParsingType::Numeric(Some(Decimal::from_str("1").unwrap()));
        let single_median = Median::new(&init_val);
        assert_eq!(single_median.compute().to_string(), "1");
    }

    #[test]
    fn test_even_median_returns_mean_middle() {
        // makes sure that when there is an even number of values in a dataset,
        // and records[floor(n/2)] and records[ceil(n/2)] are different,
        // median computes the middle value
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("0").unwrap()));
        let mut median = Median::new(&init_parsing);
        let addl_vals = vec!["1", "2", "3"];
        for val in addl_vals {
            let parsed_val = ParsingType::Numeric(Some(Decimal::from_str(val).unwrap()));
            median.update(&parsed_val);
        }
        assert_eq!(median.compute().to_string(), "1.5");
    }

    #[test]
    fn test_odd_median_returns_middle_value() {
        // makes sure that in an odd, unsorted set of records, it'll return the middle value
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("3").unwrap()));
        let mut median = Median::new(&init_parsing);
        let addl_vals = vec!["1", "9", "7", "8", "5", "2", "5", "5"];
        for val in addl_vals {
            let parsed_val = ParsingType::Numeric(Some(Decimal::from_str(val).unwrap()));
            median.update(&parsed_val);
        }
        assert_eq!(median.compute().to_string(), "5");
    }

    #[test]
    fn test_multiple_middle_values_returns_middle_val() {
        // makes sure the median returns correctly when there are repeat values in the middle
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("3").unwrap()));
        let mut median = Median::new(&init_parsing);
        let add_vals = vec!["5", "6", "1", "4", "3"];
        for val in add_vals {
            let parsed_val = ParsingType::Numeric(Some(Decimal::from_str(val).unwrap()));
            median.update(&parsed_val);
        }
        assert_eq!(median.compute().to_string(), "3.5");
    }

    // test summation
    #[test]
    fn test_truncation_sum() {
        // in floating point numbers, 0.1 + 0.2 != 0.3
        // But Decimal types should have more numeric stability
        // This makes sure the Decimal typing is properly set up
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("0.1").unwrap()));
        let mut summation = Sum::new(&init_parsing);
        let new_parsing = ParsingType::Numeric(Some(Decimal::from_str("0.2").unwrap()));
        summation.update(&new_parsing);
        assert_eq!(summation.to_output(), "0.3");
    }

    #[test]
    fn test_single_sum_returns_self() {
        // makes sure that a single aggregated value `x` will return x
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("10").unwrap()));
        let summation = Sum::new(&init_parsing);
        assert_eq!(summation.to_output(), "10");
    }

    #[test]
    fn test_simple_addition() {
        // tests simple summation capability, making sure int + float works
        let init_parsing = ParsingType::Numeric(Some(Decimal::from_str("10").unwrap()));
        let mut summation = Sum::new(&init_parsing);
        let addl_vals = vec!["0.3", "100", "3.2"];
        for val in addl_vals {
            let parsed_val = ParsingType::Numeric(Some(Decimal::from_str(val).unwrap()));
            summation.update(&parsed_val);
        }
        assert_eq!(summation.to_output(), "113.5");
    }

    // mode calculation
    #[test]
    fn test_mode_on_one_value() {
        // makes sure initializing properly enters a new value
        let init_parsing = ParsingType::Text(Some(String::from("a")));
        let mode = Mode::new(&init_parsing);
        assert_eq!(mode.to_output(), String::from("a"));
    }
    #[test]
    fn test_mode_returns_most_commonly_appearing_value() {
        // tests the basic functionality of mode
        let init_parsing = ParsingType::Text(Some(String::from("a")));
        let new_vals = vec!["b".to_string(), "c".to_string(), "a".to_string()];
        let mut mode = Mode::new(&init_parsing);
        for val in new_vals {
            let parsed_val = ParsingType::Text(Some(val));
            mode.update(&parsed_val);
        }
        assert_eq!(mode.to_output(), String::from("a"));
    }
    #[test]
    fn test_mode_returns_first_initialized_on_tie() {
        // makes sure that if two values appear the same number of times
        // the returned value is the first value appearing in the data
        for _i in 1..=10000 {
            let init_parsing = ParsingType::Text(Some("a".to_string()));
            let mut mode = Mode::new(&init_parsing);
            let addl_vals = vec!["b".to_string(), "a".to_string(), "b".to_string()];
            for val in addl_vals {
                let parsed_val = ParsingType::Text(Some(val));
                mode.update(&parsed_val);
            }
            assert_eq!(mode.to_output(), "a".to_string());
        }
    }

    #[test]
    fn test_minimum_finds_smallest_val() {
        // tests the Minimum function
        let first_parse = ParsingType::Numeric(Some(Decimal::from_str("5").unwrap()));
        let mut minimum = Minimum::new(&first_parse);
        let all_vals = vec!["12", "12.3", "2", "7.8"];
        for val in all_vals {
            let dectype = Decimal::from_str(val).unwrap();
            let parsing_val = ParsingType::Numeric(Some(dectype));
            minimum.update(&parsing_val);
        }
        assert_eq!(minimum.to_output(), "2".to_string());
    }

    #[test]
    fn test_minimum_string() {
        let str_dates = vec![
            "2010-01-20".to_string(),
            "2010-02-10".to_string(),
            "2009-12-31".to_string(),
        ];
        let mut min_str = Minimum::new(&ParsingType::Text(Some("2010-01-18".to_string())));
        for date in str_dates {
            min_str.update(&ParsingType::Text(Some(date)));
        }
        assert_eq!(min_str.to_output(), "2009-12-31".to_string());
    }

    #[test]
    fn test_maximum_string() {
        let str_dates = vec![
            "2010-01-20".to_string(),
            "2010-02-10".to_string(),
            "2009-12-31".to_string(),
        ];
        let mut max_str = Maximum::new(&ParsingType::Text(Some("2010-01-18".to_string())));
        for date in str_dates {
            max_str.update(&ParsingType::Text(Some(date)));
        }
        assert_eq!(max_str.to_output(), "2010-02-10".to_string());
    }

    fn get_dates_for_date_aggfuncs() -> Vec<ParsingType> {
        // Returns a vector of ParsingType::DateType objects for Range and Min and Max funcs
        let first_parse =
            ParsingType::DateTypes(Some(NaiveDate::from_ymd(2017, 1, 30).and_hms(0, 0, 0)));
        let second_parse =
            ParsingType::DateTypes(Some(NaiveDate::from_ymd(2016, 12, 15).and_hms(0, 0, 0)));
        let third_parse =
            ParsingType::DateTypes(Some(NaiveDate::from_ymd(2016, 12, 15).and_hms(0, 1, 12)));
        vec![first_parse, second_parse, third_parse]
    }

    #[test]
    fn test_minimum_date_finds_earliest_date() {
        let parsed_dates = get_dates_for_date_aggfuncs();
        let mut min_date = Minimum::new(&parsed_dates[0]);
        min_date.update(&parsed_dates[1]);
        min_date.update(&parsed_dates[2]);
        assert_eq!(min_date.to_output(), "2016-12-15 00:00:00".to_string());
    }

    #[test]
    fn test_maximum_date_finds_latest_date() {
        let parsed_dates = get_dates_for_date_aggfuncs();
        let mut max_date = Maximum::new(&parsed_dates[0]);
        max_date.update(&parsed_dates[1]);
        max_date.update(&parsed_dates[2]);
        assert_eq!(max_date.to_output(), "2017-01-30 00:00:00".to_string());
    }

    #[test]
    fn test_range_finds_diff_in_days() {
        // makes sure the range with dates finds the number of days between min and max
        let parsed_dates = get_dates_for_date_aggfuncs();
        let mut range_date = Range::new(&parsed_dates[0]);
        range_date.update(&parsed_dates[1]);
        range_date.update(&parsed_dates[2]);
        assert_eq!(range_date.to_output(), "46".to_string())
    }

    #[test]
    fn test_range_finds_partial_diff() {
        // makes sure the range works properly with HMS (ie DateTime instead of Date)
        let mut range_date = Range::new(&ParsingType::DateTypes(Some(
            NaiveDate::from_ymd(2016, 1, 1).and_hms(1, 12, 13),
        )));
        range_date.update(&ParsingType::DateTypes(Some(
            NaiveDate::from_ymd(2016, 1, 1).and_hms(7, 12, 13),
        )));
        assert_eq!(range_date.to_output(), "0.25".to_string());
    }

    #[test]
    fn test_maximum_finds_largest_val() {
        // tests the maximum function
        let first_parse = ParsingType::Numeric(Some(Decimal::from_str("5").unwrap()));
        let mut maximum = Maximum::new(&first_parse);
        let all_vals = vec!["12", "12.3", "2", "7.8"];
        for val in all_vals {
            let dectype = Decimal::from_str(val).unwrap();
            let parsing_val = ParsingType::Numeric(Some(dectype));
            maximum.update(&parsing_val);
        }
        assert_eq!(maximum.to_output(), "12.3".to_string());
    }

    #[test]
    fn test_range() {
        // tests the range function
        let first_parse = ParsingType::Numeric(Some(Decimal::from_str("5").unwrap()));
        let mut range = Range::new(&first_parse);
        let all_vals = vec!["12", "12.3", "2", "7.8"];
        for val in all_vals {
            let dectype = Decimal::from_str(val).unwrap();
            let parsing_val = ParsingType::Numeric(Some(dectype));
            range.update(&parsing_val);
        }
        assert_eq!(range.to_output(), "10.3".to_string());
    }
}
