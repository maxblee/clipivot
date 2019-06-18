use std::collections::HashMap;

pub enum AggTypes {
    Count,
}

pub trait AggregationMethod {
    type Aggfunc;

}

struct Count {
    val: usize,
}

impl AggregationMethod for Count {
    type Aggfunc = Count;
}

pub struct Aggregator<T>
where
    T: AggregationMethod,
{
    aggregations: HashMap<(String, String), T>,
}