use std::collections::HashMap;

pub enum AggTypes {
    Count,
}

pub trait AggregationMethod {
    type Aggfunc;

    /// Returns the Aggregation method (e.g. AggTypes::Count)
    fn get_aggtype(&self) -> AggTypes;
    /// Instantiates a new Aggregation method
    fn new(parsed_val: String) -> Self;
    /// Updates an existing method
    fn update(&mut self, parsed_val: String);
}

struct Count {
    val: usize,
}

impl AggregationMethod for Count {
    type Aggfunc = Count;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Count }
    fn new(parsed_val: String) -> Self {
        Count { val: 1 }
    }
    fn update(&mut self, parsed_val: String) {
        self.val += 1;
    }
}

pub struct Aggregator<T>
where
    T: AggregationMethod,
{
    aggregations: HashMap<(String, String), T>,
}