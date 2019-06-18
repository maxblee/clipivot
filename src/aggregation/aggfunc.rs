use std::collections::HashMap;

pub enum AggTypes {
    Count,
}

pub trait AggregationMethod {
    type Aggfunc;

    /// Returns the Aggregation method (e.g. AggTypes::Count)
    fn get_aggtype(&self) -> AggTypes;
    /// Instantiates a new Aggregation method
    fn new(parsed_val: &str) -> Self;
    /// Updates an existing method
    fn update(&mut self, parsed_val: &str);
}

struct Count {
    val: usize,
}

impl AggregationMethod for Count {
    type Aggfunc = Count;

    fn get_aggtype(&self) -> AggTypes { AggTypes::Count }
    fn new(parsed_val: &str) -> Self {
        Count { val: 1 }
    }
    fn update(&mut self, parsed_val: &str) {
        self.val += 1;
    }
}

pub struct Aggregator<T>
where
    T: AggregationMethod,
{
    aggregations: HashMap<(String, String), T>,
}

impl <T: AggregationMethod> Aggregator<T> {
    fn update_aggregations(&mut self, indexname: String, columnname: String, parsed_val: &str) {
        // modified from
        // https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| val.update(parsed_val))
            .or_insert(T::new(parsed_val));
    }
}