use std::collections::HashMap;

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
    fn parse_val(&self, new_val: String) -> ParsingType {
        match self.values_type {
            ParsingType::Text(_) => ParsingType::Text(Some(new_val)),
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
}

impl <T: AggregationMethod> Aggregator<T> {
    fn update_aggregations(&mut self, indexname: String, columnname: String, parsed_val: &ParsingType) {
        // modified from
        // https://users.rust-lang.org/t/efficient-string-hashmaps-for-a-frequency-count/7752
        self.aggregations.entry((indexname, columnname))
            .and_modify(|val| val.update(parsed_val))
            .or_insert(T::new(parsed_val));
    }
}