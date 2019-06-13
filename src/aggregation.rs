use clap::ArgMatches;

mod errors;
use crate::aggregation::errors::CsvPivotError;

#[derive(Debug, PartialEq)]
pub struct CliConfig {
    filename: Option<String>,
    rows: Option<Vec<usize>>,
    columns: Option<Vec<usize>>,
    aggfunc: Option<String>,
    values: Option<usize>,
}

impl CliConfig {
    pub fn from_arg_matches(arg_matches : ArgMatches) -> Result<CliConfig, CsvPivotError> {
        let values : usize = arg_matches.value_of("value").unwrap().parse()?;
        let rows = CliConfig::parse_column(arg_matches
            .values_of("rows").unwrap().collect())?;
        let columns = CliConfig::parse_column(arg_matches
            .values_of("columns").unwrap().collect())?;
        let filename = arg_matches.value_of("filename").map(String::from);
        let aggfunc = arg_matches.value_of("aggfunc").map(String::from);
        let cfg = CliConfig{
            filename,
            rows: Some(rows),
            columns: Some(columns),
            aggfunc,
            values: Some(values),
        };
        Ok(cfg)
    }
    pub fn parse_column(column: Vec<&str>) -> Result<Vec<usize>, CsvPivotError> {
        let mut indexes = Vec::new();
        for idx in column {
            let index_val = idx.parse()?;
            indexes.push(index_val);
        }
        Ok(indexes)
    }
}

pub fn run(arg_matches : ArgMatches) -> Result<(), CsvPivotError> {
    let config = CliConfig::from_arg_matches(arg_matches)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_matches_yield_proper_config() {
        /// Makes sure the CliConfig::from_arg_matches impl works properly
        // Note: I eventually want this to come from a setup func, but have to deal with
        // lifetimes for that :(
        let yaml = load_yaml!("cli.yml");
        let matches = clap::App::from_yaml(yaml)
            .version(crate_version!())
            .author(crate_authors!())
            .get_matches_from(vec!["csvpivot", "count", "tmp/layoffs.csv", "--rows=3", "--cols=1", "--val=0"]);
        let expected_config = CliConfig {
            filename: Some("tmp/layoffs.csv".to_string()),
            rows: Some(vec![3]),
            columns: Some(vec![1]),
            values: Some(0),
            aggfunc: Some("count".to_string())
        };
        assert_eq!(CliConfig::from_arg_matches(matches).unwrap(), expected_config)
    }
}