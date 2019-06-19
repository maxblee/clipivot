use crate::errors::CsvPivotError;

#[derive(Debug, PartialEq)]
pub enum ParsingType {
    Text(Option<String>)
}

#[derive(Debug, PartialEq)]
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
    pub fn parse_val(&self, new_val: &str) -> Result<ParsingType, CsvPivotError> {
        match self.values_type {
            ParsingType::Text(_) => Ok(ParsingType::Text(Some(new_val.to_string()))),
        }
    }
}