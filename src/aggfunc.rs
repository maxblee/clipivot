use crate::parsing::ParsingType;

#[derive(Debug, PartialEq)]
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
    fn to_output(&self) -> String;
}

pub struct Count {
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