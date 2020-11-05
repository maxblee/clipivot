//! The `aggfunc` module is the central module for computing statistics from a stream of records.
//!
//! The central component of this is a trait called `Accumulate` that implements a `new` function on initialization,
//! an `update` function to add a new record, and a `compute` function to compute the final value of the aggregation.
//! This trait requires two types, an input type (which is used by the `new` and `update` functions) and an `output` type.
//!
//! Internally, all of the structs implementing this trait are used in the main `aggregation` module
//! with the input type bounded by `FromStr` so the tool can convert from string records to the internal data types
//! that these aggregation types manipulate. And the output type is bounded by `Display` so the tool can write
//! the outputs to standard output.

use crate::parsing::DecimalWrapper;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::marker::PhantomData;

/// Accumulates records from a stream, in order to allow functions to be optimized for minimal memory usage.
pub trait Accumulate<I, O> {
    /// Creates a new object with an initial value (often based on the value of `item`.)
    ///
    /// This has a separate function for the initialization because some functions like sample standard deviation
    /// update differently than they initialize.
    fn new(item: I) -> Self;
    /// Adds a new value to the accumulator.
    fn update(&mut self, item: I);
    /// Computes the final value. Returns an option value, which is usually guaranteed to be Some(val)
    /// (with the exception of `StdDev`.)
    fn compute(&self) -> Option<O>;
}

/// The total number of records added to the accumulator.
pub struct Count<I>(usize, PhantomData<I>);

impl<I> Accumulate<I, usize> for Count<I> {
    fn new(_item: I) -> Count<I> {
        Count(1, PhantomData)
    }

    fn update(&mut self, _item: I) {
        self.0 += 1;
    }

    fn compute(&self) -> Option<usize> {
        Some(self.0)
    }
}

/// The total number of *unique* records.
pub struct CountUnique<I>(HashSet<I>);

