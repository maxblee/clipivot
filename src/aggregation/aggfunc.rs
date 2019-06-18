//! This module is designed for handling the aggregation functions.
//! I've heavily based its design off of [this useful](http://is.gd/N0N5ni)
//! playground that I found on
//! [Reddit](https://www.reddit.com/r/rust/comments/2rdoxx/enum_variants_as_types/).

#![feature(associated_types)]
/// An enum that I use to determine how to collect the records for aggregation.
/// It corresponds to the `aggfunc` parameter in the CLI.
/// Currently, it only supports counting.
///
/// However, I eventually want to support:
/// * sum
/// * mean
/// * standard deviation
/// * median
/// * minimum
/// * maximum
/// * mode
/// * unique records (the equivalent of `COUNT(DISTINCT column)` in `SQL`)
/// * non-null records (the equivalent of `COUNT(column) WHERE column IS NOT NULL`)
///
/// If you have any additional ideas of what aggregation methods I should support, let me know.
pub enum AggType {
    /// counts the total number of records
    Count,
}


#[derive(Debug, PartialEq)]
struct Count {
    val: Option<usize>,
}

pub trait AggregationMethod {
    type Aggfunc;

    fn aggtype(&self) -> AggType;
    fn new(parsed_val: String) -> Self;
    fn update(&mut self, parsed_val: String);
}

impl AggregationMethod for Count {
    type Aggfunc = Count;

    fn new(parsed_val: String) -> Self {
        Count {
            val: Some(1),
        }
    }

    fn update(&mut self, parsed_val: String) {
        self.val = match self.val {
            Some(cur_count) => Some(cur_count+1),
            None => None
        };
    }

    fn aggtype(&self) -> AggType { AggType::Count }

}

//    /// A method for updating the count
//    // TODO: Move to aggfunc.rs
//    fn add_count(&mut self, indexname: String, columnname: String) {
//        // from https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
//        self.aggregations.entry((indexname, columnname))
//            .and_modify(|val| {
//                let AggregateType::Count(cur_count) = *val;
//                *val = AggregateType::Count(cur_count + 1)
//            })
//            .or_insert(AggregateType::Count(1));
//    }