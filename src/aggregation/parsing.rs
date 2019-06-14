#[derive(Debug, PartialEq)]
pub enum ParsingType {
    StringType,
}

#[derive(Debug, PartialEq)]
pub struct ParsingHelper {
    values_type: ParsingType,
    possible_values: Vec<ParsingType>
}

impl Default for ParsingHelper {
    fn default() -> ParsingHelper {
        ParsingHelper {
            values_type: ParsingType::StringType,
            possible_values: vec![ParsingType::StringType]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}