impl<I> Accumulate<I, usize> for CountUnique<I>
where
    I: std::cmp::Eq,
    I: std::hash::Hash,
{
    fn new(item: I) -> CountUnique<I> {
        let mut vals = HashSet::new();
        vals.insert(item);
        CountUnique(vals)
    }

    fn update(&mut self, item: I) {
        self.0.insert(item);
    }

    fn compute(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

/// The largest value (or the value that would appear last in a sorted array)
pub struct Maximum<I>(I);

impl<I> Accumulate<I, I> for Maximum<I>
where
    I: std::cmp::PartialOrd,
    I: std::clone::Clone,
{
    fn new(item: I) -> Maximum<I> {
        Maximum(item)
    }

    fn update(&mut self, item: I) {
        if self.0 < item {
            self.0 = item;
        }
    }

    fn compute(&self) -> Option<I> {
        Some(self.0.clone())
    }
}

/// The mean. This is only implemented for `DecimalWrapper`, though it  could probably be extended for floating point
/// types.
pub struct Mean {
    running_sum: DecimalWrapper,
    running_count: usize,
}

impl Accumulate<DecimalWrapper, DecimalWrapper> for Mean {
    fn new(item: DecimalWrapper) -> Mean {
        Mean {
            running_sum: item,
            running_count: 1,
        }
    }

    fn update(&mut self, item: DecimalWrapper) {
        self.running_sum.item += item.item;
        self.running_count += 1;
    }

    fn compute(&self) -> Option<DecimalWrapper> {
        let decimal_count = Decimal::new(self.running_count as i64, 0);
        let result = self.running_sum.item / decimal_count;
        Some(DecimalWrapper { item: result })
    }
}

/// The median value. I've stored values in a `BTreeMap` in order to minimize memory usage.
/// As a result, this is the least performant of all the functions (running at `Nlog(m)`, rather than
/// the `N` of all the other algorithms (where `m` is the number of *unique* values in the accumulator).
pub struct Median {
    values: BTreeMap<DecimalWrapper, usize>,
    num: usize,
}

impl Accumulate<DecimalWrapper, DecimalWrapper> for Median {
    fn new(item: DecimalWrapper) -> Median {
        let mut mapping = BTreeMap::new();
        mapping.insert(item, 1);
        Median {
            values: mapping,
            num: 1,
        }
    }

    fn update(&mut self, item: DecimalWrapper) {
        self.values
            .entry(item)
            .and_modify(|val| *val += 1)
            .or_insert(1);
        self.num += 1;
    }

    fn compute(&self) -> Option<DecimalWrapper> {
        let mut cur_count = 0;
        let mut cur_val = DecimalWrapper {
            item: Decimal::new(0, 0),
        };
        // creating an iter bc we're stopping at N/2
        let mut iter = self.values.iter();
        while (cur_count as f64) < (self.num as f64 / 2.) {
            // should break before iter.next().is_none()
            let (result, count) = iter.next().unwrap();
            cur_count += count;
            cur_val = *result;
        }
        // -- take the mean if we have an even number of records and end at *exactly* the midpoint.
        if (self.num % 2) == 0
            && ((cur_count as f64) - (self.num as f64 / 2.)).abs() < std::f64::EPSILON
        {
            // iter.next() will always be Some(_) because this is always initialized with
            let median = (cur_val + *iter.next().unwrap().0)
                / DecimalWrapper {
                    item: Decimal::new(2, 0),
                };
            Some(median)
        } else {
            Some(cur_val)
        }
    }
}

/// The minimum value
pub struct Minimum<I>(I);

impl<I> Accumulate<I, I> for Minimum<I>
where
    I: std::cmp::PartialOrd,
    I: std::clone::Clone,
{
    fn new(item: I) -> Minimum<I> {
        Minimum(item)
    }

    fn update(&mut self, item: I) {
        if self.0 > item {
            self.0 = item;
        }
    }

    fn compute(&self) -> Option<I> {
        Some(self.0.clone())
    }
}

/// A combination of the minimum and maximum values, producing a string concatenating
/// the minimum value and the maximum value together, separated by a hyphen.
pub struct MinMax<I> {
    max_val: I,
    min_val: I,
}

impl<I> Accumulate<I, String> for MinMax<I>
where
    I: std::fmt::Display,
    I: std::cmp::PartialOrd,
    I: std::clone::Clone,
{
    fn new(item: I) -> MinMax<I> {
        MinMax {
            min_val: item.clone(),
            max_val: item,
        }
    }

    fn update(&mut self, item: I) {
        if self.min_val > item {
            self.min_val = item;
        } else if self.max_val < item {
            self.max_val = item;
        }
    }

    fn compute(&self) -> Option<String> {
        Some(format!("{} - {}", self.min_val, self.max_val))
    }
}

/// The most commonly appearing item.
///
/// If there is more than one mode, it returns
/// the item that reached the maximum value first. So in the case of
/// ["a", "b", "b", "a"], it will return "b" because "b" was the first
/// value to appear twice.
pub struct Mode<I> {
    histogram: HashMap<I, usize>,
    max_count: usize,
    max_val: I,
}

impl<I> Accumulate<I, I> for Mode<I>
where
    I: std::cmp::PartialOrd,
    I: std::cmp::Eq,
    I: std::hash::Hash,
    I: std::clone::Clone,
{
    fn new(item: I) -> Mode<I> {
        let mut histogram = HashMap::new();
        let max_val = item.clone();
        histogram.insert(item, 1);
        Mode {
            histogram,
            max_count: 1,
            max_val,
        }
    }

    fn update(&mut self, item: I) {
        // barely adapted from https://docs.rs/indexmap/1.0.2/indexmap/map/struct.IndexMap.html
        let new_count = *self.histogram.get(&item).unwrap_or(&0) + 1;
        if new_count > self.max_count {
            self.max_count = new_count;
            self.max_val = item.clone();
        }
        *self.histogram.entry(item).or_insert(0) += 1;
    }

    fn compute(&self) -> Option<I> {
        Some(self.max_val.clone())
    }
}

/// The range, or the difference between the minimum and maximum values (where the minimum value is subtracted from the maximum value).
pub struct Range<I, O> {
    max_val: I,
    min_val: I,
    phantom: PhantomData<O>,
}

impl<I, O> Accumulate<I, O> for Range<I, O>
where
    I: std::cmp::PartialOrd,
    I: std::ops::Sub<Output = O>,
    I: std::marker::Copy,
{
    #[allow(clippy::clone_on_copy)]
    fn new(item: I) -> Range<I, O> {
        Range {
            min_val: item,
            max_val: item.clone(),
            phantom: PhantomData,
        }
    }

    fn update(&mut self, item: I) {
        if self.min_val > item {
            self.min_val = item;
        }
        if self.max_val < item {
            self.max_val = item;
        }
    }

    fn compute(&self) -> Option<O> {
        Some(self.max_val - self.min_val)
    }
}

/// Computes the *sample* variance in a single pass, using
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

impl Accumulate<f64, f64> for StdDev {
    fn new(item: f64) -> Self {
        StdDev {
            q: 0.,
            m: item,
            num_records: 1.,
        }
    }

    fn update(&mut self, item: f64) {
        self.num_records += 1.;
        let squared_diff = (item - self.m).powi(2);
        self.q += ((self.num_records - 1.) * squared_diff) / self.num_records;
        self.m += (item - self.m) / self.num_records;
    }

    fn compute(&self) -> Option<f64> {
        if self.num_records <= 1. {
            return None;
        }
        Some((self.q / (self.num_records - 1.)).sqrt())
    }
}

/// The running sum of a stream of values.
pub struct Sum<I>(I);

impl<I> Accumulate<I, I> for Sum<I>
where
    I: std::ops::AddAssign,
    I: std::fmt::Display,
    I: std::marker::Copy,
{
    fn new(item: I) -> Sum<I> {
        Sum(item)
    }

    fn update(&mut self, item: I) {
        self.0 += item;
    }

    fn compute(&self) -> Option<I> {
        Some(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{self, CustomDateObject, DecimalWrapper};
    use proptest::prelude::*;
    use proptest::test_runner::Config;

    #[test]
    fn test_unique_count() {
        let update_vals = vec!["apple", "pie", "is", "good"]
            .into_iter()
            .map(|v| v.to_string());
        let mut no_dups = CountUnique::new("really".to_string());
        let mut dup = CountUnique::new("good".to_string());
        for val in update_vals {
            no_dups.update(val.clone());
            dup.update(val.clone());
        }
        assert_eq!(no_dups.compute().unwrap(), 5);
        assert_eq!(dup.compute().unwrap(), 4);
    }

    #[test]
    fn test_max_string() {
        let update_vals = vec!["2019-02-03", "2020-01-03", "2018-01-02"]
            .into_iter()
            .map(|v| v.to_string());
        let mut max_vals = Maximum::new("2019-12-31".to_string());
        for val in update_vals {
            max_vals.update(val);
        }
        assert_eq!(max_vals.compute(), Some("2020-01-03".to_string()));
    }

    #[test]
    fn test_max_dates() {
        // there's probably a better way of handling this, but this uses the same
        // code as parsing does, so shouldn't affect either
        let _ex = parsing::set_date_format("%Y-%m-%d %H:%M:%S".to_string());
        let date_updates = vec![
            "2019-02-03 12:23:10",
            "2020-01-03 13:45:02",
            "2018-01-02 12:23:10",
        ];
        let cust_date: CustomDateObject = "2019-12-31 01:20:13".parse().unwrap();
        let mut date_vals = Maximum::new(cust_date);
        for val in date_updates {
            let date_parse: CustomDateObject = val.parse().unwrap();
            date_vals.update(date_parse);
        }
        assert_eq!(
            date_vals.compute().unwrap().to_string(),
            "2020-01-03 13:45:02".to_string()
        );
    }

    #[test]
    fn test_max_decimals() {
        let updates = vec!["1.2", "2e-7", "2E3", "10000"];
        let start_dec: DecimalWrapper = ".278".parse().unwrap();
        let mut max_dec = Maximum::new(start_dec);
        for val in updates {
            max_dec.update(val.parse().unwrap());
        }
        assert_eq!(max_dec.compute().unwrap().to_string(), "10000".to_string());
    }

    #[test]
    fn test_min_string() {
        let update_vals = vec!["2019-02-03", "2020-01-03", "2018-01-02"]
            .into_iter()
            .map(|v| v.to_string());
        let mut max_vals = Minimum::new("2019-12-31".to_string());
        for val in update_vals {
            max_vals.update(val);
        }
        assert_eq!(max_vals.compute(), Some("2018-01-02".to_string()));
    }

    #[test]
    fn test_min_dates() {
        // there's probably a better way of handling this, but this uses the same
        // code as parsing does, so shouldn't affect either
        let _ex = parsing::set_date_format("%Y-%m-%d %H:%M:%S".to_string());
        let date_updates = vec![
            "2019-02-03 12:23:10",
            "2020-01-03 13:45:02",
            "2018-01-02 12:23:10",
        ];
        let cust_date: CustomDateObject = "2019-12-31 01:20:13".parse().unwrap();
        let mut date_vals = Minimum::new(cust_date);
        for val in date_updates {
            let date_parse: CustomDateObject = val.parse().unwrap();
            date_vals.update(date_parse);
        }
        assert_eq!(
            date_vals.compute().unwrap().to_string(),
            "2018-01-02 12:23:10".to_string()
        );
    }

    #[test]
    fn test_min_decimals() {
        let updates = vec!["1.2", "2e-7", "2E3", "10000"];
        let start_dec: DecimalWrapper = ".278".parse().unwrap();
        let mut max_dec = Minimum::new(start_dec);
        for val in updates {
            max_dec.update(val.parse().unwrap());
        }
        assert_eq!(
            max_dec.compute().unwrap().to_string(),
            "0.0000002".to_string()
        );
    }

    #[test]
    fn test_minmax_string() {
        let update_vals = vec!["2019-02-03", "2020-01-03", "2018-01-02"]
            .into_iter()
            .map(|v| v.to_string());
        let mut max_vals = MinMax::new("2019-12-31".to_string());
        for val in update_vals {
            max_vals.update(val);
        }
        assert_eq!(
            max_vals.compute(),
            Some("2018-01-02 - 2020-01-03".to_string())
        );
    }

    #[test]
    fn test_minmax_dates() {
        // there's probably a better way of handling this, but this uses the same
        // code as parsing does, so shouldn't affect either
        let _ex = parsing::set_date_format("%Y-%m-%d %H:%M:%S".to_string());
        let date_updates = vec![
            "2019-02-03 12:23:10",
            "2020-01-03 13:45:02",
            "2018-01-02 12:23:10",
        ];
        let cust_date: CustomDateObject = "2019-12-31 01:20:13".parse().unwrap();
        let mut date_vals = MinMax::new(cust_date);
        for val in date_updates {
            let date_parse: CustomDateObject = val.parse().unwrap();
            date_vals.update(date_parse);
        }
        assert_eq!(
            date_vals.compute().unwrap().to_string(),
            "2018-01-02 12:23:10 - 2020-01-03 13:45:02".to_string()
        );
    }

    #[test]
    fn test_minmax_decimals() {
        let updates = vec!["1.2", "2e-7", "2E3", "10000"];
        let start_dec: DecimalWrapper = ".278".parse().unwrap();
        let mut max_dec = MinMax::new(start_dec);
        for val in updates {
            max_dec.update(val.parse().unwrap());
        }
        assert_eq!(
            max_dec.compute().unwrap().to_string(),
            "0.0000002 - 10000".to_string()
        );
    }

    #[test]
    fn test_range_dates() {
        // there's probably a better way of handling this, but this uses the same
        // code as parsing does, so shouldn't affect either
        let _ex = parsing::set_date_format("%Y-%m-%d %H:%M:%S".to_string());
        let date_updates = vec![
            "2019-02-03 00:00:00",
            "2020-01-03 12:00:00",
            "2018-01-02 06:00:00",
        ];
        let cust_date: CustomDateObject = "2019-12-31 01:20:13".parse().unwrap();
        let mut date_vals = Range::new(cust_date);
        for val in date_updates {
            let date_parse: CustomDateObject = val.parse().unwrap();
            date_vals.update(date_parse);
        }
        assert_eq!(date_vals.compute().unwrap(), 731.25);
    }

    #[test]
    fn test_median() {
        let dec1: DecimalWrapper = "2".parse().unwrap();
        let mut dec_vals = Median::new(dec1);
        assert_eq!(dec_vals.compute().unwrap().to_string(), "2".to_string());
        let new_vals = vec!["3", "5"];
        for val in new_vals {
            let dec: DecimalWrapper = val.parse().unwrap();
            dec_vals.update(dec);
        }
        assert_eq!(dec_vals.compute().unwrap().to_string(), "3".to_string());
        let next_val: DecimalWrapper = "1".parse().unwrap();
        dec_vals.update(next_val);
        assert_eq!(dec_vals.compute().unwrap().to_string(), "2.5".to_string());
        let mult_middle_vals: DecimalWrapper = "3".parse().unwrap();
        let mut mult_median = Median::new(mult_middle_vals);
        for val in vec!["5", "6", "1", "4", "3"] {
            mult_median.update(val.parse().unwrap());
        }
        assert_eq!(
            mult_median.compute().unwrap().to_string(),
            "3.5".to_string()
        );
    }

    #[test]
    fn test_range_decimals() {
        let updates = vec!["1.2", "2E3", "10000"];
        let start_dec: DecimalWrapper = "19".parse().unwrap();
        let mut max_dec = Range::new(start_dec);
        for val in updates {
            max_dec.update(val.parse().unwrap());
        }
        assert_eq!(max_dec.compute().unwrap().to_string(), "9998.8".to_string());
    }

    #[test]
    fn test_mode() {
        let mut mode = Mode::new("a".to_string());
        assert_eq!(mode.compute().unwrap().to_string(), "a".to_string());
        let new_vals = vec!["b", "c", "a"].into_iter().map(|v| v.to_string());
        for val in new_vals {
            mode.update(val);
        }
        assert_eq!(mode.compute().unwrap(), "a".to_string());
        for _i in 1..=10000 {
            let mut mode_ordering = Mode::new("a".to_string());
            for val in vec!["b", "a", "b"].into_iter().map(|v| v.to_string()) {
                mode_ordering.update(val);
            }
            assert_eq!(mode.compute().unwrap(), "a".to_string());
        }
    }

    #[test]
    fn test_sum() {
        let dec_num: DecimalWrapper = "10".parse().unwrap();
        let mut summation = Sum::new(dec_num);
        assert_eq!(summation.compute().unwrap().to_string(), "10".to_string());
        let addl_vals = vec!["0.3", "100", "3.2"];
        for val in addl_vals {
            summation.update(val.parse().unwrap());
        }
        assert_eq!(
            summation.compute().unwrap().to_string(),
            "113.5".to_string()
        );
    }

    proptest! {
        #![proptest_config(Config::with_cases(100))]
        #[test]
        fn test_count_gets_raw_count(mut string_vecs in prop::collection::vec(any::<String>(), 1 .. 50)) {
            let total_count = string_vecs.len();
            let count_split = string_vecs.split_off(1);
            let mut count_obj = Count::new(string_vecs[0].clone());
            for item in count_split {
                count_obj.update(item);
            }
            assert_eq!(count_obj.compute().unwrap(), total_count);
        }
    }
}
