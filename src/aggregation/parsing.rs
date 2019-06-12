#[derive(Debug)]
pub enum ParsingType {
    StringType,
}

#[derive(Debug)]
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